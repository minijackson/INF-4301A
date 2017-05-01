use ast::*;
use env::{Environment, ValueInfo};
use processing::Evaluate;
use type_sys;

pub trait PatternMatch {
    fn pattern_match(&self, rhs: &type_sys::Value, env: &mut Environment<ValueInfo>) -> bool;
}

impl PatternMatch for Expr {
    fn pattern_match(&self, rhs: &type_sys::Value, env: &mut Environment<ValueInfo>) -> bool {
        use ast::Expr::*;

        match *self {
            Variable { ref name, .. } => {
                let assign = Expr::Assign {
                    name: name.clone(),
                    name_span: Span(0, 0),
                    value: Box::new(Value(rhs.clone())),
                    value_span: Span(0, 0),
                };

                assign.evaluate(env);
                true
            }

            Array { ref values, .. } => {
                if let type_sys::Value::Array{ values: ref candidate_values, .. } = *rhs {
                    values.len() == candidate_values.len() &&
                    values
                        .iter()
                        .zip(candidate_values)
                        .all(|(&(ref value, _), candidate)| value.pattern_match(&*candidate, env))
                } else {
                    panic!("Wrong pattern");
                }
            }

            Tuple(ref values) => {
                if let type_sys::Value::Tuple { values: ref candidate_values, .. } = *rhs {
                    values.len() == candidate_values.len() &&
                    values
                        .iter()
                        .zip(candidate_values)
                        .all(|(value, candidate)| value.pattern_match(&*candidate, env))
                } else {
                    panic!("Wrong pattern");
                }
            }

            Value(ref value) => {
                value == rhs
            }

            _ => panic!("Forbidden pattern"),
        }
    }
}

#[cfg(test)]
mod tests {
    use env::Environment;
    use parser;
    use processing::{Evaluate, TypeCheck};
    use type_sys::Value::*;

    macro_rules! assert_result {

        ( $expr:expr, $expected:expr ) => {
            let mut ast = parser::parse_Expression($expr)
                .unwrap();
            // The type checker might modify the AST a bit before the evaluation
            ast.type_check(&mut Environment::new()).unwrap();
            let res = ast.evaluate(&mut Environment::new());
            assert_eq!(res, $expected);
        }

    }

    #[test]
    fn array() {
        assert_result!("let var x := 1 in match [x] := [42], x end", Integer(42));
        assert_result!("let var x := 1 in match [x, 1] := [42], x end", Integer(1));
        assert_result!("let var x := 1 in match [x, 1] := [42, 2], x end", Integer(1));
    }

}
