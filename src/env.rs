use ast::Binding;
use type_sys::{Value,Type};

use std::collections::{LinkedList, HashMap};
use std::collections::hash_map::Entry;

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
