use std::fmt;

use error::ConversionError;

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum Type {
    Void,
    Integer,
    Float,
    Bool, /* String, Array, */
}

#[derive(Debug,Clone,PartialEq)]
pub enum Value {
    Void,
    Integer(i32),
    Float(f32),
    Bool(bool), /* String, Array, */
}

impl Value {
    pub fn truthy(&self) -> Result<bool, ConversionError> {
        use self::Value::*;

        match self {
            &Integer(0) => Ok(false),
            &Integer(_) => Ok(true),
            &Float(0f32) => Ok(false),
            &Float(_) => Ok(true),
            &Bool(false) => Ok(false),
            &Bool(true) => Ok(true),
            &Void => Err(ConversionError::new(Type::Void, Type::Bool)),
        }
    }

    pub fn get_type(&self) -> Type {
        use self::Value::*;

        match self {
            &Void => Type::Void,
            &Integer(_) => Type::Integer,
            &Float(_) => Type::Float,
            &Bool(_) => Type::Bool,
        }
    }

    pub fn into_int(self) -> Result<Self, ConversionError> {
        use self::Value::*;

        match self {
            Integer(_) => Ok(self),
            Float(val) => Ok(Integer(val as i32)),
            Bool(val) => Ok(Integer(val as i32)),
            Void => Err(ConversionError::new(Type::Void, Type::Integer)),
        }
    }

    pub fn into_float(self) -> Result<Self, ConversionError> {
        use self::Value::*;

        match self {
            Integer(val) => Ok(Float(val as f32)),
            Float(_) => Ok(self),
            Bool(val) => Ok(Float(if val { 1f32 } else { 0f32 })),
            Void => Err(ConversionError::new(Type::Void, Type::Float)),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Value::*;

        match self {
            &Integer(value) => write!(f, "{}", value),
            &Float(value) => write!(f, "{}", value),
            &Bool(value) => write!(f, "{}", value),
            &Void => Err(fmt::Error::default()),
        }
    }
}
