use super::types::{OrcaLiquidityData, OrcaPositionData, OrcaSwapData};
use crate::{
    // UnifiedEvent removed - using Event trait from riglr_events_core
    types::TransferData,
};
use serde::{Deserialize, Serialize};
use std::any::Any;

// Import Event trait and EventMetadata from riglr-events-core
use riglr_events_core::{Event, EventKind, EventMetadata};

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

/// Orca Whirlpool swap event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrcaSwapEvent {
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
    /// Orca-specific swap data
    pub swap_data: OrcaSwapData,
    /// Associated token transfer data
    pub transfer_data: Vec<TransferData>,
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    pub metadata: EventMetadata,
}

impl OrcaSwapEvent {
    /// Creates a new OrcaSwapEvent with the provided parameters and swap data
    pub fn new(params: EventParameters, swap_data: OrcaSwapData) -> Self {
        let metadata = EventMetadata::default();

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
            metadata,
        }
    }

    /// Sets the transfer data for this swap event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

/// Orca position event (open/close)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrcaPositionEvent {
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
    /// Orca-specific position data
    pub position_data: OrcaPositionData,
    /// Whether the position is being opened (true) or closed (false)
    pub is_open: bool,
    /// Associated token transfer data
    pub transfer_data: Vec<TransferData>,
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    pub metadata: EventMetadata,
}

impl OrcaPositionEvent {
    /// Creates a new OrcaPositionEvent with the provided parameters and position data
    pub fn new(params: EventParameters, position_data: OrcaPositionData, is_open: bool) -> Self {
        let metadata = EventMetadata::default();

        Self {
            id: params.id,
            signature: params.signature,
            slot: params.slot,
            block_time: params.block_time,
            block_time_ms: params.block_time_ms,
            program_received_time_ms: params.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: params.index,
            position_data,
            is_open,
            transfer_data: Vec::new(),
            metadata,
        }
    }

    /// Sets the transfer data for this position event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

/// Orca liquidity event (increase/decrease)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrcaLiquidityEvent {
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
    /// Orca-specific liquidity data
    pub liquidity_data: OrcaLiquidityData,
    /// Associated token transfer data
    pub transfer_data: Vec<TransferData>,
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    pub metadata: EventMetadata,
}

impl OrcaLiquidityEvent {
    /// Creates a new OrcaLiquidityEvent with the provided parameters and liquidity data
    pub fn new(params: EventParameters, liquidity_data: OrcaLiquidityData) -> Self {
        let metadata = EventMetadata::default();

        Self {
            id: params.id,
            signature: params.signature,
            slot: params.slot,
            block_time: params.block_time,
            block_time_ms: params.block_time_ms,
            program_received_time_ms: params.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: params.index,
            liquidity_data,
            transfer_data: Vec::new(),
            metadata,
        }
    }

    /// Sets the transfer data for this liquidity event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

// New Event trait implementation for OrcaSwapEvent
impl Event for OrcaSwapEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        &self.metadata.kind
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        &mut self.metadata
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

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(riglr_events_core::error::EventError::Serialization)
    }
}

// New Event trait implementation for OrcaPositionEvent
impl Event for OrcaPositionEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        &self.metadata.kind
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        &mut self.metadata
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

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(riglr_events_core::error::EventError::Serialization)
    }
}

// New Event trait implementation for OrcaLiquidityEvent
impl Event for OrcaLiquidityEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        &self.metadata.kind
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        &mut self.metadata
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

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(riglr_events_core::error::EventError::Serialization)
    }
}
