use quick_builder_derive::QuickBuilder;

#[derive(QuickBuilder, PartialEq, Debug)]
#[invariant(|this|*this.first > this.third)]
struct Foo<'a, T> {
    #[invariant(|f|!f.is_nan())]
    first: &'a f32,
    second: T,
    third: f32,
}

pub fn main() {
    let float = 1.;

    let expected = Foo {
        first: &float,
        second: 10,
        third: 0.3,
    };

    let built = Foo::builder()
        .first(&float)
        .second(10)
        .third(0.3)
        .build()
        .expect("building this must not fail");

    assert_eq!(expected, built);
}
