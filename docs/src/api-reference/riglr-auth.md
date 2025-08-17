# riglr-auth API Reference

Comprehensive API documentation for the `riglr-auth` crate.

## Table of Contents

### Structs

- [`AuthConfig`](#authconfig)
- [`AuthProvider`](#authprovider)
- [`CacheConfig`](#cacheconfig)
- [`MagicConfig`](#magicconfig)
- [`MagicProvider`](#magicprovider)
- [`NetworkOverride`](#networkoverride)
- [`PrivyClaims`](#privyclaims)
- [`PrivyConfig`](#privyconfig)
- [`PrivyEvmSigner`](#privyevmsigner)
- [`PrivyEvmSigner`](#privyevmsigner)
- [`PrivyEvmTransactionParams`](#privyevmtransactionparams)
- [`PrivyProvider`](#privyprovider)
- [`PrivyRpcRequest`](#privyrpcrequest)
- [`PrivyRpcResponse`](#privyrpcresponse)
- [`PrivySolanaSigner`](#privysolanasigner)
- [`PrivySolanaSigner`](#privysolanasigner)
- [`PrivySolanaTransactionParams`](#privysolanatransactionparams)
- [`PrivyUserData`](#privyuserdata)
- [`PrivyWallet`](#privywallet)
- [`UserInfo`](#userinfo)
- [`Web3AuthConfig`](#web3authconfig)
- [`Web3AuthProvider`](#web3authprovider)

### Traits

- [`AuthenticationProvider`](#authenticationprovider)
- [`CompositeSignerFactoryExt`](#compositesignerfactoryext)
- [`ProviderConfig`](#providerconfig)

### Enums

- [`AuthError`](#autherror)
- [`AuthProviderType`](#authprovidertype)
- [`LinkedAccount`](#linkedaccount)

### Functions

- [`as_str`](#as_str)
- [`auth_type`](#auth_type)
- [`create_privy_provider`](#create_privy_provider)
- [`email`](#email)
- [`evm_wallet`](#evm_wallet)
- [`is_retriable`](#is_retriable)
- [`magic`](#magic)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`privy`](#privy)
- [`solana_wallet`](#solana_wallet)
- [`web3auth`](#web3auth)
- [`with_api_url`](#with_api_url)
- [`with_auth_url`](#with_auth_url)
- [`with_cache`](#with_cache)

## Structs

### AuthConfig

**Source**: `src/config.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct AuthConfig { /// Provider-specific configuration pub provider_config: HashMap<String, String>, /// Cache configuration #[serde(default)]
```

Base configuration for authentication providers

---

### AuthProvider

**Source**: `src/provider.rs`

```rust
pub struct AuthProvider { provider_type: AuthProviderType, inner: Box<dyn SignerFactory>, }
```

Main authentication provider wrapper

---

### CacheConfig

**Source**: `src/config.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct CacheConfig { /// Enable caching of user data #[serde(default = "default_cache_enabled")]
```

Cache configuration for authentication providers

---

### MagicConfig

**Source**: `magic/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MagicConfig { /// Magic publishable API key pub publishable_key: String, /// Magic secret key pub secret_key: String, /// API base URL #[serde(default = "default_api_url")]
```

Magic.link configuration

---

### MagicProvider

**Source**: `magic/mod.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct MagicProvider { config: MagicConfig, client: reqwest::Client, }
```

Magic.link provider implementation

---

### NetworkOverride

**Source**: `src/config.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct NetworkOverride { /// Override RPC URL for this network pub rpc_url: Option<String>, /// Override chain ID pub chain_id: Option<u64>, /// Custom parameters #[serde(default)]
```

Network-specific configuration overrides

---

### PrivyClaims

**Source**: `privy/types.rs`

**Attributes**:
```rust
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
```

```rust
pub struct PrivyClaims { /// Subject (user ID)
```

JWT claims for Privy tokens

---

### PrivyConfig

**Source**: `privy/config.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct PrivyConfig { /// Privy application ID pub app_id: String, /// Privy application secret pub app_secret: String, /// Privy API base URL (defaults to <https://api.privy.io>)
```

Privy authentication provider configuration

---

### PrivyEvmSigner

**Source**: `privy/signer.rs`

**Attributes**:
```rust
#[cfg(feature = "evm")]
#[derive(Debug, Clone)]
```

```rust
pub struct PrivyEvmSigner { client: reqwest::Client, address: String, wallet_id: String, network: EvmNetworkConfig, }
```

Privy EVM signer implementation

---

### PrivyEvmSigner

**Source**: `privy/signer.rs`

**Attributes**:
```rust
#[cfg(not(feature = "evm"))]
```

```rust
pub struct PrivyEvmSigner;
```

---

### PrivyEvmTransactionParams

**Source**: `privy/types.rs`

**Attributes**:
```rust
#[derive(Debug, Serialize)]
```

```rust
pub struct PrivyEvmTransactionParams { /// From address pub from: String, /// To address pub to: String, /// Value in hex #[serde(skip_serializing_if = "Option::is_none")]
```

Privy transaction parameters for EVM

---

### PrivyProvider

**Source**: `privy/provider.rs`

```rust
pub struct PrivyProvider { config: PrivyConfig, client: reqwest::Client, #[cfg(feature = "caching")]
```

Privy authentication provider

---

### PrivyRpcRequest

**Source**: `privy/types.rs`

**Attributes**:
```rust
#[derive(Debug, Serialize)]
```

```rust
pub struct PrivyRpcRequest { /// Wallet address pub address: String, /// Chain type pub chain_type: String, /// RPC method pub method: String, /// CAIP-2 chain identifier pub caip2: String, /// Method parameters pub params: serde_json::Value, }
```

Privy RPC request structure

---

### PrivyRpcResponse

**Source**: `privy/types.rs`

**Attributes**:
```rust
#[derive(Debug, Deserialize)]
```

```rust
pub struct PrivyRpcResponse { /// Response data pub data: serde_json::Value, }
```

Privy RPC response structure

---

### PrivySolanaSigner

**Source**: `privy/signer.rs`

**Attributes**:
```rust
#[cfg(feature = "solana")]
#[derive(Clone)]
```

```rust
pub struct PrivySolanaSigner { client: reqwest::Client, address: String, rpc: Arc<RpcClient>, network: SolanaNetworkConfig, }
```

Privy Solana signer implementation

---

### PrivySolanaSigner

**Source**: `privy/signer.rs`

**Attributes**:
```rust
#[cfg(not(feature = "solana"))]
```

```rust
pub struct PrivySolanaSigner;
```

---

### PrivySolanaTransactionParams

**Source**: `privy/types.rs`

**Attributes**:
```rust
#[derive(Debug, Serialize)]
```

```rust
pub struct PrivySolanaTransactionParams { /// Base64-encoded transaction pub transaction: String, /// Encoding type pub encoding: String, }
```

Privy transaction parameters for Solana

---

### PrivyUserData

**Source**: `privy/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct PrivyUserData { /// User ID pub id: String, /// Linked accounts pub linked_accounts: Vec<LinkedAccount>, /// Whether user is verified #[serde(default)]
```

Privy user data structure

---

### PrivyWallet

**Source**: `privy/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct PrivyWallet { /// Wallet ID #[serde(default)]
```

Privy wallet information

---

### UserInfo

**Source**: `src/provider.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct UserInfo { /// Unique user identifier pub id: String, /// Email address if available pub email: Option<String>, /// Solana wallet address if available pub solana_address: Option<String>, /// EVM wallet address if available pub evm_address: Option<String>, /// Whether the user is verified pub verified: bool, /// Additional metadata pub metadata: std::collections::HashMap<String, serde_json::Value>, }
```

User information returned from authentication providers

---

### Web3AuthConfig

**Source**: `web3auth/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct Web3AuthConfig { /// Web3Auth client ID pub client_id: String, /// Network (mainnet, testnet, cyan, aqua, celeste)
```

Web3Auth configuration

---

### Web3AuthProvider

**Source**: `web3auth/mod.rs`

**Attributes**:
```rust
#[allow(dead_code)]
```

```rust
pub struct Web3AuthProvider { config: Web3AuthConfig, client: reqwest::Client, }
```

Web3Auth provider implementation

---

## Traits

### AuthenticationProvider

**Source**: `src/provider.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait AuthenticationProvider: SignerFactory { ... }
```

Base trait for authentication providers with additional functionality

**Methods**:

#### `validate_token`

```rust
async fn validate_token(&self, token: &str) -> AuthResult<UserInfo>;
```

#### `refresh_token`

```rust
async fn refresh_token(&self, _token: &str) -> AuthResult<String> {
```

#### `revoke_token`

```rust
async fn revoke_token(&self, _token: &str) -> AuthResult<()> {
```

---

### CompositeSignerFactoryExt

**Source**: `src/lib.rs`

```rust
pub trait CompositeSignerFactoryExt { ... }
```

Extension trait for CompositeSignerFactory to easily register auth providers

**Methods**:

#### `register_provider`

```rust
fn register_provider(&mut self, provider: AuthProvider);
```

---

### ProviderConfig

**Source**: `src/config.rs`

```rust
pub trait ProviderConfig: Send + Sync { ... }
```

Common trait for provider-specific configurations

**Methods**:

#### `validate`

```rust
fn validate(&self) -> Result<(), crate::AuthError>;
```

#### `provider_name`

```rust
fn provider_name(&self) -> &'static str;
```

#### `from_env`

```rust
fn from_env() -> Result<Self, crate::AuthError> where Self: Sized;
```

---

## Enums

### AuthError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum AuthError { /// Token validation failed #[error("Token validation failed: {0}")] TokenValidation(String), /// Missing required credentials #[error("Missing required credential: {0}")] MissingCredential(String), /// User not verified or authorized #[error("User not verified: {0}")] NotVerified(String), /// Network or API request failed #[error("API request failed: {0}")] ApiError(String), /// Configuration error #[error("Configuration error: {0}")] ConfigError(String), /// Unsupported operation #[error("Unsupported operation: {0}")] UnsupportedOperation(String), /// No wallet found for user #[error("No wallet found for user: {0}")] NoWallet(String), /// Generic error with source #[error("Authentication error: {0}")] Other(#[from] anyhow::Error), }
```

Main error type for authentication operations

**Variants**:

- `TokenValidation(String)`
- `MissingCredential(String)`
- `NotVerified(String)`
- `ApiError(String)`
- `ConfigError(String)`
- `UnsupportedOperation(String)`
- `NoWallet(String)`
- `Other(#[from] anyhow::Error)`

---

### AuthProviderType

**Source**: `src/provider.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
```

```rust
pub enum AuthProviderType { /// Privy authentication provider Privy, /// Web3Auth authentication provider Web3Auth, /// Magic.link authentication provider Magic, /// Custom authentication provider with a name Custom(String), }
```

Authentication provider types

**Variants**:

- `Privy`
- `Web3Auth`
- `Magic`
- `Custom(String)`

---

### LinkedAccount

**Source**: `privy/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
```

```rust
pub enum LinkedAccount { /// Wallet account Wallet(PrivyWallet), /// Email account Email { address: String, #[serde(default)] verified: bool, }, /// Phone account Phone { number: String, #[serde(default)] verified: bool, }, /// Social account Social { provider: String, username: Option<String>, #[serde(default)] verified: bool, }, /// Other account types #[serde(other)] Other, }
```

Linked account types in Privy

**Variants**:

- `Wallet(PrivyWallet)`
- `Email`
- `address`
- `verified`
- `Phone`
- `number`
- `verified`
- `Social`
- `provider`
- `username`
- `verified`
- `Other`

---

## Functions

### as_str

**Source**: `src/provider.rs`

```rust
pub fn as_str(&self) -> &str
```

Get the string identifier for this provider type

---

### auth_type

**Source**: `src/provider.rs`

```rust
pub fn auth_type(&self) -> String
```

Get the authentication type string

---

### create_privy_provider

**Source**: `privy/provider.rs`

```rust
pub fn create_privy_provider() -> AuthResult<PrivyProvider>
```

Convenience function to create a Privy provider from environment variables

---

### email

**Source**: `privy/types.rs`

```rust
pub fn email(&self) -> Option<String>
```

Get email if available

---

### evm_wallet

**Source**: `privy/types.rs`

```rust
pub fn evm_wallet(&self) -> Option<&PrivyWallet>
```

Get EVM wallet if available

---

### is_retriable

**Source**: `src/error.rs`

```rust
pub fn is_retriable(&self) -> bool
```

Check if error is retriable (network/transient issues)

---

### magic

**Source**: `src/provider.rs`

**Attributes**:
```rust
#[cfg(feature = "magic")]
```

```rust
pub fn magic(config: crate::magic::MagicConfig) -> Self
```

Create a Magic.link authentication provider

---

### new

**Source**: `src/provider.rs`

```rust
pub fn new(provider_type: AuthProviderType, inner: Box<dyn SignerFactory>) -> Self
```

Create a new authentication provider

---

### new

**Source**: `magic/mod.rs`

```rust
pub fn new(publishable_key: String, secret_key: String) -> Self
```

Create a new Magic configuration

---

### new

**Source**: `magic/mod.rs`

```rust
pub fn new(config: MagicConfig) -> Self
```

Create a new Magic provider

---

### new

**Source**: `privy/signer.rs`

```rust
pub fn new(client: reqwest::Client, address: String, network: SolanaNetworkConfig) -> Self
```

---

### new

**Source**: `privy/signer.rs`

```rust
pub fn new( client: reqwest::Client, address: String, wallet_id: String, network: EvmNetworkConfig, ) -> Self
```

---

### new

**Source**: `privy/config.rs`

```rust
pub fn new(app_id: String, app_secret: String) -> Self
```

Create a new Privy configuration

---

### new

**Source**: `privy/provider.rs`

```rust
pub fn new(config: PrivyConfig) -> Self
```

Create a new Privy provider

---

### new

**Source**: `web3auth/mod.rs`

```rust
pub fn new(client_id: String, verifier: String) -> Self
```

Create a new Web3Auth configuration

---

### new

**Source**: `web3auth/mod.rs`

```rust
pub fn new(config: Web3AuthConfig) -> Self
```

Create a new Web3Auth provider

---

### privy

**Source**: `src/provider.rs`

**Attributes**:
```rust
#[cfg(feature = "privy")]
```

```rust
pub fn privy(config: crate::privy::PrivyConfig) -> Self
```

Create a Privy authentication provider

---

### solana_wallet

**Source**: `privy/types.rs`

```rust
pub fn solana_wallet(&self) -> Option<&PrivyWallet>
```

Get Solana wallet if available

---

### web3auth

**Source**: `src/provider.rs`

**Attributes**:
```rust
#[cfg(feature = "web3auth")]
```

```rust
pub fn web3auth(config: crate::web3auth::Web3AuthConfig) -> Self
```

Create a Web3Auth authentication provider

---

### with_api_url

**Source**: `privy/config.rs`

```rust
pub fn with_api_url(mut self, url: String) -> Self
```

Create configuration with custom API URL

---

### with_auth_url

**Source**: `privy/config.rs`

```rust
pub fn with_auth_url(mut self, url: String) -> Self
```

Create configuration with custom auth URL

---

### with_cache

**Source**: `privy/config.rs`

```rust
pub fn with_cache(mut self, enabled: bool) -> Self
```

Enable or disable caching

---


---

*This documentation was automatically generated from the source code.*