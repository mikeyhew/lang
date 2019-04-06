use {
    crate::{
        vm::{Value, VmResult},
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
