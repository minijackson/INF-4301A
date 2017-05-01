use ast::*;
use type_sys::Value;
use env::{Environment, BindingInfo, ValueInfo};
use processing::pattern_match::PatternMatch;

pub trait Evaluate {
    fn evaluate(&self, env: &mut Environment<ValueInfo>) -> Value;
}

impl Evaluate for Exprs {
    fn evaluate(&self, env: &mut Environment<ValueInfo>) -> Value {
        let mut value = Value::Void;
        for expr in &self.exprs {
            value = expr.evaluate(env);
        }
        value
    }
}

impl Evaluate for Expr {
    fn evaluate(&self, env: &mut Environment<ValueInfo>) -> Value {
        use ast::Expr::*;
        use type_sys;

        match *self {
            Grouping(ref exprs) => exprs.evaluate(env),

            Let(ref bindings, ref function_decls, ref exprs) => {
                env.enter_scope();

                for binding in bindings.iter() {
                    let value = binding.value.evaluate(env);
                    env.declare_var(binding.name.clone(),
                                     BindingInfo::Variable {
                                         declaration: binding.clone(),
                                         info: ValueInfo(value),
                                     })
                        .unwrap();
                }

                for function_decl in function_decls.iter() {
                    env.declare_func(function_decl.clone()).unwrap();
                }

                let rv = exprs.evaluate(env);

                env.leave_scope();
                rv
            }

            Assign {
                ref name,
                ref value,
                ..
            } => {
                let value = value.evaluate(env);
                env.assign(name, value.clone());
                value
            }

            PatternMatch { ref lhs, ref rhs, .. } => {
                let var_save = env.scopes.clone();

                let res = lhs.pattern_match(&rhs.evaluate(env), env);

                if !res {
                    env.scopes = var_save;
                }

                type_sys::Value::Bool(res)
            }

            Function { ref name, ref args, .. } => {
                let args = args.iter()
                    .map(|&(ref expr, _)| expr.evaluate(env))
                    .collect::<Vec<type_sys::Value>>();

                let mut user_defined = false;
                let mut user_func = None;

                if let Some(func) = env.get_func(name) {
                    user_defined = true;

                    user_func = Some(func.clone());
                }

                if user_defined {
                    let func = user_func.unwrap();

                    env.enter_scope();

                    for (ind, value) in args.into_iter().enumerate() {
                        let current_arg = &func.args[ind];

                        env.declare_var(current_arg.name.clone(),
                                         BindingInfo::Argument {
                                             declaration: current_arg.clone(),
                                             info: ValueInfo(value),
                                         })
                            .unwrap();
                    }

                    let rv = func.body.evaluate(env);

                    env.leave_scope();
                    rv
                } else {
                    env.call_builtin(name, &args)
                }
            }

            If {
                ref cond,
                ref true_branch,
                ref false_branch,
                ..
            } => {
                if cond.evaluate(env).truthy() {
                    true_branch.evaluate(env)
                } else {
                    false_branch.evaluate(env)
                }
            }

            While { ref cond, ref expr, .. } => {
                while cond.evaluate(env).truthy() {
                    expr.evaluate(env);
                }
                type_sys::Value::Void
            }

            For {
                ref binding,
                ref goal,
                ref expr,
                ..
            } => {
                env.enter_scope();

                let val = binding.value.evaluate(env);
                env.declare_var(binding.name.clone(),
                                 BindingInfo::Variable {
                                     declaration: (**binding).clone(),
                                     info: ValueInfo(val.clone()),
                                 })
                    .unwrap();

                let upper = goal.evaluate(env);
                match (val, upper) {
                    (type_sys::Value::Integer(mut val), type_sys::Value::Integer(upper)) => {
                        while val < upper {
                            expr.evaluate(env);
                            val = 1 +
                                  if let type_sys::Value::Integer(val) =
                                *env.get_var(&binding.name).unwrap().get_value() {
                                      val
                                  } else {
                                      panic!("Variable {} is not of type Integer anymore",
                                             binding.name);
                                  };
                            env.assign(&binding.name, type_sys::Value::Integer(val));
                        }
                    }
                    other => {
                        panic!("{:?} is not of type (Integer, Integer) in for loop evaluation",
                               other)
                    }
                }
                env.leave_scope();
                type_sys::Value::Void
            }

            BinaryOp {
                ref lhs,
                ref rhs,
                ref op,
                ..
            } => {
                let args = vec![lhs.evaluate(env), rhs.evaluate(env)];
                env.call_builtin(&op.to_string(), &args)
            }

            UnaryOp { ref expr, ref op, .. } => {
                let args = vec![expr.evaluate(env)];
                env.call_builtin(&format!("un{}", op.to_string()), &args)
            }

            Cast { ref expr, ref dest, .. } => expr.evaluate(env).into(dest),

            Variable { ref name, .. } => {
                env.get_var(name)
                    .expect(format!("Unbounded variable: {}", name).as_str())
                    .get_value()
                    .clone()
            }

            Array {
                ref values,
                ref declared_type,
                ..
            } => {
                type_sys::Value::Array {
                    element_type: declared_type.clone().unwrap(),
                    values: values
                        .iter()
                        .map(|&(ref expr, _)| expr.evaluate(env))
                        .collect(),
                }
            }

            Tuple(ref exprs) => {
                let (element_types, values) = exprs
                    .iter()
                    .map(|expr| {
                             let value = expr.evaluate(env);
                             let type_ = value.get_type();
                             (type_, value)
                         })
                    .unzip();

                type_sys::Value::Tuple {
                    element_types,
                    values,
                }
            }

            Value(ref value) => value.clone(),

        }
    }
}

