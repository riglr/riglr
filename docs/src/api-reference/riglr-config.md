# riglr-config API Reference

Comprehensive API documentation for the `riglr-config` crate.

## Table of Contents

### Enums

- [`AiProvider`](#aiprovider)
- [`BlockchainProvider`](#blockchainprovider)
- [`ConfigError`](#configerror)
- [`DataProvider`](#dataprovider)
- [`Environment`](#environment)
- [`EnvironmentSource`](#environmentsource)
- [`Feature`](#feature)

### Functions

- [`app`](#app)
- [`build`](#build)
- [`builder`](#builder)
- [`database`](#database)
- [`disable`](#disable)
- [`enable`](#enable)
- [`exists`](#exists)
- [`extract_by_prefix`](#extract_by_prefix)
- [`extract_chain_rpc_urls`](#extract_chain_rpc_urls)
- [`extract_contract_overrides`](#extract_contract_overrides)
- [`extract_rpc_urls`](#extract_rpc_urls)
- [`features`](#features)
- [`from_env`](#from_env)
- [`generic`](#generic)
- [`get`](#get)
- [`get_ai_key`](#get_ai_key)
- [`get_blockchain_key`](#get_blockchain_key)
- [`get_chain`](#get_chain)
- [`get_or`](#get_or)
- [`get_rpc_url`](#get_rpc_url)
- [`get_supported_chains`](#get_supported_chains)
- [`global`](#global)
- [`has_ai_provider`](#has_ai_provider)
- [`has_blockchain_provider`](#has_blockchain_provider)
- [`has_data_provider`](#has_data_provider)
- [`io`](#io)
- [`is_custom_enabled`](#is_custom_enabled)
- [`is_enabled`](#is_enabled)
- [`load_chain_contracts`](#load_chain_contracts)
- [`network`](#network)
- [`new`](#new)
- [`parse`](#parse)
- [`providers`](#providers)
- [`require`](#require)
- [`try_global`](#try_global)
- [`validate`](#validate)
- [`validate`](#validate)
- [`validate`](#validate)
- [`validate`](#validate)
- [`validate`](#validate)
- [`validate`](#validate)
- [`validate`](#validate)
- [`validate`](#validate)
- [`validate_api_key`](#validate_api_key)
- [`validate_email`](#validate_email)
- [`validate_eth_address`](#validate_eth_address)
- [`validate_percentage`](#validate_percentage)
- [`validate_port`](#validate_port)
- [`validate_positive`](#validate_positive)
- [`validate_range`](#validate_range)
- [`validate_solana_address`](#validate_solana_address)
- [`validate_url`](#validate_url)
- [`validation`](#validation)

### Traits

- [`Validator`](#validator)

### Structs

- [`AppConfig`](#appconfig)
- [`ChainConfig`](#chainconfig)
- [`ChainContract`](#chaincontract)
- [`Config`](#config)
- [`ConfigBuilder`](#configbuilder)
- [`DatabaseConfig`](#databaseconfig)
- [`FeaturesConfig`](#featuresconfig)
- [`NetworkConfig`](#networkconfig)
- [`NetworkTimeouts`](#networktimeouts)
- [`PoolConfig`](#poolconfig)
- [`ProvidersConfig`](#providersconfig)
- [`RetryConfig`](#retryconfig)
- [`TransactionConfig`](#transactionconfig)

## Enums

### AiProvider

**Source**: `src/providers.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
```

```rust
pub enum AiProvider { /// Anthropic Claude AI provider Anthropic, /// OpenAI provider OpenAI, /// Groq provider Groq, /// Perplexity AI provider Perplexity, }
```

AI provider enumeration

**Variants**:

- `Anthropic`
- `OpenAI`
- `Groq`
- `Perplexity`

---

### BlockchainProvider

**Source**: `src/providers.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
```

```rust
pub enum BlockchainProvider { /// Alchemy blockchain data provider Alchemy, /// Infura blockchain infrastructure provider Infura, /// QuickNode blockchain infrastructure provider QuickNode, /// Moralis Web3 development platform Moralis, }
```

Blockchain data provider enumeration

**Variants**:

- `Alchemy`
- `Infura`
- `QuickNode`
- `Moralis`

---

### ConfigError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Debug, Error)]
```

```rust
pub enum ConfigError { /// Environment variable not found #[error("Missing environment variable: {0}")] MissingEnvVar(String), /// Invalid configuration value #[error("Invalid configuration: {0}")] ValidationError(String), /// Failed to parse configuration #[error("Failed to parse configuration: {0}")] ParseError(String), /// IO error #[error("IO error: {0}")] IoError(String), /// Chain not supported #[error("Chain {0} is not supported")] ChainNotSupported(u64), /// Provider not configured #[error("Provider {0} is not configured")] ProviderNotConfigured(String), /// Generic error #[error("{0}")] Generic(String), }
```

Configuration errors

**Variants**:

- `MissingEnvVar(String)`
- `ValidationError(String)`
- `ParseError(String)`
- `IoError(String)`
- `ChainNotSupported(u64)`
- `ProviderNotConfigured(String)`
- `Generic(String)`

---

### DataProvider

**Source**: `src/providers.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
```

```rust
pub enum DataProvider { /// DexScreener DEX analytics provider DexScreener, /// CoinGecko cryptocurrency data provider CoinGecko, /// CoinMarketCap cryptocurrency market data provider CoinMarketCap, /// Twitter social media data provider Twitter, /// LunarCrush social analytics provider LunarCrush, }
```

Data provider enumeration

**Variants**:

- `DexScreener`
- `CoinGecko`
- `CoinMarketCap`
- `Twitter`
- `LunarCrush`

---

### Environment

**Source**: `src/app.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
```

```rust
pub enum Environment { /// Development environment for local testing and debugging Development, /// Staging environment for pre-production testing Staging, /// Production environment for live deployment Production, }
```

Application environment

**Variants**:

- `Development`
- `Staging`
- `Production`

---

### EnvironmentSource

**Source**: `src/environment.rs`

```rust
pub enum EnvironmentSource { /// Load from system environment System, /// Load from .env file DotEnv(String), /// Load from custom source Custom(Box<CustomEnvSource>), }
```

Source for loading environment variables

**Variants**:

- `System`
- `DotEnv(String)`
- `Custom(Box<CustomEnvSource>)`

---

### Feature

**Source**: `src/features.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
```

```rust
pub enum Feature { /// Enable trading functionality Trading, /// Enable cross-chain bridging Bridging, /// Enable social media monitoring SocialMonitoring, /// Enable graph-based memory GraphMemory, /// Enable real-time streaming Streaming, /// Enable webhook notifications Webhooks, /// Enable analytics collection Analytics, /// Enable debug mode Debug, /// Enable experimental features Experimental, }
```

Feature enumeration

**Variants**:

- `Trading`
- `Bridging`
- `SocialMonitoring`
- `GraphMemory`
- `Streaming`
- `Webhooks`
- `Analytics`
- `Debug`
- `Experimental`

---

## Functions

### app

**Source**: `src/lib.rs`

```rust
pub fn app(mut self, config: AppConfig) -> Self
```

Set application configuration

---

### build

**Source**: `src/lib.rs`

```rust
pub fn build(self) -> ConfigResult<Config>
```

Build the configuration

---

### builder

**Source**: `src/lib.rs`

```rust
pub fn builder() -> ConfigBuilder
```

Create a builder for constructing configuration programmatically

---

### database

**Source**: `src/lib.rs`

```rust
pub fn database(mut self, config: DatabaseConfig) -> Self
```

Set database configuration

---

### disable

**Source**: `src/features.rs`

```rust
pub fn disable(&mut self, feature: Feature)
```

Disable a feature

---

### enable

**Source**: `src/features.rs`

```rust
pub fn enable(&mut self, feature: Feature)
```

Enable a feature

---

### exists

**Source**: `src/environment.rs`

```rust
pub fn exists(&self, key: &str) -> bool
```

Check if an environment variable exists

---

### extract_by_prefix

**Source**: `src/environment.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub fn extract_by_prefix(prefix: &str) -> Vec<(String, String)>
```

Helper to extract values by prefix

---

### extract_chain_rpc_urls

**Source**: `src/environment.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub fn extract_chain_rpc_urls() -> Vec<(u64, String)>
```

Helper to extract and parse chain IDs from RPC_URL_{CHAIN_ID} pattern

---

### extract_contract_overrides

**Source**: `src/environment.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub fn extract_contract_overrides(chain_id: u64) -> Vec<(String, String)>
```

Helper to extract and parse contract addresses

---

### extract_rpc_urls

**Source**: `src/network.rs`

```rust
pub fn extract_rpc_urls(&mut self)
```

Extract RPC URLs from environment using RPC_URL_{CHAIN_ID} convention

---

### features

**Source**: `src/lib.rs`

```rust
pub fn features(mut self, config: FeaturesConfig) -> Self
```

Set features configuration

---

### from_env

**Source**: `src/lib.rs`

```rust
pub fn from_env() -> Arc<Self>
```

Load configuration from environment variables (fail-fast)

This will:
1. Load .env file if present
2. Parse environment variables
3. Apply convention-based patterns (RPC_URL_{CHAIN_ID})
4. Load chains.toml if specified
5. Validate all configuration
6. Store globally for access via Config::global()

---

### generic

**Source**: `src/error.rs`

```rust
pub fn generic<S: Into<String>>(msg: S) -> Self
```

Create a generic error

---

### get

**Source**: `src/environment.rs`

```rust
pub fn get(&self, key: &str) -> Option<String>
```

Get an environment variable

---

### get_ai_key

**Source**: `src/providers.rs`

```rust
pub fn get_ai_key(&self, provider: AiProvider) -> Option<&str>
```

Get the API key for an AI provider

---

### get_blockchain_key

**Source**: `src/providers.rs`

```rust
pub fn get_blockchain_key(&self, provider: BlockchainProvider) -> Option<&str>
```

Get the API key for a blockchain provider

---

### get_chain

**Source**: `src/network.rs`

```rust
pub fn get_chain(&self, chain_id: u64) -> Option<&ChainConfig>
```

Get chain configuration

---

### get_or

**Source**: `src/environment.rs`

```rust
pub fn get_or(&self, key: &str, default: String) -> String
```

Get an optional environment variable with default

---

### get_rpc_url

**Source**: `src/network.rs`

```rust
pub fn get_rpc_url(&self, chain_id: u64) -> Option<String>
```

Get RPC URL for a specific chain ID

---

### get_supported_chains

**Source**: `src/network.rs`

```rust
pub fn get_supported_chains(&self) -> Vec<u64>
```

Get all supported chain IDs

---

### global

**Source**: `src/lib.rs`

```rust
pub fn global() -> Arc<Self>
```

Get the global configuration instance

Panics if configuration hasn't been loaded via from_env()

---

### has_ai_provider

**Source**: `src/providers.rs`

```rust
pub fn has_ai_provider(&self, provider: AiProvider) -> bool
```

Check if a specific AI provider is configured

---

### has_blockchain_provider

**Source**: `src/providers.rs`

```rust
pub fn has_blockchain_provider(&self, provider: BlockchainProvider) -> bool
```

Check if a blockchain provider is configured

---

### has_data_provider

**Source**: `src/providers.rs`

```rust
pub fn has_data_provider(&self, provider: DataProvider) -> bool
```

Check if a data provider is configured

---

### io

**Source**: `src/error.rs`

```rust
pub fn io<S: Into<String>>(msg: S) -> Self
```

Create an IO error

---

### is_custom_enabled

**Source**: `src/features.rs`

```rust
pub fn is_custom_enabled(&self, name: &str) -> bool
```

Check if a custom feature is enabled

---

### is_enabled

**Source**: `src/features.rs`

```rust
pub fn is_enabled(&self, feature: Feature) -> bool
```

Check if a feature is enabled

---

### load_chain_contracts

**Source**: `src/network.rs`

```rust
pub fn load_chain_contracts(&mut self) -> ConfigResult<()>
```

Load chain contracts from chains.toml file

---

### network

**Source**: `src/lib.rs`

```rust
pub fn network(mut self, config: NetworkConfig) -> Self
```

Set network configuration

---

### new

**Source**: `src/lib.rs`

```rust
pub fn new() -> Self
```

Create a new configuration builder with defaults

---

### parse

**Source**: `src/error.rs`

```rust
pub fn parse<S: Into<String>>(msg: S) -> Self
```

Create a parse error

---

### providers

**Source**: `src/lib.rs`

```rust
pub fn providers(mut self, config: ProvidersConfig) -> Self
```

Set providers configuration

---

### require

**Source**: `src/environment.rs`

```rust
pub fn require(&self, key: &str) -> ConfigResult<String>
```

Get a required environment variable

---

### try_global

**Source**: `src/lib.rs`

```rust
pub fn try_global() -> Option<Arc<Self>>
```

Try to get the global configuration instance

---

### validate

**Source**: `src/app.rs`

```rust
pub fn validate(&self) -> ConfigResult<()>
```

Validates the application configuration for correctness

---

### validate

**Source**: `src/app.rs`

```rust
pub fn validate(&self) -> ConfigResult<()>
```

Validates the retry configuration for correctness

---

### validate

**Source**: `src/features.rs`

```rust
pub fn validate(&self) -> ConfigResult<()>
```

Validate the features configuration for consistency and warnings

---

### validate

**Source**: `src/database.rs`

```rust
pub fn validate(&self) -> ConfigResult<()>
```

Validates all database configuration settings

This method validates:
- Redis URL format and connectivity
- Neo4j URL format (if provided)
- ClickHouse URL format (if provided)
- PostgreSQL URL format (if provided)
- Connection pool configuration

# Errors

Returns `ConfigError` if any validation fails

---

### validate

**Source**: `src/database.rs`

```rust
pub fn validate(&self) -> ConfigResult<()>
```

Validates connection pool configuration settings

This method validates:
- Maximum connections is greater than 0
- Minimum connections doesn't exceed maximum connections
- Connection timeout is greater than 0

# Errors

Returns `ConfigError` if any validation fails

---

### validate

**Source**: `src/network.rs`

```rust
pub fn validate(&self) -> ConfigResult<()>
```

Validates the network configuration

Checks that all URLs are properly formatted and contract addresses are valid

---

### validate

**Source**: `src/providers.rs`

```rust
pub fn validate(&self) -> ConfigResult<()>
```

Validate API key formats and configurations

---

### validate

**Source**: `src/lib.rs`

```rust
pub fn validate(&self) -> ConfigResult<()>
```

Validate the entire configuration

---

### validate_api_key

**Source**: `src/validation.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub fn validate_api_key(key: &str, name: &str) -> ConfigResult<()>
```

Validate an API key format

---

### validate_email

**Source**: `src/validation.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub fn validate_email(email: &str) -> ConfigResult<()>
```

Validate an email address

---

### validate_eth_address

**Source**: `src/validation.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub fn validate_eth_address(address: &str) -> ConfigResult<()>
```

Validate an Ethereum address

---

### validate_percentage

**Source**: `src/validation.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub fn validate_percentage(value: f64, name: &str) -> ConfigResult<()>
```

Validate a percentage value (0-100)

---

### validate_port

**Source**: `src/validation.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub fn validate_port(port: u16) -> ConfigResult<()>
```

Validate a port number

---

### validate_positive

**Source**: `src/validation.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub fn validate_positive<T: PartialOrd + Default + std::fmt::Display>( value: T, name: &str, ) -> ConfigResult<()>
```

Validate a positive number

---

### validate_range

**Source**: `src/validation.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub fn validate_range<T: PartialOrd + std::fmt::Display>( value: T, min: T, max: T, name: &str, ) -> ConfigResult<()>
```

Validate a range

---

### validate_solana_address

**Source**: `src/validation.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub fn validate_solana_address(address: &str) -> ConfigResult<()>
```

Validate a Solana address

---

### validate_url

**Source**: `src/validation.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub fn validate_url(url: &str) -> ConfigResult<()>
```

Validate a URL

---

### validation

**Source**: `src/error.rs`

```rust
pub fn validation<S: Into<String>>(msg: S) -> Self
```

Create a validation error

---

## Traits

### Validator

**Source**: `src/validation.rs`

```rust
pub trait Validator { ... }
```

Trait for validatable configuration

**Methods**:

#### `validate`

```rust
fn validate(&self) -> ConfigResult<()>;
```

---

## Structs

### AppConfig

**Source**: `src/app.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct AppConfig { /// Server port #[serde(default = "default_port")]
```

Application configuration

---

### ChainConfig

**Source**: `src/network.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct ChainConfig { /// Chain ID pub id: u64, /// Human-readable chain name pub name: String, /// RPC URL (overrides global RPC_URL_{CHAIN_ID} if set)
```

Chain-specific configuration

---

### ChainContract

**Source**: `src/network.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
```

```rust
pub struct ChainContract { /// Uniswap V3 router address #[serde(default)]
```

Contract addresses for a chain

---

### Config

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct Config { /// Application-level configuration #[serde(flatten)]
```

Main configuration structure that aggregates all subsystems

---

### ConfigBuilder

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[derive(Default)]
```

```rust
pub struct ConfigBuilder { app: AppConfig, database: DatabaseConfig, network: NetworkConfig, providers: ProvidersConfig, features: FeaturesConfig, }
```

Builder for constructing configuration programmatically

---

### DatabaseConfig

**Source**: `src/database.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct DatabaseConfig { /// Redis connection URL pub redis_url: String, /// Neo4j connection URL (optional, for graph memory)
```

Database configuration

---

### FeaturesConfig

**Source**: `src/features.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct FeaturesConfig { /// Enable trading functionality #[serde(default = "default_true")]
```

Feature flags configuration

---

### NetworkConfig

**Source**: `src/network.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct NetworkConfig { /// Solana RPC URL pub solana_rpc_url: String, /// Solana WebSocket URL (optional)
```

Network configuration

---

### NetworkTimeouts

**Source**: `src/network.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct NetworkTimeouts { /// RPC request timeout in seconds #[serde(default = "default_rpc_timeout")]
```

Network timeout configuration

---

### PoolConfig

**Source**: `src/database.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct PoolConfig { /// Maximum number of connections in the pool #[serde(default = "default_max_connections")]
```

Database connection pool configuration

---

### ProvidersConfig

**Source**: `src/providers.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
```

```rust
pub struct ProvidersConfig { // AI Providers /// API key for Anthropic Claude #[serde(default)]
```

External API providers configuration

---

### RetryConfig

**Source**: `src/app.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct RetryConfig { /// Maximum number of retry attempts #[serde(default = "default_max_retries")]
```

Retry configuration

---

### TransactionConfig

**Source**: `src/app.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct TransactionConfig { /// Maximum gas price in gwei #[serde(default = "default_max_gas_price")]
```

Transaction configuration

---


---

*This documentation was automatically generated from the source code.*