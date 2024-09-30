#![allow(dead_code)]
#![allow(unused_variables)]
use std::fmt::Display;

use quick_builder_derive::QuickBuilder;

#[derive(Debug, QuickBuilder)]
#[validate(|me|me.f > *me.r)]
pub struct Foo<'a, T1: Default, T2>
where
    T2: Display,
{
    #[validate(|f|!f.is_nan()||!f.is_finite())]
    f: f64,
    // #[validate]
    x: T1,
    #[validate(|f|true)]
    y: T2,
    #[validate(validation)]
    r: &'a mut f64,
}

fn validation<T>(f: &T) -> bool {
    std::mem::size_of::<T>() <= 4
}

#[test]
fn foo() {
    let f = validation;
    if !(f)(&1.) {}

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
    // let f7 = Foo::builder().f(1.).x(4).y(1.).r(&3.).build().unwrap();
    let mut float = f64::NAN;
    let f8 = Foo::builder()
        .f(1.)
        .x(2.)
        .y(4.)
        .r(&mut float)
        .build()
        .unwrap();
}

#[inline]
fn check<F, T>(t: &T, f: F) -> bool
where
    F: Fn(&T) -> bool,
{
    (f)(t)
}

fn bar() {
    let x = 0f32;
    check(&x, |f| f.is_nan());
}
