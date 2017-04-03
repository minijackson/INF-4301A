use std::fmt;

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Type {
    Void, Integer, Bool, /* Float, String, Array, */
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum Value {
    Void, Integer(i32), Bool(bool), /* Float, String, Array, */
}

impl Value {
    pub fn truthy(&self) -> bool {
        use self::Value::*;

        match self {
            &Integer(0) => false,
            &Integer(_) => true,
            &Bool(false) => false,
            &Bool(true) => true,
            &Void => panic!("Tried to convert void to bool")
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Value::*;

        match self {
            &Integer(value) => write!(f, "{}", value),
            &Bool(value) => write!(f, "{}", value),
            &Void => panic!("Void is not printable")
        }
    }
}
