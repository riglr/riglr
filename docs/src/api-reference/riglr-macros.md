# riglr-macros API Reference

Comprehensive API documentation for the `riglr-macros` crate.

## Table of Contents

### Tools

- [`advanced_swap`](#advanced_swap)
- [`async_tool`](#async_tool)
- [`bridge_tokens`](#bridge_tokens)
- [`calculate_compound_interest`](#calculate_compound_interest)
- [`compute_hash`](#compute_hash)
- [`documented_tool`](#documented_tool)
- [`fetch`](#fetch)
- [`jupiter_swap`](#jupiter_swap)
- [`process_complex_data`](#process_complex_data)
- [`process_data`](#process_data)
- [`swap_tokens`](#swap_tokens)
- [`sync_tool`](#sync_tool)

### Structs

- [`Args`](#args)
- [`SwapTokensArgs`](#swaptokensargs)
- [`SwapTokensTool`](#swaptokenstool)
- [`Tool`](#tool)

### Functions

- [`complex_generic`](#complex_generic)
- [`derive_into_tool_error`](#derive_into_tool_error)
- [`execute`](#execute)
- [`new`](#new)
- [`new`](#new)
- [`swap_tokens_tool`](#swap_tokens_tool)
- [`tool`](#tool)

## Tools

### advanced_swap

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[tool]
```

```rust
async fn advanced_swap( input_mint: String, output_mint: String, amount: u64, ) -> Result<SwapResult, SwapError>
```

Advanced token swap with detailed error handling

---

### async_tool

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[tool]
```

```rust
async fn async_tool() -> Result<String, ToolError>
```

---

### bridge_tokens

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[tool]
```

```rust
async fn bridge_tokens( /// Source token address source_token: String, /// Destination chain identifier dest_chain: String, /// Destination token address dest_token: String, /// Amount to bridge in base units amount: u64, /// Recipient address on destination chain recipient: String, ) -> Result<BridgeResult, BridgeError>
```

Bridge tokens between different blockchains

Automatically detects the source chain from the current signer
and handles cross-chain bridging operations.

---

### calculate_compound_interest

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[tool]
```

```rust
fn calculate_compound_interest( /// Principal amount in dollars principal: f64, /// Annual interest rate as a decimal (e.g., 0.05 for 5%) annual_rate: f64, /// Time period in years years: f64, /// Number of times interest is compounded per year compounds_per_year: u32, ) -> Result<f64, ToolError>
```

Calculate compound interest for a given principal, rate, and time

This is a computational tool that doesn't require async operations,
so it's implemented as a synchronous function that runs efficiently
within the async Tool framework.

---

### compute_hash

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[tool]
```

```rust
async fn compute_hash( /// Data to hash data: Vec<u8>, /// Number of iterations iterations: u32, ) -> Result<String, ToolError>
```

CPU-intensive cryptographic operation

This uses spawn_blocking to avoid blocking the async runtime

---

### documented_tool

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[tool]
```

```rust
async fn documented_tool( /// This helps the AI understand this parameter param: String, ) -> Result<String, ToolError>
```

This description helps AI models understand the tool's purpose

---

### fetch

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[tool(description = "Fetches the URL and returns the body as text.")]
```

```rust
async fn fetch(url: String) -> Result<String, Error>
```

---

### jupiter_swap

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[tool]
```

```rust
async fn jupiter_swap( /// Input token mint address input_mint: String, /// Output token mint address output_mint: String, /// Amount to swap in base units amount: u64, /// Maximum slippage in basis points max_slippage_bps: u16, ) -> Result<String, SwapError>
```

Swap tokens on Solana using Jupiter aggregator

This tool automatically accesses the current signer from the context,
eliminating the need to pass signing credentials explicitly.

---

### process_complex_data

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[tool]
```

```rust
async fn process_complex_data( /// JSON representation of the data to process data: serde_json::Value, ) -> Result<ProcessedResult, ProcessError>
```

---

### process_data

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[tool]
```

```rust
async fn process_data<T>( /// The data to process (must be JSON-serializable) data: T, /// Processing options options: ProcessingOptions, ) -> Result<ProcessedData, ProcessingError> where T: Serialize + Deserialize + JsonSchema + Send + Sync,
```

Generic tool that can process any serializable data

---

### swap_tokens

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[tool]
```

```rust
async fn swap_tokens( /// Source token mint address from_mint: String, /// Destination token mint address to_mint: String, /// Amount to swap in base units amount: u64, /// Optional slippage tolerance (default: 0.5%) #[serde(default = "default_slippage")] slippage_bps: Option<u16>, ) -> Result<String, SwapError>
```

---

### sync_tool

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[tool]
```

```rust
fn sync_tool() -> Result<String, ToolError>
```

---

## Structs

### Args

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct Args { #(#param_fields),*
```

---

### SwapTokensArgs

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
```

```rust
pub struct SwapTokensArgs { /// Source token mint address pub from_mint: String, /// Destination token mint address pub to_mint: String, /// Amount to swap in base units pub amount: u64, /// Optional slippage tolerance (default: 0.5%)
```

---

### SwapTokensTool

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[derive(Clone)]
```

```rust
pub struct SwapTokensTool;
```

---

### Tool

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[derive(Clone)]
```

```rust
pub struct Tool;
```

---

## Functions

### complex_generic

**Source**: `src/lib.rs`

```rust
// async fn complex_generic<T: ComplexTrait>(data: T) -> Result<(), Error>
```

---

### derive_into_tool_error

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[proc_macro_derive(IntoToolError, attributes(tool_error))]
```

```rust
pub fn derive_into_tool_error(input: TokenStream) -> TokenStream
```

Derives automatic conversion from an error enum to ToolError.

This macro generates a `From<YourError> for ToolError` implementation
that automatically classifies errors as retriable or permanent based on
naming conventions in variant names.

# Classification Rules

Errors are classified as **retriable** if their variant names contain:
- `Rpc`, `Network`, `Connection`, `Timeout`, `TooManyRequests`, `RateLimit`
- `Api` (for external API errors)
- `Http` (for HTTP-related errors)

Errors are classified as **permanent** if their variant names contain:
- `Invalid`, `Parse`, `Serialization`, `NotFound`, `Unauthorized`
- `InsufficientBalance`, `InsufficientFunds`
- All other unmatched variants (conservative default)

# Custom Classification

You can override the automatic classification using attributes:

```rust,ignore
#[derive(IntoToolError)]
enum MyError {
#[tool_error(retriable)]
CustomError(String),

#[tool_error(permanent)]
NetworkError(String), // Override default retriable classification

#[tool_error(rate_limited)]
ApiQuotaExceeded,
}
```

# Examples

```rust,ignore
use riglr_macros::IntoToolError;
use thiserror::Error;

#[derive(Error, Debug, IntoToolError)]
enum SolanaError {
#[error("RPC error: {0}")]
RpcError(String),  // Automatically retriable

#[error("Invalid address: {0}")]
InvalidAddress(String),  // Automatically permanent

#[error("Network timeout")]
NetworkTimeout,  // Automatically retriable

#[error("Insufficient balance")]
InsufficientBalance,  // Automatically permanent

#[tool_error(retriable)]
#[error("Custom error: {0}")]
CustomError(String),  // Explicitly retriable
}
```

---

### execute

**Source**: `src/lib.rs`

```rust
pub async fn execute(&self) -> Result<String, ToolError>
```

---

### new

**Source**: `src/lib.rs`

```rust
pub fn new() -> Self
```

---

### new

**Source**: `src/lib.rs`

```rust
pub fn new() -> Self
```

Create a new instance of this tool

---

### swap_tokens_tool

**Source**: `src/lib.rs`

```rust
pub fn swap_tokens_tool() -> std::sync::Arc<dyn riglr_core::Tool>
```

---

### tool

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[proc_macro_attribute]
```

```rust
pub fn tool(attr: TokenStream, item: TokenStream) -> TokenStream
```

The `#[tool]` procedural macro that converts functions and structs into Tool implementations.

This macro supports:
- Async functions with arbitrary parameters and Result return types
- Structs that have an `execute` method
- Automatic JSON schema generation using `schemars`
- Documentation extraction from doc comments
- Parameter descriptions from doc comments on function arguments

Attributes supported:
- description = "..."

---


---

*This documentation was automatically generated from the source code.*