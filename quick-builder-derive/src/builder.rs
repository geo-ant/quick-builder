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
    let builder_state_ident = state.ident();
    let original_struct_ident = &input.ident;
    let builder_ident = format_ident!("{}Builder", original_struct_ident);
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

    // helper function to generate the builder type with a given count of
    // initialized fields, e.g FooBuilder<'a,T1,T2,2>
    let builder_type_with_count = |count: usize| {
        quote! {#builder_ident <#type_generics_without_angle_brackets, #count>}
    };

    let initial_builder_type = builder_type_with_count(0);

    // this is for defining the builder struct,
    // implementing a constructor on it
    // and defining the Builder method on the original struct
    let builder_struct_tokens = quote! {
         // note we must stick our generic parameter at the end, because otherwise
         // the compiler might complain that lifetimes have to go first.
         // the __INIT_FIELD_COUNT tells us the number of fields that have been
         // initialized. Initialized happens top to bottom in order of declaration.
         // Thus, the builder starts at count 0, which indicates no
         // fields have been initialized.
         #builder_vis struct #builder_ident <#struct_generics, const __INIT_FIELD_COUNT: #field_index_type> #original_where_clause{
             state: #builder_state_ident #original_ty_generics,
         }

         impl #original_impl_generics #initial_builder_type #original_where_clause {
            pub fn new() -> Self {
                Self { state: #builder_state_ident::#original_ty_generics::uninit()}
            }
         }

         impl #original_impl_generics #original_struct_ident #original_ty_generics
             #original_where_clause {
                 //@todo make this visibility configurable
                 #builder_vis fn builder() -> #initial_builder_type {
                     #builder_ident::new()
                 }
         }
    };

    // now we construct the chain of setter function on the builder, where
    // we go from __INIT_FIELD_COUNT i to count i+1 by setting the field at
    // index i (starting with index 0, in order of declaration). That means that
    // we transitively know that if the field at index i is set,
    // all fields at indices 0,...,i have been set.
    let setters = fields.iter().enumerate().map(|(count, field)| {
        let previous_builder_type = builder_type_with_count(count);
        let next_builder_type = builder_type_with_count(count + 1);
        let setter_fn = field.ident.as_ref().map(|ident| format_ident!("{}", ident));
        let field_ident = &field.ident;
        let field_type = &field.ty;
        let setter_tokens = quote! {

         impl #original_impl_generics #previous_builder_type #original_where_clause {
            fn #setter_fn (self, #field_ident : #field_type) -> #next_builder_type {
                let mut state = self.state;
                state.#field_ident.write(#field_ident);
                #builder_ident { state }
            }
         }

        };
        setter_tokens
    });

    // this is to generate the build method on the final form of the builder where we
    // know that all fields have been initialized.
    let final_builder = builder_type_with_count(fields.len());
    let builder_tokens = quote! {
         impl #original_impl_generics #final_builder #original_where_clause {
            fn build(self) -> #original_struct_ident #original_ty_generics {
                // Safety: this is safe because we know all fields have been
                // initialized at this point.
                let finished = unsafe {self.state.assume_init()};
                finished
            }
         }
    };

    let tokens = quote! {
        #builder_struct_tokens

        #(#setters)*

        #builder_tokens
    };

    Ok(Builder {
        ident: builder_ident,
        tokens,
    })
}

#[derive(Copy, Clone, PartialEq, Eq)]
/// the type of the const generic field index
//@note(geo) I noticed something interesting, since I was previously allowing
// i64 as the type and had negative numbers for the associated constant. As soon
// as I used negative numbers, the type deduction goes out of the window for
// rust-analyzer (the compiler itself is fine) and the autocompletion will
// suggest methods that aren't even implemented for a specific generic builder
// instance.
pub enum FieldIndexType {
    Usize,
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
        Ok(Self::Usize)
    }
}

impl ToTokens for FieldIndexType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FieldIndexType::Usize => quote! { usize },
        }
        .to_tokens(tokens)
    }
}
