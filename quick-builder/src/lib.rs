#![allow(dead_code)]
#![allow(unused_variables)]
use std::fmt::Display;

use quick_builder_derive::QuickBuilder;

#[derive(Debug, QuickBuilder)]
pub struct Foo<'a, T1: Default, T2>
where
    T2: Display,
{
    x: T1,
    y: T2,
    r: &'a T2,
}

unsafe fn foo() {
    // let f: Foo<i32, u32> = Foo::builder().set_x(1).set_y(2).set_r(&1).build();
    // let f2 = Foo::builder().set_x(1.).set_y(2).set_r(&1).build();
    // let f3 = Foo::builder().set_x(1.).set_y(2).set_r(&1).build();
    // let f4 = Foo::builder().set_x(1.).set_y(2.).set_r(&3f32).build();
    // let f5 = Foo::<i32, i32>::builder()
    //     .set_x(1)
    //     .set_y(2)
    //     .set_r(&3)
    //     .build();
    // let f6 = Foo::builder().set_x(1.).set_y(3.).set_r(&3.).build();
    let f7 = Foo::builder().x(1.).y(1.).r(&3.).build();
    let f8 = Foo::builder().x(2.).y(3.).r(&3.).build();
}
