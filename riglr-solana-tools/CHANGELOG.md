# Changelog - riglr-solana-tools

## [Unreleased]

### Changed
- **Rate Limiting Modernization**: Removed manual rate limiting checks in favor of framework-level rate limiting
  - Rate limiting is now automatically handled by riglr-core's ToolWorker before tool execution
  - Framework applies rate limits based on SignerContext user_id when available
  - Eliminates need for manual rate limiter extension checks in individual tools

### Removed
- **Manual Rate Limiting**: Removed explicit rate limiter checks in pump.fun tools
  - Removed manual `context.get_extension::<Arc<RateLimiter>>()` calls
  - Removed direct `rate_limiter.check_rate_limit()` invocations
  - Removed unused `riglr_core::util::RateLimiter` import
- **Boilerplate Code**: Eliminated redundant rate limiting implementation across tools

### Improved
- **Developer Experience**: Tools are now simpler and focus on their core functionality
- **Consistency**: All tools in the riglr ecosystem now use consistent framework-level rate limiting
- **Architecture**: Better separation of concerns between framework responsibilities and tool logic

### Documentation
- Updated comments to clarify that rate limiting is handled by the framework
- Removed rate limiting guidance from tool-specific documentation

## [0.3.0] - Previous Release
- Initial Solana tools implementation with pump.fun support