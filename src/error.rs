use ast::Binding;
use type_sys::Type;

use itertools::Itertools;
use lalrpop_util;
use rustyline::error::ReadlineError;
use term;

use std::fmt;
use std::error::Error;
use std::io::{stderr, Write};

pub fn handle_error<'a>(filename: &str, err: Box<Error + 'a>) {
    let mut t = term::stderr().unwrap();
    t.fg(term::color::BRIGHT_RED).unwrap();
    t.attr(term::Attr::Bold).unwrap();
    writeln!(&mut stderr(),
             "While processing \"{}\"\n{}: {}\n",
             filename,
             err.description(),
             err)
            .unwrap();
    t.reset().unwrap();
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
        write!(f, "variable `{}` was already declared", self.name)
    }
}

impl Error for AlreadyDeclaredError {
    fn description(&self) -> &str {
        "multiple declaration"
    }

    fn cause(&self) -> Option<&Error> {
        None
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
        // TODO: provide alternatives
        write!(f, "{}{:?}", self.func_name, self.arg_types)
    }
}

impl Error for NoSuchSignatureError {
    fn description(&self) -> &str {
        "no such signature"
    }

    fn cause(&self) -> Option<&Error> {
        None
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
        write!(f, "\"{}\"", self.name)
    }
}

impl Error for UnboundedVarError {
    fn description(&self) -> &str {
        "unbounded variable"
    }

    fn cause(&self) -> Option<&Error> {
        None
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
        write!(f, "\"{}\"", self.name)
    }
}

impl Error for UndefinedFunctionError {
    fn description(&self) -> &str {
        "undefined function"
    }

    fn cause(&self) -> Option<&Error> {
        None
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

impl Error for TypeCheckError {
    fn description(&self) -> &str {
        use self::TypeCheckError::*;

        match *self {
            MismatchedTypes(ref err) => err.description(),
            IncompatibleArmTypes(ref err) => err.description(),
            NoSuchSignature(ref err) => err.description(),
            UnboundedVar(ref err) => err.description(),
            AlreadyDeclared(ref err) => err.description(),
            UndefinedFunction(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        use self::TypeCheckError::*;

        match *self {
            MismatchedTypes(ref err) => Some(err),
            IncompatibleArmTypes(ref err) => Some(err),
            NoSuchSignature(ref err) => Some(err),
            UnboundedVar(ref err) => Some(err),
            AlreadyDeclared(ref err) => Some(err),
            UndefinedFunction(ref err) => Some(err),
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
        write!(f, "expected `{:?}`, got `{:?}`", self.expected, self.got)
    }
}

impl Error for MismatchedTypesError {
    fn description(&self) -> &str {
        "mismatched types"
    }

    fn cause(&self) -> Option<&Error> {
        None
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
        write!(f,
               "if arms have incomptible types: true branch `{:?}`, false branch `{:?}`",
               self.expected,
               self.got)
    }
}

impl Error for IncompatibleArmTypesError {
    fn description(&self) -> &str {
        "incompatible arm types"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

pub type OrigPopParseError<'a> = lalrpop_util::ParseError<usize, (usize, &'a str), ()>;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError<'a> {
    InvalidToken { location: usize },

    UnrecognizedToken {
        token: Option<(usize, (usize, &'a str), usize)>,
        expected: Vec<String>,
    },

    ExtraToken { token: (usize, (usize, &'a str), usize), },
}

impl<'a> From<OrigPopParseError<'a>> for ParseError<'a> {
    fn from(err: OrigPopParseError<'a>) -> Self {
        match err {
            lalrpop_util::ParseError::InvalidToken { location } => {
                ParseError::InvalidToken { location }
            }
            lalrpop_util::ParseError::UnrecognizedToken { token, expected } => {
                ParseError::UnrecognizedToken { token, expected }
            }
            lalrpop_util::ParseError::ExtraToken { token } => ParseError::ExtraToken { token },
            lalrpop_util::ParseError::User { error: _ } => unreachable!("User parse error encountered"),
        }
    }
}

impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ParseError::*;

        match *self {
            InvalidToken { ref location } => {
                write!(f, "Invalid token found at position {}", location)
            }
            UnrecognizedToken {
                ref token,
                ref expected,
            } => {
                match *token {
                    None => write!(f, "Unexpected EOF")?,
                    Some((ref start, (_, ref token), ref end)) => {
                        write!(f, "Unexpected \"{}\" found at {}-{}", token, start, end)?
                    }
                }
                if !expected.is_empty() {
                    write!(f, ", expected one of: {}", expected.iter().join(", "))?;
                }
                Ok(())
            }
            ExtraToken { token: (ref start, (_, ref token), ref end) } => {
                write!(f, "Extra token `{}` found at {}:{}", token, start, end)
            }
        }
    }
}

impl<'a> Error for ParseError<'a> {
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
