use {
    crate::{
        ast::{Expr, ExprKind, StmtKind, Name, Span},
        util::{Map, join, mapping},
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
    #[display(fmt = "Number")]
    Number,
    #[display(fmt = "String")]
    String_,
    #[display(fmt = "Type")]
    Type,
    #[display(fmt = "TypeError")]
    Error,
}

type ErrorContext = Vec<TypeError>;

thread_local! {
    static ERROR_CONTEXT: RefCell<ErrorContext> = RefCell::new(ErrorContext::new());
}

pub struct TypeError {
    pub message: String,
    pub span: Span,
}

impl TypeError {
    fn emit(self) {
        ERROR_CONTEXT.with(|errors|{
            errors.borrow_mut().push(self);
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

pub fn infer_type(expr: &Expr, type_context: &TypeContext) -> Result<Type, Vec<TypeError>> {
    assert!(ERROR_CONTEXT.with(|errors| errors.borrow().len() == 0));

    let ty = infer_type_internal(expr, type_context);

    ERROR_CONTEXT.with(|errors| {
        if errors.borrow().len() == 0 {
            Ok(ty)
        } else {
            // replace ERROR_CONTEXT with an empty vec, and return the errors we just got from it
            Err(mem::replace(&mut *errors.borrow_mut(), Vec::new()))
        }
    })
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
                    number + 1,
                    tuple_type
                )
            }
        }

        ExprKind::RecordValue(map) => {
            // TODO: handle dependent records
            Type::Record(map.iter().map(|(n, e)| {
                (n.name.clone(), infer_type_internal(e, type_context))
            }).collect())
        }
        ExprKind::RecordFieldAccess(..) => unimplemented!("RecordFieldAccess"),

        ExprKind::Block(stmts, expr) => {
            let type_context = stmts.iter().fold(type_context.clone(), |type_context, stmt| {
                match &stmt.kind {
                    StmtKind::Let(ident, expr) => {
                        let ty = infer_type_internal(expr, &type_context);
                        type_context.extend(ident.name.clone(), ty)
                    }
                }
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

        ExprKind::Parenthesized(expr) => infer_type_internal(&*expr, type_context),

        ExprKind::NilType |
        ExprKind::RecordType(_) |
        ExprKind::TupleType(_) => Type::Type,
    }
}
