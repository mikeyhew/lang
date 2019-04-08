use {
    crate::{
        eval::{EvalError, EvalResult},
        value::{Value, Type}
    },
};
#[macro_use]
use crate::eval;

pub trait BuiltinFunc: Send + Sync {
    fn call(&self, arg: &Value) -> EvalResult<Value>;
    fn clone(&self) -> Box<dyn BuiltinFunc>;
}

impl<F> BuiltinFunc for F
where
    F: Fn(&Value) -> EvalResult<Value>,
    F: Clone + Send + Sync + 'static,
{
    fn call(&self, arg: &Value) -> EvalResult<Value> {
        self(arg)
    }
    fn clone(&self) -> Box<dyn BuiltinFunc> {
        Box::new(<F as Clone>::clone(self))
    }
}

impl Clone for Box<dyn BuiltinFunc> {
    fn clone(&self) -> Self {
        BuiltinFunc::clone(&**self)
    }
}

pub fn builtin_func<F>(f: F) -> Value
where
    F: Fn(&Value) -> EvalResult<Value>,
    F: Clone + Send + Sync + 'static,
{
    Value::BuiltinFunc(Box::new(f))
}

pub fn builtins() -> Vec<(&'static str, Type, Value)> {
    use Value::*;
    vec![
        ("Type", Type(TypeType), TypeType),
        ("Number", Type(TypeType), NumberType),
        ("String", Type(TypeType), StringType),
        (
            "Fn",
            Type(Value::FuncType(
                Box::new(Type(TypeType)),
                Box::new(Type(Value::FuncType(
                    Box::new(Type(TypeType)),
                    Box::new(Type(TypeType)),
                ))),
            )),
            builtin_func(|input_ty| {
                if let Some(input_ty) = input_ty.as_type() {
                    Ok(builtin_func(move |output_ty| {
                        if let Some(output_ty) = output_ty.as_type() {
                            Ok(Value::FuncType(
                                Box::new(input_ty.clone()),
                                Box::new(output_ty.clone()),
                            ))
                        } else {
                            Err(EvalError(format!(
                                "expected a type, found {}", input_ty
                            )))
                        }
                    }))
                } else {
                    Err(EvalError(format!(
                        "expected a type, found {}", input_ty
                    )))
                }
            })
        ),
    ]
}
