# Changelog - riglr-config

## [Unreleased]

### Added
- Added `router`, `quoter`, and `factory` fields to `ChainContract` struct for Uniswap V3 contract addresses
- Created comprehensive `chains.toml.example` file with contract addresses for major chains (Ethereum, Polygon, Arbitrum, Optimism, Base, BSC, Avalanche, and testnets)
- Environment variable override support for contract addresses using patterns like `ROUTER_{chain_id}`, `QUOTER_{chain_id}`, `FACTORY_{chain_id}`
- **New trait**: `AddressValidator` trait for pluggable blockchain address validation
  - Allows different blockchain validation logic to be injected without tight coupling
  - Supports dependency inversion principle - config layer defines interface, blockchain crates provide implementation

### Changed
- **BREAKING**: Removed `Validator` trait - config structs now directly implement `validate_config()` method
- **BREAKING**: `enable_bridging` is now disabled by default to avoid requiring LIFI_API_KEY for basic usage
- **BREAKING**: `NetworkConfig::validate_config()` now accepts optional `AddressValidator` parameter
  - Address validation is now optional and only performed when a validator is provided
  - Config validation is decoupled from specific blockchain validation logic
  - Application binaries are responsible for providing appropriate validators
- Simplified validation API - each config struct now has its own `validate_config()` method

### Fixed
- Fixed default configuration validation that was requiring LIFI_API_KEY even when bridging was not actively used
- Improved cross-dependency validation between configuration sections

### Removed
- Removed legacy `validation.rs` module (functionality already integrated into config structs)
- Removed `#[allow(dead_code)]` attributes from `network.rs` by properly scoping test-only functions with `#[cfg(test)]`
- **BREAKING**: Removed `riglr-evm-common` dependency to enforce architectural purity
  - `riglr-config` is now blockchain-agnostic and has no dependencies on blockchain-specific crates
  - Address validation is now delegated to application binaries via the `AddressValidator` trait
- Removed EVM-specific address validation tests (moved to appropriate blockchain-specific crates)

### Maintenance
- Cleaned up unused imports and code
- Functions `extract_by_prefix`, `extract_chain_rpc_urls`, and `extract_contract_overrides` are now properly marked as test-only

## [0.2.0] - Previous Release
- Initial configuration management system