use ast::*;
use builtins;
use type_sys::Value;
use env::{Environment, ValueInfo};

pub trait Evaluate {
    fn evaluate(&self, bindings: &mut Environment<ValueInfo>) -> Value;
}

impl Evaluate for Exprs {
    fn evaluate(&self, bindings: &mut Environment<ValueInfo>) -> Value {
        let mut value = Value::Void;
        for expr in self.exprs.iter() {
            value = expr.evaluate(bindings);
        }
        value
    }
}

impl Evaluate for Expr {
    fn evaluate(&self, bindings: &mut Environment<ValueInfo>) -> Value {
        use ast::Expr::*;
        use ast::BinaryOpCode::*;
        use ast::UnaryOpCode::*;
        use type_sys;

        match self {
            &Grouping(ref exprs) => exprs.evaluate(bindings),

            &Let(ref assignments, ref exprs) => {
                bindings.enter_scope();
                for binding in assignments.iter() {
                    let value = binding.value.evaluate(bindings);
                    bindings.declare(binding.variable.clone(),
                                     ValueInfo {
                                         value: value,
                                         declaration: binding.clone(),
                                     });
                }

                let rv = exprs.evaluate(bindings);
                bindings.leave_scope();
                rv
            }

            &Assign(ref name, ref expr) => {
                let value = expr.evaluate(bindings);
                bindings.assign(name, value.clone());
                value
            }

            &Function(ref name, ref args) => {
                let args = args.iter()
                    .map(|ref expr| expr.evaluate(bindings))
                    .collect();
                builtins::resolve_func(name.clone(), args)
            }

            &If(ref cond, ref true_branch, ref false_branch) => {
                if cond.evaluate(bindings).truthy() {
                    true_branch.evaluate(bindings)
                } else {
                    false_branch.evaluate(bindings)
                }
            }

            &While(ref cond, ref expr) => {
                while cond.evaluate(bindings).truthy() {
                    expr.evaluate(bindings);
                }
                type_sys::Value::Void
            }

            &BinaryOp(ref lhs, ref rhs, ref op) => {
                let args = vec![lhs.evaluate(bindings), rhs.evaluate(bindings)];
                match op {
                    &Add => builtins::plus(args),
                    &Sub => builtins::minus(args),
                    &Mul => builtins::mul(args),
                    &Div => builtins::div(args),

                    &Lt => builtins::lower(args),
                    &Le => builtins::lower_eq(args),
                    &Gt => builtins::greater(args),
                    &Ge => builtins::greater_eq(args),
                    &Eq => builtins::equal(args),
                    &Ne => builtins::not_equal(args),
                }
            }

            &UnaryOp(ref exp, ref op) => {
                let args = vec![exp.evaluate(bindings)];
                match op {
                    &Plus => builtins::un_plus(args),
                    &Minus => builtins::un_minus(args),
                }
            }

            &Variable(ref name) => {
                bindings.get(name)
                    .expect(format!("Unbounded variable: {}", name).as_str())
                    .value
                    .clone()
            }

            &Value(ref value) => value.clone(),

        }
    }
}
