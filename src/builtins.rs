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

#[allow(unused_assignments)]
pub fn plus(args: Vec<Value>) -> Value {
    use self::Value::*;
    let (lhs, rhs) = get_args!(args, Integer, Integer);
    Integer(lhs + rhs)
}

#[allow(unused_assignments)]
pub fn minus(args: Vec<Value>) -> Value {
    use self::Value::*;
    let (lhs, rhs) = get_args!(args, Integer, Integer);
    Integer(lhs - rhs)
}

#[allow(unused_assignments)]
pub fn mul(args: Vec<Value>) -> Value {
    use self::Value::*;
    let (lhs, rhs) = get_args!(args, Integer, Integer);
    Integer(lhs * rhs)
}

#[allow(unused_assignments)]
pub fn div(args: Vec<Value>) -> Value {
    use self::Value::*;
    let (lhs, rhs) = get_args!(args, Integer, Integer);
    Integer(lhs / rhs)
}
