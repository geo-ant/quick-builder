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
    foo: T,
    #[invariant(|me|me.as_ref().is_some_and(|v|*v>T::default()))]
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
fn happy_paths_for_builders_with_invariants() {}
