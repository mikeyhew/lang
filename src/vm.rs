use {
    crate::{
        ast::{
            Expr,
            ExprKind,
            Stmt,
            StmtKind,
            Name,
            Number,
        },
        typeck::Type,
        util::{
            Map,
            join,
            mapping,
        }
    },
    custom_debug_derive::CustomDebug,
    derive_more::Display,
};

pub use crate::context::ValueContext;

#[derive(Debug, Display, Clone)]
#[display(fmt = "VmError: {}", message)]
pub struct VmError {
    message: String,
}

impl VmError {
    pub fn new(message: String) -> Self {
        Self {message}
    }
}

pub type VmResult<T> = Result<T, VmError>;

macro_rules! type_error {
    ($($tt:tt)*) => {
        return Err(VmError::new(format!($($tt)*)))
    };
}

#[derive(Display, Clone)]
#[derive(CustomDebug)]
pub enum Value {
    #[display(fmt = "nil")]
    Nil,
    #[display(fmt = "{{{}}}", r#"join(", ", _0.iter().map(mapping("=")))"#)]
    Record(Map<Name, Value>),
    #[display(fmt = "({})", r#"join(", ", _0.iter())"#)]
    Tuple(Vec<Value>),
    #[display(fmt = "[Function]")]
    Closure(Name, Box<Expr>, #[debug(skip)] ValueContext),
    #[display(fmt = "[Builtin]")]
    BuiltinFunc(#[debug(skip)] fn(&Value) -> VmResult<Value>),
    #[display(fmt = "{}", _0)]
    Number(Number),
    #[display(fmt = "{:?}", _0)]
    String_(String),
    #[display(fmt = "{}", _0)]
    Type(Type),
}

impl Value {
    fn access_record_field(&self, name: &Name) -> Result<Value, VmError> {
        match self {
            Value::Record(map) => {
                if let Some(value) = map.get(name) {
                    Ok(value.clone())
                } else {
                    type_error!("record {} doesn't have a field named {}", self, name)
                }
            }
            _ => type_error!("expected record with field `{}`, found {}", name, self)
        }
    }

    fn access_tuple_field(&self, number: usize) -> Result<Value, VmError> {
        match self {
            Value::Tuple(values) => {
                let number: usize = number.into();

                if let Some(value) = values.get(number) {
                    Ok(value.clone())
                } else {
                    type_error!(
                        "expected tuple with at least {} elements, found one with only {}: {}",
                        number + 1, values.len(), self
                    )
                }
            }
            _ => type_error!(
                "expected tuple with at least {} elements, found {}",
                number + 1 , self
            )
        }
    }
}

pub fn evaluate_type(expr: &Expr, context: &ValueContext) -> Result<Type, VmError> {
    let value = evaluate(expr, context)?;

    Ok(match value {
        Value::Type(ty) => ty,
        _ => type_error!("expected a type, found {}", value),
    })
}

pub fn evaluate_stmt(stmt: &Stmt, context: &ValueContext) -> Result<ValueContext, VmError> {
    match &stmt.kind {
        StmtKind::Let(ident, expr) => {
            let value = evaluate(expr, &context)?;
            Ok(context.extend(ident.name.clone(), value))
        }
    }
}

pub fn evaluate(expr: &Expr, context: &ValueContext) -> Result<Value, VmError> {
    Ok(match &expr.kind {
        ExprKind::Nil => Value::Nil,
        ExprKind::NilType => Value::Type(Type::Nil),

        ExprKind::RecordValue(entries) => {
            let map = entries.iter().try_fold(Map::default(), |mut map, (ident, expr)| {
                map.insert(ident.name.clone(), evaluate(expr, context)?);
                Ok(map)
            })?;

            Value::Record(map)
        }
        ExprKind::RecordType(entries) => {
            let map = entries.iter().try_fold(Map::default(), |mut map, (ident, expr)| {
                map.insert(ident.name.clone(), evaluate_type(expr, context)?);
                Ok(map)
            })?;

            Value::Type(Type::Record(map))
        }
        ExprKind::RecordFieldAccess(ref expr, ref field_name) => {
            evaluate(expr, context)?.access_record_field(&field_name.name)?
        }
        ExprKind::Tuple(exprs) => {
            let values = exprs.iter().try_fold(Vec::new(), |mut values, expr| {
                values.push(evaluate(expr, context)?);
                Ok(values)
            })?;

            Value::Tuple(values)
        }
        ExprKind::TupleType(exprs) => {
            let values = exprs.iter().try_fold(Vec::new(), |mut values, expr| {
                values.push(evaluate_type(expr, context)?);
                Ok(values)
            })?;

            Value::Type(Type::Tuple(values))
        }
        ExprKind::TupleFieldAccess(ref expr, ref field_number) => {
            evaluate(expr, context)?.access_tuple_field(*field_number)?
        }
        ExprKind::Block(stmts, expr) => {
            let context = stmts.iter()
                .try_fold(context.clone(), |context, stmt| evaluate_stmt(stmt, &context))?;

            match expr {
                Some(expr) => evaluate(expr, &context)?,
                None => Value::Nil,
            }
        }
        ExprKind::Var(ident) => {
            match context.lookup(&ident.name) {
                Some(value) => value.clone(),
                None => type_error!("Unknown variable {}", ident.name),
            }
        },
        ExprKind::Closure(ident, _, body) => {
            Value::Closure(ident.name.clone(), body.clone(), context.clone())
        }
        ExprKind::Call(callee, arg) => {
            let callee = evaluate(callee, context)?;
            let arg = evaluate(arg, context)?;

            match callee {
                Value::Closure(arg_name, body, context) => {
                    let context = context.extend(arg_name.clone(), arg);
                    evaluate(&*body, &context)?
                }
                Value::BuiltinFunc(func) => {
                    func(&arg)?
                }
                _ => type_error!("expected a function, found {}", callee),
            }
        },
        ExprKind::NumberLiteral(number) => Value::Number(*number),
        ExprKind::StringLiteral(s) => Value::String_(s.clone()),
        ExprKind::Parenthesized(ref expr) => evaluate(expr, context)?,
    })
}
