pub mod ast;
pub mod builtins;
pub mod env;
pub mod parser;
pub mod processing;
pub mod repl;
pub mod type_sys;

use processing::{Evaluate,Print};
use env::{Environment,ValueInfo};

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

    let exprs = parser::parse_Expressions(content.as_str()).unwrap();

    do_the_thing(exprs, &mut Environment::new());
}

fn do_the_thing(exprs: ast::Exprs, mut bindings: &mut Environment<ValueInfo>) {
    println!("Result: {:?}", exprs);
    println!("===== Pretty printing =====\n{}===========================", &exprs.pretty_print(0));
    println!("Final value: {:?}", &exprs.evaluate(&mut bindings));
}
