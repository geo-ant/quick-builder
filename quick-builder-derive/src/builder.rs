use crate::{builder_state::BuilderState, detail::StructDeriveInput, error::CompileError};
use quote::{format_ident, quote, ToTokens};
use syn::{ext::IdentExt, token::Pub, Ident, Visibility};

mod special_generics;

/// the builder struct and its impl blocks
pub struct Builder {
    /// the name of the builder itself
    ident: proc_macro2::Ident,
    /// the tokens for the struct and the implementation
    tokens: proc_macro2::TokenStream,
}

impl Builder {
    /// get the name of the struct itself
    pub fn ident(&self) -> &proc_macro2::Ident {
        &self.ident
    }
}

impl ToTokens for Builder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.tokens.to_tokens(tokens)
    }
}

pub fn make_builder(
    input: &StructDeriveInput,
    state: &BuilderState,
) -> Result<Builder, CompileError> {
    let state_ident = state.ident();
    let original_ident = &input.ident;
    let builder_ident = format_ident!("{}Builder", original_ident);

    // these are the generics for the original type and the internal state
    // This is not the same as for the builder, since the builder has one additional
    // const-generic parameter which is the index of the last initialized field.
    // The builder can use the same where clause but it needs its own type and
    // impl generics.
    let (original_impl_generics, original_ty_generics, original_where_clause) =
        input.generics.split_for_impl();

    // let id = format_ident!("foo");

    let builder_struct_tokens = quote! {
         // @todo make this visibility configurable
         // pub struct #builder_ident <const __INIT_FIELDS_COUNT: usize, #struct_generics> #where_clause{
         //     state: #state_ident #ty_generics,
         // }

         // // impl #impl_generics Default for #builder_ident <0, #struct_generics> #where_clause {
         // //    fn default() -> Self {
         // //        todo!()
         // //        // Self { state: #state_ident #ty_generics ::uninit()}
         // //    }

         // }
    };

    let fields = match input.data.fields {
        syn::Fields::Named(ref named) => &named.named,
        _ => unreachable!("struct must only have named fields"),
    };

    if fields.is_empty() {
        return Err(CompileError::new_spanned(
            &input.ident,
            "QuickBuilder: not possible to derive on struct without fields",
        ));
    }

    let tokens = quote! {
        #builder_struct_tokens
    };

    Ok(Builder {
        ident: builder_ident,
        tokens,
    })
}
