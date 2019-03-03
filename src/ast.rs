use codespan::{ByteSpan, ByteIndex};
use std::{
	convert::TryInto,
	fmt,
};

#[derive(Clone)]
pub struct Spanned<T> {
	pub value: T,
	pub span: ByteSpan,
}

impl<T> fmt::Debug for Spanned<T>
where
	T: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if f.alternate() {
			write!(f, "{:#?} @ {}", self.value, self.span)
		} else {
			write!(f, "{:?} @ {}", self.value, self.span)
		}
	}
}

impl<T> Spanned<T> {
	pub fn from_value_and_byte_offsets(value: T, start: usize, end: usize) -> Self {
		let start: u32 = start.try_into().expect("start index too big");
		let end: u32 = end.try_into().expect("end index too big");
		let span = ByteSpan::new(ByteIndex(start), ByteIndex(end));

		Self { value, span }
	}
}

#[derive(Debug, Clone)]
pub enum Expr {
	EmptyRecord,
	RecordValue(Vec<(Spanned<Ident>, Spanned<Expr>)>),
	RecordType(Vec<(Spanned<Ident>, Spanned<Expr>)>),
	Tuple(Vec<Spanned<Expr>>),
	Block(Vec<Spanned<Expr>>, Option<Box<Spanned<Expr>>>),
	Var(Ident),
	Number(isize),
}

#[derive(Debug, Clone)]
pub struct Ident(String);

impl From<String> for Ident {
	fn from(s: String) -> Self {
		Ident(s)
	}
}

impl<'a> From<&'a str> for Ident {
	fn from(s: &'a str) -> Self {
		Ident(s.into())
	}
}
