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
use error::{print_error, ParseError, TypeCheckError};

extern crate itertools;
extern crate lalrpop_util;
extern crate rustyline;
extern crate term;

pub fn do_the_thing(mut exprs: ast::Exprs, mut bindings: &mut Environment<ValueInfo>) -> Result<(), TypeCheckError> {
    println!("Result: {:?}", exprs);
    println!("===== Pretty printing =====\n{}===========================", &exprs.pretty_print(0));
    println!("Final type (type checker): {:?}", &mut exprs.type_check(&mut Environment::new())?);
    println!("Final value: {:?}", &exprs.evaluate(&mut bindings));
    Ok(())
}

pub fn parse_expressions<'a>(partial_input: &'a str) -> Result<ast::Exprs, ParseError<'a>> {
    match parser::parse_Expressions(partial_input) {
        Ok(exprs) => Ok(exprs),
        Err(err) => Err(From::from(err.clone())),
    }
}
