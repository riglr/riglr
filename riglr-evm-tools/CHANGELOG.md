# Changelog - riglr-evm-tools

## [Unreleased]

### Added
- **Smart Chain ID Resolution**: Intelligent `chain_id` parameter handling across all EVM tools
  - Tools automatically use `chain_id` from active EVM SignerContext when parameter is None
  - Fallback hierarchy: explicit parameter → SignerContext chain_id → Ethereum mainnet (1)
  - Enhanced developer experience by reducing need to explicitly pass chain_id in signed contexts

### Changed
- **Balance Tools**: Updated with smart chain_id resolution
  - `get_eth_balance_with_context()`: Auto-resolves chain_id from SignerContext
  - `get_erc20_balance_with_context()`: Auto-resolves chain_id from SignerContext
- **Swap Tools**: Updated with smart chain_id resolution
  - `get_uniswap_quote()`: Auto-resolves chain_id from SignerContext for quoter address lookup
- **Network Tools**: Updated with smart chain_id resolution (for future extensibility)
  - `get_block_number_with_context()`: Smart resolution with detailed logging
  - `get_gas_price_with_context()`: Smart resolution with detailed logging
- **Enhanced Documentation**: All tool functions now document the smart chain_id behavior
  - Clear explanation of resolution hierarchy in function docs
  - Examples showing usage with and without explicit chain_id

### Fixed
- Resolved unused variable warnings in balance checking functions

### Documentation
- Comprehensive documentation of smart chain_id resolution behavior
- Usage examples for both explicit and automatic chain_id scenarios

## [0.3.0] - Previous Release
- Initial EVM tools implementation