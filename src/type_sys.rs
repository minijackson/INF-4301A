use std::fmt;

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
    pub fn truthy(&self) -> bool {
        use self::Value::*;

        match self {
            &Integer(0) => false,
            &Integer(_) => true,
            &Float(0f32) => false,
            &Float(_) => true,
            &Bool(false) => false,
            &Bool(true) => true,
            &Void => panic!("Tried to convert void to bool"),
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

    pub fn into_int(self) -> Self {
        use self::Value::*;

        match self {
            Integer(_) => self,
            Float(val) => Integer(val as i32),
            Bool(val) => Integer(val as i32),
            Void => panic!("Tried to convert void to int"),
        }
    }

    pub fn into_float(self) -> Self {
        use self::Value::*;

        match self {
            Integer(val) => Float(val as f32),
            Float(_) => self,
            Bool(val) => Float(if val { 1f32 } else { 0f32 }),
            Void => panic!("Tried to convert void to float"),
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
            &Void => panic!("Void is not printable"),
        }
    }
}
