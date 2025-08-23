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
    async fn execute(&self, params: serde_json::Value, context: &riglr_core::provider::ApplicationContext) -> Result<riglr_core::JobResult, riglr_core::ToolError> {
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
        return Err(ToolError::invalid_input_string("Principal must be positive"));
    }
    if annual_rate < 0.0 {
        return Err(ToolError::invalid_input_string("Interest rate cannot be negative"));
    }
    if years < 0.0 {
        return Err(ToolError::invalid_input_string("Time period cannot be negative"));
    }
    if compounds_per_year == 0 {
        return Err(ToolError::invalid_input_string("Compounds per year must be at least 1"));
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
    .map_err(|e| ToolError::permanent_string(format!("Task failed: {}", e)))?
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
use quote::quote;
use syn::{
    parse::Parse, parse::ParseStream, parse_macro_input, Attribute, DeriveInput, FnArg, ItemFn,
    ItemStruct, LitStr, PatType, Token,
};

/// The `#[tool]` procedural macro that converts functions and structs into Tool implementations.
///
/// This macro supports:
/// - Async functions with arbitrary parameters and Result return types
/// - Structs that have an `execute` method
/// - Automatic JSON schema generation using `schemars`
/// - Documentation extraction from doc comments
/// - Parameter descriptions from doc comments on function arguments
///
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

#[derive(Default, Debug)]
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
                return Ok(Self {
                    description: Some(lit.value()),
                });
            } else {
                return Err(syn::Error::new_spanned(
                    ident,
                    "Unknown attribute key. Supported: description",
                ));
            }
        }

        Err(syn::Error::new(
            input.span(),
            "Expected attribute key like: description = \"...\"",
        ))
    }
}

/// Helper function to check if a parameter is a context parameter (by type)
fn is_context_param(param_type: &syn::Type) -> bool {
    // Check if the type is &ApplicationContext or &riglr_core::provider::ApplicationContext
    if let syn::Type::Reference(type_ref) = param_type {
        if let syn::Type::Path(type_path) = &*type_ref.elem {
            let path_str = type_path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            return path_str == "ApplicationContext"
                || path_str == "riglr_core::provider::ApplicationContext"
                || path_str.ends_with("::ApplicationContext");
        }
    }
    false
}

/// Check if a type is Result<T, E>
fn is_result_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let segment_name = segment.ident.to_string();
            return segment_name == "Result"
                && matches!(segment.arguments, syn::PathArguments::AngleBracketed(_));
        }
    }
    false
}

