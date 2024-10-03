pub use quick_builder_derive::QuickBuilder;
use std::fmt::Display;

#[derive(Debug, QuickBuilder)]
#[invariant(|me|me.f > *me.r)]
pub struct Foo<'a, T1: Default, T2>
where
    T2: Display,
{
    #[invariant(|f|!f.is_nan()||*f>0.)]
    f: f64,
    x: T1,
    #[invariant(|f|true)]
    y: T2,
    #[invariant(validation)]
    // #[invariant = validate]
    r: &'a mut f64,
}

fn validation<T>(f: &T) -> bool {
    std::mem::size_of::<T>() <= 4
}

#[test]
fn foo() {
    // let f = validation;
    // if !(f)(&1.) {}

    // // let f: Foo<i32, u32> = Foo::builder().set_x(1).set_y(2).set_r(&1).build();
    // // let f2 = Foo::builder().set_x(1.).set_y(2).set_r(&1).build();
    // // let f3 = Foo::builder().set_x(1.).set_y(2).set_r(&1).build();
    // // let f4 = Foo::builder().set_x(1.).set_y(2.).set_r(&3f32).build();
    // // let f5 = Foo::<i32, i32>::builder()
    // //     .set_x(1)
    // //     .set_y(2)
    // //     .set_r(&3)
    // //     .build();
    // // let f6 = Foo::builder().set_x(1.).set_y(3.).set_r(&3.).build();
    // // let f7 = Foo::builder().f(1.).x(4).y(1.).r(&3.).build().unwrap();
    let mut float = f64::NAN;
    let f8 = Foo::builder()
        .f(1.)
        .x(2.)
        .y(4.)
        .r(&mut float)
        .build()
        .unwrap();
    let f9 = Foo::builder().f(1.).x(3).y(10.).r(&mut float).build();
    let f10 = Foo::builder().f(2.).x(4).y("hallo").r(&mut float).build();
    let f11 = Foo::builder().f(0.1).x(2.).y(33).r(&mut float).build();
    let _f21 = Foo::builder().f(0.1).x(2.).y(33);

    // let f10 = Foo::builder2().f(1.).x(3.).y(11).r(&mut float);
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

// impl<'a, T1: Default, T2: Display> Foo<'a, T1, T2> {
//     pub fn builder2() -> builder2::FooBuilder2<'a, T1, T2, ()> {
//         builder2::FooBuilder2::new()
//     }
// }

// mod builder2 {
//     use super::*;

//     // pub struct Foo<'a, T1: Default, T2>
//     // f: f64,
//     // x: T1,
//     // #[validate(|f|true)]
//     // y: T2,
//     // #[validate(validation)]
//     // r: &'a mut f64,
//     pub struct FooBuilder2<'a, T1: Default, T2, __State>
//     where
//         T2: Display,
//     {
//         state: __State,
//         phantom: PhantomData<(f64, T1, T2, &'a mut f64)>,
//     }

//     impl<'a, T1: Default, T2> FooBuilder2<'a, T1, T2, ()>
//     where
//         T2: Display,
//     {
//         pub fn new() -> Self {
//             Self {
//                 state: Default::default(),
//                 phantom: Default::default(),
//             }
//         }
//     }

//     impl<'a, T1: Default, T2> FooBuilder2<'a, T1, T2, ()>
//     where
//         T2: Display,
//     {
//         #[allow(unused_parens)]
//         pub fn f(self, f: f64) -> FooBuilder2<'a, T1, T2, (f64,)> {
//             FooBuilder2 {
//                 state: (f,),
//                 phantom: Default::default(),
//             }
//         }
//     }

//     impl<'a, T1: Default, T2> FooBuilder2<'a, T1, T2, (f64,)>
//     where
//         T2: Display,
//     {
//         pub fn x(self, x: T1) -> FooBuilder2<'a, T1, T2, (f64, T1)> {
//             let state = self.state;
//             FooBuilder2 {
//                 state: (state.0, x),
//                 phantom: Default::default(),
//             }
//         }
//     }

//     impl<'a, T1: Default, T2> FooBuilder2<'a, T1, T2, (f64, T1)>
//     where
//         T2: Display,
//     {
//         pub fn y(self, y: T2) -> FooBuilder2<'a, T1, T2, (f64, T1, T2)> {
//             let state = self.state;
//             FooBuilder2 {
//                 state: (state.0, state.1, y),
//                 phantom: Default::default(),
//             }
//         }
//     }

//     impl<'a, T1: Default, T2> FooBuilder2<'a, T1, T2, (f64, T1, T2)>
//     where
//         T2: Display,
//     {
//         pub fn r(self, r: &'a mut f64) -> FooBuilder2<'a, T1, T2, (f64, T1, T2, &'a mut f64)> {
//             let state = self.state;
//             FooBuilder2 {
//                 state: (state.0, state.1, state.2, r),
//                 phantom: Default::default(),
//             }
//         }
//     }
//     impl<'a, T1: Default, T2> FooBuilder2<'a, T1, T2, (f64, T1, T2, &'a mut f64)>
//     where
//         T2: Display,
//     {
//         pub fn build(self) -> Foo<'a, T1, T2> {
//             let finished = Foo {
//                 f: self.state.0,
//                 x: self.state.1,
//                 y: self.state.2,
//                 r: self.state.3,
//             };
//             finished
//         }
//     }
// }
