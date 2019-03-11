#![warn(rust_2018_idioms)]
#![allow(unreachable_pub)]

mod ast;
#[allow(rust_2018_idioms)]
mod parser;
mod typeck;
mod util;
mod vm;

use {
    crate::{
        parser::ExprParser,
        typeck::infer_type,
    },
    rustyline::{
        error::ReadlineError::{Interrupted, Eof},
    },
};

fn main() {
    let mut line_reader = rustyline::Editor::<()>::new();

    if let Err(_) = line_reader.load_history("history.txt") {
        println!("No previous history.");
    }

    loop {
        let line = match line_reader.readline("> ") {
            Ok(line) => {
                line_reader.add_history_entry(line.as_str());
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

        let type_context = typeck::TypeContext::new();
        let ty = match infer_type(&expr, &type_context){
            Ok(ty) => ty,
            Err(errors) => {
                for error in errors {
                    println!("{} at {}", error.message, error.span);
                }
                continue
            }
        };

        println!("type: {}", ty);

        let context = vm::ValueContext::new();
        let value = match vm::evaluate(&expr, &context) {
            Ok(value) => value,
            Err(err) => {
                println!("{}", err);
                continue
            }
        };

        println!("{}", value);
    }

    line_reader.save_history("history.txt").unwrap();
}
