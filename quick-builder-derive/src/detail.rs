use proc_macro2::Ident;
use syn::{Attribute, Data, DataStruct, DeriveInput, Generics, Visibility};

use crate::error::CompileError;

/// this is syn's DeriveInput where we know that the contained data is a struct
/// and not anything else
pub struct StructDeriveInput {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub generics: Generics,
    pub data: DataStruct,
}

const EXPECTED_STRUCT: &'static str =
    "Expected struct: QuickBuilder can only be derived on structs";

/// get an instance from the derive input. If this is not a struct, then
/// returns an error.
impl TryFrom<DeriveInput> for StructDeriveInput {
    type Error = CompileError;

    fn try_from(input: DeriveInput) -> Result<Self, Self::Error> {
        match input.data {
            Data::Struct(data) => Ok(Self {
                attrs: input.attrs,
                vis: input.vis,
                ident: input.ident,
                generics: input.generics,
                data,
            }),
            Data::Enum(data) => Err(CompileError::new_spanned(data.enum_token, EXPECTED_STRUCT)),
            Data::Union(data) => Err(CompileError::new_spanned(data.union_token, EXPECTED_STRUCT)),
        }
    }
}
