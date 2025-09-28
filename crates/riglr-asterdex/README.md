# riglr-asterdex

A comprehensive suite of rig-compatible tools for interacting with the Asterdex Futures v3 API, enabling AI agents to trade perpetual futures on Asterdex.

## Features

### 🚀 Trading Tools
- **Order Management**: Place market and limit orders with full control
- **Order Cancellation**: Cancel existing orders by ID
- **Order Query**: Get detailed information about specific orders
- **Leverage Control**: Adjust leverage for trading pairs (1x-125x)

### 📊 Position & Account Management
- **Position Tracking**: Monitor open positions with P&L calculations
- **Account Information**: Query account status, balances, and permissions
- **Balance Management**: Track asset balances and available funds
- **Open Orders**: List and manage all active orders

### 🔐 Authentication
- **Web3 v3 Signature**: Implements Asterdex's unique Web3-based authentication
- **Secure Signing**: ECDSA signature with Keccak-256 hashing
- **Automatic Nonce Generation**: Microsecond-precision timestamps
- **ABI Encoding**: Proper parameter encoding for signature verification

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
riglr-asterdex = { workspace = true }
```

## Quick Start

```rust
use riglr_asterdex::{
    place_asterdex_order,
    get_asterdex_positions,
    get_asterdex_account_info,
    OrderParams,
};
use riglr_core::SignerContext;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up your signer (requires valid API wallet private key)
    let signer = create_your_signer();

    SignerContext::with_signer(signer, async {
        // Place a limit order
        let order_params = OrderParams {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: "0.001".to_string(),
            order_type: "LIMIT".to_string(),
            price: Some("50000.0".to_string()),
            position_side: Some("BOTH".to_string()),
            reduce_only: Some(false),
            time_in_force: Some("GTC".to_string()),
        };

        let result = place_asterdex_order(&context, order_params).await?;
        println!("Order placed: {:?}", result);

        // Check positions
        let positions = get_asterdex_positions(&context, None).await?;
        for position in positions {
            println!("Position: {} {} @ {}",
                position.position_amt,
                position.symbol,
                position.entry_price
            );
        }

        // Get account info
        let account = get_asterdex_account_info(&context).await?;
        println!("Account balance: {}", account.total_wallet_balance);

        Ok(())
    }).await?;

    Ok(())
}
```

## API Documentation

### Trading Functions

#### `place_asterdex_order`
Places a new order on Asterdex.

**Parameters:**
- `symbol`: Trading pair (e.g., "BTCUSDT", "ETHUSDT")
- `side`: "BUY" or "SELL"
- `quantity`: Order size as string
- `order_type`: "MARKET" or "LIMIT"
- `price`: Required for limit orders
- `position_side`: "LONG", "SHORT", or "BOTH"
- `reduce_only`: Whether order only reduces position
- `time_in_force`: "GTC", "IOC", "FOK", or "GTX"

#### `cancel_asterdex_order`
Cancels an existing order.

**Parameters:**
- `symbol`: Trading pair
- `order_id`: Order ID to cancel
- `orig_client_order_id`: Alternative to order_id

#### `get_asterdex_order`
Retrieves information about a specific order.

**Parameters:**
- `symbol`: Trading pair
- `order_id`: Order ID to query

#### `set_asterdex_leverage`
Adjusts leverage for a trading pair.

**Parameters:**
- `symbol`: Trading pair
- `leverage`: Leverage value (1-125)

### Position & Account Functions

#### `get_asterdex_positions`
Returns all open positions or positions for a specific symbol.

**Parameters:**
- `symbol`: Optional trading pair filter

#### `get_asterdex_account_info`
Returns comprehensive account information including balances and permissions.

#### `get_asterdex_balance`
Returns all non-zero asset balances.

#### `get_asterdex_open_orders`
Returns all open orders, optionally filtered by symbol.

**Parameters:**
- `symbol`: Optional trading pair filter

## Authentication Details

This crate implements Asterdex's v3 authentication signature mechanism:

1. **Parameter Preparation**: All business parameters are converted to strings and sorted by key
2. **JSON Stringification**: Parameters are serialized to compact JSON
3. **ABI Encoding**: The JSON string, user address, signer address, and nonce are ABI-encoded
4. **Hashing**: Keccak-256 hash is computed from the encoded data
5. **ECDSA Signing**: The hash is signed using the signer's private key
6. **Request Assembly**: The signature, user, signer, and nonce are included in the final request

## Error Handling

The crate provides comprehensive error handling with automatic retry logic:

- **Permanent Errors**: Invalid inputs, authentication failures, insufficient balance
- **Retriable Errors**: Network issues, temporary API errors
- **Rate Limiting**: Automatic detection and appropriate error categorization

## Examples

See the `examples/` directory for complete working examples:

- `perpetual_trading.rs`: Demonstrates all major features including order placement, position management, and account queries

## Safety and Security

- All API calls use secure HTTPS connections
- Private keys are never logged or exposed
- Automatic timeout handling (30 seconds default)
- Comprehensive input validation

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test`
2. Code follows Rust conventions: `cargo clippy`
3. Documentation is updated for new features
4. Examples demonstrate new functionality

## License

This project is licensed under the same terms as the riglr workspace.

## Support

For issues, questions, or contributions, please visit the [riglr repository](https://github.com/your-org/riglr).

## Disclaimer

This software is provided "as is" without warranty of any kind. Trading cryptocurrencies and derivatives involves substantial risk. Always test thoroughly with small amounts before using in production.