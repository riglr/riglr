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
    let description_lit = if description.is_empty() {
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
    let field_assignments = param_names.iter().map(|name| {
        quote! { args.#name }
    });

    // Check if function is async
    let is_async = function.sig.asyncness.is_some();
    let await_token = if is_async {
        quote! { .await }
    } else {
        quote! {}
    };

    // Generate the JSON schema function
    let _schema_gen = if !param_fields.is_empty() {
        quote! {
            fn schema(&self) -> serde_json::Value {
                let schema = schemars::schema_for!(#args_struct_name);
                serde_json::to_value(schema).unwrap_or_else(|_| serde_json::json!({}))
            }
        }
    } else {
        quote! {
            fn schema(&self) -> serde_json::Value {
                serde_json::json!({
                    "type": "object",
                    "properties": {}
                })
            }
        }
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
                        }
                    }
                }
            }

            fn name(&self) -> &str {
                stringify!(#fn_name)
            }
        }

        // If this is intended to be rig-compatible, also generate rig::Tool implementation
        // Note: rig-compat feature removed to avoid unused cfg warnings
        #[async_trait::async_trait]
        impl rig_core::Tool for #tool_struct_name {
            const NAME: &'static str = stringify!(#fn_name);

            type Error = Box<dyn std::error::Error + Send + Sync>;
            type Args = #args_struct_name;
            type Output = serde_json::Value;

            async fn definition(&self, _prompt: String) -> ::rig_core::ToolDefinition {
                let schema = self.schema();

                ::rig_core::ToolDefinition {
                    name: stringify!(#fn_name).to_string(),
                    description: #description_lit.to_string(),
                    parameters: schema,
                }
            }

            async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
                let result = #fn_name(#(args.#param_names),*)#await_token?;
                Ok(serde_json::to_value(result)?)
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
    let description_lit = if description.is_empty() {
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

        // If this is intended to be rig-compatible, also generate rig::Tool implementation
        // Note: rig-compat feature removed to avoid unused cfg warnings
        #[async_trait::async_trait]
        impl rig_core::Tool for #struct_name {
            const NAME: &'static str = stringify!(#struct_name);

            type Error = Box<dyn std::error::Error + Send + Sync>;
            type Args = Self;
            type Output = serde_json::Value;

            async fn definition(&self, _prompt: String) -> ::rig_core::ToolDefinition {
                let schema = schemars::schema_for!(Self);

                ::rig_core::ToolDefinition {
                    name: stringify!(#struct_name).to_string(),
                    description: #description_lit.to_string(),
                    parameters: serde_json::to_value(schema).unwrap_or_else(|_| serde_json::json!({})),
                }
            }

            async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
                let result = args.execute().await?;
                Ok(serde_json::to_value(result)?)
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
