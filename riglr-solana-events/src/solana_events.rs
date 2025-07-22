//! Solana-specific event implementations that bridge between old and new traits.
//!
//! This module provides event types that implement both the legacy UnifiedEvent trait
//! and the new riglr-events-core Event trait for seamless migration.

use std::any::Any;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use crate::events::core::traits::UnifiedEvent;
use crate::types::{EventMetadata, EventType, ProtocolType, TransferData, SwapData};
use riglr_events_core::prelude::*;

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

// Implement the legacy UnifiedEvent trait for backward compatibility
impl UnifiedEvent for SolanaEvent {
    fn id(&self) -> &str {
        &self.legacy_metadata.id
    }

    fn event_type(&self) -> EventType {
        self.legacy_metadata.event_type.clone()
    }

    fn signature(&self) -> &str {
        &self.legacy_metadata.signature
    }

    fn slot(&self) -> u64 {
        self.legacy_metadata.slot
    }

    fn program_received_time_ms(&self) -> i64 {
        self.legacy_metadata.program_received_time_ms
    }

    fn program_handle_time_consuming_ms(&self) -> i64 {
        self.legacy_metadata.program_handle_time_consuming_ms
    }

    fn set_program_handle_time_consuming_ms(&mut self, time: i64) {
        self.legacy_metadata.program_handle_time_consuming_ms = time;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn UnifiedEvent> {
        Box::new(self.clone())
    }

    fn set_transfer_data(&mut self, transfer_data: Vec<TransferData>, swap_data: Option<SwapData>) {
        self.transfer_data = transfer_data;
        
        // Update the data payload with swap information if provided
        if let Some(swap_data) = swap_data {
            if let Ok(mut data_obj) = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(self.data.clone()) {
                data_obj.insert("swap_data".to_string(), serde_json::to_value(swap_data).unwrap_or(serde_json::Value::Null));
                self.data = serde_json::Value::Object(data_obj);
            }
        }
    }

    fn index(&self) -> String {
        self.legacy_metadata.index.clone()
    }

    fn protocol_type(&self) -> ProtocolType {
        self.legacy_metadata.protocol_type.clone()
    }

    fn timestamp(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(self.legacy_metadata.program_received_time_ms as u64)
    }

    fn transaction_hash(&self) -> Option<String> {
        Some(self.legacy_metadata.signature.clone())
    }

    fn block_number(&self) -> Option<u64> {
        Some(self.legacy_metadata.slot)
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
    pub fn swap(
        id: String,
        signature: String,
        slot: u64,
        block_time: i64,
        protocol_type: ProtocolType,
        program_id: solana_sdk::pubkey::Pubkey,
        input_mint: solana_sdk::pubkey::Pubkey,
        output_mint: solana_sdk::pubkey::Pubkey,
        amount_in: u64,
        amount_out: u64,
    ) -> Self {
        let legacy_metadata = EventMetadata::new(
            id.clone(),
            signature,
            slot,
            block_time,
            block_time * 1000,
            protocol_type,
            EventType::Swap,
            program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        let swap_data = serde_json::json!({
            "input_mint": input_mint.to_string(),
            "output_mint": output_mint.to_string(),
            "amount_in": amount_in,
            "amount_out": amount_out,
        });

        Self::new(legacy_metadata, swap_data)
    }

    /// Create a liquidity event
    pub fn liquidity(
        id: String,
        signature: String,
        slot: u64,
        block_time: i64,
        protocol_type: ProtocolType,
        program_id: solana_sdk::pubkey::Pubkey,
        is_add: bool,
        mint_a: solana_sdk::pubkey::Pubkey,
        mint_b: solana_sdk::pubkey::Pubkey,
        amount_a: u64,
        amount_b: u64,
    ) -> Self {
        let event_type = if is_add { EventType::AddLiquidity } else { EventType::RemoveLiquidity };
        
        let legacy_metadata = EventMetadata::new(
            id.clone(),
            signature,
            slot,
            block_time,
            block_time * 1000,
            protocol_type,
            event_type,
            program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        let liquidity_data = serde_json::json!({
            "is_add": is_add,
            "mint_a": mint_a.to_string(),
            "mint_b": mint_b.to_string(),
            "amount_a": amount_a,
            "amount_b": amount_b,
        });

        Self::new(legacy_metadata, liquidity_data)
    }

    /// Create a generic protocol event
    pub fn protocol_event(
        id: String,
        signature: String,
        slot: u64,
        block_time: i64,
        protocol_type: ProtocolType,
        event_type: EventType,
        program_id: solana_sdk::pubkey::Pubkey,
        data: serde_json::Value,
    ) -> Self {
        let legacy_metadata = EventMetadata::new(
            id.clone(),
            signature,
            slot,
            block_time,
            block_time * 1000,
            protocol_type,
            event_type,
            program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        Self::new(legacy_metadata, data)
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

        // Test UnifiedEvent interface
        assert_eq!(UnifiedEvent::id(&event), "test-event");
        assert_eq!(event.event_type(), EventType::Swap);
        assert_eq!(event.signature(), "signature123");
        assert_eq!(event.slot(), 12345);

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

        let event = SolanaEvent::swap(
            "swap-test".to_string(),
            "sig456".to_string(),
            67890,
            1234567890,
            ProtocolType::RaydiumAmmV4,
            program_id,
            input_mint,
            output_mint,
            1000000,
            900000,
        );

        assert_eq!(UnifiedEvent::id(&event), "swap-test");
        assert_eq!(event.event_type(), EventType::Swap);
        assert_eq!(event.signature(), "sig456");
        assert_eq!(event.slot(), 67890);
        assert_eq!(event.protocol_type(), ProtocolType::RaydiumAmmV4);

        let swap_data = event.extract_data::<serde_json::Value>().unwrap();
        assert_eq!(swap_data["amount_in"], 1000000);
        assert_eq!(swap_data["amount_out"], 900000);
    }

    #[test]
    fn test_liquidity_convenience_function() {
        let program_id = solana_sdk::pubkey::Pubkey::new_unique();
        let mint_a = solana_sdk::pubkey::Pubkey::new_unique();
        let mint_b = solana_sdk::pubkey::Pubkey::new_unique();

        let event = SolanaEvent::liquidity(
            "liq-test".to_string(),
            "sig789".to_string(),
            11111,
            1234567890,
            ProtocolType::OrcaWhirlpool,
            program_id,
            true, // is_add
            mint_a,
            mint_b,
            500000,
            250000,
        );

        assert_eq!(UnifiedEvent::id(&event), "liq-test");
        assert_eq!(event.event_type(), EventType::AddLiquidity);
        assert_eq!(event.signature(), "sig789");
        assert_eq!(event.slot(), 11111);

        let liq_data = event.extract_data::<serde_json::Value>().unwrap();
        assert_eq!(liq_data["is_add"], true);
        assert_eq!(liq_data["amount_a"], 500000);
        assert_eq!(liq_data["amount_b"], 250000);
    }

    #[test]
    fn test_transfer_data_handling() {
        let program_id = solana_sdk::pubkey::Pubkey::new_unique();
        let mut event = SolanaEvent::swap(
            "transfer-test".to_string(),
            "sig999".to_string(),
            22222,
            1234567890,
            ProtocolType::Jupiter,
            program_id,
            solana_sdk::pubkey::Pubkey::new_unique(),
            solana_sdk::pubkey::Pubkey::new_unique(),
            1000000,
            900000,
        );

        let transfer_data = vec![
            TransferData {
                source: solana_sdk::pubkey::Pubkey::new_unique(),
                destination: solana_sdk::pubkey::Pubkey::new_unique(),
                mint: Some(solana_sdk::pubkey::Pubkey::new_unique()),
                amount: 1000000,
            }
        ];

        let swap_data = SwapData {
            input_mint: solana_sdk::pubkey::Pubkey::new_unique(),
            output_mint: solana_sdk::pubkey::Pubkey::new_unique(),
            amount_in: 1000000,
            amount_out: 900000,
        };

        // Test UnifiedEvent interface
        event.set_transfer_data(transfer_data.clone(), Some(swap_data));
        assert_eq!(event.transfer_data.len(), 1);

        // Verify swap data was added to the event data
        let event_data = event.extract_data::<serde_json::Value>().unwrap();
        assert!(event_data.get("swap_data").is_some());
    }

    #[test]
    fn test_json_serialization() {
        let program_id = solana_sdk::pubkey::Pubkey::new_unique();
        let event = SolanaEvent::swap(
            "json-test".to_string(),
            "sigABC".to_string(),
            33333,
            1234567890,
            ProtocolType::MeteoraDlmm,
            program_id,
            solana_sdk::pubkey::Pubkey::new_unique(),
            solana_sdk::pubkey::Pubkey::new_unique(),
            2000000,
            1800000,
        );

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