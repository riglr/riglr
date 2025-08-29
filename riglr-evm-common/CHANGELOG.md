# Changelog - riglr-evm-common

## [Unreleased]

### Added
- **High-Precision Math Feature**: Optional `high-precision` feature flag for exact decimal calculations
  - Adds `rust_decimal` dependency with "serde-with-str" feature when enabled
  - Provides conditional compilation versions of all conversion functions:
    - `eth_to_wei()` / `wei_to_eth()`
    - `gwei_to_wei()` / `wei_to_gwei()` 
    - `token_to_smallest_unit()` / `smallest_unit_to_token()`
  - Default behavior: f64-based conversions (fast, sufficient for most use cases)
  - High-precision behavior: rust_decimal::Decimal-based conversions (exact precision, ideal for financial applications)
- **AddressValidator trait implementation**: `EvmAddressValidator` for riglr-config integration
  - Implements address validation interface defined in riglr-config
  - Enables dependency inversion - config crate defines interface, EVM crate provides implementation
  - Supports pluggable validation without tight coupling between crates

### Changed
- **Conditional imports**: Made imports conditional based on feature flags to eliminate unused import warnings

### Fixed
- Fixed unused import warnings when high-precision feature is enabled

### Documentation
- Feature flag usage examples and compilation instructions
- Clear explanation of precision vs performance trade-offs
- Migration guidance for applications requiring exact precision

## [0.3.0] - Previous Release
- Initial EVM common utilities and types