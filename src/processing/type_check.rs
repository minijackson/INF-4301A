use ast::*;
use env::{Environment, BindingInfo, TypeInfo};
use error::{IncompatibleArmTypesError, MismatchedTypesError, NoSuchSignatureError, TypeCheckError};
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
                                 BindingInfo {
                                     declaration: binding.clone(),
                                     info: TypeInfo(type_),
                                 })
                    .map_err(|mut err| {
                        err.span = binding.span;
                        err
                    })?;
                }

                let final_type = exprs.type_check(env)?;
                env.leave_scope();
                Ok(final_type)
            }

            &mut Assign {
                     ref name,
                     ref mut value,
                     ref value_span,
                 } => {
                let assign_type = value.type_check(env)?;

                let var_info = env.get_var(name)?;
                let declared_type = var_info.info.0;

                if declared_type != assign_type {
                    return Err(
                        TypeCheckError::MismatchedTypes(
                            MismatchedTypesError::from_binding(
                                var_info.declaration.clone(),
                                declared_type,
                                assign_type,
                                *value_span
                                )
                            )
                        );
                }

                Ok(declared_type)
            }

            &mut Function {
                     ref name,
                     ref mut args,
                     ref span,
                 } => {
                let arg_types = args.iter_mut()
                    //.map(|&mut (ref mut expr, ref span)| expr.type_check(env).map(|arg_type| (arg_type, span)))
                    .map(|&mut (ref mut expr, _)| expr.type_check(env))
                    .collect::<Result<_, _>>()?;

                env.get_builtin(name)?
                    .return_type(&arg_types)
                    .map(|return_type| *return_type)
                    .ok_or(TypeCheckError::NoSuchSignature(NoSuchSignatureError::new(name.clone(), arg_types.clone(), *span)))
            }

            &mut If {
                     ref mut cond,
                     ref mut true_branch,
                     ref mut false_branch,
                     ref cond_span,
                     ref false_branch_span,
                 } => {
                if cond.type_check(env)? == Void {
                    return Err(TypeCheckError::MismatchedTypes(MismatchedTypesError::new(Bool,
                                                                                         Void,
                                                                                         *cond_span)));
                }

                let true_branch_type = true_branch.type_check(env)?;
                let false_branch_type = false_branch.type_check(env)?;

                if true_branch_type != false_branch_type {
                    return Err(TypeCheckError::IncompatibleArmTypes(IncompatibleArmTypesError::new(
                           true_branch_type,
                           false_branch_type,
                           *false_branch_span)));
                }

                Ok(true_branch_type)
            }

            &mut While {
                     ref mut cond,
                     ref mut expr,
                     ref cond_span,
                 } => {
                if cond.type_check(env)? == Void {
                    return Err(TypeCheckError::MismatchedTypes(MismatchedTypesError::new(Bool,
                                                                                         Void,
                                                                                         *cond_span)));
                }

                expr.type_check(env)?;

                Ok(Void)
            }

            &mut For {
                     ref mut binding,
                     ref mut goal,
                     ref mut expr,
                     ref goal_span,
                 } => {
                env.enter_scope();

                let binding_type = binding.value.type_check(env)?;
                let goal_type = goal.type_check(env)?;

                if binding_type != Integer {
                    return Err(TypeCheckError::MismatchedTypes(MismatchedTypesError::new(Integer,
                                                                                         binding_type,
                                                                                         binding.value_span)));
                }

                env.declare(binding.variable.clone(),
                             BindingInfo {
                                 declaration: (**binding).clone(),
                                 info: TypeInfo(binding_type),
                             })?;

                if goal_type != Integer {
                    return Err(TypeCheckError::MismatchedTypes(MismatchedTypesError::new(Integer,
                                                                                         goal_type,
                                                                                         *goal_span)));
                }

                expr.type_check(env)?;

                env.leave_scope();
                Ok(Void)
            }

            &mut BinaryOp(ref mut lhs, ref mut rhs, ref op) => {
                let arg_types = vec![lhs.type_check(env)?, rhs.type_check(env)?];

                let name = &op.to_string();

                env.get_builtin(name)?
                    .return_type(&arg_types)
                    .map(|return_type| *return_type)
                    // TODO
                    .ok_or(TypeCheckError::NoSuchSignature(NoSuchSignatureError::new(name.clone(), arg_types.clone(), Span(0, 0))))
            }

            &mut UnaryOp(ref mut expr, ref op) => {
                let arg_types = vec![expr.type_check(env)?];

                let name = &format!("un{}", op.to_string());

                env.get_builtin(name)?
                    .return_type(&arg_types)
                    .map(|return_type| *return_type)
                    .ok_or(TypeCheckError::NoSuchSignature(NoSuchSignatureError::new(name.clone(), arg_types.clone(), Span(0, 0))))
            }

            &mut Variable(ref name) => {
                env.get_var(name)
                    .map(|var| var.info.0)
                    .map_err(TypeCheckError::UnboundedVar)
            }

            &mut Value(ref value) => Ok(value.get_type()),

        }
    }
}
