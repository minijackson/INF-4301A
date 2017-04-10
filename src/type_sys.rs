use std::char;
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
            // TODO
            Str(_) => Err(ConversionError::new(Type::Str, Type::Bool, (0, 0))),
            Void => Err(ConversionError::new(Type::Void, Type::Bool, (0, 0))),
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
            // TODO
            Str(_) => Err(ConversionError::new(Type::Str, Type::Integer, (0, 0))),
            Void => Err(ConversionError::new(Type::Void, Type::Integer, (0, 0))),
        }
    }

    pub fn into_float(self) -> Result<Self, ConversionError> {
        use self::Value::*;

        match self {
            Integer(val) => Ok(Float(val as f32)),
            Float(_) => Ok(self),
            Bool(val) => Ok(Float(if val { 1f32 } else { 0f32 })),
            // TODO
            Str(_) => Err(ConversionError::new(Type::Str, Type::Float, (0, 0))),
            Void => Err(ConversionError::new(Type::Void, Type::Float, (0, 0))),
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
                         Some('x') => {
                             chars
                                 .by_ref()
                                 .take(2)
                                 .fold(0u8,
                                       |acc, c| acc * 16 + c.to_digit(16).unwrap() as u8) as char
                         }
                         Some('u') => {
                             let val = chars
                                 .by_ref()
                                 .take(4)
                                 .fold(0,
                                       |acc, c| acc * 16 + c.to_digit(16).unwrap());
                             char::from_u32(val).unwrap()
                         }
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
