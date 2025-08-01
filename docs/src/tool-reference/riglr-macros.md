# riglr-macros Tool Reference

This page contains documentation for tools provided by the `riglr-macros` crate.

## Available Tools

- [`execute`](#execute) - src/lib.rs
- [`derive_into_tool_error`](#derive_into_tool_error) - src/lib.rs

## Tool Functions

### execute

**Source**: `src/lib.rs`

```rust
pub async fn execute(&self) -> Result<String, ToolError>
```

*No documentation available for this tool.*

---

### derive_into_tool_error

**Source**: `src/lib.rs`

```rust
pub fn derive_into_tool_error(input: TokenStream) -> TokenStream
```

**Documentation:**

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


---


---

*This documentation was automatically generated from the source code.*