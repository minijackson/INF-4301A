use ast::*;
use env::{Environment, TypeInfo};
use error::{TypeCheckError, MismatchedTypesError, IncompatibleArmTypesError};
use type_sys::Type;
use type_sys::Type::*;

pub trait TypeCheck {
    fn type_check(&mut self, env: &mut Environment<TypeInfo>) -> Result<Type, TypeCheckError>;
}


impl TypeCheck for Exprs {
    fn type_check(&mut self, env: &mut Environment<TypeInfo>) -> Result<Type, TypeCheckError> {
        let mut final_type = Void;
        for expr in self.exprs.iter_mut() {
            final_type = expr.type_check(env)?;
        }
        Ok(final_type)
    }
}
impl TypeCheck for Expr {
    fn type_check(&mut self, env: &mut Environment<TypeInfo>) -> Result<Type, TypeCheckError> {
        use ast::Expr::*;

        match self {
            &mut Grouping(ref mut exprs) => exprs.type_check(env),

            &mut Let(ref mut bindings, ref mut exprs) => {
                env.enter_scope();
                for binding in bindings.iter_mut() {
                    let type_ = binding.value.type_check(env)?;
                    env.declare(binding.variable.clone(),
                                 TypeInfo {
                                     type_,
                                     declaration: binding.clone(),
                                 })
                        .map_err(TypeCheckError::AlreadyDeclared)?;
                }

                let final_type = exprs.type_check(env)?;
                env.leave_scope();
                Ok(final_type)
            }

            &mut Assign(ref name, ref mut expr) => {
                let assign_type = expr.type_check(env)?;

                let var_info = env.get_var(name).map_err(TypeCheckError::UnboundedVar)?;
                let declared_type = var_info.type_;

                if declared_type != assign_type {
                    return Err(
                        TypeCheckError::MismatchedTypes(
                            MismatchedTypesError::from_binding(
                                var_info.declaration.clone(),
                                declared_type,
                                assign_type)
                            )
                        );
                }

                Ok(declared_type)
            }

            &mut Function(ref name, ref mut args) => {
                let arg_types = args.iter_mut()
                    .map(|ref mut expr| expr.type_check(env))
                    .collect::<Result<_, _>>()?;

                env.get_builtin(name)
                    .map_err(TypeCheckError::UndefinedFunction)?
                    .return_type(&arg_types)
                    .map_err(TypeCheckError::NoSuchSignature)
                    .map(|return_type| *return_type)
            }

            &mut If(ref mut expr, ref mut true_branch, ref mut false_branch) => {
                if expr.type_check(env)? == Void {
                    return Err(TypeCheckError::MismatchedTypes(MismatchedTypesError::new(Bool,
                                                                                         Void)));
                }

                let true_branch_type = true_branch.type_check(env)?;
                let false_branch_type = false_branch.type_check(env)?;

                if true_branch_type != false_branch_type {
                    return Err(TypeCheckError::IncompatibleArmTypes(IncompatibleArmTypesError::new(
                           true_branch_type,
                           false_branch_type)));
                }

                Ok(true_branch_type)
            }

            &mut While(ref mut cond, ref mut expr) => {
                if cond.type_check(env)? == Void {
                    return Err(TypeCheckError::MismatchedTypes(MismatchedTypesError::new(Bool,
                                                                                         Void)));
                }

                expr.type_check(env)?;

                Ok(Void)
            }

            &mut BinaryOp(ref mut lhs, ref mut rhs, ref op) => {
                let arg_types = vec![lhs.type_check(env)?, rhs.type_check(env)?];

                env.get_builtin(&op.to_string())
                    .map_err(TypeCheckError::UndefinedFunction)?
                    .return_type(&arg_types)
                    .map_err(TypeCheckError::NoSuchSignature)
                    .map(|return_type| *return_type)
            }

            &mut UnaryOp(ref mut expr, ref op) => {
                let arg_types = vec![expr.type_check(env)?];


                env.get_builtin(&format!("un{}", op.to_string()))
                    .map_err(TypeCheckError::UndefinedFunction)?
                    .return_type(&arg_types)
                    .map_err(TypeCheckError::NoSuchSignature)
                    .map(|return_type| *return_type)
            }

            &mut Variable(ref name) => {
                env.get_var(name)
                    .map_err(TypeCheckError::UnboundedVar)
                    .map(|var| var.type_)
            }

            &mut Value(ref value) => Ok(value.get_type()),

        }
    }
}
