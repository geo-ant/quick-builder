#![deny(proc_macro_derive_resolution_fallback)]
use quick_builder::QuickBuilder;

#[derive(QuickBuilder)]
#[allow(dead_code)]
pub struct Foo {
    first: i32,
    second: u32,
}

#[test]
// the actual tests of the macro are in the integration tests of the
// quick-builder-derive-crate. This test just confirms we can use the
// macro.
fn integration_test_works() {
    _ = Foo::builder().first(1).second(2).build();
}
