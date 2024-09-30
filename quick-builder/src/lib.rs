#![allow(dead_code)]
#![allow(unused_variables)]
use std::fmt::Display;

use quick_builder_derive::QuickBuilder;

#[derive(Debug, QuickBuilder)]
pub struct Foo<'a, T1: Default, T2>
where
    T2: Display,
{
    #[validate(|f|!f.is_nan())]
    f: f64,
    // #[validate]
    x: T1,
    #[validate(|f|{f= 1.0})]
    y: T2,
    #[validate(validation)]
    r: &'a T2,
}

fn validation(f: f64) -> bool {
    !f.is_nan()
}

fn foo() {
    let f = validation;

    if !(f)(1.) {}

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
    let f7 = Foo::builder().f(1.).x(4).y(1.).r(&3.).build();
    let f8 = Foo::builder().f(1.).x(2.).y(4.).r(&3.).build();
}
