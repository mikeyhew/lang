#![warn(rust_2018_idioms)]
#![allow(unused_extern_crates)]
#![allow(unreachable_pub)]


// lalrpop-generated code is not written for Rust 2018
#[allow(rust_2018_idioms)]
mod parser;
// mod lexer;

use rustyline::{
    error::ReadlineError::{Interrupted, Eof},
};

use crate::{
    parser::ExprParser,
};

#[derive(Debug, Clone)]
pub enum Expr {
    Type,
    RecordType(Vec<(String, Expr)>),
    Record(Vec<(String, Expr)>),
    EmptyRecord,
    Tuple(Vec<Expr>),
    Var(String),
    Number(isize),
}

fn main() {
    let mut line_reader = rustyline::Editor::<()>::new();

    if let Err(_) = line_reader.load_history("history.txt") {
        println!("No previous history.");
    }

    loop {
        let line = match line_reader.readline("> ") {
            Ok(line) => {
                line_reader.add_history_entry(&line);
                line
            },
            Err(Interrupted) => {
                continue
            },
            Err(Eof) => {
                println!("Goodbye!");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        };

        let expr = match ExprParser::new().parse(&line) {
            Ok(term) => term,
            Err(err) => {
                println!("{}", err);
                continue
            }
        };
        println!("{:?}", expr);
    }

    line_reader.save_history("history.txt").unwrap();
}
