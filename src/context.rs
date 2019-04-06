use {
    crate::{
        ast::Name,
        builtin::builtin_func,
        util::Map,
        vm::{Value, VmError},
        typeck::Type,
    },
    lazy_static::lazy_static,
};

#[derive(Clone)]
pub struct Context<Value> {
    map: Map<Name, Value>,
}

impl<Value> Context<Value> {
    pub fn new() -> Self
    where Self: Default {
        Default::default()
    }

    pub fn extend(&self, name: Name, value: Value) -> Self
    where
        Value: Clone,
    {
        let mut map = self.map.clone();
        map.insert(name, value);
        Self {map}
    }

    pub fn lookup(&self, name: &Name) -> Option<&Value> {
        self.map.get(name)
    }
}

pub type ValueContext = Context<Value>;

impl Default for ValueContext {
    fn default() -> Self {
        let (_, values) = &*DEFAULT_CONTEXTS;
        Context {map: values.clone()}
    }
}

pub type TypeContext = Context<Type>;

impl Default for TypeContext {
    fn default() -> Self {
        let (types, _) = &*DEFAULT_CONTEXTS;
        Context {map: types.clone()}
    }
}

lazy_static! {
    static ref DEFAULT_CONTEXTS: (Map<Name, Type>, Map<Name, Value>) = {
        let initial_values: Vec<(&'static str, Type, Value)> = vec![
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
            )
        ];

        let mut types = Map::default();
        let mut values = Map::default();

        for (name, ty, value) in initial_values {
            types.insert(name.into(), ty);
            values.insert(name.into(), value);
        }

        (types, values)
    };
}
