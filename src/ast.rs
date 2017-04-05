use type_sys;

#[derive(Debug,Clone,PartialEq)]
pub struct Exprs {
    pub exprs: Vec<Box<Expr>>
}

#[derive(Debug,Clone,PartialEq)]
pub enum Expr {
    Grouping(Exprs),
    Let(Vec<Binding>, Exprs),
    Assign(String, Box<Expr>),
    Function(String, Vec<Box<Expr>>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    While(Box<Expr>, Box<Expr>),
    BinaryOp(Box<Expr>, Box<Expr>, BinaryOpCode),
    UnaryOp(Box<Expr>, UnaryOpCode),
    Variable(String),
    Value(type_sys::Value),
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum BinaryOpCode {
    Add, Sub, Mul, Div,
    Lt, Le, Gt, Ge, Eq, Ne
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum UnaryOpCode {
    Plus, Minus
}

#[derive(Debug,Clone,PartialEq)]
pub struct Binding {
    pub variable: String,
    pub value: Expr
}
