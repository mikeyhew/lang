#![macro_use]
use crate::{
    ast::Span,
};

pub struct EvalError(pub String);

pub type EvalResult<T> = Result<T, EvalError>;

#[macro_export]
macro_rules! eval_error {
    ($fmt:expr, $($tt:tt)*) => {{
        let message = format!($fmt, $($tt)*);
        $crate::eval::EvalError(message)
    }}
}

