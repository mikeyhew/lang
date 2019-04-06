use codespan::{ByteSpan, ByteIndex};
use std::{
	convert::TryInto,
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

#[derive(Debug, Clone)]
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
	Nil,
	NilType,
	RecordValue(Vec<(Ident, Expr)>),
	RecordType(Vec<(Ident, Expr)>),
	RecordFieldAccess(Box<Expr>, Ident),
	Tuple(Vec<Expr>),
	TupleType(Vec<Expr>),
	TupleFieldAccess(Box<Expr>, usize),
	Block(Vec<Stmt>, Option<Box<Expr>>),
	Var(Ident),
	Closure(Ident, Option<Box<Expr>>, Box<Expr>),
	Call(Box<Expr>, Box<Expr>),
	NumberLiteral(Number),
	StringLiteral(String),
	Parenthesized(Box<Expr>),
}

#[derive(Debug, Clone)]
pub struct Stmt {
	pub kind: StmtKind,
	pub span: Span,
}

impl Stmt {
	pub fn new((kind, span): (StmtKind, Span)) -> Self {
		Self {kind, span}
	}
}

#[derive(Debug, Clone)]
pub enum StmtKind {
	Let(Ident, Box<Expr>),
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

pub struct ReplLine {
	pub kind: ReplLineKind,
	pub span: Span,
}

pub enum ReplLineKind {
	/// Like a Block expression, but without braces around it.
	/// This is the "normal" type of REPL line, as opposed to one with a `:` in front
	/// Which usually means you are changing a setting (but that's not implemented atm)
	Block(Vec<Stmt>, Option<Expr>),
}
