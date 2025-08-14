/*!
# riglr-macros

Procedural macros for riglr - dramatically reducing boilerplate when creating blockchain tools.

The `#[tool]` macro is the cornerstone of riglr's developer experience, transforming simple async
functions, synchronous functions, and structs into full-featured blockchain tools with automatic error handling, JSON
schema generation, and seamless `rig` framework integration.

## Overview

The `#[tool]` macro automatically implements the `Tool` trait for both async and sync functions, as well as structs,
eliminating the need to write ~30 lines of boilerplate code per tool. It generates:

1. **Parameter struct** with proper JSON schema and serde annotations
2. **Tool trait implementation** with error handling and type conversion
3. **Documentation extraction** from doc comments for AI model consumption
4. **SignerContext integration** for secure blockchain operations
5. **Convenience constructors** for easy instantiation

## Code Generation Process

When you apply `#[tool]` to a function, the macro performs the following transformations:

### 1. Parameter Extraction and Struct Generation

```rust,ignore
// Your function:
#[tool]
async fn swap_tokens(
    /// Source token mint address
    from_mint: String,
    /// Destination token mint address  
    to_mint: String,
    /// Amount to swap in base units
    amount: u64,
    /// Optional slippage tolerance (default: 0.5%)
    #[serde(default = "default_slippage")]
    slippage_bps: Option<u16>,
) -> Result<String, SwapError> { ... }

// Generated args struct:
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwapTokensArgs {
    /// Source token mint address
    pub from_mint: String,
    /// Destination token mint address  
    pub to_mint: String,
    /// Amount to swap in base units
    pub amount: u64,
    /// Optional slippage tolerance (default: 0.5%)
    #[serde(default = "default_slippage")]
    pub slippage_bps: Option<u16>,
}
```

### 2. Tool Struct and Trait Implementation Generation

```rust,ignore
// Generated tool struct:
#[derive(Clone)]
pub struct SwapTokensTool;

impl SwapTokensTool {
    pub fn new() -> Self { Self }
}

#[async_trait::async_trait]
impl riglr_core::Tool for SwapTokensTool {
    async fn execute(&self, params: serde_json::Value) -> Result<riglr_core::JobResult, Box<dyn std::error::Error + Send + Sync>> {
        // 1. Parse parameters with detailed error messages
        let args: SwapTokensArgs = serde_json::from_value(params)
            .map_err(|e| format!("Failed to parse parameters: {}", e))?;

        // 2. Call your original function
        let result = swap_tokens(args.from_mint, args.to_mint, args.amount, args.slippage_bps).await;

        // 3. Convert results to standardized JobResult format
        match result {
            Ok(value) => Ok(riglr_core::JobResult::Success {
                value: serde_json::to_value(value)?,
                tx_hash: None,
            }),
            Err(error) => {
                // 4. Structured error handling with retry logic
                let tool_error: riglr_core::ToolError = error.into();
                match tool_error {
                    riglr_core::ToolError::Retriable(msg) => Ok(riglr_core::JobResult::Failure {
                        error: msg,
                        retriable: true,
                    }),
                    riglr_core::ToolError::Permanent(msg) => Ok(riglr_core::JobResult::Failure {
                        error: msg,
                        retriable: false,
                    }),
                    riglr_core::ToolError::RateLimited(msg) => Ok(riglr_core::JobResult::Failure {
                        error: format!("Rate limited: {}", msg),
                        retriable: true,
                    }),
                    riglr_core::ToolError::InvalidInput(msg) => Ok(riglr_core::JobResult::Failure {
                        error: format!("Invalid input: {}", msg),
                        retriable: false,
                    }),
                    riglr_core::ToolError::SignerContext(err) => Ok(riglr_core::JobResult::Failure {
                        error: format!("Signer error: {}", err),
                        retriable: false,
                    }),
                }
            }
        }
    }

    fn name(&self) -> &str {
        "swap_tokens"
    }
}

// Convenience constructor
pub fn swap_tokens_tool() -> std::sync::Arc<dyn riglr_core::Tool> {
    std::sync::Arc::new(SwapTokensTool::new())
}
```

### 3. Documentation Processing and Description Attribute

The macro extracts documentation from three sources and wires them into the Tool implementation:

- **Function docstrings** → Tool descriptions for AI models
- **Parameter docstrings** → JSON schema field descriptions  
- **Type annotations** → JSON schema type information

You can also provide an explicit AI-facing description using the attribute:

```rust,ignore
#[tool(description = "Fetches the URL and returns the body as text.")]
async fn fetch(url: String) -> Result<String, Error> { ... }
```

Priority logic for the generated `Tool::description()` method:
- If `description = "..."` attribute is present, that string is used
- Else, the item's rustdoc comments are used
- Else, an empty string is returned

This enables AI models to understand exactly what each tool does and how to use it properly.

## Constraints and Requirements

### Function Requirements

1. **Return Type**: Must be `Result<T, E>` where `E: Into<riglr_core::ToolError>`
   ```rust,ignore
   // ✅ Valid
   async fn valid_tool() -> Result<String, MyError> { ... }
   
   // ❌ Invalid - not a Result
   async fn invalid_tool() -> String { ... }
   
   // ❌ Invalid - error type doesn't implement Into<ToolError>
   async fn bad_error() -> Result<String, std::io::Error> { ... }
   ```

2. **Parameters**: All parameters must implement `serde::Deserialize + schemars::JsonSchema`
   ```rust,ignore
   // ✅ Valid - standard types implement these automatically
   async fn good_params(address: String, amount: u64) -> Result<(), ToolError> { ... }
   
   // ❌ Invalid - custom types need derives
   struct CustomType { field: String }
   async fn bad_params(custom: CustomType) -> Result<(), ToolError> { ... }
   ```

3. **Function Type**: The macro supports both async and synchronous functions
   ```rust,ignore
   // ✅ Valid - async function
   #[tool]
   async fn async_tool() -> Result<String, ToolError> { ... }
   
   // ✅ Valid - sync function (executed within async context)
   #[tool]
   fn sync_tool() -> Result<String, ToolError> { ... }
   ```
   
   Synchronous functions are automatically wrapped to work within the async Tool trait.
   They execute synchronously within the async `execute` method.

4. **Documentation**: Function and parameters should have doc comments for AI consumption
   ```rust,ignore
   /// This description helps AI models understand the tool's purpose
   #[tool]
   async fn documented_tool(
       /// This helps the AI understand this parameter
       param: String,
   ) -> Result<String, ToolError> { ... }
   ```

### Struct Requirements

For struct-based tools, additional requirements apply:

1. **Execute Method**: Must have an async `execute` method returning `Result<T, E>`
2. **Serde Traits**: Must derive `Serialize`, `Deserialize`, and `JsonSchema`
3. **Clone**: Must be `Clone` for multi-use scenarios

```rust,ignore
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema, Clone)]
#[tool]
struct MyStructTool {
    config: String,
}

impl MyStructTool {
    pub async fn execute(&self) -> Result<String, ToolError> {
        // Implementation
        Ok(format!("Processed: {}", self.config))
    }
}
```

## Complex Usage Examples

### Synchronous Function Example

The macro supports both async and sync functions. Sync functions are useful for
computational tools that don't require I/O operations:

```rust,ignore
use riglr_core::ToolError;

/// Calculate compound interest for a given principal, rate, and time
/// 
/// This is a computational tool that doesn't require async operations,
/// so it's implemented as a synchronous function that runs efficiently
/// within the async Tool framework.
#[tool]
fn calculate_compound_interest(
    /// Principal amount in dollars
    principal: f64,
    /// Annual interest rate as a decimal (e.g., 0.05 for 5%)
    annual_rate: f64,
    /// Time period in years
    years: f64,
    /// Number of times interest is compounded per year
    compounds_per_year: u32,
) -> Result<f64, ToolError> {
    if principal <= 0.0 {
        return Err(ToolError::invalid_input_from_msg("Principal must be positive"));
    }
    if annual_rate < 0.0 {
        return Err(ToolError::invalid_input_from_msg("Interest rate cannot be negative"));
    }
    if years < 0.0 {
        return Err(ToolError::invalid_input_from_msg("Time period cannot be negative"));
    }
    if compounds_per_year == 0 {
        return Err(ToolError::invalid_input_from_msg("Compounds per year must be at least 1"));
    }

    let rate_per_compound = annual_rate / compounds_per_year as f64;
    let total_compounds = compounds_per_year as f64 * years;
    let final_amount = principal * (1.0 + rate_per_compound).powf(total_compounds);
    
    Ok(final_amount)
}
```

#### Important Note on CPU-Intensive Sync Functions

The `#[tool]` macro executes synchronous functions directly within the async executor's thread.
This is fine for quick computations, but **CPU-intensive operations can block the async runtime**.

For CPU-intensive work, wrap your function in `tokio::task::spawn_blocking` **before** applying
the `#[tool]` macro:

```rust,ignore
use riglr_core::ToolError;

/// CPU-intensive cryptographic operation
/// 
/// This uses spawn_blocking to avoid blocking the async runtime
#[tool]
async fn compute_hash(
    /// Data to hash
    data: Vec<u8>,
    /// Number of iterations
    iterations: u32,
) -> Result<String, ToolError> {
    // Move CPU-intensive work to a blocking thread pool
    tokio::task::spawn_blocking(move || {
        // Simulate expensive computation
        let mut hash = data;
        for _ in 0..iterations {
            hash = sha256::digest(&hash).into_bytes();
        }
        Ok(hex::encode(hash))
    })
    .await
    .map_err(|e| ToolError::permanent_from_msg(format!("Task failed: {}", e)))?
}
```

**Guidelines for choosing between sync and async with spawn_blocking:**
- **Use sync functions** for quick calculations (< 1ms), simple data transformations, or validation
- **Use async + spawn_blocking** for CPU-intensive work like cryptography, complex parsing, or heavy computation
- **Use regular async** for I/O operations like network requests or database queries

### Generic Parameters and Type Constraints

```rust,ignore
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

/// Generic tool that can process any serializable data
#[tool]
async fn process_data<T>(
    /// The data to process (must be JSON-serializable)
    data: T,
    /// Processing options
    options: ProcessingOptions,
) -> Result<ProcessedData, ProcessingError>
where
    T: Serialize + Deserialize + JsonSchema + Send + Sync,
{
    // The macro handles generic constraints properly
    let serialized = serde_json::to_string(&data)?;
    // ... processing logic
    Ok(ProcessedData::new(serialized))
}
```

### SignerContext Integration

Tools automatically have access to the current blockchain signer:

```rust,ignore
use riglr_core::signer::SignerContext;

/// Swap tokens on Solana using Jupiter aggregator
/// 
/// This tool automatically accesses the current signer from the context,
/// eliminating the need to pass signing credentials explicitly.
#[tool]
async fn jupiter_swap(
    /// Input token mint address
    input_mint: String,
    /// Output token mint address  
    output_mint: String,
    /// Amount to swap in base units
    amount: u64,
    /// Maximum slippage in basis points
    max_slippage_bps: u16,
) -> Result<String, SwapError> {
    // Access the current signer automatically
    let signer = SignerContext::current().await?;
    
    // Derive RPC client from signer
    let rpc_client = signer.rpc_client();
    
    // Get quote from Jupiter
    let quote = get_jupiter_quote(&input_mint, &output_mint, amount, max_slippage_bps).await?;
    
    // Build and sign transaction
    let tx = build_swap_transaction(quote, &signer.pubkey()).await?;
    let signed_tx = signer.sign_transaction(tx).await?;
    
    // Send transaction
    let signature = rpc_client.send_and_confirm_transaction(&signed_tx).await?;
    
    Ok(signature.to_string())
}
```

### Multi-Chain Tool with Dynamic Signer Selection

```rust,ignore
use riglr_core::signer::{SignerContext, ChainType};

/// Bridge tokens between different blockchains
/// 
/// Automatically detects the source chain from the current signer
/// and handles cross-chain bridging operations.
#[tool]
async fn bridge_tokens(
    /// Source token address
    source_token: String,
    /// Destination chain identifier  
    dest_chain: String,
    /// Destination token address
    dest_token: String,
    /// Amount to bridge in base units
    amount: u64,
    /// Recipient address on destination chain
    recipient: String,
) -> Result<BridgeResult, BridgeError> {
    let signer = SignerContext::current().await?;
    
    // Dynamic chain detection
    let bridge_operation = match signer.chain_type() {
        ChainType::Solana => {
            SolanaBridge::new(signer).bridge_to_evm(
                source_token, dest_chain, dest_token, amount, recipient
            ).await?
        },
        ChainType::Ethereum => {
            EthereumBridge::new(signer).bridge_to_solana(
                source_token, dest_token, amount, recipient
            ).await?
        },
        ChainType::Polygon => {
            PolygonBridge::new(signer).bridge_cross_chain(
                source_token, dest_chain, dest_token, amount, recipient
            ).await?
        },
        _ => return Err(BridgeError::UnsupportedChain),
    };
    
    Ok(bridge_operation)
}
```

### Error Handling and Retry Logic

The macro automatically integrates with riglr's structured error handling:

```rust,ignore
use riglr_core::ToolError;

#[derive(thiserror::Error, Debug)]
enum SwapError {
    #[error("Insufficient balance: need {required}, have {available}")]
    InsufficientBalance { required: u64, available: u64 },
    
    #[error("Network congestion, retry in {retry_after_seconds}s")]
    NetworkCongestion { retry_after_seconds: u64 },
    
    #[error("Slippage too high: expected {expected}%, got {actual}%")]
    SlippageTooHigh { expected: f64, actual: f64 },
    
    #[error("Invalid token mint: {mint}")]
    InvalidToken { mint: String },
}

// Implement conversion to ToolError for automatic retry logic
impl From<SwapError> for ToolError {
    fn from(error: SwapError) -> Self {
        match error {
            SwapError::NetworkCongestion { .. } => ToolError::Retriable(error.to_string()),
            SwapError::InsufficientBalance { .. } => ToolError::Permanent(error.to_string()),
            SwapError::SlippageTooHigh { .. } => ToolError::Permanent(error.to_string()),
            SwapError::InvalidToken { .. } => ToolError::Permanent(error.to_string()),
        }
    }
}

/// Advanced token swap with detailed error handling
#[tool]
async fn advanced_swap(
    input_mint: String,
    output_mint: String,
    amount: u64,
) -> Result<SwapResult, SwapError> {
    let signer = SignerContext::current().await?;
    
    // Check balance first
    let balance = get_token_balance(&signer, &input_mint).await?;
    if balance < amount {
        return Err(SwapError::InsufficientBalance {
            required: amount,
            available: balance,
        });
    }
    
    // Attempt swap with retries for transient failures
    match attempt_swap(&signer, &input_mint, &output_mint, amount).await {
        Err(SwapError::NetworkCongestion { .. }) => {
            // The macro will automatically mark this as retriable
            Err(SwapError::NetworkCongestion { retry_after_seconds: 10 })
        },
        result => result,
    }
}
```

### Testing Tool Implementations

The macro-generated code is designed to be easily testable:

```rust,ignore
#[cfg(test)]
mod tests {
    use super::*;
    use riglr_core::signer::{MockSigner, SignerContext};
    use serde_json::json;

    #[tokio::test]
    async fn test_swap_tool_execution() {
        // Create mock signer with expected behavior
        let mock_signer = MockSigner::new()
            .with_token_balance("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", 1000000)  // USDC
            .expect_transaction("swap")
            .returns_signature("5j7s2Hz2UnknownTxHash");

        // Test the generated tool
        let tool = SwapTokensTool::new();
        
        let result = SignerContext::new(&mock_signer).execute(async {
            tool.execute(json!({
                "fromMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "toMint": "So11111111111111111111111111111111111111112",
                "amount": 1000000,
                "slippageBps": 50
            })).await
        }).await;

        assert!(result.is_ok());
        mock_signer.verify_all_expectations();
    }
}
```

## Best Practices

### 1. Parameter Design
- Use descriptive parameter names that clearly indicate their purpose
- Provide comprehensive doc comments for each parameter
- Use appropriate default values with `#[serde(default)]` where applicable
- Group related parameters into structs for complex operations

### 2. Error Handling  
- Define custom error types that implement `Into<ToolError>`
- Use structured errors that provide actionable information
- Distinguish between retriable and permanent errors appropriately
- Include relevant context in error messages

### 3. Documentation
- Write clear, concise function descriptions that explain the tool's purpose
- Document any side effects or state changes
- Include examples in doc comments where helpful
- Explain any complex parameters or return values

### 4. Performance Considerations
- Use `Arc<dyn Tool>` for tools that will be shared across threads
- Implement `Clone` efficiently for struct-based tools
- Consider caching for expensive operations that don't change frequently
- Use appropriate timeouts for network operations

## Macro Limitations

### Current Limitations

1. **Generic Functions**: Limited support for complex generic constraints
2. **Lifetime Parameters**: Not currently supported in tool functions  
3. **Associated Types**: Cannot use associated types in parameters
4. **Const Generics**: No support for const generic parameters

### Workarounds

For complex generic scenarios, consider using trait objects or type erasure:

```rust,ignore
// Instead of:
// #[tool]
// async fn complex_generic<T: ComplexTrait>(data: T) -> Result<(), Error> { ... }

// Use:
#[tool]  
async fn process_complex_data(
    /// JSON representation of the data to process
    data: serde_json::Value,
) -> Result<ProcessedResult, ProcessError> {
    // Deserialize to specific types inside the function
    let typed_data: MyType = serde_json::from_value(data)?;
    // ... process typed_data
}
```

## Integration with External Crates

The macro is designed to work seamlessly with the broader Rust ecosystem:

### Serde Integration
- Automatic `#[serde(rename_all = "camelCase")]` for JavaScript compatibility
- Support for all serde attributes on parameters
- Custom serialization/deserialization via serde derives

### JSON Schema Generation
- Automatic schema generation via `schemars` crate
- Support for complex nested types and enums
- Custom schema attributes for fine-tuned control

### Async Runtime Compatibility
- Works with any async runtime (tokio, async-std, etc.)
- Proper handling of async trait implementations
- Support for async error handling patterns

The `#[tool]` macro transforms riglr from a collection of utilities into a cohesive, 
developer-friendly framework for building sophisticated blockchain AI agents.
*/

