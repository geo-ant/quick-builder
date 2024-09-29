use crate::detail::StructDeriveInput;
use crate::error::CompileError;
use quote::{format_ident, quote, ToTokens};

/// see the documentation for the function below
pub struct BuilderState {
    /// the name of the struct, to be used in other
    /// items that need it. Don't try and use it for quoting, use the item
    /// directly.
    ident: proc_macro2::Ident,
    /// the tokens covering the definition and impl block of this structure
    tokens: proc_macro2::TokenStream,
    // /// the generics from the original input that already were split for implementation
    // split_generics: (ImplGenerics, TypeGenerics, Option<
}

impl BuilderState {
    /// get the name of the struct
    pub fn ident(&self) -> &proc_macro2::Ident {
        &self.ident
    }
}

impl ToTokens for BuilderState {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.tokens.to_tokens(tokens)
    }
}

/// Generate the struct definition for the internal state of the builder
/// So for a struct
/// pub struct Foo<T1,T2> {
///   a : T1,
///   b : T2,
///   // ...,
///  }
///
/// it will generate a structure for the internal state with an
/// impl block like so
///
///
/// struct __FooBuilderInternal<T1,T2> {
///   a: MaybeUninit<T1>,
///   b: MaybeUninit<T2>,
///   //...
/// }
///
/// impl<T1,T2> __FooBuilderInternal<T1,T2> {
///    fn uninit() -> Self {
///       Self {
///          a: MaybeUninit::uninit(),
///          b: MaybeUninit::uninit(),
///       }
///    }
///    
///    unsafe fn assume_init(self)  -> Foo<T1,T2> {
///       Foo {
///         a : self.a.assume_init(),
///         b : self.b.assume_init(),
///       }
///     }
///  }
/// ```
///
pub fn make_builder_state(input: &StructDeriveInput) -> Result<BuilderState, CompileError> {
    let original_ident = &input.ident;
    // identifier for the internal builder state
    let ident = format_ident!("__{}BuilderState", original_ident);

    let where_clause = &input.generics.where_clause;
    let generic_params = &input.generics.params;

    let fields = match input.data.fields {
        syn::Fields::Named(ref named) => &named.named,
        _ => unreachable!("struct can only have named fields"),
    };

    let validators = fields.iter().for_each(|f| {
        println!("{}:", f.ident.as_ref().unwrap());

        f.attrs.iter().for_each(|at| println!("\t{:#?}", at));
    });

    // Create new fields by wrapping each type in MaybeUninit<T>
    let maybe_uninit_fields = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_ty = &field.ty;
        quote! {
            #field_ident: ::core::mem::MaybeUninit<#field_ty>
        }
    });

    // make all fields uninitialized for construction
    let field_initializers_uninit = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_ty = &field.ty;
        quote! {
            #field_ident: ::core::mem::MaybeUninit::<#field_ty>::uninit()
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

    Ok(BuilderState { ident, tokens })
}
