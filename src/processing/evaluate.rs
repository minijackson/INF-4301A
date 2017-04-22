use ast::*;
use type_sys::Value;
use env::{Environment, BindingInfo, ValueInfo};

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

            Function {
                ref name,
                ref args,
                ..
            } => {
                let args = args.iter().map(|&(ref expr, _)| expr.evaluate(env)).collect::<Vec<type_sys::Value>>();

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

                        env.declare_var(current_arg.name.clone(), BindingInfo::Argument {
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
                if cond.evaluate(env).truthy().unwrap() {
                    true_branch.evaluate(env)
                } else {
                    false_branch.evaluate(env)
                }
            }

            While {
                ref cond,
                ref expr,
                ..
            } => {
                while cond.evaluate(env).truthy().unwrap() {
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
                            val += 1;
                            env.assign(&binding.name, type_sys::Value::Integer(val));
                        }
                    }
                    other => {
                        unreachable!("{:?} not an int, weren't you supposed to be good at coding?",
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

            UnaryOp {
                ref expr,
                ref op,
                ..
            } => {
                let args = vec![expr.evaluate(env)];
                env.call_builtin(&format!("un{}", op.to_string()), &args)
            }

            Cast {
                ref expr,
                ref dest,
                ..
            } => {
                expr.evaluate(env).into(dest)
            }

            Variable {
                ref name,
                ..
            } => {
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
                    values: values.iter().map(|&(ref expr, _)| expr.evaluate(env)).collect(),
                }
            }

            Tuple(ref exprs) => {
                let (element_types, values) = exprs.iter().map(|expr| {
                    let value = expr.evaluate(env);
                    let type_ = value.get_type();
                    (type_, value)
                })
                .unzip();

                type_sys::Value::Tuple { element_types, values }
            }

            Value(ref value) => value.clone(),

        }
    }
}
