use ast::Binding;
use type_sys::Type;

use itertools::Itertools;
use lalrpop_util;
use rustyline::error::ReadlineError;

use std::fmt;
use std::error::Error;
use std::io::{stderr, Write};

pub fn handle_error<'a>(filename: &str, err: Box<Error + 'a>) {
    writeln!(&mut stderr(), "While processing: {}\n{}: {}\n", filename, err.description(), err).unwrap();
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConversionError {
    pub from: Type,
    pub to: Type,
}

impl ConversionError {
    pub fn new(from: Type, to: Type) -> Self {
        ConversionError { from, to }
    }
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Unnatural conversion from {:?} to {:?}",
               self.from,
               self.to)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlreadyDeclaredError {
    pub name: String,
}

impl AlreadyDeclaredError {
    pub fn new(name: String) -> Self {
        AlreadyDeclaredError { name }
    }
}

impl fmt::Display for AlreadyDeclaredError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: say where
        write!(f, "Variable `{}` was already declared", self.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoSuchSignatureError {
    pub func_name: String,
    pub arg_types: Vec<Type>,
}

impl NoSuchSignatureError {
    pub fn new(func_name: String, arg_types: Vec<Type>) -> Self {
        NoSuchSignatureError {
            func_name,
            arg_types,
        }
    }
}

impl fmt::Display for NoSuchSignatureError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No such signature: {}{:?}", self.func_name, self.arg_types)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnboundedVarError {
    pub name: String,
}

impl UnboundedVarError {
    pub fn new(name: String) -> Self {
        UnboundedVarError { name }
    }
}

impl fmt::Display for UnboundedVarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unbounded variable: {}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UndefinedFunctionError {
    pub name: String,
}

impl UndefinedFunctionError {
    pub fn new(name: String) -> Self {
        UndefinedFunctionError { name }
    }
}

impl fmt::Display for UndefinedFunctionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Undefined function: {}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeCheckError {
    MismatchedTypes(MismatchedTypesError),
    IncompatibleArmTypes(IncompatibleArmTypesError),
    NoSuchSignature(NoSuchSignatureError),
    UnboundedVar(UnboundedVarError),
    AlreadyDeclared(AlreadyDeclaredError),
    UndefinedFunction(UndefinedFunctionError),
}

impl fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TypeCheckError::*;

        match *self {
            MismatchedTypes(ref err) => write!(f, "{}", err),
            IncompatibleArmTypes(ref err) => write!(f, "{}", err),
            NoSuchSignature(ref err) => write!(f, "{}", err),
            UnboundedVar(ref err) => write!(f, "{}", err),
            AlreadyDeclared(ref err) => write!(f, "{}", err),
            UndefinedFunction(ref err) => write!(f, "{}", err),
        }
    }
}

impl From<MismatchedTypesError> for TypeCheckError {
    fn from(err: MismatchedTypesError) -> Self {
        TypeCheckError::MismatchedTypes(err)
    }
}

impl From<IncompatibleArmTypesError> for TypeCheckError {
    fn from(err: IncompatibleArmTypesError) -> Self {
        TypeCheckError::IncompatibleArmTypes(err)
    }
}

impl From<NoSuchSignatureError> for TypeCheckError {
    fn from(err: NoSuchSignatureError) -> Self {
        TypeCheckError::NoSuchSignature(err)
    }
}

impl From<UnboundedVarError> for TypeCheckError {
    fn from(err: UnboundedVarError) -> Self {
        TypeCheckError::UnboundedVar(err)
    }
}

impl From<AlreadyDeclaredError> for TypeCheckError {
    fn from(err: AlreadyDeclaredError) -> Self {
        TypeCheckError::AlreadyDeclared(err)
    }
}

