use {
    crate::{
        ast::{Name},
        builtin::builtins,
        typeck::Type,
        vm::Value,
    },
};

#[derive(Debug, Clone)]
struct TypeEntry {
    name: Name,
    ty: Type,
    value: Value,
}

#[derive(Debug, Clone)]
pub struct TypeContext {
    entries: Vec<TypeEntry>,
}

impl Default for TypeContext {
    fn default() -> Self {
        let entries = builtins().into_iter()
            .map(|(name, ty, value)| {
                TypeEntry {name: name.into(), ty, value}
            }).collect();

        Self {entries}
    }
}

impl TypeContext {
    pub fn extend(&self, name: Name, ty: Type, value: Value) -> Self {
        let mut entries = self.entries.clone();
        entries.push(TypeEntry {name, ty, value});
        Self {entries}
    }

    pub fn lookup(&self, name: &Name) -> Option<Type> {
        self.entries.iter().rev()
            .find(|entry| entry.name == *name)
            .map(|entry| entry.ty.clone())
    }

    pub fn as_value_context(&self) -> ValueContext {
        let entries = self.entries.iter()
            .map(|TypeEntry {name, value, ..}| {
                ValueEntry {name: name.clone(), value: value.clone()}
            }).collect();

        ValueContext {entries}
    }
}

#[derive(Debug, Clone)]
struct ValueEntry {
    name: Name,
    value: Value,
}

#[derive(Debug, Clone)]
pub struct ValueContext {
    entries: Vec<ValueEntry>,
}

impl Default for ValueContext {
    fn default() -> Self {
        let entries = builtins().into_iter()
            .map(|(name, _, value)| {
                ValueEntry {name: name.into(), value}
            }).collect();

        Self {entries}
    }
}

impl ValueContext {
    pub fn extend(&self, name: Name, value: Value) -> Self {
        let mut entries = self.entries.clone();
        entries.push(ValueEntry {name, value});
        Self {entries}
    }

    pub fn lookup(&self, name: &Name) -> Option<Value> {
        self.entries.iter().rev()
            .find(|entry| entry.name == *name)
            .map(|entry| entry.value.clone())
    }
}

