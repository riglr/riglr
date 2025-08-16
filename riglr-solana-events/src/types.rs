use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

// Re-export types from riglr-events-core
pub use riglr_events_core::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, BorshDeserialize)]
pub enum ProtocolType {
    OrcaWhirlpool,
    MeteoraDlmm,
    MarginFi,
    Bonk,
    PumpSwap,
    RaydiumAmm,
    RaydiumAmmV4,
    RaydiumClmm,
    RaydiumCpmm,
    Jupiter,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, BorshDeserialize, Default)]
pub enum EventType {
    Swap,
    AddLiquidity,
    RemoveLiquidity,
    Borrow,
    Repay,
    Liquidate,
    Transfer,
    Mint,
    Burn,
    CreatePool,
    UpdatePool,
    Transaction,
    Block,
    ContractEvent,
    PriceUpdate,
    OrderBook,
    Trade,
    FeeUpdate,
    BonkBuyExactIn,
    BonkBuyExactOut,
    BonkSellExactIn,
    BonkSellExactOut,
    BonkInitialize,
    BonkMigrateToAmm,
    BonkMigrateToCpswap,
    PumpSwapBuy,
    PumpSwapSell,
    PumpSwapCreatePool,
    PumpSwapDeposit,
    PumpSwapWithdraw,
    PumpSwapSetParams,
    RaydiumSwap,
    RaydiumDeposit,
    RaydiumWithdraw,
    RaydiumAmmV4SwapBaseIn,
    RaydiumAmmV4SwapBaseOut,
    RaydiumAmmV4Deposit,
    RaydiumAmmV4Initialize2,
    RaydiumAmmV4Withdraw,
    RaydiumAmmV4WithdrawPnl,
    RaydiumClmmSwap,
    RaydiumClmmSwapV2,
    RaydiumClmmCreatePool,
    RaydiumClmmOpenPositionV2,
    RaydiumClmmIncreaseLiquidityV2,
    RaydiumClmmDecreaseLiquidityV2,
    RaydiumClmmClosePosition,
    RaydiumClmmOpenPositionWithToken22Nft,
    RaydiumCpmmSwap,
    RaydiumCpmmSwapBaseInput,
    RaydiumCpmmSwapBaseOutput,
    RaydiumCpmmDeposit,
    RaydiumCpmmWithdraw,
    RaydiumCpmmCreatePool,
    OpenPosition,
    ClosePosition,
    IncreaseLiquidity,
    DecreaseLiquidity,
    Deposit,
    Withdraw,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, BorshDeserialize)]
pub struct EventMetadata {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub protocol_type: ProtocolType,
    pub event_type: EventType,
    pub program_id: Pubkey,
    pub index: String,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
}

impl EventMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        signature: String,
        slot: u64,
        block_time: i64,
        block_time_ms: i64,
        protocol_type: ProtocolType,
        event_type: EventType,
        program_id: Pubkey,
        index: String,
        program_received_time_ms: i64,
    ) -> Self {
        Self {
            id,
            signature,
            slot,
            block_time,
            block_time_ms,
            protocol_type,
            event_type,
            program_id,
            index,
            program_received_time_ms,
            program_handle_time_consuming_ms: 0,
        }
    }

    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferData {
    pub source: Pubkey,
    pub destination: Pubkey,
    pub mint: Option<Pubkey>,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapData {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount_in: u64,
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

// Bridge functions to migrate between old and new metadata formats
impl EventMetadata {
    /// Convert from riglr-events-core EventMetadata to local EventMetadata
    pub fn from_core_metadata(
        core_metadata: &riglr_events_core::types::EventMetadata,
        protocol_type: ProtocolType,
        event_type: EventType,
        program_id: Pubkey,
        index: String,
    ) -> Self {
        Self {
            id: core_metadata.id.clone(),
            signature: core_metadata
                .chain_data
                .as_ref()
                .and_then(|cd| cd.transaction_id())
                .unwrap_or_else(|| "unknown".to_string()),
            slot: core_metadata
                .chain_data
                .as_ref()
                .map(|cd| cd.block_id().parse::<u64>().unwrap_or(0))
                .unwrap_or(0),
            block_time: core_metadata.timestamp.timestamp(),
            block_time_ms: core_metadata.timestamp.timestamp_millis(),
            protocol_type,
            event_type,
            program_id,
            index,
            program_received_time_ms: core_metadata.received_at.timestamp_millis(),
            program_handle_time_consuming_ms: 0,
        }
    }

    /// Convert to riglr-events-core EventMetadata
    pub fn to_core_metadata(
        &self,
        kind: riglr_events_core::types::EventKind,
        source: String,
    ) -> riglr_events_core::types::EventMetadata {
        let chain_data = riglr_events_core::types::ChainData::Solana {
            slot: self.slot,
            signature: Some(self.signature.clone()),
            program_id: Some(self.program_id),
            instruction_index: self.index.parse::<usize>().ok(),
        };

        riglr_events_core::types::EventMetadata::with_timestamp(
            self.id.clone(),
            kind,
            source,
            chrono::DateTime::from_timestamp(self.block_time, 0).unwrap_or_else(chrono::Utc::now),
        )
        .with_chain_data(chain_data)
    }
}
