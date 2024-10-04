use builder::make_builder;
use detail::StructDeriveInput;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod builder;
mod detail;
mod error;
mod validation;

/// helper macro for returning compile errors even in functions which don't return
/// Result<T,E>, but instead return token streams.
macro_rules! try2 {
    ($result:expr) => {
        match $result {
            Ok(payload) => payload,
            Err(err) => {
                return err.into();
            }
        }
    };
}

#[proc_macro_derive(QuickBuilder, attributes(invariant))]
pub fn quick_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: StructDeriveInput = try2!(parse_macro_input!(input as DeriveInput).try_into());

    let builder = try2!(make_builder(&input));

    quote! {
        #builder
    }
    .into()
}
