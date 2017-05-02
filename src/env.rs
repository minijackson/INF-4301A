//! Where the environment is managed (hopefully in an ecological manner)
//!
//! The environment comprise the scopes (variables and functions), builtins and defined types.
//! There is currently no way of defining custom types, but the generic type system is implemented
//! (see the [`type_sys::Generic`] enum)
//!
//! [`type_sys::Generic`]: ../type_sys/enum.Generic.html

use ast::{Declaration, FunctionDecl, ArgumentDecl, VariableDecl, Span};
use builtins;
use error::AlreadyDeclaredError;
use type_sys::{Value, Type, Generic, AbstractType, SumType, Match};

use std::collections::{LinkedList, HashMap};
use std::collections::hash_map::Entry;

/// The main struct containing the whole environment
///
/// A god-like structure if you ask me
///
/// The `T` generic parameter corresponds to what will be stored as a binding info (type, value,
/// etc.)
pub struct Environment<T> {
    /// The scopes
    // TODO: change that abomination of a LinkedList
    pub scopes: LinkedList<Scope<T>>,
    /// The defined builtins (defined globally)
    pub builtins: HashMap<&'static str, BuiltinInfo>,
    /// The defined generic types (defined globally)
    pub types: HashMap<&'static str, Generic>,
}

/// A scope. Contains functions and variables
///
/// The `T` generic parameter corresponds to what will be stored as a binding info (type, value,
/// etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct Scope<T> {
    /// The variables in the current scope
    pub variables: HashMap<String, BindingInfo<T>>,
    /// The functions in the current scope
    pub functions: HashMap<String, FunctionDecl>,
}

impl<T> Scope<T> {
    /// Create a new empty scope
    pub fn new() -> Self {
        Scope {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }
}

/// A struct used to store some info about a builtin function
pub struct BuiltinInfo {
    /// The name of the builtin
    pub name: String,
    /// The signatures for this builtin
    pub signatures: HashMap<Vec<Generic>, Type>,
    /// A pointer to the Rust function (defined in the [`builtins`](../builtins/index.html) module)
    ///
    /// The `'static` thing in the type means that this pointer must be defined for a static
    /// lifetime, hence valid for the duration of the whole program.
    ///
    /// A `Box` is needed because `FnMut` is a trait, not a type, and so does not have a compile
    /// time known size. Wrapping it inside a box is equivalent to store it as a pointer /
    /// reference.
    pub call: Box<FnMut(&[Value]) -> Value + 'static>,
}

impl BuiltinInfo {
    /// Create a new builtin info struct
    pub fn new(name: String,
               signatures: HashMap<Vec<Generic>, Type>,
               call: Box<FnMut(&[Value]) -> Value>)
               -> Self {
        BuiltinInfo {
            name,
            signatures,
            call,
        }
    }

    /// Get the return type of the function provided a list of arguments
    pub fn return_type(&self, arg_types: &[Type], types: &HashMap<&str, Generic>) -> Option<Type> {
        self.signatures
            .iter()
            .find(|&(params, _)| {
                      if arg_types.len() != params.len() {
                          return false;
                      }

                      params
                          .iter()
                          .zip(arg_types)
                          .all(|(cand_type, arg_type)| cand_type.match_with(arg_type, types))
                  })
            .map(|(_, type_)| type_.clone())
    }
}

/// Stores the info of a binding (function or argument)
///
/// The `T` generic parameter corresponds to what will be stored as a binding info (type, value,
/// etc.)
#[derive(Debug, Clone, PartialEq)]
pub enum BindingInfo<T> {
    /// The variable variant
    Variable { declaration: VariableDecl, info: T },
    /// The argument variant
    Argument { declaration: ArgumentDecl, info: T },
}

impl<T> BindingInfo<T> {
    /// Get the original declaration for this binding.
    pub fn get_declaration(&self) -> Declaration {
        use self::BindingInfo::*;

        match *self {
            Variable { ref declaration, .. } => Declaration::Variable(declaration.clone()),
            Argument { ref declaration, .. } => Declaration::Argument(declaration.clone()),
        }
    }
}

impl BindingInfo<TypeInfo> {
    /// Get the type of this binding
    pub fn get_type(&self) -> &Type {
        use self::BindingInfo::*;

        match *self {
            Variable { ref info, .. } |
            Argument { ref info, .. } => &info.0,
        }
    }
}

impl BindingInfo<ValueInfo> {
    /// Get the current value of this binding
    pub fn get_value(&self) -> &Value {
        use self::BindingInfo::*;

        match *self {
            Variable { ref info, .. } |
            Argument { ref info, .. } => &info.0,
        }
    }

