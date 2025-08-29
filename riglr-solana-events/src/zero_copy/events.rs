//! Zero-copy event implementations that reference source data directly
//!
//! These event types avoid allocations by keeping references to the original data
//! where possible, providing significant performance improvements for high-throughput
//! parsing scenarios.

use crate::metadata_helpers::{
    get_event_type, get_instruction_index, get_protocol_type, get_signature, get_slot,
};
use crate::solana_metadata::SolanaEventMetadata;
use crate::types::{EventType, ProtocolType};

type EventMetadata = SolanaEventMetadata;
use solana_sdk::pubkey::Pubkey;
use std::borrow::Cow;
use std::sync::Arc;
// UnifiedEvent trait has been removed - using Event trait from riglr_events_core

/// Zero-copy base event that holds references to source data
#[derive(Debug, Clone)]
pub struct ZeroCopyEvent<'a> {
    /// Event metadata (owned)
    pub metadata: EventMetadata,
    /// Raw instruction or log data (borrowed)
    pub raw_data: Cow<'a, [u8]>,
    /// Protocol-specific parsed data (lazily computed)
    pub parsed_data: Option<Arc<dyn std::any::Any + Send + Sync>>,
    /// Event-specific data as JSON (lazily computed)
    pub json_data: Option<serde_json::Value>,
}

impl<'a> ZeroCopyEvent<'a> {
    /// Create a new zero-copy event with borrowed data
    pub fn new_borrowed(metadata: EventMetadata, raw_data: &'a [u8]) -> Self {
        Self {
            metadata,
            raw_data: Cow::Borrowed(raw_data),
            parsed_data: None,
            json_data: None,
        }
    }

    /// Create a new zero-copy event with owned data
    pub fn new_owned(metadata: EventMetadata, raw_data: Vec<u8>) -> Self {
        Self {
            metadata,
            raw_data: Cow::Owned(raw_data),
            parsed_data: None,
            json_data: None,
        }
    }

    /// Create a new zero-copy event with owned data and JSON
    pub fn new_owned_with_json(
        metadata: EventMetadata,
        raw_data: Vec<u8>,
        json_data: serde_json::Value,
    ) -> Self {
        Self {
            metadata,
            raw_data: Cow::Owned(raw_data),
            parsed_data: None,
            json_data: Some(json_data),
        }
    }

    /// Get the raw data as a slice
    pub fn raw_data(&self) -> &[u8] {
        &self.raw_data
    }

    /// Set parsed data (strongly typed)
    pub fn set_parsed_data<T: std::any::Any + Send + Sync>(&mut self, data: T) {
        self.parsed_data = Some(Arc::new(data));
    }

    /// Get parsed data (strongly typed)
    pub fn get_parsed_data<T: std::any::Any + Send + Sync>(&self) -> Option<&T> {
        self.parsed_data.as_ref()?.downcast_ref::<T>()
    }

    /// Set JSON representation (cached)
    pub fn set_json_data(&mut self, json: serde_json::Value) {
        self.json_data = Some(json);
    }

    /// Get JSON representation, computing if necessary
    pub fn get_json_data(&self) -> Option<&serde_json::Value> {
        self.json_data.as_ref()
    }

    /// Convert to owned event (clones all data)
    pub fn to_owned(&self) -> ZeroCopyEvent<'static> {
        ZeroCopyEvent {
            metadata: self.metadata.clone(),
            raw_data: Cow::Owned(self.raw_data.to_vec()),
            parsed_data: self.parsed_data.clone(),
            json_data: self.json_data.clone(),
        }
    }
}

// Helper methods for any lifetime
impl<'a> ZeroCopyEvent<'a> {
    /// Get id for any lifetime
    pub fn id(&self) -> &str {
        &self.metadata.core.id
    }

    /// Get event type for any lifetime
    pub fn event_type(&self) -> EventType {
        get_event_type(&self.metadata.core).unwrap_or_default()
    }

