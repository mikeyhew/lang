pub use crate::ast::{
    Expr,
    ExprKind,
    Ident,
    Name,
    Number,
    Span,
};
use derive_more::{
    Display,
};
use std::{
    fmt,
};

pub type Map<K, V> = fnv::FnvHashMap<K, V>;

#[derive(Debug, Display, Clone)]
#[display(fmt = "TypeError: {}", _0)]
pub struct TypeError(String);

macro_rules! type_error {
    ($($tt:tt)*) => {
        return Err(TypeError(format!($($tt)*)))
    };
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    Nil,
    Record(Map<Name, Value>),
    Tuple(Vec<Value>),
    Number(Number),
    String_(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Record(map) => {
                write!(f, "{{")?;

                let mut first = true;

                for (name, value) in map.iter() {
                    if !first {
                        write!(f, ", ")?;
                    } else {
                        first = false;
                    }

                    write!(f, "{}={}", name, value)?;
                }

                write!(f, "}}")
            }
            Value::Tuple(vec) => {
                write!(f, "(")?;

                let mut first = true;

                for value in vec.iter() {
                    if !first {
                        write!(f, ", ")?;
                    } else {
                        first = false;
                    }

                    write!(f, "{}", value)?;
                }

                write!(f, ")")
            }
            Value::Number(n) => write!(f, "{}", n),
            Value::String_(s) => write!(f, "{:?}", s),
        }
    }
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

pub struct Context {
    map: Map<Name, Value>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            map: Map::default(),
        }
    }

    pub fn extend(&self, name: Name, value: Value) -> Self {
        let mut map = self.map.clone();
        map.insert(name, value);
        Self {map}
    }

    pub fn get(&self, name: &Name) -> Option<&Value> {
        self.map.get(name)
    }
}

pub fn evaluate(expr: &Expr, context: &Context) -> Result<Value, TypeError> {
    Ok(match &expr.kind {
        ExprKind::EmptyRecord => Value::Nil,
        ExprKind::RecordValue(entries) => {
            let mut map = Map::default();

            for (ident, expr) in entries.iter() {
                map.insert(ident.name.clone(), evaluate(expr, context)?);
            }

            Value::Record(map)
        }
        ExprKind::RecordType(..) => unimplemented!("RecordType"),
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
        ExprKind::Block(..) => unimplemented!("Block"),
        ExprKind::Let(ref ident, ref value, ref body) => {
            let value = evaluate(value, context)?;
            let context = context.extend(ident.name.clone(), value);
            evaluate(body, &context)?
        }
        ExprKind::Var(ident) => {
            match context.get(&ident.name) {
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