use heck::ToPascalCase;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Attribute, DeriveInput, FnArg, ItemFn, ItemStruct, PatType, parse::Parse, parse::ParseStream, LitStr, Token};

/// The `#[tool]` procedural macro that converts functions and structs into Tool implementations.
///
/// This macro supports:
/// - Async functions with arbitrary parameters and Result return types
/// - Structs that have an `execute` method
/// - Automatic JSON schema generation using `schemars`
/// - Documentation extraction from doc comments
/// - Parameter descriptions from doc comments on function arguments
/// Attributes supported:
/// - description = "..."
#[proc_macro_attribute]
pub fn tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = item.clone();

    let tool_attrs = match syn::parse::<ToolAttr>(attr) {
        Ok(attrs) => attrs,
        Err(_) => ToolAttr { description: None },
    };

    // Try to parse as function first, then as struct
    if let Ok(function) = syn::parse::<ItemFn>(input.clone()) {
        handle_function(function, tool_attrs).into()
    } else if let Ok(structure) = syn::parse::<ItemStruct>(input) {
        handle_struct(structure, tool_attrs).into()
    } else {
        syn::Error::new_spanned(
            proc_macro2::TokenStream::from(item),
            "#[tool] can only be applied to async functions or structs",
        )
        .to_compile_error()
        .into()
    }
}

