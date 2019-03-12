use {
    crate::{
        ast::Name,
        util::Map,
        vm::Value,
        typeck::Type,
    },
    lazy_static::lazy_static,
};

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
            ("nil", Type::Nil, Value::Nil),
            ("Nil", Type::Type, Value::Type(Type::Nil)),
            ("Type", Type::Type, Value::Type(Type::Type)),
            ("Number", Type::Type, Value::Type(Type::Number)),
            ("String", Type::Type, Value::Type(Type::String_)),
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
