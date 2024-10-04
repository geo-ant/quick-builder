#![allow(dead_code)]
#![deny(proc_macro_derive_resolution_fallback)]
use quick_builder::QuickBuilder;

#[derive(QuickBuilder)]
pub struct Foo {
    first: i32,
    second: u32,
}

// verify integration with the getset crate
#[derive(getset::Getters, QuickBuilder)]
pub struct Bar {
    #[invariant(|f|*f>0)]
    #[getset(get)]
    first: u32,
    #[getset(get)]
    second: i8,
}

// verify integration with the derive-getters crate
#[derive(derive_getters::Getters, QuickBuilder)]
pub struct Baz {
    #[invariant(|f|*f>0)]
    #[getter(copy)]
    first: u32,
    second: i8,
}

#[test]
// the actual tests of the macro are in the integration tests of the
// quick-builder-derive-crate. This test just confirms we can use the
// macro.
fn integration_test_works() {
    let _foo = Foo::builder().first(1).second(2).build();
    let bar = Bar::builder().first(1).second(2).build().unwrap();
    let _first: &u32 = bar.first();
    let _second: &i8 = bar.second();
    let baz = Baz::builder().first(1).second(2).build().unwrap();
    let _first: u32 = baz.first();
    let _second: &i8 = baz.second();
}