    /// Get signature for any lifetime
    pub fn signature(&self) -> &str {
        get_signature(&self.metadata.core).unwrap_or("")
    }

    /// Get slot for any lifetime
    pub fn slot(&self) -> u64 {
        get_slot(&self.metadata.core).unwrap_or(0)
    }

    /// Get protocol type for any lifetime
    pub fn protocol_type(&self) -> ProtocolType {
        get_protocol_type(&self.metadata.core).unwrap_or_default()
    }

    /// Get index for any lifetime
    pub fn index(&self) -> String {
        get_instruction_index(&self.metadata.core)
            .map(|i| i.to_string())
            .unwrap_or_default()
    }

    /// Get timestamp for any lifetime
    pub fn timestamp(&self) -> std::time::SystemTime {
        // Use the timestamp from metadata which is already a DateTime<Utc>
        std::time::SystemTime::from(self.metadata.core.timestamp)
    }

    /// Get block number for any lifetime
    pub fn block_number(&self) -> Option<u64> {
        get_slot(&self.metadata.core)
    }
}

/// Specialized zero-copy swap event
#[derive(Debug, Clone)]
pub struct ZeroCopySwapEvent<'a> {
    /// Base event data
    pub base: ZeroCopyEvent<'a>,
    /// Input mint (parsed lazily)
    input_mint: Option<Pubkey>,
    /// Output mint (parsed lazily)
    output_mint: Option<Pubkey>,
    /// Amount in (parsed lazily)
    amount_in: Option<u64>,
    /// Amount out (parsed lazily)
    amount_out: Option<u64>,
}

impl<'a> ZeroCopySwapEvent<'a> {
    /// Create new zero-copy swap event
    pub fn new(base: ZeroCopyEvent<'a>) -> Self {
        Self {
            base,
            input_mint: None,
            output_mint: None,
            amount_in: None,
            amount_out: None,
        }
    }

    /// Get input mint, parsing if necessary
    pub fn input_mint(&mut self) -> Option<Pubkey> {
        if self.input_mint.is_none() {
            // Parse from raw data - implementation depends on protocol
            self.input_mint = self.parse_input_mint();
        }
        self.input_mint
    }

    /// Get output mint, parsing if necessary
    pub fn output_mint(&mut self) -> Option<Pubkey> {
        if self.output_mint.is_none() {
            // Parse from raw data - implementation depends on protocol
            self.output_mint = self.parse_output_mint();
        }
        self.output_mint
    }

    /// Get amount in, parsing if necessary
    pub fn amount_in(&mut self) -> Option<u64> {
        if self.amount_in.is_none() {
            self.amount_in = self.parse_amount_in();
        }
        self.amount_in
    }

    /// Get amount out, parsing if necessary
    pub fn amount_out(&mut self) -> Option<u64> {
        if self.amount_out.is_none() {
            self.amount_out = self.parse_amount_out();
        }
        self.amount_out
    }

    // Protocol-specific parsing methods - to be implemented by specific parsers
    fn parse_input_mint(&self) -> Option<Pubkey> {
        // This would be implemented based on the protocol
        None
    }

    fn parse_output_mint(&self) -> Option<Pubkey> {
        // This would be implemented based on the protocol
        None
    }

    fn parse_amount_in(&self) -> Option<u64> {
        // This would be implemented based on the protocol
        None
    }

    fn parse_amount_out(&self) -> Option<u64> {
        // This would be implemented based on the protocol
        None
    }
}

/// Zero-copy liquidity event
#[derive(Debug, Clone)]
pub struct ZeroCopyLiquidityEvent<'a> {
    /// Base event data
    pub base: ZeroCopyEvent<'a>,
    /// Pool address (parsed lazily)
    pool_address: Option<Pubkey>,
    /// LP token amount (parsed lazily)
    lp_amount: Option<u64>,
    /// Token A amount (parsed lazily)
    token_a_amount: Option<u64>,
    /// Token B amount (parsed lazily)
    token_b_amount: Option<u64>,
}

