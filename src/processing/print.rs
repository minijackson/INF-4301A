use ast::*;
use type_sys;

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
                let mut fmt_exprs = exprs.pretty_print(indent + 2);

                // Add a comma for single expr grouping
                if exprs.exprs.len() == 1 {
                    fmt_exprs.pop();
                    fmt_exprs += ",\n";
                }

                format!("(\n{}{})", fmt_exprs, ws)
            }

            &Let(ref bindings, ref function_decls, ref exprs) => {
                format!("let\n{}{}{}in\n{}{}end",
                        bindings
                            .iter()
                            .map(|binding| format!("{}\n", binding.pretty_print(indent + 2)))
                            .join(""),
                        function_decls
                            .iter()
                            .map(|binding| format!("{}\n", binding.pretty_print(indent + 2)))
                            .join(""),
                        ws,
                        exprs.pretty_print(indent + 2),
                        ws)
            }

            &Assign {
                 ref name,
                 ref value,
                 ..
             } => format!("{} := {}", name, value.pretty_print(indent)),

            &Function { ref name, ref args, .. } => {
                format!("{}({})",
                        name,
                        args.iter()
                            .map(|&(ref exp, _)| exp.pretty_print(indent))
                            .join(", "))
            }

            &If {
                 ref cond,
                 ref true_branch,
                 ref false_branch,
                 ..
             } => {
                format!("(if {} then {} else {})",
                        cond.pretty_print(indent),
                        true_branch.pretty_print(indent),
                        false_branch.pretty_print(indent))
            }

            &While { ref cond, ref expr, .. } => {
                format!("(while {} do {})",
                        cond.pretty_print(indent),
                        expr.pretty_print(indent))
            }

            &For {
                 ref binding,
                 ref goal,
                 ref expr,
                 ..
             } => {
                format!("(for {} to {} do {})",
                        binding.pretty_print(indent),
                        goal.pretty_print(indent),
                        expr.pretty_print(indent))
            }

            &BinaryOp {
                 ref lhs,
                 ref rhs,
                 ref op,
                 ..
             } => {
                let op_symbol = match op {
                    &Add => "+",
                    &Sub => "-",
                    &Mul => "*",
                    &Div => "/",

                    &Lt => "<",
                    &Le => "<=",
                    &Gt => ">",
                    &Ge => ">=",
                    &Eq => "=",
                    &Ne => "<>",
                };

                format!("({} {} {})",
                        &lhs.pretty_print(indent),
                        op_symbol,
                        &rhs.pretty_print(indent))
            }

            &UnaryOp { ref expr, ref op, .. } => {
                match op {
                    &Plus => format!("(+{})", expr.pretty_print(indent)),
                    &Minus => format!("(-{})", expr.pretty_print(indent)),
                }
            }

            &Cast { ref expr, ref dest, .. } => {
                format!("({} as {:?})", expr.pretty_print(indent), dest)
            }

            &Variable { ref name, .. } => name.clone(),

            // For strings, use the debug trait to add quotes
            &Value(type_sys::Value::Str(ref value)) => format!("{:?}", value),

            &Value(ref value) => value.to_string(),

        }
    }
}

impl Print for Exprs {
    fn pretty_print(&self, indent: usize) -> String {
        let strws = " ".repeat(indent);
        let ws = strws.as_str();

        self.exprs
            .iter()
            .map(|exp| exp.pretty_print(indent))
            .map(|disp| format!("{}{}", ws, disp))
            .join(",\n") + "\n"
    }
}

impl Print for VariableDecl {
    fn pretty_print(&self, indent: usize) -> String {
        let strws = " ".repeat(indent);
        let ws = strws.as_str();

        format!("{}var {} := {}",
                ws,
                self.name,
                self.value.pretty_print(indent))
    }
}

impl Print for FunctionDecl {
    fn pretty_print(&self, indent: usize) -> String {
        let strws = " ".repeat(indent);
        let ws = strws.as_str();

        let args = self.args
            .iter()
            .map(|ref arg| arg.pretty_print(indent))
            .join(", ");

        format!("{}function {}({}) : {:?} = {}",
                ws,
                self.name,
                args,
                self.return_type,
                self.body.pretty_print(indent))
    }
}

impl Print for ArgumentDecl {
    fn pretty_print(&self, _indent: usize) -> String {
        format!("{}: {:?}", self.name, self.type_)
    }
}

#[cfg(test)]
mod test {
    use super::Print;

