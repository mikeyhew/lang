#![warn(rust_2018_idioms)]
#![allow(unreachable_pub)]

mod ast;
mod builtin;
mod context;
#[allow(rust_2018_idioms)]
mod parser;
mod typeck;
mod util;
mod vm;

use {
    crate::{
        ast::ReplLineKind,
        context::{TypeContext, ValueContext},
        parser::ReplLineParser,
        typeck::{Type, typeck_stmt, infer_type},
        vm::{Value, evaluate, evaluate_stmt},
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

    let mut type_context = TypeContext::default();
    let mut value_context = ValueContext::default();

    'repl: loop {
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

        let repl_line = match ReplLineParser::new().parse(&line) {
            Ok(repl_line) => repl_line,
            Err(err) => {
                println!("{}", err);
                continue
            }
        };

        match &repl_line.kind {
            ReplLineKind::Block(stmts, expr) => {
                // type-check and evaluate each statement, replacing the contexts after statement
                for stmt in stmts {
                    match typeck_stmt(&stmt, &type_context) {
                        Ok(tcx) => type_context = tcx,
                        Err(errors) => {
                            for error in errors {
                                println!("{} at {}", error.message, error.span);
                            }
                            continue 'repl
                        }
                    }

                    match evaluate_stmt(&stmt, &value_context) {
                        Ok(vcx) => value_context = vcx,
                        Err(err) => {
                            println!("{}", err);
                            continue 'repl
                        }
                    }
                }

                let (value, ty) = match expr {
                    None => (Value::Nil, Type::Nil),
                    Some(expr) => {
                        let ty = match infer_type(expr, &type_context) {
                            Ok(ty) => ty,
                            Err(errors) => {
                                for error in errors {
                                    println!("{} at {}", error.message, error.span);
                                }
                                continue 'repl
                            }
                        };

                        let value = match evaluate(expr, &value_context) {
                            Ok(value) => value,
                            Err(err) => {
                                println!("{}", err);
                                continue 'repl
                            }
                        };

                        (value, ty)
                    }
                };

                println!("{}: {}", value, ty);
            }
        }
    }

    line_reader.save_history("history.txt").unwrap();
}
