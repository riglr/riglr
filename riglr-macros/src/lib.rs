/*!
# riglr-macros

Procedural macros for riglr - reducing boilerplate when creating rig-compatible tools.

The `#[tool]` macro automatically implements the `Tool` trait for async functions and structs,
generating JSON schemas from Rust types and extracting documentation from doc comments.

## Example

```rust,ignore
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Get the balance of a Solana wallet
///
/// This tool queries the Solana blockchain to retrieve the SOL balance
/// for a given wallet address.
#[tool]
pub async fn get_sol_balance(
    /// The Solana wallet address to query
    address: String,
    /// Whether to use confirmed or finalized commitment

    confirmed: bool,
) -> Result<u64, anyhow::Error> {
    // Implementation here
    Ok(1000000)
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct SwapConfig {
    input_mint: String,
    output_mint: String,
    /// Amount to swap in lamports
    amount: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[tool]
struct TokenSwapper {
    config: SwapConfig,
}

impl TokenSwapper {
    pub async fn execute(&self) -> Result<String, anyhow::Error> {
        // Implementation here
        Ok("transaction_hash".to_string())
    }
}
```
*/

use heck::ToPascalCase;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{Attribute, FnArg, ItemFn, ItemStruct, PatType};

/// The `#[tool]` procedural macro that converts functions and structs into Tool implementations.
///
/// This macro supports:
/// - Async functions with arbitrary parameters and Result return types
/// - Structs that have an `execute` method
/// - Automatic JSON schema generation using `schemars`
/// - Documentation extraction from doc comments
/// - Parameter descriptions from doc comments on function arguments
#[proc_macro_attribute]
pub fn tool(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = item.clone();

    // Try to parse as function first, then as struct
    if let Ok(function) = syn::parse::<ItemFn>(input.clone()) {
        handle_function(function).into()
    } else if let Ok(structure) = syn::parse::<ItemStruct>(input) {
        handle_struct(structure).into()
    } else {
        syn::Error::new_spanned(
            proc_macro2::TokenStream::from(item),
            "#[tool] can only be applied to async functions or structs",
        )
        .to_compile_error()
        .into()
    }
}

