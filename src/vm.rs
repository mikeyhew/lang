pub use crate::ast::{
    Expr,
    Ident,
    Spanned,
};
use derive_more::{
    Display,
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
    Record(Map<Ident, Value>),
    Tuple(Vec<Value>),
    Number(isize),
    String_(String),
}

impl Value {
    fn access_record_field(&self, name: &Ident) -> Result<Value, TypeError> {
        match self {
            Value::Record(map) => {
                if let Some(value) = map.get(name) {
                    Ok(value.clone())
                } else {
                    type_error!("Record {:?} doesn't have a field named {}", self, name.as_ref())
                }
            }
            _ => type_error!("expected Record, found {:?}", self)
        }
    }

    fn access_tuple_field(&self, number: usize) -> Result<Value, TypeError> {
        match self {
            Value::Tuple(values) => {
                let number: usize = number.into();

                if let Some(value) = values.get(number) {
                    Ok(value.clone())
                } else {
                    type_error!("Tuple {:?} doesn't have a field at {}", self, number)
                }
            }
            _ => type_error!("expected Tuple, found {:?}", self)
        }
    }
}

pub struct Context {
    map: Map<Ident, Value>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            map: Map::default(),
        }
    }

    pub fn extend(&self, name: Ident, value: Value) -> Self {
        let mut map = self.map.clone();
        map.insert(name, value);
        Self {map}
    }

    pub fn get(&self, name: &Ident) -> Option<&Value> {
        self.map.get(name)
    }
}

pub fn evaluate(expr: &Spanned<Expr>, context: &Context) -> Result<Value, TypeError> {
    Ok(match &expr.value {
        Expr::EmptyRecord => Value::Nil,
        Expr::RecordValue(entries) => {
            let mut map = Map::default();

            for (name, expr) in entries.iter() {
                map.insert(name.value.clone(), evaluate(expr, context)?);
            }

            Value::Record(map)
        }
        Expr::RecordType(..) => unimplemented!("RecordType"),
        Expr::Tuple(exprs) => {
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
        Expr::Block(..) => unimplemented!("Block"),
        Expr::Let(name, value, body) => {
            let value = evaluate(value, context)?;
            let context = context.extend(name.value.clone(), value);
            evaluate(body, &context)?
        }
        Expr::Var(name) => {
            match context.get(&name) {
                Some(value) => value.clone(),
                None => type_error!("Unknown variable {}", name),
            }
        },
        Expr::RecordFieldAccess(ref expr, ref field_name) => {
            evaluate(expr, context)?.access_record_field(field_name)?
        }
        Expr::TupleFieldAccess(ref expr, field_number) => {
            evaluate(expr, context)?.access_tuple_field(*field_number)?
        }
        Expr::NumberLiteral(number) => Value::Number(*number),
        Expr::StringLiteral(s) => Value::String_(s.clone()),
    })
}
