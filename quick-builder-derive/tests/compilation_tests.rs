#[test]
fn simple_builder_compiles() {
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/simple_builder.rs")
}

#[test]
fn build_function_does_not_exist_before_last_setter_is_called() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/*.rs")
}
