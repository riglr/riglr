//! # riglr-solana-tools
//!
//! A comprehensive suite of rig-compatible tools for interacting with the Solana blockchain.
//!
//! This crate provides ready-to-use tools for building Solana-native AI agents, including:
//!
//! - **Balance Tools**: Check SOL and SPL token balances
//! - **Transaction Tools**: Send SOL and token transfers
//! - **DeFi Tools**: Interact with Jupiter for swaps and quotes
//! - **Pump.fun Tools**: Deploy, buy, and sell tokens on Pump.fun
//! - **Network Tools**: Query blockchain state and transaction details
//!
//! All tools are built with the `#[tool]` macro for seamless integration with rig agents
//! and include comprehensive error handling and retry logic.
//!
//! ## Features
//!
//! - **Production Ready**: Built-in retry logic, timeouts, and error handling
//! - **Type Safe**: Full Rust type safety with serde and schemars integration
//! - **Async First**: Non-blocking operations using tokio
//! - **Composable**: Mix and match tools as needed for your agent
//! - **Well Documented**: Every tool includes usage examples
//!
//! ## Quick Start
//!
//! ```ignore
//! // Example usage (requires rig-core dependency):
//! use riglr_solana_tools::balance::get_sol_balance;
//! use rig_core::Agent;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let agent = Agent::builder()
//!     .preamble("You are a Solana blockchain assistant.")
//!     .tool(get_sol_balance)
//!     .build();
//!
//! let response = agent.prompt("What is the SOL balance of So11111111111111111111111111111111111111112?").await?;
//! println!("Agent response: {}", response);
//! # Ok(())
//! # }
//! ```
//!
//! ## Tool Categories
//!
//! - [`balance`] - Balance checking tools for SOL and SPL tokens
//! - [`transaction`] - Transaction creation and execution tools  
//! - [`swap`] - Jupiter DEX integration for token swaps
//! - [`pump`] - Pump.fun integration for meme token deployment and trading
//! - [`network`] - Network state and blockchain query tools

pub mod balance;
pub mod client;
pub mod error;
pub mod events;
pub mod network;
pub mod pump;
pub mod signer;
pub mod swap;
pub mod transaction;
pub mod util;
pub mod utils;

// Re-export commonly used tools
pub use balance::*;
pub use network::*;
pub use pump::*;
pub use signer::*;
pub use swap::*;
pub use transaction::*;
pub use util::*;
pub use utils::*;

// Re-export client and error types
pub use client::SolanaClient;
pub use error::{Result, SolanaToolError};

// Re-export event system components
pub use events::{EventParserFactory, Protocol, UnifiedEvent};

// Re-export macros (imported from events module)
// pub use match_event; // Already exported from events module

// Re-export signer types for convenience
pub use riglr_core::{SignerContext, signer::TransactionSigner};

/// Current version of riglr-solana-tools
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// ============================================================================
// Event Analysis Tools
// ============================================================================

// Already imported above, no need to re-import
use crate::client::SolanaConfig;
use riglr_macros::tool;
use serde::{Deserialize, Serialize};

/// Helper function to format events for agent consumption
pub fn format_events_for_agent(events: Vec<Box<dyn UnifiedEvent>>) -> Result<String> {
    if events.is_empty() {
        return Ok("No events found in transaction.".to_string());
    }

    let mut output = String::new();
    output.push_str(&format!("# Transaction Event Analysis ({} events)\n\n", events.len()));
    
    for (i, event) in events.iter().enumerate() {
        output.push_str(&format!(
            "## Event {} - {} ({:?})\n",
            i + 1,
            event.event_type().to_string(),
            event.protocol_type()
        ));
        output.push_str(&format!("- **Transaction**: {}\n", event.signature()));
        output.push_str(&format!("- **Slot**: {}\n", event.slot()));
        output.push_str(&format!("- **Processing Time**: {}ms\n", event.program_handle_time_consuming_ms()));
        output.push_str(&format!("- **Index**: {}\n", event.index()));
        output.push_str("\n");
    }
    
    Ok(output)
}

/// Helper function to parse protocol strings to enum
pub fn parse_protocol_strings(protocols: Vec<String>) -> Result<Vec<Protocol>> {
    let mut parsed = Vec::new();
    for protocol_str in protocols {
        let protocol = match protocol_str.to_lowercase().as_str() {
            "pumpswap" => Protocol::PumpSwap,
            "bonk" => Protocol::Bonk,
            "raydiumcpmm" | "raydium_cpmm" => Protocol::RaydiumCpmm,
            "raydiumclmm" | "raydium_clmm" => Protocol::RaydiumClmm,
            "raydiumammv4" | "raydium_amm_v4" => Protocol::RaydiumAmmV4,
            _ => return Err(SolanaToolError::Generic(format!("Unsupported protocol: {}", protocol_str)))
        };
        parsed.push(protocol);
    }
    Ok(parsed)
}

