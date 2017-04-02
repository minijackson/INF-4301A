#[derive(Debug,PartialEq,Eq)]
pub struct Exprs {
    pub exprs: Vec<Box<Expr>>,
    pub type_annotation: Type
}

#[derive(Debug,PartialEq,Eq)]
pub struct NaiveExprs {
    pub exprs: Vec<Box<NaiveExpr>>,
}

#[derive(Debug,PartialEq,Eq)]
pub struct Expr {
    pub kind: ExprKind,
    pub type_annotation: Type
}

#[derive(Debug,PartialEq,Eq)]
pub enum ExprKind {
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
pub enum NaiveExpr {
    Grouping(NaiveExprs),
    Let(Vec<NaiveBinding>, NaiveExprs),
    Function(String, Vec<Box<NaiveExpr>>),
    If(Box<NaiveExpr>, Box<NaiveExpr>, Box<NaiveExpr>),
    BinaryOp(Box<NaiveExpr>, Box<NaiveExpr>, BinaryOpCode),
    UnaryOp(Box<NaiveExpr>, UnaryOpCode),
    Variable(String),
    Num(i32),
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum BinaryOpCode {
    Add, Sub, Mul, Div,
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum UnaryOpCode {
    Plus, Minus
}

#[derive(Debug,PartialEq,Eq)]
pub struct NaiveBinding {
    pub variable: String,
    pub value: NaiveExpr
}

#[derive(Debug,PartialEq,Eq)]
pub struct Binding {
    pub variable: String,
    pub value: Expr
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Type {
    /* Void, */Integer, /* Float, Bool, String, Array, */
}
