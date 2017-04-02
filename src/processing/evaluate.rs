use ast::*;
use std::collections::HashMap;

pub trait Evaluate {
    fn evaluate(&self, bindings: &mut HashMap<String, i32>) -> i32;
}

impl Evaluate for Expr {
    fn evaluate(&self, bindings: &mut HashMap<String, i32>) -> i32 {
        use ast::ExprKind::*;
        use ast::BinaryOpCode::*;
        use ast::UnaryOpCode::*;

        let &Expr { ref kind, .. } = self;

        match kind {

            &Grouping(ref exprs) => {
                exprs.evaluate(bindings)
            }

            &Let(ref assignments, ref exprs) => {
                for binding in assignments.iter() {
                    let value = binding.value.evaluate(bindings);
                    bindings.insert(binding.variable.clone(), value);
                }

                exprs.evaluate(bindings)
            }

            &Function(ref name, ref args) => {
                if name == "print" && args.len() == 1 {
                    println!("=> {}", &args[0].evaluate(bindings));
                    return 0;
                } else {
                    panic!("Unknown function: {}/{}", name, args.len());
                }
            }

            &If(box ref cond, ref true_branch, ref false_branch) => {
                if cond.evaluate(bindings) != 0 {
                    true_branch.evaluate(bindings)
                } else {
                    false_branch.evaluate(bindings)
                }
            }

            &BinaryOp(box ref lhs, box ref rhs, ref op) => {
                match op {
                    &Add => lhs.evaluate(bindings) + rhs.evaluate(bindings),
                    &Sub => lhs.evaluate(bindings) - rhs.evaluate(bindings),
                    &Mul => lhs.evaluate(bindings) * rhs.evaluate(bindings),
                    &Div => lhs.evaluate(bindings) / rhs.evaluate(bindings),
                }
            }

            &UnaryOp(box ref exp, ref op) => {
                match op {
                    &Plus => exp.evaluate(bindings),
                    &Minus => -exp.evaluate(bindings),
                }
            }

            &Variable(ref name) => {
                *bindings.get(name).expect(format!("Unbounded variable: {}", name).as_str())
            }

            &Num(value) => value,

        }
    }
}

impl Evaluate for Exprs {
    fn evaluate(&self, bindings: &mut HashMap<String, i32>) -> i32 {
        let mut value = 0;
        for expr in self.exprs.iter() {
            value = expr.evaluate(bindings);
        }
        value
    }
}