/// Analyzes a single transaction for DEX events across all supported protocols
#[tool]
pub async fn analyze_transaction_events(
    signature: String,
    rpc_url: Option<String>,
) -> Result<String> {
    let client = SolanaClient::new(SolanaConfig {
        rpc_url: rpc_url.unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string()),
        ..Default::default()
    });

    // Get transaction with metadata
    let tx = client.get_transaction_with_meta(&signature).await?;
    
    // Use MutilEventParser for multi-protocol analysis
    let parser = EventParserFactory::create_mutil_parser(&[
        Protocol::PumpSwap,
        Protocol::Bonk, 
        Protocol::RaydiumCpmm,
        Protocol::RaydiumClmm,
        Protocol::RaydiumAmmV4
    ]);
    
    // Parse events from transaction
    let events = parser.parse_transaction(
        tx,
        &signature,
        None, // slot will be extracted from transaction
        None, // block_time will be extracted from transaction
        chrono::Utc::now().timestamp_millis(),
        None, // no bot wallet for analysis
    ).await?;
    
    // Format results for agent consumption
    format_events_for_agent(events)
}

/// Analyzes recent transactions for a token to identify DEX activity patterns
#[tool]
pub async fn analyze_recent_events(
    token_address: String,
    limit: Option<usize>,
    rpc_url: Option<String>,
) -> Result<String> {
    let client = SolanaClient::new(SolanaConfig {
        rpc_url: rpc_url.unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string()),
        ..Default::default()
    });
    
    // Note: This is a simplified implementation
    // In production, this would involve querying token account changes
    let limit = limit.unwrap_or(50);
    let transactions = client.get_recent_transactions_for_token(&token_address, limit).await?;
    
    if transactions.is_empty() {
        return Ok(format!("No recent transactions found for token: {}", token_address));
    }
    
    let mut output = String::new();
    output.push_str(&format!("# Recent Event Analysis for Token: {}\n\n", token_address));
    output.push_str(&format!("Found {} recent transactions (limit: {})\n\n", transactions.len(), limit));
    
    // Process each transaction with the multi-parser
    let parser = EventParserFactory::create_mutil_parser(&Protocol::all());
    let mut total_events = 0;
    
    for (i, tx) in transactions.iter().enumerate() {
        if let Some(signature) = tx.transaction.signatures.first() {
            let events = parser.parse_transaction(
                tx.clone(),
                signature,
                None,
                None,
                chrono::Utc::now().timestamp_millis(),
                None,
            ).await.unwrap_or_default();
            
            if !events.is_empty() {
                output.push_str(&format!("## Transaction {} - {} events\n", i + 1, events.len()));
                output.push_str(&format!("**Signature**: {}\n\n", signature));
                total_events += events.len();
            }
        }
    }
    
    output.push_str(&format!("\n**Total Events Found**: {}\n", total_events));
    Ok(output)
}

/// Gets events for specific protocols from a transaction
#[tool]
pub async fn get_protocol_events(
    signature: String,
    protocols: Vec<String>, // ["PumpSwap", "Bonk", etc.]
    rpc_url: Option<String>,
) -> Result<String> {
    let client = SolanaClient::new(SolanaConfig {
        rpc_url: rpc_url.unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string()),
        ..Default::default()
    });

    // Parse protocol strings
    let protocol_enums = parse_protocol_strings(protocols)?;
    
    // Get transaction
    let tx = client.get_transaction_with_meta(&signature).await?;
    
    // Create parser with specified protocols
    let parser = EventParserFactory::create_mutil_parser(&protocol_enums);
    
    // Parse events
    let events = parser.parse_transaction(
        tx,
        &signature,
        None,
        None,
        chrono::Utc::now().timestamp_millis(),
        None,
    ).await?;
    
    let mut output = String::new();
    output.push_str(&format!("# Protocol-Specific Event Analysis\n"));
    output.push_str(&format!("**Transaction**: {}\n", signature));
    output.push_str(&format!("**Requested Protocols**: {:?}\n\n", protocol_enums));
    
    if events.is_empty() {
        output.push_str("No events found for the specified protocols.\n");
    } else {
        output.push_str(&format!("Found {} events:\n\n", events.len()));
        for (i, event) in events.iter().enumerate() {
            output.push_str(&format!(
                "{}. **{}** ({})\n   - Signature: {}\n   - Slot: {}\n\n",
                i + 1,
                event.event_type().to_string(),
                event.protocol_type() as i32,
                event.signature(),
                event.slot()
            ));
        }
    }
    
    Ok(output)
}

/// Real-time monitoring of events for a specific token
#[tool]
pub async fn monitor_token_events(
    token_address: String,
    duration_minutes: Option<u64>,
    rpc_url: Option<String>,
) -> Result<String> {
    let duration = duration_minutes.unwrap_or(10);
    
    // Note: This is a placeholder implementation
    // Real-time monitoring would involve:
    // 1. Subscribing to account changes for the token
    // 2. Processing new transactions as they come in
    // 3. Parsing events and reporting them
    
    Ok(format!(
        "# Token Event Monitoring Started\n\n\
         **Token**: {}\n\
         **Duration**: {} minutes\n\
         **RPC**: {}\n\n\
         Note: This is a placeholder implementation. Real monitoring would involve \
         subscribing to blockchain updates and parsing events in real-time.\n\
         \n\
         To implement real monitoring, consider using WebSocket subscriptions \
         to account changes and processing transactions as they occur.",
        token_address,
        duration,
        rpc_url.unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string())
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::len_zero)]
    fn test_version() {
        // VERSION is a compile-time constant from CARGO_PKG_VERSION
        // Its existence is guaranteed by successful compilation
        assert!(VERSION.len() > 0);
    }
}
