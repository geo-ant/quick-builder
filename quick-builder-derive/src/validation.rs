//! this module helps us pass the #[validate(my_function)] and e.g.
//! #[validate(|arg| arg.len() == 3]. Valid expressions are closures with
//! one argument returning a bool, or paths that must point to a function
//! with one argument returning a bool. The argument must be of type
//! `&Foo` where `Foo` is the structure for which we created the builder.

use syn::{
    parse::{Parse, ParseBuffer},
    Expr, ExprClosure, Meta, Path,
};

use crate::error::CompileError;

pub struct ValidateAttribute {
    /// the expression in brackets in the validation attribute
    expression: ValidationExpression,
}

/// the expression for validation
enum ValidationExpression {
    /// a closure is defined
    /// (there are some aspects that we can verify, but not all)
    Closure(ExprClosure),
    /// a path to a function is given
    /// (there's nothing more about this we can verify at macro expansion time)
    Path(Path),
}

impl TryFrom<&Meta> for ValidationExpression {
    type Error = CompileError;
    fn try_from(meta: &Meta) -> Result<Self, CompileError> {
        match meta {
            Meta::Path(path) => Ok(Self::Path(path.clone())),
            Meta::List(list) => {
                let closure: Expr = syn::parse(list.tokens.clone().into()).unwrap();
                todo!()
            }
            Meta::NameValue(value) => Err(CompileError::new_spanned(
                value,
                "QuickBuilder: Attribute arguments must either be a function name or a closure.",
            )),
        }
    }
}
