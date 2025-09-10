# riglr-config

{{#include ../../../riglr-config/README.md}}

## API Reference

## Key Components

> The most important types and functions in this crate.

### `Config`

Main configuration structure that aggregates all subsystems

[→ Full documentation](#structs)

### `ConfigBuilder`

Builder for creating Config instances programmatically

[→ Full documentation](#structs)

---

### Contents

- [Structs](#structs)
- [Enums](#enums)
- [Traits](#traits)
- [Type Aliases](#type-aliases)

### Structs

> Core data structures and types.

#### `AppConfig`

Application configuration

---

#### `ChainConfig`

Chain-specific configuration

---

#### `ChainContract`

Contract addresses for a chain

---

#### `DatabaseConfig`

Database configuration

---

#### `EvmNetworkConfig`

EVM-specific network configuration

---

#### `FeaturesConfig`

Feature flags configuration

---

#### `NetworkConfig`

Network configuration

---

#### `NetworkTimeouts`

Network timeout configuration

---

#### `PoolConfig`

Database connection pool configuration

---

#### `ProvidersConfig`

External API providers configuration

---

#### `RetryConfig`

Retry configuration

---

#### `SolanaNetworkConfig`

Solana-specific network configuration

---

#### `TransactionConfig`

Transaction configuration

---

### Enums

> Enumeration types for representing variants.

#### `AiProvider`

AI provider enumeration

**Variants:**

- `Anthropic`
  - Anthropic Claude AI provider
- `OpenAI`
  - OpenAI provider
- `Groq`
  - Groq provider
- `Perplexity`
  - Perplexity AI provider

---

#### `BlockchainProvider`

Blockchain data provider enumeration

**Variants:**

- `Alchemy`
  - Alchemy blockchain data provider
- `Infura`
  - Infura blockchain infrastructure provider
- `QuickNode`
  - QuickNode blockchain infrastructure provider
- `Moralis`
  - Moralis Web3 development platform

---

#### `ConfigError`

Configuration errors

**Variants:**

- `MissingEnvVar`
  - Environment variable not found
- `EnvParse`
  - Failed to parse environment variables
- `ValidationError`
  - Invalid configuration value
- `ParseError`
  - Failed to parse configuration
- `IoError`
  - IO error
- `ChainNotSupported`
  - Chain not supported
- `ProviderNotConfigured`
  - Provider not configured
- `ConfigLocked`
  - Configuration already locked
- `Generic`
  - Generic error

---

#### `DataProvider`

Data provider enumeration

**Variants:**

- `DexScreener`
  - DexScreener DEX analytics provider
- `CoinGecko`
  - CoinGecko cryptocurrency data provider
- `CoinMarketCap`
  - CoinMarketCap cryptocurrency market data provider
- `Twitter`
  - Twitter social media data provider
- `LunarCrush`
  - LunarCrush social analytics provider

---

#### `Environment`

Application environment

**Variants:**

- `Development`
  - Development environment for local testing and debugging
- `Staging`
  - Staging environment for pre-production testing
- `Production`
  - Production environment for live deployment

---

#### `EnvironmentSource`

Source of environment variables (for testing and custom providers)

**Variants:**

- `System`
  - Use system environment variables
- `Custom`
  - Use custom environment provider (for testing)

---

#### `Feature`

Feature enumeration

**Variants:**

- `Trading`
  - Enable trading functionality
- `Bridging`
  - Enable cross-chain bridging
- `SocialMonitoring`
  - Enable social media monitoring
- `GraphMemory`
  - Enable graph-based memory
- `Streaming`
  - Enable real-time streaming
- `Webhooks`
  - Enable webhook notifications
- `Analytics`
  - Enable analytics collection
- `Debug`
  - Enable debug mode
- `Experimental`
  - Enable experimental features

---

#### `LogLevel`

Log level configuration

**Variants:**

- `Trace`
  - Trace level - most verbose
- `Debug`
  - Debug level - detailed debugging
- `Info`
  - Info level - general information
- `Warn`
  - Warning level - warnings
- `Error`
  - Error level - only errors

---

### Traits

> Trait definitions for implementing common behaviors.

#### `AddressValidator`

Trait for validating blockchain addresses

This trait allows different blockchain address validation logic to be plugged into
the configuration system without creating tight coupling to specific blockchain crates.

**Methods:**

- `validate()`
  - Validate an address string

---

### Type Aliases

#### `ConfigResult`

Configuration result type

**Type:** `<T, >`

---
