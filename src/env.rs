use ast::{Binding, Span};
use builtins;
use error::{AlreadyDeclaredError, UnboundedVarError, UndefinedFunctionError};
use type_sys::{Value, Type};

use std::collections::{LinkedList, HashMap};
use std::collections::hash_map::Entry;

pub struct Environment<T> {
    scopes: LinkedList<HashMap<String, BindingInfo<T>>>,
    builtins: HashMap<String, FunctionInfo>,
}

pub struct FunctionInfo {
    pub name: String,
    pub signatures: HashMap<Vec<Type>, Type>,
    pub call: Box<FnMut(Vec<Value>) -> Value + 'static>,
}

impl FunctionInfo {
    pub fn new(name: String,
               signatures: HashMap<Vec<Type>, Type>,
               call: Box<FnMut(Vec<Value>) -> Value>)
               -> Self {
        FunctionInfo {
            name,
            signatures,
            call,
        }
    }

    pub fn is_defined_for(&self, arg_types: &Vec<Type>) -> bool {
        self.signatures.contains_key(arg_types)
    }

    pub fn return_type(&self, arg_types: &Vec<Type>) -> Option<&Type> {
        self.signatures
            .get(arg_types)
    }
}

pub struct BindingInfo<T> {
    pub declaration: Binding,
    pub info: T,
}

pub struct TypeInfo(pub Type);

pub struct ValueInfo(pub Value);

impl<T> Environment<T> {
    pub fn new() -> Self {
        use self::Type::*;

        macro_rules! quick_hashmap {
            ( $( $key:expr => $value:expr ),* ) => {
                {
                    let mut rv = HashMap::new();
                    $(rv.insert($key, $value);)*
                    rv
                }
            }
        }

        let plus_sig = quick_hashmap!(
                    vec![Integer, Integer] => Integer,
                    vec![Float, Float] => Float,
                    vec![Str, Str] => Str
                    );

        let arit_sig = quick_hashmap!(
                    vec![Integer, Integer] => Integer,
                    vec![Float, Float] => Float
                    );

        let cmp_sig = quick_hashmap!(
                    vec![Integer, Integer] => Bool,
                    vec![Float, Float] => Bool,
                    vec![Str, Str] => Bool
                    );

        let unary_sig = quick_hashmap!(
                    vec![Integer] => Integer,
                    vec![Float] => Float
                    );

        let print_sig = quick_hashmap!(
                    vec![Integer] => Void,
                    vec![Float] => Void,
                    vec![Str] => Void
                    );

        Self {
            scopes: LinkedList::new(),
            builtins: quick_hashmap!(
                "+".to_string() => FunctionInfo::new("+".to_string(), plus_sig.clone(), Box::new(builtins::plus)),
                "-".to_string() => FunctionInfo::new("-".to_string(), arit_sig.clone(), Box::new(builtins::minus)),
                "*".to_string() => FunctionInfo::new("*".to_string(), arit_sig.clone(), Box::new(builtins::mul)),
                "/".to_string() => FunctionInfo::new("/".to_string(), arit_sig,         Box::new(builtins::div)),

                "<".to_string()  => FunctionInfo::new("<".to_string(),  cmp_sig.clone(), Box::new(builtins::lower)),
                "<=".to_string() => FunctionInfo::new("<=".to_string(), cmp_sig.clone(), Box::new(builtins::lower_eq)),
                ">".to_string()  => FunctionInfo::new(">".to_string(),  cmp_sig.clone(), Box::new(builtins::greater)),
                ">=".to_string() => FunctionInfo::new(">=".to_string(), cmp_sig.clone(), Box::new(builtins::greater_eq)),
                "=".to_string()  => FunctionInfo::new("=".to_string(),  cmp_sig.clone(), Box::new(builtins::equal)),
                "<>".to_string() => FunctionInfo::new("<>".to_string(), cmp_sig,         Box::new(builtins::not_equal)),

                "un+".to_string() => FunctionInfo::new("un+".to_string(), unary_sig.clone(), Box::new(builtins::un_plus)),
                "un-".to_string() => FunctionInfo::new("un-".to_string(), unary_sig,         Box::new(builtins::un_minus)),

                "print".to_string() => FunctionInfo::new("print".to_string(), print_sig.clone(), Box::new(builtins::print)),
                "println".to_string() => FunctionInfo::new("println".to_string(), print_sig, Box::new(builtins::println))
                ),
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push_front(HashMap::new());
    }

    pub fn leave_scope(&mut self) {
        self.scopes
            .pop_front()
            .expect("Tried to leave a scope when not in a scope");
    }

    pub fn declare(&mut self, name: String, info: BindingInfo<T>) -> Result<(), AlreadyDeclaredError> {
        let scope = self.scopes
            .front_mut()
            .expect("Trying to declare a variable out of scope");

        match scope.entry(name.clone()) {
            Entry::Occupied(entry) => Err(AlreadyDeclaredError::new(name, entry.get().declaration.clone(), Span(0, 0))),

            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(info);
                Ok(())
            }
        }
    }

    pub fn get_var(&self, name: &String) -> Result<&BindingInfo<T>, UnboundedVarError> {
        self.scopes
            .iter()
            .find(|scope| scope.contains_key(name))
            .map(|scope| scope.get(name).unwrap())
            // TODO
            .ok_or(UnboundedVarError::new(name.clone(), Span(0, 0)))
    }

    pub fn get_var_mut(&mut self, name: &String) -> Result<&mut BindingInfo<T>, UnboundedVarError> {
        self.scopes
            .iter_mut()
            .find(|scope| scope.contains_key(name))
            .map(|scope| scope.get_mut(name).unwrap())
            // TODO
            .ok_or(UnboundedVarError::new(name.clone(), Span(0, 0)))
    }

    pub fn get_builtin(&self, name: &String) -> Result<&FunctionInfo, UndefinedFunctionError> {
        self.builtins
            .get(name)
            // TODO
            .ok_or(UndefinedFunctionError::new(name.clone(), Span(0, 0)))
    }

    pub fn get_builtin_mut(&mut self,
                           name: &String)
                           -> Result<&mut FunctionInfo, UndefinedFunctionError> {
        self.builtins
            .get_mut(name)
            // TODO
            .ok_or(UndefinedFunctionError::new(name.clone(), Span(0, 0)))
    }

    pub fn call_builtin(&mut self, name: &String, args: Vec<Value>) -> Value {
        (self.builtins
             .get_mut(name)
             .expect("No such function")
             .call)(args)
    }
}

impl Environment<ValueInfo> {
    pub fn assign(&mut self, name: &String, value: Value) {
        self.get_var_mut(name)
            .expect(format!("Could not find variable {} in current scope", name).as_str())
            .info.0 = value;
    }
}
