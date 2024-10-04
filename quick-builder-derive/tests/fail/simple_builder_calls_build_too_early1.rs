use quick_builder_derive::QuickBuilder;

#[derive(QuickBuilder, PartialEq, Debug)]
#[invariant(|this|*this.first > this.third)]
struct Foo<'a, T> {
    #[invariant(|f|!f.is_nan())]
    first: &'a f32,
    second: T,
    third: f32,
}

fn foo() {
    let float = 1.;

    // this should fail because the builder method is only available after
    // third(...) has been called.
    let built = Foo::<i32>::builder()
        .first(&float)
        // .second(10)
        // .third(0.3)
        .build();
    // .expect("building this must not fail");
}

pub fn main() {}
