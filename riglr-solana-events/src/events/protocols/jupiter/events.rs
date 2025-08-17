use super::types::JupiterSwapData;
use crate::{
    // UnifiedEvent removed - using Event trait from riglr_events_core
    types::TransferData,
    // UnifiedEvent removed - events now implement Event trait directly
};
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::any::Any;

// Import new Event trait from riglr-events-core
use chrono::{DateTime, Utc};
use riglr_events_core::{Event, EventKind, EventMetadata};
use std::collections::HashMap;

/// Parameters for creating event metadata, reducing function parameter count
#[derive(Debug, Clone, Default)]
pub struct EventParameters {
    /// Unique identifier for the event
    pub id: String,
    /// Transaction signature
    pub signature: String,
    /// Solana slot number
    pub slot: u64,
    /// Block timestamp in seconds
    pub block_time: i64,
    /// Block timestamp in milliseconds
    pub block_time_ms: i64,
    /// Time when the program received the event in milliseconds
    pub program_received_time_ms: i64,
    /// Event index within the transaction
    pub index: String,
}

impl EventParameters {
    /// Creates a new EventParameters instance with the provided values
    pub fn new(
        id: String,
        signature: String,
        slot: u64,
        block_time: i64,
        block_time_ms: i64,
        program_received_time_ms: i64,
        index: String,
    ) -> Self {
        Self {
            id,
            signature,
            slot,
            block_time,
            block_time_ms,
            program_received_time_ms,
            index,
        }
    }
}

/// Jupiter swap event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JupiterSwapEvent {
    /// Unique identifier for the event
    pub id: String,
    /// Transaction signature
    pub signature: String,
    /// Solana slot number
    pub slot: u64,
    /// Block timestamp in seconds
    pub block_time: i64,
    /// Block timestamp in milliseconds
    pub block_time_ms: i64,
    /// Time when the program received the event in milliseconds
    pub program_received_time_ms: i64,
    /// Time spent handling the event in milliseconds
    pub program_handle_time_consuming_ms: i64,
    /// Event index within the transaction
    pub index: String,
    /// Jupiter-specific swap data
    pub swap_data: JupiterSwapData,
    /// Associated token transfer data
    pub transfer_data: Vec<TransferData>,
}

impl JupiterSwapEvent {
    /// Creates a new JupiterSwapEvent with the provided parameters and swap data
    pub fn new(params: EventParameters, swap_data: JupiterSwapData) -> Self {
        Self {
            id: params.id,
            signature: params.signature,
            slot: params.slot,
            block_time: params.block_time,
            block_time_ms: params.block_time_ms,
            program_received_time_ms: params.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: params.index,
            swap_data,
            transfer_data: Vec::default(),
        }
    }

    /// Adds transfer data to the swap event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

// UnifiedEvent implementation removed - now using Event trait

/// Jupiter liquidity provision event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JupiterLiquidityEvent {
    /// Unique identifier for the event
    pub id: String,
    /// Transaction signature
    pub signature: String,
    /// Solana slot number
    pub slot: u64,
    /// Block timestamp in seconds
    pub block_time: i64,
    /// Block timestamp in milliseconds
    pub block_time_ms: i64,
    /// Time when the program received the event in milliseconds
    pub program_received_time_ms: i64,
    /// Time spent handling the event in milliseconds
    pub program_handle_time_consuming_ms: i64,
    /// Event index within the transaction
    pub index: String,
    /// User account providing/removing liquidity
    pub user: solana_sdk::pubkey::Pubkey,
    /// First token mint address
    pub mint_a: solana_sdk::pubkey::Pubkey,
    /// Second token mint address
    pub mint_b: solana_sdk::pubkey::Pubkey,
    /// Amount of token A
    pub amount_a: u64,
    /// Amount of token B
    pub amount_b: u64,
    /// Amount of liquidity tokens
    pub liquidity_amount: u64,
    /// Whether this is a liquidity removal operation
    pub is_remove: bool,
    /// Associated token transfer data
    pub transfer_data: Vec<TransferData>,
}

// UnifiedEvent implementation removed - now using Event trait

/// Jupiter swap event with borsh (for simple events)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
pub struct JupiterSwapBorshEvent {
    /// Event metadata (skipped during serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: EventMetadata,
    /// User account performing the swap
    pub user: Pubkey,
    /// Input token mint address
    pub input_mint: Pubkey,
    /// Output token mint address
    pub output_mint: Pubkey,
    /// Input token amount
    pub input_amount: u64,
    /// Output token amount received
    pub output_amount: u64,
    /// Slippage tolerance in basis points
    pub slippage_bps: u16,
    /// Platform fee in basis points
    pub platform_fee_bps: u8,
}

// JupiterSwapBorshEvent uses Event trait

// New Event trait implementation for JupiterSwapEvent
impl Event for JupiterSwapEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &EventKind {
        &EventKind::Swap
    }

    fn metadata(&self) -> &EventMetadata {
        use once_cell::sync::Lazy;
        static DEFAULT_METADATA: Lazy<EventMetadata> = Lazy::new(|| EventMetadata {
            id: String::default(),
            kind: EventKind::Swap,
            timestamp: DateTime::<Utc>::MIN_UTC,
            received_at: DateTime::<Utc>::MIN_UTC,
            source: String::from("solana"),
            chain_data: None,
            custom: HashMap::new(),
        });
        &DEFAULT_METADATA
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        // This would need proper implementation with actual mutable metadata storage
        unimplemented!("Mutable metadata not yet implemented")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}

// New Event trait implementation for JupiterLiquidityEvent
impl Event for JupiterLiquidityEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &EventKind {
        &EventKind::Liquidity
    }

    fn metadata(&self) -> &EventMetadata {
        use once_cell::sync::Lazy;
        static DEFAULT_METADATA: Lazy<EventMetadata> = Lazy::new(|| EventMetadata {
            id: String::default(),
            kind: EventKind::Liquidity,
            timestamp: DateTime::<Utc>::MIN_UTC,
            received_at: DateTime::<Utc>::MIN_UTC,
            source: String::from("solana"),
            chain_data: None,
            custom: HashMap::new(),
        });
        &DEFAULT_METADATA
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        // This would need proper implementation with actual mutable metadata storage
        unimplemented!("Mutable metadata not yet implemented")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}

// New Event trait implementation for JupiterSwapBorshEvent
impl Event for JupiterSwapBorshEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        &EventKind::Swap
    }

    fn metadata(&self) -> &EventMetadata {
        use once_cell::sync::Lazy;
        static DEFAULT_METADATA: Lazy<EventMetadata> = Lazy::new(|| EventMetadata {
            id: String::default(),
            kind: EventKind::Swap,
            timestamp: DateTime::<Utc>::MIN_UTC,
            received_at: DateTime::<Utc>::MIN_UTC,
            source: String::from("solana"),
            chain_data: None,
            custom: HashMap::new(),
        });
        &DEFAULT_METADATA
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        // This would need proper implementation with actual mutable metadata storage
        unimplemented!("Mutable metadata not yet implemented")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}
