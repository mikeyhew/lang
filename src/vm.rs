use {
    crate::{
        ast::{
            Expr,
            ExprKind,
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
    derive_more::Display,
};

pub use crate::context::ValueContext;

#[derive(Debug, Display, Clone)]
#[display(fmt = "TypeError: {}", _0)]
pub struct TypeError(String);

macro_rules! type_error {
    ($($tt:tt)*) => {
        return Err(TypeError(format!($($tt)*)))
    };
}

#[derive(Debug, Display, Clone, Eq, PartialEq)]
pub enum Value {
    #[display(fmt = "nil")]
    Nil,
    #[display(fmt = "{{{}}}", r#"join(", ", _0.iter().map(mapping("=")))"#)]
    Record(Map<Name, Value>),
    #[display(fmt = "type ({})", r#"join(", ", _0.iter())"#)]
    Tuple(Vec<Value>),
    #[display(fmt = "{}", _0)]
    Number(Number),
    #[display(fmt = "{}", _0)]
    String_(String),
    #[display(fmt = "{}", _0)]
    Type(Type),
}

impl Value {
    fn access_record_field(&self, name: &Name) -> Result<Value, TypeError> {
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

    fn access_tuple_field(&self, number: usize) -> Result<Value, TypeError> {
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

pub fn evaluate_type(expr: &Expr, context: &ValueContext) -> Result<Type, TypeError> {
    let value = evaluate(expr, context)?;

    Ok(match value {
        Value::Type(ty) => ty,
        _ => type_error!("expected a type, found {}", value),
    })
}

pub fn evaluate(expr: &Expr, context: &ValueContext) -> Result<Value, TypeError> {
    Ok(match &expr.kind {
        ExprKind::EmptyRecord | ExprKind::EmptyTuple => Value::Nil,
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
        ExprKind::Tuple(exprs) => {
            match exprs.len() {
                0 => Value::Nil,
                1 => evaluate(&exprs[0], context)?,
                _ => {
                    let mut values = Vec::new();

                    for expr in exprs.iter() {
                        values.push(evaluate(expr, context)?);
                    }

                    Value::Tuple(values)
                }
            }
        }
        ExprKind::TupleType(types) => unimplemented!("TupleType"),
        ExprKind::Block(..) => unimplemented!("Block"),
        ExprKind::Let(ref ident, ref value, ref body) => {
            let value = evaluate(value, context)?;
            let context = context.extend(ident.name.clone(), value);
            evaluate(body, &context)?
        }
        ExprKind::Var(ident) => {
            match context.lookup(&ident.name) {
                Some(value) => value.clone(),
                None => type_error!("Unknown variable {}", ident.name),
            }
        },
        ExprKind::RecordFieldAccess(ref expr, ref field_name) => {
            evaluate(expr, context)?.access_record_field(&field_name.name)?
        }
        ExprKind::TupleFieldAccess(ref expr, ref field_number) => {
            evaluate(expr, context)?.access_tuple_field(*field_number)?
        }
        ExprKind::NumberLiteral(number) => Value::Number(*number),
        ExprKind::StringLiteral(s) => Value::String_(s.clone()),
        ExprKind::Parenthesized(ref expr) => evaluate(expr, context)?,


    })
}
