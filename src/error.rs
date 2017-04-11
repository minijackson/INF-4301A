use ast::{Binding, Span};
use type_sys::Type;

use itertools::Itertools;
use lalrpop_util;
use rustyline::error::ReadlineError;
use term;

use std::fmt;
use std::error::Error;
use std::io::{stderr, Write};

pub fn print_error<T>(filename: &str, input: &str, err: T)
    where T: Error + Hint
{
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

    for hint in err.hints().into_iter() {
        print_hints(input, hint);
    }
}

pub fn print_hints(input: &str, hint: Hinter) {
    let mut t = term::stderr().unwrap();

    let Span(mut start, end) = hint.span;
    let span_len = end - start;
    let (offset, line) = extract_line(input, start);

    start -= offset;

    writeln!(&mut stderr(), "  {}", line).unwrap();

    let color = match hint.type_ {
        HinterType::Error => term::color::BRIGHT_RED,
        HinterType::Warning => term::color::BRIGHT_YELLOW,
        HinterType::Info => term::color::BLUE,
    };

    t.fg(color).unwrap();
    t.attr(term::Attr::Bold).unwrap();

    writeln!(&mut stderr(),
             "  {}{}--- {}\n",
             " ".repeat(start),
             "^".repeat(span_len),
             hint.message)
            .unwrap();

    t.reset().unwrap();
}

fn extract_line<'a>(input: &'a str, pos: usize) -> (usize, &'a str) {
    let mut start = pos;
    let mut end = pos;

    while let Some(ch) = input.chars().nth(start) {
        if start == 0 {
            break;
        }
        if ch == '\n' {
            start += 1;
            break;
        }
        start -= 1;
    }

    while let Some(ch) = input.chars().nth(end) {
        if ch == '\n' {
            break;
        }
        end += 1;
    }

    (start, &input[start..end])
}

pub struct Hinter {
    pub type_: HinterType,
    pub span: Span,
    pub message: String,
}

pub enum HinterType {
    Error,
    Warning,
    Info,
}

