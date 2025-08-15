use std::any::Any;
use serde::{Deserialize, Serialize};
use crate::{
    // UnifiedEvent removed - using Event trait from riglr_events_core
    types::{TransferData},
};
use super::types::{MeteoraSwapData, MeteoraLiquidityData, MeteoraDynamicLiquidityData};

// Import new Event trait from riglr-events-core
use riglr_events_core::{Event, EventKind, EventMetadata as CoreEventMetadata};

/// Meteora DLMM swap event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraSwapEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub swap_data: MeteoraSwapData,
    pub transfer_data: Vec<TransferData>,
    #[serde(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
}

impl MeteoraSwapEvent {
    pub fn new(
        id: String,
        signature: String,
        slot: u64,
        block_time: i64,
        block_time_ms: i64,
        program_received_time_ms: i64,
        index: String,
        swap_data: MeteoraSwapData,
    ) -> Self {
        Self {
            id,
            signature,
            slot,
            block_time,
            block_time_ms,
            program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index,
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


// New Event trait implementation
impl Event for MeteoraSwapEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            &EventKind::Swap // Meteora swap event
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata.as_ref().unwrap_or_else(|| {
            panic!("Core metadata not initialized for MeteoraSwapEvent")
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
                EventKind::Swap,
                "meteora".to_string(),
                chrono::DateTime::from_timestamp(self.block_time, 0)
                    .unwrap_or_else(|| chrono::Utc::now()),
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

/// Meteora DLMM liquidity event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraLiquidityEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub liquidity_data: MeteoraLiquidityData,
    pub transfer_data: Vec<TransferData>,
    #[serde(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
}


// New Event trait implementation for MeteoraLiquidityEvent
impl Event for MeteoraLiquidityEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            if self.liquidity_data.is_add {
                &EventKind::Liquidity
            } else {
                &EventKind::Liquidity
            }
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata.as_ref().unwrap_or_else(|| {
            panic!("Core metadata not initialized for MeteoraLiquidityEvent")
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

            let event_kind = EventKind::Liquidity;

            self.core_metadata = Some(CoreEventMetadata::with_timestamp(
                self.id.clone(),
                event_kind,
                "meteora".to_string(),
                chrono::DateTime::from_timestamp(self.block_time, 0)
                    .unwrap_or_else(|| chrono::Utc::now()),
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

/// Meteora Dynamic AMM liquidity event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDynamicLiquidityEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub liquidity_data: MeteoraDynamicLiquidityData,
    pub transfer_data: Vec<TransferData>,
    #[serde(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
}


// New Event trait implementation for MeteoraDynamicLiquidityEvent
impl Event for MeteoraDynamicLiquidityEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            if self.liquidity_data.is_deposit {
                &EventKind::Liquidity
            } else {
                &EventKind::Liquidity
            }
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata.as_ref().unwrap_or_else(|| {
            panic!("Core metadata not initialized for MeteoraDynamicLiquidityEvent")
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

            let event_kind = EventKind::Liquidity;

            self.core_metadata = Some(CoreEventMetadata::with_timestamp(
                self.id.clone(),
                event_kind,
                "meteora".to_string(),
                chrono::DateTime::from_timestamp(self.block_time, 0)
                    .unwrap_or_else(|| chrono::Utc::now()),
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