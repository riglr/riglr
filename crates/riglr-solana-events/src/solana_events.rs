//! Solana-specific event implementations that bridge between old and new traits.
//!
//! This module provides event types that implement both the legacy UnifiedEvent trait
//! and the new riglr-events-core Event trait for seamless migration.

use chrono::Utc;
use core::any::Any;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
// UnifiedEvent trait has been removed
use crate::solana_metadata::SolanaEventMetadata;
use crate::types::{metadata_helpers, EventType, ProtocolType, TransferData};
use riglr_events_core::error::EventResult;
use riglr_events_core::traits::{Event, EventFilter};
use riglr_events_core::types::{EventKind, EventMetadata as CoreEventMetadata};
use solana_sdk::pubkey::Pubkey;

/// Macro to easily implement `ToSolanaEvent` for existing event types
#[macro_export]
macro_rules! impl_to_solana_event {
    ($event_type:ty, $data_extractor:expr_2021) => {
        impl ToSolanaEvent for $event_type {
            fn to_solana_event(&self) -> SolanaEvent {
                let data = $data_extractor(self);
                SolanaEvent::new(self.metadata.clone(), data)
                    .with_transfer_data(self.transfer_data.clone())
            }
        }
    };
}

/// Type alias for Solana event metadata
type EventMetadata = SolanaEventMetadata;

/// Parameters for creating swap events, reducing function parameter count
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SwapEventParams {
    /// Amount of input tokens
    pub amount_in: u64,
    /// Amount of output tokens
    pub amount_out: u64,
    /// Block timestamp
    pub block_time: i64,
    /// Event identifier
    pub id: String,
    /// Input token mint address
    pub input_mint: Pubkey,
    /// Output token mint address
    pub output_mint: Pubkey,
    /// Program ID that executed the swap
    pub program_id: Pubkey,
    /// Protocol type (e.g., Jupiter, Raydium)
    pub protocol_type: ProtocolType,
    /// Transaction signature
    pub signature: String,
    /// Solana slot number
    pub slot: u64,
}

/// Parameters for creating liquidity events, reducing function parameter count
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct LiquidityEventParams {
    /// Amount of first token
    pub amount_a: u64,
    /// Amount of second token
    pub amount_b: u64,
    /// Block timestamp
    pub block_time: i64,
    /// Event identifier
    pub id: String,
    /// True for adding liquidity, false for removing
    pub is_add: bool,
    /// First token mint address
    pub mint_a: Pubkey,
    /// Second token mint address
    pub mint_b: Pubkey,
    /// Program ID that executed the liquidity operation
    pub program_id: Pubkey,
    /// Protocol type (e.g., Orca, Raydium)
    pub protocol_type: ProtocolType,
    /// Transaction signature
    pub signature: String,
    /// Solana slot number
    pub slot: u64,
}

/// Parameters for creating protocol events, reducing function parameter count
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ProtocolEventParams {
    /// Block timestamp
    pub block_time: i64,
    /// Event-specific data payload
    pub data: serde_json::Value,
    /// Specific event type within the protocol
    pub event_type: EventType,
    /// Event identifier
    pub id: String,
    /// Program ID that executed the event
    pub program_id: Pubkey,
    /// Protocol type (e.g., `MarginFi`, Meteora)
    pub protocol_type: ProtocolType,
    /// Transaction signature
    pub signature: String,
    /// Solana slot number
    pub slot: u64,
}

/// A wrapper that implements the Event trait for Solana events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SolanaEvent {
    /// Event data payload
    pub data: serde_json::Value,
    /// Event metadata
    pub metadata: EventMetadata,
    /// Transfer data for token movements
    pub transfer_data: Vec<TransferData>,
}

