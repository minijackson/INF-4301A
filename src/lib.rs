//! Where everything is really done.
//!
//! The main module only have the highest level functions: launch the whole pipeline, process a
//! file, etc.

pub mod ast;
pub mod builtins;
pub mod env;
pub mod error;
pub mod parser;
pub mod processing;
pub mod repl;
pub mod type_sys;

use processing::{Evaluate, Print, TypeCheck};
use env::{Environment, ValueInfo};
use error::{print_error, ParseError, TypeCheckError};

extern crate itertools;
extern crate lalrpop_util;
extern crate rustyline;
extern crate term;

use std::fs::File;
use std::io::prelude::*;

/// Evaluate the given file
pub fn evaluate_file(filename: &str) {
    let mut file = File::open(filename)
        .expect(format!("Could not open file {}", filename).as_str());
    let mut content = String::new();

    file.read_to_string(&mut content).unwrap();

    match parse_expressions(content.as_str()) {
        Ok(exprs) => {
            if let Err(err) = do_the_thing(exprs, &mut Environment::new()) {
                print_error(filename, &content, &err);
            }
        }
        Err(err) => {
            print_error(filename, &content, &err);
        }
    }
}

/// Evaluate the given AST (going through the type checker, pretty printing, printing the AST, ...)
///
/// # Examples
///
/// ```
/// use compilib::do_the_thing;
/// use compilib::ast::*;
/// use compilib::ast::Expr::*;
/// use compilib::type_sys::Value::*;
/// use compilib::env::Environment;
///
/// let res = do_the_thing(Exprs { exprs: vec![Box::new(Value(Integer(42)))] },
///                        &mut Environment::new());
/// assert!(res.is_ok());
///
/// let res = do_the_thing(Exprs { exprs: vec![Box::new(BinaryOp {
///                            lhs: Box::new(Value(Integer(42))),
///                            rhs: Box::new(Value(Float(69f64))),
///                            op: BinaryOpCode::Add,
///                            span: Span(0, 5),
///                        })] },
///                        &mut Environment::new());
/// assert!(res.is_err());
/// ```
pub fn do_the_thing(mut exprs: ast::Exprs, mut bindings: &mut Environment<ValueInfo>) -> Result<(), TypeCheckError> {
    println!("Result: {:?}", exprs);
    println!("===== Pretty printing =====\n{}===========================", &exprs.pretty_print(0));
    println!("Final type (type checker): {:?}", &mut exprs.type_check(&mut Environment::new())?);
    println!("Final value: {:?}", &exprs.evaluate(&mut bindings));
    Ok(())
}

/// Parse several expresssions (comma separated) given a corpus.
///
/// Mainly used to convert a lalrpop error to a `ParseError`
pub fn parse_expressions(partial_input: &str) -> Result<ast::Exprs, ParseError> {
    match parser::parse_Expressions(partial_input) {
        Ok(exprs) => Ok(exprs),
        Err(err) => Err(From::from(err.clone())),
    }
}