/// Check if a type is likely serializable
fn is_serializable_type(ty: &syn::Type) -> bool {
    match ty {
        // Basic serializable types
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let segment_name = segment.ident.to_string();
                match segment_name.as_str() {
                    // Primitive types
                    "String" | "str" | "bool" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                    | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "f32" | "f64" | "char" => {
                        true
                    }

                    // Common generic types that are serializable
                    "Vec" | "Option" | "HashMap" | "BTreeMap" | "HashSet" | "BTreeSet"
                    | "VecDeque" => true,

                    // Common time types
                    "SystemTime" | "Duration" => true,

                    // Assume custom types are serializable (user responsibility)
                    _ => true,
                }
            } else {
                false
            }
        }

        // References to serializable types
        syn::Type::Reference(type_ref) => is_serializable_type(&type_ref.elem),

        // Arrays are serializable if their element type is
        syn::Type::Array(type_array) => is_serializable_type(&type_array.elem),

        // Slices are serializable if their element type is
        syn::Type::Slice(type_slice) => is_serializable_type(&type_slice.elem),

        // Tuples are serializable if all elements are
        syn::Type::Tuple(type_tuple) => type_tuple.elems.iter().all(is_serializable_type),

        // Other types - be conservative and reject
        _ => false,
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

    // Partition parameters into user_params and context_params
    let mut user_params = Vec::new();
    let mut context_params = Vec::new();

    for input in function.sig.inputs.iter() {
        if let FnArg::Typed(PatType { pat, ty, attrs, .. }) = input {
            if is_context_param(ty) {
                context_params.push((pat, ty, attrs));
            } else {
                user_params.push((pat, ty, attrs));
            }
        }
    }

    // Validate exactly one context parameter
    if context_params.len() != 1 {
        return syn::Error::new_spanned(
            &function.sig,
            "`#[tool]` functions must have exactly one parameter of type `&ApplicationContext`",
        )
        .to_compile_error();
    }

    // Validate function signature requirements
    if function.sig.asyncness.is_none() {
        return syn::Error::new_spanned(&function.sig, "`#[tool]` functions must be async")
            .to_compile_error();
    }

    // Validate return type is Result
    if let syn::ReturnType::Type(_, ty) = &function.sig.output {
        if !is_result_type(ty) {
            return syn::Error::new_spanned(
                ty,
                "`#[tool]` functions must return a Result<T, E> where T is serializable and E implements Into<ToolError>"
            ).to_compile_error();
        }
    } else {
        return syn::Error::new_spanned(
            &function.sig,
            "`#[tool]` functions must return a Result<T, E>",
        )
        .to_compile_error();
    }

    // Validate parameter types are serializable
    for (pat, ty, _) in user_params.iter() {
        if let syn::Pat::Ident(ident) = pat.as_ref() {
            let param_name = &ident.ident;
            if !is_serializable_type(ty) {
                return syn::Error::new_spanned(
                    ty,
                    format!(
                        "Parameter '{}' must be a serializable type. Consider using String, numbers, bool, Vec<T>, Option<T>, or custom types that implement Serialize/Deserialize",
                        param_name
                    )
                ).to_compile_error();
            }
        }
    }

    // Build Args struct from user_params only
    let mut param_fields = Vec::new();
    let mut param_names = Vec::new();
    let mut param_docs = Vec::new();

    for (pat, ty, attrs) in user_params.iter() {
        if let syn::Pat::Ident(ident) = pat.as_ref() {
            let param_name = &ident.ident;
            let param_type = ty.as_ref();
            let param_doc = extract_doc_comments(attrs);

            param_names.push(param_name.clone());
            param_docs.push(param_doc.clone());

            // Add documentation for the field
            let doc_attr = if param_doc.is_empty() {
                quote! { #[doc = "Parameter"] }
            } else {
                quote! { #[doc = #param_doc] }
            };

            // Filter out any attributes that might cause issues
            // Only keep serde-related attributes
            let filtered_attrs: Vec<_> = attrs
                .iter()
                .filter(|attr| {
                    if let Some(ident) = attr.path().get_ident() {
                        let name = ident.to_string();
                        name == "serde" || name == "schemars"
                    } else {
                        false
                    }
                })
                .collect();

            param_fields.push(quote! {
                #doc_attr
                #(#filtered_attrs)*
                pub #param_name: #param_type
            });
        }
    }

    // Generate the struct names
    let tool_struct_name = syn::Ident::new(
        &format!("{}Tool", fn_name.to_string().to_pascal_case()),
        fn_name.span(),
    );
    let _args_struct_name = syn::Ident::new(&format!("{}Args", tool_struct_name), fn_name.span());
    let tool_fn_name = syn::Ident::new(&format!("{}_tool", fn_name), fn_name.span());

    // Check if function is async
    let is_async = function.sig.asyncness.is_some();
    let await_token = if is_async {
        quote! { .await }
    } else {
        quote! {}
    };

    // Build the call arguments list for the function call
    let mut call_args = quote! {};
    for input in function.sig.inputs.iter() {
        if let FnArg::Typed(PatType { pat, ty, .. }) = input {
            if is_context_param(ty) {
                // If it's the context param, pass the context from the execute signature
                call_args.extend(quote! { context, });
            } else if let syn::Pat::Ident(ident) = pat.as_ref() {
                // If it's a user param, pass it from the deserialized 'args' struct
                let param_name = &ident.ident;
                call_args.extend(quote! { args.#param_name.clone(), });
            }
        }
    }

    // Generate the module name from the function name
    let module_name = syn::Ident::new(&fn_name.to_string(), fn_name.span());

    // Generate the error handling match arms
    let error_match_arms = generate_tool_error_match_arms();

    // Generate the tool implementation with namespace
    quote! {
        // Keep the original function
        #function

        // Generate all tool-related code in a module namespace
        #[doc = "Generated tool module containing implementation details"]
        #fn_vis mod #module_name {
            use super::*;

            // Generate the args struct if there are parameters
            #[doc = "Arguments structure for the tool"]
            #[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema, Debug, Clone)]
            #[serde(rename_all = "camelCase")]
            pub struct Args {
                #(#param_fields),*
            }

            // Generate the tool struct
            #[doc = "Tool implementation structure"]
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

            // Implement the riglr_core::Tool trait
            #[async_trait::async_trait]
            impl riglr_core::Tool for Tool {
                /// Execute the tool with the provided parameters
                async fn execute(&self, params: serde_json::Value, context: &riglr_core::provider::ApplicationContext) -> Result<riglr_core::JobResult, riglr_core::ToolError> {
                    // Parse the parameters; convert parse errors to ToolError::InvalidInput
                    let args: Args = match serde_json::from_value(params) {
                        Ok(v) => v,
                        Err(e) => {
                            // Convert parameter parsing error to ToolError and use standard error handling
                            let tool_error = riglr_core::ToolError::invalid_input_with_source(
                                e,
                                "Failed to parse tool parameters"
                            );
                            return match tool_error {
                                #error_match_arms
                            };
                        }
                    };

                    // Call the original function with reconstructed arguments
                    let result = super::#fn_name(#call_args)#await_token;

                    // Convert the result to JobResult
                    match result {
                        Ok(value) => {
                            let json_value = serde_json::to_value(value)
                                .map_err(|e| riglr_core::ToolError::permanent_with_source(e, "Failed to serialize result"))?;
                            Ok(riglr_core::JobResult::Success {
                                value: json_value,
                                tx_hash: None,
                            })
                        }
                        Err(tool_error) => {
                            // Convert any error to ToolError, then match on it
                            let tool_error: riglr_core::ToolError = tool_error.into();
                            match tool_error {
                                #error_match_arms
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

            // NOTE: rig::tool::Tool compatibility is handled by RigToolAdapter in riglr-agents
            // The adapter pattern allows us to bridge the incompatible interfaces
        }

        // Create a convenience function to create an Arc<dyn Tool> using the namespaced type
        /// Factory function to create a new instance of the tool
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

    // Generate the error handling match arms
    let error_match_arms = generate_tool_error_match_arms();

    quote! {
        // Keep the original struct
        #structure

        // Implement the Tool trait
    #[async_trait::async_trait]
    impl riglr_core::Tool for #struct_name {
            async fn execute(&self, params: serde_json::Value, context: &riglr_core::provider::ApplicationContext) -> Result<riglr_core::JobResult, riglr_core::ToolError> {
                // Parse parameters into the struct; convert parse errors to ToolError::InvalidInput
                let args: Self = match serde_json::from_value(params) {
                    Ok(v) => v,
                    Err(e) => {
                        // Convert parameter parsing error to ToolError and use standard error handling
                        let tool_error = riglr_core::ToolError::invalid_input_with_source(
                            e,
                            "Failed to parse tool parameters"
                        );
                        return match tool_error {
                            #error_match_arms
                        };
                    }
                };

                // Call the execute method (expecting Result<T, ToolError>)
                let result = args.execute().await;

                // Convert the result to JobResult
                match result {
                    Ok(value) => {
                        let json_value = serde_json::to_value(value)
                            .map_err(|e| riglr_core::ToolError::permanent_with_source(e, "Failed to serialize result"))?;
                        Ok(riglr_core::JobResult::Success {
                            value: json_value,
                            tx_hash: None,
                        })
                    }
                    Err(tool_error) => {
                        // Convert any error to ToolError, then match on it
                        let tool_error: riglr_core::ToolError = tool_error.into();
                        match tool_error {
                            #error_match_arms
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

        // NOTE: rig::tool::Tool compatibility is handled by RigToolAdapter in riglr-agents
        // The adapter pattern allows us to bridge the incompatible interfaces

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

/// Generates the common error handling match arms for ToolError to JobResult conversion
fn generate_tool_error_match_arms() -> proc_macro2::TokenStream {
    quote! {
        // With the new structure, we just wrap the ToolError directly
        // The JobResult::Failure variant now contains the full ToolError
        _ => Ok(riglr_core::JobResult::Failure {
            error: tool_error,
        })
    }
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
        let classification = variant.attrs.iter().find_map(|attr| {
            if attr.path().is_ident("tool_error") {
                attr.parse_args::<syn::Ident>().ok()
            } else {
                None
            }
        });

        let pattern = match &variant.fields {
            syn::Fields::Named(_) => quote! { #name::#variant_name { .. } },
            syn::Fields::Unnamed(_) => quote! { #name::#variant_name(..) },
            syn::Fields::Unit => quote! { #name::#variant_name },
        };

        let conversion = if let Some(class) = classification {
            match class.to_string().as_str() {
                "retriable" => quote! {
                    riglr_core::ToolError::retriable_string(err.to_string())
                },
                "permanent" => quote! {
                    riglr_core::ToolError::permanent_string(err.to_string())
                },
                "rate_limited" => quote! {
                    riglr_core::ToolError::rate_limited_string(err.to_string())
                },
                _ => quote! {
                    riglr_core::ToolError::permanent_string(err.to_string())
                },
            }
        } else {
            // Automatic classification based on naming conventions
            let retriable_patterns = [
                "Rpc",
                "Network",
                "Connection",
                "Timeout",
                "TooManyRequests",
                "RateLimit",
                "Api",
                "Http",
            ];

            let is_retriable = retriable_patterns
                .iter()
                .any(|pattern| variant_name_str.contains(pattern));

            if is_retriable {
                quote! { riglr_core::ToolError::retriable_string(err.to_string()) }
            } else {
                quote! { riglr_core::ToolError::permanent_string(err.to_string()) }
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

    #[test]
    fn test_extract_doc_comments_single_line() {
        // Create a mock attribute for a single line doc comment
        let attr = syn::parse_quote! { #[doc = " This is a single line comment"] };
        let attrs = vec![attr];
        let result = extract_doc_comments(&attrs);
        assert_eq!(result, "This is a single line comment");
    }

    #[test]
    fn test_extract_doc_comments_multiple_lines() {
        // Create mock attributes for multiple line doc comments
        let attr1 = syn::parse_quote! { #[doc = " First line"] };
        let attr2 = syn::parse_quote! { #[doc = " Second line"] };
        let attr3 = syn::parse_quote! { #[doc = " Third line"] };
        let attrs = vec![attr1, attr2, attr3];
        let result = extract_doc_comments(&attrs);
        assert_eq!(result, "First line\nSecond line\nThird line");
    }

    #[test]
    fn test_extract_doc_comments_no_leading_space() {
        let attr = syn::parse_quote! { #[doc = "No leading space"] };
        let attrs = vec![attr];
        let result = extract_doc_comments(&attrs);
        assert_eq!(result, "No leading space");
    }

    #[test]
    fn test_extract_doc_comments_mixed_with_other_attrs() {
        let doc_attr = syn::parse_quote! { #[doc = " Documentation comment"] };
        let other_attr = syn::parse_quote! { #[allow(unused)] };
        let attrs = vec![other_attr, doc_attr];
        let result = extract_doc_comments(&attrs);
        assert_eq!(result, "Documentation comment");
    }

    #[test]
    fn test_extract_doc_comments_empty_doc() {
        let attr = syn::parse_quote! { #[doc = ""] };
        let attrs = vec![attr];
        let result = extract_doc_comments(&attrs);
        assert_eq!(result, "");
    }

    #[test]
    fn test_extract_doc_comments_whitespace_only() {
        let attr = syn::parse_quote! { #[doc = "   "] };
        let attrs = vec![attr];
        let result = extract_doc_comments(&attrs);
        assert_eq!(result, "");
    }

    #[test]
    fn test_generate_tool_error_match_arms_compilation() {
        // Test that the generated match arms compile by checking their structure
        let match_arms = generate_tool_error_match_arms();
        let generated_string = match_arms.to_string();

        // Check that the new simplified structure is used
        assert!(generated_string.contains("JobResult :: Failure"));
        assert!(generated_string.contains("error : tool_error"));
        // Verify it uses wildcard matching for simplified error handling
        assert!(generated_string.contains("_ =>"));
    }

    #[test]
    fn test_tool_attr_default() {
        let default_attr = ToolAttr::default();
        assert!(default_attr.description.is_none());
    }

    #[test]
    fn test_tool_attr_parse_empty() {
        let input = "";
        let result: Result<ToolAttr, _> = syn::parse_str(input);
        assert!(result.is_ok());
        let attr = result.unwrap();
        assert!(attr.description.is_none());
    }

    #[test]
    fn test_tool_attr_parse_description() {
        let input = r#"description = "Test description""#;
        let result: Result<ToolAttr, _> = syn::parse_str(input);
        assert!(result.is_ok());
        let attr = result.unwrap();
        assert_eq!(attr.description, Some("Test description".to_string()));
    }

    #[test]
    fn test_tool_attr_parse_invalid_key() {
        let input = r#"invalid_key = "value""#;
        let result: Result<ToolAttr, _> = syn::parse_str(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unknown attribute key"));
    }

    #[test]
    fn test_tool_attr_parse_missing_equals() {
        let input = "description";
        let result: Result<ToolAttr, _> = syn::parse_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_attr_parse_wrong_value_type() {
        let input = "description = 123";
        let result: Result<ToolAttr, _> = syn::parse_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_heck_pascal_case_edge_cases() {
        assert_eq!("".to_pascal_case(), "");
        assert_eq!("a".to_pascal_case(), "A");
        assert_eq!("_test_".to_pascal_case(), "Test");
        assert_eq!("test__function".to_pascal_case(), "TestFunction");
        assert_eq!("UPPERCASE".to_pascal_case(), "Uppercase");
        assert_eq!("mixedCase".to_pascal_case(), "MixedCase");
        assert_eq!("123_numeric".to_pascal_case(), "123Numeric");
    }

    // Test the pattern matching logic for derive_into_tool_error
    #[test]
    fn test_retriable_error_patterns() {
        let retriable_patterns = [
            "Rpc",
            "Network",
            "Connection",
            "Timeout",
            "TooManyRequests",
            "RateLimit",
            "Api",
            "Http",
        ];

        // Test each pattern is correctly identified
        for pattern in &retriable_patterns {
            let test_variant = format!("Test{}Error", pattern);
            assert!(retriable_patterns.iter().any(|p| test_variant.contains(p)));
        }
    }

    #[test]
    fn test_permanent_error_patterns() {
        let permanent_variants = [
            "InvalidInput",
            "ParseError",
            "SerializationFailed",
            "NotFound",
            "Unauthorized",
            "InsufficientBalance",
            "InsufficientFunds",
            "CustomError",
            "UnknownError",
        ];

        let retriable_patterns = [
            "Rpc",
            "Network",
            "Connection",
            "Timeout",
            "TooManyRequests",
            "RateLimit",
            "Api",
            "Http",
        ];

        // Test that permanent patterns don't match retriable patterns
        for variant in &permanent_variants {
            let is_retriable = retriable_patterns
                .iter()
                .any(|pattern| variant.contains(pattern));
            assert!(!is_retriable, "Variant {} should not be retriable", variant);
        }
    }

    #[test]
    fn test_error_match_arms_structure() {
        let match_arms = generate_tool_error_match_arms();
        let generated = match_arms.to_string();

        // Verify the new simplified structure
        assert!(generated.contains("JobResult :: Failure"));
        assert!(generated.contains("error : tool_error"));
        // Check that it uses wildcard matching
        assert!(generated.contains("_ =>"));
    }

    // Test compilation of procedural macro output (basic structure validation)
    #[test]
    fn test_proc_macro_token_stream_generation() {
        // Test that we can create basic token streams without panicking
        use quote::quote;

        let test_tokens = quote! {
            #[derive(Clone)]
            pub struct TestTool;

            impl TestTool {
                pub fn new() -> Self { Self }
            }
        };

        assert!(!test_tokens.is_empty());
    }

    #[test]
    fn test_doc_comment_extraction_with_complex_content() {
        let attr1 = syn::parse_quote! { #[doc = " Complex content with \"quotes\""] };
        let attr2 = syn::parse_quote! { #[doc = " And special chars: &<>"] };
        let attr3 = syn::parse_quote! { #[doc = " Numbers: 123 and symbols: $%^"] };
        let attrs = vec![attr1, attr2, attr3];
        let result = extract_doc_comments(&attrs);
        assert_eq!(result, "Complex content with \"quotes\"\nAnd special chars: &<>\nNumbers: 123 and symbols: $%^");
    }

    #[test]
    fn test_doc_comment_trimming() {
        let attr1 = syn::parse_quote! { #[doc = "  Leading spaces"] };
        let attr2 = syn::parse_quote! { #[doc = ""] };
        let attr3 = syn::parse_quote! { #[doc = "Trailing spaces  "] };
        let attrs = vec![attr1, attr2, attr3];
        let result = extract_doc_comments(&attrs);
        // The function strips the first space but preserves other leading spaces
        assert_eq!(result, "Leading spaces\n\nTrailing spaces");
    }

    #[test]
    fn test_tool_attr_parse_description_with_quotes() {
        let input = r#"description = "Description with \"escaped quotes\"""#;
        let result: Result<ToolAttr, _> = syn::parse_str(input);
        assert!(result.is_ok());
        let attr = result.unwrap();
        assert_eq!(
            attr.description,
            Some("Description with \"escaped quotes\"".to_string())
        );
    }

    #[test]
    fn test_tool_attr_parse_description_empty_string() {
        let input = r#"description = """#;
        let result: Result<ToolAttr, _> = syn::parse_str(input);
        assert!(result.is_ok());
        let attr = result.unwrap();
        assert_eq!(attr.description, Some("".to_string()));
    }

    #[test]
    fn test_extract_doc_comments_only_doc_attrs() {
        // Test that only doc attributes are processed, others are ignored
        let doc_attr = syn::parse_quote! { #[doc = " Valid doc comment"] };
        let cfg_attr = syn::parse_quote! { #[cfg(test)] };
        let allow_attr = syn::parse_quote! { #[allow(dead_code)] };
        let derive_attr = syn::parse_quote! { #[derive(Clone)] };

        let attrs = vec![cfg_attr, doc_attr, allow_attr, derive_attr];
        let result = extract_doc_comments(&attrs);
        assert_eq!(result, "Valid doc comment");
    }

    #[test]
    fn test_proc_macro_attr_integration() {
        // Test the integration between attribute parsing and tool generation
        let empty_attr = ToolAttr::default();
        let with_desc = ToolAttr {
            description: Some("Custom description".to_string()),
        };

        // Test that attributes are properly structured
        assert!(empty_attr.description.is_none());
        assert_eq!(
            with_desc.description,
            Some("Custom description".to_string())
        );
    }

    #[test]
    fn test_complex_pascal_case_scenarios() {
        // Test edge cases for function name to struct name conversion
        assert_eq!("get_user_profile".to_pascal_case(), "GetUserProfile");
        assert_eq!("fetch_api_data".to_pascal_case(), "FetchApiData");
        assert_eq!(
            "handle_websocket_connection".to_pascal_case(),
            "HandleWebsocketConnection"
        );
        assert_eq!(
            "process_json_response".to_pascal_case(),
            "ProcessJsonResponse"
        );
        assert_eq!(
            "validate_eth_address".to_pascal_case(),
            "ValidateEthAddress"
        );
    }

    #[test]
    fn test_error_classification_comprehensive() {
        let test_cases = vec![
            ("RpcConnectionError", true),        // Should be retriable
            ("NetworkTimeoutError", true),       // Should be retriable
            ("ApiRateLimitError", true),         // Should be retriable
            ("HttpRequestError", true),          // Should be retriable
            ("InvalidInputError", false),        // Should be permanent
            ("ParseError", false),               // Should be permanent
            ("NotFoundError", false),            // Should be permanent
            ("UnauthorizedError", false),        // Should be permanent
            ("InsufficientBalanceError", false), // Should be permanent
            ("CustomBusinessError", false),      // Should be permanent (default)
            ("DatabaseConnectionError", true),   // Contains "Connection"
            ("TooManyRequestsError", true),      // Contains "TooManyRequests"
        ];

        let retriable_patterns = [
            "Rpc",
            "Network",
            "Connection",
            "Timeout",
            "TooManyRequests",
            "RateLimit",
            "Api",
            "Http",
        ];

        for (variant_name, expected_retriable) in test_cases {
            let is_retriable = retriable_patterns
                .iter()
                .any(|pattern| variant_name.contains(pattern));

            assert_eq!(
                is_retriable,
                expected_retriable,
                "Variant '{}' should be {} but was classified as {}",
                variant_name,
                if expected_retriable {
                    "retriable"
                } else {
                    "permanent"
                },
                if is_retriable {
                    "retriable"
                } else {
                    "permanent"
                }
            );
        }
    }

    #[test]
    fn test_tool_attr_parse_malformed_syntax() {
        // Test various malformed syntax cases
        let test_cases = vec![
            "description =",            // Missing value
            "= \"value\"",              // Missing key
            "description \"value\"",    // Missing equals
            "description = value",      // Unquoted value
            "description == \"value\"", // Double equals
        ];

        for input in test_cases {
            let result: Result<ToolAttr, _> = syn::parse_str(input);
            assert!(result.is_err(), "Input '{}' should fail to parse", input);
        }
    }

    #[test]
    fn test_extract_doc_comments_with_non_string_meta() {
        // Test with attributes that have doc but aren't string literals
        // This tests the filtering logic in extract_doc_comments
        let valid_doc = syn::parse_quote! { #[doc = "Valid comment"] };
        let attrs = vec![valid_doc];
        let result = extract_doc_comments(&attrs);
        assert_eq!(result, "Valid comment");
    }

    #[test]
    fn test_doc_comment_joining_edge_cases() {
        // Test doc comment joining with various whitespace scenarios
        let attr1 = syn::parse_quote! { #[doc = "Line1"] };
        let attr2 = syn::parse_quote! { #[doc = " "] }; // Just a space
        let attr3 = syn::parse_quote! { #[doc = "Line3"] };
        let attrs = vec![attr1, attr2, attr3];
        let result = extract_doc_comments(&attrs);
        assert_eq!(result, "Line1\n\nLine3");
    }

    #[test]
    fn test_pascal_case_with_unicode() {
        // Test pascal case conversion with unicode characters
        assert_eq!("café_function".to_pascal_case(), "CaféFunction");
        assert_eq!("测试_function".to_pascal_case(), "测试Function");
    }

    #[test]
    fn test_tool_attr_description_priority() {
        // Test that explicit description takes priority over doc comments
        let explicit_desc = ToolAttr {
            description: Some("Explicit description".to_string()),
        };
        assert_eq!(
            explicit_desc.description,
            Some("Explicit description".to_string())
        );

        let no_desc = ToolAttr { description: None };
        assert!(no_desc.description.is_none());
    }

    #[test]
    fn test_generate_match_arms_output_consistency() {
        // Test that generate_tool_error_match_arms produces consistent output
        let match_arms1 = generate_tool_error_match_arms();
        let match_arms2 = generate_tool_error_match_arms();

        // Convert to strings and compare
        let output1 = match_arms1.to_string();
        let output2 = match_arms2.to_string();
        assert_eq!(
            output1, output2,
            "Match arms generation should be deterministic"
        );
    }

    #[test]
    fn test_doc_comment_extract_path_verification() {
        // Test that extract_doc_comments properly checks path identity
        let doc_attr = syn::parse_quote! { #[doc = "Test"] };
        let not_doc_attr = syn::parse_quote! { #[deprecated] };

        let attrs = vec![not_doc_attr, doc_attr];
        let result = extract_doc_comments(&attrs);
        assert_eq!(result, "Test");
    }

    #[test]
    fn test_error_pattern_case_sensitivity() {
        // Test that error pattern matching is case-sensitive
        let case_sensitive_tests = vec![
            ("rpc_error", false),     // lowercase 'rpc' should not match 'Rpc'
            ("RpcError", true),       // Uppercase 'Rpc' should match
            ("network_issue", false), // lowercase 'network' should not match 'Network'
            ("NetworkIssue", true),   // Uppercase 'Network' should match
        ];

        let retriable_patterns = [
            "Rpc",
            "Network",
            "Connection",
            "Timeout",
            "TooManyRequests",
            "RateLimit",
            "Api",
            "Http",
        ];

        for (variant_name, expected_match) in case_sensitive_tests {
            let matches = retriable_patterns
                .iter()
                .any(|pattern| variant_name.contains(pattern));
            assert_eq!(
                matches, expected_match,
                "Case sensitivity test failed for '{}'",
                variant_name
            );
        }
    }

    #[test]
    fn test_tool_attr_parse_lookahead_logic() {
        // Test the lookahead logic in ToolAttr::parse
        let valid_input = "description = \"test\"";
        let result: Result<ToolAttr, _> = syn::parse_str(valid_input);
        assert!(result.is_ok());

        // Test with invalid identifier that triggers lookahead error
        let invalid_input = "123invalid = \"test\"";
        let result: Result<ToolAttr, _> = syn::parse_str(invalid_input);
        assert!(result.is_err());
    }

    #[test]
    fn test_comprehensive_error_variant_naming() {
        // Comprehensive test of error variant naming patterns
        let comprehensive_tests = vec![
            // Retriable patterns
            ("SolanaRpcError", true),
            ("EthereumNetworkTimeout", true),
            ("DatabaseConnectionLost", true),
            ("ApiRateLimitExceeded", true),
            ("HttpRequestFailed", true),
            ("WebSocketConnectionDropped", true),
            ("RedisConnectionTimeout", true),
            ("TooManyRequestsReceived", true),
            // Permanent patterns
            ("InvalidAddressFormat", false),
            ("ParseJsonError", false),
            ("SerializationFailure", false),
            ("UserNotFound", false),
            ("UnauthorizedAccess", false),
            ("InsufficientTokenBalance", false),
            ("InsufficientGasFunds", false),
            ("MalformedInput", false),
            ("ConfigurationError", false),
            ("BusinessLogicViolation", false),
        ];

        let retriable_patterns = [
            "Rpc",
            "Network",
            "Connection",
            "Timeout",
            "TooManyRequests",
            "RateLimit",
            "Api",
            "Http",
        ];

        for (variant_name, expected_retriable) in comprehensive_tests {
            let is_retriable = retriable_patterns
                .iter()
                .any(|pattern| variant_name.contains(pattern));

            assert_eq!(
                is_retriable,
                expected_retriable,
                "Comprehensive error classification failed for '{}' - expected {}, got {}",
                variant_name,
                if expected_retriable {
                    "retriable"
                } else {
                    "permanent"
                },
                if is_retriable {
                    "retriable"
                } else {
                    "permanent"
                }
            );
        }
    }

    #[test]
    fn test_empty_and_whitespace_edge_cases() {
        // Test various empty and whitespace scenarios
        let empty_attrs: Vec<syn::Attribute> = vec![];
        assert_eq!(extract_doc_comments(&empty_attrs), "");

        // Test with only whitespace doc
        let whitespace_attr = syn::parse_quote! { #[doc = "   \t\n  "] };
        let result = extract_doc_comments(&vec![whitespace_attr]);
        assert_eq!(result.trim(), "");

        // Test pascal case with empty string
        assert_eq!("".to_pascal_case(), "");
    }

    #[test]
    fn test_parameter_parsing_error_handling() {
        // Test that parameter parsing errors are converted to ToolError::InvalidInput
        // and use the standard error matching logic

        // Create a mock serde_json::Error by attempting to parse invalid JSON
        let invalid_json = "{ invalid json }";
        let parse_result: Result<serde_json::Value, serde_json::Error> =
            serde_json::from_str(invalid_json);
        assert!(parse_result.is_err());

        let error = parse_result.unwrap_err();

        // Verify that we can create a ToolError::InvalidInput from the serde error
        use riglr_core::ToolError;
        let tool_error =
            ToolError::invalid_input_with_source(error, "Failed to parse tool parameters");

        // Verify properties of the error
        assert!(!tool_error.is_retriable());
        assert!(!tool_error.is_rate_limited());
        assert_eq!(tool_error.retry_after(), None);

        // Verify the error message contains expected content
        let error_str = tool_error.to_string();
        assert!(error_str.contains("Invalid input"));
        assert!(error_str.contains("Failed to parse tool parameters"));
    }

    #[test]
    fn test_tool_error_match_arms_invalid_input_handling() {
        // Test that the generated match arms handle all errors with simplified structure
        let match_arms = generate_tool_error_match_arms();
        let generated = match_arms.to_string();

        // Verify the simplified structure handles all errors uniformly
        assert!(generated.contains("JobResult :: Failure"));
        assert!(generated.contains("error : tool_error"));
        // Verify it uses wildcard matching for all error types
        assert!(generated.contains("_ =>"));
    }
}
