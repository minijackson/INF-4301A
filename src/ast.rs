#[derive(Debug)]
pub enum Expr {
    Function(String, Vec<Box<Expr>>),
    BinaryOp(Box<Expr>, Box<Expr>, OpCode),
    Num(i32),
}

#[derive(Debug)]
pub enum OpCode {
    Add, Sub, Mul, Div,
}
