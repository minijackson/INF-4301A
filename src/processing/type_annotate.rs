use ast::*;

use std::collections::HashMap;

pub trait TypeAnnotate {
    type Typed;

    fn type_annotate(&self, var_types: &mut HashMap<String, Type>) -> Self::Typed;
}

impl TypeAnnotate for NaiveExpr {
    type Typed = Expr;

    fn type_annotate(&self, var_types: &mut HashMap<String, Type>) -> Expr {
        use ast::NaiveExpr::*;

        match self {

            &Grouping(ref exprs) => {
                let typed_exprs = exprs.type_annotate(var_types);
                let my_type = typed_exprs.type_annotation;
                Expr { kind: ExprKind::Grouping(typed_exprs), type_annotation: my_type }
            }

            &Let(ref assignments, ref exprs) => {
                let typed_assignments: Vec<_> = assignments.iter()
                    .map(|assign| {
                        let ref variable = assign.variable;
                        let typed_value = assign.value.type_annotate(var_types);
                        var_types.insert(variable.clone(), typed_value.type_annotation);
                        Binding { variable: variable.clone(), value: typed_value }
                    })
                    .collect();

                let typed_exprs = exprs.type_annotate(var_types);
                let my_type = typed_exprs.type_annotation;
                Expr { kind: ExprKind::Let(typed_assignments, typed_exprs), type_annotation: my_type }
            }

            &Function(ref name, ref args) => {
                if name == "print" && args.len() == 1 {
                    let typed_args = args.iter()
                        .map(|expr| Box::new(expr.type_annotate(var_types)))
                        .collect();
                    Expr { kind: ExprKind::Function(name.clone(), typed_args), type_annotation: Type::Integer }
                } else {
                    panic!("Unknown function: {}/{}", name, args.len());
                }
            }

            &If(box ref cond, ref true_branch, ref false_branch) => {
                let typed_cond = Box::new(cond.type_annotate(var_types));
                let typed_true_branch = Box::new(true_branch.type_annotate(var_types));
                let typed_false_branch = Box::new(false_branch.type_annotate(var_types));

                if typed_true_branch.type_annotation != typed_false_branch.type_annotation {
                    panic!("If true branch and false branch does not return the same type");
                }

                let my_type = typed_true_branch.type_annotation;

                Expr { kind: ExprKind::If(typed_cond, typed_true_branch, typed_false_branch), type_annotation: my_type }
            }

            &BinaryOp(box ref lhs, box ref rhs, ref op) => {
                let typed_lhs = Box::new(lhs.type_annotate(var_types));
                let typed_rhs = Box::new(rhs.type_annotate(var_types));

                if typed_lhs.type_annotation != typed_lhs.type_annotation {
                    panic!("Left and right hand side does not return the same type");
                }

                let my_type = typed_lhs.type_annotation;

                Expr { kind: ExprKind::BinaryOp(typed_lhs, typed_rhs, *op), type_annotation: my_type }
            }

            &UnaryOp(box ref exp, ref op) => {
                let typed_exp = Box::new(exp.type_annotate(var_types));
                let my_type = typed_exp.type_annotation;
                Expr { kind: ExprKind::UnaryOp(typed_exp, *op), type_annotation: my_type }
            }

            &Variable(ref name) => {
                Expr { kind: ExprKind::Variable(name.clone()), type_annotation: *var_types.get(name).expect(format!("Could not find variable {}", name).as_str()) }
            }

            &Num(value) => Expr { kind: ExprKind::Num(value), type_annotation: Type::Integer }

        }
    }
}

impl TypeAnnotate for NaiveExprs {
    type Typed = Exprs;

    fn type_annotate(&self, var_types: &mut HashMap<String, Type>) -> Exprs {
        if self.exprs.len() == 0 {
            panic!("Expressions of nothingness");
        }

        let typed_exprs: Vec<_> = self.exprs.iter()
            .map(|expr| Box::new(expr.type_annotate(var_types)))
            .collect();

        let my_type = typed_exprs.last().unwrap().type_annotation;

        Exprs { exprs: typed_exprs, type_annotation: my_type }
    }
}