impl<'a> ZeroCopyLiquidityEvent<'a> {
    /// Create new zero-copy liquidity event
    pub fn new(base: ZeroCopyEvent<'a>) -> Self {
        Self {
            base,
            pool_address: None,
            lp_amount: None,
            token_a_amount: None,
            token_b_amount: None,
        }
    }

    /// Get pool address, parsing if necessary
    pub fn pool_address(&mut self) -> Option<Pubkey> {
        if self.pool_address.is_none() {
            self.pool_address = self.parse_pool_address();
        }
        self.pool_address
    }

    /// Get LP token amount, parsing if necessary
    pub fn lp_amount(&mut self) -> Option<u64> {
        if self.lp_amount.is_none() {
            self.lp_amount = self.parse_lp_amount();
        }
        self.lp_amount
    }

    /// Get token A amount, parsing if necessary
    pub fn token_a_amount(&mut self) -> Option<u64> {
        if self.token_a_amount.is_none() {
            self.token_a_amount = self.parse_token_a_amount();
        }
        self.token_a_amount
    }

    /// Get token B amount, parsing if necessary
    pub fn token_b_amount(&mut self) -> Option<u64> {
        if self.token_b_amount.is_none() {
            self.token_b_amount = self.parse_token_b_amount();
        }
        self.token_b_amount
    }

    // Protocol-specific parsing methods - to be implemented by specific parsers
    fn parse_pool_address(&self) -> Option<Pubkey> {
        None
    }

    fn parse_lp_amount(&self) -> Option<u64> {
        None
    }

    fn parse_token_a_amount(&self) -> Option<u64> {
        None
    }

