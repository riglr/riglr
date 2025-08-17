use super::types::{
    MarginFiBorrowData, MarginFiDepositData, MarginFiLiquidationData, MarginFiRepayData,
    MarginFiWithdrawData,
};
use crate::{
    // UnifiedEvent removed - using Event trait from riglr_events_core
    types::TransferData,
};
use serde::{Deserialize, Serialize};
use std::any::Any;

// Import new Event trait from riglr-events-core
use riglr_events_core::{Event, EventKind, EventMetadata};

/// Parameters for creating event metadata, reducing function parameter count
#[derive(Debug, Clone, Default)]
pub struct EventParameters {
    /// Unique identifier for the event
    pub id: String,
    /// Transaction signature hash
    pub signature: String,
    /// Solana slot number when the transaction was processed
    pub slot: u64,
    /// Block timestamp in seconds since Unix epoch
    pub block_time: i64,
    /// Block timestamp in milliseconds since Unix epoch
    pub block_time_ms: i64,
    /// Timestamp when the program received the transaction in milliseconds
    pub program_received_time_ms: i64,
    /// Index of the instruction within the transaction
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

/// MarginFi deposit event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginFiDepositEvent {
    /// Unique identifier for the event
    pub id: String,
    /// Transaction signature hash
    pub signature: String,
    /// Solana slot number when the transaction was processed
    pub slot: u64,
    /// Block timestamp in seconds since Unix epoch
    pub block_time: i64,
    /// Block timestamp in milliseconds since Unix epoch
    pub block_time_ms: i64,
    /// Timestamp when the program received the transaction in milliseconds
    pub program_received_time_ms: i64,
    /// Time spent handling the transaction in milliseconds
    pub program_handle_time_consuming_ms: i64,
    /// Index of the instruction within the transaction
    pub index: String,
    /// MarginFi-specific deposit operation data
    pub deposit_data: MarginFiDepositData,
    /// Associated token transfer data for this deposit
    pub transfer_data: Vec<TransferData>,
    /// Event metadata for unified event handling
    #[serde(skip)]
    pub metadata: EventMetadata,
}

impl MarginFiDepositEvent {
    /// Creates a new MarginFi deposit event with the provided parameters and deposit data
    pub fn new(params: EventParameters, deposit_data: MarginFiDepositData) -> Self {
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
            deposit_data,
            transfer_data: Vec::new(),
            metadata,
        }
    }

    /// Adds transfer data to the deposit event and returns the modified event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

// New Event trait implementation for MarginFiDepositEvent
impl Event for MarginFiDepositEvent {
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

/// MarginFi withdraw event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginFiWithdrawEvent {
    /// Unique identifier for the event
    pub id: String,
    /// Transaction signature hash
    pub signature: String,
    /// Solana slot number when the transaction was processed
    pub slot: u64,
    /// Block timestamp in seconds since Unix epoch
    pub block_time: i64,
    /// Block timestamp in milliseconds since Unix epoch
    pub block_time_ms: i64,
    /// Timestamp when the program received the transaction in milliseconds
    pub program_received_time_ms: i64,
    /// Time spent handling the transaction in milliseconds
    pub program_handle_time_consuming_ms: i64,
    /// Index of the instruction within the transaction
    pub index: String,
    /// MarginFi-specific withdraw operation data
    pub withdraw_data: MarginFiWithdrawData,
    /// Associated token transfer data for this withdrawal
    pub transfer_data: Vec<TransferData>,
    /// Event metadata for unified event handling
    #[serde(skip)]
    pub metadata: EventMetadata,
}

impl MarginFiWithdrawEvent {
    /// Creates a new MarginFi withdraw event with the provided parameters and withdraw data
    pub fn new(params: EventParameters, withdraw_data: MarginFiWithdrawData) -> Self {
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
            withdraw_data,
            transfer_data: Vec::new(),
            metadata,
        }
    }

    /// Sets the transfer data for this withdraw event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

