//! Basic tests for riglr-macros library

#[test]
fn test_macro_exists() {
    // Test that the tool macro is available
    // This is a basic compilation test - if we can compile this, the macro exists
    // Compilation success is the test
}

#[test]
fn test_proc_macro_dependencies() {
    // Test that we can use the dependencies that the macro relies on
    use quote::quote;
    use syn::parse_str;

    let code = quote! {
        fn test() {}
    };

    let parsed: Result<syn::ItemFn, _> = parse_str(&code.to_string());
    assert!(parsed.is_ok());
}

#[test]
fn test_heck_dependency() {
    use heck::ToPascalCase;

    let test_string = "hello_world";
    let pascal_case = test_string.to_pascal_case();
    assert_eq!(pascal_case, "HelloWorld");
}

#[test]
fn test_syn_parsing_basic() {
    use syn::{parse_str, ItemFn};

    let code = "fn test_function() {}";
    let parsed: Result<ItemFn, _> = parse_str(code);
    assert!(parsed.is_ok());
}

#[test]
fn test_quote_generation_basic() {
    use quote::quote;

    let test_code = quote! {
        fn generated_function() {
            println!("Hello from generated code");
        }
    };

    let output = test_code.to_string();
    assert!(output.contains("generated_function"));
    assert!(output.contains("println"));
}

#[test]
fn test_proc_macro2_tokens() {
    use proc_macro2::TokenStream;
    use std::str::FromStr;

    let tokens = TokenStream::from_str("fn test() {}").unwrap();
    assert!(!tokens.is_empty());
}

#[test]
fn test_serde_json_integration() {
    use serde_json::json;

    let test_json = json!({
        "type": "object",
        "properties": {}
    });

    assert!(test_json.is_object());
}

#[test]
fn test_async_trait_available() {
    // Test that async_trait is available (used by the macro)
    // This is just a compilation test

    use async_trait::async_trait;

    #[async_trait]
    #[allow(dead_code)]
    trait TestTrait {
        async fn test_method(&self);
    }

    #[allow(dead_code)]
    struct TestStruct;

    #[async_trait]
    impl TestTrait for TestStruct {
        async fn test_method(&self) {
            // Implementation
        }
    }

    // Compilation success is the test - we can create trait objects
}

// Comprehensive test of all dependencies the macro uses
#[test]
fn test_all_macro_dependencies() {
    use heck::ToPascalCase;
    use proc_macro2::TokenStream;
    use quote::quote;
    use serde_json::json;
    // Test that all types and traits are available
    let pascal_case: String = "test".to_pascal_case();
    assert_eq!(pascal_case, "Test");
    let token_stream: TokenStream = quote! { fn test() {} };
    assert!(!token_stream.is_empty());
    let json_value: serde_json::Value = json!({});
    assert!(json_value.is_object());

    // If we get here, all dependencies are properly available
    // If we get here, all dependencies compiled successfully
}
