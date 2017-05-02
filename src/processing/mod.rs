//! The module that gather modules that parse an entire AST.
//!
//! The way this is done is by defining a trait inside that module, that will be implemented by
//! [`ast::Expr`], [`ast::Exprs`] and sometimes more, a call recursively this trait's function(s).
//!
//! [`ast::Expr`]: ../ast/enum.Expr.html
//! [`ast::Exprs`]: ../ast/struct.Exprs.html

mod evaluate;
mod print;
mod type_check;
pub mod pattern_match_check;
pub mod pattern_match;

pub use self::evaluate::*;
pub use self::print::*;
pub use self::type_check::*;