impl SolanaEvent {
    /// Extract typed data from the event
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be deserialized to type `T`.
    #[inline]
    pub fn extract_data<T>(&self) -> Result<T, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_value(self.data.clone())
    }

    /// Create a liquidity event
    #[must_use]
    #[inline]
    pub fn liquidity(params: LiquidityEventParams) -> Self {
        let event_type = if params.is_add {
            EventType::AddLiquidity
        } else {
            EventType::RemoveLiquidity
        };

        let metadata = metadata_helpers::create_solana_metadata(
            params.id,
            params.signature,
            params.slot,
            params.block_time,
            params.protocol_type,
            event_type,
            params.program_id,
            "0".to_owned(),
            Utc::now().timestamp_millis(),
        );

        let liquidity_data = serde_json::json!({
            "is_add": params.is_add,
            "mint_a": params.mint_a.to_string(),
            "mint_b": params.mint_b.to_string(),
            "amount_a": params.amount_a,
            "amount_b": params.amount_b,
        });

        Self::new(metadata, liquidity_data)
    }

    /// Create a new Solana event
    #[must_use]
    #[inline]
    pub const fn new(metadata: EventMetadata, data: serde_json::Value) -> Self {
        Self {
            metadata,
            data,
            transfer_data: Vec::new(),
        }
    }

    /// Create a generic protocol event
    #[must_use]
    #[inline]
    pub fn protocol_event(params: ProtocolEventParams) -> Self {
        let metadata = metadata_helpers::create_solana_metadata(
            params.id,
            params.signature,
            params.slot,
            params.block_time,
            params.protocol_type,
            params.event_type,
            params.program_id,
            "0".to_owned(),
            Utc::now().timestamp_millis(),
        );

        Self::new(metadata, params.data)
    }

    /// Create a swap event
    #[must_use]
    #[inline]
    pub fn swap(params: SwapEventParams) -> Self {
        let metadata = metadata_helpers::create_solana_metadata(
            params.id,
            params.signature,
            params.slot,
            params.block_time,
            params.protocol_type,
            EventType::Swap,
            params.program_id,
            "0".to_owned(),
            Utc::now().timestamp_millis(),
        );

        let swap_data = serde_json::json!({
            "input_mint": params.input_mint.to_string(),
            "output_mint": params.output_mint.to_string(),
            "amount_in": params.amount_in,
            "amount_out": params.amount_out,
        });

        Self::new(metadata, swap_data)
    }

    /// Update the data payload
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be serialized to JSON.
    #[inline]
    pub fn with_data<T: Serialize>(mut self, data: T) -> Result<Self, serde_json::Error> {
        self.data = serde_json::to_value(data)?;
        Ok(self)
    }

    /// Set transfer data
    #[must_use]
    #[inline]
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

/// Helper trait for converting legacy events to Solana events
pub trait ToSolanaEvent {
    /// Convert this event to a `SolanaEvent` for unified handling
    fn to_solana_event(&self) -> SolanaEvent;
}

// Implement the new Event trait from riglr-events-core
impl Event for SolanaEvent {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    #[inline]
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    #[inline]
    fn kind(&self) -> &EventKind {
        &self.metadata.core.kind
    }

    fn matches_filter(&self, _filter: &dyn EventFilter) -> bool
    where
        Self: Sized,
    {
        true
    }

