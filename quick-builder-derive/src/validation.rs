//! this module helps us pass the #[invariant(my_function)] and e.g.
//! #[invariant(|arg| arg.len() == 3]. Valid expressions are closures with
//! one argument returning a bool, or paths that must point to a function
//! with one argument returning a bool. The argument must be of type
//! `&Foo` where `Foo` is the structure for which we created the builder.
use proc_macro2::Span;
use quote::ToTokens;
use syn::{spanned::Spanned, visit::Visit, Attribute, ExprClosure, Meta, Path};

const INVARIANT_ATTR: &str = "invariant";

use crate::{builder::FINISHED_VALUE_IDENT, error::CompileError};

// #[derive(Debug)]
/// the invariant-attribute:
/// the struct itself can have 0 or 1 of these attributes and each field
/// can have 0 or one of these attributes.
///
/// The attribute itself comes in one of two forms
/// EITHER `#[invariant(function_name)]`, where `function_name` is the path to a
/// function
/// OR `#[invariant(|x| !x.is_nan())]`, where the expression in brackets is a closure.
/// For fields, the closure argument is deduced to be reference-to-field (except
/// if the field is itself a reference, then the reference is passed directly).
///
/// If the attribute is on the struct, the finished struct itself is passed for
/// validation before it is built.
pub struct InvariantAttribute {
    /// the expression in brackets in the validation attribute
    expression: InvariantExpression,
}

impl InvariantAttribute {
    /// get the expression for validation as tokens. This is just the function name
    /// or the code of the closure inside the attribute braces. No additional magic has been performed.
    pub fn expression(&self) -> impl ToTokens + '_ {
        &self.expression
    }

    pub fn expression_span(&self) -> Span {
        self.expression.full_span()
    }
}

impl InvariantAttribute {
    /// try parsing the invariant attribute from a list of attributes of a field.
    /// A field may have ZERO or ONE invariant attributes, hence the Option<...>.
    /// The option has None value if the field does not have a invariant attribute,
    /// otherwise it is Some(...). If an error occurs during parsing, or if more than
    /// one invariant attribute is present, returns an error.
    pub fn new(attributes: &[Attribute]) -> Result<Option<Self>, CompileError> {
        // helper predicate that helps us find the invariant attribute
        // this does not check whether the construct itself is valid, but only whether
        // the attribute is *called* correctly.
        let is_invariant_attribute = |attr: &Attribute| match attr.meta {
            Meta::Path(ref path) => path.is_ident(INVARIANT_ATTR),
            Meta::List(ref list) => list.path.is_ident(INVARIANT_ATTR),
            Meta::NameValue(ref name_value) => name_value.path.is_ident(INVARIANT_ATTR),
        };

        // get the zero or one invariant attributes
        // return an error if one was encountered, also return an error when more
        // than one attribute exists.
        let result: Result<Option<&Attribute>, CompileError> =
            attributes.iter().try_fold(None, |init, curr| match init {
                None => {
                    if is_invariant_attribute(curr) {
                        Ok(Some(curr))
                    } else {
                        Ok(None)
                    }
                }
                Some(previous) => {
                    if is_invariant_attribute(curr) {
                        Err(CompileError::new_spanned(
                            curr,
                            "only one attribute of this kind allowed per item",
                        ))
                    } else {
                        Ok(Some(previous))
                    }
                }
            });
        let maybe_invariant_attr = result?;
        let Some(invariant_attr) = maybe_invariant_attr else {
            return Ok(None);
        };

        let expression = InvariantExpression::try_from(&invariant_attr.meta)?;

        Ok(Some(Self { expression }))
    }
}

/// the expression inside the braces of the #[invariant(...)] attribute
// #[derive(Debug)]
enum InvariantExpression {
    /// a closure is defined
    /// (there are some aspects that we can verify, but not all)
    Closure(ExprClosure),
    /// a path to a function is given
    /// (there's nothing more about this we can verify at macro expansion time)
    Path(Path),
}

