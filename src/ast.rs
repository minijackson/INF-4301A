#[derive(Debug,PartialEq,Eq)]
pub struct Exprs {
    pub exprs: Vec<Box<Expr>>
}

#[derive(Debug,PartialEq,Eq)]
pub enum Expr {
    Grouping(Exprs),
    Let(Vec<Binding>, Exprs),
    Function(String, Vec<Box<Expr>>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    BinaryOp(Box<Expr>, Box<Expr>, BinaryOpCode),
    UnaryOp(Box<Expr>, UnaryOpCode),
    Variable(String),
    Num(i32),
}

#[derive(Debug,PartialEq,Eq)]
pub enum BinaryOpCode {
    Add, Sub, Mul, Div,
}

#[derive(Debug,PartialEq,Eq)]
pub enum UnaryOpCode {
    Plus, Minus
}

#[derive(Debug,PartialEq,Eq)]
pub struct Binding {
    pub variable: String,
    pub value: Expr
}