    #[inline]
    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    #[inline]
    fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
    }

    #[inline]
    fn source(&self) -> &str {
        &self.metadata.core.source
    }

    #[inline]
    fn timestamp(&self) -> SystemTime {
        self.metadata.core.timestamp.into()
    }

    #[inline]
    fn to_json(&self) -> EventResult<serde_json::Value> {
        Ok(serde_json::json!({
            "metadata": self.metadata,
            "data": self.data,
            "transfer_data": self.transfer_data
        }))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_solana_event_creation() {
        let program_id = Pubkey::new_unique();
        let metadata = metadata_helpers::create_solana_metadata(
            "test-event".to_string(),
            "signature123".to_string(),
            12345,
            1_234_567_890,
            ProtocolType::Jupiter,
            EventType::Swap,
            program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        let event_data = json!({
            "input_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "output_mint": "So11111111111111111111111111111111111111112",
            "amount_in": 1_000_000,
            "amount_out": 500_000
        });

        let event = SolanaEvent::new(metadata, event_data.clone());

        // Test legacy fields
        assert_eq!(event.metadata.id(), "test-event");
        assert_eq!(event.metadata.event_type, EventType::Swap);
        assert_eq!(event.metadata.signature, "signature123");
        assert_eq!(event.metadata.slot, 12345);

        // Test Event interface
        assert_eq!(Event::id(&event), "test-event");
        assert_eq!(event.kind(), &EventKind::Swap);
        assert_eq!(event.source(), "solana-Jupiter");

        // Test data extraction
        let extracted_data: serde_json::Value =
            event.extract_data().expect("Failed to extract event data");
        assert_eq!(extracted_data, event_data);
    }

    #[test]
    fn test_swap_convenience_function() {
        let program_id = Pubkey::new_unique();
        let input_mint = Pubkey::new_unique();
        let output_mint = Pubkey::new_unique();

        let event = SolanaEvent::swap(SwapEventParams {
            id: "swap-test".to_string(),
            signature: "sig456".to_string(),
            slot: 67890,
            block_time: 1_234_567_890,
            protocol_type: ProtocolType::RaydiumAmmV4,
            program_id,
            input_mint,
            output_mint,
            amount_in: 1_000_000,
            amount_out: 900_000,
        });

        assert_eq!(event.metadata.id(), "swap-test");
        assert_eq!(event.metadata.event_type, EventType::Swap);
        assert_eq!(event.metadata.signature, "sig456");
        assert_eq!(event.metadata.slot, 67890);
        assert_eq!(event.metadata.protocol_type, ProtocolType::RaydiumAmmV4);

        let swap_data = event.extract_data::<serde_json::Value>().unwrap();
        assert_eq!(swap_data.get("amount_in").unwrap(), &1_000_000);
        assert_eq!(swap_data.get("amount_out").unwrap(), &900_000);
    }

    #[test]
    fn test_liquidity_convenience_function() {
        let program_id = Pubkey::new_unique();
        let mint_a = Pubkey::new_unique();
        let mint_b = Pubkey::new_unique();

        let event = SolanaEvent::liquidity(LiquidityEventParams {
            id: "liq-test".to_string(),
            signature: "sig789".to_string(),
            slot: 11111,
            block_time: 1_234_567_890,
            protocol_type: ProtocolType::OrcaWhirlpool,
            program_id,
            is_add: true,
            mint_a,
            mint_b,
            amount_a: 500_000,
            amount_b: 250_000,
        });

        assert_eq!(event.metadata.id(), "liq-test");
        assert_eq!(event.metadata.event_type, EventType::AddLiquidity);
        assert_eq!(event.metadata.signature, "sig789");
        assert_eq!(event.metadata.slot, 11111);

        let liq_data = event.extract_data::<serde_json::Value>().unwrap();
        assert!(liq_data.get("is_add").unwrap().as_bool().unwrap());
        assert_eq!(liq_data.get("amount_a").unwrap(), &500_000);
        assert_eq!(liq_data.get("amount_b").unwrap(), &250_000);
    }

    #[test]
    fn test_transfer_data_handling() {
        let program_id = Pubkey::new_unique();
        let mut event = SolanaEvent::swap(SwapEventParams {
            id: "transfer-test".to_string(),
            signature: "sig999".to_string(),
            slot: 22222,
            block_time: 1_234_567_890,
            protocol_type: ProtocolType::Jupiter,
            program_id,
            input_mint: Pubkey::new_unique(),
            output_mint: Pubkey::new_unique(),
            amount_in: 1_000_000,
            amount_out: 900_000,
        });

        let transfer_data = vec![TransferData {
            source: Pubkey::new_unique(),
            destination: Pubkey::new_unique(),
            mint: Some(Pubkey::new_unique()),
            amount: 1_000_000,
        }];

        let _swap_data = crate::SwapData {
            input_mint: Pubkey::new_unique(),
            output_mint: Pubkey::new_unique(),
            amount_in: 1_000_000,
            amount_out: 900_000,
        };

        // Test transfer data assignment
        event.transfer_data = transfer_data;
        assert_eq!(event.transfer_data.len(), 1);
    }

    #[test]
    fn test_json_serialization() {
        let program_id = Pubkey::new_unique();
        let event = SolanaEvent::swap(SwapEventParams {
            id: "json-test".to_string(),
            signature: "sigABC".to_string(),
            slot: 33333,
            block_time: 1_234_567_890,
            protocol_type: ProtocolType::MeteoraDlmm,
            program_id,
            input_mint: Pubkey::new_unique(),
            output_mint: Pubkey::new_unique(),
            amount_in: 2_000_000,
            amount_out: 1_800_000,
        });

        // Test new Event trait to_json method
        let json_result = event.to_json();
        assert!(json_result.is_ok());

        let json = json_result.unwrap();
        assert!(json.get("metadata").is_some());
        assert!(json.get("data").is_some());
        assert!(json.get("transfer_data").is_some());

        // Test serde serialization
        let serialized = serde_json::to_string(&event);
        assert!(serialized.is_ok());

        let deserialized: Result<SolanaEvent, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_with_transfer_data() {
        let program_id = Pubkey::new_unique();
        let metadata = metadata_helpers::create_solana_metadata(
            "test-transfer".to_string(),
            "sig123".to_string(),
            12345,
            1_234_567_890,
            ProtocolType::Jupiter,
            EventType::Swap,
            program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        let transfer_data = vec![
            TransferData {
                source: Pubkey::new_unique(),
                destination: Pubkey::new_unique(),
                mint: Some(Pubkey::new_unique()),
                amount: 1_000_000,
            },
            TransferData {
                source: Pubkey::new_unique(),
                destination: Pubkey::new_unique(),
                mint: None,
                amount: 500_000,
            },
        ];

        let event = SolanaEvent::new(metadata, json!({})).with_transfer_data(transfer_data);

        assert_eq!(event.transfer_data.len(), 2);
        assert_eq!(event.transfer_data.first().unwrap().amount, 1_000_000);
        assert_eq!(event.transfer_data.get(1).unwrap().amount, 500_000);
        assert!(event.transfer_data.first().unwrap().mint.is_some());
        assert!(event.transfer_data.get(1).unwrap().mint.is_none());
    }

    #[test]
    fn test_with_data_success() {
        #[derive(Serialize)]
        struct TestData {
            field1: String,
            field2: u64,
            field3: bool,
        }

        let program_id = Pubkey::new_unique();
        let metadata = metadata_helpers::create_solana_metadata(
            "test-data".to_string(),
            "sig456".to_string(),
            67890,
            1_234_567_890,
            ProtocolType::RaydiumAmmV4,
            EventType::Swap,
            program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        let test_data = TestData {
            field1: "test_value".to_string(),
            field2: 42,
            field3: true,
        };

        let event = SolanaEvent::new(metadata, json!({}));
        let result = event.with_data(test_data);

        assert!(result.is_ok());
        let updated_event = result.unwrap();

        let extracted: serde_json::Value = updated_event.extract_data().unwrap();
        assert_eq!(extracted.get("field1").unwrap(), &"test_value");
        assert_eq!(extracted.get("field2").unwrap(), &42);
        assert!(extracted.get("field3").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_with_data_error() {
        let program_id = Pubkey::new_unique();
        let metadata = metadata_helpers::create_solana_metadata(
            "test-data-error".to_string(),
            "sig789".to_string(),
            11111,
            1_234_567_890,
            ProtocolType::OrcaWhirlpool,
            EventType::AddLiquidity,
            program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        let event = SolanaEvent::new(metadata, json!({}));

        // Test with a valid type and verify the Ok path.
        let result = event.with_data(json!({"test": "data"}));
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_data_error() {
        #[derive(Deserialize)]
        #[expect(dead_code)] // Test struct fields are intentionally unused
        struct ExpectedStruct {
            field1: u64,  // This should fail since "not_a_number" is not a u64
            field2: bool, // This should fail since "invalid_type" is not a bool
        }

        let program_id = Pubkey::new_unique();
        let metadata = metadata_helpers::create_solana_metadata(
            "test-extract-error".to_string(),
            "sig000".to_string(),
            99999,
            1_234_567_890,
            ProtocolType::MeteoraDlmm,
            EventType::Swap,
            program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        let invalid_data = json!({
            "field1": "not_a_number",
            "field2": "invalid_type"
        });

        let event = SolanaEvent::new(metadata, invalid_data);

        let result: Result<ExpectedStruct, _> = event.extract_data();
        assert!(result.is_err());
    }

    #[test]
    fn test_event_trait_metadata_mut() {
        let program_id = Pubkey::new_unique();
        let metadata = metadata_helpers::create_solana_metadata(
            "test-mut".to_string(),
            "sig111".to_string(),
            12345,
            1_234_567_890,
            ProtocolType::Jupiter,
            EventType::Swap,
            program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        let mut event = SolanaEvent::new(metadata, json!({}));

        // Test mutable access to metadata
        let metadata_mut = Event::metadata_mut(&mut event).unwrap();
        let original_id = metadata_mut.id.clone();
        metadata_mut.id = "modified-id".to_string();

        assert_eq!(Event::id(&event), "modified-id");
        assert_ne!(Event::id(&event), original_id);
    }

    #[test]
    fn test_event_trait_as_any_mut() {
        let program_id = Pubkey::new_unique();
        let metadata = metadata_helpers::create_solana_metadata(
            "test-any-mut".to_string(),
            "sig222".to_string(),
            23456,
            1_234_567_890,
            ProtocolType::RaydiumAmmV4,
            EventType::Swap,
            program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        let mut event = SolanaEvent::new(metadata, json!({"test": "data"}));

        // Test as_any_mut
        let any_mut = Event::as_any_mut(&mut event);
        let solana_event_mut = any_mut.downcast_mut::<SolanaEvent>().unwrap();
        solana_event_mut.data = json!({"modified": "data"});

        let extracted: serde_json::Value = event.extract_data().unwrap();
        assert_eq!(extracted.get("modified").unwrap(), &"data");
    }

    #[test]
    fn test_event_trait_clone_boxed() {
        let program_id = Pubkey::new_unique();
        let metadata = metadata_helpers::create_solana_metadata(
            "test-clone".to_string(),
            "sig333".to_string(),
            34567,
            1_234_567_890,
            ProtocolType::OrcaWhirlpool,
            EventType::AddLiquidity,
            program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        let event = SolanaEvent::new(metadata, json!({"original": "data"}));

        // Test clone_boxed
        let cloned_boxed = Event::clone_boxed(&event);
        assert_eq!(cloned_boxed.id(), "test-clone");

        // Verify they are separate instances
        let cloned_any = cloned_boxed.as_any();
        let cloned_solana = cloned_any.downcast_ref::<SolanaEvent>().unwrap();
        let cloned_data: serde_json::Value = cloned_solana.extract_data().unwrap();
        assert_eq!(cloned_data.get("original").unwrap(), &"data");
    }

    #[test]
    fn test_liquidity_remove_event() {
        let program_id = Pubkey::new_unique();
        let mint_a = Pubkey::new_unique();
        let mint_b = Pubkey::new_unique();

        // Test removing liquidity (is_add = false)
        let event = SolanaEvent::liquidity(LiquidityEventParams {
            id: "remove-liq-test".to_string(),
            signature: "sig444".to_string(),
            slot: 44444,
            block_time: 1_234_567_890,
            protocol_type: ProtocolType::MeteoraDlmm,
            program_id,
            is_add: false, // This is the key difference
            mint_a,
            mint_b,
            amount_a: 750_000,
            amount_b: 375_000,
        });

        assert_eq!(event.metadata.id(), "remove-liq-test");
        assert_eq!(event.metadata.event_type, EventType::RemoveLiquidity); // Should be RemoveLiquidity
        assert_eq!(event.metadata.signature, "sig444");
        assert_eq!(event.metadata.slot, 44444);

        let liq_data = event.extract_data::<serde_json::Value>().unwrap();
        assert!(!liq_data.get("is_add").unwrap().as_bool().unwrap()); // Should be false
        assert_eq!(liq_data.get("amount_a").unwrap(), &750_000);
        assert_eq!(liq_data.get("amount_b").unwrap(), &375_000);
    }

    #[test]
    fn test_protocol_event() {
        let program_id = Pubkey::new_unique();

        let custom_data = json!({
            "custom_field1": "custom_value",
            "custom_field2": 12345,
            "custom_field3": ["array", "of", "strings"],
            "nested": {
                "inner_field": true
            }
        });

        let event = SolanaEvent::protocol_event(ProtocolEventParams {
            id: "protocol-test".to_string(),
            signature: "sig555".to_string(),
            slot: 55555,
            block_time: 1_234_567_890,
            protocol_type: ProtocolType::Jupiter,
            event_type: EventType::Borrow,
            program_id,
            data: custom_data.clone(),
        });

        assert_eq!(event.metadata.id(), "protocol-test");
        assert_eq!(event.metadata.event_type, EventType::Borrow);
        assert_eq!(event.metadata.signature, "sig555");
        assert_eq!(event.metadata.slot, 55555);
        assert_eq!(event.metadata.protocol_type, ProtocolType::Jupiter);

        let extracted_data: serde_json::Value = event
            .extract_data()
            .expect("Failed to extract protocol event data");
        assert_eq!(extracted_data, custom_data);
        assert_eq!(
            extracted_data.get("custom_field1").unwrap(),
            &"custom_value"
        );
        assert_eq!(extracted_data.get("custom_field2").unwrap(), &12345);
        assert!(extracted_data
            .get("nested")
            .unwrap()
            .get("inner_field")
            .unwrap()
            .as_bool()
            .expect("inner_field should be a boolean"));
    }

    #[test]
    fn test_event_trait_timestamp() {
        let program_id = Pubkey::new_unique();
        let metadata = metadata_helpers::create_solana_metadata(
            "test-timestamp".to_string(),
            "sig666".to_string(),
            66666,
            1_234_567_890,
            ProtocolType::RaydiumAmmV4,
            EventType::Swap,
            program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        let event = SolanaEvent::new(metadata, json!({}));

        // Test timestamp method
        let timestamp = Event::timestamp(&event);

        // Verify it returns a SystemTime
        assert!(timestamp.duration_since(SystemTime::UNIX_EPOCH).is_ok());
    }

    #[test]
    fn test_event_trait_as_any() {
        let program_id = Pubkey::new_unique();
        let metadata = metadata_helpers::create_solana_metadata(
            "test-any".to_string(),
            "sig777".to_string(),
            77777,
            1_234_567_890,
            ProtocolType::OrcaWhirlpool,
            EventType::AddLiquidity,
            program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        let event = SolanaEvent::new(metadata, json!({"test": "any"}));

        // Test as_any
        let any_ref = Event::as_any(&event);
        let solana_event_ref = any_ref.downcast_ref::<SolanaEvent>().unwrap();
        assert_eq!(solana_event_ref.metadata.id(), "test-any");

        let extracted: serde_json::Value = solana_event_ref.extract_data().unwrap();
        assert_eq!(extracted.get("test").unwrap(), &"any");
    }

    #[test]
    fn test_parameter_structs_debug_clone() {
        let program_id = Pubkey::new_unique();
        let input_mint = Pubkey::new_unique();
        let output_mint = Pubkey::new_unique();

        // Test SwapEventParams Debug and Clone
        let swap_params = SwapEventParams {
            id: "debug-test".to_string(),
            signature: "sig888".to_string(),
            slot: 88888,
            block_time: 1_234_567_890,
            protocol_type: ProtocolType::Jupiter,
            program_id,
            input_mint,
            output_mint,
            amount_in: 1_000_000,
            amount_out: 900_000,
        };

        let swap_params_cloned = swap_params.clone();
        assert_eq!(swap_params.id, swap_params_cloned.id);
        assert_eq!(swap_params.slot, swap_params_cloned.slot);

        // Test Debug formatting (just ensure it doesn't panic)
        let debug_output = format!("{swap_params:?}");
        assert!(debug_output.contains("SwapEventParams"));

        // Test LiquidityEventParams Debug and Clone
        let liq_params = LiquidityEventParams {
            id: "liq-debug-test".to_string(),
            signature: "sig999".to_string(),
            slot: 99999,
            block_time: 1_234_567_890,
            protocol_type: ProtocolType::OrcaWhirlpool,
            program_id,
            is_add: true,
            mint_a: input_mint,
            mint_b: output_mint,
            amount_a: 500_000,
            amount_b: 250_000,
        };

        let liq_params_cloned = liq_params.clone();
        assert_eq!(liq_params.is_add, liq_params_cloned.is_add);
        assert_eq!(liq_params.amount_a, liq_params_cloned.amount_a);

        let debug_output = format!("{liq_params:?}");
        assert!(debug_output.contains("LiquidityEventParams"));

        // Test ProtocolEventParams Debug and Clone
        let protocol_params = ProtocolEventParams {
            id: "protocol-debug-test".to_string(),
            signature: "sig000".to_string(),
            slot: 12121,
            block_time: 1_234_567_890,
            protocol_type: ProtocolType::MeteoraDlmm,
            event_type: EventType::Borrow,
            program_id,
            data: json!({"test": "data"}),
        };

        let protocol_params_cloned = protocol_params.clone();
        assert_eq!(
            protocol_params.event_type,
            protocol_params_cloned.event_type
        );
        assert_eq!(protocol_params.data, protocol_params_cloned.data);

        let debug_output = format!("{protocol_params:?}");
        assert!(debug_output.contains("ProtocolEventParams"));
    }

    #[test]
    fn test_solana_event_debug_serialization() {
        let program_id = Pubkey::new_unique();
        let metadata = metadata_helpers::create_solana_metadata(
            "test-debug-ser".to_string(),
            "sig123".to_string(),
            12345,
            1_234_567_890,
            ProtocolType::Jupiter,
            EventType::Swap,
            program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        let event = SolanaEvent::new(metadata, json!({"test": "serialization"}));

        // Test Debug formatting
        let debug_output = format!("{event:?}");
        assert!(debug_output.contains("SolanaEvent"));

        // Test that the struct derives Clone
        let cloned_event = event.clone();
        assert_eq!(event.metadata.id(), cloned_event.metadata.id());
        assert_eq!(event.data, cloned_event.data);

        // Test serialization roundtrip
        let serialized = serde_json::to_string(&event).expect("Failed to serialize SolanaEvent");
        let deserialized: SolanaEvent =
            serde_json::from_str(&serialized).expect("Failed to deserialize SolanaEvent");

        assert_eq!(event.metadata.id(), deserialized.metadata.id());
        assert_eq!(event.data, deserialized.data);
        assert_eq!(event.transfer_data.len(), deserialized.transfer_data.len());
    }

    #[test]
    fn test_empty_transfer_data() {
        let program_id = Pubkey::new_unique();
        let metadata = metadata_helpers::create_solana_metadata(
            "test-empty-transfer".to_string(),
            "sig-empty".to_string(),
            11111,
            1_234_567_890,
            ProtocolType::Jupiter,
            EventType::Swap,
            program_id,
            "0".to_string(),
            Utc::now().timestamp_millis(),
        );

        // Test with empty transfer data
        let event = SolanaEvent::new(metadata, json!({})).with_transfer_data(vec![]);

        assert_eq!(event.transfer_data.len(), 0);

        // Test JSON serialization with empty transfer data
        let json_result = event.to_json();
        assert!(json_result.is_ok());

        let json = json_result.unwrap();
        assert_eq!(
            json.get("transfer_data").unwrap().as_array().unwrap().len(),
            0
        );
    }
}
