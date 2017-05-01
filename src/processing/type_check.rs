use ast::*;
use env::{Environment, BindingInfo, TypeInfo};
use error::{ArrayTypeDecl, ConversionError, IncompatibleArmTypesError,
            InconsistentArrayTypingError, MismatchedTypesError, NoSuchSignatureError,
            TypeCheckError, UnboundedVarError, UndefinedFunctionError, UntypedEmptyArrayError,
            VoidVarDeclartionError};
use processing::pattern_match_check::PatternMatchCheck;
use type_sys::Type;

pub trait TypeCheck {
    fn type_check(&mut self, env: &mut Environment<TypeInfo>) -> Result<Type, TypeCheckError>;
}

impl TypeCheck for Exprs {
    fn type_check(&mut self, env: &mut Environment<TypeInfo>) -> Result<Type, TypeCheckError> {
        let mut final_type = Type::Void;
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

                let var_info =
                    env.get_var(name)
                        .ok_or_else(|| UnboundedVarError::new(name.clone(), *name_span))?;
                let declared_type = var_info.get_type();

                if *declared_type != assign_type {
                    return Err(MismatchedTypesError::from_binding(var_info
                                                                      .get_declaration()
                                                                      .clone(),
                                                                  declared_type.clone().into(),
                                                                  assign_type,
                                                                  *value_span)
                                       .into());
                }

                Ok(declared_type.clone())
            }

