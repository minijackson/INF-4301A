use ast::Binding;
use builtins;
use type_sys::{Value,Type};

use std::collections::{LinkedList, HashMap};
use std::collections::hash_map::Entry;

pub struct Environment<T> {
    scopes: LinkedList<HashMap<String, T>>,
    builtins: HashMap<String, Box<FnMut(Vec<Value>) -> Value + 'static>>,
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
        Self {
            scopes: LinkedList::new(),
            builtins: {
                let mut rv: HashMap<String, Box<FnMut(Vec<Value>) -> Value + 'static>> = HashMap::new();

                rv.insert(String::from("+"), Box::new(builtins::plus));
                rv.insert(String::from("-"), Box::new(builtins::minus));
                rv.insert(String::from("*"), Box::new(builtins::mul));
                rv.insert(String::from("/"), Box::new(builtins::div));

                rv.insert(String::from("<"), Box::new(builtins::lower));
                rv.insert(String::from("<="), Box::new(builtins::lower_eq));
                rv.insert(String::from(">"), Box::new(builtins::greater));
                rv.insert(String::from(">="), Box::new(builtins::greater_eq));
                rv.insert(String::from("="), Box::new(builtins::equal));
                rv.insert(String::from("<>"), Box::new(builtins::not_equal));

                rv.insert(String::from("un+"), Box::new(builtins::un_plus));
                rv.insert(String::from("un-"), Box::new(builtins::un_minus));

                rv.insert(String::from("print"), Box::new(builtins::print));

                rv
            }
        }
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

    pub fn get_var(&self, name: &String) -> Option<&T> {
        self.scopes
            .iter()
            .find(|scope| scope.contains_key(name))
            .map(|scope| scope.get(name).unwrap())
    }

    pub fn get_var_mut(&mut self, name: &String) -> Option<&mut T> {
        self.scopes
            .iter_mut()
            .find(|scope| scope.contains_key(name))
            .map(|scope| scope.get_mut(name).unwrap())
    }

    pub fn call_builtin(&mut self, name: &String, args: Vec<Value>) -> Value {
        self.builtins.get_mut(name).expect("No such function")(args)
    }

}

impl Environment<ValueInfo> {

    pub fn assign(&mut self, name: &String, value: Value) {
        self.get_var_mut(name)
            .expect(format!("Could not find variable {} in current scope", name).as_str())
            .value = value;
    }

}
