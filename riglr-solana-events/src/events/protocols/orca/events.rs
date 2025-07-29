use std::any::Any;
use serde::{Deserialize, Serialize};
use crate::{
    // UnifiedEvent removed - using Event trait from riglr_events_core
    types::{TransferData},
};
use super::types::{OrcaSwapData, OrcaPositionData, OrcaLiquidityData};

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

/// Orca Whirlpool swap event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaSwapEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub swap_data: OrcaSwapData,
    pub transfer_data: Vec<TransferData>,
    #[serde(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
}

impl OrcaSwapEvent {
    pub fn new(params: EventParameters, swap_data: OrcaSwapData) -> Self {
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
            core_metadata: None,
        }
    }

    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}


/// Orca position event (open/close)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaPositionEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub position_data: OrcaPositionData,
    pub is_open: bool,
    pub transfer_data: Vec<TransferData>,
    #[serde(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
}


/// Orca liquidity event (increase/decrease)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaLiquidityEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub liquidity_data: OrcaLiquidityData,
    pub transfer_data: Vec<TransferData>,
    #[serde(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
}


// New Event trait implementation for OrcaSwapEvent
impl Event for OrcaSwapEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            &EventKind::Swap
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata.as_ref().unwrap_or_else(|| {
            panic!("Core metadata not initialized for OrcaSwapEvent")
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
                EventKind::Swap,
                "orca".to_string(),
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

// New Event trait implementation for OrcaPositionEvent
impl Event for OrcaPositionEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            &EventKind::Contract
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata.as_ref().unwrap_or_else(|| {
            panic!("Core metadata not initialized for OrcaPositionEvent")
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
                EventKind::Contract,
                "orca".to_string(),
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

// New Event trait implementation for OrcaLiquidityEvent
impl Event for OrcaLiquidityEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            &EventKind::Liquidity
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata.as_ref().unwrap_or_else(|| {
            panic!("Core metadata not initialized for OrcaLiquidityEvent")
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
                EventKind::Liquidity,
                "orca".to_string(),
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