            PatternMatch {
                ref mut lhs,
                ref mut rhs,
                ..
            } => {
                lhs.check_match(rhs, env)?;
                Ok(Type::Bool)
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
                        .cloned()
                        .ok_or_else(|| {
                                        NoSuchSignatureError::new(name.clone(), arg_types, *span)
                                            .into()
                                    })
                } else if let Some(builtin) = env.get_builtin(name) {
                    builtin
                        .return_type(&arg_types, &env.types)
                        .ok_or_else(|| {
                                        NoSuchSignatureError::new(name.clone(), arg_types, *span)
                                            .into()
                                    })
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
                let cond_type = cond.type_check(env)?;
                if !cond_type.may_truthy() {
                    return Err(ConversionError::new(cond_type, Type::Bool, *cond_span).into());
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
                let cond_type = cond.type_check(env)?;
                if !cond_type.may_truthy() {
                    return Err(ConversionError::new(cond_type, Type::Bool, *cond_span).into());
                }

                expr.type_check(env)?;

                Ok(Type::Void)
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

                if binding_type != Type::Integer {
                    return Err(MismatchedTypesError::new(Type::Integer.into(),
                                                         binding_type,
                                                         binding.value_span)
                                       .into());
                }

                if goal_type != binding_type {
                    return Err(MismatchedTypesError::new(binding_type.into(),
                                                         goal_type,
                                                         *goal_span)
                                       .into());
                }

                env.declare_var(binding.name.clone(),
                                 BindingInfo::Variable {
                                     declaration: (**binding).clone(),
                                     info: TypeInfo(binding_type),
                                 })?;

                expr.type_check(env)?;

                env.leave_scope();
                Ok(Type::Void)
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
                    .return_type(&arg_types, &env.types)
                    .ok_or_else(|| {
                                    NoSuchSignatureError::new(name.clone(),
                                                              arg_types.clone(),
                                                              *span)
                                            .into()
                                })
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
                    .return_type(&arg_types, &env.types)
                    .ok_or_else(|| {
                                    NoSuchSignatureError::new(name.clone(),
                                                              arg_types.clone(),
                                                              *span)
                                            .into()
                                })
            }

            Cast {
                ref mut expr,
                ref expr_span,
                ref dest,
            } => {
                let src_type = expr.type_check(env)?;

                if src_type.is_convertible_to(dest) {
                    Ok(dest.clone())
                } else {
                    Err(ConversionError::new(src_type, dest.clone(), *expr_span).into())
                }

            }

            Variable { ref name, ref span } => {
                env.get_var(name)
                    .map(|var| var.get_type())
                    .cloned()
                    .ok_or_else(|| UnboundedVarError::new(name.clone(), *span).into())
            }

            Array {
                ref mut values,
                ref mut declared_type,
                declared_type_span,
                span,
            } => {
                let first_span;

                let type_ = match *declared_type {
                    Some(ref type_) => {
                        first_span = None;
                        type_.clone()
                    }
                    None => {
                        let type_ = values
                            .get_mut(0)
                            .map(|&mut (ref mut value, _)| value)
                            .ok_or_else(|| UntypedEmptyArrayError::new(span))?
                            .type_check(env)?;

                        first_span = Some(values[0].1);

                        type_
                    }
                };

                *declared_type = Some(type_.clone());

                let types = values
                    .iter_mut()
                    .map(|&mut (ref mut expr, _)| expr.type_check(env))
                    .collect::<Result<Vec<_>, _>>()?;

                let intruder = types
                    .iter()
                    .position(|candidate_type| *candidate_type != type_);

                if let Some(pos) = intruder {
                    let wrong_type = types[pos].clone();
                    let span = values[pos].1;

                    return Err(InconsistentArrayTypingError {
                                       expected: type_,
                                       got: wrong_type,
                                       argument_id: pos,
                                       span,
                                       type_decl: if let Some(span) = first_span {
                                           ArrayTypeDecl::FirstElem(span)
                                       } else {
                                           ArrayTypeDecl::Explicit(declared_type_span.unwrap())
                                       },
                                   }
                                   .into());
                }

                Ok(Type::Array(Box::new(type_)))
            }

            Tuple(ref mut exprs) => {
                let element_types = exprs
                    .iter_mut()
                    .map(|expr| expr.type_check(env))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Type::Tuple(element_types))
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
                                 info: TypeInfo(arg.type_.clone()),
                             })
                .map_err(|mut err| {
                             err.span = arg.span;
                             err
                         })?;
        }

        let final_type = self.body.type_check(env)?;

        if final_type != self.return_type {
            return Err(MismatchedTypesError::from_binding(Declaration::Function(self.clone()),
                                                          self.return_type.clone().into(),
                                                          final_type,
                                                          self.body_span)
                               .into());
        }

        env.leave_scope();
        Ok(self.return_type.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::TypeCheck;

    use env::Environment;
    use error::*;
    use parser;
    use type_sys::Generic;
    use type_sys::Type::*;

    macro_rules! assert_err {

        ( $expr:expr, $err:pat ) => {
            let res = parser::parse_Expression($expr)
                .unwrap()
                .type_check(&mut Environment::new());
            assert!(match res {
                Err($err) => true,
                _ => false,
            });
        };

        ( $expr:expr, $err:pat if $guard:expr ) => {
            let res = parser::parse_Expression($expr)
                .unwrap()
                .type_check(&mut Environment::new());
            assert!(match res {
                Err($err) if $guard => true,
                _ => false,
            });
        };

    }

    macro_rules! assert_type {
        ( $expr:expr, $ok:expr ) => {
            assert_eq!(parser::parse_Expression($expr)
                           .unwrap()
                           .type_check(&mut Environment::new()),
                       Ok($ok));
        }
    }

    #[test]
    fn grouping() {

        // Report inner errors
        assert_err!("(2, 3+4.)",
            TypeCheckError::NoSuchSignature(
                NoSuchSignatureError { ref func_name, .. }
                )
            if func_name == "+"
            );

        assert_type!("(1, 2)", Integer);
        assert_type!("(true, 2)", Integer);
    }

    #[test]
    fn let_block() {
        use ast::Declaration;

        assert_type!("let in end", Void);
        assert_type!("let in 2 end", Integer);
        assert_type!("let in true, 2 end", Integer);

        // Report inner errors
        assert_err!("let in 2 + 3.4 end",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");
        assert_err!("let
                        var x := 2 + 3.4
                    in
                    end",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");

        assert_err!("let
                        var x := while 1 do 1
                    in
                    end",
                    TypeCheckError::VoidVarDeclaration(VoidVarDeclartionError { ref name, ..  })
                    if name == "x");

        assert_err!("let
                        var x := 1
                        var x := 2
                    in
                    end",
                    TypeCheckError::AlreadyDeclared(AlreadyDeclaredError {
                        ref name,
                        orig_declaration: Declaration::Variable(_),
                        ..
                    })
                    if name == "x"
                   );

        assert_err!("let
                        function x(): Integer := 1
                        function x(): Integer := 2
                    in
                    end",
            TypeCheckError::AlreadyDeclared(
                AlreadyDeclaredError {
                    ref name,
                    orig_declaration: Declaration::Function(_),
                    ..
                }
                )
            if name == "x"
            );

        assert_err!("let
                        function a(x: Bool, x: Bool): Integer := 1
                    in
                    end",
                    TypeCheckError::AlreadyDeclared(AlreadyDeclaredError {
                        ref name,
                        orig_declaration: Declaration::Argument(_),
                        ..
                    })
                    if name == "x"
                   );

        assert_type!("let
                        var x := 1
                     in
                        x
                     end",
                     Integer);
        assert_type!("let
                         var x := true
                     in
                         x
                     end",
                     Bool);
        assert_type!("let
                         function x(): Bool := true
                     in
                        x()
                     end",
                     Bool);
    }

    #[test]
    fn assign() {
        use ast::Declaration;

        assert_err!("let var x := 0 in x := 2+3.4 end",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");

        assert_err!("x := 42",
                    TypeCheckError::UnboundedVar(UnboundedVarError { ref name, .. })
                    if name == "x");

        assert_err!("let
                        function x(): Bool := true
                    in
                        x := 42
                    end",
                    TypeCheckError::UnboundedVar(UnboundedVarError { ref name, .. })
                    if name == "x");

        assert_err!("let
                        function x(x: Bool): Bool := (x := 42, true)
                    in
                    end",
                    TypeCheckError::MismatchedTypes(MismatchedTypesError {
                        expected: Generic::Builtin(Bool),
                        got: Integer,
                        binding: Some(Declaration::Argument(ref arg)),
                        ..
                    })
                    if arg.name == "x");

        assert_type!("let
                         function x(x: Bool): Bool := (x := false, x)
                     in
                     end",
                     Void);
    }

    #[test]
    fn pattern_match() {
        assert_type!("match 1 := 1", Bool);

        // Real pattern match tests are in the src/processing/pattern_match.rs and
        // src/processing/pattern_match_check.rs files.
    }

    #[test]
    fn function() {
        assert_err!("hello()",
                    TypeCheckError::UndefinedFunction(UndefinedFunctionError { ref name, .. })
                    if name == "hello");

        assert_err!("let
                        function x(x: Bool): Bool := true
                    in
                        x(2 + 3.4)
                    end",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");

        assert_err!("let
                        function x(x: Bool, y: Bool): Bool := true
                    in
                        x(true, 2 + 3.4)
                    end",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");

        assert_type!("let
                         function x(): Bool := true
                     in
                        x()
                     end",
                     Bool);

        assert_err!("println()",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "println");

        assert_err!("println(1, 2, 3)",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "println");

        assert_err!("let
                        function x(x: Bool): Bool := true
                    in
                        x()
                    end",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "x");

        assert_err!("let
                        function x(x: Bool): Bool := true
                    in
                        x(1, 2, 3)
                    end",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "x");

        assert_err!("let
                        function x(x: Bool): Bool := true
                    in
                        x(42)
                    end",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "x");

        assert_type!("let
                        function fact(n: Integer): Integer := if n then n*fact(n-1) else 1
                     in
                        fact(5)
                     end",
                     Integer);
    }

    #[test]
    fn if_block() {
        assert_err!("if 2+3.4 then 1 else 2",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");
        assert_err!("if true then 2+3.4 else 2",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");
        assert_err!("if true then 1 else 2+3.4",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");

        assert_type!("if true then true else false", Bool);
        assert_type!("if true then 1 else 2", Integer);
        assert_type!("if 1 then 1 else 2", Integer);
        assert_type!("if [42] then 1 else 2", Integer);

        assert_err!("if true then true else 2",
                    TypeCheckError::IncompatibleArmTypes(IncompatibleArmTypesError {
                        expected: Bool,
                        got: Integer,
                        ..
                    }));

        assert_err!(r#"if "hello" then 1 else 2"#,
                    TypeCheckError::Conversion(ConversionError {
                        from: Str,
                        to: Bool,
                        ..
                    }));
    }

    #[test]
    fn while_block() {
        assert_err!("while 2+3.4 do 1",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");
        assert_err!("while true do 2+3.4",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");

        assert_type!("while true do true", Void);
        assert_type!("while true do 1", Void);
        assert_type!("while 1 do 1", Void);
        assert_type!("while [42] do 1", Void);

        assert_err!(r#"while "hello" do 1"#,
                    TypeCheckError::Conversion(ConversionError {
                        from: Str,
                        to: Bool,
                        ..
                    }));
    }

    #[test]
    fn for_block() {
        assert_err!("for var x := 2+3.4 to 10 do x",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");
        assert_err!("for var x := 1 to 2+3.4 do x",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");
        assert_err!("for var x := 1 to 10 do 2+3.4",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");

        assert_type!("for var x := 1 to 10 do 1", Void);
        assert_type!("for var x := 1 to 10 do x", Void);

        assert_err!("for var x := 1.5 to 10 do x",
                    TypeCheckError::MismatchedTypes(MismatchedTypesError {
                        expected: Generic::Builtin(Integer),
                        got: Float,
                        ..
                    }));
        assert_err!("for var x := 1 to 1.5 do x",
                    TypeCheckError::MismatchedTypes(MismatchedTypesError {
                        expected: Generic::Builtin(Integer),
                        got: Float,
                        ..
                    }));
    }

    #[test]
    fn binary_ops() {
        assert_type!("2+2", Integer);
        assert_type!("2-2", Integer);
        assert_type!("2*2", Integer);
        assert_type!("2/2", Integer);

        assert_type!("2.+2.", Float);
        assert_type!("2.-2.", Float);
        assert_type!("2.*2.", Float);
        assert_type!("2./2.", Float);

        assert_err!("2+3.4",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");
        // This is not JavaScript here, please
        assert_err!(r#"2-"3""#,
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "-");

        assert_type!("2=2", Bool);
        assert_type!(r#""hello"="world""#, Bool);
        //assert_type!("[2]=[2]", Bool);
        assert_type!("2<>2", Bool);
        assert_type!("2>2", Bool);
        assert_type!("2>=2", Bool);
        assert_type!("2<2", Bool);
        assert_type!("2<=2", Bool);
    }

    #[test]
    fn unary_ops() {
        assert_type!("-2", Integer);
        assert_type!("-2.", Float);
        assert_type!("+2", Integer);
        assert_type!("+2.", Float);

        assert_err!(r#"-"2""#,
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "un-");
    }

    #[test]
    fn cast() {
        assert_type!("2 as Float", Float);
        assert_type!("2 as Bool", Bool);
        assert_type!("[1, 2, 3] as Array(Float)", Array(Box::new(Float)));
        assert_type!("{1, 2, 3} as Tuple(Integer, Float, Bool)", Tuple(vec![Integer, Float, Bool]));
        assert_type!("{1, 2, 3} as Array(Float)", Array(Box::new(Float)));

        assert_err!("[1, 2, 3] as Integer",
                    TypeCheckError::Conversion(ConversionError {
                        ref from,
                        to: Integer,
                        ..
                    })
                    if *from == Array(Box::new(Integer)));

        assert_err!("{1, 2, 3} as Integer",
                    TypeCheckError::Conversion(ConversionError {
                        ref from,
                        to: Integer,
                        ..
                    })
                    if *from == Tuple(vec![Integer, Integer, Integer]));
    }

    #[test]
    fn variable() {
        assert_type!("let
                        var x := 3
                     in
                        x
                     end",
                     Integer);
        assert_type!("let
                        var x := true
                     in
                        x
                     end",
                     Bool);

        assert_err!("x",
                    TypeCheckError::UnboundedVar(UnboundedVarError { ref name, .. })
                    if name == "x");
    }

    #[test]
    fn array() {
        assert_err!("[2+3.4]",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");
        assert_err!("[1, 2+3.4]",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");

        assert_type!("Bool[]", Array(Box::new(Bool)));
        assert_type!("[1, 2, 3]", Array(Box::new(Integer)));
        assert_type!("[1., 2., 3.]", Array(Box::new(Float)));

        assert_err!("[]",
                    TypeCheckError::UntypedEmptyArray(UntypedEmptyArrayError { .. }));
        assert_err!("Integer[true]",
                    TypeCheckError::InconsistentArrayTyping(InconsistentArrayTypingError {
                        expected: Integer,
                        got: Bool,
                        argument_id: 0,
                        ..
                    }));
        assert_err!("[true, 1]",
                    TypeCheckError::InconsistentArrayTyping(InconsistentArrayTypingError {
                        expected: Bool,
                        got: Integer,
                        argument_id: 1,
                        ..
                    }));
    }

    #[test]
    fn tuple() {
        assert_err!("{2+3.4}",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");
        assert_err!("{1, 2+3.4}",
                    TypeCheckError::NoSuchSignature(NoSuchSignatureError { ref func_name, .. })
                    if func_name == "+");

        assert_type!("{}", Tuple(vec![]));
        assert_type!("{1}", Tuple(vec![Integer]));
        assert_type!("{1, true}", Tuple(vec![Integer, Bool]));
    }

    #[test]
    fn value() {
        assert_type!("2", Integer);
        assert_type!("true", Bool);
        assert_type!(r#""2""#, Str);
    }
}
