# riglr-solana-common API Reference

Comprehensive API documentation for the `riglr-solana-common` crate.

## Table of Contents

### Enums

- [`SolanaCommonError`](#solanacommonerror)

### Structs

- [`SolanaAccount`](#solanaaccount)
- [`SolanaConfig`](#solanaconfig)
- [`SolanaTransactionData`](#solanatransactiondata)

### Functions

- [`account_count`](#account_count)
- [`create_shared_solana_client`](#create_shared_solana_client)
- [`create_solana_client`](#create_solana_client)
- [`decode_instructions`](#decode_instructions)
- [`default_solana_config`](#default_solana_config)
- [`fee_payer_pubkey`](#fee_payer_pubkey)
- [`format_balance`](#format_balance)
- [`format_solana_address`](#format_solana_address)
- [`lamports_to_sol`](#lamports_to_sol)
- [`new`](#new)
- [`new`](#new)
- [`parse_commitment`](#parse_commitment)
- [`signers`](#signers)
- [`sol_to_lamports`](#sol_to_lamports)
- [`string_to_pubkey`](#string_to_pubkey)
- [`to_pubkey`](#to_pubkey)
- [`validate_rpc_url`](#validate_rpc_url)
- [`validate_solana_address`](#validate_solana_address)
- [`with_compute_units`](#with_compute_units)
- [`writable_accounts`](#writable_accounts)

## Enums

### SolanaCommonError

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, thiserror::Error)]
```

```rust
pub enum SolanaCommonError { #[error("Invalid public key: {0}")] InvalidPubkey(String), #[error("Client error: {0}")] ClientError(String), #[error("Parse error: {0}")] ParseError(String), }
```

Error types for Solana operations shared across crates

**Variants**:

- `InvalidPubkey(String)`
- `ClientError(String)`
- `ParseError(String)`

---

## Structs

### SolanaAccount

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct SolanaAccount { pub pubkey: String, pub is_signer: bool, pub is_writable: bool, }
```

Common Solana account metadata

---

### SolanaConfig

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct SolanaConfig { pub rpc_url: String, pub commitment: String, pub timeout_seconds: u64, }
```

Shared configuration for Solana operations

---

### SolanaTransactionData

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct SolanaTransactionData { /// Recent blockhash for the transaction pub recent_blockhash: String, /// Fee payer account (base58 encoded)
```

Solana transaction metadata shared between crates

---

## Functions

### account_count

**Source**: `src/types.rs`

```rust
pub fn account_count(&self) -> usize
```

Get total number of accounts

---

### create_shared_solana_client

**Source**: `src/utils.rs`

```rust
pub fn create_shared_solana_client(config: &SolanaConfig) -> Arc<RpcClient>
```

Create a shared Solana RPC client

---

### create_solana_client

**Source**: `src/utils.rs`

```rust
pub fn create_solana_client(config: &SolanaConfig) -> RpcClient
```

Create a Solana RPC client with the given configuration

---

### decode_instructions

**Source**: `src/types.rs`

```rust
pub fn decode_instructions(&self) -> Result<Vec<u8>, SolanaCommonError>
```

Decode instructions data from base64

---

### default_solana_config

**Source**: `src/utils.rs`

```rust
pub fn default_solana_config() -> SolanaConfig
```

Get the default Solana configuration from environment or defaults

---

### fee_payer_pubkey

**Source**: `src/types.rs`

```rust
pub fn fee_payer_pubkey(&self) -> Result<Pubkey, SolanaCommonError>
```

Get fee payer as Pubkey

---

### format_balance

**Source**: `src/utils.rs`

```rust
pub fn format_balance(lamports: u64) -> String
```

Format a balance for display with appropriate units

---

### format_solana_address

**Source**: `src/types.rs`

```rust
pub fn format_solana_address(pubkey: &Pubkey) -> String
```

Format a Solana address for display

---

### lamports_to_sol

**Source**: `src/utils.rs`

```rust
pub fn lamports_to_sol(lamports: u64) -> f64
```

Convert lamports to SOL for display

---

### new

**Source**: `src/types.rs`

```rust
pub fn new( pubkey: &str, is_signer: bool, is_writable: bool, ) -> Result<Self, SolanaCommonError>
```

---

### new

**Source**: `src/types.rs`

```rust
pub fn new( recent_blockhash: String, fee_payer: String, instructions_data: String, accounts: Vec<SolanaAccount>, ) -> Result<Self, SolanaCommonError>
```

Create new Solana transaction data

---

### parse_commitment

**Source**: `src/types.rs`

```rust
pub fn parse_commitment(commitment: &str) -> solana_sdk::commitment_config::CommitmentLevel
```

Parse a commitment level string

---

### signers

**Source**: `src/types.rs`

```rust
pub fn signers(&self) -> Vec<&SolanaAccount>
```

Get signer accounts

---

### sol_to_lamports

**Source**: `src/utils.rs`

```rust
pub fn sol_to_lamports(sol: f64) -> u64
```

Convert SOL to lamports

---

### string_to_pubkey

**Source**: `src/utils.rs`

```rust
pub fn string_to_pubkey(s: &str) -> Result<Pubkey, SolanaCommonError>
```

Convert a string to a Solana Pubkey with better error handling

---

### to_pubkey

**Source**: `src/types.rs`

```rust
pub fn to_pubkey(&self) -> Result<Pubkey, SolanaCommonError>
```

---

### validate_rpc_url

**Source**: `src/utils.rs`

```rust
pub async fn validate_rpc_url(url: &str) -> Result<(), SolanaCommonError>
```

Validate that an RPC URL is reachable

---

### validate_solana_address

**Source**: `src/types.rs`

```rust
pub fn validate_solana_address(address: &str) -> Result<Pubkey, SolanaCommonError>
```

Helper function to validate Solana addresses

---

### with_compute_units

**Source**: `src/types.rs`

```rust
pub fn with_compute_units(mut self, limit: u32, price: u64) -> Self
```

Add compute unit configuration

---

### writable_accounts

**Source**: `src/types.rs`

```rust
pub fn writable_accounts(&self) -> Vec<&SolanaAccount>
```

Get writable accounts

---


---

*This documentation was automatically generated from the source code.*