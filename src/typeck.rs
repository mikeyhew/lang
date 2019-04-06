use {
    crate::{
        ast::{Expr, ExprKind, Stmt, StmtKind, Name, Span},
        util::{Map, join, mapping},
        vm::{evaluate_type, ValueContext},
    },
    derive_more::{Display},
    std::{
        cell::RefCell,
        mem,
    }
};

pub use crate::context::TypeContext;

#[derive(Debug, Display, Clone, Eq, PartialEq)]
pub enum Type {
    #[display(fmt = "Nil")]
    Nil,
    #[display(fmt = "{{{}}}", r#"join(", ", _0.iter().map(mapping(": ")))"#)]
    Record(Map<Name, Type>),
    #[display(fmt = "type ({})", r#"join(", ", _0.iter())"#)]
    Tuple(Vec<Type>),
    #[display(fmt = "Fn(({}, {}))", _0, _1)]
    Func(Box<Type>, Box<Type>),
    #[display(fmt = "Number")]
    Number,
    #[display(fmt = "String")]
    String_,
    #[display(fmt = "Type")]
    Type,
    #[display(fmt = "TypeError")]
    Error,
}

pub struct ErrorContext {
    in_use: bool,
    errors: Vec<TypeError>,
}

impl ErrorContext {
    fn new() -> Self {
        Self {
            in_use: false,
            errors: Vec::new(),
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

pub fn infer_type(expr: &Expr, type_context: &TypeContext) -> Result<Type, Vec<TypeError>> {
    collect_type_errors(|| infer_type_internal(expr, type_context))
}

fn infer_type_internal(expr: &Expr, type_context: &TypeContext) -> Type {
    match &expr.kind {
        ExprKind::Nil => Type::Nil,

        ExprKind::NumberLiteral(_) => Type::Number,
        ExprKind::StringLiteral(_) => Type::String_,

        ExprKind::Tuple(vec) => {
            // TODO: support dependent tuples
            Type::Tuple(vec.iter().map(|e| infer_type_internal(e, type_context)).collect())
        }
        ExprKind::TupleType(vec) => {
            vec.iter().for_each(|ty_expr| {
                typeck_type_internal(ty_expr, type_context);
            });

            Type::Type
        }
        ExprKind::TupleFieldAccess(tuple_expr, number) => {
            let tuple_type = infer_type_internal(tuple_expr, type_context);

            match &tuple_type {
                Type::Tuple(field_types) => {
                    if let Some(field_type) = field_types.get(*number){
                        field_type.clone()
                    } else {
                        type_error!(
                            expr.span,
                            "field number {} is out of range for tuple {}",
                            number, tuple_type
                        )
                    }
                }
                _ => type_error!(
                    tuple_expr.span,
                    "expected a tuple with at least {} elements, found {}",
                    number + 1, tuple_type
                )
            }
        }

        ExprKind::RecordValue(pairs) => {
            // TODO: handle dependent records
            Type::Record(pairs.iter().map(|(ident, expr)| {
                (ident.name.clone(), infer_type_internal(expr, type_context))
            }).collect())
        }
        ExprKind::RecordType(pairs) => {
            pairs.iter().for_each(|(_, ty_expr)| {
                typeck_type_internal(ty_expr, type_context);
            });

            Type::Type
        }
        ExprKind::RecordFieldAccess(..) => unimplemented!("RecordFieldAccess"),

        ExprKind::Block(stmts, expr) => {
            let type_context = stmts.iter().fold(type_context.clone(), |type_context, stmt| {
                typeck_stmt_internal(stmt, &type_context)
            });

            match expr {
                Some(expr) => infer_type_internal(expr, &type_context),
                None => Type::Nil,
            }
        }

        ExprKind::Var(ident) => {
            match type_context.lookup(&ident.name) {
                Some(ty) => ty.clone(),
                None => type_error!(ident.span, "Undeclared variable {}", ident.name),
            }
        }

        ExprKind::Closure(ident, ty_expr, body) => {
            if let Some(ty_expr) = ty_expr {
                let arg_ty = match typeck_type_internal(ty_expr, type_context) {
                    Type::Error => Type::Error,
                    Type::Type => {
                        // TODO: more than just the default context
                        evaluate_type(ty_expr, &ValueContext::default())
                            .unwrap_or_else(|err| {
                                type_error!(
                                    ty_expr.span,
                                    "type failed to evaluate: {}", err
                                )
                            })
                    }
                    ty => unreachable!(&format!("bad type: {}", ty)),
                };
                let type_context = type_context.extend(ident.name.clone(), arg_ty.clone());

                let return_ty = infer_type_internal(body, &type_context);

                Type::Func(Box::new(arg_ty), Box::new(return_ty))
            } else {
                type_error!(ident.span, "Cannot infer the type of {}", ident)
            }
        }

        ExprKind::Call(callee, arg) => {
            let callee_ty = infer_type_internal(callee, type_context);
            let arg_ty = infer_type_internal(arg, type_context);

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

        ExprKind::Parenthesized(expr) => infer_type_internal(&*expr, type_context),

        ExprKind::NilType => Type::Type,
    }
}

fn typeck_stmt_internal(stmt: &Stmt, type_context: &TypeContext) -> TypeContext {
    match &stmt.kind {
        StmtKind::Let(ident, expr) => {
            let ty = infer_type_internal(expr, &type_context);
            type_context.extend(ident.name.clone(), ty)
        }
    }
}

pub fn typeck_stmt(stmt: &Stmt, type_context: &TypeContext) -> Result<TypeContext, Vec<TypeError>> {
    collect_type_errors(|| typeck_stmt_internal(stmt, type_context))
}

/// typechecks a type expression, and returns whether or not it succeeded
fn typeck_type_internal(ty_expr: &Expr, type_context: &TypeContext) -> Type {
    match infer_type_internal(ty_expr, type_context) {
        ty@Type::Type | ty@Type::Error => ty,
        ty => {
            type_error!(
                ty_expr.span,
                "expected a type, found a value of type {}", ty,
            )
        }
    }
}
