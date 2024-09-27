// //! this module re-implements (steals) the implementations from
// //! syn's [`ImplGenerics`](https://docs.rs/syn/latest/syn/struct.ImplGenerics.html)
// //! and [`TypeGenerics`](https://docs.rs/syn/latest/syn/struct.TypeGenerics.html)
// //! and modifies them slightly so that the const generic parameter
// //! can be passed correctly. For the TypeGenerics, the constant generic
// //! parameter is passed as a concrete number, so we never actually need it
// //! in the impl block
// /// the generics that we need for the builder struct definition. We don't actually
// /// need them in the impl blocks. Let me explain:
// ///
// /// For a struct like this (that has lifetimes and generic parameters)
// /// ```
// /// struct Foo<'a,T1:Debug,T2>
// ///   where T2: Default {
// ///    first: &'a T1,
// ///    second: T2,
// /// }
// /// ```
// /// our builder will look like this:
// /// struct FooBuilder<'a, const __INIT_FIELDS: i64, T1: Debug, T2>
// ///   where T2: Default {
// ///    first: core::mem::MaybeUninit::<&'a T1>,
// ///    second: core::mem::MaybeUninit::<T2>,
// /// ```
// /// The problem is that we need to stick the const generic parameter
// pub struct BuilderStructDefinitionGenerics<'a, T, const I: i64> {
//     a: &'a T,
// }

use quote::ToTokens;
use syn::{GenericParam, Generics, Token};

/// a helper type that is stolen from the syn crate. It gives us the type generics
/// without the enclosing <...> brackets.
pub struct TypeGenericsWithoutAngleBrackets<'a>(&'a Generics);

impl<'a> From<&'a Generics> for TypeGenericsWithoutAngleBrackets<'a> {
    fn from(generics: &'a Generics) -> Self {
        Self(generics)
    }
}

impl<'a> ToTokens for TypeGenericsWithoutAngleBrackets<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.0.params.is_empty() {
            return;
        }

        // leave out the starting `<`
        // TokensOrDefault(&self.0.lt_token).to_tokens(tokens);

        // Print lifetimes before types and consts, regardless of their
        // order in self.params.
        let mut trailing_or_empty = true;
        for param in self.0.params.pairs() {
            if let GenericParam::Lifetime(def) = *param.value() {
                // Leave off the lifetime bounds and attributes
                def.lifetime.to_tokens(tokens);
                param.punct().to_tokens(tokens);
                trailing_or_empty = param.punct().is_some();
            }
        }
        for param in self.0.params.pairs() {
            if let GenericParam::Lifetime(_) = **param.value() {
                continue;
            }
            if !trailing_or_empty {
                <Token![,]>::default().to_tokens(tokens);
                trailing_or_empty = true;
            }
            match param.value() {
                GenericParam::Lifetime(_) => unreachable!(),
                GenericParam::Type(param) => {
                    // Leave off the type parameter defaults
                    param.ident.to_tokens(tokens);
                }
                GenericParam::Const(param) => {
                    // Leave off the const parameter defaults
                    param.ident.to_tokens(tokens);
                }
            }
            param.punct().to_tokens(tokens);
        }

        // leave out the trailing `>`
        // TokensOrDefault(&self.0.gt_token).to_tokens(tokens);
    }
}
