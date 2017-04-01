#![feature(box_syntax,box_patterns,slice_patterns)]

extern crate rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub mod calculator;
pub mod ast;
pub mod processing;

use processing::{Evaluate,Print};

use std::collections::HashMap;

fn main() {
    let mut rl = Editor::<()>::new();
    if let Err(_) = rl.load_history("history.txt") {
        println!("No previous history.");
    }

    let mut bindings = HashMap::new();

    loop {
        let readline = rl.readline("> ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                let exps = calculator::parse_Expressions(line.as_str()).unwrap();
                println!("Result: {:?}", exps);
                println!("===== RPN =====\n{}===============", &exps.reverse_polish());
                //println!("Lisp: {}", &exp.lisp());
                println!("Value: {}", &exps.evaluate(&mut bindings));
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
