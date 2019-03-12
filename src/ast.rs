use codespan::{ByteSpan, ByteIndex};
use std::{
	convert::TryInto,
	fmt,
};
use derive_more::{
	Display,
	From,
	Into,
};

#[derive(Debug, Display, Clone, From)]
pub struct Span(ByteSpan);

impl Span {
	pub fn from_byte_offsets(start: usize, end: usize) -> Self {
		let start: u32 = start.try_into().expect("start index too big");
		let end: u32 = end.try_into().expect("end index too big");
		ByteSpan::new(ByteIndex(start), ByteIndex(end)).into()
	}
}

#[derive(Debug, Display, From, Into, Clone, Hash, Eq, PartialEq)]
pub struct Name(String);

impl AsRef<str> for Name {
	fn as_ref(&self) -> &str {
		self.0.as_ref()
	}
}

impl<'a> From<&'a str> for Name {
	fn from(name: &'a str) -> Self {
		name.to_string().into()
	}
}

#[derive(Debug, Display, From, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Number(i64);

#[derive(Debug, Display, Clone)]
#[display(fmt = "{}", kind)]
pub struct Expr {
	pub kind: ExprKind,
	pub span: Span,
}

impl Expr {
	pub fn new((kind, span): (ExprKind, Span)) -> Self {
		Self {kind, span}
	}
}

#[derive(Debug, Clone)]
pub enum ExprKind {
	EmptyRecord,
	RecordValue(Vec<(Ident, Expr)>),
	RecordType(Vec<(Ident, Expr)>),
	Tuple(Vec<Expr>),
	TupleType(Vec<Expr>),
	Block(Vec<Expr>, Option<Box<Expr>>),
	Var(Ident),
	Let(Ident, Box<Expr>, Box<Expr>),
	RecordFieldAccess(Box<Expr>, Ident),
	TupleFieldAccess(Box<Expr>, usize),
	NumberLiteral(Number),
	StringLiteral(String),
	Parenthesized(Box<Expr>),
}

impl fmt::Display for ExprKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ExprKind::EmptyRecord => write!(f, "{{}}"),
			ExprKind::RecordValue(pairs) => {
				write!(f, "{{")?;

				for (key, value) in pairs {
					write!(f, "{}={}", key, value)?;
				}

				write!(f, "}}")
			}
			_ => unimplemented!()
		}
	}
}

#[derive(Debug, Display, Clone)]
#[display(fmt = "{}", name)]
pub struct Ident {
	pub name: Name,
	pub span: Span,
}

impl Ident {
	pub fn new(name: &str, span: Span) -> Self {
		Self {
			name: name.into(),
			span,
		}
	}
}

impl AsRef<Name> for Ident {
	fn as_ref(&self) -> &Name {
		&self.name
	}
}
