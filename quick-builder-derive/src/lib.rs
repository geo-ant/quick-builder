use builder::make_builder;
use builder_state::make_builder_state;
use detail::StructDeriveInput;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

mod builder;
mod builder_state;
mod detail;
mod error;

/// helper macro for returning compile errors even in functions which don't return
/// results but return token streams.
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

#[proc_macro_derive(QuickBuilder, attributes(validate, validate_fn))]
pub fn quick_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: StructDeriveInput = try2!(parse_macro_input!(input as DeriveInput).try_into());

    let builder_state = try2!(make_builder_state(&input));

    let builder = try2!(make_builder(&input, &builder_state));

    quote! {

        #builder_state

        #builder

    }
    .into()
}
