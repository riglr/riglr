use std::any::Any;
use serde::{Deserialize, Serialize};
use crate::{
    // UnifiedEvent removed - using Event trait from riglr_events_core
    types::{TransferData},
};
use super::types::{
    MarginFiDepositData, MarginFiWithdrawData, MarginFiBorrowData, 
    MarginFiRepayData, MarginFiLiquidationData
};

// Import new Event trait from riglr-events-core
use riglr_events_core::{Event, EventKind, EventMetadata as CoreEventMetadata};

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

/// MarginFi deposit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiDepositEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub deposit_data: MarginFiDepositData,
    pub transfer_data: Vec<TransferData>,
    #[serde(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
}

impl MarginFiDepositEvent {
    pub fn new(params: EventParameters, deposit_data: MarginFiDepositData) -> Self {
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
            core_metadata: None,
        }
    }

    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}


// New Event trait implementation for MarginFiDepositEvent
impl Event for MarginFiDepositEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            &EventKind::Transfer // MarginFi deposit event
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata.as_ref().unwrap_or_else(|| {
            panic!("Core metadata not initialized for MarginFiDepositEvent")
        })
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        if self.core_metadata.is_none() {
            // Create new core metadata from legacy fields
            let chain_data = riglr_events_core::types::ChainData::Solana {
                slot: self.slot,
                signature: Some(self.signature.clone()),
                program_id: None, // Will be set by parser
                instruction_index: self.index.parse::<usize>().ok(),
            };

            self.core_metadata = Some(CoreEventMetadata::with_timestamp(
                self.id.clone(),
                EventKind::Transfer,
                "marginfi".to_string(),
                chrono::DateTime::from_timestamp(self.block_time, 0)
                    .unwrap_or_else(chrono::Utc::now),
            ).with_chain_data(chain_data));
        }
        self.core_metadata.as_mut().unwrap()
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
        serde_json::to_value(self)
            .map_err(riglr_events_core::error::EventError::Serialization)
    }
}

/// MarginFi withdraw event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiWithdrawEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub withdraw_data: MarginFiWithdrawData,
    pub transfer_data: Vec<TransferData>,
    #[serde(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
}


// New Event trait implementation for MarginFiWithdrawEvent
impl Event for MarginFiWithdrawEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            &EventKind::Transfer
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata.as_ref().unwrap_or_else(|| {
            panic!("Core metadata not initialized for MarginFiWithdrawEvent")
        })
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        if self.core_metadata.is_none() {
            let chain_data = riglr_events_core::types::ChainData::Solana {
                slot: self.slot,
                signature: Some(self.signature.clone()),
                program_id: None,
                instruction_index: self.index.parse::<usize>().ok(),
            };

            self.core_metadata = Some(CoreEventMetadata::with_timestamp(
                self.id.clone(),
                EventKind::Transfer,
                "marginfi".to_string(),
                chrono::DateTime::from_timestamp(self.block_time, 0)
                    .unwrap_or_else(chrono::Utc::now),
            ).with_chain_data(chain_data));
        }
        self.core_metadata.as_mut().unwrap()
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
        serde_json::to_value(self)
            .map_err(riglr_events_core::error::EventError::Serialization)
    }
}

/// MarginFi borrow event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiBorrowEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub borrow_data: MarginFiBorrowData,
    pub transfer_data: Vec<TransferData>,
    #[serde(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
}


/// MarginFi repay event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiRepayEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub repay_data: MarginFiRepayData,
    pub transfer_data: Vec<TransferData>,
    #[serde(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
}


/// MarginFi liquidation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiLiquidationEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub liquidation_data: MarginFiLiquidationData,
    pub transfer_data: Vec<TransferData>,
    #[serde(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
}

// New Event trait implementation for MarginFiBorrowEvent
impl Event for MarginFiBorrowEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            &EventKind::Transfer
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata.as_ref().unwrap_or_else(|| {
            panic!("Core metadata not initialized for MarginFiBorrowEvent")
        })
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        if self.core_metadata.is_none() {
            let chain_data = riglr_events_core::types::ChainData::Solana {
                slot: self.slot,
                signature: Some(self.signature.clone()),
                program_id: None,
                instruction_index: self.index.parse::<usize>().ok(),
            };

            self.core_metadata = Some(CoreEventMetadata::with_timestamp(
                self.id.clone(),
                EventKind::Transfer,
                "marginfi".to_string(),
                chrono::DateTime::from_timestamp(self.block_time, 0)
                    .unwrap_or_else(chrono::Utc::now),
            ).with_chain_data(chain_data));
        }
        self.core_metadata.as_mut().unwrap()
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
        serde_json::to_value(self)
            .map_err(riglr_events_core::error::EventError::Serialization)
    }
}

// New Event trait implementation for MarginFiRepayEvent
impl Event for MarginFiRepayEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            &EventKind::Transfer
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata.as_ref().unwrap_or_else(|| {
            panic!("Core metadata not initialized for MarginFiRepayEvent")
        })
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        if self.core_metadata.is_none() {
            let chain_data = riglr_events_core::types::ChainData::Solana {
                slot: self.slot,
                signature: Some(self.signature.clone()),
                program_id: None,
                instruction_index: self.index.parse::<usize>().ok(),
            };

            self.core_metadata = Some(CoreEventMetadata::with_timestamp(
                self.id.clone(),
                EventKind::Transfer,
                "marginfi".to_string(),
                chrono::DateTime::from_timestamp(self.block_time, 0)
                    .unwrap_or_else(chrono::Utc::now),
            ).with_chain_data(chain_data));
        }
        self.core_metadata.as_mut().unwrap()
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
        serde_json::to_value(self)
            .map_err(riglr_events_core::error::EventError::Serialization)
    }
}

// New Event trait implementation for MarginFiLiquidationEvent
impl Event for MarginFiLiquidationEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            &EventKind::Transfer
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata.as_ref().unwrap_or_else(|| {
            panic!("Core metadata not initialized for MarginFiLiquidationEvent")
        })
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        if self.core_metadata.is_none() {
            let chain_data = riglr_events_core::types::ChainData::Solana {
                slot: self.slot,
                signature: Some(self.signature.clone()),
                program_id: None,
                instruction_index: self.index.parse::<usize>().ok(),
            };

            self.core_metadata = Some(CoreEventMetadata::with_timestamp(
                self.id.clone(),
                EventKind::Transfer,
                "marginfi".to_string(),
                chrono::DateTime::from_timestamp(self.block_time, 0)
                    .unwrap_or_else(chrono::Utc::now),
            ).with_chain_data(chain_data));
        }
        self.core_metadata.as_mut().unwrap()
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
        serde_json::to_value(self)
            .map_err(riglr_events_core::error::EventError::Serialization)
    }
}
