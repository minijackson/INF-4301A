use ast::{Declaration, Span};
use type_sys::Type;

use itertools::Itertools;
use lalrpop_util;
use rustyline::error::ReadlineError;
use term;

use std::fmt;
use std::error::Error;
use std::io::{stderr, Write};

pub fn print_error<T>(filename: &str, input: &str, err: &T)
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

    for hint in &err.hints() {
        print_hints(input, hint);
    }
}

pub fn print_hints(input: &str, hint: &Hinter) {
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

fn extract_line(input: &str, pos: usize) -> (usize, &str) {
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
                 message: format!("Got a `{:?}` here", self.from),
             }]
    }
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Unnatural conversion from `{:?}` to `{:?}`",
               self.from,
               self.to)
    }
}

impl Error for ConversionError {
    fn description(&self) -> &str {
        "conversion error"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlreadyDeclaredError {
    pub name: String,
    pub span: Span,
    // TODO: make that a reference
    pub orig_declaration: Declaration,
}

impl AlreadyDeclaredError {
    pub fn new(name: String, orig_declaration: Declaration, span: Span) -> Self {
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
                 span: self.orig_declaration.span(),
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
    VoidVarDeclaration(VoidVarDeclartionError),
    Conversion(ConversionError),
    IncompatibleArmTypes(IncompatibleArmTypesError),
    NoSuchSignature(NoSuchSignatureError),
    UnboundedVar(UnboundedVarError),
    AlreadyDeclared(AlreadyDeclaredError),
    UndefinedFunction(UndefinedFunctionError),
    UntypedEmptyArray(UntypedEmptyArrayError),
    InconsistentArrayTyping(InconsistentArrayTypingError),
}

impl Hint for TypeCheckError {
    fn hints(&self) -> Vec<Hinter> {
        use self::TypeCheckError::*;

        match *self {
            MismatchedTypes(ref err) => err.hints(),
            VoidVarDeclaration(ref err) => err.hints(),
            Conversion(ref err) => err.hints(),
            IncompatibleArmTypes(ref err) => err.hints(),
            NoSuchSignature(ref err) => err.hints(),
            UnboundedVar(ref err) => err.hints(),
            AlreadyDeclared(ref err) => err.hints(),
            UndefinedFunction(ref err) => err.hints(),
            UntypedEmptyArray(ref err) => err.hints(),
            InconsistentArrayTyping(ref err) => err.hints(),
        }
    }
}

impl fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TypeCheckError::*;

        match *self {
            MismatchedTypes(ref err) => write!(f, "{}", err),
            VoidVarDeclaration(ref err) => write!(f, "{}", err),
            Conversion(ref err) => write!(f, "{}", err),
            IncompatibleArmTypes(ref err) => write!(f, "{}", err),
            NoSuchSignature(ref err) => write!(f, "{}", err),
            UnboundedVar(ref err) => write!(f, "{}", err),
            AlreadyDeclared(ref err) => write!(f, "{}", err),
            UndefinedFunction(ref err) => write!(f, "{}", err),
            UntypedEmptyArray(ref err) => write!(f, "{}", err),
            InconsistentArrayTyping(ref err) => write!(f, "{}", err),
        }
    }
}

impl Error for TypeCheckError {
    fn description(&self) -> &str {
        use self::TypeCheckError::*;

        match *self {
            MismatchedTypes(ref err) => err.description(),
            VoidVarDeclaration(ref err) => err.description(),
            Conversion(ref err) => err.description(),
            IncompatibleArmTypes(ref err) => err.description(),
            NoSuchSignature(ref err) => err.description(),
            UnboundedVar(ref err) => err.description(),
            AlreadyDeclared(ref err) => err.description(),
            UndefinedFunction(ref err) => err.description(),
            UntypedEmptyArray(ref err) => err.description(),
            InconsistentArrayTyping(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        use self::TypeCheckError::*;

        match *self {
            MismatchedTypes(ref err) => Some(err),
            VoidVarDeclaration(ref err) => Some(err),
            Conversion(ref err) => Some(err),
            IncompatibleArmTypes(ref err) => Some(err),
            NoSuchSignature(ref err) => Some(err),
            UnboundedVar(ref err) => Some(err),
            AlreadyDeclared(ref err) => Some(err),
            UndefinedFunction(ref err) => Some(err),
            UntypedEmptyArray(ref err) => Some(err),
            InconsistentArrayTyping(ref err) => Some(err),
        }
    }
}

impl From<MismatchedTypesError> for TypeCheckError {
    fn from(err: MismatchedTypesError) -> Self {
        TypeCheckError::MismatchedTypes(err)
    }
}

impl From<VoidVarDeclartionError> for TypeCheckError {
    fn from(err: VoidVarDeclartionError) -> Self {
        TypeCheckError::VoidVarDeclaration(err)
    }
}

impl From<ConversionError> for TypeCheckError {
    fn from(err: ConversionError) -> Self {
        TypeCheckError::Conversion(err)
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

impl From<UntypedEmptyArrayError> for TypeCheckError {
    fn from(err: UntypedEmptyArrayError) -> Self {
        TypeCheckError::UntypedEmptyArray(err)
    }
}

impl From<InconsistentArrayTypingError> for TypeCheckError {
    fn from(err: InconsistentArrayTypingError) -> Self {
        TypeCheckError::InconsistentArrayTyping(err)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MismatchedTypesError {
    pub expected: Type,
    pub got: Type,
    // TODO: make that a reference
    pub binding: Option<Declaration>,
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

    pub fn from_binding(binding: Declaration, expected: Type, got: Type, span: Span) -> Self {
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
            let binding_type = match *binding {
                Declaration::Variable(_) => "Variable",
                Declaration::Argument(_) => "Argument",
                Declaration::Function(_) => "Function",
            };

            res.push(Hinter {
                type_: HinterType::Info,
                span: binding.span(),
                message: format!("{} `{}` is of type `{:?}` as deduced by this declaration",
                                 binding_type,
                                 binding.name(),
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
pub struct VoidVarDeclartionError {
    pub name: String,
    pub value_span: Span,
}

impl VoidVarDeclartionError {
    pub fn new(name: String, value_span: Span) -> Self {
        VoidVarDeclartionError { name, value_span }
    }
}

impl Hint for VoidVarDeclartionError {
    fn hints(&self) -> Vec<Hinter> {
        vec![Hinter {
                 type_: HinterType::Error,
                 span: self.value_span,
                 message: "Got a `Void` here".to_string(),
             }]
    }
}

impl fmt::Display for VoidVarDeclartionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "variable `{}` declared as `Void`", self.name)
    }
}

impl Error for VoidVarDeclartionError {
    fn description(&self) -> &str {
        "void variable"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncompatibleArmTypesError {
    pub expected: Type,
    pub got: Type,
    pub true_branch_span: Span,
    pub false_branch_span: Span,
}

impl IncompatibleArmTypesError {
    pub fn new(expected: Type, got: Type, true_branch_span: Span, false_branch_span: Span) -> Self {
        IncompatibleArmTypesError {
            expected,
            got,
            true_branch_span,
            false_branch_span,
        }
    }
}

impl Hint for IncompatibleArmTypesError {
    fn hints(&self) -> Vec<Hinter> {
        vec![Hinter {
                 type_: HinterType::Warning,
                 span: self.false_branch_span,
                 message: format!("Resolved as a `{:?}`", self.got),
             },
             Hinter {
                 type_: HinterType::Warning,
                 span: self.true_branch_span,
                 message: format!("True branch resolved as `{:?}`", self.expected),
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

#[derive(Debug, Clone, PartialEq)]
pub struct UntypedEmptyArrayError {
    span: Span,
}

impl UntypedEmptyArrayError {
    pub fn new(span: Span) -> Self {
        UntypedEmptyArrayError { span }
    }
}

impl Hint for UntypedEmptyArrayError {
    fn hints(&self) -> Vec<Hinter> {
        vec![Hinter {
                 type_: HinterType::Error,
                 span: self.span,
                 message: format!("Add a type before this array"),
             }]
    }
}

impl fmt::Display for UntypedEmptyArrayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "empty array must be type annotated")
    }
}

impl Error for UntypedEmptyArrayError {
    fn description(&self) -> &str {
        "untyped empty array"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayTypeDecl {
    Explicit(Span),
    FirstElem(Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InconsistentArrayTypingError {
    pub expected: Type,
    pub got: Type,
    pub argument_id: usize,
    pub span: Span,
    pub type_decl: ArrayTypeDecl,
}

impl Hint for InconsistentArrayTypingError {
    fn hints(&self) -> Vec<Hinter> {
        vec![Hinter {
                 type_: HinterType::Error,
                 span: self.span,
                 message: format!("Got a `{:?}` here", self.got),
             },

             match self.type_decl {
                 ArrayTypeDecl::Explicit(span) => {
                     Hinter {
                         type_: HinterType::Info,
                         span,
                         message: format!("The element type was declared here"),
                     }
                 }
                 ArrayTypeDecl::FirstElem(span) => {
                     Hinter {
                         type_: HinterType::Info,
                         span,
                         message: format!("The element type: `{:?}` was deduced from here",
                                          self.expected),
                     }
                 }
             }]
    }
}

impl fmt::Display for InconsistentArrayTypingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "element {} is not consistent with the type of element inside this array: `{:?}`",
               self.argument_id,
               self.expected)
    }
}

impl Error for InconsistentArrayTypingError {
    fn description(&self) -> &str {
        "inconsistent array typing"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

pub type OrigPopParseError<'a> = lalrpop_util::ParseError<usize, (usize, &'a str), UserParseError>;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError<'a> {
    InvalidToken { location: usize },

    UnrecognizedToken {
        token: Option<(usize, (usize, &'a str), usize)>,
        expected: Vec<String>,
    },

    ExtraToken { token: (usize, (usize, &'a str), usize), },

    User { error: UserParseError },
}

impl<'a> Hint for ParseError<'a> {
    fn hints(&self) -> Vec<Hinter> {
        use self::ParseError::*;

        vec![Hinter {
                 type_: HinterType::Error,
                 span: match *self {
                     InvalidToken { location } => Span(location, location + 1),
                     UnrecognizedToken { token: Some((start, _, end)), .. } |
                     ExtraToken { token: (start, _, end) } => Span(start, end),
                     UnrecognizedToken { token: None, .. } => return vec![],
                     User { ref error } => return error.hints(),
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
            lalrpop_util::ParseError::User { error } => ParseError::User { error },
        }
    }
}

impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ParseError::*;

        match *self {
            InvalidToken { .. } => write!(f, "Invalid token"),
            UnrecognizedToken {
                ref token,
                ref expected,
            } => {
                match *token {
                    None => write!(f, "Unexpected EOF")?,
                    Some((_, (_, token), _)) => write!(f, "Unexpected \"{}\"", token)?,
                }
                if !expected.is_empty() {
                    write!(f,
                           ", expected one of: {}",
                           expected
                               .iter()
                               .map(|x| match x.as_str() {
                                        r##"r#"\"(?:[^\"\\\\]|\\\\.)*\""#"## => "string literal",
                                        r##"r#"[0-9]+"#"## => "integer literal",
                                        r##"r#"[0-9]+\\.[0-9]*"#"## => "float literal",
                                        r##"r#"[[:alpha:]][[:alnum:]_]*"#"## => "identifier",
                                        _ => x,
                                    })
                               .join(", "))?;
                }
                Ok(())
            }
            ExtraToken { token: (_, (_, token), _) } => write!(f, "Extra token `{}`", token),
            User { ref error } => write!(f, "{}", error),
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
            User { ref error } => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserParseError {
    IntegerOverflow { span: Span },
}

impl Hint for UserParseError {
    fn hints(&self) -> Vec<Hinter> {
        use self::UserParseError::*;

        vec![Hinter {
                 type_: HinterType::Error,
                 span: match *self {
                     IntegerOverflow { span } => span,
                 },
                 message: "inputted here".to_string(),
             }]
    }
}

impl fmt::Display for UserParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::UserParseError::*;

        write!(f,
               "{}",
               match *self {
                   IntegerOverflow { .. } => "Integer overflow",
               })
    }
}

impl Error for UserParseError {
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
            Readline(_) => vec![],
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
