use itertools::Itertools;

use ast::*;

use std::collections::HashMap;

pub trait Evaluate {
    fn evaluate(&self, bindings: &mut HashMap<String, i32>) -> i32;
}

pub trait Print {
    fn pretty_print(&self, indent: usize) -> String;
}

impl Evaluate for Expr {
    fn evaluate(&self, bindings: &mut HashMap<String, i32>) -> i32 {
        use ast::Expr::*;
        use ast::BinaryOpCode::*;
        use ast::UnaryOpCode::*;

        match self {
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

impl Print for Expr {
    fn pretty_print(&self, indent: usize) -> String {
        use ast::Expr::*;
        use ast::BinaryOpCode::*;
        use ast::UnaryOpCode::*;

        let strws = " ".repeat(indent);
        let ws = strws.as_str();

        match self {
            &Grouping(ref exprs) => {
                format!("(\n{}{})", exprs.pretty_print(indent + 2), ws)
            }
            &Let(ref assignments, ref exprs) => {
                format!("let\n{}{}in\n{}{}end",
                        assignments.iter()
                            .map(|binding| {
                                binding.pretty_print(indent + 2)
                            })
                            .join(""),
                        ws,
                        exprs.pretty_print(indent + 2),
                        ws)
            }
            &Function(ref name, ref args) => {
                format!("{}({})",
                        name,
                        args.iter()
                            .map(|exp| exp.pretty_print(indent))
                            .join(", ")
                        )
            }
            &If(box ref cond, ref true_branch, ref false_branch) => {
                format!("if {} then {} else {}", cond.pretty_print(indent), true_branch.pretty_print(indent), false_branch.pretty_print(indent))
            }
            &BinaryOp(box ref lhs, box ref rhs, ref op) => {
                match op {
                    &Add => format!("{} + {}", &lhs.pretty_print(indent), &rhs.pretty_print(indent)),
                    &Sub => format!("{} - {}", &lhs.pretty_print(indent), &rhs.pretty_print(indent)),
                    &Mul => format!("{} * {}", &lhs.pretty_print(indent), &rhs.pretty_print(indent)),
                    &Div => format!("{} / {}", &lhs.pretty_print(indent), &rhs.pretty_print(indent)),
                }
            }
            &UnaryOp(box ref exp, ref op) => {
                match op {
                    &Plus => format!("+{}", &exp.pretty_print(indent)),
                    &Minus => format!("-{}", &exp.pretty_print(indent)),
                }
            }
            &Variable(ref name) => name.clone(),
            &Num(value) => value.to_string(),
        }
    }
}

impl Print for Exprs {
    fn pretty_print(&self, indent: usize) -> String {
        let strws = " ".repeat(indent);
        let ws    = strws.as_str();

        self.exprs.iter()
            .map(|exp| exp.pretty_print(indent))
            .map(|disp| format!("{}{}", ws, disp))
            .join(",\n") + "\n"
    }
}

impl Print for Binding {
    fn pretty_print(&self, indent: usize) -> String {
        let strws = " ".repeat(indent);
        let ws    = strws.as_str();

        format!("{}var {} := {}\n", ws, self.variable, self.value.pretty_print(indent))
    }
}

#[cfg(test)]
mod test {
    use calculator;
    use super::Print;

    #[test]
    fn pretty_print() {
        let exp_str = "2+2+2";
        let expected = "2 + 2 + 2";
        let exp = calculator::parse_Expression(exp_str).unwrap();
        let result = &exp.pretty_print(0);
        assert_eq!(expected,result);
    }
}
