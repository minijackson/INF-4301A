use type_sys;

use std::fmt;

#[derive(Debug,Clone,PartialEq)]
pub struct Exprs {
    pub exprs: Vec<Box<Expr>>,
}

#[derive(Debug,Clone,PartialEq)]
pub enum Expr {
    Grouping(Exprs),
    Let(Vec<Binding>, Exprs),
    Assign(String, Box<Expr>),
    Function(String, Vec<Box<Expr>>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    While(Box<Expr>, Box<Expr>),
    For(Box<Binding>, Box<Expr>, Box<Expr>),
    BinaryOp(Box<Expr>, Box<Expr>, BinaryOpCode),
    UnaryOp(Box<Expr>, UnaryOpCode),
    Variable(String),
    Value(type_sys::Value),
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum BinaryOpCode {
    Add,
    Sub,
    Mul,
    Div,

    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl fmt::Display for BinaryOpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::BinaryOpCode::*;

        write!(f,
               "{}",
               match self {
                   &Add => "+",
                   &Sub => "-",
                   &Mul => "*",
                   &Div => "/",

                   &Lt => "<",
                   &Le => "<=",
                   &Gt => ">",
                   &Ge => ">=",
                   &Eq => "=",
                   &Ne => "<>",
               })
    }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum UnaryOpCode {
    Plus,
    Minus,
}

impl fmt::Display for UnaryOpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::UnaryOpCode::*;

        write!(f,
               "{}",
               match self {
                   &Plus => "+",
                   &Minus => "-",
               })
    }
}

#[derive(Debug,Clone,PartialEq)]
pub struct Binding {
    pub variable: String,
    pub value: Expr,
}
