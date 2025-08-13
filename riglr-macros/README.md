# riglr-macros

Procedural macros for riglr.

## `#[tool]` macro

Implements `riglr_core::Tool` for async functions and structs, generating:
- Args struct with serde/schemars
- Tool trait impl with error mapping
- Description wiring for AI via `Tool::description()`

### Description attribute

You can set an explicit, AI-friendly description string:

```rust
#[tool(description = "Fetches a URL and returns the body as text.")]
async fn fetch(url: String) -> Result<String, Error> { /* ... */ }
```

Priority:
1. Attribute `description = "..."`
2. Rust doc comments on the item
3. Empty string

This description is returned by `Tool::description()`.
