use std::fmt;

use error::ConversionError;

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum Type {
    Void,
    Integer,
    Float,
    Bool,
    Str, /* Array, */
}

#[derive(Debug,Clone,PartialEq)]
pub enum Value {
    Void,
    Integer(i32),
    Float(f32),
    Bool(bool),
    Str(String), /* Array, */
}

impl Value {
    pub fn truthy(&self) -> Result<bool, ConversionError> {
        use self::Value::*;

        match *self {
            Integer(0) => Ok(false),
            Integer(_) => Ok(true),
            Float(0f32) => Ok(false),
            Float(_) => Ok(true),
            Bool(false) => Ok(false),
            Bool(true) => Ok(true),
            Str(_) => Err(ConversionError::new(Type::Str, Type::Bool)),
            Void => Err(ConversionError::new(Type::Void, Type::Bool)),
        }
    }

    pub fn get_type(&self) -> Type {
        use self::Value::*;

        match *self {
            Void => Type::Void,
            Integer(_) => Type::Integer,
            Float(_) => Type::Float,
            Bool(_) => Type::Bool,
            Str(_) => Type::Str,
        }
    }

    pub fn into_int(self) -> Result<Self, ConversionError> {
        use self::Value::*;

        match self {
            Integer(_) => Ok(self),
            Float(val) => Ok(Integer(val as i32)),
            Bool(val) => Ok(Integer(val as i32)),
            Str(_) => Err(ConversionError::new(Type::Str, Type::Integer)),
            Void => Err(ConversionError::new(Type::Void, Type::Integer)),
        }
    }

    pub fn into_float(self) -> Result<Self, ConversionError> {
        use self::Value::*;

        match self {
            Integer(val) => Ok(Float(val as f32)),
            Float(_) => Ok(self),
            Bool(val) => Ok(Float(if val { 1f32 } else { 0f32 })),
            Str(_) => Err(ConversionError::new(Type::Str, Type::Float)),
            Void => Err(ConversionError::new(Type::Void, Type::Float)),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Value::*;

        match *self {
            Integer(ref value) => write!(f, "{}", value),
            Float(ref value) => write!(f, "{}", value),
            Bool(ref value) => write!(f, "{}", value),
            Str(ref value) => write!(f, "{}", value),
            Void => Err(fmt::Error::default()),
        }
    }
}

pub fn unescape_str(input: String) -> String {
    let mut res = String::with_capacity(input.len());

    let mut chars = input.chars();

    while let Some(ch) = chars.next() {
        res.push(if ch != '\\' {
                     ch
                 } else {
                     match chars.next() {
                         Some('n') => '\n',
                         Some('r') => '\r',
                         Some('t') => '\t',
                         Some(ch) => ch,
                         None => panic!("Un-ended string"),
                     }
                 });
    }

    res
}
