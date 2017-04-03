use ast::*;

use itertools::Itertools;

pub trait Print {
    fn pretty_print(&self, indent: usize) -> String;
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
            &Assign(ref name, ref expr) => format!("{} := {}", name, expr.pretty_print(indent)),
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
    use parser;
    use super::Print;

    #[test]
    fn pretty_print() {
        let exp_str = "2+2+2";
        let expected = "2 + 2 + 2";
        let exp = parser::parse_Expression(exp_str).unwrap();
        let result = &exp.pretty_print(0);
        assert_eq!(expected,result);
    }
}
