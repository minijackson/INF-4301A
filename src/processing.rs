use ast::*;

use std::collections::HashMap;

pub trait Evaluable {
	fn evaluate(&self, bindings: &mut HashMap<String, i32>) -> i32;
}

pub trait Printer {
	fn reverse_polish(&self) -> String;
}

impl Evaluable for Expr {
	fn evaluate(&self, bindings: &mut HashMap<String, i32>) -> i32 {
		use ast::Expr::*;
		use ast::BinaryOpCode::*;
		use ast::UnaryOpCode::*;

	match self {
		&Assignment(ref name, box ref exp) => {
			let value = exp.evaluate(bindings);
			bindings.insert(name.clone(), value);
			value
		},
			&Function(ref name, ref args) => {
				if name == "print" && args.len() == 1 {
					println!("=> {}", &args[0].evaluate(bindings));
					return 0;
				} else {
					panic!("Unknown function: {}/{}", name, args.len());
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
				&Minus => -exp.evaluate(bindings)
			}
		}
		&Variable(ref name) => {
			*bindings.get(name).expect(format!("Unbounded variable: {}", name).as_str())
		}
		&Num(value) => value,
	}
    }
}

impl Evaluable for Exprs {
	fn evaluate(&self, bindings: &mut HashMap<String, i32>) -> i32 {
		let mut value = 0;
		for expr in self.exprs.iter() {
			value = expr.evaluate(bindings);
		}
		value
	}
}

impl Printer for Expr {
	fn reverse_polish(&self) -> String {
		use ast::Expr::*;
		use ast::BinaryOpCode::*;
		use ast::UnaryOpCode::*;

		match self {
			&Assignment(ref name, box ref exp) => {
				format!("{} {} =", name, self.reverse_polish())
			}
			&Function(ref name, ref args) => {
				format!("{}{}",
						args.iter()
						.fold(String::new(),
							|s, arg| format!("{}{} ", s, arg.reverse_polish())),
						name)
			}
			&BinaryOp(box ref lhs, box ref rhs, ref op) => {
				match op {
					&Add => format!("{} {} +", &lhs.reverse_polish(), &rhs.reverse_polish()),
						&Sub => format!("{} {} -", &lhs.reverse_polish(), &rhs.reverse_polish()),
						&Mul => format!("{} {} *", &lhs.reverse_polish(), &rhs.reverse_polish()),
						&Div => format!("{} {} /", &lhs.reverse_polish(), &rhs.reverse_polish()),
				}
			}
			&UnaryOp(box ref exp, ref op) => {
				match op {
					&Plus => format!("{} ++", &exp.reverse_polish()),
						&Minus => format!("{} --", &exp.reverse_polish())
				}
			}
			&Variable(ref name) => {
				name.clone()
			}
			&Num(value) => value.to_string(),
		}
	}
}

impl Printer for Exprs {
	fn reverse_polish(&self) -> String {
		let mut result = "0";
		for expr in self.exprs.iter(){
			result = expr.reverse_polish().as_str();
		}
		result;
	}
}
pub fn lisp(tree: &Expr) -> String {
	use ast::Expr::*;
	use ast::BinaryOpCode::*;
	use ast::UnaryOpCode::*;

    match tree {
        &Assignment(ref name, box ref exp) => {
            format!("(let {} {})", name, lisp(exp))
        }
        &Function(ref name, ref args) => {
            format!("({}{})",
                    name,
                    args.iter()
                        .fold(String::new(),
                              |s, arg| format!("{} ({})", s, arg.reverse_polish())))
        }
        &BinaryOp(box ref lhs, box ref rhs, ref op) => {
            match op {
                &Add => format!("(+ {} {})", lisp(&lhs), lisp(&rhs)),
                &Sub => format!("(- {} {})", lisp(&lhs), lisp(&rhs)),
                &Mul => format!("(* {} {})", lisp(&lhs), lisp(&rhs)),
                &Div => format!("(/ {} {})", lisp(&lhs), lisp(&rhs)),
            }
        }
        &UnaryOp(box ref exp, ref op) => {
            match op {
                &Plus => format!("(++ {})", lisp(&exp)),
                &Minus => format!("(-- {})", lisp(&exp))
            }
        }
        &Variable(ref name) => {
            name.clone()
        }
        &Num(value) => value.to_string(),
    }
}

