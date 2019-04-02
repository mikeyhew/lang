#![warn(rust_2018_idioms)]
#![allow(unreachable_pub)]

mod ast;
mod context;
#[allow(rust_2018_idioms)]
mod parser;
mod typeck;
mod util;
mod vm;

use {
    crate::{
        ast::ReplLineKind,
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

    let mut type_context = typeck::TypeContext::new();
    let mut value_context = vm::ValueContext::new();

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
                // type-check and evaluate each statement, replacing type_context and context for each one
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

        // let ty = match infer_type(&expr, &type_context){
        //     Ok(ty) => ty,
        //     Err(errors) => {
        //         for error in errors {
        //             println!("{} at {}", error.message, error.span);
        //         }
        //         continue
        //     }
        // };

        // println!("type: {}", ty);

        // let context = vm::ValueContext::new();
        // let value = match vm::evaluate(&expr, &context) {
        //     Ok(value) => value,
        //     Err(err) => {
        //         println!("{}", err);
        //         continue
        //     }
        // };

        // println!("{}", value);
    }

    line_reader.save_history("history.txt").unwrap();
}
