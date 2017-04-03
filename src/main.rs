#![feature(box_syntax,box_patterns,slice_patterns)]

pub mod ast;
pub mod builtins;
pub mod common;
pub mod parser;
pub mod processing;

use processing::{Evaluate,Print};
use common::{Environment,ValueInfo};

extern crate rustyline;
extern crate itertools;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let argc = env::args().count();
    if argc == 1 {
        repl();
    } else if argc == 2 {
        evaluate_file(env::args().nth(1).unwrap());
    }
}

fn repl() {
    let mut rl = Editor::<()>::new();
    if let Err(_) = rl.load_history("history.txt") {
        println!("No previous history.");
    }

    let mut bindings = Environment::new();

    loop {
        let readline = rl.readline("> ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                do_the_thing(line, &mut bindings)
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
        rl.save_history("history.txt").unwrap();
    }
}

fn evaluate_file(filename: String) {
    let mut file = File::open(filename.clone())
        .expect(format!("Could not open file {}", filename).as_str());
    let mut content = String::new();

    file.read_to_string(&mut content).unwrap();

    do_the_thing(content, &mut Environment::new());
}

fn do_the_thing<'a>(input: String, mut bindings: &'a mut Environment<ValueInfo>) {
    let exps = parser::parse_Expressions(input.as_str()).unwrap();
    println!("Result: {:?}", exps);
    println!("===== Pretty printing =====\n{}===========================", &exps.pretty_print(0));
    //println!("Lisp: {}", &exp.lisp());
    println!("Final value: {:?}", &exps.evaluate(&mut bindings));
}
