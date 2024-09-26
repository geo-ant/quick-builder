use std::fmt::Display;

use quick_builder_derive::QuickBuilder;

#[derive(Debug, QuickBuilder)]
pub struct Foo<T1: Default, T2>
where
    T2: Display,
{
    x: T1,
    y: T2,
}

unsafe fn foo() {
    let f: Foo<i32, u32> = __FooBuilderState::uninit().assume_init();
}