#[cfg(test)]
mod tests {
    use super::Evaluate;

    use env::Environment;
    use parser;
    use processing::TypeCheck;
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
    fn grouping() {
        assert_result!("(1, 2, 3)", Integer(3));
        assert_result!("(1, 2, true)", Bool(true));
    }

    #[test]
    fn let_block() {
        assert_result!("let
                       in
                       end",
                       Void);

        assert_result!("let
                       in
                          42
                       end",
                       Integer(42));

        assert_result!(r#"let
                       in
                          true,
                          "hello",
                          false,
                       end"#,
                       Bool(false));

        // Variable and function declarations are checked in the `variable`, `assign` and
        // `function` test functions.
    }

    #[test]
    fn assign() {
        assert_result!("let var x := 1 in x := 2 end", Integer(2));
        assert_result!("let var x := 1 in x := 2, x + 40 end", Integer(42));
    }

    #[test]
    fn pattern_match() {
        assert_result!("match 1 := 1", Bool(true));

        // Real pattern match tests are in the src/processing/pattern_match.rs and
        // src/processing/pattern_match_check.rs files.
    }

    #[test]
    fn function() {
        assert_result!("let
                          function x(): Bool := true
                       in
                          x()
                       end",
                       Bool(true));

        assert_result!("let
                          function x(): Integer := 2 + 2
                       in
                          x()
                       end",
                       Integer(4));

        assert_result!("let
                          function x(y: Integer): Integer := y + 2
                       in
                          x(40)
                       end",
                       Integer(42));

        assert_result!("let
                          function fact(n: Integer): Integer := if n then n*fact(n-1) else 1
                       in
                          fact(5)
                       end",
                       Integer(120));
    }

    #[test]
    fn if_block() {
        assert_result!("if true then true else false", Bool(true));
        assert_result!("if false then true else false", Bool(false));
        assert_result!("if 1+1 then 2-1 else 1+1", Integer(1));
        assert_result!("if 1-1 then 2-1 else 1+1", Integer(2));
        assert_result!("if Bool[] then true else false", Bool(false));
        assert_result!("if [true, false] then true else false", Bool(true));
        assert_result!("let
                          var x := 0
                       in
                          if x then x := 2 else x := 3,
                          x
                       end",
                       Integer(3));
        assert_result!("let
                          var x := 1
                       in
                          if x then x := 2 else x := 3,
                          x
                       end",
                       Integer(2));
    }

