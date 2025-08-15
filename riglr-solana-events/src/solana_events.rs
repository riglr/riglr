//! Solana-specific event implementations that bridge between old and new traits.
//!
//! This module provides event types that implement both the legacy UnifiedEvent trait
//! and the new riglr-events-core Event trait for seamless migration.

use std::any::Any;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use chrono::Utc;
// UnifiedEvent trait has been removed
use crate::types::{EventMetadata, EventType, ProtocolType, TransferData};
use riglr_events_core::prelude::*;

/// Parameters for creating swap events, reducing function parameter count
#[derive(Debug, Clone)]
pub struct SwapEventParams {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub protocol_type: ProtocolType,
    pub program_id: solana_sdk::pubkey::Pubkey,
    pub input_mint: solana_sdk::pubkey::Pubkey,
    pub output_mint: solana_sdk::pubkey::Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
}

/// Parameters for creating liquidity events, reducing function parameter count
#[derive(Debug, Clone)]
pub struct LiquidityEventParams {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub protocol_type: ProtocolType,
    pub program_id: solana_sdk::pubkey::Pubkey,
    pub is_add: bool,
    pub mint_a: solana_sdk::pubkey::Pubkey,
    pub mint_b: solana_sdk::pubkey::Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
}

/// Parameters for creating protocol events, reducing function parameter count  
#[derive(Debug, Clone)]
pub struct ProtocolEventParams {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub protocol_type: ProtocolType,
    pub event_type: EventType,
    pub program_id: solana_sdk::pubkey::Pubkey,
    pub data: serde_json::Value,
}

/// A wrapper that implements both UnifiedEvent and Event traits for seamless migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaEvent {
    /// Legacy metadata for compatibility
    pub legacy_metadata: EventMetadata,
    /// Core metadata for new functionality  
    pub core_metadata: riglr_events_core::types::EventMetadata,
    /// Event data payload
    pub data: serde_json::Value,
    /// Transfer data for token movements
    pub transfer_data: Vec<TransferData>,
}

impl SolanaEvent {
    /// Create a new Solana event from legacy metadata
    pub fn new(legacy_metadata: EventMetadata, data: serde_json::Value) -> Self {
        let core_metadata = legacy_metadata.to_core_metadata(
            legacy_metadata.event_type.to_event_kind(),
            format!("solana-{}", legacy_metadata.protocol_type)
        );

        Self {
            legacy_metadata,
            core_metadata,
            data,
            transfer_data: Vec::new(),
        }
    }

    /// Create from both legacy and core metadata (for precise control)
    pub fn with_metadata(
        legacy_metadata: EventMetadata, 
        core_metadata: riglr_events_core::types::EventMetadata,
        data: serde_json::Value,
    ) -> Self {
        Self {
            legacy_metadata,
            core_metadata,
            data,
            transfer_data: Vec::new(),
        }
    }

    /// Set transfer data
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }

    /// Update the data payload
    pub fn with_data<T: Serialize>(mut self, data: T) -> Result<Self, serde_json::Error> {
        self.data = serde_json::to_value(data)?;
        Ok(self)
    }

    /// Extract typed data from the event
    pub fn extract_data<T>(&self) -> Result<T, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_value(self.data.clone())
    }
}


// Implement the new Event trait from riglr-events-core
impl riglr_events_core::traits::Event for SolanaEvent {
    fn id(&self) -> &str {
        &self.core_metadata.id
    }

    fn kind(&self) -> &riglr_events_core::types::EventKind {
        &self.core_metadata.kind
    }

    fn metadata(&self) -> &riglr_events_core::types::EventMetadata {
        &self.core_metadata
    }

    fn metadata_mut(&mut self) -> &mut riglr_events_core::types::EventMetadata {
        &mut self.core_metadata
    }

    fn timestamp(&self) -> SystemTime {
        self.core_metadata.timestamp.into()
    }

