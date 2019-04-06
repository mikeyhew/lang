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
        builtin::BuiltinFunc,
        typeck::Type,
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

macro_rules! vm_error {
    ($($tt:tt)*) => {
        return Err(VmError::new(format!($($tt)*)))
    };
}

#[derive(Display, Clone)]
#[derive(CustomDebug)]
pub enum Value {
    #[display(fmt = "nil")]
    Nil,
    #[display(fmt = "[Function]")]
    Closure(Name, Box<Expr>, #[debug(skip)] ValueContext),
    #[display(fmt = "[Builtin]")]
    BuiltinFunc(#[debug(skip)] Box<dyn BuiltinFunc>),
    #[display(fmt = "{}", _0)]
    Number(Number),
    #[display(fmt = "{:?}", _0)]
    String_(String),
    #[display(fmt = "{}", _0)]
    Type(Type),
}

pub fn evaluate_type(expr: &Expr, context: &ValueContext) -> Result<Type, VmError> {
    let value = evaluate(expr, context)?;

    Ok(match value {
        Value::Type(ty) => ty,
        _ => vm_error!("expected a type, found {}", value),
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
                None => vm_error!("Unknown variable {}", ident.name),
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
                    func.call(&arg)?
                }
                _ => vm_error!("expected a function, found {}", callee),
            }
        },
        ExprKind::NumberLiteral(number) => Value::Number(*number),
        ExprKind::StringLiteral(s) => Value::String_(s.clone()),
        ExprKind::Parenthesized(ref expr) => evaluate(expr, context)?,
    })
}