    #[test]
    fn while_block() {
        assert_result!("while 0 do 1", Void);
        assert_result!("let
                           var x := 42
                        in
                           while x <> 0 do x := x - 1,
                           x
                        end",
                       Integer(0));
        assert_result!("let
                           var x := 42
                        in
                           while x <> 42 do x := 0,
                           x
                        end",
                       Integer(42));
    }

    #[test]
    fn for_block() {
        assert_result!("for var x := 0 to 42 do x", Void);
        assert_result!("let
                          var x := 0
                       in
                          for var y := 0 to 5 do x := x + y,
                          x
                       end",
                       Integer(10));
        assert_result!("let
                          var x := 0
                       in
                          for var y := 0 to 0 do x := 42,
                          x
                       end",
                       Integer(0));
        assert_result!("let
                          var x := 0
                       in
                          for var y := 69 to 42 do x := 42,
                          x
                       end",
                       Integer(0));
        assert_result!("let
                          var x := 0
                       in
                          for var y := 0 to 42 do (
                             x := x + 1,
                             y := 41
                          ),
                          x
                       end",
                       Integer(1));
    }

    #[test]
    fn binary_ops() {
        assert_result!("2+2", Integer(4));
        assert_result!("2-2", Integer(0));
        assert_result!("6*7", Integer(42));
        assert_result!("3/2", Integer(1));
        assert_result!("3=2", Bool(false));
        assert_result!("2=2", Bool(true));
        assert_result!("2<>2", Bool(false));
        assert_result!("3<>2", Bool(true));
        assert_result!("3>2", Bool(true));
        assert_result!("3>=2", Bool(true));
        assert_result!("3<2", Bool(false));
        assert_result!("3<=2", Bool(false));

        assert_result!(r#""hello" = "world""#, Bool(false));
        assert_result!(r#""hello" = "hello""#, Bool(true));
        assert_result!(r#""hello" >= "hello""#, Bool(true));
        assert_result!(r#""hello" <= "hello""#, Bool(true));
        assert_result!(r#""hello" > "hello""#, Bool(false));
        assert_result!(r#""hello" < "world""#, Bool(true));
        assert_result!(r#""hello" > "world""#, Bool(false));
        assert_result!(r#""a" < "aaa""#, Bool(true));

        assert_result!("[1, 2, 3] = [1, 2, 3]", Bool(true));
        assert_result!("[1, 2, 3] = [1, 2, 3, 4]", Bool(false));
        assert_result!("[1, 2, 3, 4] = [1, 2, 3]", Bool(false));
        assert_result!("[1] = [2]", Bool(false));
        assert_result!("[1, 2] = [1, 3]", Bool(false));
        assert_result!("[1, 2] <= [1, 2]", Bool(true));
        assert_result!("[1, 2] >= [1, 2]", Bool(true));
        assert_result!("[1, 2] < [1, 3]", Bool(true));
        assert_result!("[1, 2] < [1, 2, 3]", Bool(true));
        assert_result!("[1, 3] > [1, 2]", Bool(true));
        assert_result!("[1, 2, 3] > [1, 2]", Bool(true));

        // For the tuple comparisons to work, the generic type system must be more powerful than it
        // currently is.
    }

    #[test]
    fn unary_ops() {
        assert_result!("-2", Integer(-2));
        assert_result!("-2.", Float(-2f64));
        assert_result!("-0.3", Float(-0.3f64));
        assert_result!("-0", Integer(0));
        assert_result!("-0.", Float(0f64));
        assert_result!("+2", Integer(2));
        assert_result!("+2.", Float(2f64));
        assert_result!("+0.3", Float(0.3f64));
        assert_result!("+0", Integer(0));
        assert_result!("+0.", Float(0f64));
    }

    #[test]
    fn cast() {
        assert_result!("1 as Float", Float(1f64));
        assert_result!("1.5 as Float", Float(1.5f64));
        assert_result!("1.5 as Integer", Integer(1));
        assert_result!("1.7 as Integer", Integer(1));

        assert_result!("Integer[] as Array(Float)",
                       Array {
                           element_type: Type::Float,
                           values: vec![],
                       });
        assert_result!("[1, 2, 3] as Array(Float)",
                       Array {
                           element_type: Type::Float,
                           values: vec![Float(1f64), Float(2f64), Float(3f64)],
                       });

        assert_result!("{1, 2.3, 3} as Tuple(Integer, Integer, Integer)",
                       Tuple {
                           element_types: vec![Type::Integer, Type::Integer, Type::Integer],
                           values: vec![Integer(1), Integer(2), Integer(3)],
                       });
        assert_result!("{1, 2.3, 4} as Array(Integer)",
                       Array {
                           element_type: Type::Integer,
                           values: vec![Integer(1), Integer(2), Integer(4)],
                       });
        assert_result!("[{1, 2.3, 4}, {4, 3.2, 1}] as Array(Tuple(Float, Integer, Bool))",
                       Array {
                           element_type: Type::Tuple(vec![Type::Float, Type::Integer, Type::Bool]),
                           values: vec![Tuple {
                                            element_types: vec![Type::Float,
                                                                Type::Integer,
                                                                Type::Bool],
                                            values: vec![Float(1f64), Integer(2), Bool(true)],
                                        },
                                        Tuple {
                                            element_types: vec![Type::Float,
                                                                Type::Integer,
                                                                Type::Bool],
                                            values: vec![Float(4f64), Integer(3), Bool(true)],
                                        }],
                       });
    }

    #[test]
    fn variable() {
        assert_result!("let
                          var x := 0
                       in
                          x
                       end",
                       Integer(0));
        assert_result!("let
                          var x := true
                       in
                          x
                       end",
                       Bool(true));
        assert_result!("let
                          function x(x: Integer): Integer := x
                       in
                          x(42)
                       end",
                       Integer(42));
        assert_result!("let
                          function x(x: Integer): Integer := x + 40
                       in
                          x(2)
                       end",
                       Integer(42));
    }

    #[test]
    fn array() {
        assert_result!("Integer[]",
                       Array {
                           element_type: Type::Integer,
                           values: vec![],
                       });
        assert_result!("[1]",
                       Array {
                           element_type: Type::Integer,
                           values: vec![Integer(1)],
                       });
        assert_result!("[1 + 1]",
                       Array {
                           element_type: Type::Integer,
                           values: vec![Integer(2)],
                       });
        assert_result!("[1, 1 + 1, 6/2]",
                       Array {
                           element_type: Type::Integer,
                           values: vec![Integer(1), Integer(2), Integer(3)],
                       });
    }

    #[test]
    fn tuple() {
        assert_result!("{}",
                       Tuple {
                           element_types: vec![],
                           values: vec![],
                       });
        assert_result!("{1}",
                       Tuple {
                           element_types: vec![Type::Integer],
                           values: vec![Integer(1)],
                       });
        assert_result!("{true}",
                       Tuple {
                           element_types: vec![Type::Bool],
                           values: vec![Bool(true)],
                       });
        assert_result!("{1, true}",
                       Tuple {
                           element_types: vec![Type::Integer, Type::Bool],
                           values: vec![Integer(1), Bool(true)],
                       });
        assert_result!("{{}, {1}, true}",
                       Tuple {
                           element_types: vec![Type::Tuple(vec![]),
                                               Type::Tuple(vec![Type::Integer]),
                                               Type::Bool],
                           values: vec![Tuple {
                                            element_types: vec![],
                                            values: vec![],
                                        },
                                        Tuple {
                                            element_types: vec![Type::Integer],
                                            values: vec![Integer(1)],
                                        },
                                        Bool(true)],
                       });
    }

    #[test]
    fn value() {
        assert_result!("1", Integer(1));
        assert_result!("3.14", Float(3.14f64));
        assert_result!("true", Bool(true));
        assert_result!(r#""hello""#, Str("hello".to_string()));
    }
}