    // Theses tests suppose the parser is correct so we don't have to manually input the AST
    use parser;

    macro_rules! perfect_coding {
        ( $expr:expr ) => {
            let expected = $expr;
            let expr = parser::parse_Expression($expr).unwrap();
            let result = expr.pretty_print(0);
            assert_eq!(expected, result);
        }
    }

    macro_rules! not_perfect_coding {
        ( $expr:expr ) => {
            let expected = $expr;
            let expr = parser::parse_Expression($expr).unwrap();
            let result = expr.pretty_print(0);
            assert_ne!(expected, result);
        }
    }

    #[test]
    fn simple_operators() {
        perfect_coding!("(2 + 2)");
        perfect_coding!("(2 - 2)");
        perfect_coding!("(2 * 2)");
        perfect_coding!("(2 / 2)");
        perfect_coding!("(2 < 2)");
        perfect_coding!("(2 <= 2)");
        perfect_coding!("(2 > 2)");
        perfect_coding!("(2 >= 2)");
        perfect_coding!("(-2)");
        perfect_coding!("(+2)");
    }

    #[test]
    fn chained_operators() {
        perfect_coding!("((2 + 2) + 2)");
        perfect_coding!("((2 + (+2)) - (-2))");
        perfect_coding!("((2 <> 2) > 2)");
    }

    #[test]
    fn grouping() {
        // Should not be parsed as a grouping, but as a simple parenthesis
        not_perfect_coding!("(
  2
)");

        // To resolve the ambiguity, simply add a comma at the end (Rust's tuples style)
        //
        // Seemingly useless, but forcing groupings to have at least 2 expressions is uglier in the
        // parser.
        perfect_coding!("(
  2,
)");

        perfect_coding!("(
  2,
  2
)");

        perfect_coding!("(
  2,
  2,
  2
)");

        perfect_coding!("(
  (
    2,
    2,
    2
  ),
  2,
  (
    2,
    2,
    2
  )
)");
    }

    #[test]
    fn let_block() {
        perfect_coding!("let
  var x := 2
in
  2
end");

        perfect_coding!("let
  function x(a: Integer, b: Integer) : Integer = (a * b)
in
  x(1, 2)
end");

        perfect_coding!("let
  var x := let
    var x := 2
  in
    2
  end
in
  let
    var x := 2
  in
    2
  end
end");

    }

    #[test]
    fn assign() {
        perfect_coding!("x := 1");
        perfect_coding!("x := y := 2");
    }

    #[test]
    fn function_call() {
        perfect_coding!("x()");
        perfect_coding!("x(1)");
        perfect_coding!("x(1, 2, 3)");
        perfect_coding!("x(y(1))");
        perfect_coding!("x(1, y(2), 3)");
    }

    #[test]
    fn if_block() {
        perfect_coding!("(if 1 then 1 else 0)");
        perfect_coding!("(if (if 1 then 1 else 0) then (if 1 then 1 else 0) else (if 1 then 1 else 0))");
        perfect_coding!("(if (
  0,
  1
) then (
  0,
  1
) else (
  0,
  1
))");
    }

    #[test]
    fn while_block() {
        perfect_coding!("(while 1 do 1)");
        perfect_coding!("(while (while 1 do 1) do (while 1 do 1))");
    }

    #[test]
    fn for_block() {
        perfect_coding!("(for var x := 1 to 1 do 1)");
        perfect_coding!("(for var x := (for var x := 1 to 2 do 1) to (for var x := 1 to 2 do 1) do (for var x := 1 to 2 do 1))");
    }

    #[test]
    fn cast() {
        perfect_coding!("((+2) as Str)");
        perfect_coding!("(+(2 as Str))");
        perfect_coding!("(+(2 as Str))");
    }

    #[test]
    fn variable() {
        perfect_coding!("hello");
        perfect_coding!("hello_world");
        perfect_coding!("hello_world2");
        // Should be catched by the parser's unit tests but hey, why not?
        not_perfect_coding!("hello-world2");
    }

    #[test]
    fn value() {
        perfect_coding!("true");
        perfect_coding!("false");
        perfect_coding!("0");
        perfect_coding!("1");
        perfect_coding!("42");
        perfect_coding!("69");
        perfect_coding!("1337");
        perfect_coding!("10.");
        perfect_coding!("13.37");
        perfect_coding!(r#""hello""#);
        perfect_coding!(r#""hel\"lo""#);
    }

}
