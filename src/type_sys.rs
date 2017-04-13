use ast::Span;
use error::ConversionError;

use std::char;
use std::fmt;

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum Type {
    Void,
    Integer,
    Float,
    Bool,
    Str, /* Array, */
}

impl Type {
    pub fn is_convertible_to(&self, dest: Type) -> bool {
        use self::Type::*;

        match *self {
            Void => {
                match dest {
                    Void => true,
                    _ => false,
                }
            }
            Integer => {
                match dest {
                    Void => true,
                    Integer => true,
                    Float => true,
                    Bool => true,
                    Str => true,
                }
            }
            Float => {
                match dest {
                    Void => true,
                    Integer => true,
                    Float => true,
                    Bool => true,
                    Str => true,
                }
            }
            Bool => {
                match dest {
                    Void => true,
                    Integer => false,
                    Float => false,
                    Bool => true,
                    Str => true,
                }
            }
            Str => {
                match dest {
                    Void => true,
                    // TODO
                    Integer => false,
                    Float => false,
                    Bool => false,
                    Str => true,
                }
            }
        }
    }
}

#[derive(Debug,Clone,PartialEq)]
pub enum Value {
    Void,
    Integer(i64),
    Float(f64),
    Bool(bool),
    Str(String), /* Array, */
}

impl Value {
    pub fn truthy(&self) -> Result<bool, ConversionError> {
        use self::Value::*;

        match *self {
            Integer(0) => Ok(false),
            Integer(_) => Ok(true),
            Float(0f64) => Ok(false),
            Float(_) => Ok(true),
            Bool(false) => Ok(false),
            Bool(true) => Ok(true),
            // TODO
            Str(_) => Err(ConversionError::new(Type::Str, Type::Bool, Span(0, 0))),
            Void => Err(ConversionError::new(Type::Void, Type::Bool, Span(0, 0))),
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

    pub fn into(self, dest: Type) -> Self {
        use self::Value::*;

        match self {
            Void => {
                match dest {
                    Type::Void => Void,
                    _ => panic!("Unnatural conversion at runtime"),
                }
            }
            Integer(val) => {
                match dest {
                    Type::Void => Void,
                    Type::Integer => Integer(val),
                    Type::Float => Float(val as f64),
                    Type::Bool => Bool(if val == 1 { true } else { false }),
                    Type::Str => Str(val.to_string()),
                }
            }
            Float(val) => {
                match dest {
                    Type::Void => Void,
                    Type::Integer => Integer(val as i64),
                    Type::Float => Float(val),
                    Type::Bool => Bool(if val == 1f64 { true } else { false }),
                    Type::Str => Str(val.to_string()),
                }
            }
            Bool(val) => {
                match dest {
                    Type::Void => Void,
                    Type::Integer => panic!("Unnatural conversion at runtime"),
                    Type::Float => panic!("Unnatural conversion at runtime"),
                    Type::Bool => Bool(val),
                    Type::Str => Str(val.to_string()),
                }
            }
            Str(val) => {
                match dest {
                    Type::Void => Value::Void,
                    // TODO
                    Type::Integer => panic!("Unnatural conversion at runtime"),
                    Type::Float => panic!("Unnatural conversion at runtime"),
                    Type::Bool => panic!("Unnatural conversion at runtime"),
                    Type::Str => Str(val),
                }
            }
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
                                 .fold(0, |acc, c| acc * 16 + c.to_digit(16).unwrap());
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
