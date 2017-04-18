use ast::*;
use env::{Environment, BindingInfo, TypeInfo};
use error::{ConversionError, IncompatibleArmTypesError, MismatchedTypesError,
            NoSuchSignatureError, TypeCheckError, UnboundedVarError, UndefinedFunctionError,
            VoidVarDeclartionError};
use type_sys::Type;
use type_sys::Type::*;

pub trait TypeCheck {
    fn type_check(&mut self, env: &mut Environment<TypeInfo>) -> Result<Type, TypeCheckError>;
}

impl TypeCheck for Exprs {
    fn type_check(&mut self, env: &mut Environment<TypeInfo>) -> Result<Type, TypeCheckError> {
        let mut final_type = Void;
        for expr in &mut self.exprs {
            final_type = expr.type_check(env)?;
        }
        Ok(final_type)
    }
}

impl TypeCheck for Expr {
    fn type_check(&mut self, env: &mut Environment<TypeInfo>) -> Result<Type, TypeCheckError> {
        use ast::Expr::*;

        match *self {
            Grouping(ref mut exprs) => exprs.type_check(env),

            Let(ref mut bindings, ref mut function_decls, ref mut exprs) => {
                env.enter_scope();

                for binding in bindings.iter_mut() {
                    let type_ = binding.value.type_check(env)?;

                    if type_ == Type::Void {
                        return Err(VoidVarDeclartionError::new(binding.name.clone(),
                                                               binding.value_span)
                                           .into());
                    }

                    env.declare_var(binding.name.clone(),
                                     BindingInfo::Variable {
                                         declaration: binding.clone(),
                                         info: TypeInfo(type_),
                                     })
                        .map_err(|mut err| {
                                     err.span = binding.span;
                                     err
                                 })?;
                }

                for function_decl in function_decls.iter_mut() {
                    // Declare before cheking the type of the body, to allow
                    // recursion.
                    env.declare_func(function_decl.clone())?;
                    function_decl.type_check(env)?;
                }

                let final_type = exprs.type_check(env)?;

                env.leave_scope();
                Ok(final_type)
            }

            Assign {
                     ref name,
                     ref name_span,
                     ref mut value,
                     ref value_span,
                 } => {
                let assign_type = value.type_check(env)?;

                let var_info = env.get_var(name)
                    .ok_or_else(|| UnboundedVarError::new(name.clone(), *name_span))?;
                let declared_type = var_info.get_type();

                if declared_type != assign_type {
                    return Err(MismatchedTypesError::from_binding(var_info
                                                                      .get_declaration()
                                                                      .clone(),
                                                                  declared_type,
                                                                  assign_type,
                                                                  *value_span)
                                       .into());
                }

                Ok(declared_type)
            }

            Function {
                     ref name,
                     ref mut args,
                     ref span,
                 } => {
                let arg_types = args.iter_mut()
                    //.map(|&mut (ref mut expr, ref span)| expr.type_check(env).map(|arg_type| (arg_type, span)))
                    .map(|&mut (ref mut expr, _)| expr.type_check(env))
                    .collect::<Result<Vec<_>, _>>()?;

                let mut user_defined = false;
                let mut func = None;

                if let Some(user_func) = env.get_func(name) {
                    user_defined = true;
                    func = Some(user_func.clone());
                }

                if user_defined {
                    func.unwrap()
                        .return_type(&arg_types)
                        .ok_or_else(|| NoSuchSignatureError::new(name.clone(), arg_types.clone(), *span)
                                   .into())
                } else if let Some(builtin) = env.get_builtin_mut(name) {
                    builtin
                        .return_type(&arg_types)
                        .ok_or_else(|| NoSuchSignatureError::new(name.clone(), arg_types.clone(), *span)
                                   .into())
                } else {
                    Err(UndefinedFunctionError::new(name.clone(), *span).into())
                }
            }

            If {
                     ref mut cond,
                     ref cond_span,
                     ref mut true_branch,
                     ref true_branch_span,
                     ref mut false_branch,
                     ref false_branch_span,
                 } => {
                if cond.type_check(env)? == Void {
                    return Err(MismatchedTypesError::new(Bool, Void, *cond_span).into());
                }

                let true_branch_type = true_branch.type_check(env)?;
                let false_branch_type = false_branch.type_check(env)?;

                if true_branch_type != false_branch_type {
                    return Err(IncompatibleArmTypesError::new(true_branch_type,
                                                              false_branch_type,
                                                              *true_branch_span,
                                                              *false_branch_span)
                                       .into());
                }

                Ok(true_branch_type)
            }

            While {
                     ref mut cond,
                     ref mut expr,
                     ref cond_span,
                 } => {
                if cond.type_check(env)? == Void {
                    return Err(MismatchedTypesError::new(Bool, Void, *cond_span).into());
                }

                expr.type_check(env)?;

                Ok(Void)
            }

            For {
                     ref mut binding,
                     ref mut goal,
                     ref mut expr,
                     ref goal_span,
                 } => {
                env.enter_scope();

                let binding_type = binding.value.type_check(env)?;
                let goal_type = goal.type_check(env)?;

                if binding_type != Integer {
                    return Err(MismatchedTypesError::new(Integer,
                                                         binding_type,
                                                         binding.value_span)
                                       .into());
                }

                env.declare_var(binding.name.clone(),
                                 BindingInfo::Variable {
                                     declaration: (**binding).clone(),
                                     info: TypeInfo(binding_type),
                                 })?;

                if goal_type != Integer {
                    return Err(MismatchedTypesError::new(Integer, goal_type, *goal_span).into());
                }

                expr.type_check(env)?;

                env.leave_scope();
                Ok(Void)
            }

            BinaryOp {
                     ref mut lhs,
                     ref mut rhs,
                     ref op,
                     ref span,
                 } => {
                let arg_types = vec![lhs.type_check(env)?, rhs.type_check(env)?];

                let name = &op.to_string();

                env.get_builtin(name)
                    .ok_or_else(|| UndefinedFunctionError::new(name.clone(), *span))?
                    .return_type(&arg_types)
                    .ok_or_else(|| NoSuchSignatureError::new(name.clone(), arg_types.clone(), *span).into())
            }

            UnaryOp {
                     ref mut expr,
                     ref op,
                     ref span,
                 } => {
                let arg_types = vec![expr.type_check(env)?];

                let name = &format!("un{}", op.to_string());

                env.get_builtin(name)
                    .ok_or_else(|| UndefinedFunctionError::new(name.clone(), *span))?
                    .return_type(&arg_types)
                    .ok_or_else(|| NoSuchSignatureError::new(name.clone(), arg_types.clone(), *span).into())
            }

            Cast {
                     ref mut expr,
                     ref expr_span,
                     dest,
                 } => {
                let src_type = expr.type_check(env)?;

                if src_type.is_convertible_to(dest) {
                    Ok(dest)
                } else {
                    Err(ConversionError::new(src_type, dest, *expr_span).into())
                }

            }

            Variable { ref name, ref span } => {
                env.get_var(name)
                    .map(|var| var.get_type())
                    .ok_or_else(|| UnboundedVarError::new(name.clone(), *span).into())
            }

            Value(ref value) => Ok(value.get_type()),

        }
    }
}

impl TypeCheck for FunctionDecl {
    fn type_check(&mut self, env: &mut Environment<TypeInfo>) -> Result<Type, TypeCheckError> {
        env.enter_scope();

        for arg in &self.args {
            env.declare_var(arg.name.clone(),
                             BindingInfo::Argument {
                                 declaration: arg.clone(),
                                 info: TypeInfo(arg.type_),
                             })
                .map_err(|mut err| {
                             err.span = arg.span;
                             err
                         })?;
        }

        let final_type = self.body.type_check(env)?;

        if final_type != self.return_type {
            return Err(MismatchedTypesError::from_binding(Declaration::Function(self.clone()),
                                                          self.return_type,
                                                          final_type,
                                                          self.body_span)
                               .into());
        }

        env.leave_scope();
        Ok(self.return_type)
    }
}
