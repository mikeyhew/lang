use {
    crate::{
        ast::{Name, Number, Expr},
        builtin::BuiltinFunc,
        context::Context,
    },
    std::{
        fmt::{self, Display},
    },
    derive_more::{
        Display, From, Into,
    },
    custom_debug_derive::CustomDebug,
};

#[derive(CustomDebug, Clone, Display)]
pub enum Value {
    Nil,
    NilType,

    StringLiteral(String),
    StringType,

    NumberLiteral(Number),
    NumberType,

    #[display(fmt = "[Closure]")]
    Closure {
        arg_name: Name,
        arg_type: Box<Type>,
        body: Expr,
        #[debug(skip)]
        context: Context,
    },
    #[display(fmt = "[BuiltinFunc]")]
    BuiltinFunc(#[debug(skip)] Box<dyn BuiltinFunc>),
    #[display(fmt = "Fn({})({})", _0, _1)]
    FuncType(Box<Type>, Box<Type>),

    #[display(fmt = "{}", _0)]
    Param(Name, ParamDepth),
    #[display(fmt = "({})({})", _0, _1)]
    Call(Box<Value>, Box<Value>),

    TypeType,

    Error,
}

fn display_callee<'a>(callee: &'a Value) -> impl Display + 'a {
    struct Callee<'a>(&'a Value);
    impl Display for Callee<'_> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self.0 {
                Value::Closure {..} => write!(f, "({})", self.0),
                _ => write!(f, "{}", self.0),
            }
        }
    }
    Callee(callee)
}

impl Value {
    pub fn as_type(&self) -> Option<Type> {
        use Value::*;
        match self {
            NilType | StringType | NumberType
            | FuncType(..) | Param(..) | Call(..)
            | TypeType | Error => Some(Type(self.clone())),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Display)]
pub struct Type(pub Value);

#[derive(Debug, Clone, Copy, Display, From, Into)]
pub struct ParamDepth(pub u32);
