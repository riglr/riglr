//! Compile-time tests for the #[tool] macro using trybuild.

#[test]
fn test_macro_compilation() {
    let t = trybuild::TestCases::new();

    // Test successful compilations
    t.pass("tests/ui/simple_function.rs");
    t.pass("tests/ui/function_with_params.rs");
    t.pass("tests/ui/struct_tool.rs");
    t.pass("tests/ui/invalid_non_async.rs"); // Actually passes since we support non-async

    // Test compilation failures
    t.compile_fail("tests/ui/invalid_no_params.rs");
}
