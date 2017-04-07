use ast::*;
use env::{Environment, TypeInfo};
use type_sys::Type;
use type_sys::Type::*;

pub trait TypeCheck {
    fn type_check(&mut self, env: &mut Environment<TypeInfo>) -> Type;
}


impl TypeCheck for Exprs {
    fn type_check(&mut self, env: &mut Environment<TypeInfo>) -> Type {
        let mut final_type = Void;
        for expr in self.exprs.iter_mut() {
            final_type = expr.type_check(env);
        }
        final_type
    }
}
impl TypeCheck for Expr {
    fn type_check(&mut self, env: &mut Environment<TypeInfo>) -> Type {
        use ast::Expr::*;

        match self {
            &mut Grouping(ref mut exprs) => exprs.type_check(env),

            &mut Let(ref mut bindings, ref mut exprs) => {
                env.enter_scope();
                for binding in bindings.iter_mut() {
                    let type_ = binding.value.type_check(env);
                    env.declare(binding.variable.clone(),
                                TypeInfo {
                                    type_: type_,
                                    declaration: binding.clone(),
                                });
                }

                let final_type = exprs.type_check(env);
                env.leave_scope();
                final_type
            }

            &mut Assign(ref name, ref mut expr) => {
                let declared_type = env.get_var(name).expect("Unbounded variable").type_;
                let assign_type = expr.type_check(env);

                if declared_type != assign_type {
                    panic!("Assigning {} of type: {:?} with wrong type: {:?}",
                           name,
                           declared_type,
                           assign_type);
                }

                declared_type
            }

            &mut Function(ref name, ref mut args) => {
                let arg_types = args.iter_mut()
                    .map(|ref mut expr| expr.type_check(env))
                    .collect();

                match env.get_builtin(name).expect("Undefined function").return_type(&arg_types) {
                    Ok(return_type) => *return_type,
                    Err(_) => panic!("Undefined signature: {}{:?}", name, arg_types)
                }
            }

            &mut If(ref mut expr, ref mut true_branch, ref mut false_branch) => {
                if expr.type_check(env) == Void {
                    panic!("Conditional expression cannot be Void");
                }

                let true_branch_type = true_branch.type_check(env);
                let false_branch_type = false_branch.type_check(env);

                if true_branch_type != false_branch_type {
                    panic!("If arms have incompatible types: {:?} and {:?}",
                           true_branch_type,
                           false_branch_type);
                }

                true_branch_type
            }

            &mut While(ref mut cond, ref mut expr) => {
                if cond.type_check(env) == Void {
                    panic!("Conditional expression cannot be Void");
                }

                expr.type_check(env);

                Void
            }

            &mut BinaryOp(ref mut lhs, ref mut rhs, ref op) => {
                let arg_types = vec![lhs.type_check(env), rhs.type_check(env)];

                match env.get_builtin(&op.to_string()).expect("No such function").return_type(&arg_types) {
                    Ok(return_type) => *return_type,
                    Err(_) => panic!("Undefined signature: {}{:?}", op, arg_types)
                }
            }

            &mut UnaryOp(ref mut expr, ref op) => {
                let arg_types = vec![expr.type_check(env)];


                match env.get_builtin(&op.to_string()).expect("No such function").return_type(&arg_types) {
                    Ok(return_type) => *return_type,
                    Err(_) => panic!("Undefined signature: {}{:?}", op, arg_types)
                }
            }

            &mut Variable(ref name) => env.get_var(name).expect("Unbounded variable").type_,

            &mut Value(ref value) => value.get_type(),

        }
    }
}