// New Event trait implementation for MarginFiWithdrawEvent
impl Event for MarginFiWithdrawEvent {
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

/// MarginFi borrow event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginFiBorrowEvent {
    /// Unique identifier for the event
    pub id: String,
    /// Transaction signature hash
    pub signature: String,
    /// Solana slot number when the transaction was processed
    pub slot: u64,
    /// Block timestamp in seconds since Unix epoch
    pub block_time: i64,
    /// Block timestamp in milliseconds since Unix epoch
    pub block_time_ms: i64,
    /// Timestamp when the program received the transaction in milliseconds
    pub program_received_time_ms: i64,
    /// Time spent handling the transaction in milliseconds
    pub program_handle_time_consuming_ms: i64,
    /// Index of the instruction within the transaction
    pub index: String,
    /// MarginFi-specific borrow operation data
    pub borrow_data: MarginFiBorrowData,
    /// Associated token transfer data for this borrow
    pub transfer_data: Vec<TransferData>,
    /// Event metadata for unified event handling
    #[serde(skip)]
    pub metadata: EventMetadata,
}

impl MarginFiBorrowEvent {
    /// Creates a new MarginFi borrow event with the provided parameters and borrow data
    pub fn new(params: EventParameters, borrow_data: MarginFiBorrowData) -> Self {
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
            borrow_data,
            transfer_data: Vec::new(),
            metadata,
        }
    }

    /// Sets the transfer data for this borrow event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

/// MarginFi repay event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginFiRepayEvent {
    /// Unique identifier for the event
    pub id: String,
    /// Transaction signature hash
    pub signature: String,
    /// Solana slot number when the transaction was processed
    pub slot: u64,
    /// Block timestamp in seconds since Unix epoch
    pub block_time: i64,
    /// Block timestamp in milliseconds since Unix epoch
    pub block_time_ms: i64,
    /// Timestamp when the program received the transaction in milliseconds
    pub program_received_time_ms: i64,
    /// Time spent handling the transaction in milliseconds
    pub program_handle_time_consuming_ms: i64,
    /// Index of the instruction within the transaction
    pub index: String,
    /// MarginFi-specific repay operation data
    pub repay_data: MarginFiRepayData,
    /// Associated token transfer data for this repayment
    pub transfer_data: Vec<TransferData>,
    /// Event metadata for unified event handling
    #[serde(skip)]
    pub metadata: EventMetadata,
}

impl MarginFiRepayEvent {
    /// Creates a new MarginFi repay event with the provided parameters and repay data
    pub fn new(params: EventParameters, repay_data: MarginFiRepayData) -> Self {
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
            repay_data,
            transfer_data: Vec::new(),
            metadata,
        }
    }

    /// Sets the transfer data for this repay event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

/// MarginFi liquidation event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginFiLiquidationEvent {
    /// Unique identifier for the event
    pub id: String,
    /// Transaction signature hash
    pub signature: String,
    /// Solana slot number when the transaction was processed
    pub slot: u64,
    /// Block timestamp in seconds since Unix epoch
    pub block_time: i64,
    /// Block timestamp in milliseconds since Unix epoch
    pub block_time_ms: i64,
    /// Timestamp when the program received the transaction in milliseconds
    pub program_received_time_ms: i64,
    /// Time spent handling the transaction in milliseconds
    pub program_handle_time_consuming_ms: i64,
    /// Index of the instruction within the transaction
    pub index: String,
    /// MarginFi-specific liquidation operation data
    pub liquidation_data: MarginFiLiquidationData,
    /// Associated token transfer data for this liquidation
    pub transfer_data: Vec<TransferData>,
    /// Event metadata for unified event handling
    #[serde(skip)]
    pub metadata: EventMetadata,
}

impl MarginFiLiquidationEvent {
    /// Creates a new MarginFi liquidation event with the provided parameters and liquidation data
    pub fn new(params: EventParameters, liquidation_data: MarginFiLiquidationData) -> Self {
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
            liquidation_data,
            transfer_data: Vec::new(),
            metadata,
        }
    }

    /// Sets the transfer data for this liquidation event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

// New Event trait implementation for MarginFiBorrowEvent
impl Event for MarginFiBorrowEvent {
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

// New Event trait implementation for MarginFiRepayEvent
impl Event for MarginFiRepayEvent {
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

// New Event trait implementation for MarginFiLiquidationEvent
impl Event for MarginFiLiquidationEvent {
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
