//! Basic trybuild test to verify macro functionality

#[test]
fn test_basic_pass() {
    let t = trybuild::TestCases::new();

    // Test a single valid case
    t.pass("tests/trybuild/pass/basic_tool.rs");
}

#[test]
fn test_basic_fail() {
    let t = trybuild::TestCases::new();

    // Test a single invalid case
    t.compile_fail("tests/trybuild/fail/non_async_function.rs");
}
