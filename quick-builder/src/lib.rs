#![allow(clippy::needless_doctest_main)]
#![doc= include_str!("../Readme.md")]
#![warn(missing_docs)]

#[deprecated = "This crate is deprecated. Please switch to the bon crate for fallible builders."]
pub use quick_builder_derive::QuickBuilder;
