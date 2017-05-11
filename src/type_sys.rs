//! The type system
//!
//! This module enumerates the available types, the values ("runtime typed" containers) and
//! functions that manipulate these types / values.

use itertools::Itertools;

use std::char;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

/// A Type (really?!)
#[derive(Clone,PartialEq,Eq,Hash)]
pub enum Type {
    /// The Void type
    Void,
    /// The Integer type
    Integer,
    /// The Float type
    Float,
    /// The Bool type
    Bool,
    /// The Str type
    Str,
    /// The Array type
    Array(Box<Type>),
    /// The Tuple type
    Tuple(Vec<Type>),
}

impl Type {
    /// Returns true if the current type may be checked for a truthy value (inside a if or a while)
    pub fn may_truthy(&self) -> bool {
        use self::Type::*;

        match *self {
            Integer | Float | Bool | Array(_) => true,
            _ => false,
        }
    }

    /// Returns true if the current type is convertible to a given type
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

                if !types.is_empty() {

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

/// A Value (...)
#[derive(Debug,Clone,PartialEq)]
pub enum Value {
    /// The Void value
    Void,
    /// The Integer value
    Integer(i64),
    /// The Float value
    Float(f64),
    /// The Bool value
    Bool(bool),
    /// The Str value
    Str(String),
    /// The Array value
    Array {
        /// The type of this array's elements
        element_type: Type,
        /// The value of this array's elements
        values: Vec<Value>,
    },
    /// The Tuple value
    Tuple {
        /// The types of this tuple's elements
        element_types: Vec<Type>,
        /// The value of this tuple's elements
        values: Vec<Value>,
    },
}

impl Value {
    /// Returns true if the current value is truthy (inside a if or a while)
    pub fn truthy(&self) -> bool {
        use self::Value::*;

        match *self {
            Integer(0) | Float(0f64) | Bool(false) => false,
            Integer(_) | Float(_) | Bool(true) => true,
            Array { ref values, .. } => !values.is_empty(),
            _ => panic!("Invalid value truthy-checked"),
        }
    }

    /// Get the type of the current value
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

    /// Convert the current value to another type
    ///
    /// Consumes the value.
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

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use self::Value::*;

        match (self, other) {
            (&Integer(lhs), &Integer(rhs)) => lhs.partial_cmp(&rhs),
            (&Float(lhs), &Float(rhs)) => lhs.partial_cmp(&rhs),
            (&Bool(lhs), &Bool(rhs)) => lhs.partial_cmp(&rhs),
            (&Str(ref lhs), &Str(ref rhs)) => lhs.partial_cmp(rhs),
            (&Array { values: ref lhs, .. }, &Array { values: ref rhs, .. }) => lhs.partial_cmp(rhs),
            (&Tuple { values: ref lhs, .. }, &Tuple { values: ref rhs, .. }) => lhs.partial_cmp(rhs),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Value::*;
        use std::f64::EPSILON;

        match *self {
            Integer(ref value) => write!(f, "{}", value),
            Float(ref value) => {
                if (value.floor() - *value).abs() < EPSILON {
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

/// A trait that tells if a given generic type match with a concrete type
pub trait Match {
    fn match_with(&self, given_type: &Type, types: &HashMap<&str, Generic>) -> bool;
}

/// A generic type
#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum Generic {
    /// The concrete type variant
    Builtin(Type),
    /// An abstract type (for parametric types like Arrays or Tuples)
    Abstract(AbstractType),
    /// An sum type (also called variant)
    Sum(SumType),
    /// An named type (see the [`env`](../env/index.html) module)
    Named(String),
    /// Any type
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
                if let Some(candidate) = types.get(name.as_str()) {
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

/// An abstract type
///
/// Used to allow Arrays and Tuples to have [`Generic`](enum.Generic.html) types inside them.
#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum AbstractType {
    /// The Array variant
    Array(Box<Generic>),
    /// The Tuple variant
    ///
    /// Very imperfect and very not used: this does not allow a variadic number of types
    Tuple(Box<Generic>),
}

impl Match for AbstractType {
    fn match_with(&self, given_type: &Type, types: &HashMap<&str, Generic>) -> bool {
        use self::AbstractType::*;

        match (self, given_type) {
            (&Array(ref el_type), &Type::Array(ref given_el_type)) => {
                (*el_type).match_with(&*given_el_type, types)
            }
            (&Tuple(ref el_type), &Type::Tuple(ref given_el_types)) => {
                given_el_types.iter().all(|given_type| {
                    (*el_type).match_with(&*given_type, types)
                })
            }
            _ => false,
        }
    }
}

/// A sum type (also called variant)
#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct SumType {
    /// The possible variants
    pub possibilities: Vec<Generic>,
}

impl Match for SumType {
    fn match_with(&self, given_type: &Type, types: &HashMap<&str, Generic>) -> bool {
        self.possibilities
            .iter()
            .any(|candidate| candidate.match_with(given_type, types))
    }
}

/// Unescape a string
///
/// This will evaluate escape sequence and return the given string, unescaped.
///
/// If an escape sequence is misused, this function will return an error with the name of the
/// misused escape sequence.
pub fn unescape_str(input: &str) -> Result<String, char> {
    let mut res = String::with_capacity(input.len());

    let mut chars = input.chars();

    while let Some(ch) = chars.next() {
        res.push(if ch != '\\' {
                     ch
                 } else {
                     match chars.next() {
                         Some('x') => {
                             if chars.clone().by_ref().count() < 2 {
                                 return Err('x');
                             }

                             chars
                                 .by_ref()
                                 .take(2)
                                 .fold(0u8,
                                       |acc, c| acc * 16 + c.to_digit(16).unwrap() as u8) as
                                 char
                         }
                         Some('u') => {
                             if chars.clone().by_ref().count() < 4 {
                                 return Err('u');
                             }

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

    Ok(res)
}