pub trait Hint {
    fn hints(&self) -> Vec<Hinter>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConversionError {
    pub from: Type,
    pub to: Type,
    pub span: Span,
}

impl ConversionError {
    pub fn new(from: Type, to: Type, span: Span) -> Self {
        ConversionError { from, to, span }
    }
}

impl Hint for ConversionError {
    fn hints(&self) -> Vec<Hinter> {
        vec![Hinter {
                 type_: HinterType::Error,
                 span: self.span,
                 message: format!("Got a `{:?}` here", self.to),
             }]
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
    pub span: Span,
    // TODO: make that a reference
    pub orig_declaration: Binding,
}

impl AlreadyDeclaredError {
    pub fn new(name: String, orig_declaration: Binding, span: Span) -> Self {
        AlreadyDeclaredError {
            name,
            orig_declaration,
            span,
        }
    }
}

impl Hint for AlreadyDeclaredError {
    fn hints(&self) -> Vec<Hinter> {
        vec![Hinter {
                 type_: HinterType::Error,
                 span: self.span,
                 message: "Redeclared here".to_string(),
             },
             Hinter {
                 type_: HinterType::Info,
                 span: self.orig_declaration.span,
                 message: "First declared here".to_string(),
             }]
    }
}

impl fmt::Display for AlreadyDeclaredError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    pub span: Span,
}

impl NoSuchSignatureError {
    pub fn new(func_name: String, arg_types: Vec<Type>, span: Span) -> Self {
        NoSuchSignatureError {
            func_name,
            arg_types,
            span,
        }
    }
}

impl Hint for NoSuchSignatureError {
    fn hints(&self) -> Vec<Hinter> {
        vec![Hinter {
                 type_: HinterType::Error,
                 span: self.span,
                 message: "Used here".to_string(),
             }]
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
    pub span: Span,
}

impl UnboundedVarError {
    pub fn new(name: String, span: Span) -> Self {
        UnboundedVarError { name, span }
    }
}

impl Hint for UnboundedVarError {
    fn hints(&self) -> Vec<Hinter> {
        vec![Hinter {
                 type_: HinterType::Error,
                 span: self.span,
                 message: "Used here".to_string(),
             }]
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
    pub span: Span,
}

impl UndefinedFunctionError {
    pub fn new(name: String, span: Span) -> Self {
        UndefinedFunctionError { name, span }
    }
}

impl Hint for UndefinedFunctionError {
    fn hints(&self) -> Vec<Hinter> {
        vec![Hinter {
                 type_: HinterType::Error,
                 span: self.span,
                 message: "Used here".to_string(),
             }]
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

impl Hint for TypeCheckError {
    fn hints(&self) -> Vec<Hinter> {
        use self::TypeCheckError::*;

        match *self {
            MismatchedTypes(ref err) => err.hints(),
            IncompatibleArmTypes(ref err) => err.hints(),
            NoSuchSignature(ref err) => err.hints(),
            UnboundedVar(ref err) => err.hints(),
            AlreadyDeclared(ref err) => err.hints(),
            UndefinedFunction(ref err) => err.hints(),
        }
    }
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
    pub span: Span,
}

impl MismatchedTypesError {
    pub fn new(expected: Type, got: Type, span: Span) -> Self {
        MismatchedTypesError {
            expected,
            got,
            binding: None,
            span,
        }
    }

    pub fn from_binding(binding: Binding, expected: Type, got: Type, span: Span) -> Self {
        MismatchedTypesError {
            expected,
            got,
            binding: Some(binding),
            span,
        }
    }
}

impl Hint for MismatchedTypesError {
    fn hints(&self) -> Vec<Hinter> {
        let mut res = vec![Hinter {
                               type_: HinterType::Error,
                               span: self.span,
                               message: format!("Got a `{:?}` here", self.got),
                           }];

        if let Some(ref binding) = self.binding {
            res.push(Hinter {
                type_: HinterType::Info,
                span: binding.span,
                message: format!("Var `{}` is of type `{:?}` as deduced by this declaration",
                                 binding.variable,
                                 self.expected),
            });
        }

        res
    }
}

impl fmt::Display for MismatchedTypesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    pub span: Span,
}

impl IncompatibleArmTypesError {
    pub fn new(expected: Type, got: Type, span: Span) -> Self {
        IncompatibleArmTypesError {
            expected,
            got,
            span,
        }
    }
}

impl Hint for IncompatibleArmTypesError {
    fn hints(&self) -> Vec<Hinter> {
        vec![Hinter {
                 type_: HinterType::Error,
                 span: self.span,
                 message: format!("Resolved as a `{:?}`", self.got),
             }]
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

impl<'a> Hint for ParseError<'a> {
    fn hints(&self) -> Vec<Hinter> {
        use self::ParseError::*;

        vec![Hinter {
                 type_: HinterType::Error,
                 span: match *self {
                     InvalidToken { location } => Span(location, location + 1),
                     UnrecognizedToken {
                         token: Some((start, _, end)),
                         expected: _,
                     } => Span(start, end),
                     UnrecognizedToken {
                         token: None,
                         expected: _,
                     } => panic!("TODO"),
                     ExtraToken { token: (start, _, end) } => Span(start, end),
                 },
                 message: "Encountered here".to_string(),
             }]
    }
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
            InvalidToken { location: _ } => write!(f, "Invalid token"),
            UnrecognizedToken {
                ref token,
                ref expected,
            } => {
                match *token {
                    None => write!(f, "Unexpected EOF")?,
                    Some((_, (_, ref token), _)) => write!(f, "Unexpected \"{}\"", token)?,
                }
                if !expected.is_empty() {
                    write!(f, ", expected one of: {}", expected.iter().join(", "))?;
                }
                Ok(())
            }
            ExtraToken { token: (_, (_, ref token), _) } => write!(f, "Extra token `{}`", token),
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

impl<'a> Hint for REPLError<'a> {
    fn hints(&self) -> Vec<Hinter> {
        use self::REPLError::*;

        match *self {
            Readline(_) => panic!("TODO"),
            Parse(ref err) => err.hints(),
        }
    }
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