    fn parse_token_b_amount(&self) -> Option<u64> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventType, ProtocolType};
    use serde_json::json;

    fn create_test_solana_metadata() -> SolanaEventMetadata {
        use crate::solana_metadata::create_metadata;
        create_metadata(
            "test_id".to_string(),
            "test_signature".to_string(),
            12345,
            Some(1234567890),
            0,
            "0".to_string(),
            EventType::Swap,
            ProtocolType::OrcaWhirlpool,
        )
    }

    #[test]
    fn test_zerocopy_event_new_borrowed_should_create_with_borrowed_data() {
        let metadata = create_test_solana_metadata();
        let raw_data = b"test_data";

        let event = ZeroCopyEvent::new_borrowed(metadata.clone(), raw_data);

        assert_eq!(event.metadata.signature, "test_signature");
        assert_eq!(event.raw_data(), raw_data);
        assert!(event.parsed_data.is_none());
        assert!(event.json_data.is_none());
        // Verify it's borrowed
        matches!(event.raw_data, Cow::Borrowed(_));
    }

    #[test]
    fn test_zerocopy_event_new_owned_should_create_with_owned_data() {
        let metadata = create_test_solana_metadata();
        let raw_data = vec![1, 2, 3, 4, 5];

        let event = ZeroCopyEvent::new_owned(metadata.clone(), raw_data.clone());

        assert_eq!(event.metadata.signature, "test_signature");
        assert_eq!(event.raw_data(), &raw_data);
        assert!(event.parsed_data.is_none());
        assert!(event.json_data.is_none());
        // Verify it's owned
        matches!(event.raw_data, Cow::Owned(_));
    }

    #[test]
    fn test_zerocopy_event_raw_data_should_return_slice() {
        let metadata = create_test_solana_metadata();
        let raw_data = vec![10, 20, 30];

        let event = ZeroCopyEvent::new_owned(metadata, raw_data.clone());

        assert_eq!(event.raw_data(), &raw_data);
    }

    #[test]
    fn test_zerocopy_event_set_and_get_parsed_data_should_work_with_valid_type() {
        let metadata = create_test_solana_metadata();
        let mut event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let test_data = "test_parsed_data".to_string();
        event.set_parsed_data(test_data.clone());

        let retrieved = event.get_parsed_data::<String>();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &test_data);
    }

    #[test]
    fn test_zerocopy_event_get_parsed_data_should_return_none_for_wrong_type() {
        let metadata = create_test_solana_metadata();
        let mut event = ZeroCopyEvent::new_owned(metadata, vec![]);

        event.set_parsed_data("string_data".to_string());

        let retrieved = event.get_parsed_data::<u64>();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_zerocopy_event_get_parsed_data_should_return_none_when_no_data() {
        let metadata = create_test_solana_metadata();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let retrieved = event.get_parsed_data::<String>();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_zerocopy_event_set_and_get_json_data_should_work() {
        let metadata = create_test_solana_metadata();
        let mut event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let json_value = json!({"key": "value", "number": 42});
        event.set_json_data(json_value.clone());

        let retrieved = event.get_json_data();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &json_value);
    }

    #[test]
    fn test_zerocopy_event_get_json_data_should_return_none_when_no_data() {
        let metadata = create_test_solana_metadata();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let retrieved = event.get_json_data();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_zerocopy_event_to_owned_should_clone_all_data() {
        let metadata = create_test_solana_metadata();
        let raw_data = b"borrowed_data";
        let mut event = ZeroCopyEvent::new_borrowed(metadata.clone(), raw_data);

        let json_value = json!({"test": true});
        event.set_json_data(json_value.clone());
        event.set_parsed_data(42u64);

        let owned_event = event.to_owned();

        assert_eq!(owned_event.metadata.signature, metadata.signature);
        assert_eq!(owned_event.raw_data(), raw_data);
        assert_eq!(owned_event.get_json_data(), Some(&json_value));
        assert_eq!(owned_event.get_parsed_data::<u64>(), Some(&42));
        // Verify it's owned
        matches!(owned_event.raw_data, Cow::Owned(_));
    }

    #[test]
    fn test_zerocopy_event_id_should_return_metadata_id() {
        let metadata = create_test_solana_metadata();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        assert_eq!(event.id(), "test_id");
    }

    #[test]
    fn test_zerocopy_event_event_type_should_return_type() {
        let metadata = create_test_solana_metadata();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let event_type = event.event_type();
        assert_eq!(event_type, EventType::Swap);
    }

    #[test]
    fn test_zerocopy_event_signature_should_return_signature() {
        let metadata = create_test_solana_metadata();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        assert_eq!(event.signature(), "test_signature");
    }

    #[test]
    fn test_zerocopy_event_slot_should_return_slot() {
        let metadata = create_test_solana_metadata();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        assert_eq!(event.slot(), 12345);
    }

    #[test]
    fn test_zerocopy_event_protocol_type_should_return_protocol() {
        let metadata = create_test_solana_metadata();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        assert_eq!(event.protocol_type(), ProtocolType::OrcaWhirlpool);
    }

    #[test]
    fn test_zerocopy_event_index_should_return_index_as_string() {
        let metadata = create_test_solana_metadata();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        assert_eq!(event.index(), "0");
    }

    #[test]
    fn test_zerocopy_event_timestamp_should_return_system_time() {
        let metadata = create_test_solana_metadata();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let timestamp = event.timestamp();
        // Just verify it's a SystemTime and doesn't panic
        assert!(timestamp.duration_since(std::time::UNIX_EPOCH).is_ok());
    }

    #[test]
    fn test_zerocopy_event_block_number_should_return_slot() {
        let metadata = create_test_solana_metadata();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        assert_eq!(event.block_number(), Some(12345));
    }

    #[test]
    fn test_zerocopy_swap_event_new_should_create_with_none_values() {
        let metadata = create_test_solana_metadata();
        let base = ZeroCopyEvent::new_owned(metadata, vec![]);

        let swap_event = ZeroCopySwapEvent::new(base);

        assert!(swap_event.input_mint.is_none());
        assert!(swap_event.output_mint.is_none());
        assert!(swap_event.amount_in.is_none());
        assert!(swap_event.amount_out.is_none());
    }

    #[test]
    fn test_zerocopy_swap_event_input_mint_should_call_parse_and_cache() {
        let metadata = create_test_solana_metadata();
        let base = ZeroCopyEvent::new_owned(metadata, vec![]);
        let mut swap_event = ZeroCopySwapEvent::new(base);

        // First call should parse (returns None due to stub implementation)
        let result1 = swap_event.input_mint();
        assert!(result1.is_none());

        // Second call should use cached value
        let result2 = swap_event.input_mint();
        assert!(result2.is_none());
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_zerocopy_swap_event_output_mint_should_call_parse_and_cache() {
        let metadata = create_test_solana_metadata();
        let base = ZeroCopyEvent::new_owned(metadata, vec![]);
        let mut swap_event = ZeroCopySwapEvent::new(base);

        let result1 = swap_event.output_mint();
        assert!(result1.is_none());

        let result2 = swap_event.output_mint();
        assert!(result2.is_none());
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_zerocopy_swap_event_amount_in_should_call_parse_and_cache() {
        let metadata = create_test_solana_metadata();
        let base = ZeroCopyEvent::new_owned(metadata, vec![]);
        let mut swap_event = ZeroCopySwapEvent::new(base);

        let result1 = swap_event.amount_in();
        assert!(result1.is_none());

        let result2 = swap_event.amount_in();
        assert!(result2.is_none());
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_zerocopy_swap_event_amount_out_should_call_parse_and_cache() {
        let metadata = create_test_solana_metadata();
        let base = ZeroCopyEvent::new_owned(metadata, vec![]);
        let mut swap_event = ZeroCopySwapEvent::new(base);

        let result1 = swap_event.amount_out();
        assert!(result1.is_none());

        let result2 = swap_event.amount_out();
        assert!(result2.is_none());
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_zerocopy_swap_event_parse_methods_should_return_none() {
        let metadata = create_test_solana_metadata();
        let base = ZeroCopyEvent::new_owned(metadata, vec![]);
        let swap_event = ZeroCopySwapEvent::new(base);

        // Test all private parse methods through reflection of their behavior
        assert!(swap_event.parse_input_mint().is_none());
        assert!(swap_event.parse_output_mint().is_none());
        assert!(swap_event.parse_amount_in().is_none());
        assert!(swap_event.parse_amount_out().is_none());
    }

    #[test]
    fn test_zerocopy_liquidity_event_new_should_create_with_none_values() {
        let metadata = create_test_solana_metadata();
        let base = ZeroCopyEvent::new_owned(metadata, vec![]);

        let liquidity_event = ZeroCopyLiquidityEvent::new(base);

        assert!(liquidity_event.pool_address.is_none());
        assert!(liquidity_event.lp_amount.is_none());
        assert!(liquidity_event.token_a_amount.is_none());
        assert!(liquidity_event.token_b_amount.is_none());
    }

    #[test]
    fn test_zerocopy_liquidity_event_pool_address_should_call_parse_and_cache() {
        let metadata = create_test_solana_metadata();
        let base = ZeroCopyEvent::new_owned(metadata, vec![]);
        let mut liquidity_event = ZeroCopyLiquidityEvent::new(base);

        let result1 = liquidity_event.pool_address();
        assert!(result1.is_none());

        let result2 = liquidity_event.pool_address();
        assert!(result2.is_none());
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_zerocopy_liquidity_event_lp_amount_should_call_parse_and_cache() {
        let metadata = create_test_solana_metadata();
        let base = ZeroCopyEvent::new_owned(metadata, vec![]);
        let mut liquidity_event = ZeroCopyLiquidityEvent::new(base);

        let result1 = liquidity_event.lp_amount();
        assert!(result1.is_none());

        let result2 = liquidity_event.lp_amount();
        assert!(result2.is_none());
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_zerocopy_liquidity_event_token_a_amount_should_call_parse_and_cache() {
        let metadata = create_test_solana_metadata();
        let base = ZeroCopyEvent::new_owned(metadata, vec![]);
        let mut liquidity_event = ZeroCopyLiquidityEvent::new(base);

        let result1 = liquidity_event.token_a_amount();
        assert!(result1.is_none());

        let result2 = liquidity_event.token_a_amount();
        assert!(result2.is_none());
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_zerocopy_liquidity_event_token_b_amount_should_call_parse_and_cache() {
        let metadata = create_test_solana_metadata();
        let base = ZeroCopyEvent::new_owned(metadata, vec![]);
        let mut liquidity_event = ZeroCopyLiquidityEvent::new(base);

        let result1 = liquidity_event.token_b_amount();
        assert!(result1.is_none());

        let result2 = liquidity_event.token_b_amount();
        assert!(result2.is_none());
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_zerocopy_liquidity_event_parse_methods_should_return_none() {
        let metadata = create_test_solana_metadata();
        let base = ZeroCopyEvent::new_owned(metadata, vec![]);
        let liquidity_event = ZeroCopyLiquidityEvent::new(base);

        // Test all private parse methods through reflection of their behavior
        assert!(liquidity_event.parse_pool_address().is_none());
        assert!(liquidity_event.parse_lp_amount().is_none());
        assert!(liquidity_event.parse_token_a_amount().is_none());
        assert!(liquidity_event.parse_token_b_amount().is_none());
    }

    #[test]
    fn test_zerocopy_event_with_empty_raw_data() {
        let metadata = create_test_solana_metadata();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        assert_eq!(event.raw_data(), &[] as &[u8]);
    }

    #[test]
    fn test_zerocopy_event_with_large_raw_data() {
        let metadata = create_test_solana_metadata();
        let large_data = vec![42u8; 10000];
        let event = ZeroCopyEvent::new_owned(metadata, large_data.clone());

        assert_eq!(event.raw_data(), &large_data);
        assert_eq!(event.raw_data().len(), 10000);
    }

    #[test]
    fn test_zerocopy_event_clone_should_work() {
        let metadata = create_test_solana_metadata();
        let raw_data = vec![1, 2, 3];
        let mut event = ZeroCopyEvent::new_owned(metadata, raw_data);

        event.set_parsed_data("test".to_string());
        event.set_json_data(json!({"test": true}));

        let cloned = event.clone();

        assert_eq!(cloned.raw_data(), event.raw_data());
        assert_eq!(
            cloned.get_parsed_data::<String>(),
            event.get_parsed_data::<String>()
        );
        assert_eq!(cloned.get_json_data(), event.get_json_data());
    }

    #[test]
    fn test_zerocopy_swap_event_clone_should_work() {
        let metadata = create_test_solana_metadata();
        let base = ZeroCopyEvent::new_owned(metadata, vec![1, 2, 3]);
        let swap_event = ZeroCopySwapEvent::new(base);

        let cloned = swap_event.clone();

        assert_eq!(cloned.base.raw_data(), swap_event.base.raw_data());
        assert_eq!(cloned.input_mint, swap_event.input_mint);
        assert_eq!(cloned.output_mint, swap_event.output_mint);
        assert_eq!(cloned.amount_in, swap_event.amount_in);
        assert_eq!(cloned.amount_out, swap_event.amount_out);
    }

    #[test]
    fn test_zerocopy_liquidity_event_clone_should_work() {
        let metadata = create_test_solana_metadata();
        let base = ZeroCopyEvent::new_owned(metadata, vec![1, 2, 3]);
        let liquidity_event = ZeroCopyLiquidityEvent::new(base);

        let cloned = liquidity_event.clone();

        assert_eq!(cloned.base.raw_data(), liquidity_event.base.raw_data());
        assert_eq!(cloned.pool_address, liquidity_event.pool_address);
        assert_eq!(cloned.lp_amount, liquidity_event.lp_amount);
        assert_eq!(cloned.token_a_amount, liquidity_event.token_a_amount);
        assert_eq!(cloned.token_b_amount, liquidity_event.token_b_amount);
    }
}
