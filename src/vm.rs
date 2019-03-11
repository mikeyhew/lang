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
    fmt::{self, Display},
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
    // #[display(fmt = "nil")]
    Nil,
    // #[display(fmt = "Nil")]
    NilType,
    // #[display(fmt = "{{{}}}", join(", ", _0.iter().map(|(k, v)| mapping("=", k, v))))]
    Record(Map<Name, Value>),
    // #[display(fmt = "{{{}}}", join(", ", _0.iter().map(|(k, v)| mapping(": ", k, v))))]
    RecordType(Map<Name, Value>),
    // #[display(fmt = "({})", join(", ", _0.iter()))]
    Tuple(Vec<Value>),
    // #[display(fmt = "type ({})", join(", ", _0.iter()))]
    TupleType(Vec<Value>),
    // #[display(fmt = "{}", _0)]
    Number(Number),
    // #[display(fmt = "Number")]
    NumberType,
    // #[display(fmt = "{:?}", _0)]
    String_(String),
    // #[display(fmt = "String")]
    StringType,
}

fn join<I: Iterator<Item=impl Display> + Clone>(
    joiner: &'static str,
    it: I,
) -> impl Display {
    struct Join<I>(&'static str, I);

    impl<I: Iterator<Item=impl Display> + Clone> Display for Join<I> {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Join(joiner, it) = self;
            let mut it: I = it.clone();

            match it.next() {
                Some(d) => write!(fmt, "{}", d)?,
                None => return Ok(()),
            }

            for d in it {
                write!(fmt, "{}{}", joiner, d)?;
            }

            Ok(())
        }
    }

    Join(joiner, it)
}

struct Mapping<K, V> {
    between: &'static str,
    key: K,
    value: V,
}

impl<K: Display, V: Display> Display for Mapping<K, V> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}{}{}", self.key, self.between, self.value)
    }
}

fn mapping<K: Display, V: Display>(
    between: &'static str,
) -> impl FnMut((K, V)) -> Mapping<K, V> + Clone {
    move |(key, value)| Mapping {between, key, value}
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::NilType => write!(f, "Nil"),
            Value::Record(map) => {
                write!(f, "{}", join(", ", map.iter().map(mapping("="))))
            }
            Value::RecordType(map) => {
                write!(f, "{{{}}}", join(", ", map.iter().map(mapping(": "))))
            }
            Value::Tuple(vec) => {
                write!(f, "({})", join(", ", vec.iter()))
            }
            Value::TupleType(vec) => {
                write!(f, "type ({})", join(", ", vec.iter()))
            }
            Value::Number(n) => write!(f, "{}", n),
            Value::NumberType => write!(f, "Number"),
            Value::String_(s) => write!(f, "{:?}", s),
            Value::StringType => write!(f, "String"),
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