impl InvariantExpression {
    fn full_span(&self) -> proc_macro2::Span {
        match self {
            InvariantExpression::Closure(closure) => closure.body.span(),
            InvariantExpression::Path(path) => path.span(),
        }
    }
}

impl ToTokens for InvariantExpression {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            InvariantExpression::Closure(closure) => closure.to_tokens(tokens),
            InvariantExpression::Path(path) => path.to_tokens(tokens),
        }
    }
}

impl TryFrom<&Meta> for InvariantExpression {
    type Error = CompileError;
    fn try_from(meta: &Meta) -> Result<Self, CompileError> {
        match meta {
            // this means that just the attribute, without the braces has been given
            // we must return an error. Meaning just #[invariant]. If a path (e.g.
            // function name is given in braces, like #[invariant(my_fun)], this
            // is handled in the case below.
            Meta::Path(path) => Err(CompileError::new_spanned(
                path,
                "attribute requires closure or function name for validation in braces",
            )),
            Meta::List(list) => {
                // first try parsing this as a path
                if let Ok(path) = syn::parse::<Path>(list.tokens.clone().into()) {
                    if !path.segments.is_empty() {
                        return Ok(Self::Path(path));
                    } else {
                        return Err(CompileError::new_spanned(
                            path,
                            format!("attribute must have form #[{INVARIANT_ATTR}(expression)], where expression is a function name or closure"),
                        ));
                    }
                }
                // otherwise this must be a closure
                if let Ok(closure) = syn::parse::<ExprClosure>(list.tokens.clone().into()) {
                    // we can do some error checks for better error messages.
                    // We have no actual type information but we can make sure that
                    // the closure is a single-argument closure that is not async
                    if closure.asyncness.is_some() {
                        Err(CompileError::new_spanned(
                           closure.asyncness,
                            "async in validation closure not allowed",
                        ))
                    } else if closure.capture.is_some() {
                        Err(CompileError::new_spanned(
                            closure.capture,
                            "move capture in validation closure not allowed",
                        ))
                    } else if closure.inputs.len() != 1 {
                        Err(CompileError::new_spanned(
                            closure,
                            "validation closure must have exactly one argument",
                        ))
                    } else {
                        check_closure(&closure)?;
                        Ok(Self::Closure(closure))
                    }
                } else {
                    Err(CompileError::new_spanned(
                        meta,
                        format!("attribute must have form #[{INVARIANT_ATTR}(expression)], where expression is a function name or closure"),
                    ))
                }
            }
            Meta::NameValue(value) => Err(CompileError::new_spanned(
                value,
                format!("attribute must have form #[{INVARIANT_ATTR}(expression)], where expression is a function name or closure"),
            )),
        }
    }
}

/// a helper function that makes sure that the closure does not use
/// self or the identifier that we use for the finished instance in the builder
/// in its body.
fn check_closure(closure: &ExprClosure) -> Result<(), CompileError> {
    let mut checker = ClosureCheckVisitor::default();
    checker.visit_expr(&closure.body);
    checker.check()
}

struct ClosureCheckVisitor {
    state: Result<(), CompileError>,
}

impl Default for ClosureCheckVisitor {
    fn default() -> Self {
        Self { state: Ok(()) }
    }
}

impl ClosureCheckVisitor {
    /// check if everything went well after visiting
    fn check(self) -> Result<(), CompileError> {
        self.state
    }
}

impl<'ast> Visit<'ast> for ClosureCheckVisitor {
    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if node.qself.is_none() && node.path.is_ident("self") {
            self.state = Err(CompileError::new_spanned(
                &node.path,
                "self keyword not allowed in validation closure",
            ));
        }

        // this identifier is also forbidden in closures, because it refers
        // to the finished value of the structure inside the builder
        if node.qself.is_none() && node.path.is_ident(FINISHED_VALUE_IDENT) {
            self.state = Err(CompileError::new_spanned(
                &node.path,
                format!(
                    "identifier {} is reserved and may not be used in validation closure",
                    FINISHED_VALUE_IDENT
                ),
            ));
        }
    }
}
