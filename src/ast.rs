use type_sys;

use std::fmt;

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct Span(pub usize, pub usize);

#[derive(Debug,Clone,PartialEq)]
pub struct Exprs {
    pub exprs: Vec<Box<Expr>>,
}

#[derive(Debug,Clone,PartialEq)]
pub enum Expr {
    Grouping(Exprs),
    Let(Vec<Binding>, Exprs),
    Assign {
        name: String,
        value: Box<Expr>,
        value_span: Span,
    },
    Function {
        name: String,
        args: Vec<(Box<Expr>, Span)>,
        span: Span,
    },
    If {
        cond: Box<Expr>,
        true_branch: Box<Expr>,
        false_branch: Box<Expr>,
        cond_span: Span,
        false_branch_span: Span,
    },
    While {
        cond: Box<Expr>,
        expr: Box<Expr>,
        cond_span: Span,
    },
    For {
        binding: Box<Binding>,
        goal: Box<Expr>,
        expr: Box<Expr>,
        goal_span: Span,
    },
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
    pub span: Span,
    pub value_span: Span,
}
