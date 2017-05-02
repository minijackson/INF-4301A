//! The module where the pattern match type checking is implemented

use ast::*;
use env::{Environment, TypeInfo};
use error::{TypeCheckError, MismatchedTypesError};
use processing::TypeCheck;

/// That trait that must be implemented by the part of the AST for pattern match type checking
pub trait PatternMatchCheck: TypeCheck {
    fn check_match(&mut self,
                   rhs: &mut Self,
                   env: &mut Environment<TypeInfo>)
                   -> Result<(), TypeCheckError>;
}

impl PatternMatchCheck for Expr {
    fn check_match(&mut self,
                   rhs: &mut Self,
                   env: &mut Environment<TypeInfo>)
                   -> Result<(), TypeCheckError> {
        use ast::Expr::*;

        if let Variable { ref name, span } = *self {
            let mut assign = Expr::Assign {
                name: name.clone(),
                name_span: span,
                value: Box::new(rhs.clone()),
                // TODO
                value_span: Span(0, 0),
            };

            assign.type_check(env)?;
            Ok(())
        } else {
            let my_type = self.type_check(env)?;
            let rhs_type = rhs.type_check(env)?;

            if my_type != rhs_type {
                // TODO
                return Err(MismatchedTypesError::new(my_type.into(), rhs_type, Span(0, 0)).into());
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use env::Environment;
    use error::*;
    use parser;
    use processing::TypeCheck;
    use type_sys::Generic;
    use type_sys::Type::*;

    macro_rules! assert_err {

        ( $expr:expr, $err:pat ) => {
            let res = parser::parse_Expression($expr)
                .unwrap()
                .type_check(&mut Environment::new());
            assert!(match res {
                Err($err) => true,
                _ => false,
            });
        };

        ( $expr:expr, $err:pat if $guard:expr ) => {
            let res = parser::parse_Expression($expr)
                .unwrap()
                .type_check(&mut Environment::new());
            assert!(match res {
                Err($err) if $guard => true,
                _ => false,
            });
        };

    }

    macro_rules! assert_type {
        ( $expr:expr, $ok:expr ) => {
            assert_eq!(parser::parse_Expression($expr)
                           .unwrap()
                           .type_check(&mut Environment::new()),
                       Ok($ok));
        }
    }

    #[test]
    fn value() {
        assert_err!("match 1 := 2+3.4",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");

        assert_type!("match 1 := 1", Bool);
        assert_err!("match 1 := 1.",
                    TypeCheckError::MismatchedTypes(MismatchedTypesError {
                        expected: Generic::Builtin(Integer),
                        got: Float,
                        ..
                    }));
        assert_err!(r#"match 1 := "hello""#,
                    TypeCheckError::MismatchedTypes(MismatchedTypesError {
                        expected: Generic::Builtin(Integer),
                        got: Str,
                        ..
                    }));
    }

    #[test]
    fn array() {
        assert_err!("match [1] := [2+3.4]",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");

        assert_type!("match Integer[] := Integer[]", Bool);
        assert_err!("match Integer[] := Float[]",
                    TypeCheckError::MismatchedTypes(MismatchedTypesError {
                        ref expected,
                        ref got,
                        ..
                    })
                    if *expected == Generic::Builtin(Array(Box::new(Integer))) &&
                       *got == Array(Box::new(Float)));
    }

    #[test]
    fn tuple() {
        assert_err!("match {1} := {2+3.4}",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");

        assert_type!("match {} := {}", Bool);
        assert_err!("match {1} := {2.}",
                    TypeCheckError::MismatchedTypes(MismatchedTypesError {
                        ref expected,
                        ref got,
                        ..
                    })
                    if *expected == Generic::Builtin(Tuple(vec![Integer])) &&
                       *got == Tuple(vec![Float]));
    }

}
