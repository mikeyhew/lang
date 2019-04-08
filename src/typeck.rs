use {
    crate::{
        ast::{Name, Expr, ExprKind, Stmt, StmtKind, Span},
        context::TypeContext,
        util::ParamDepth,
        vm::{evaluate, Value},
    },
    derive_more::{Display},
    std::{
        cell::RefCell,
        mem,
    }
};

#[derive(Debug, Display, Clone, Eq, PartialEq)]
pub enum Type {
    #[display(fmt = "Nil")]
    Nil,
    #[display(fmt = "Fn({})({})", _0, _1)]
    Func(Box<Type>, Box<Type>),
    #[display(fmt = "Number")]
    Number,
    #[display(fmt = "String")]
    String_,
    #[display(fmt = "{}", _0)]
    Param(Name, ParamDepth),
    #[display(fmt = "Type")]
    Type,
    #[display(fmt = "TypeError")]
    Error,
}

pub struct ErrorContext {
    in_use: bool,
    errors: Vec<TypeError>,
    has_error: bool,
}

impl ErrorContext {
    fn new() -> Self {
        Self {
            in_use: false,
            errors: Vec::new(),
            has_error: false,
        }
    }
}

thread_local! {
    static ERROR_CONTEXT: RefCell<ErrorContext> = RefCell::new(ErrorContext::new());
}

#[derive(Clone)]
pub struct TypeError {
    pub message: String,
    pub span: Span,
}

impl TypeError {
    fn emit(self) {
        ERROR_CONTEXT.with(|error_context|{
            assert!(error_context.borrow().in_use);

            error_context.borrow_mut().errors.push(self);
            error_context.borrow_mut().has_error = true;
        })
    }
}

macro_rules! type_error {
    ($span:expr, $($fmt_args:tt)*) => {{
        let error = TypeError {
            message: format!($($fmt_args)*),
            span: $span.clone(),
        };

        error.emit();

        Type::Error
    }};
}

/// Takes a function that potentially stores type errors in ERROR_CONTEXT,
/// and returns Err(Vec<TypeError>) if there are errors, and
/// Ok(T) otherwise
fn collect_type_errors<T>(f: impl FnOnce() -> T) -> Result<T, Vec<TypeError>> {
    ERROR_CONTEXT.with(|error_context| {
        assert!(!error_context.borrow().in_use);
        error_context.borrow_mut().in_use = true;
    });

    let value = f();

    ERROR_CONTEXT.with(|error_context| {
        let error_context = mem::replace(&mut *error_context.borrow_mut(), ErrorContext::new());
        assert!(error_context.in_use);

        if error_context.errors.len() == 0 {
            Ok(value)
        } else {
            Err(error_context.errors)
        }
    })
}

fn let_me_know_if_an_error_occurs<T>(f: impl FnOnce() -> T) -> (T, bool) {
    let had_error = ERROR_CONTEXT.with(|error_context| {
        error_context.borrow().has_error
    });

    let output = f();

    ERROR_CONTEXT.with(|error_context| {
        let error_occurred = error_context.borrow().has_error;
        error_context.borrow_mut().has_error = had_error || error_occurred;
        (output, error_occurred)
    })
}

pub fn infer_type(expr: &Expr, context: &TypeContext) -> Result<Type, Vec<TypeError>> {
    collect_type_errors(|| infer_type_internal(expr, context))
}

fn infer_type_internal(expr: &Expr, context: &TypeContext) -> Type {
    match &expr.kind {
        ExprKind::NumberLiteral(_) => Type::Number,
        ExprKind::StringLiteral(_) => Type::String_,

        ExprKind::Block(stmts, expr) => {
            let context = stmts.iter().fold(context.clone(), |context, stmt| {
                typeck_stmt_internal(stmt, &context)
            });

            match expr {
                Some(expr) => infer_type_internal(expr, &context),
                None => Type::Nil,
            }
        }

        ExprKind::Var(ident) => {
            match context.lookup(&ident.name) {
                Some(ty) => ty.clone(),
                None => type_error!(ident.span, "Undeclared variable {}", ident.name),
            }
        }

        ExprKind::Closure(ident, ty_expr, body) => {
            if let Some(ty_expr) = ty_expr {
                let arg_ty = evaluate_type(ty_expr, context);
                let context = context.extend(ident.name.clone(), arg_ty.clone(), Value::Param(ident.name.clone(), ParamDepth::ZERO));

                let return_ty = infer_type_internal(body, &context);

                Type::Func(Box::new(arg_ty), Box::new(return_ty))
            } else {
                type_error!(ident.span, "Cannot infer the type of {}", ident)
            }
        }

        ExprKind::Call(callee, arg) => {
            let callee_ty = infer_type_internal(callee, context);
            let arg_ty = infer_type_internal(arg, context);

            if let Type::Func(input_ty, output_ty) = callee_ty {
                if *input_ty == arg_ty {
                    (*output_ty).clone()
                } else {
                    type_error!(
                        arg.span,
                        "expected {}, found {}", input_ty, arg_ty
                    )
                }
            } else {
                type_error!(
                    callee.span,
                    "expected a function, found a value of type {}", callee_ty
                )
            }
        }

        ExprKind::Parenthesized(expr) => infer_type_internal(&*expr, context),
    }
}

fn evaluate_type(ty_expr: &Expr, context: &TypeContext) -> Type {
    let (ty_expr_type, error_occurred) = let_me_know_if_an_error_occurs(|| {
        infer_type_internal(ty_expr, context)
    });

    if error_occurred {
        return Type::Error
    }

    match ty_expr_type {
        Type::Type => (),
        Type::Error => unreachable!("type error should result in error_occurred being true"),
        ty => return type_error!(
            ty_expr.span,
            "expected a type, found a value of type {}", ty
        ),
    }

    let value = match evaluate(ty_expr, &context.as_value_context()) {
        Ok(value) => value,
        Err(err) => return type_error!(
            ty_expr.span,
            "vm error occurred during evaluation of type expression: {}", err
        )
    };

    match value {
        Value::Type(ty) => ty,
        Value::Param(name, depth) => Type::Param(name, depth),
        _ => type_error!(
            ty_expr.span,
            "evaluation of type expression returned a non-type value: {}", value
        )
    }
}

fn typeck_stmt_internal(stmt: &Stmt, context: &TypeContext) -> TypeContext {
    match &stmt.kind {
        StmtKind::Let(ident, expr) => {
            let ty = infer_type_internal(expr, &context);
            let value = evaluate(expr, &context.as_value_context()).expect("failed to evaluate");
            context.extend(ident.name.clone(), ty, value)
        }
    }
}

pub fn typeck_stmt(stmt: &Stmt, context: &TypeContext) -> Result<TypeContext, Vec<TypeError>> {
    collect_type_errors(|| typeck_stmt_internal(stmt, context))
}