    fn source(&self) -> &str {
        &self.core_metadata.source
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn riglr_events_core::traits::Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> EventResult<serde_json::Value> {
        Ok(serde_json::json!({
            "legacy_metadata": self.legacy_metadata,
            "core_metadata": self.core_metadata,
            "data": self.data,
            "transfer_data": self.transfer_data
        }))
    }
}

/// Helper trait for converting legacy events to Solana events
pub trait ToSolanaEvent {
    fn to_solana_event(&self) -> SolanaEvent;
}

/// Macro to easily implement ToSolanaEvent for existing event types
#[macro_export]
macro_rules! impl_to_solana_event {
    ($event_type:ty, $data_extractor:expr) => {
        impl ToSolanaEvent for $event_type {
            fn to_solana_event(&self) -> SolanaEvent {
                let data = $data_extractor(self);
                SolanaEvent::new(self.metadata.clone(), data)
                    .with_transfer_data(self.transfer_data.clone())
            }
        }
    };
}

/// Convenience functions for creating Solana events from protocol-specific events
impl SolanaEvent {
    /// Create a swap event
    pub fn swap(params: SwapEventParams) -> Self {
        let legacy_metadata = EventMetadata::new(
            params.id.clone(),
            params.signature,
            params.slot,
            params.block_time,
            params.block_time * 1000,
            params.protocol_type,
            EventType::Swap,
            params.program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        let swap_data = serde_json::json!({
            "input_mint": params.input_mint.to_string(),
            "output_mint": params.output_mint.to_string(),
            "amount_in": params.amount_in,
            "amount_out": params.amount_out,
        });

        Self::new(legacy_metadata, swap_data)
    }

    /// Create a liquidity event
    pub fn liquidity(params: LiquidityEventParams) -> Self {
        let event_type = if params.is_add { EventType::AddLiquidity } else { EventType::RemoveLiquidity };
        
        let legacy_metadata = EventMetadata::new(
            params.id.clone(),
            params.signature,
            params.slot,
            params.block_time,
            params.block_time * 1000,
            params.protocol_type,
            event_type,
            params.program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        let liquidity_data = serde_json::json!({
            "is_add": params.is_add,
            "mint_a": params.mint_a.to_string(),
            "mint_b": params.mint_b.to_string(),
            "amount_a": params.amount_a,
            "amount_b": params.amount_b,
        });

        Self::new(legacy_metadata, liquidity_data)
    }

    /// Create a generic protocol event
    pub fn protocol_event(params: ProtocolEventParams) -> Self {
        let legacy_metadata = EventMetadata::new(
            params.id.clone(),
            params.signature,
            params.slot,
            params.block_time,
            params.block_time * 1000,
            params.protocol_type,
            params.event_type,
            params.program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        Self::new(legacy_metadata, params.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_solana_event_creation() {
        let program_id = solana_sdk::pubkey::Pubkey::new_unique();
        let legacy_metadata = EventMetadata::new(
            "test-event".to_string(),
            "signature123".to_string(),
            12345,
            1234567890,
            1234567890000,
            ProtocolType::Jupiter,
            EventType::Swap,
            program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        let event_data = json!({
            "input_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "output_mint": "So11111111111111111111111111111111111111112",
            "amount_in": 1000000,
            "amount_out": 500000
        });

        let event = SolanaEvent::new(legacy_metadata.clone(), event_data.clone());

        // Test legacy fields
        assert_eq!(event.legacy_metadata.id, "test-event");
        assert_eq!(event.legacy_metadata.event_type, EventType::Swap);
        assert_eq!(event.legacy_metadata.signature, "signature123");
        assert_eq!(event.legacy_metadata.slot, 12345);

        // Test Event interface  
        assert_eq!(riglr_events_core::traits::Event::id(&event), "test-event");
        assert_eq!(event.kind(), &riglr_events_core::types::EventKind::Swap);
        assert_eq!(event.source(), "solana-Jupiter");

        // Test data extraction
        let extracted_data: serde_json::Value = event.extract_data().unwrap();
        assert_eq!(extracted_data, event_data);
    }

    #[test]
    fn test_swap_convenience_function() {
        let program_id = solana_sdk::pubkey::Pubkey::new_unique();
        let input_mint = solana_sdk::pubkey::Pubkey::new_unique();
        let output_mint = solana_sdk::pubkey::Pubkey::new_unique();

        let event = SolanaEvent::swap(SwapEventParams {
            id: "swap-test".to_string(),
            signature: "sig456".to_string(),
            slot: 67890,
            block_time: 1234567890,
            protocol_type: ProtocolType::RaydiumAmmV4,
            program_id,
            input_mint,
            output_mint,
            amount_in: 1000000,
            amount_out: 900000,
        });

        assert_eq!(event.legacy_metadata.id, "swap-test");
        assert_eq!(event.legacy_metadata.event_type, EventType::Swap);
        assert_eq!(event.legacy_metadata.signature, "sig456");
        assert_eq!(event.legacy_metadata.slot, 67890);
        assert_eq!(event.legacy_metadata.protocol_type, ProtocolType::RaydiumAmmV4);

        let swap_data = event.extract_data::<serde_json::Value>().unwrap();
        assert_eq!(swap_data["amount_in"], 1000000);
        assert_eq!(swap_data["amount_out"], 900000);
    }

    #[test]
    fn test_liquidity_convenience_function() {
        let program_id = solana_sdk::pubkey::Pubkey::new_unique();
        let mint_a = solana_sdk::pubkey::Pubkey::new_unique();
        let mint_b = solana_sdk::pubkey::Pubkey::new_unique();

        let event = SolanaEvent::liquidity(LiquidityEventParams {
            id: "liq-test".to_string(),
            signature: "sig789".to_string(),
            slot: 11111,
            block_time: 1234567890,
            protocol_type: ProtocolType::OrcaWhirlpool,
            program_id,
            is_add: true,
            mint_a,
            mint_b,
            amount_a: 500000,
            amount_b: 250000,
        });

        assert_eq!(event.legacy_metadata.id, "liq-test");
        assert_eq!(event.legacy_metadata.event_type, EventType::AddLiquidity);
        assert_eq!(event.legacy_metadata.signature, "sig789");
        assert_eq!(event.legacy_metadata.slot, 11111);

        let liq_data = event.extract_data::<serde_json::Value>().unwrap();
        assert_eq!(liq_data["is_add"], true);
        assert_eq!(liq_data["amount_a"], 500000);
        assert_eq!(liq_data["amount_b"], 250000);
    }

    #[test]
    fn test_transfer_data_handling() {
        let program_id = solana_sdk::pubkey::Pubkey::new_unique();
        let mut event = SolanaEvent::swap(SwapEventParams {
            id: "transfer-test".to_string(),
            signature: "sig999".to_string(),
            slot: 22222,
            block_time: 1234567890,
            protocol_type: ProtocolType::Jupiter,
            program_id,
            input_mint: solana_sdk::pubkey::Pubkey::new_unique(),
            output_mint: solana_sdk::pubkey::Pubkey::new_unique(),
            amount_in: 1000000,
            amount_out: 900000,
        });

        let transfer_data = vec![
            TransferData {
                source: solana_sdk::pubkey::Pubkey::new_unique(),
                destination: solana_sdk::pubkey::Pubkey::new_unique(),
                mint: Some(solana_sdk::pubkey::Pubkey::new_unique()),
                amount: 1000000,
            }
        ];

        let _swap_data = crate::SwapData {
            input_mint: solana_sdk::pubkey::Pubkey::new_unique(),
            output_mint: solana_sdk::pubkey::Pubkey::new_unique(),
            amount_in: 1000000,
            amount_out: 900000,
        };

        // Test transfer data assignment
        event.transfer_data = transfer_data.clone();
        assert_eq!(event.transfer_data.len(), 1);
    }

    #[test]
    fn test_json_serialization() {
        let program_id = solana_sdk::pubkey::Pubkey::new_unique();
        let event = SolanaEvent::swap(SwapEventParams {
            id: "json-test".to_string(),
            signature: "sigABC".to_string(),
            slot: 33333,
            block_time: 1234567890,
            protocol_type: ProtocolType::MeteoraDlmm,
            program_id,
            input_mint: solana_sdk::pubkey::Pubkey::new_unique(),
            output_mint: solana_sdk::pubkey::Pubkey::new_unique(),
            amount_in: 2000000,
            amount_out: 1800000,
        });

        // Test new Event trait to_json method
        let json_result = event.to_json();
        assert!(json_result.is_ok());

        let json = json_result.unwrap();
        assert!(json.get("legacy_metadata").is_some());
        assert!(json.get("core_metadata").is_some());
        assert!(json.get("data").is_some());
        assert!(json.get("transfer_data").is_some());

        // Test serde serialization
        let serialized = serde_json::to_string(&event);
        assert!(serialized.is_ok());

        let deserialized: Result<SolanaEvent, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
    }
}