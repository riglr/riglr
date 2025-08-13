# riglr-solana-tools

*This page is auto-generated from the source code.*

Comprehensive Solana blockchain integration tools.

## Categories

- [Token Operations](#token-operations)
- [DeFi Protocols](#defi-protocols)
- [Risk Analysis](#risk-analysis)
- [Transaction Management](#transaction-management)

---

## Token Operations

### `get_sol_balance`
**Description:**  
Get SOL balance for a wallet address.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `address` | `String` | Yes | The Solana wallet address to check |

**Returns:** `Result<f64, ToolError>` (Balance in SOL)

**Example:**
```rust
let balance = get_sol_balance(
    "11111111111111111111111111111111".to_string()
).await?;
println!("Balance: {} SOL", balance);
```

---

### `get_token_balance`
**Description:**  
Get SPL token balance for a wallet.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `wallet` | `String` | Yes | Wallet address |
| `mint` | `String` | Yes | Token mint address |

**Returns:** `Result<f64, ToolError>` (Token balance)

---

### `transfer_sol`
**Description:**  
Transfer SOL to another wallet.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `to` | `String` | Yes | Recipient address |
| `amount` | `f64` | Yes | Amount in SOL |

**Returns:** `Result<String, ToolError>` (Transaction signature)

---

## DeFi Protocols

### `jupiter_swap`
**Description:**  
Execute a token swap using Jupiter aggregator.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `input_mint` | `String` | Yes | Input token mint |
| `output_mint` | `String` | Yes | Output token mint |
| `amount` | `f64` | Yes | Input amount |
| `slippage_bps` | `u16` | No | Slippage in basis points (default: 50) |

**Returns:** `Result<SwapResult, ToolError>`

**Example:**
```rust
let result = jupiter_swap(
    "So11111111111111111111111111111111111111112".to_string(), // SOL
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
    1.0,
    Some(100)
).await?;
```

---

### `raydium_add_liquidity`
**Description:**  
Add liquidity to a Raydium pool.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pool_id` | `String` | Yes | Raydium pool address |
| `token_a_amount` | `f64` | Yes | Amount of token A |
| `token_b_amount` | `f64` | Yes | Amount of token B |

**Returns:** `Result<String, ToolError>` (Transaction signature)

---

### `pump_fun_buy`
**Description:**  
Buy tokens on Pump.fun bonding curve.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `mint` | `String` | Yes | Token mint address |
| `sol_amount` | `f64` | Yes | SOL amount to spend |
| `min_tokens` | `Option<f64>` | No | Minimum tokens to receive |

**Returns:** `Result<PumpFunTradeResult, ToolError>`

---

## Risk Analysis

### `analyze_token_risk`
**Description:**  
Analyze token for potential risks using RugCheck.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `mint` | `String` | Yes | Token mint to analyze |

**Returns:** `Result<RiskAnalysis, ToolError>`

**Example:**
```rust
let risk = analyze_token_risk(token_mint).await?;
if risk.score > 80 {
    println!("High risk token! Score: {}", risk.score);
    for issue in risk.issues {
        println!("- {}", issue);
    }
}
```

---

### `get_token_holders`
**Description:**  
Get top holders of a token.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `mint` | `String` | Yes | Token mint address |
| `limit` | `Option<usize>` | No | Number of holders to return (default: 10) |

**Returns:** `Result<Vec<Holder>, ToolError>`

---

## Transaction Management

### `send_transaction`
**Description:**  
Send a signed transaction to the network.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `transaction` | `Vec<u8>` | Yes | Serialized transaction |
| `skip_preflight` | `Option<bool>` | No | Skip preflight checks |

**Returns:** `Result<String, ToolError>` (Transaction signature)

---

### `get_transaction_status`
**Description:**  
Check the status of a transaction.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `signature` | `String` | Yes | Transaction signature |

**Returns:** `Result<TransactionStatus, ToolError>`

---

### `simulate_transaction`
**Description:**  
Simulate a transaction without sending it.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `instructions` | `Vec<Instruction>` | Yes | Instructions to simulate |

**Returns:** `Result<SimulationResult, ToolError>`

---

## Type Definitions

### `SwapResult`
```rust
struct SwapResult {
    pub transaction_signature: String,
    pub input_amount: f64,
    pub output_amount: f64,
    pub price_impact: f64,
}
```

### `RiskAnalysis`
```rust
struct RiskAnalysis {
    pub score: u8,  // 0-100, higher is riskier
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub metadata: TokenMetadata,
}
```

### `Holder`
```rust
struct Holder {
    pub address: String,
    pub balance: f64,
    pub percentage: f64,
}
```

## Error Patterns

Common error scenarios and their classifications:

- **Insufficient balance**: Permanent error
- **Network timeout**: Retriable error
- **Invalid address**: Permanent error
- **RPC rate limit**: Retriable error
- **Slippage exceeded**: Permanent error (user must adjust)

## See Also

- [Solana Arbitrage Bot Tutorial](../tutorials/solana-arbitrage-bot.md)
- [Event Parsing System](../concepts/event-parsing.md)
- [Error Handling Philosophy](../concepts/error-handling.md)