#[derive(Debug)]
pub struct Exprs {
    pub exprs: Vec<Box<Expr>>
}

#[derive(Debug)]
pub enum Expr {
    Assignment(String, Box<Expr>),
    Function(String, Vec<Box<Expr>>),
    BinaryOp(Box<Expr>, Box<Expr>, BinaryOpCode),
    UnaryOp(Box<Expr>, UnaryOpCode),
    Variable(String),
    Num(i32),
}

#[derive(Debug)]
pub enum BinaryOpCode {
    Add, Sub, Mul, Div,
}

#[derive(Debug)]
pub enum UnaryOpCode {
    Plus, Minus
}
