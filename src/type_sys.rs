use std::fmt;

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Type {
    Void, Integer, /* Float, Bool, String, Array, */
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum Value {
    Void, Integer(i32), /* Float, Bool, String, Array, */
}

impl Value {
    pub fn truthy(&self) -> bool {
        use self::Value::*;

        match self {
            &Integer(0) => false,
            _ => true
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Value::*;

        match self {
            &Integer(value) => write!(f, "{}", value),
            _ => panic!("Unsupported type to print")
        }
    }
}
