use quick_builder_derive::QuickBuilder;

#[derive(PartialEq, Debug, QuickBuilder)]
#[invariant(|this|this.first as f32 > this.second)]
struct WithoutGenerics {
    first: i32,
    #[invariant(|this|!this.is_nan())]
    second: f32,
    third: String,
}

#[derive(PartialEq, Debug, QuickBuilder)]
struct WithOneGeneric<T: PartialEq>
where
    T: std::fmt::Debug + Ord + Default,
{
    #[invariant(checks::check_foo)]
    foo: T,
    #[invariant(|me|me.as_ref().is_some_and(|v|*v>T::default()))]
    bar: Option<T>,
}

#[derive(Debug, PartialEq, QuickBuilder)]
#[invariant(|this|this.width*this.height == this.data.len())]
struct ImageMut<'a, T: Copy>
where
    T: Default,
{
    width: usize,
    #[invariant(|h|*h>0)]
    height: usize,
    data: &'a mut [T],
}

mod checks {
    pub fn check_foo<T: Default + Ord>(t: &T) -> bool {
        t < &Default::default()
    }
}

#[test]
fn happy_paths_for_builders_with_invariants() {
    let built = WithoutGenerics::builder()
        .first(123)
        .second(32.0)
        .third("hello".into())
        .build()
        .expect("builder with valid invariants should not fail");
    let expected = WithoutGenerics {
        first: 123,
        second: 32.0,
        third: "hello".into(),
    };
    assert_eq!(built, expected);

    let built = WithOneGeneric::builder()
        .foo(-1)
        .bar(Some(337))
        .build()
        .expect("builder with valid invariants should not fail");
    let expected = WithOneGeneric {
        foo: -1,
        bar: Some(337),
    };

    assert_eq!(built, expected);

    let mut data1 = [1, 2, 3, 4, 5, 6];
    let mut data2 = [1, 2, 3, 4, 5, 6];
    let built = ImageMut::builder()
        .width(2)
        .height(3)
        .data(&mut data1)
        .build()
        .expect("builder with valid invariants should not fail");
    let expected = ImageMut {
        width: 2,
        height: 3,
        data: &mut data2,
    };
    assert_eq!(built, expected);
}
