//! The module defining AST types and some AST related functions.

use type_sys;

use std::fmt;

/// Represents a span in the user code
///
/// Composed of the start index and end index of the pointed substring from the user code.
/// Useful for outputting nice error messages.
#[derive(Debug,Clone,Copy,PartialEq)]
pub struct Span(pub usize, pub usize);

/// An ordered aggregation of expressions.
#[derive(Debug,Clone,PartialEq)]
pub struct Exprs {
    pub exprs: Vec<Box<Expr>>,
}

/// A single expression
#[derive(Debug,Clone,PartialEq)]
pub enum Expr {
    /// A single expression representing multiple chained expressions
    Grouping(Exprs),

    /// A new scope.
    ///
    /// Composed of a list of variable declarations, a list of function declarations, and a list of
    /// expressions to execute in that new scope.
    Let(Vec<VariableDecl>, Vec<FunctionDecl>, Exprs),

    /// A single expression representing multiple chained expressions
    Assign {
        /// The name of the variable to assign
        name: String,
        /// The location of the name
        name_span: Span,
        /// The value to assign
        value: Box<Expr>,
        /// The location of the value
        value_span: Span,
    },

    /// A pattern match expression
    ///
    /// # Examples
    ///
    /// Provided x was defined and is of type Integer, this will return 1
    ///
    /// ```text
    /// match [x, 2, 3] := [1, 2, 3],
    /// x
    /// ```
    PatternMatch {
        /// The left hand side of the pattern match
        lhs: Box<Expr>,
        /// The location of the left hand side
        lhs_span: Span,
        /// The right hand side of the pattern match
        rhs: Box<Expr>,
        /// The location of the right hand side
        rhs_span: Span,
    },

    /// A function call
    Function {
        /// The name of the function
        name: String,
        /// Its arguments
        args: Vec<(Box<Expr>, Span)>,
        /// The location of the call
        span: Span,
    },

    /// An If statement (obviously)
    If {
        /// The condition
        cond: Box<Expr>,
        /// The location of the condition
        cond_span: Span,
        /// What to do if the condition is true
        true_branch: Box<Expr>,
        /// The location of the true branch
        true_branch_span: Span,
        /// What to do if the condition is false
        false_branch: Box<Expr>,
        /// The location of the false branch
        false_branch_span: Span,
    },

    /// A While loop (obviously)
    While {
        /// The condition
        cond: Box<Expr>,
        /// The location of the condition
        cond_span: Span,
        /// The body of the While loop
        expr: Box<Expr>,
    },

    /// A For loop (obviously)
    For {
        /// The name and initial value of the variable iterated over
        binding: Box<VariableDecl>,
        /// The upper bound for the iterated variable
        goal: Box<Expr>,
        /// The location of the upper bound
        goal_span: Span,
        /// The body of the For loop
        expr: Box<Expr>,
    },

    /// A binary operator
    BinaryOp {
        /// The left hand side of the operator
        lhs: Box<Expr>,
        /// The right hand side of the operator
        rhs: Box<Expr>,
        /// The operator
        op: BinaryOpCode,
        /// The location of the whole expression
        span: Span,
    },

    /// An unary operator
    UnaryOp {
        /// The body
        expr: Box<Expr>,
        /// The operator
        op: UnaryOpCode,
        /// The location of the whole expression
        span: Span,
    },

    /// A cast operation
    Cast {
        /// The expression to cast
        expr: Box<Expr>,
        /// The location of the expression
        expr_span: Span,
        /// The type to cast to
        dest: type_sys::Type,
    },

    /// A read of a variable
    Variable {
        /// The name of the variable
        name: String,
        /// The location of the read
        span: Span
    },

    /// An Array (obviously)
    Array {
        /// The values inside the array
        values: Vec<(Box<Expr>, Span)>,
        /// The type of this array's elements (if explicitly provided)
        ///
        /// If not explicitly provided, the type checker will deduce the element type and change
        /// this field.
        declared_type: Option<type_sys::Type>,
        /// The location of the explicit element type declaration
        declared_type_span: Option<Span>,
        /// The location of the whole expression
        span: Span,
    },

    /// A Tuple (obviously)
    ///
    /// Composed of a list of expressions
    Tuple(Vec<Box<Expr>>),

    /// A literal value
    Value(type_sys::Value),
}

/// Represents a binary operator
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
               match *self {
                   Add => "+",
                   Sub => "-",
                   Mul => "*",
                   Div => "/",

                   Lt => "<",
                   Le => "<=",
                   Gt => ">",
                   Ge => ">=",
                   Eq => "=",
                   Ne => "<>",
               })
    }
}

/// Represents an unary operator
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
               match *self {
                   Plus => "+",
                   Minus => "-",
               })
    }
}


/// Represents any declaration
#[derive(Debug,Clone,PartialEq)]
pub enum Declaration {
    /// A variable declaration (in a Let expression)
    Variable(VariableDecl),
    /// A function declaration (in a Let expression)
    Function(FunctionDecl),
    /// A argument declaration (in a function declaration)
    Argument(ArgumentDecl),
}

impl Declaration {
    /// Returns the name of the declaration (variable name, function name, argument name...)
    pub fn name(&self) -> &String {
        use self::Declaration::*;

        match *self {
            Variable(VariableDecl { ref name, .. }) |
            Function(FunctionDecl { ref name, .. }) |
            Argument(ArgumentDecl { ref name, .. }) => name,
        }
    }

    /// Returns the location of the declaration
    pub fn span(&self) -> Span {
        use self::Declaration::*;

        match *self {
            Variable(VariableDecl { span, .. }) |
            Function(FunctionDecl { signature_span: span, .. }) |
            Argument(ArgumentDecl { span, .. }) => span,
        }
    }
}

/// Represents a variable declaration
#[derive(Debug,Clone,PartialEq)]
pub struct VariableDecl {
    /// The name of the variable
    pub name: String,
    /// The value of the variable
    pub value: Expr,
    /// The location of the declaration
    pub span: Span,
    /// The location of the assigned value
    pub value_span: Span,
}

/// Represents a function declaration
#[derive(Debug,Clone,PartialEq)]
pub struct FunctionDecl {
    /// The name of the function
    pub name: String,
    /// The arguments of the function
    pub args: Vec<ArgumentDecl>,
    /// The return type
    pub return_type: type_sys::Type,
    /// The location of the whole signature
    pub signature_span: Span,
    /// The body of the function
    pub body: Box<Expr>,
    /// The location of the body of the function
    pub body_span: Span,
}

impl FunctionDecl {
    /// Get the return type of the function provided a list of arguments
    ///
    /// Returns None if the given arguments are not valid for this function
    pub fn return_type(&self, arg_types: &[type_sys::Type]) -> Option<&type_sys::Type> {
        if self.args.len() == arg_types.len() &&
           arg_types
               .iter()
               .zip(&self.args)
               .all(|(type_got, &ArgumentDecl { ref type_, .. })| type_got == type_) {
            Some(&self.return_type)
        } else {
            None
        }
    }
}

/// Represents an argument declaration
#[derive(Debug,Clone,PartialEq)]
pub struct ArgumentDecl {
    /// The argument name
    pub name: String,
    /// The argument type
    pub type_: type_sys::Type,
    /// The location of the argument declaration
    pub span: Span,
}