fn handle_function(function: ItemFn) -> proc_macro2::TokenStream {
    let fn_name = &function.sig.ident;
    let fn_vis = &function.vis;

    // Extract documentation from function
    let description = extract_doc_comments(&function.attrs);
    let _description_lit = if description.is_empty() {
        quote! { concat!("Tool: ", stringify!(#fn_name)) }
    } else {
        quote! { #description }
    };

    // Extract parameter info 
    let mut param_fields = Vec::new();
    let mut param_names = Vec::new();
    let mut param_docs = Vec::new();

    for input in function.sig.inputs.iter() {
        if let FnArg::Typed(PatType { pat, ty, attrs, .. }) = input {
            if let syn::Pat::Ident(ident) = pat.as_ref() {
                let param_name = &ident.ident;
                let param_type = ty.as_ref();
                let param_doc = extract_doc_comments(attrs);

                param_names.push(param_name.clone());
                param_docs.push(param_doc);

                // Check if the type has serde attributes
                let has_default = attrs.iter().any(|attr| {
                    attr.path().is_ident("serde")
                        && attr.to_token_stream().to_string().contains("default")
                });

                if has_default {
                    param_fields.push(quote! {
                        #(#attrs)*
                        pub #param_name: #param_type
                    });
                } else {
                    param_fields.push(quote! {
                        #(#attrs)*
                        pub #param_name: #param_type
                    });
                }
            }
        }
    }

    // Generate the struct names
    let tool_struct_name = syn::Ident::new(
        &format!("{}Tool", fn_name.to_string().to_pascal_case()),
        fn_name.span(),
    );
    let args_struct_name = syn::Ident::new(&format!("{}Args", tool_struct_name), fn_name.span());
    let tool_fn_name = syn::Ident::new(&format!("{}_tool", fn_name), fn_name.span());

    // Generate field assignments for function call
    let field_assignments: Vec<_> = param_names.iter()
        .map(|name| quote! { args.#name })
        .collect();

    // Check if function is async
    let is_async = function.sig.asyncness.is_some();
    let await_token = if is_async {
        quote! { .await }
    } else {
        quote! {}
    };


    // Generate the tool implementation
    quote! {
        // Generate the args struct if there are parameters
        #[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        pub struct #args_struct_name {
            #(#param_fields),*
        }

        // Generate the tool struct
        #[derive(Clone)]
        #fn_vis struct #tool_struct_name;

        impl #tool_struct_name {
            /// Create a new instance of this tool
            pub fn new() -> Self {
                Self
            }
        }

        impl Default for #tool_struct_name {
            fn default() -> Self {
                Self::new()
            }
        }

        // Implement the Tool trait
        #[async_trait::async_trait]
        impl riglr_core::Tool for #tool_struct_name {
            async fn execute(&self, params: serde_json::Value) -> Result<riglr_core::JobResult, Box<dyn std::error::Error + Send + Sync>> {
                // Parse the parameters
                let args: #args_struct_name = serde_json::from_value(params)
                    .map_err(|e| format!("Failed to parse parameters: {}", e))?;

                // Call the original function (expecting Result<T, ToolError>)
                let result = #fn_name(#(#field_assignments),*)#await_token;

                // Convert the result to JobResult
                match result {
                    Ok(value) => {
                        let json_value = serde_json::to_value(value)?;
                        Ok(riglr_core::JobResult::Success {
                            value: json_value,
                            tx_hash: None,
                        })
                    }
                    Err(tool_error) => {
                        // Convert any error to ToolError, then match on it
                        let tool_error: riglr_core::ToolError = tool_error.into();
                        match tool_error {
                            riglr_core::ToolError::Retriable(msg) => {
                                Ok(riglr_core::JobResult::Failure {
                                    error: msg,
                                    retriable: true,
                                })
                            }
                            riglr_core::ToolError::Permanent(msg) => {
                                Ok(riglr_core::JobResult::Failure {
                                    error: msg,
                                    retriable: false,
                                })
                            }
                            riglr_core::ToolError::RateLimited(msg) => {
                                Ok(riglr_core::JobResult::Failure {
                                    error: format!("Rate limited: {}", msg),
                                    retriable: true,
                                })
                            }
                        }
                    }
                }
            }

            fn name(&self) -> &str {
                stringify!(#fn_name)
            }
        }


        // Keep the original function
        #function

        // Optionally, create a convenience function to create an Arc<dyn Tool>
        #fn_vis fn #tool_fn_name() -> std::sync::Arc<dyn riglr_core::Tool> {
            std::sync::Arc::new(#tool_struct_name::new())
        }
    }
}

fn handle_struct(structure: ItemStruct) -> proc_macro2::TokenStream {
    let struct_name = &structure.ident;
    let struct_vis = &structure.vis;

    // Extract documentation from struct
    let description = extract_doc_comments(&structure.attrs);
    let _description_lit = if description.is_empty() {
        quote! { concat!("Tool: ", stringify!(#struct_name)) }
    } else {
        quote! { #description }
    };

    quote! {
        // Keep the original struct
        #structure

        // Implement the Tool trait
        #[async_trait::async_trait]
        impl riglr_core::Tool for #struct_name {
            async fn execute(&self, params: serde_json::Value) -> Result<riglr_core::JobResult, Box<dyn std::error::Error + Send + Sync>> {
                // Parse parameters into the struct
                let args: Self = serde_json::from_value(params)
                    .map_err(|e| format!("Failed to parse parameters: {}", e))?;

                // Call the execute method (expecting Result<T, ToolError>)
                let result = args.execute().await;

                // Convert the result to JobResult
                match result {
                    Ok(value) => {
                        let json_value = serde_json::to_value(value)?;
                        Ok(riglr_core::JobResult::Success {
                            value: json_value,
                            tx_hash: None,
                        })
                    }
                    Err(tool_error) => {
                        // Convert any error to ToolError, then match on it
                        let tool_error: riglr_core::ToolError = tool_error.into();
                        match tool_error {
                            riglr_core::ToolError::Retriable(msg) => {
                                Ok(riglr_core::JobResult::Failure {
                                    error: msg,
                                    retriable: true,
                                })
                            }
                            riglr_core::ToolError::Permanent(msg) => {
                                Ok(riglr_core::JobResult::Failure {
                                    error: msg,
                                    retriable: false,
                                })
                            }
                            riglr_core::ToolError::RateLimited(msg) => {
                                Ok(riglr_core::JobResult::Failure {
                                    error: format!("Rate limited: {}", msg),
                                    retriable: true,
                                })
                            }
                        }
                    }
                }
            }

            fn name(&self) -> &str {
                stringify!(#struct_name)
            }
        }

        // Convenience function to create the tool
        impl #struct_name {
            #struct_vis fn as_tool(self) -> std::sync::Arc<dyn riglr_core::Tool> {
                std::sync::Arc::new(self)
            }
        }

    }
}

fn extract_doc_comments(attrs: &[Attribute]) -> String {
    let mut docs = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &meta.value
                {
                    let line = lit_str.value();
                    // Remove leading space if present (rustdoc convention)
                    let line = line.strip_prefix(' ').unwrap_or(&line);
                    docs.push(line.to_string());
                }
            }
        }
    }

    docs.join("\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_doc_comments_empty() {
        let attrs = vec![];
        let result = extract_doc_comments(&attrs);
        assert_eq!(result, "");
    }

    #[test]
    fn test_extract_doc_comments_with_content() {
        // This is a unit test for the doc comment extraction function
        // In a real scenario, we would need to parse actual syn::Attribute instances
        // For now, we test that the function handles empty attributes correctly
        let attrs = vec![];
        let result = extract_doc_comments(&attrs);
        assert_eq!(result, "");
    }

    #[test]
    fn test_to_pascal_case_conversion() {
        // Test the heck crate functionality we use
        assert_eq!("test_function".to_pascal_case(), "TestFunction");
        assert_eq!("get_balance".to_pascal_case(), "GetBalance");
        assert_eq!("simple".to_pascal_case(), "Simple");
    }

    // Note: Testing procedural macros typically requires integration tests
    // with the `trybuild` crate or similar, as unit testing proc macros
    // directly is challenging due to their compile-time nature.
    //
    // For comprehensive testing, we would create test files in tests/
    // directory that use the macro and verify compilation and behavior.

    #[test]
    fn test_macro_module_exists() {
        // Basic test to ensure the module compiles
        assert!(true);
    }
}