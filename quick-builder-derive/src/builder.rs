use crate::{
    builder_state::BuilderState, detail::StructDeriveInput, error::CompileError,
    validation::ValidateAttribute,
};
use proc_macro2::Span;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use special_generics::TypeGenericsWithoutAngleBrackets;
use syn::{spanned::Spanned, Fields};

mod special_generics;

/// the builder struct and its impl blocks
pub struct Builder {
    /// the tokens for the struct and the implementation
    tokens: proc_macro2::TokenStream,
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

    // fields.iter().for_each(|field| {
    //     println!("{}", field.ident.as_ref().unwrap());
    //     field.attrs.iter().for_each(|attr| {
    //         println!("{:?}", attr);
    //     });
    // });

    // the validate attribut on the struct itself, if any
    let struct_validate_attribute = ValidateAttribute::new(&input.attrs)?;

    // an iterator over the validate attributes (if any) of the individual fields.
    // Errors should be passed on as compile errors.
    // there is a 1-to-1 correspondence between the fields and the items in this iterator.
    let field_validate_attributes = fields
        .iter()
        .map(|f| ValidateAttribute::new(&f.attrs))
        .collect::<Result<Vec<_>, _>>()?;

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
    // Also if we have no validate-attributes either on the struct itself or
    // on any of the fields, we return the type `Foo` from `FooBuilder`,
    // otherwise we return an `Option<Foo>` that fails if either of the
    // validate expressions fails.
    let final_builder = builder_type_with_count(fields.len());

    let builder_tokens;
    let has_validators = struct_validate_attribute.is_some()
        || field_validate_attributes.iter().any(|val| val.is_some());
    if !has_validators {
        // this is the simple case: if no validation is performed, we just return
        // the struct itself
        builder_tokens = quote! {
             impl #original_impl_generics #final_builder #original_where_clause {
                fn build(self) -> #original_struct_ident #original_ty_generics {
                    // Safety: this is safe because we know all fields have been
                    // initialized at this point.
                    let finished = unsafe {self.state.assume_init()};
                    finished
                }
             }
        };
    } else {
        // in case we have validators, we return an Optional that only contains
        // the value if all validators pass successfully.

        let finished_ident = quote! { finished };

        // the validator logic to be pasted inside the build function
        let field_validator_logic = fields
            .iter()
            .zip(field_validate_attributes.iter())
            .flat_map(|(field, maybe_validator)| {
                let Some(validator) = maybe_validator else {
                    return None;
                };
                let field_ident = field
                    .ident
                    .as_ref()
                    .expect("named fields must have identifiers");
                // let validator_binding = format_ident!("{}_validator", field_ident);

                let validator_expression = validator.expression();

                let span = validator.expression_span();
                // this is & for all types except references and pointers which
                // are directly passed to the validators. All other types are
                // passed as references.
                let ref_qualifier = match field.ty {
                    syn::Type::Ptr(_) => None,
                    syn::Type::Reference(_) => None,
                    _ => Some(syn::token::And {
                        spans: [Span::call_site()],
                    }),
                };

                Some(quote_spanned! {span=>

                    // this is a trick to make sure the correct type gets
                    // deduced on the closures
                    let is_validated : bool = __is_valid(#ref_qualifier #finished_ident . #field_ident,#validator_expression);
                    // let is_validated = Self::__is_valid(#ref_qualifier #finished_ident . # field_ident,validator); 
                    if !is_validated {
                        return None;
                    }

                })
            });

        let struct_validator_logic = struct_validate_attribute.map(|validator| {
            let validator_expression = validator.expression();
            let span = validator_expression.span();
            quote_spanned! {span=>
                let is_validated : bool = __is_valid(& #finished_ident,#validator_expression);
                if !is_validated {
                    return None;
                }
            }
        });

        builder_tokens = quote! {
             impl #original_impl_generics #final_builder #original_where_clause {


                fn build(self) -> ::core::option::Option<#original_struct_ident #original_ty_generics> {
                    // this function helps us with making sure the arguments
                    // of the closures get deduced correctly
                    // it is used above.
                    #[inline(always)]
                    fn __is_valid<__T,__F>(val: &__T, func: __F) -> bool
                    where for<'__life> __F: FnOnce(&__T) -> bool {
                        (func)(val)
                    }
                    // Safety: this is safe because we know all fields have been
                    // initialized at this point.
                    // finished structure, this still has to undergo validation
                    let #finished_ident = unsafe {self.state.assume_init()};

                    #(#field_validator_logic)*

                    #struct_validator_logic

                    Some(#finished_ident)
                }
             }
        };
    }

    let tokens = quote! {
        #builder_struct_tokens

        #(#setters)*

        #builder_tokens
    };

    Ok(Builder { tokens })
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
