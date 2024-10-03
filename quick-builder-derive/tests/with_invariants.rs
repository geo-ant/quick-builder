use quick_builder_derive::QuickBuilder;

#[derive(PartialEq, Debug, QuickBuilder)]
#[invariant(|this|this.first as f32 > this.second)]
struct WithoutGenerics {
    first: i32,
    #[invariant(|this|*this>0.)]
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
    #[invariant(|w|*w>0)]
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

#[test]
fn sad_path_for_builder_without_generics() {
    let built = WithoutGenerics::builder()
        .first(123)
        .second(-1.)
        .third("hello".into())
        .build();

    assert_eq!(built, None);

    let built = WithoutGenerics::builder()
        .first(123)
        .second(124.)
        .third("hello".into())
        .build();
    assert_eq!(built, None);
}

#[test]
fn sad_path_for_builder_with_one_generic() {
    let built = WithOneGeneric::builder().foo(1).bar(Some(337)).build();
    assert_eq!(built, None);
    let built = WithOneGeneric::builder().foo(-1).bar(Some(-337)).build();
    assert_eq!(built, None);
    let built = WithOneGeneric::builder().foo(1).bar(Some(-337)).build();
    assert_eq!(built, None);
}

#[test]
fn sad_path_for_builder_with_generics_and_lifetimes() {
    let incorrectly_sized_data = &mut [1., 2., 3., 4.];
    let built = ImageMut::builder()
        .width(2)
        .height(3)
        .data(incorrectly_sized_data)
        .build();
    assert_eq!(built, None);

    // correctly sized, but the width and height invariants should trigger
    let mut zero_data: [f32; 0] = [];
    let built = ImageMut::builder()
        .width(0)
        .height(3)
        .data(&mut zero_data)
        .build();
    assert_eq!(built, None);
    let mut zero_data: [f32; 0] = [];

    let built = ImageMut::builder()
        .width(2)
        .height(0)
        .data(&mut zero_data)
        .build();
    assert_eq!(built, None);
}
