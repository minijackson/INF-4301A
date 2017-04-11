use ast::*;
use type_sys::Value;
use env::{Environment, ValueInfo};

pub trait Evaluate {
    fn evaluate(&self, env: &mut Environment<ValueInfo>) -> Value;
}

impl Evaluate for Exprs {
    fn evaluate(&self, env: &mut Environment<ValueInfo>) -> Value {
        let mut value = Value::Void;
        for expr in self.exprs.iter() {
            value = expr.evaluate(env);
        }
        value
    }
}

impl Evaluate for Expr {
    fn evaluate(&self, env: &mut Environment<ValueInfo>) -> Value {
        use ast::Expr::*;
        use type_sys;

        match self {
            &Grouping(ref exprs) => exprs.evaluate(env),

            &Let(ref bindings, ref exprs) => {
                env.enter_scope();
                for binding in bindings.iter() {
                    let value = binding.value.evaluate(env);
                    env.declare(binding.variable.clone(),
                                 ValueInfo {
                                     value: value,
                                     declaration: binding.clone(),
                                 })
                        .unwrap();
                }

                let rv = exprs.evaluate(env);
                env.leave_scope();
                rv
            }

            &Assign(ref name, ref expr) => {
                let value = expr.evaluate(env);
                env.assign(name, value.clone());
                value
            }

            &Function(ref name, ref args) => {
                let args = args.iter()
                    .map(|ref expr| expr.evaluate(env))
                    .collect();
                env.call_builtin(&name, args)
            }

            &If(ref cond, ref true_branch, ref false_branch) => {
                if cond.evaluate(env).truthy().unwrap() {
                    true_branch.evaluate(env)
                } else {
                    false_branch.evaluate(env)
                }
            }

            &While(ref cond, ref expr) => {
                while cond.evaluate(env).truthy().unwrap() {
                    expr.evaluate(env);
                }
                type_sys::Value::Void
            }

            &For(ref binding, ref expr, ref expr2) => {
                env.enter_scope();
                use std::ops::Deref;

                let var = binding.value.evaluate(env);
                env.declare(binding.variable.clone(),
                             ValueInfo {
                                 value: var.clone(),
                                 declaration: binding.deref().clone(),
                             })
                    .unwrap();
                println!("{:?}", binding);    
                let var2 = expr.evaluate(env);
                match (var,var2) {
                   (type_sys::Value::Integer(mut var),type_sys::Value::Integer(var2)) => {
                        while var < var2 {
                            expr2.evaluate(env);
                            var = var + 1;
                        }
                    }
                    other => unreachable!("{:?} not an int, weren't you supposed to be good at coding?", other) 
                }
                env.leave_scope();
                type_sys::Value::Void
            }

            &BinaryOp(ref lhs, ref rhs, ref op) => {
                let args = vec![lhs.evaluate(env), rhs.evaluate(env)];
                env.call_builtin(&op.to_string(), args)
            }

            &UnaryOp(ref expr, ref op) => {
                let args = vec![expr.evaluate(env)];
                env.call_builtin(&format!("un{}", op.to_string()), args)
            }

            &Variable(ref name) => {
                env.get_var(name)
                    .expect(format!("Unbounded variable: {}", name).as_str())
                    .value
                    .clone()
            }

            &Value(ref value) => value.clone(),

        }
    }
}
