#![warn(rust_2018_idioms)]
#![allow(unreachable_pub)]

mod ast;
#[allow(rust_2018_idioms)]
mod parser;
mod util;
mod vm;

use rustyline::{
    error::ReadlineError::{Interrupted, Eof},
};
use parser::ExprParser;

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
        // println!("{:#?}", expr);
        // // empty line
        // println!();

        let context = vm::Context::new();

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
