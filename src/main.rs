#![feature(box_syntax,box_patterns,slice_patterns)]

#[macro_use]
extern crate nom;

use nom::{IResult, digit};

#[derive(Debug)]
enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Num(i32),
}

named!(parens<Expr>, ws!(delimited!(tag!("("), expression, tag!(")"))));

named!(factor<Expr>,
       alt!(
           map_res!(
               ws!(digit),
               |value: &[u8]|
                   std::str::from_utf8(value).unwrap().parse::<i32>().map(Expr::Num)
           )
           | parens
       )
);

named!(term<Expr>,
       do_parse!(
           lhs: factor >>
           res: fold_many0!(
               do_parse!(
                   op: alt!(tag!("*") | tag!("/")) >>
                   value: factor >>
                   (op, value)
               ),
               lhs,
               |acc, (op, value): (&[u8], Expr)| {
                   if op[0] as char == '*' {
                       Expr::Mul(box acc, box value)
                   } else {
                       Expr::Div(box acc, box value)
                   }
               }
           ) >>
           (res)
       )
);

named!(expression<Expr>,
       do_parse!(
           lhs: term >>
           res: fold_many0!(
               do_parse!(
                   op: alt!(tag!("+") | tag!("-")) >>
                   value: term >>
                   (op, value)
               ),
               lhs,
               |acc, (op, value): (&[u8], Expr)| {
                   if op[0] as char == '+' {
                       Expr::Add(box acc, box value)
                   } else {
                       Expr::Sub(box acc, box value)
                   }
               }
           ) >>
           (res)
       )
);

fn consume(tree: &Expr) -> i32 {
    use Expr::*;

    match tree {
        &Add(box ref lhs, box ref rhs) => consume(&lhs) + consume(&rhs),
        &Sub(box ref lhs, box ref rhs) => consume(&lhs) - consume(&rhs),
        &Mul(box ref lhs, box ref rhs) => consume(&lhs) * consume(&rhs),
        &Div(box ref lhs, box ref rhs) => consume(&lhs) / consume(&rhs),
        &Num(value) => value,
    }
}

fn reverse_polish(tree: &Expr) -> String {
    use Expr::*;

    match tree {
        &Add(box ref lhs, box ref rhs) => format!("{} {} +", reverse_polish(&lhs), reverse_polish(&rhs)),
        &Sub(box ref lhs, box ref rhs) => format!("{} {} -", reverse_polish(&lhs), reverse_polish(&rhs)),
        &Mul(box ref lhs, box ref rhs) => format!("{} {} *", reverse_polish(&lhs), reverse_polish(&rhs)),
        &Div(box ref lhs, box ref rhs) => format!("{} {} /", reverse_polish(&lhs), reverse_polish(&rhs)),
        &Num(value) => value.to_string(),
    }
}

fn main() {
    let exp = expression(&b"42 +3 * 7- 1/1"[..]);
    println!("Expression: {:?}", exp);
    if let IResult::Done(&[], exp) = exp {
        println!("Value: {}", consume(&exp));
        println!("RPN: {}", reverse_polish(&exp));
    } else {
        println!("Error while parsing: {:?}", exp);
    }
}