    /// Set the value of this binding
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

/// Stores a Type
///
/// Used in a [`BindingInfo`](enum.BindingInfo.html).
///
/// Could have used the Type type directly, but now that I see it, it's a bit late.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo(pub Type);

/// Stores a Value
///
/// Used in a [`BindingInfo`](enum.BindingInfo.html).
///
/// Could have used the Value type directly, but now that I see it, it's a bit late.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueInfo(pub Value);

impl<T> Environment<T> {
    /// Create a new environment
    ///
    /// This will create an environment with no current scope, the default builtins, and the
    /// default generic types.
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
                    vec![Integer.into(), Integer.into()] => Integer,
                    vec![Float.into(), Float.into()] => Float,
                    vec![Str.into(), Str.into()] => Str
                    );

        let arit_sig = quick_hashmap!(
                    vec![Integer.into(), Integer.into()] => Integer,
                    vec![Float.into(), Float.into()] => Float
                    );

        // Not exactly correct: the two parameters are not guaranteed to be of the same type.
        //
        // This will result in that any comparison with objects of different types will always
        // return `false`. I would prefer a type error.
        let cmp_sig = quick_hashmap!(
                    vec![Generic::Named("Comparable".to_string()), Generic::Named("Comparable".to_string())] => Bool
                    );

        let unary_sig = quick_hashmap!(
                    vec![Integer.into()] => Integer,
                    vec![Float.into()] => Float
                    );

        let print_sig = quick_hashmap!(
                    vec![Generic::Any] => Void
                    );

        let number_type = Generic::Sum(SumType {
            possibilities: vec![
                Integer.into(),
                Float.into(),
            ]
        });

        let printable_type = Generic::Sum(SumType {
                    possibilities: vec![
                        Integer.into(),
                        Float.into(),
                        Bool.into(),
                        Str.into(),
                        Generic::Abstract(AbstractType::Array(Box::new(Generic::Named("Printable".to_string())))),
                    ]
                });

        let comparable_type = Generic::Sum(SumType {
            possibilities: vec![
                Integer.into(),
                Float.into(),
                Bool.into(),
                Str.into(),
                Generic::Abstract(AbstractType::Array(Box::new(Generic::Named("Comparable".to_string())))),
            ]
        });

        Self {
            scopes: LinkedList::new(),
            builtins: quick_hashmap!(
                "+" => BuiltinInfo::new("+".to_string(), plus_sig.clone(), Box::new(builtins::plus)),
                "-" => BuiltinInfo::new("-".to_string(), arit_sig.clone(), Box::new(builtins::minus)),
                "*" => BuiltinInfo::new("*".to_string(), arit_sig.clone(), Box::new(builtins::mul)),
                "/" => BuiltinInfo::new("/".to_string(), arit_sig,         Box::new(builtins::div)),

                "<"  => BuiltinInfo::new("<".to_string(),  cmp_sig.clone(), Box::new(builtins::lower)),
                "<=" => BuiltinInfo::new("<=".to_string(), cmp_sig.clone(), Box::new(builtins::lower_eq)),
                ">"  => BuiltinInfo::new(">".to_string(),  cmp_sig.clone(), Box::new(builtins::greater)),
                ">=" => BuiltinInfo::new(">=".to_string(), cmp_sig.clone(), Box::new(builtins::greater_eq)),
                "="  => BuiltinInfo::new("=".to_string(),  cmp_sig.clone(), Box::new(builtins::equal)),
                "<>" => BuiltinInfo::new("<>".to_string(), cmp_sig,         Box::new(builtins::not_equal)),

                "un+" => BuiltinInfo::new("un+".to_string(), unary_sig.clone(), Box::new(builtins::un_plus)),
                "un-" => BuiltinInfo::new("un-".to_string(), unary_sig,         Box::new(builtins::un_minus)),

                "print"   => BuiltinInfo::new("print".to_string(), print_sig.clone(), Box::new(builtins::print)),
                "println" => BuiltinInfo::new("println".to_string(), print_sig, Box::new(builtins::println))
                ),

            types: quick_hashmap!(
                "Printable" => printable_type,
                "Comparable" => comparable_type,
                "Number" => number_type
                ),
        }
    }

