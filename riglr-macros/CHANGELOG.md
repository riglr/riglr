# Changelog - riglr-macros

## [Unreleased]

### Added
- **Security and Business Logic Validation**: New comprehensive section in Best Practices with practical examples
  - Emphasizes that macro/serde handle format validation, but business logic validation is developer's responsibility
  - Financial operations examples: positive amounts, slippage validation (< 5%), balance checks
  - Smart contract interaction examples: contract whitelisting, re-entrancy protection, gas limit validation
  - Data integrity examples: transaction hash validation, cross-reference verification
- **Stricter Error Handling**: New trybuild tests demonstrating proper error handling patterns
  - `wrapped_std_errors.rs` shows correct pattern using `#[derive(IntoToolError)]` for wrapping stdlib errors
  - `std_library_errors.rs` demonstrates compilation failures for direct stdlib error usage
- Comprehensive "Generated Code Example" section in README showing side-by-side comparison of user code and macro-generated code
- Detailed explanation of what the `#[tool]` macro generates:
  - Args struct for user parameters (ApplicationContext excluded)
  - Tool struct implementation
  - Tool trait implementation with automatic context injection
  - Factory function for creating tool instances

### Changed
- **BREAKING**: Stricter error handling in `#[tool]` macro - removed automatic conversion for standard library errors
  - `std::io::Error` and `reqwest::Error` no longer automatically convert to `ToolError`
  - All error types must now explicitly implement `Into<ToolError>`
  - Encourages use of `#[derive(IntoToolError)]` macro or manual `From` implementations for better error classification
  - Updated documentation and examples to show correct error handling patterns
- Improved documentation clarity with explicit examples of generated code
- Enhanced README with better explanation of type-based dependency injection

### Documentation
- Added clear visualization of macro transformation process
- Explained key points about generated code structure
- Improved understanding for developers using the macro

## [0.2.0] - Previous Release
- Initial procedural macro implementation for tool generation