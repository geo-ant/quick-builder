use std::fmt::Display;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;

/// bit nicer interface for compile errors
pub struct CompileError {
    inner: syn::Error,
}

impl From<syn::Error> for CompileError {
    fn from(inner: syn::Error) -> Self {
        Self { inner }
    }
}

impl From<CompileError> for TokenStream {
    fn from(error: CompileError) -> Self {
        error.inner.into_compile_error().into()
    }
}

impl CompileError {
    /// uses syn::Error::new internally
    ///
    /// Use Error::new when the error needs to be triggered on some span
    /// other than where the parse stream is currently positioned.
    pub fn new(span: Span, message: impl Display) -> Self {
        Self {
            inner: syn::Error::new(span, message),
        }
    }

    /// uses syn::Error::new_spanned internally
    ///
    /// Creates an error with the specified message spanning the given syntax tree node.
    /// Unlike the Error::new constructor, this constructor takes an argument
    /// tokens which is a syntax tree node. This allows the resulting Error to
    /// attempt to span all tokens inside of tokens. While you would typically
    /// be able to use the Spanned trait with the above Error::new constructor,
    /// implementation limitations today mean that Error::new_spanned may
    /// provide a higher-quality error message on stable Rust.
    pub fn new_spanned(tokens: impl ToTokens, message: impl Display) -> Self {
        Self {
            inner: syn::Error::new_spanned(tokens, message),
        }
    }
}
