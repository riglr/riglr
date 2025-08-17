//! Zero-copy event implementations that reference source data directly
//!
//! These event types avoid allocations by keeping references to the original data
//! where possible, providing significant performance improvements for high-throughput
//! parsing scenarios.

use crate::metadata_helpers::{get_event_type, get_instruction_index, get_protocol_type, get_signature, get_slot};
use crate::types::{EventMetadata, EventType, ProtocolType};
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
        &self.metadata.id
    }

    /// Get event type for any lifetime
    pub fn event_type(&self) -> EventType {
        get_event_type(&self.metadata).unwrap_or_default()
    }

    /// Get signature for any lifetime
    pub fn signature(&self) -> &str {
        get_signature(&self.metadata).unwrap_or("")
    }

    /// Get slot for any lifetime
    pub fn slot(&self) -> u64 {
        get_slot(&self.metadata).unwrap_or(0)
    }

    /// Get protocol type for any lifetime
    pub fn protocol_type(&self) -> ProtocolType {
        get_protocol_type(&self.metadata).unwrap_or_default()
    }

    /// Get index for any lifetime
    pub fn index(&self) -> String {
        get_instruction_index(&self.metadata).map(|i| i.to_string()).unwrap_or_default()
    }

    /// Get timestamp for any lifetime
    pub fn timestamp(&self) -> std::time::SystemTime {
        // Use the timestamp from metadata which is already a DateTime<Utc>
        std::time::SystemTime::from(self.metadata.timestamp)
    }

    /// Get block number for any lifetime
    pub fn block_number(&self) -> Option<u64> {
        get_slot(&self.metadata)
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
