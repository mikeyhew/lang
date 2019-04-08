use crate::{
    ast::Name,
    builtin::builtins,
    value::{Value, Type, ParamDepth},
};

#[derive(Debug, Clone)]
enum Entry {
    Value(Name, Type, Value),
    Param(Name, Type),
}

#[derive(Debug, Clone)]
pub struct Context {
    entries: Vec<Entry>,
}

impl Default for Context {
    fn default() -> Self {
        let entries = builtins().into_iter()
            .map(|(name, ty, value)| Entry::Value(name.into(), ty, value))
            .collect();

        Context {entries}
    }
}

impl Context {
    fn lookup_name(&self, name: &Name) -> Option<(Type, Value)> {
        self.entries.iter().rev().enumerate()
            .filter_map(|(i, entry)| match entry {
                Entry::Value(name2, ty, value) if name2 == name => {
                    Some((ty.clone(), value.clone()))
                }
                Entry::Param(name2, ty) if name2 == name => {
                    Some((ty.clone(), Value::Param(name.clone(), ParamDepth(i as u32))))
                }
                _ => None,
            }).next()
    }
}
