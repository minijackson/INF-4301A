use ast::Binding;

use std::collections::{LinkedList, HashMap};
use std::collections::hash_map::Entry;
use std::fmt;

pub struct Environment<T> {
    scopes: LinkedList<HashMap<String, T>>,
}

pub struct TypeInfo {
    pub type_: Type,
    pub declaration: Binding,
}

pub struct ValueInfo {
    pub value: Value,
    pub declaration: Binding,
}

pub struct DeclarationInfo {
    pub declaration: Binding,
}

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

impl<T> Environment<T> {
    pub fn new() -> Self {
        Self { scopes: LinkedList::new() }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push_front(HashMap::new());
    }

    pub fn leave_scope(&mut self) {
        self.scopes.pop_front().expect("Tried to leave a scope when not in a scope");
    }

    pub fn declare(&mut self, name: String, info: T) {
        match self.scopes
            .front_mut()
            .expect("Trying to declare a variable out of scope")
            .entry(name.clone()) {

            Entry::Occupied(_) => {
                panic!("{} is already defined", name);
            }

            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(info);
            }
        }
    }

    pub fn get(&self, name: &String) -> Option<&T> {
        self.scopes
            .iter()
            .find(|scope| scope.contains_key(name))
            .map(|scope| scope.get(name).unwrap())
    }
}
