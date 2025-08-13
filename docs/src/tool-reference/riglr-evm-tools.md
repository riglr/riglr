# riglr-evm-tools Tool Reference

This page contains documentation for tools provided by the `riglr-evm-tools` crate.

## Available Tools

- [`get_eth_balance`](#get_eth_balance) - src/balance.rs
- [`get_erc20_balance`](#get_erc20_balance) - src/balance.rs
- [`call_contract_read`](#call_contract_read) - src/contract.rs
- [`call_contract_write`](#call_contract_write) - src/contract.rs
- [`read_erc20_info`](#read_erc20_info) - src/contract.rs
- [`transfer_eth`](#transfer_eth) - src/transaction.rs
- [`transfer_erc20`](#transfer_erc20) - src/transaction.rs
- [`get_transaction_receipt`](#get_transaction_receipt) - src/transaction.rs
- [`get_uniswap_quote`](#get_uniswap_quote) - src/swap.rs
- [`perform_uniswap_swap`](#perform_uniswap_swap) - src/swap.rs

## Tool Functions

### get_eth_balance

**Source**: `src/balance.rs`

```rust
pub async fn get_eth_balance( address: String, block_number: Option<u64>, ) -> std::result::Result<BalanceResult, Box<dyn std::error::Error + Send + Sync>>
```

*No documentation available for this tool.*

---

### get_erc20_balance

**Source**: `src/balance.rs`

```rust
pub async fn get_erc20_balance( address: String, token_address: String, fetch_metadata: Option<bool>, ) -> std::result::Result<TokenBalanceResult, Box<dyn std::error::Error + Send + Sync>>
```

*No documentation available for this tool.*

---

### call_contract_read

**Source**: `src/contract.rs`

```rust
pub async fn call_contract_read( contract_address: String, function_selector: String, params: Vec<String>, ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>
```

*No documentation available for this tool.*

---

### call_contract_write

**Source**: `src/contract.rs`

```rust
pub async fn call_contract_write( contract_address: String, function_selector: String, params: Vec<String>, gas_limit: Option<u64>, ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>
```

*No documentation available for this tool.*

---

### read_erc20_info

**Source**: `src/contract.rs`

```rust
pub async fn read_erc20_info( token_address: String, ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>
```

*No documentation available for this tool.*

---

### transfer_eth

**Source**: `src/transaction.rs`

```rust
pub async fn transfer_eth( to_address: String, amount_eth: f64, gas_price_gwei: Option<u64>, nonce: Option<u64>, ) -> std::result::Result<TransactionResult, Box<dyn std::error::Error + Send + Sync>>
```

*No documentation available for this tool.*

---

### transfer_erc20

**Source**: `src/transaction.rs`

```rust
pub async fn transfer_erc20( token_address: String, to_address: String, amount: String, decimals: u8, gas_price_gwei: Option<u64>, ) -> std::result::Result<TransactionResult, Box<dyn std::error::Error + Send + Sync>>
```

*No documentation available for this tool.*

---

### get_transaction_receipt

**Source**: `src/transaction.rs`

```rust
pub async fn get_transaction_receipt( tx_hash: String, ) -> std::result::Result<TransactionResult, Box<dyn std::error::Error + Send + Sync>>
```

*No documentation available for this tool.*

---

### get_uniswap_quote

**Source**: `src/swap.rs`

```rust
pub async fn get_uniswap_quote( token_in: String, token_out: String, amount_in: String, decimals_in: u8, decimals_out: u8, fee_tier: Option<u32>, slippage_bps: Option<u16>, ) -> std::result::Result<UniswapQuote, Box<dyn std::error::Error + Send + Sync>>
```

*No documentation available for this tool.*

---

### perform_uniswap_swap

**Source**: `src/swap.rs`

```rust
pub async fn perform_uniswap_swap( token_in: String, token_out: String, amount_in: String, decimals_in: u8, amount_out_minimum: String, fee_tier: Option<u32>, deadline_seconds: Option<u64>, ) -> std::result::Result<UniswapSwapResult, Box<dyn std::error::Error + Send + Sync>>
```

*No documentation available for this tool.*

---


---

*This documentation was automatically generated from the source code.*