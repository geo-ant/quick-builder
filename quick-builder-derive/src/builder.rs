use std::default;

use crate::{builder_state::BuilderState, detail::StructDeriveInput, error::CompileError};
use quote::{format_ident, quote, ToTokens};
use special_generics::TypeGenericsWithoutAngleBrackets;
use syn::Fields;

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

    // this is like the impl generics but without the enclosing <...>
    let struct_generics = &input.generics.params;
    let type_generics_without_angle_brackets =
        TypeGenericsWithoutAngleBrackets::from(&input.generics);

    let field_index_type = FieldIndexType::new(&input.data.fields)?;

    // @todo make this visibility configurable
    let builder_vis = syn::token::Pub::default();

    let builder_struct_tokens = quote! {
         // note we must stick our generic parameter at the end, because otherwise
         // the compiler might complain that lifetimes have to go first.
         // the __INIT_FIELD_INDEX points to the
         #builder_vis struct #builder_ident <#struct_generics, const __INIT_FIELD_INDEX: #field_index_type> #original_where_clause{
             state: #state_ident #original_ty_generics,
         }

         impl #original_impl_generics Default for #builder_ident <#type_generics_without_angle_brackets,-1> #original_where_clause {
            fn default() -> Self {
                Self { state: #state_ident::#original_ty_generics::uninit()}
            }
         }

         impl #original_impl_generics #original_ident #original_ty_generics
             #original_where_clause {
                 //@todo make this visibility configurable
                 #builder_vis fn builder() -> #builder_ident <#type_generics_without_angle_brackets,-1> {
                     Default::default()
                 }
         }
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

#[derive(Copy, Clone, PartialEq, Eq)]
/// the type of the const generic field index
pub enum FieldIndexType {
    I64,
}

impl FieldIndexType {
    /// construct a new field index generic type that takes the number of
    /// fields into account.
    pub fn new(fields: &Fields) -> Result<Self, CompileError> {
        if (i64::MAX as usize) < fields.len() {
            return Err(CompileError::new_spanned(
                &fields,
                "QuickBuilder: too many fields in structure",
            ));
        }
        Ok(Self::I64)
    }
}

impl ToTokens for FieldIndexType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FieldIndexType::I64 => quote! { i64 },
        }
        .to_tokens(tokens)
    }
}
