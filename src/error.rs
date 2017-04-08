use ast::Binding;
use type_sys::Type;

use std::fmt;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct AlreadyDeclaredError {
    pub name: String,
}

impl AlreadyDeclaredError {
    pub fn new(name: String) -> Self {
        AlreadyDeclaredError { name }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct UnboundedVarError {
    pub name: String,
}

impl UnboundedVarError {
    pub fn new(name: String) -> Self {
        UnboundedVarError { name }
    }
}

#[derive(Debug)]
pub struct UndefinedFunctionError {
    pub name: String,
}

impl UndefinedFunctionError {
    pub fn new(name: String) -> Self {
        UndefinedFunctionError { name }
    }
}

#[derive(Debug)]
pub enum TypeCheckError {
    MismatchedTypes(MismatchedTypesError),
    IncompatibleArmTypes(IncompatibleArmTypesError),
    NoSuchSignature(NoSuchSignatureError),
    UnboundedVar(UnboundedVarError),
    AlreadyDeclared(AlreadyDeclaredError),
    UndefinedFunction(UndefinedFunctionError),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct IncompatibleArmTypesError {
    pub expected: Type,
    pub got: Type,
}

impl IncompatibleArmTypesError {
    pub fn new(expected: Type, got: Type) -> Self {
        IncompatibleArmTypesError { expected, got }
    }
}
