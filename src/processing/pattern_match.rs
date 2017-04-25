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
