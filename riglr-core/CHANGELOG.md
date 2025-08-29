# Changelog - riglr-core

## [Unreleased]

### Added
- **BREAKING**: Automatic rate limiting integration in `ApplicationContext` and `ToolWorker`
  - `ApplicationContext` now includes a `rate_limiter: Arc<RateLimiter>` field initialized with default settings (100 requests per minute)
  - `ToolWorker::process_job` automatically applies rate limiting based on `SignerContext::user_id()` before tool execution
  - Rate limiting is now handled at the framework level, eliminating need for manual rate limiter checks in tools
- **Production Security Documentation**: Comprehensive security guidance in `util::secure_keys` module and main README
  - Clear warnings that file-based key storage is for development only
  - Detailed recommendations for production key management (OS keychains, cloud secret managers, HSMs)
  - Practical examples using `keyring` crate and cloud providers (AWS Secrets Manager, etc.)
  - Security best practices for key rotation, access control, and audit logging
- Unified retry logic using `backoff` crate's `ExponentialBackoff` implementation
- Enhanced `retry_async` function in `retry.rs` with proper exponential backoff and jitter
- Error classification system (`ErrorClass`) for determining retry behavior
- Time-based token bucket algorithm for `RateLimiter` with continuous token replenishment
- Fractional token support in rate limiter for smoother rate limiting
- Warning log when generic errors are converted to `ToolError::Permanent` to encourage specific error handling

### Changed
- **Rate Limiter**: Replaced request-count-based token replenishment with time-based algorithm
  - Tokens now replenish continuously based on elapsed time
  - More predictable rate limiting behavior
  - Better handling of burst traffic with configurable burst size
- **BREAKING**: `GenericTransactionProcessor` now returns errors for `get_status` and `wait_for_confirmation` methods instead of placeholder implementations
- Refactored `ToolWorker::execute_with_retries` to use unified retry helper instead of manual retry loops
- Removed redundant retry implementations (`create_backoff_strategy` and `wait_with_backoff` methods)
- Improved safety by ensuring generic implementations fail explicitly rather than returning incorrect results

### Removed
- **BREAKING**: Removed EVM-specific utilities from `util` module:
  - `parse_evm_address` - Use `riglr_evm_common::parse_evm_address` instead
  - `format_evm_address` - Use `riglr_evm_common::format_address` instead
  - `is_valid_evm_address` - Use `riglr_evm_common::validate_evm_address` instead
  - These functions have been moved to the new `riglr-evm-common` crate to maintain blockchain agnosticism
- **BREAKING**: Removed `riglr-evm-common` dependency to enforce architectural purity
  - `riglr-core` is now completely chain-agnostic and has no dependencies on blockchain-specific crates
  - This ensures strict unidirectional dependency flow in the riglr architecture

### Fixed
- Fixed compilation issues with backoff crate integration
- Resolved unused variable warnings in retry logic
- Fixed test failures related to configuration validation

### Security
- Generic transaction processor now safely indicates when chain-specific implementation is required
- Prevents accidental use of placeholder implementations in production

## [0.2.0] - Previous Release
- Core abstractions and tool worker implementation