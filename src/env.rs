use ast::{Declaration, FunctionDecl, ArgumentDecl, VariableDecl, Span};
use builtins;
use error::AlreadyDeclaredError;
use type_sys::{Value, Type};

use std::collections::{LinkedList, HashMap};
use std::collections::hash_map::Entry;

pub struct Environment<T> {
    // TODO: change that abomination of a LinkedList
    scopes: LinkedList<Scope<T>>,
    builtins: HashMap<String, BuiltinInfo>,
}

pub struct Scope<T> {
    pub variables: HashMap<String, BindingInfo<T>>,
    pub functions: HashMap<String, FunctionDecl>,
}

impl<T> Scope<T> {
    pub fn new() -> Self {
        Scope {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }
}

pub struct BuiltinInfo {
    pub name: String,
    pub signatures: HashMap<Vec<Type>, Type>,
    pub call: Box<FnMut(&[Value]) -> Value + 'static>,
}

impl BuiltinInfo {
    pub fn new(name: String,
               signatures: HashMap<Vec<Type>, Type>,
               call: Box<FnMut(&[Value]) -> Value>)
               -> Self {
        BuiltinInfo {
            name,
            signatures,
            call,
        }
    }

    pub fn is_defined_for(&self, arg_types: &[Type]) -> bool {
        self.signatures.contains_key(arg_types)
    }

    pub fn return_type(&self, arg_types: &[Type]) -> Option<Type> {
        self.signatures.get(arg_types).cloned()
    }
}

pub enum BindingInfo<T> {
    Variable { declaration: VariableDecl, info: T },
    Argument { declaration: ArgumentDecl, info: T },
}

impl<T> BindingInfo<T> {
    pub fn get_declaration(&self) -> Declaration {
        use self::BindingInfo::*;

        match *self {
            Variable { ref declaration, .. } => Declaration::Variable(declaration.clone()),
            Argument { ref declaration, .. } => Declaration::Argument(declaration.clone()),
        }
    }
}

impl BindingInfo<TypeInfo> {
    pub fn get_type(&self) -> Type {
        use self::BindingInfo::*;

        match *self {
            Variable { ref info, .. } |
            Argument { ref info, .. } => info.0,
        }
    }
}

impl BindingInfo<ValueInfo> {
    pub fn get_value(&self) -> &Value {
        use self::BindingInfo::*;

        match *self {
            Variable { ref info, .. } |
            Argument { ref info, .. } => &info.0,
        }
    }

    pub fn set_value(&mut self, value: Value) {
        use self::BindingInfo::*;

        match *self {
            Variable { ref mut info, .. } |
            Argument { ref mut info, .. } => {
                info.0 = value;
            }
        }
    }
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
                "+".to_string() => BuiltinInfo::new("+".to_string(), plus_sig.clone(), Box::new(builtins::plus)),
                "-".to_string() => BuiltinInfo::new("-".to_string(), arit_sig.clone(), Box::new(builtins::minus)),
                "*".to_string() => BuiltinInfo::new("*".to_string(), arit_sig.clone(), Box::new(builtins::mul)),
                "/".to_string() => BuiltinInfo::new("/".to_string(), arit_sig,         Box::new(builtins::div)),

                "<".to_string()  => BuiltinInfo::new("<".to_string(),  cmp_sig.clone(), Box::new(builtins::lower)),
                "<=".to_string() => BuiltinInfo::new("<=".to_string(), cmp_sig.clone(), Box::new(builtins::lower_eq)),
                ">".to_string()  => BuiltinInfo::new(">".to_string(),  cmp_sig.clone(), Box::new(builtins::greater)),
                ">=".to_string() => BuiltinInfo::new(">=".to_string(), cmp_sig.clone(), Box::new(builtins::greater_eq)),
                "=".to_string()  => BuiltinInfo::new("=".to_string(),  cmp_sig.clone(), Box::new(builtins::equal)),
                "<>".to_string() => BuiltinInfo::new("<>".to_string(), cmp_sig,         Box::new(builtins::not_equal)),

                "un+".to_string() => BuiltinInfo::new("un+".to_string(), unary_sig.clone(), Box::new(builtins::un_plus)),
                "un-".to_string() => BuiltinInfo::new("un-".to_string(), unary_sig,         Box::new(builtins::un_minus)),

                "print".to_string() => BuiltinInfo::new("print".to_string(), print_sig.clone(), Box::new(builtins::print)),
                "println".to_string() => BuiltinInfo::new("println".to_string(), print_sig, Box::new(builtins::println))
                ),
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push_front(Scope::new());
    }

    pub fn leave_scope(&mut self) {
        self.scopes
            .pop_front()
            .expect("Tried to leave a scope when not in a scope");
    }

    pub fn declare_var(&mut self,
                       name: String,
                       info: BindingInfo<T>)
                       -> Result<(), AlreadyDeclaredError> {
        let scope = &mut self.scopes
                             .front_mut()
                             .expect("Trying to declare a variable out of scope")
                             .variables;

        match scope.entry(name.clone()) {
            Entry::Occupied(entry) => {
                Err(AlreadyDeclaredError::new(name, entry.get().get_declaration(), Span(0, 0)))
            }

            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(info);
                Ok(())
            }
        }
    }

    pub fn get_var(&self, name: &str) -> Option<&BindingInfo<T>> {
        self.scopes
            .iter()
            .find(|scope| scope.variables.contains_key(name))
            .map(|scope| &scope.variables[name])
    }

    pub fn get_var_mut(&mut self, name: &str) -> Option<&mut BindingInfo<T>> {
        self.scopes
            .iter_mut()
            .find(|scope| scope.variables.contains_key(name))
            .map(|scope| scope.variables.get_mut(name).unwrap())
    }

    pub fn declare_func(&mut self, decl: FunctionDecl) -> Result<(), AlreadyDeclaredError> {
        let scope = &mut self.scopes
                             .front_mut()
                             .expect("Trying to declare a variable out of scope")
                             .functions;

        match scope.entry(decl.name.clone()) {
            Entry::Occupied(entry) => {
                Err(AlreadyDeclaredError::new(decl.name,
                                              Declaration::Function(entry.get().clone()),
                                              decl.signature_span))
            }

            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(decl);
                Ok(())
            }
        }
    }

    pub fn get_func(&self, name: &str) -> Option<&FunctionDecl> {
        self.scopes
            .iter()
            .find(|scope| scope.functions.contains_key(name))
            .map(|scope| &scope.functions[name])
    }

    pub fn get_builtin(&self, name: &str) -> Option<&BuiltinInfo> {
        self.builtins.get(name)
    }

    pub fn get_builtin_mut(&mut self, name: &str) -> Option<&mut BuiltinInfo> {
        self.builtins.get_mut(name)
    }

    pub fn call_builtin(&mut self, name: &str, args: &[Value]) -> Value {
        (self.builtins
             .get_mut(name)
             .expect("No such function")
             .call)(&args)
    }
}

impl Environment<ValueInfo> {
    pub fn assign(&mut self, name: &str, value: Value) {
        self.get_var_mut(name)
            .expect(format!("Could not find variable {} in current scope", name).as_str())
            .set_value(value);
    }
}
