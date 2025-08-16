use super::types::JupiterSwapData;
use crate::{
    // UnifiedEvent removed - using Event trait from riglr_events_core
    events::common::EventMetadata,
    types::TransferData,
    // UnifiedEvent removed - events now implement Event trait directly
};
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::any::Any;

// Import new Event trait from riglr-events-core
use chrono::{DateTime, Utc};
use riglr_events_core::{Event, EventKind, EventMetadata as CoreEventMetadata};
use std::collections::HashMap;

/// Parameters for creating event metadata, reducing function parameter count
#[derive(Debug, Clone)]
pub struct EventParameters {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub index: String,
}

impl EventParameters {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterSwapEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub swap_data: JupiterSwapData,
    pub transfer_data: Vec<TransferData>,
}

impl JupiterSwapEvent {
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
            transfer_data: Vec::new(),
        }
    }

    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

// UnifiedEvent implementation removed - now using Event trait

/// Jupiter liquidity provision event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterLiquidityEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub user: solana_sdk::pubkey::Pubkey,
    pub mint_a: solana_sdk::pubkey::Pubkey,
    pub mint_b: solana_sdk::pubkey::Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub liquidity_amount: u64,
    pub is_remove: bool,
    pub transfer_data: Vec<TransferData>,
}

// UnifiedEvent implementation removed - now using Event trait

/// Jupiter swap event with borsh (for simple events)
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct JupiterSwapBorshEvent {
    #[serde(skip)]
    pub metadata: EventMetadata,
    pub user: Pubkey,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub input_amount: u64,
    pub output_amount: u64,
    pub slippage_bps: u16,
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

    fn metadata(&self) -> &CoreEventMetadata {
        use once_cell::sync::Lazy;
        static DEFAULT_METADATA: Lazy<CoreEventMetadata> = Lazy::new(|| CoreEventMetadata {
            id: String::new(),
            kind: EventKind::Swap,
            timestamp: DateTime::<Utc>::MIN_UTC,
            received_at: DateTime::<Utc>::MIN_UTC,
            source: String::from("solana"),
            chain_data: None,
            custom: HashMap::new(),
        });
        &DEFAULT_METADATA
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
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

    fn metadata(&self) -> &CoreEventMetadata {
        use once_cell::sync::Lazy;
        static DEFAULT_METADATA: Lazy<CoreEventMetadata> = Lazy::new(|| CoreEventMetadata {
            id: String::new(),
            kind: EventKind::Liquidity,
            timestamp: DateTime::<Utc>::MIN_UTC,
            received_at: DateTime::<Utc>::MIN_UTC,
            source: String::from("solana"),
            chain_data: None,
            custom: HashMap::new(),
        });
        &DEFAULT_METADATA
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
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

    fn metadata(&self) -> &CoreEventMetadata {
        use once_cell::sync::Lazy;
        static DEFAULT_METADATA: Lazy<CoreEventMetadata> = Lazy::new(|| CoreEventMetadata {
            id: String::new(),
            kind: EventKind::Swap,
            timestamp: DateTime::<Utc>::MIN_UTC,
            received_at: DateTime::<Utc>::MIN_UTC,
            source: String::from("solana"),
            chain_data: None,
            custom: HashMap::new(),
        });
        &DEFAULT_METADATA
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
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
