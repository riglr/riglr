use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

// Re-export types from riglr-events-core
pub use riglr_events_core::prelude::*;

// Use SolanaEventMetadata as EventMetadata for all Solana events
pub use crate::solana_metadata::SolanaEventMetadata as EventMetadata;

/// Enumeration of supported Solana DeFi protocols
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize,
)]
pub enum ProtocolType {
    /// Orca Whirlpool concentrated liquidity pools
    OrcaWhirlpool,
    /// Meteora Dynamic Liquidity Market Maker
    MeteoraDlmm,
    /// MarginFi lending protocol
    MarginFi,
    /// Bonk decentralized exchange
    Bonk,
    /// PumpSwap automated market maker
    PumpSwap,
    /// Raydium automated market maker
    RaydiumAmm,
    /// Raydium AMM V4 implementation
    RaydiumAmmV4,
    /// Raydium concentrated liquidity market maker
    RaydiumClmm,
    /// Raydium constant product market maker
    RaydiumCpmm,
    /// Jupiter aggregator protocol
    Jupiter,
    /// Other protocol not explicitly supported
    Other(String),
}

impl Default for ProtocolType {
    fn default() -> Self {
        ProtocolType::Other("Unknown".to_string())
    }
}

impl std::fmt::Display for ProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolType::OrcaWhirlpool => write!(f, "Orca Whirlpool"),
            ProtocolType::MeteoraDlmm => write!(f, "Meteora DLMM"),
            ProtocolType::MarginFi => write!(f, "MarginFi"),
            ProtocolType::Bonk => write!(f, "Bonk"),
            ProtocolType::PumpSwap => write!(f, "PumpSwap"),
            ProtocolType::RaydiumAmm => write!(f, "Raydium AMM"),
            ProtocolType::RaydiumAmmV4 => write!(f, "Raydium AMM V4"),
            ProtocolType::RaydiumClmm => write!(f, "Raydium CLMM"),
            ProtocolType::RaydiumCpmm => write!(f, "Raydium CPMM"),
            ProtocolType::Jupiter => write!(f, "Jupiter"),
            ProtocolType::Other(name) => write!(f, "{}", name),
        }
    }
}

/// Enumeration of DeFi event types supported across protocols
#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    BorshSerialize,
    BorshDeserialize,
    Default,
)]
pub enum EventType {
    /// Token swap transaction
    Swap,
    /// Add liquidity to a pool
    AddLiquidity,
    /// Remove liquidity from a pool
    RemoveLiquidity,
    /// Borrow funds from a lending protocol
    Borrow,
    /// Repay borrowed funds
    Repay,
    /// Liquidate an undercollateralized position
    Liquidate,
    /// Transfer tokens between accounts
    Transfer,
    /// Mint new tokens
    Mint,
    /// Burn existing tokens
    Burn,
    /// Create a new liquidity pool
    CreatePool,
    /// Update pool parameters
    UpdatePool,
    /// General transaction event
    Transaction,
    /// Block-level event
    Block,
    /// Smart contract execution event
    ContractEvent,
    /// Price update event
    PriceUpdate,
    /// Order book update
    OrderBook,
    /// Trade execution
    Trade,
    /// Fee structure update
    FeeUpdate,
    /// Bonk buy with exact input amount
    BonkBuyExactIn,
    /// Bonk buy with exact output amount
    BonkBuyExactOut,
    /// Bonk sell with exact input amount
    BonkSellExactIn,
    /// Bonk sell with exact output amount
    BonkSellExactOut,
    /// Bonk pool initialization
    BonkInitialize,
    /// Bonk migration to AMM
    BonkMigrateToAmm,
    /// Bonk migration to constant product swap
    BonkMigrateToCpswap,
    /// PumpSwap buy transaction
    PumpSwapBuy,
    /// PumpSwap sell transaction
    PumpSwapSell,
    /// PumpSwap pool creation
    PumpSwapCreatePool,
    /// PumpSwap deposit
    PumpSwapDeposit,
    /// PumpSwap withdrawal
    PumpSwapWithdraw,
    /// PumpSwap parameter update
    PumpSwapSetParams,
    /// Raydium swap transaction
    RaydiumSwap,
    /// Raydium deposit
    RaydiumDeposit,
    /// Raydium withdrawal
    RaydiumWithdraw,
    /// Raydium AMM V4 swap with base token input
    RaydiumAmmV4SwapBaseIn,
    /// Raydium AMM V4 swap with base token output
    RaydiumAmmV4SwapBaseOut,
    /// Raydium AMM V4 deposit
    RaydiumAmmV4Deposit,
    /// Raydium AMM V4 second initialization
    RaydiumAmmV4Initialize2,
    /// Raydium AMM V4 withdrawal
    RaydiumAmmV4Withdraw,
    /// Raydium AMM V4 profit and loss withdrawal
    RaydiumAmmV4WithdrawPnl,
    /// Raydium CLMM swap
    RaydiumClmmSwap,
    /// Raydium CLMM swap version 2
    RaydiumClmmSwapV2,
    /// Raydium CLMM pool creation
    RaydiumClmmCreatePool,
    /// Raydium CLMM open position version 2
    RaydiumClmmOpenPositionV2,
    /// Raydium CLMM increase liquidity version 2
    RaydiumClmmIncreaseLiquidityV2,
    /// Raydium CLMM decrease liquidity version 2
    RaydiumClmmDecreaseLiquidityV2,
    /// Raydium CLMM close position
    RaydiumClmmClosePosition,
    /// Raydium CLMM open position with Token22 NFT
    RaydiumClmmOpenPositionWithToken22Nft,
    /// Raydium CPMM swap
    RaydiumCpmmSwap,
    /// Raydium CPMM swap with base input
    RaydiumCpmmSwapBaseInput,
    /// Raydium CPMM swap with base output
    RaydiumCpmmSwapBaseOutput,
    /// Raydium CPMM deposit
    RaydiumCpmmDeposit,
    /// Raydium CPMM withdrawal
    RaydiumCpmmWithdraw,
    /// Raydium CPMM pool creation
    RaydiumCpmmCreatePool,
    /// Open a liquidity position
    OpenPosition,
    /// Close a liquidity position
    ClosePosition,
    /// Increase liquidity in position
    IncreaseLiquidity,
    /// Decrease liquidity in position
    DecreaseLiquidity,
    /// General deposit operation
    Deposit,
    /// General withdrawal operation
    Withdraw,
    /// Unknown or unclassified event type
    #[default]
    Unknown,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl EventType {
    /// Convert local EventType to riglr-events-core EventKind
    pub fn to_event_kind(&self) -> riglr_events_core::types::EventKind {
        match self {
            Self::Swap => riglr_events_core::types::EventKind::Swap,
            Self::AddLiquidity | Self::IncreaseLiquidity => {
                riglr_events_core::types::EventKind::Liquidity
            }
            Self::RemoveLiquidity | Self::DecreaseLiquidity => {
                riglr_events_core::types::EventKind::Liquidity
            }
            Self::Transfer | Self::Deposit | Self::Withdraw => {
                riglr_events_core::types::EventKind::Transfer
            }
            Self::Transaction => riglr_events_core::types::EventKind::Transaction,
            Self::Block => riglr_events_core::types::EventKind::Block,
            Self::ContractEvent => riglr_events_core::types::EventKind::Contract,
            Self::PriceUpdate => riglr_events_core::types::EventKind::Price,
            Self::Trade => riglr_events_core::types::EventKind::Swap,
            _ => riglr_events_core::types::EventKind::Custom(self.to_string()),
        }
    }

    /// Convert from riglr-events-core EventKind to local EventType
    pub fn from_event_kind(kind: &riglr_events_core::types::EventKind) -> Self {
        match kind {
            riglr_events_core::types::EventKind::Transaction => Self::Transaction,
            riglr_events_core::types::EventKind::Block => Self::Block,
            riglr_events_core::types::EventKind::Contract => Self::ContractEvent,
            riglr_events_core::types::EventKind::Transfer => Self::Transfer,
            riglr_events_core::types::EventKind::Swap => Self::Swap,
            riglr_events_core::types::EventKind::Liquidity => Self::AddLiquidity,
            riglr_events_core::types::EventKind::Price => Self::PriceUpdate,
            riglr_events_core::types::EventKind::External => Self::ContractEvent,
            riglr_events_core::types::EventKind::Custom(name) => {
                // Try to match custom names back to specific event types
                match name.as_str() {
                    "RaydiumSwap" => Self::RaydiumSwap,
                    "JupiterSwap" => Self::Swap,
                    "PumpSwapBuy" => Self::PumpSwapBuy,
                    "PumpSwapSell" => Self::PumpSwapSell,
                    _ => Self::Unknown,
                }
            }
        }
    }
}

/// Data structure for token transfer events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferData {
    /// Source account public key
    pub source: Pubkey,
    /// Destination account public key
    pub destination: Pubkey,
    /// Token mint public key (None for SOL transfers)
    pub mint: Option<Pubkey>,
    /// Amount transferred in token's smallest unit
    pub amount: u64,
}

