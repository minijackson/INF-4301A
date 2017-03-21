#![feature(box_syntax,box_patterns,slice_patterns)]

extern crate rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub mod calculator;

pub mod ast;
use ast::{Expr,OpCode};

fn evaluate(tree: &Expr) -> i32 {
    use Expr::*;
    use OpCode::*;

    match tree {
        &BinaryOp(box ref lhs, box ref rhs, ref op) => match op {
            &Add => evaluate(lhs) + evaluate(rhs),
            &Sub => evaluate(lhs) - evaluate(rhs),
            &Mul => evaluate(lhs) * evaluate(rhs),
            &Div => evaluate(lhs) / evaluate(rhs),
        },
        &Num(value) => value,
    }
}

fn reverse_polish(tree: &Expr) -> String {
    use Expr::*;
    use OpCode::*;

    match tree {
        &BinaryOp(box ref lhs, box ref rhs, ref op) => match op {
            &Add => format!("{} {} +", reverse_polish(&lhs), reverse_polish(&rhs)),
            &Sub => format!("{} {} -", reverse_polish(&lhs), reverse_polish(&rhs)),
            &Mul => format!("{} {} *", reverse_polish(&lhs), reverse_polish(&rhs)),
            &Div => format!("{} {} /", reverse_polish(&lhs), reverse_polish(&rhs)),
        },
        &Num(value) => value.to_string(),
    }
}

fn lisp(tree: &Expr) -> String {
    use Expr::*;
    use OpCode::*;

    match tree {
        &BinaryOp(box ref lhs, box ref rhs, ref op) => match op {
            &Add => format!("(+ {} {})", lisp(&lhs), lisp(&rhs)),
            &Sub => format!("(- {} {})", lisp(&lhs), lisp(&rhs)),
            &Mul => format!("(* {} {})", lisp(&lhs), lisp(&rhs)),
            &Div => format!("(/ {} {})", lisp(&lhs), lisp(&rhs)),
        },
        &Num(value) => value.to_string(),
    }
}

fn main() {
    let mut rl = Editor::<()>::new();
    if let Err(_) = rl.load_history("history.txt") {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline("> ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                let exp = calculator::parse_Expression(line.as_str()).unwrap();
                println!("Result: {:?}", exp);
                println!("Value: {}", evaluate(&exp));
                println!("RPN: {}", reverse_polish(&exp));
                println!("Lisp: {}", lisp(&exp));
            },
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
        rl.save_history("history.txt").unwrap();
    }
}
