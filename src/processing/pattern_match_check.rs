use ast::*;
use env::{Environment, TypeInfo};
use error::{TypeCheckError, MismatchedTypesError};
use processing::TypeCheck;

pub trait PatternMatchCheck: TypeCheck {
    fn check_match(&mut self,
                   rhs: &mut Self,
                   env: &mut Environment<TypeInfo>)
                   -> Result<(), TypeCheckError>;
}

impl PatternMatchCheck for Expr {
    fn check_match(&mut self,
                   rhs: &mut Self,
                   env: &mut Environment<TypeInfo>)
                   -> Result<(), TypeCheckError> {
        use ast::Expr::*;

        if let Variable { ref name, span } = *self {
            let mut assign = Expr::Assign {
                name: name.clone(),
                name_span: span,
                value: Box::new(rhs.clone()),
                // TODO
                value_span: Span(0, 0),
            };

            assign.type_check(env)?;
            Ok(())
        } else {
            let my_type = self.type_check(env)?;
            let rhs_type = rhs.type_check(env)?;

            if my_type != rhs_type {
                // TODO
                return Err(MismatchedTypesError::new(my_type.into(), rhs_type, Span(0, 0)).into());
            }

            Ok(())
        }
    }
}
