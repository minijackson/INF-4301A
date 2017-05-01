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
            // This should not set the value but pin the value to ensure subsequent use of the same
            // variable must have the same value (Prolog / Erlang style), and then assign variables
            // if everything matches, not saving the environment and restore it if the pattern does
            // not match (NOT elegant), but hey, time is missing, I even file like I won't finish
            // this sent...
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
    use type_sys::Type;

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
    fn value() {
        assert_result!("match 1 := 1", Bool(true));
        assert_result!("match 0 := 1", Bool(false));
        assert_result!("match 2.5 := 2.5", Bool(true));
        assert_result!("match 2 := 1 + 1", Bool(true));
        // Float: oops!
        //assert_result!("match 0.6 := 0.1 + 0.2 + 0.3", Bool(true));
        assert_result!(r#"match "hello" := "hello""#, Bool(true));
        assert_result!(r#"match "hello" := "world""#, Bool(false));

        assert_result!("let var x := 0 in match x := 42, x end", Integer(42));
        assert_result!(r#"let var x := "" in match x := "hello", x end"#, Str("hello".to_string()));
    }

    #[test]
    fn array() {
        assert_result!("match Integer[] := Integer[]", Bool(true));
        assert_result!("match [1] := [1]", Bool(true));
        assert_result!("match [1] := [2]", Bool(false));
        assert_result!("let var x := 1 in match [x] := [42], x end", Integer(42));
        assert_result!("let var x := 1 in match [x, 1] := [42], x end", Integer(1));
        assert_result!("let var x := 1 in match [x, 1] := [42, 2], x end", Integer(1));
        assert_result!("let var x := false in match [x, false] := [true, false], x end", Bool(true));
        assert_result!("let var x := false in match [x, false] := [true, true], x end", Bool(false));
    }

    #[test]
    fn tuple() {
        assert_result!("let var x := 1 in match [x] := [42], x end", Integer(42));
        assert_result!("let var x := 1 in match [x, 1] := [42], x end", Integer(1));
        assert_result!("let var x := 1 in match [x, 1] := [42, 2], x end", Integer(1));
    }

    #[test]
    fn megamix() {
        assert_result!("match [{}] := [{}]", Bool(true));
        assert_result!("match [{2, 3}, {4, 5}] := [{1 + 1, 6 / 2}, {4., 5} as Tuple(Integer, Integer)]", Bool(true));
        assert_result!(r#"let
                          function make_thingy(x: Integer, y: Float): Tuple(Integer, Float) := {x + 40, y * 3.}
                          var x := {0, 0.}
                       in
                          match {[x, {5, 6.}], "hello"} := {[make_thingy(2, 23.), make_thingy(-35, 2.)], "hello"},
                          x,
                       end"#,
                       Tuple {
                           element_types: vec![Type::Integer, Type::Float],
                           values: vec![Integer(42), Float(69f64)],
                       });
        assert_result!(r#"let
                          function make_thingy(x: Integer, y: Float): Tuple(Integer, Float) := {x + 40, y * 3.}
                          var x := {0, 0.}
                       in
                          match {[x, {5, 6.}], "hello"} := {[make_thingy(2, 23.), make_thingy(-35, 2.)], "world"},
                          x,
                       end"#,
                       Tuple {
                           element_types: vec![Type::Integer, Type::Float],
                           values: vec![Integer(0), Float(0f64)],
                       });
    }

}
