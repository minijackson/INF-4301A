use type_sys::Value;

// The warning "unused_assignments" is allowed in many of these functions because there in the last
// argument of the "get_args!" macro, the assignment "count += 1" is unused.

macro_rules! get_args {
    ( $args:expr, $( $type:path ),* ) => {
        {
            let mut count = 0;
            (
                $(
                    if let &$type(val) = $args.get(count).expect("Wrong number of arguments") {
                        count += 1;
                        val
                    } else {
                        panic!("Wrong arguments");
                    },
                )*
            )
        }
    };
}

macro_rules! define_operator {
    ( $symbol:tt, $func_name:ident, $ret_type:path ) => {

        #[allow(unused_assignments)]
        pub fn $func_name(args: Vec<Value>) -> Value {
            use self::Value::*;
            let (lhs, rhs) = get_args!(args, Integer, Integer);
            $ret_type(lhs $symbol rhs)
        }

    }
}

pub fn resolve_func(name: String, args: Vec<Value>) -> Value {
    match name.as_ref() {
        "print" => print(args),
        _ => panic!("Unknown function: {}/{}", name, args.len()),
    }
}

pub fn print(args: Vec<Value>) -> Value {
    use self::Value::*;
    //let (val,) = get_args!(args, Integer);
    let val = args.get(0).expect("Wrong number of arguments");
    println!("=> {}", val);
    Void
}

//===========================
//== Arithmetic operations ==
//===========================

#[allow(unused_assignments)]
pub fn un_plus(args: Vec<Value>) -> Value {
    use self::Value::*;
    let (val,) = get_args!(args, Integer);
    Integer(val)
}

#[allow(unused_assignments)]
pub fn un_minus(args: Vec<Value>) -> Value {
    use self::Value::*;
    let (val,) = get_args!(args, Integer);
    Integer(-val)
}

define_operator!(+, plus,  Integer);
define_operator!(-, minus, Integer);
define_operator!(*, mul,   Integer);
define_operator!(/, div,   Integer);

//========================
//== Logical Operations ==
//========================

define_operator!(<,  lower,      Bool);
define_operator!(<=, lower_eq,   Bool);
define_operator!(>,  greater,    Bool);
define_operator!(>=, greater_eq, Bool);

#[allow(unused_assignments)]
pub fn equal(args: Vec<Value>) -> Value {
    use self::Value::*;
    let lhs = args.get(0).expect("Wrong number of arguments");
    let rhs = args.get(1).expect("Wrong number of arguments");
    Bool(lhs == rhs)
}

#[allow(unused_assignments)]
pub fn not_equal(args: Vec<Value>) -> Value {
    use self::Value::*;
    let lhs = args.get(0).expect("Wrong number of arguments");
    let rhs = args.get(1).expect("Wrong number of arguments");
    Bool(lhs != rhs)
}
