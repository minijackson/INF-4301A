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
        name_span: Span,
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
        cond_span: Span,
        true_branch: Box<Expr>,
        true_branch_span: Span,
        false_branch: Box<Expr>,
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
        goal_span: Span,
        expr: Box<Expr>,
    },
    BinaryOp {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: BinaryOpCode,
        span: Span,
    },
    UnaryOp {
        expr: Box<Expr>,
        op: UnaryOpCode,
        span: Span,
    },
    Cast {
        expr: Box<Expr>,
        expr_span: Span,
        dest: type_sys::Type,
    },
    Variable {
        name: String,
        span: Span,
    },
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
