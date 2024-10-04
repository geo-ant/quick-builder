use quick_builder_derive::QuickBuilder;
use std::fmt::Debug;

#[derive(PartialEq, Debug, QuickBuilder)]
struct WithoutGenerics {
    first: i32,
    second: f32,
    third: String,
}

#[derive(PartialEq, Debug, QuickBuilder)]
struct WithOneGeneric<T: PartialEq>
where
    T: Debug,
{
    foo: T,
    bar: Option<T>,
}

#[derive(PartialEq, Debug, QuickBuilder)]
struct WithMultiGenericsAndLifetimes<'a, 'b: 'a, 'c, T, U: Clone>
where
    T: Copy,
    'c: 'b,
{
    first: &'a str,
    second: &'b T,
    third: &'c usize,
    fourth: Option<(T, U)>,
}

#[test]
fn builder_without_generics() {
    let built = WithoutGenerics::builder()
        .first(1)
        .second(32.0)
        .third("hello".into())
        .build();
    let expected = WithoutGenerics {
        first: 1,
        second: 32.0,
        third: "hello".into(),
    };
    assert_eq!(built, expected);
}

#[test]
fn builder_with_one_generic() {
    let built = WithOneGeneric::builder().foo(1).bar(Some(337)).build();
    let expected = WithOneGeneric {
        foo: 1,
        bar: Some(337),
    };
    assert_eq!(built, expected);
}

#[test]
fn builder_with_multi_generics() {
    let myint = Box::new(1337);
    let myfloat = Box::new(123.);
    let my_str = String::from("hi");

    let expected = WithMultiGenericsAndLifetimes {
        first: &my_str,
        second: myfloat.as_ref(),
        third: &myint,
        fourth: Some((3., 0u8)),
    };

    let built = WithMultiGenericsAndLifetimes::builder()
        .first(&my_str)
        .second(myfloat.as_ref())
        .third(&myint)
        .fourth(Some((3., 0u8)))
        .build();
    assert_eq!(built, expected);
}
