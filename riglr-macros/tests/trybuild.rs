//! Compile-time tests for the #[tool] macro using trybuild
//!
//! These tests ensure that the macro properly validates input and provides
//! helpful error messages for common mistakes.

#[test]
fn trybuild_tests() {
    let t = trybuild::TestCases::new();

    // Test valid usage patterns
    t.pass("tests/trybuild/pass/*.rs");

    // Test invalid usage patterns that should fail to compile
    t.compile_fail("tests/trybuild/fail/*.rs");
}