/// Data structure for token swap events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapData {
    /// Input token mint public key
    pub input_mint: Pubkey,
    /// Output token mint public key
    pub output_mint: Pubkey,
    /// Amount of input tokens in smallest unit
    pub amount_in: u64,
    /// Amount of output tokens in smallest unit
    pub amount_out: u64,
}

/// Metadata for streaming context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetadata {
    /// Source of the stream (e.g., "geyser", "binance", "evm-ws")
    pub stream_source: String,
    /// When the event was received by the stream
    pub received_at: std::time::SystemTime,
    /// Optional sequence number for ordering
    pub sequence_number: Option<u64>,
    /// Stream-specific metadata
    pub custom_data: Option<serde_json::Value>,
}

/// Helper functions to create core EventMetadata with Solana-specific data
pub mod metadata_helpers {
    use super::*;
    use riglr_events_core::types::ChainData;

    /// Create a core EventMetadata with Solana chain data
    pub fn create_solana_metadata(
        id: String,
        signature: String,
        slot: u64,
        block_time: i64,
        protocol_type: ProtocolType,
        event_type: EventType,
        program_id: Pubkey,
        index: String,
        program_received_time_ms: i64,
    ) -> EventMetadata {
        let kind = event_type.to_event_kind();
        let source = format!("solana-{}", protocol_type);

        let timestamp =
            chrono::DateTime::from_timestamp(block_time, 0).unwrap_or_else(chrono::Utc::now);

        let mut metadata =
            riglr_events_core::EventMetadata::with_timestamp(id, kind, source, timestamp);

        // Add Solana chain data
        let chain_data = ChainData::Solana {
            slot,
            signature: Some(signature.clone()),
            program_id: Some(program_id),
            instruction_index: index.parse().ok(),
            block_time: Some(block_time),
            protocol_data: Some(serde_json::json!({
                "protocol_type": protocol_type,
                "event_type": event_type,
                "program_received_time_ms": program_received_time_ms,
            })),
        };

        metadata = metadata.with_chain_data(chain_data);

        // Create SolanaEventMetadata wrapper
        crate::solana_metadata::SolanaEventMetadata::new(
            signature,
            slot,
            event_type,
            protocol_type,
            index,
            program_received_time_ms,
            metadata,
        )
    }
}
