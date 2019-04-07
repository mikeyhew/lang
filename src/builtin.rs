use {
    crate::{
        vm::{Value, VmError, VmResult},
        typeck::Type,
    },
};

pub trait BuiltinFunc: Send + Sync {
    fn call(&self, arg: &Value) -> VmResult<Value>;
    fn clone(&self) -> Box<dyn BuiltinFunc>;
}

impl<F> BuiltinFunc for F
where
    F: Fn(&Value) -> VmResult<Value>,
    F: Clone + Send + Sync + 'static,
{
    fn call(&self, arg: &Value) -> VmResult<Value> {
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
    F: Fn(&Value) -> VmResult<Value>,
    F: Clone + Send + Sync + 'static,
{
    Value::BuiltinFunc(Box::new(f))
}

pub fn builtins() -> Vec<(&'static str, Type, Value)> {
    vec![
        ("Type", Type::Type, Value::Type(Type::Type)),
        ("Number", Type::Type, Value::Type(Type::Number)),
        ("String", Type::Type, Value::Type(Type::String_)),
        (
            "Fn",
            Type::Func(
                Box::new(Type::Type),
                Box::new(Type::Func(
                    Box::new(Type::Type),
                    Box::new(Type::Type),
                )),
            ),
            builtin_func(|input_ty| {
                if let Value::Type(input_ty) = input_ty {
                    let input_ty = input_ty.clone();
                    Ok(builtin_func(move |output_ty| {
                        if let Value::Type(output_ty) = output_ty {
                            Ok(Value::Type(Type::Func(
                                Box::new(input_ty.clone()),
                                Box::new(output_ty.clone()),
                            )))
                        } else {
                            Err(VmError::new(format!(
                                "expected a type, found {}", input_ty
                            )))
                        }
                    }))
                } else {
                    Err(VmError::new(format!(
                        "expected a type, found {}", input_ty
                    )))
                }
            })
        ),
    ]
}
