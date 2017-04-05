use type_sys::Value;

// The warning "unused_assignments" is allowed in many of these functions because there in the last
// argument of the "get_args!" macro, the assignment "count += 1" is unused.

macro_rules! get_args {
    ( $args:expr, $( $arg_type:path ),* ) => {
        {
            let mut count = 0;
            (
                $(
                    if let &$arg_type(val) = $args.get(count).expect("Wrong number of arguments") {
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
    ( $symbol:tt, $func_name:ident, $param_type:path, $ret_type:path ) => {

        #[allow(unused_assignments)]
        pub fn $func_name(args: Vec<Value>) -> Value {
            use self::Value::*;

            let (lhs, rhs) = get_args!(args, $param_type, $param_type);
            $ret_type(lhs $symbol rhs)
        }

    }
}

macro_rules! define_arit_operator {
    ( $symbol:tt, $func_name:ident) => {

        pub fn $func_name(args: Vec<Value>) -> Value {
            use self::Value::*;

            match (&args[0], &args[1]) {
                (&Integer(lhs), &Integer(rhs)) => Integer(lhs $symbol rhs),
                (&Float(lhs), &Float(rhs)) => Float(lhs $symbol rhs),
                (lhs, rhs) => unreachable!("Wrong type of arguments in `{}`: {:?}, {:?}", stringify!($func_name), lhs, rhs)
            }
        }

    }
}

macro_rules! define_cmp_operator {
    ( $symbol:tt, $func_name:ident) => {

        pub fn $func_name(args: Vec<Value>) -> Value {
            use self::Value::*;

            match (&args[0], &args[1]) {
                (&Integer(lhs), &Integer(rhs)) => Bool(lhs $symbol rhs),
                (&Float(lhs), &Float(rhs)) => Bool(lhs $symbol rhs),
                (lhs, rhs) => unreachable!("Wrong type of arguments in `{}`: {:?}, {:?}", stringify!($func_name), lhs, rhs)
            }
        }
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

define_arit_operator!(+, plus);
define_arit_operator!(-, minus);
define_arit_operator!(*, mul);
define_arit_operator!(/, div);

//========================
//== Logical Operations ==
//========================

define_cmp_operator!(<,  lower);
define_cmp_operator!(<=, lower_eq);
define_cmp_operator!(>,  greater);
define_cmp_operator!(>=, greater_eq);

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
