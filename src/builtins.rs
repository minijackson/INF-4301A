//! Where all the builtins are defined (Rust code-wise)
//!
//! This includes print / println functions, binary / unary operators...

use type_sys::Value;
use type_sys::Value::*;

macro_rules! define_arit_operator {
    ( $symbol:tt, $func_name:ident) => {

        pub fn $func_name(args: &[Value]) -> Value {
            match (&args[0], &args[1]) {
                (&Integer(lhs), &Integer(rhs)) => Integer(lhs $symbol rhs),
                (&Float(lhs), &Float(rhs)) => Float(lhs $symbol rhs),
                (lhs, rhs) => unreachable!("Wrong type of arguments in `{}`: {:?}, {:?}",
                                           stringify!($func_name),
                                           lhs,
                                           rhs)
            }
        }

    }
}

macro_rules! define_cmp_operator {
    ( $symbol:tt, $func_name:ident) => {
        pub fn $func_name(args: &[Value]) -> Value {
            Bool(args[0] $symbol args[1])
        }
    }
}

//===================
//== Builtin funcs ==
//===================

pub fn print(args: &[Value]) -> Value {
    print!("{}", args[0]);
    Void
}

pub fn println(args: &[Value]) -> Value {
    println!("{}", args[0]);
    Void
}

//===========================
//== Arithmetic operations ==
//===========================

pub fn un_plus(args: &[Value]) -> Value {
    match &args[0] {
        &Integer(val) => Integer(val),
        &Float(val) => Float(val),
        val => unreachable!("Wrong type of arguments in `un+`: {:?}", val),
    }
}

pub fn un_minus(args: &[Value]) -> Value {
    match &args[0] {
        &Integer(val) => Integer(-val),
        &Float(val) => Float(-val),
        val => unreachable!("Wrong type of arguments in `un-`: {:?}", val),
    }
}

pub fn plus(args: &[Value]) -> Value {
    match (&args[0], &args[1]) {
        (&Integer(lhs), &Integer(rhs)) => Integer(lhs + rhs),
        (&Float(lhs), &Float(rhs)) => Float(lhs + rhs),
        (&Str(ref lhs), &Str(ref rhs)) => Str(lhs.clone() + rhs.as_str()),
        (lhs, rhs) => unreachable!("Wrong type of arguments in `plus`: {:?}, {:?}",
                                   lhs,
                                   rhs)
    }
}

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

pub fn equal(args: &[Value]) -> Value {
    use std::f64::EPSILON;

    match (&args[0], &args[1]) {
        (&Float(lhs), &Float(rhs)) => Bool((lhs - rhs).abs() < EPSILON),
        (lhs, rhs) => Bool(lhs == rhs),
    }
}

pub fn not_equal(args: &[Value]) -> Value {
    Bool(args[0] != args[1])
}
