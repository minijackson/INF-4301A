use ast::Span;
use error::ConversionError;

use itertools::Itertools;

use std::char;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone,PartialEq,Eq,Hash)]
pub enum Type {
    Void,
    Integer,
    Float,
    Bool,
    Str,
    Array(Box<Type>),
    Tuple(Vec<Type>),
}

impl Type {
    pub fn is_convertible_to(&self, dest: &Type) -> bool {
        use self::Type::*;

        match *self {
            Void => {
                match *dest {
                    Void => true,
                    _ => false,
                }
            }
            Integer | Float => {
                match *dest {
                    Void | Integer | Float | Bool | Str => true,
                    Array(_) | Tuple(_) => false,
                }
            }
            Bool => {
                match *dest {
                    Void | Bool | Str => true,
                    Integer | Float | Array(_) | Tuple(_) => false,
                }
            }
            Str => {
                match *dest {
                    Void | Str => true,
                    // TODO
                    Integer | Float | Bool | Array(_) | Tuple(_) => false,
                }
            }
            Array(ref my_type) => {
                match *dest {
                    Void => true,
                    Array(ref type_) => my_type.is_convertible_to(type_),
                    Integer | Float | Bool | Str | Tuple(_) => false,
                }
            }
            Tuple(ref my_types) => {
                match *dest {
                    Void => true,
                    Array(ref type_) => {
                        my_types
                            .iter()
                            .all(|my_type| my_type.is_convertible_to(type_))
                    }
                    Tuple(ref types) => {
                        my_types.len() == types.len() &&
                        my_types
                            .iter()
                            .zip(types)
                            .all(|(my_type, type_)| my_type.is_convertible_to(type_))
                    }
                    Integer | Float | Bool | Str => false,
                }
            }
        }
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Type::*;

        match *self {
            Void => write!(f, "Void"),
            Integer => write!(f, "Integer"),
            Float => write!(f, "Float"),
            Bool => write!(f, "Bool"),
            Str => write!(f, "Str"),
            Array(ref type_) => write!(f, "Array({:?})", type_),
            Tuple(ref types) => {
                write!(f, "Tuple(")?;

                if types.len() > 0 {

                    write!(f, "{:?}", types[0])?;

                    for type_ in types.iter().skip(1) {
                        write!(f, ", {:?}", type_)?;
                    }
                }

                write!(f, ")")
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
    Str(String),
    Array {
        element_type: Type,
        values: Vec<Value>,
    },
    Tuple {
        element_types: Vec<Type>,
        values: Vec<Value>,
    },
}

impl Value {
    pub fn truthy(&self) -> Result<bool, ConversionError> {
        use self::Value::*;

        match *self {
            Integer(0) | Float(0f64) | Bool(false) => Ok(false),
            Integer(_) | Float(_) | Bool(true) => Ok(true),
            Array { ref values, .. } => Ok(values.len() != 0),
            // TODO
            Str(_) => Err(ConversionError::new(Type::Str, Type::Bool, Span(0, 0))),
            Tuple { ref element_types, .. } => {
                Err(ConversionError::new(Type::Tuple(element_types.clone()),
                                         Type::Bool,
                                         Span(0, 0)))
            }
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
            Array { ref element_type, .. } => Type::Array(Box::new(element_type.clone())),
            Tuple { ref element_types, .. } => Type::Tuple(element_types.clone()),
        }
    }

    pub fn into(self, dest: &Type) -> Self {
        use self::Value::*;

        match self {
            Void => {
                match *dest {
                    Type::Void => Void,
                    _ => panic!("Unnatural conversion at runtime"),
                }
            }
            Integer(val) => {
                match *dest {
                    Type::Void => Void,
                    Type::Integer => Integer(val),
                    Type::Float => Float(val as f64),
                    Type::Bool => Bool(val != 0),
                    Type::Str => Str(val.to_string()),
                    Type::Array(_) | Type::Tuple(_) => panic!("Unnatural conversion at runtime"),
                }
            }
            Float(val) => {
                match *dest {
                    Type::Void => Void,
                    Type::Integer => Integer(val as i64),
                    Type::Float => Float(val),
                    Type::Bool => Bool(val != 0f64),
                    Type::Str => Str(val.to_string()),
                    Type::Array(_) | Type::Tuple(_) => panic!("Unnatural conversion at runtime"),
                }
            }
            Bool(val) => {
                match *dest {
                    Type::Void => Void,
                    Type::Bool => Bool(val),
                    Type::Str => Str(val.to_string()),
                    Type::Integer | Type::Float | Type::Array(_) | Type::Tuple(_) => panic!("Unnatural conversion at runtime"),
                }
            }
            Str(val) => {
                match *dest {
                    Type::Void => Value::Void,
                    Type::Str => Str(val),
                    // TODO
                    Type::Integer | Type::Float | Type::Bool | Type::Array(_) | Type::Tuple(_) => {
                        panic!("Unnatural conversion at runtime")
                    }
                }
            }
            Array {
                element_type,
                values,
            } => {
                match *dest {
                    Type::Void => Value::Void,
                    Type::Array(ref new_element_type) if **new_element_type == element_type => {
                        Value::Array {
                            element_type,
                            values,
                        }
                    }
                    Type::Array(ref new_element_type) => {
                        Value::Array {
                            element_type: *new_element_type.clone(),
                            values: values
                                .into_iter()
                                .map(|value| value.into(new_element_type))
                                .collect(),
                        }
                    }
                    Type::Integer | Type::Float | Type::Bool | Type::Str | Type::Tuple(_) => {
                        panic!("Unnatural conversion at runtime")
                    }
                }
            }
            Tuple {
                element_types,
                values,
            } => {
                match *dest {
                    Type::Void => Value::Void,
                    Type::Tuple(ref new_element_types) if *new_element_types == element_types => {
                        Value::Tuple {
                            element_types,
                            values,
                        }
                    }
                    Type::Tuple(ref new_element_types) => {
                        Value::Tuple {
                            element_types: new_element_types.clone(),
                            values: values
                                .into_iter()
                                .zip(new_element_types)
                                .map(|(value, new_element_type)| value.into(new_element_type))
                                .collect(),
                        }
                    }
                    Type::Array(ref new_element_type) => {
                        Value::Array {
                            element_type: (**new_element_type).clone(),
                            values: values
                                .into_iter()
                                .map(|value| value.into(new_element_type))
                                .collect(),
                        }
                    }
                    Type::Integer | Type::Float | Type::Bool | Type::Str => panic!("Unnatural conversion at runtime"),
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
            Float(ref value) => {
                if value.floor() == *value {
                    write!(f, "{}.", value)
                } else {
                    write!(f, "{}", value)
                }
            }
            Bool(ref value) => write!(f, "{}", value),
            Str(ref value) => write!(f, "{}", value),
            Array { ref values, .. } => write!(f, "[{}]", values.iter().join(", ")),
            Tuple { ref values, .. } => write!(f, "{{{}}}", values.iter().join(", ")),
            Void => write!(f, "nil"),
        }
    }
}

pub trait Match {
    fn match_with(&self, given_type: &Type, types: &HashMap<&str, Generic>) -> bool;
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum Generic {
    Builtin(Type),
    Abstract(AbstractType),
    Sum(SumType),
    Named(String),
    Any,
}

impl Match for Generic {
    fn match_with(&self, given_type: &Type, types: &HashMap<&str, Generic>) -> bool {
        use self::Generic::*;

        match *self {
            Builtin(ref builtin) if builtin == given_type => true,
            Builtin(_) => false,
            Abstract(ref abstr) => abstr.match_with(given_type, types),
            Sum(ref sum) => sum.match_with(given_type, types),
            Named(ref name) => {
                if let Some(ref candidate) = types.get(name.as_str()) {
                    candidate.match_with(given_type, types)
                } else {
                    false
                }
            }
            Any => true,
        }
    }
}

impl From<Type> for Generic {
    fn from(builtin: Type) -> Self {
        Generic::Builtin(builtin)
    }
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum AbstractType {
    Array(Box<Generic>),
}

impl Match for AbstractType {
    fn match_with(&self, given_type: &Type, types: &HashMap<&str, Generic>) -> bool {
        use self::AbstractType::*;

        match (self, given_type) {
            (&Array(ref el_type), &Type::Array(ref given_el_type)) => {
                (*el_type).match_with(&*given_el_type, types)
            }
            _ => false,
        }
    }
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct SumType {
    pub possibilities: Vec<Generic>,
}

impl Match for SumType {
    fn match_with(&self, given_type: &Type, types: &HashMap<&str, Generic>) -> bool {
        self.possibilities
            .iter()
            .find(|&candidate| candidate.match_with(given_type, types))
            .is_some()
    }
}

pub fn unescape_str(input: &str) -> String {
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
                                       |acc, c| acc * 16 + c.to_digit(16).unwrap() as u8) as
                             char
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
