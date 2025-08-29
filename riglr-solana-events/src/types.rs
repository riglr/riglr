use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

// Re-export types from riglr-events-core
pub use riglr_events_core::prelude::*;

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
    /// General Raydium protocol
    Raydium,
    /// Jupiter aggregator protocol
    Jupiter,
    /// Serum decentralized exchange
    Serum,
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
            ProtocolType::Raydium => write!(f, "Raydium"),
            ProtocolType::Jupiter => write!(f, "Jupiter"),
            ProtocolType::Serum => write!(f, "Serum"),
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
    /// Add liquidity to a pool (alternative name)
    LiquidityProvision,
    /// Remove liquidity from a pool (alternative name)
    LiquidityRemoval,
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

    /// Create a SolanaEventMetadata without duplication in chain_data
    #[allow(clippy::too_many_arguments)]
    pub fn create_solana_metadata(
        id: String,
        signature: String,
        slot: u64,
        block_time: i64,
        protocol_type: ProtocolType,
        event_type: EventType,
        _program_id: Pubkey,
        index: String,
        program_received_time_ms: i64,
    ) -> crate::solana_metadata::SolanaEventMetadata {
        let kind = event_type.to_event_kind();
        let source = format!("solana-{}", protocol_type);

        // Use create_core_metadata to avoid duplication
        let metadata =
            crate::metadata_helpers::create_core_metadata(id, kind, source, Some(block_time));

        // Create SolanaEventMetadata wrapper with the correct parameters
        crate::solana_metadata::SolanaEventMetadata {
            signature,
            slot,
            event_type,
            protocol_type,
            index,
            program_received_time_ms,
            core: metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_events_core::types::EventKind;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    #[test]
    fn test_protocol_type_default() {
        let default_protocol = ProtocolType::default();
        assert_eq!(default_protocol, ProtocolType::Other("Unknown".to_string()));
    }

    #[test]
    fn test_protocol_type_display_all_variants() {
        assert_eq!(ProtocolType::OrcaWhirlpool.to_string(), "Orca Whirlpool");
        assert_eq!(ProtocolType::MeteoraDlmm.to_string(), "Meteora DLMM");
        assert_eq!(ProtocolType::MarginFi.to_string(), "MarginFi");
        assert_eq!(ProtocolType::Bonk.to_string(), "Bonk");
        assert_eq!(ProtocolType::PumpSwap.to_string(), "PumpSwap");
        assert_eq!(ProtocolType::RaydiumAmm.to_string(), "Raydium AMM");
        assert_eq!(ProtocolType::RaydiumAmmV4.to_string(), "Raydium AMM V4");
        assert_eq!(ProtocolType::RaydiumClmm.to_string(), "Raydium CLMM");
        assert_eq!(ProtocolType::RaydiumCpmm.to_string(), "Raydium CPMM");
        assert_eq!(ProtocolType::Jupiter.to_string(), "Jupiter");
        assert_eq!(
            ProtocolType::Other("CustomProtocol".to_string()).to_string(),
            "CustomProtocol"
        );
    }

    #[test]
    fn test_protocol_type_serialization() {
        let protocol = ProtocolType::OrcaWhirlpool;
        let serialized = serde_json::to_string(&protocol).unwrap();
        let deserialized: ProtocolType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(protocol, deserialized);
    }

    #[test]
    fn test_protocol_type_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(ProtocolType::OrcaWhirlpool, "value1");
        map.insert(ProtocolType::Jupiter, "value2");

        assert_eq!(map.get(&ProtocolType::OrcaWhirlpool), Some(&"value1"));
        assert_eq!(map.get(&ProtocolType::Jupiter), Some(&"value2"));
    }

    #[test]
    fn test_event_type_default() {
        let default_event = EventType::default();
        assert_eq!(default_event, EventType::Unknown);
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(EventType::Swap.to_string(), "Swap");
        assert_eq!(EventType::Unknown.to_string(), "Unknown");
        assert_eq!(EventType::RaydiumSwap.to_string(), "RaydiumSwap");
    }

    #[test]
    fn test_event_type_to_event_kind_swap() {
        assert_eq!(EventType::Swap.to_event_kind(), EventKind::Swap);
        assert_eq!(EventType::Trade.to_event_kind(), EventKind::Swap);
    }

    #[test]
    fn test_event_type_to_event_kind_liquidity() {
        assert_eq!(
            EventType::AddLiquidity.to_event_kind(),
            EventKind::Liquidity
        );
        assert_eq!(
            EventType::IncreaseLiquidity.to_event_kind(),
            EventKind::Liquidity
        );
        assert_eq!(
            EventType::RemoveLiquidity.to_event_kind(),
            EventKind::Liquidity
        );
        assert_eq!(
            EventType::DecreaseLiquidity.to_event_kind(),
            EventKind::Liquidity
        );
    }

    #[test]
    fn test_event_type_to_event_kind_transfer() {
        assert_eq!(EventType::Transfer.to_event_kind(), EventKind::Transfer);
        assert_eq!(EventType::Deposit.to_event_kind(), EventKind::Transfer);
        assert_eq!(EventType::Withdraw.to_event_kind(), EventKind::Transfer);
    }

    #[test]
    fn test_event_type_to_event_kind_basic_types() {
        assert_eq!(
            EventType::Transaction.to_event_kind(),
            EventKind::Transaction
        );
        assert_eq!(EventType::Block.to_event_kind(), EventKind::Block);
        assert_eq!(
            EventType::ContractEvent.to_event_kind(),
            EventKind::Contract
        );
        assert_eq!(EventType::PriceUpdate.to_event_kind(), EventKind::Price);
    }

    #[test]
    fn test_event_type_to_event_kind_custom() {
        // Test that all other variants become Custom
        let custom_event = EventType::RaydiumSwap.to_event_kind();
        match custom_event {
            EventKind::Custom(name) => assert_eq!(name, "RaydiumSwap"),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn test_event_type_from_event_kind_basic() {
        assert_eq!(
            EventType::from_event_kind(&EventKind::Transaction),
            EventType::Transaction
        );
        assert_eq!(
            EventType::from_event_kind(&EventKind::Block),
            EventType::Block
        );
        assert_eq!(
            EventType::from_event_kind(&EventKind::Contract),
            EventType::ContractEvent
        );
        assert_eq!(
            EventType::from_event_kind(&EventKind::Transfer),
            EventType::Transfer
        );
        assert_eq!(
            EventType::from_event_kind(&EventKind::Swap),
            EventType::Swap
        );
        assert_eq!(
            EventType::from_event_kind(&EventKind::Liquidity),
            EventType::AddLiquidity
        );
        assert_eq!(
            EventType::from_event_kind(&EventKind::Price),
            EventType::PriceUpdate
        );
        assert_eq!(
            EventType::from_event_kind(&EventKind::External),
            EventType::ContractEvent
        );
    }

    #[test]
    fn test_event_type_from_event_kind_custom_known() {
        let raydium_kind = EventKind::Custom("RaydiumSwap".to_string());
        assert_eq!(
            EventType::from_event_kind(&raydium_kind),
            EventType::RaydiumSwap
        );

        let jupiter_kind = EventKind::Custom("JupiterSwap".to_string());
        assert_eq!(EventType::from_event_kind(&jupiter_kind), EventType::Swap);

        let pump_buy_kind = EventKind::Custom("PumpSwapBuy".to_string());
        assert_eq!(
            EventType::from_event_kind(&pump_buy_kind),
            EventType::PumpSwapBuy
        );

        let pump_sell_kind = EventKind::Custom("PumpSwapSell".to_string());
        assert_eq!(
            EventType::from_event_kind(&pump_sell_kind),
            EventType::PumpSwapSell
        );
    }

    #[test]
    fn test_event_type_from_event_kind_custom_unknown() {
        let unknown_kind = EventKind::Custom("SomeUnknownEvent".to_string());
        assert_eq!(
            EventType::from_event_kind(&unknown_kind),
            EventType::Unknown
        );
    }

    #[test]
    fn test_transfer_data_creation() {
        let source = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        let destination = Pubkey::from_str("11111111111111111111111111111113").unwrap();
        let mint = Some(Pubkey::from_str("11111111111111111111111111111114").unwrap());

        let transfer_data = TransferData {
            source,
            destination,
            mint,
            amount: 1000,
        };

        assert_eq!(transfer_data.source, source);
        assert_eq!(transfer_data.destination, destination);
        assert_eq!(transfer_data.mint, mint);
        assert_eq!(transfer_data.amount, 1000);
    }

    #[test]
    fn test_transfer_data_serialization() {
        let source = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        let destination = Pubkey::from_str("11111111111111111111111111111113").unwrap();

        let transfer_data = TransferData {
            source,
            destination,
            mint: None,
            amount: 500,
        };

        let serialized = serde_json::to_string(&transfer_data).unwrap();
        let deserialized: TransferData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(transfer_data.source, deserialized.source);
        assert_eq!(transfer_data.destination, deserialized.destination);
        assert_eq!(transfer_data.mint, deserialized.mint);
        assert_eq!(transfer_data.amount, deserialized.amount);
    }

    #[test]
    fn test_swap_data_creation() {
        let input_mint = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        let output_mint = Pubkey::from_str("11111111111111111111111111111113").unwrap();

        let swap_data = SwapData {
            input_mint,
            output_mint,
            amount_in: 1000,
            amount_out: 950,
        };

        assert_eq!(swap_data.input_mint, input_mint);
        assert_eq!(swap_data.output_mint, output_mint);
        assert_eq!(swap_data.amount_in, 1000);
        assert_eq!(swap_data.amount_out, 950);
    }

    #[test]
    fn test_swap_data_serialization() {
        let input_mint = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        let output_mint = Pubkey::from_str("11111111111111111111111111111113").unwrap();

        let swap_data = SwapData {
            input_mint,
            output_mint,
            amount_in: 2000,
            amount_out: 1900,
        };

        let serialized = serde_json::to_string(&swap_data).unwrap();
        let deserialized: SwapData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(swap_data.input_mint, deserialized.input_mint);
        assert_eq!(swap_data.output_mint, deserialized.output_mint);
        assert_eq!(swap_data.amount_in, deserialized.amount_in);
        assert_eq!(swap_data.amount_out, deserialized.amount_out);
    }

    #[test]
    fn test_stream_metadata_creation() {
        let now = std::time::SystemTime::now();
        let custom_data = Some(serde_json::json!({"key": "value"}));

        let stream_metadata = StreamMetadata {
            stream_source: "geyser".to_string(),
            received_at: now,
            sequence_number: Some(12345),
            custom_data: custom_data.clone(),
        };

        assert_eq!(stream_metadata.stream_source, "geyser");
        assert_eq!(stream_metadata.received_at, now);
        assert_eq!(stream_metadata.sequence_number, Some(12345));
        assert_eq!(stream_metadata.custom_data, custom_data);
    }

    #[test]
    fn test_stream_metadata_serialization() {
        let now = std::time::SystemTime::UNIX_EPOCH;

        let stream_metadata = StreamMetadata {
            stream_source: "binance".to_string(),
            received_at: now,
            sequence_number: None,
            custom_data: None,
        };

        let serialized = serde_json::to_string(&stream_metadata).unwrap();
        let deserialized: StreamMetadata = serde_json::from_str(&serialized).unwrap();

        assert_eq!(stream_metadata.stream_source, deserialized.stream_source);
        assert_eq!(stream_metadata.received_at, deserialized.received_at);
        assert_eq!(
            stream_metadata.sequence_number,
            deserialized.sequence_number
        );
        assert_eq!(stream_metadata.custom_data, deserialized.custom_data);
    }

    #[test]
    fn test_metadata_helpers_create_solana_metadata() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();

        let metadata = metadata_helpers::create_solana_metadata(
            "test-id".to_string(),
            "test-signature".to_string(),
            12345,
            1640995200, // Unix timestamp
            ProtocolType::OrcaWhirlpool,
            EventType::Swap,
            program_id,
            "0".to_string(),
            1640995200000,
        );

        // Test that metadata was created successfully
        assert_eq!(metadata.signature, "test-signature");
        assert_eq!(metadata.slot, 12345);
        assert_eq!(metadata.event_type, EventType::Swap);
        assert_eq!(metadata.protocol_type, ProtocolType::OrcaWhirlpool);
        assert_eq!(metadata.index, "0");
        assert_eq!(metadata.program_received_time_ms, 1640995200000);
    }

    #[test]
    fn test_metadata_helpers_create_solana_metadata_with_invalid_timestamp() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();

        // Test with invalid timestamp - should use current time
        let metadata = metadata_helpers::create_solana_metadata(
            "test-id-2".to_string(),
            "test-signature-2".to_string(),
            54321,
            -1, // Invalid timestamp
            ProtocolType::Jupiter,
            EventType::Trade,
            program_id,
            "1".to_string(),
            1640995300000,
        );

        // Should still create metadata successfully
        assert_eq!(metadata.signature, "test-signature-2");
        assert_eq!(metadata.slot, 54321);
        assert_eq!(metadata.event_type, EventType::Trade);
        assert_eq!(metadata.protocol_type, ProtocolType::Jupiter);
    }

    #[test]
    fn test_metadata_helpers_create_solana_metadata_with_non_numeric_index() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();

        let metadata = metadata_helpers::create_solana_metadata(
            "test-id-3".to_string(),
            "test-signature-3".to_string(),
            99999,
            1640995400,
            ProtocolType::RaydiumAmm,
            EventType::AddLiquidity,
            program_id,
            "not-a-number".to_string(), // Non-numeric index
            1640995400000,
        );

        // Should still create metadata successfully
        assert_eq!(metadata.signature, "test-signature-3");
        assert_eq!(metadata.slot, 99999);
        assert_eq!(metadata.index, "not-a-number");
    }

    #[test]
    fn test_event_type_serialization() {
        let event = EventType::RaydiumClmmSwapV2;
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: EventType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_event_type_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(EventType::Swap, "swap_value");
        map.insert(EventType::AddLiquidity, "liquidity_value");

        assert_eq!(map.get(&EventType::Swap), Some(&"swap_value"));
        assert_eq!(map.get(&EventType::AddLiquidity), Some(&"liquidity_value"));
    }

    #[test]
    fn test_all_protocol_type_variants_equality() {
        let protocols = vec![
            ProtocolType::OrcaWhirlpool,
            ProtocolType::MeteoraDlmm,
            ProtocolType::MarginFi,
            ProtocolType::Bonk,
            ProtocolType::PumpSwap,
            ProtocolType::RaydiumAmm,
            ProtocolType::RaydiumAmmV4,
            ProtocolType::RaydiumClmm,
            ProtocolType::RaydiumCpmm,
            ProtocolType::Jupiter,
            ProtocolType::Other("test".to_string()),
        ];

        for protocol in &protocols {
            assert_eq!(protocol, protocol);
        }
    }

    #[test]
    fn test_all_event_type_variants_equality() {
        let events = vec![
            EventType::Swap,
            EventType::AddLiquidity,
            EventType::RemoveLiquidity,
            EventType::Borrow,
            EventType::Repay,
            EventType::Liquidate,
            EventType::Transfer,
            EventType::Mint,
            EventType::Burn,
            EventType::CreatePool,
            EventType::UpdatePool,
            EventType::Transaction,
            EventType::Block,
            EventType::ContractEvent,
            EventType::PriceUpdate,
            EventType::OrderBook,
            EventType::Trade,
            EventType::FeeUpdate,
            EventType::Unknown,
        ];

        for event in &events {
            assert_eq!(event, event);
        }
    }

    #[test]
    fn test_protocol_type_other_with_empty_string() {
        let protocol = ProtocolType::Other("".to_string());
        assert_eq!(protocol.to_string(), "");
    }

    #[test]
    fn test_protocol_type_other_with_special_characters() {
        let protocol = ProtocolType::Other("Test-Protocol_123!@#".to_string());
        assert_eq!(protocol.to_string(), "Test-Protocol_123!@#");
    }

    #[test]
    fn test_event_type_debug_format() {
        let event = EventType::BonkBuyExactIn;
        let debug_string = format!("{:?}", event);
        assert_eq!(debug_string, "BonkBuyExactIn");
    }

    #[test]
    fn test_protocol_type_debug_format() {
        let protocol = ProtocolType::MeteoraDlmm;
        let debug_string = format!("{:?}", protocol);
        assert_eq!(debug_string, "MeteoraDlmm");
    }
}
