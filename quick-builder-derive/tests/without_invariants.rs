use quick_builder_derive::QuickBuilder;

// #[derive(PartialEq, Debug, QuickBuilder)]
struct Foo {
    first: i32,
    second: f32,
    third: String,
}

#[test]
fn builder_without_generic() {
    panic!("aaH")
}
