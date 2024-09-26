use detail::StructDeriveInput;
use error::CompileError;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

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

    let builder_ident = format_ident!("{}Builder", input.ident);
    let fields = match input.data.fields {
        syn::Fields::Named(ref named) => &named.named,
        _ => unreachable!("struct can only have named fields"),
    };

    let builder_state_struct = try2!(builder_state_struct_and_impls(&input));

    quote! {

        #builder_state_struct

    }
    .into()
}

struct BuilderStateStruct {
    /// the name of the struct, to be used in other
    /// items that need it. Don't try and use it for quoting, use the item
    /// directly.
    pub ident: proc_macro2::Ident,
    tokens: proc_macro2::TokenStream,
}

impl ToTokens for BuilderStateStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.tokens.to_tokens(tokens)
    }
}

/// Generate the struct definition for the internal state of the builder
/// So for a struct
/// pub struct Foo {
///   a : T1,
///   b : T2,
///    ...,
///  }
///
/// it will generate a state
///
/// struct __FooBuilderIntenal {
///   a: MaybeUninit<T1>,
///   b: MaybeUninit<T2>,
///   ...
/// }
fn builder_state_struct_and_impls(
    input: &StructDeriveInput,
) -> Result<BuilderStateStruct, CompileError> {
    let original_ident = &input.ident;
    // identifier for the internal builder state
    let ident = format_ident!("__{}BuilderState", original_ident);

    let where_clause = &input.generics.where_clause;
    let generic_params = &input.generics.params;

    let fields = match input.data.fields {
        syn::Fields::Named(ref named) => &named.named,
        _ => unreachable!("struct can only have named fields"),
    };

    // Create new fields by wrapping each type in MaybeUninit<T>
    let maybe_uninit_fields = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_ty = &field.ty;
        quote! {
            #field_ident: core::mem::MaybeUninit<#field_ty>
        }
    });

    // make all fields uninitialized for construction
    let field_initializers_uninit = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_ty = &field.ty;
        quote! {
            #field_ident: core::mem::MaybeUninit::<#field_ty>::uninit()
        }
    });
    // make all fields uninitialized for construction
    let assume_all_init = fields.iter().map(|field| {
        let field_ident = &field.ident;
        quote! {
            #field_ident: self.#field_ident.assume_init()
        }
    });

    let (impl_generics, ty_generics, impl_where_clause) = input.generics.split_for_impl();

    let tokens = quote! {
        // visibility for the internal state can always be private
        #[doc(hidden)]
        struct #ident <#generic_params> #where_clause {
            #(#maybe_uninit_fields),*
        }

        // create an uninit() constructor which gives an instance with
        // all fields uninitialized
        impl #impl_generics #ident #ty_generics #impl_where_clause {
            fn uninit() -> Self {
                Self {
                    #(#field_initializers_uninit),*
                }

            }

            // assumes all fields are initialized and returns an
            // instance of the original struct
            unsafe fn assume_init(self) -> #original_ident #ty_generics{
                unsafe {
                    #original_ident {
                        #(#assume_all_init),*
                    }
                }
            }
        }
    };

    Ok(BuilderStateStruct { ident, tokens })
}
