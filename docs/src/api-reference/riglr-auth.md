# riglr-auth

{{#include ../../../riglr-auth/README.md}}

## API Reference

### Contents

- [Structs](#structs)
- [Enums](#enums)
- [Traits](#traits)
- [Functions](#functions)
- [Type Aliases](#type-aliases)

### Structs

> Core data structures and types.

#### `AuthConfig`

Base configuration for authentication providers

---

#### `AuthProvider`

Main authentication provider wrapper

---

#### `CacheConfig`

Cache configuration for authentication providers

---

#### `MagicConfig`

Magic.link configuration

---

#### `MagicProvider`

Magic.link provider implementation

---

#### `NetworkOverride`

Network-specific configuration overrides

---

#### `PrivyConfig`

Privy authentication provider configuration

---

#### `PrivyProvider`

Privy authentication provider

---

#### `PrivyUserData`

Privy user data structure

---

#### `PrivyWallet`

Privy wallet information

---

#### `UserInfo`

User information returned from authentication providers

---

#### `Web3AuthConfig`

Web3Auth configuration

---

#### `Web3AuthProvider`

Web3Auth provider implementation

---

### Enums

> Enumeration types for representing variants.

#### `AuthError`

Main error type for authentication operations

**Variants:**

- `TokenValidation`
  - Token validation failed
- `MissingCredential`
  - Missing required credentials
- `NotVerified`
  - User not verified or authorized
- `ApiError`
  - Network or API request failed
- `ConfigError`
  - Configuration error
- `UnsupportedOperation`
  - Unsupported operation
- `NoWallet`
  - No wallet found for user
- `Other`
  - Generic error with source

---

#### `AuthProviderType`

Authentication provider types

**Variants:**

- `Privy`
  - Privy authentication provider
- `Web3Auth`
  - Web3Auth authentication provider
- `Magic`
  - Magic.link authentication provider
- `Custom`
  - Custom authentication provider with a name

---

#### `LinkedAccount`

Linked account types in Privy

**Variants:**

- `Wallet`
  - Wallet account
- `Email`
  - Email account
- `Phone`
  - Phone account
- `Social`
  - Social account
- `Other`
  - Other account types

---

### Traits

> Trait definitions for implementing common behaviors.

#### `AuthenticationProvider`

Base trait for authentication providers with additional functionality

**Methods:**

- `validate_token()`
  - Validate a token and return user information
- `refresh_token()`
  - Refresh a token if supported
- `revoke_token()`
  - Revoke a token if supported

---

#### `CompositeSignerFactoryExt`

Extension trait for CompositeSignerFactory to easily register auth providers

**Methods:**

- `register_provider()`
  - Register an authentication provider with the factory

---

#### `ProviderConfig`

Common trait for provider-specific configurations

**Methods:**

- `validate()`
  - Validate the configuration
- `provider_name()`
  - Get provider name
- `from_env()`
  - Create from environment variables

---

### Functions

> Standalone functions and utilities.

#### `create_privy_provider`

Convenience function to create a Privy provider from environment variables

---

### Type Aliases

#### `AuthResult`

Result type alias for authentication operations

**Type:** `<T, >`

---