impl From<UndefinedFunctionError> for TypeCheckError {
    fn from(err: UndefinedFunctionError) -> Self {
        TypeCheckError::UndefinedFunction(err)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MismatchedTypesError {
    pub expected: Type,
    pub got: Type,
    // TODO: make that a reference
    pub binding: Option<Binding>,
}

impl MismatchedTypesError {
    pub fn new(expected: Type, got: Type) -> Self {
        MismatchedTypesError {
            expected,
            got,
            binding: None,
        }
    }

    pub fn from_binding(binding: Binding, expected: Type, got: Type) -> Self {
        MismatchedTypesError {
            expected,
            got,
            binding: Some(binding),
        }
    }
}

impl fmt::Display for MismatchedTypesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: say it with a binding if needed
        write!(f, "Mismatched types: expected `{:?}`, got `{:?}`", self.expected, self.got)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncompatibleArmTypesError {
    pub expected: Type,
    pub got: Type,
}

impl IncompatibleArmTypesError {
    pub fn new(expected: Type, got: Type) -> Self {
        IncompatibleArmTypesError { expected, got }
    }
}

impl fmt::Display for IncompatibleArmTypesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "If arms have incomptible types: true branch `{:?}`, false branch `{:?}`", self.expected, self.got)
    }
}

pub type OrigPopParseError<'a> = lalrpop_util::ParseError<usize, (usize, &'a str), ()>;

#[derive(Debug, Clone, PartialEq)]
pub enum PopParseError<'a> {
    InvalidToken {
        location: usize
    },

    UnrecognizedToken {
        token: Option<(usize, (usize, &'a str), usize)>,
        expected: Vec<String>
    },

    ExtraToken {
        token: (usize, (usize, &'a str), usize),
    },
}

impl<'a> From<OrigPopParseError<'a>> for PopParseError<'a> {
    fn from(err: OrigPopParseError<'a>) -> Self {
        match err {
            lalrpop_util::ParseError::InvalidToken { location } => PopParseError::InvalidToken { location },
            lalrpop_util::ParseError::UnrecognizedToken { token, expected } => PopParseError::UnrecognizedToken { token, expected },
            lalrpop_util::ParseError::ExtraToken { token } => PopParseError::ExtraToken { token },
            lalrpop_util::ParseError::User { error: _ } => unreachable!("User parse error encountered"),
        }
    }
}

impl<'a> fmt::Display for PopParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::PopParseError::*;

        match *self {
            InvalidToken { ref location } => write!(f, "Invalid token found at position {}", location),
            UnrecognizedToken { ref token, ref expected } => {
                match *token {
                    None => write!(f, "Unexpected EOF")?,
                    Some((ref start, (_, ref token), ref end)) => write!(f, "Unexpected \"{}\" found at {}-{}", token, start, end)?
                }
                if !expected.is_empty() {
                    write!(f, " expected one of: {}", expected.iter().join(", "))?;
                }
                Ok(())
            }
            ExtraToken { token: (ref start, (_, ref token), ref end) } => write!(f, "Extra token `{}` found at {}:{}", token, start, end),
        }
    }
}

impl <'a> Error for PopParseError<'a> {
    fn description(&self) -> &str {
        "parse error"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

#[derive(Debug)]
pub enum REPLError<'a> {
    Readline(ReadlineError),
    Parse(ParseError<'a>),
}

impl<'a> fmt::Display for REPLError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::REPLError::*;

        match *self {
            Readline(ref err) => write!(f, "{}", err),
            Parse(ref err) => write!(f, "{}", err),
        }
    }
}

impl<'a> Error for REPLError<'a> {
    fn description(&self) -> &str {
        use self::REPLError::*;

        match *self {
            Readline(ref err) => err.description(),
            Parse(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        use self::REPLError::*;

        match *self {
            Readline(ref err) => Some(err),
            Parse(ref err) => Some(err),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError<'a> {
    PopParse(PopParseError<'a>),
    UnfinishedExpression,
}

impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ParseError::*;
        match *self {
            UnfinishedExpression => unreachable!("UnfinishedExpression unhandled"),
            PopParse(ref err) => write!(f, "{}", err),
        }
    }
}

impl<'a> Error for ParseError<'a> {
    fn description(&self) -> &str {
        "parse error"
    }

    fn cause(&self) -> Option<&Error> {
        use self::ParseError::*;

        match *self {
            UnfinishedExpression => None,
            PopParse(ref err) => Some(err),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnfinishedExpressionError {
    pub partial_input: String,
}