#[derive(Default)]
struct ToolAttr {
    description: Option<String>,
}

impl Parse for ToolAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Ident) {
            let ident: syn::Ident = input.parse()?;
            if ident == "description" {
                input.parse::<Token![=]>()?;
                let lit: LitStr = input.parse()?;
                return Ok(Self { description: Some(lit.value()) });
            } else {
                return Err(syn::Error::new_spanned(ident, "Unknown attribute key. Supported: description"));
            }
        }

        Err(syn::Error::new(input.span(), "Expected attribute key like: description = \"...\""))
    }
}

fn handle_function(function: ItemFn, tool_attrs: ToolAttr) -> proc_macro2::TokenStream {
    let fn_name = &function.sig.ident;
    let fn_vis = &function.vis;

    // Extract documentation from function
    let description = extract_doc_comments(&function.attrs);
    let selected_description = match tool_attrs.description {
        Some(desc) => desc,
        None => description,
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


    // Generate the module name from the function name
    let module_name = syn::Ident::new(&fn_name.to_string(), fn_name.span());

    // Generate the tool implementation with namespace
    quote! {
        // Keep the original function
        #function

        // Generate all tool-related code in a module namespace
        #fn_vis mod #module_name {
            use super::*;

            // Generate the args struct if there are parameters
            #[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema, Debug, Clone)]
            #[serde(rename_all = "camelCase")]
            pub struct Args {
                #(#param_fields),*
            }

            // Generate the tool struct
            #[derive(Clone)]
            pub struct Tool;

            impl Tool {
                /// Create a new instance of this tool
                pub fn new() -> Self {
                    Self
                }
            }

            impl Default for Tool {
                fn default() -> Self {
                    Self::new()
                }
            }

            // Implement the Tool trait
            #[async_trait::async_trait]
            impl riglr_core::Tool for Tool {
                async fn execute(&self, params: serde_json::Value) -> Result<riglr_core::JobResult, Box<dyn std::error::Error + Send + Sync>> {
                    // Parse the parameters; return a structured JobResult on parse failure
                    let args: Args = match serde_json::from_value(params) {
                        Ok(v) => v,
                        Err(e) => {
                            return Ok(riglr_core::JobResult::Failure {
                                error: format!("Failed to parse parameters: {}", e),
                                retriable: false,
                            });
                        }
                    };

                    // Call the original function (expecting Result<T, ToolError>)
                    let result = super::#fn_name(#(args.#param_names),*)#await_token;

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
                                riglr_core::ToolError::Retriable { context, .. } => {
                                    Ok(riglr_core::JobResult::Failure {
                                        error: context,
                                        retriable: true,
                                    })
                                }
                                riglr_core::ToolError::Permanent { context, .. } => {
                                    Ok(riglr_core::JobResult::Failure {
                                        error: context,
                                        retriable: false,
                                    })
                                }
                                riglr_core::ToolError::RateLimited { context, .. } => {
                                    Ok(riglr_core::JobResult::Failure {
                                        error: format!("Rate limited: {}", context),
                                        retriable: true,
                                    })
                                }
                                riglr_core::ToolError::InvalidInput { context, .. } => {
                                    Ok(riglr_core::JobResult::Failure {
                                        error: format!("Invalid input: {}", context),
                                        retriable: false,
                                    })
                                }
                                riglr_core::ToolError::SignerContext(err) => {
                                    Ok(riglr_core::JobResult::Failure {
                                        error: format!("Signer error: {}", err),
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

                fn description(&self) -> &str {
                    #selected_description
                }
            }
        }

        // Create a convenience function to create an Arc<dyn Tool> using the namespaced type
        #fn_vis fn #tool_fn_name() -> std::sync::Arc<dyn riglr_core::Tool> {
            std::sync::Arc::new(#module_name::Tool::new())
        }
    }
}

fn handle_struct(structure: ItemStruct, tool_attrs: ToolAttr) -> proc_macro2::TokenStream {
    let struct_name = &structure.ident;
    let struct_vis = &structure.vis;

    // Extract documentation from struct
    let description = extract_doc_comments(&structure.attrs);
    let selected_description = match tool_attrs.description {
        Some(desc) => desc,
        None => description,
    };

    quote! {
        // Keep the original struct
        #structure

        // Implement the Tool trait
    #[async_trait::async_trait]
    impl riglr_core::Tool for #struct_name {
            async fn execute(&self, params: serde_json::Value) -> Result<riglr_core::JobResult, Box<dyn std::error::Error + Send + Sync>> {
                // Parse parameters into the struct; return a structured JobResult on parse failure
                let args: Self = match serde_json::from_value(params) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(riglr_core::JobResult::Failure {
                            error: format!("Failed to parse parameters: {}", e),
                            retriable: false,
                        });
                    }
                };

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
                riglr_core::ToolError::Retriable { context, .. } => {
                                Ok(riglr_core::JobResult::Failure {
                    error: context,
                                    retriable: true,
                                })
                            }
                riglr_core::ToolError::Permanent { context, .. } => {
                                Ok(riglr_core::JobResult::Failure {
                    error: context,
                                    retriable: false,
                                })
                            }
                riglr_core::ToolError::RateLimited { context, .. } => {
                                Ok(riglr_core::JobResult::Failure {
                    error: format!("Rate limited: {}", context),
                                    retriable: true,
                                })
                            }
                riglr_core::ToolError::InvalidInput { context, .. } => {
                                Ok(riglr_core::JobResult::Failure {
                    error: format!("Invalid input: {}", context),
                                    retriable: false,
                                })
                            }
                            riglr_core::ToolError::SignerContext(err) => {
                                Ok(riglr_core::JobResult::Failure {
                                    error: format!("Signer error: {}", err),
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

            fn description(&self) -> &str {
                #selected_description
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

/// Derives automatic conversion from an error enum to ToolError.
/// 
/// This macro generates a `From<YourError> for ToolError` implementation
/// that automatically classifies errors as retriable or permanent based on
/// naming conventions in variant names.
/// 
/// # Classification Rules
/// 
/// Errors are classified as **retriable** if their variant names contain:
/// - `Rpc`, `Network`, `Connection`, `Timeout`, `TooManyRequests`, `RateLimit`
/// - `Api` (for external API errors)
/// - `Http` (for HTTP-related errors)
/// 
/// Errors are classified as **permanent** if their variant names contain:
/// - `Invalid`, `Parse`, `Serialization`, `NotFound`, `Unauthorized`
/// - `InsufficientBalance`, `InsufficientFunds`
/// - All other unmatched variants (conservative default)
/// 
/// # Custom Classification
/// 
/// You can override the automatic classification using attributes:
/// 
/// ```rust,ignore
/// #[derive(IntoToolError)]
/// enum MyError {
///     #[tool_error(retriable)]
///     CustomError(String),
///     
///     #[tool_error(permanent)]
///     NetworkError(String), // Override default retriable classification
///     
///     #[tool_error(rate_limited)]
///     ApiQuotaExceeded,
/// }
/// ```
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_macros::IntoToolError;
/// use thiserror::Error;
/// 
/// #[derive(Error, Debug, IntoToolError)]
/// enum SolanaError {
///     #[error("RPC error: {0}")]
///     RpcError(String),  // Automatically retriable
///     
///     #[error("Invalid address: {0}")]
///     InvalidAddress(String),  // Automatically permanent
///     
///     #[error("Network timeout")]
///     NetworkTimeout,  // Automatically retriable
///     
///     #[error("Insufficient balance")]
///     InsufficientBalance,  // Automatically permanent
///     
///     #[tool_error(retriable)]
///     #[error("Custom error: {0}")]
///     CustomError(String),  // Explicitly retriable
/// }
/// ```
#[proc_macro_derive(IntoToolError, attributes(tool_error))]
pub fn derive_into_tool_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let name = input.ident;
    let variants = match input.data {
        syn::Data::Enum(ref data) => &data.variants,
        _ => {
            return TokenStream::from(quote! {
                compile_error!("IntoToolError can only be derived for enums");
            });
        }
    };
    
    let match_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_name_str = variant_name.to_string();
        
        // Check for explicit classification attribute
        let classification = variant.attrs.iter()
            .filter_map(|attr| {
                if attr.path().is_ident("tool_error") {
                    attr.parse_args::<syn::Ident>().ok()
                } else {
                    None
                }
            })
            .next();
        
        let pattern = match &variant.fields {
            syn::Fields::Named(_) => quote! { #name::#variant_name { .. } },
            syn::Fields::Unnamed(_) => quote! { #name::#variant_name(..) },
            syn::Fields::Unit => quote! { #name::#variant_name },
        };
        
        let conversion = if let Some(class) = classification {
            match class.to_string().as_str() {
                "retriable" => quote! { 
                    riglr_core::ToolError::retriable_from_msg(err.to_string()) 
                },
                "permanent" => quote! { 
                    riglr_core::ToolError::permanent_from_msg(err.to_string()) 
                },
                "rate_limited" => quote! { 
                    riglr_core::ToolError::rate_limited_from_msg(err.to_string()) 
                },
                _ => quote! { 
                    riglr_core::ToolError::permanent_from_msg(err.to_string()) 
                },
            }
        } else {
            // Automatic classification based on naming conventions
            let retriable_patterns = [
                "Rpc", "Network", "Connection", "Timeout", 
                "TooManyRequests", "RateLimit", "Api", "Http"
            ];
            
            let is_retriable = retriable_patterns.iter()
                .any(|pattern| variant_name_str.contains(pattern));
            
            if is_retriable {
                quote! { riglr_core::ToolError::retriable_from_msg(err.to_string()) }
            } else {
                quote! { riglr_core::ToolError::permanent_from_msg(err.to_string()) }
            }
        };
        
        quote! {
            #pattern => #conversion
        }
    });
    
    let expanded = quote! {
        impl From<#name> for riglr_core::ToolError {
            fn from(err: #name) -> Self {
                match err {
                    #(#match_arms),*
                }
            }
        }
    };
    
    TokenStream::from(expanded)
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
        // Compilation success is the test
    }
}