    /// Enter in a new scope
    ///
    /// This will create a child scope from the innermost scope.
    ///
    /// Note: Beware of philosophical revelations
    pub fn enter_scope(&mut self) {
        self.scopes.push_front(Scope::new());
    }

    /// Enter the current scope
    ///
    /// This will destroy the innermost scope, and return in the direct parent scope.
    pub fn leave_scope(&mut self) {
        self.scopes
            .pop_front()
            .expect("Tried to leave a scope when not in a scope");
    }

    /// Declare a new variable in the current scope
    ///
    /// Returns an [`AlreadyDeclaredError`] if a variable of the same name is already defined in
    /// the current scope.
    ///
    /// [`AlreadyDeclaredError`]: ../error/struct.AlreadyDeclaredError.html
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

    /// Lookup a variable by name
    ///
    /// This will look for the variable in all the scopes, starting with the innermost one.
    pub fn get_var(&self, name: &str) -> Option<&BindingInfo<T>> {
        self.scopes
            .iter()
            .find(|scope| scope.variables.contains_key(name))
            .map(|scope| &scope.variables[name])
    }

    /// Lookup a variable by name (mutable reference version)
    ///
    /// This will look for the variable in all the scopes, starting with the innermost one.
    pub fn get_var_mut(&mut self, name: &str) -> Option<&mut BindingInfo<T>> {
        self.scopes
            .iter_mut()
            .find(|scope| scope.variables.contains_key(name))
            .map(|scope| scope.variables.get_mut(name).unwrap())
    }

    /// Declare a new function in the current scope
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

    /// Lookup a function declaration by name
    ///
    /// This will look for the variable in all the scopes, starting with the innermost one.
    pub fn get_func(&self, name: &str) -> Option<&FunctionDecl> {
        self.scopes
            .iter()
            .find(|scope| scope.functions.contains_key(name))
            .map(|scope| &scope.functions[name])
    }

    /// Lookup a builtin info by name
    pub fn get_builtin(&self, name: &str) -> Option<&BuiltinInfo> {
        self.builtins.get(name)
    }

    /// Lookup a builtin info by name (mutable reference version)
    pub fn get_builtin_mut(&mut self, name: &str) -> Option<&mut BuiltinInfo> {
        self.builtins.get_mut(name)
    }

    /// Call a given builtin from its name
    ///
    /// Panics if the builtin is not defined
    pub fn call_builtin(&mut self, name: &str, args: &[Value]) -> Value {
        (self.builtins
             .get_mut(name)
             .expect("No such function")
             .call)(&args)
    }

    /// Get a given generic type by its name
    pub fn get_type(&self, name: &str) -> Option<&Generic> {
        self.types.get(name)
    }
}

impl Environment<ValueInfo> {
    /// Assign a variable given a name and a value
    ///
    /// Panics if the variable is not defined
    pub fn assign(&mut self, name: &str, value: Value) {
        self.get_var_mut(name)
            .expect(format!("Could not find variable {} in current scope", name).as_str())
            .set_value(value);
    }
}
