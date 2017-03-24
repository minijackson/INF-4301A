#[derive(Debug)]
pub struct Exprs {
    pub exprs: Vec<Box<Expr>>
}

#[derive(Debug)]
pub enum Expr {
    Assignment(String, Box<Expr>),
    Function(String, Vec<Box<Expr>>),
    BinaryOp(Box<Expr>, Box<Expr>, OpCode),
    Variable(String),
    Num(i32),
}

#[derive(Debug)]
pub enum OpCode {
    Add, Sub, Mul, Div,
}
