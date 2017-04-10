pub mod ast;
pub mod builtins;
pub mod env;
pub mod error;
pub mod parser;
pub mod processing;
pub mod repl;
pub mod type_sys;

use processing::{Evaluate,Print,TypeCheck};
use env::{Environment,ValueInfo};
use error::{handle_error, ParseError, TypeCheckError};

extern crate itertools;
extern crate lalrpop_util;
extern crate rustyline;

use std::env::args;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let argc = args().count();
    if argc == 1 {
        repl::start();
    } else if argc == 2 {
        evaluate_file(args().nth(1).unwrap());
    }
}

fn evaluate_file(filename: String) {
    let mut file = File::open(filename.clone())
        .expect(format!("Could not open file {}", filename).as_str());
    let mut content = String::new();

    file.read_to_string(&mut content).unwrap();

    match parse_expressions(content.as_str()) {
        Ok(exprs) => {
            if let Err(err) = do_the_thing(exprs, &mut Environment::new()) {
                handle_error(&filename, Box::new(err));
            }
        }
        Err(err) => {
            handle_error(&filename, Box::new(err));
        }
    }
}

fn do_the_thing(mut exprs: ast::Exprs, mut bindings: &mut Environment<ValueInfo>) -> Result<(), TypeCheckError> {
    println!("Result: {:?}", exprs);
    println!("===== Pretty printing =====\n{}===========================", &exprs.pretty_print(0));
    println!("Final type (type checker): {:?}", &mut exprs.type_check(&mut Environment::new())?);
    println!("Final value: {:?}", &exprs.evaluate(&mut bindings));
    Ok(())
}

fn parse_expressions<'a>(partial_input: &'a str) -> Result<ast::Exprs, ParseError<'a>> {
    match parser::parse_Expressions(partial_input) {
        Ok(exprs) => Ok(exprs),
        Err(err) => Err(From::from(err.clone())),
    }
}
