//! Core event types and data structures.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Event classification for different types of blockchain and system events.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// Blockchain transaction events
    #[default]
    Transaction,
    /// Block-level events (new blocks, reorganizations)
    Block,
    /// Smart contract events and logs
    Contract,
    /// Token transfer events
    Transfer,
    /// Swap/DEX events
    Swap,
    /// Liquidity provision events
    Liquidity,
    /// Price update events
    Price,
    /// External system events (APIs, websockets)
    External,
    /// Custom event type with string identifier
    Custom(String),
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventKind::Transaction => write!(f, "transaction"),
            EventKind::Block => write!(f, "block"),
            EventKind::Contract => write!(f, "contract"),
            EventKind::Transfer => write!(f, "transfer"),
            EventKind::Swap => write!(f, "swap"),
            EventKind::Liquidity => write!(f, "liquidity"),
            EventKind::Price => write!(f, "price"),
            EventKind::External => write!(f, "external"),
            EventKind::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Metadata associated with an event, providing context and timing information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Unique event identifier
    pub id: String,
    /// Event classification
    pub kind: EventKind,
    /// When the event occurred (blockchain time or system time)
    pub timestamp: DateTime<Utc>,
    /// When the event was received by the system
    pub received_at: DateTime<Utc>,
    /// Source that generated the event (e.g., "solana-rpc", "ethereum-ws")
    pub source: String,
    /// Blockchain-specific metadata
    pub chain_data: Option<ChainData>,
    /// Custom metadata fields
    pub custom: HashMap<String, serde_json::Value>,
}

impl EventMetadata {
    /// Create new event metadata with minimal required fields
    pub fn new(id: String, kind: EventKind, source: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            kind,
            timestamp: now,
            received_at: now,
            source,
            chain_data: None,
            custom: HashMap::new(),
        }
    }

    /// Create new event metadata with specified timestamp
    pub fn with_timestamp(
        id: String,
        kind: EventKind,
        source: String,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            kind,
            timestamp,
            received_at: Utc::now(),
            source,
            chain_data: None,
            custom: HashMap::new(),
        }
    }

    /// Add chain-specific data
    pub fn with_chain_data(mut self, chain_data: ChainData) -> Self {
        self.chain_data = Some(chain_data);
        self
    }

    /// Add custom metadata field
    pub fn with_custom<T: Serialize>(mut self, key: String, value: T) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.custom.insert(key, json_value);
        }
        self
    }

    /// Get custom metadata value
    pub fn get_custom<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.custom
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

impl Default for EventMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: String::default(),
            kind: EventKind::default(),
            timestamp: now,
            received_at: now,
            source: String::default(),
            chain_data: None,
            custom: HashMap::new(),
        }
    }
}

/// Blockchain-specific data that can be attached to events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "chain", content = "data")]
pub enum ChainData {
    /// Solana-specific data
    #[cfg(feature = "solana")]
    Solana {
        /// Slot number
        slot: u64,
        /// Transaction signature
        signature: Option<String>,
        /// Program ID
        program_id: Option<solana_sdk::pubkey::Pubkey>,
        /// Instruction index within transaction
        instruction_index: Option<usize>,
        /// Block time in Unix seconds
        block_time: Option<i64>,
        /// Custom protocol-specific data (e.g., protocol_type, event_type)
        protocol_data: Option<serde_json::Value>,
    },
    /// EVM-specific data
    #[cfg(feature = "evm")]
    Evm {
        /// Block number
        block_number: u64,
        /// Transaction hash
        transaction_hash: Option<String>,
        /// Contract address
        contract_address: Option<String>,
        /// Event log index
        log_index: Option<usize>,
        /// Chain ID
        chain_id: u64,
    },
    /// Generic chain data for unsupported chains
    Generic {
        /// Chain identifier
        chain_id: String,
        /// Generic block identifier
        block_id: String,
        /// Generic transaction identifier
        transaction_id: Option<String>,
        /// Additional data
        data: HashMap<String, serde_json::Value>,
    },
}

impl ChainData {
    /// Get the chain identifier
    pub fn chain_id(&self) -> String {
        match self {
            #[cfg(feature = "solana")]
            ChainData::Solana { .. } => "solana".to_string(),
            #[cfg(feature = "evm")]
            ChainData::Evm { chain_id, .. } => chain_id.to_string(),
            ChainData::Generic { chain_id, .. } => chain_id.clone(),
        }
    }

    /// Get the block identifier
    pub fn block_id(&self) -> String {
        match self {
            #[cfg(feature = "solana")]
            ChainData::Solana { slot, .. } => slot.to_string(),
            #[cfg(feature = "evm")]
            ChainData::Evm { block_number, .. } => block_number.to_string(),
            ChainData::Generic { block_id, .. } => block_id.clone(),
        }
    }

    /// Get the transaction identifier if available
    pub fn transaction_id(&self) -> Option<String> {
        match self {
            #[cfg(feature = "solana")]
            ChainData::Solana { signature, .. } => signature.clone(),
            #[cfg(feature = "evm")]
            ChainData::Evm {
                transaction_hash, ..
            } => transaction_hash.clone(),
            ChainData::Generic { transaction_id, .. } => transaction_id.clone(),
        }
    }
}

/// Generic event implementation suitable for most use cases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericEvent {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Event payload data
    pub data: serde_json::Value,
}

impl GenericEvent {
    /// Create a new generic event
    pub fn new(id: String, kind: EventKind, data: serde_json::Value) -> Self {
        Self {
            metadata: EventMetadata::new(id, kind, "generic".to_string()),
            data,
        }
    }

    /// Create a new generic event with source
    pub fn with_source(
        id: String,
        kind: EventKind,
        source: String,
        data: serde_json::Value,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(id, kind, source),
            data,
        }
    }

    /// Create a new generic event with metadata
    pub fn with_metadata(metadata: EventMetadata, data: serde_json::Value) -> Self {
        Self { metadata, data }
    }

    /// Extract typed data from the event
    pub fn extract_data<T>(&self) -> Result<T, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_value(self.data.clone())
    }

    /// Update the event data
    pub fn update_data<T: Serialize>(&mut self, data: T) -> Result<(), serde_json::Error> {
        self.data = serde_json::to_value(data)?;
        Ok(())
    }
}

/// Stream information for tracking event stream sources and their health.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamInfo {
    /// Unique stream identifier
    pub id: String,
    /// Human-readable stream name
    pub name: String,
    /// Stream source type (e.g., "websocket", "rpc", "kafka")
    pub source_type: String,
    /// Stream endpoint or configuration
    pub endpoint: String,
    /// Whether the stream is currently active
    pub active: bool,
    /// Last successful event timestamp
    pub last_event_at: Option<DateTime<Utc>>,
    /// Stream health metrics
    pub metrics: StreamMetrics,
}

impl StreamInfo {
    /// Create new stream info
    pub fn new(id: String, name: String, source_type: String, endpoint: String) -> Self {
        Self {
            id,
            name,
            source_type,
            endpoint,
            active: false,
            last_event_at: None,
            metrics: StreamMetrics::default(),
        }
    }

    /// Mark stream as active
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
        if active {
            self.metrics.connection_count += 1;
        }
    }

    /// Update last event timestamp
    pub fn update_last_event(&mut self) {
        self.last_event_at = Some(Utc::now());
        self.metrics.event_count += 1;
    }

    /// Record an error
    pub fn record_error(&mut self) {
        self.metrics.error_count += 1;
    }
}

/// Stream health and performance metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamMetrics {
    /// Total number of events processed
    pub event_count: u64,
    /// Total number of errors encountered
    pub error_count: u64,
    /// Total number of connections made
    pub connection_count: u64,
    /// Average processing time in microseconds
    pub avg_processing_time_us: u64,
    /// Stream uptime percentage (0-100)
    pub uptime_percentage: f64,
}

impl Default for StreamMetrics {
    fn default() -> Self {
        Self {
            event_count: 0,
            error_count: 0,
            connection_count: 0,
            avg_processing_time_us: 0,
            uptime_percentage: 0.0,
        }
    }
}

impl StreamMetrics {
    /// Calculate error rate as a percentage
    pub fn error_rate(&self) -> f64 {
        if self.event_count == 0 {
            0.0
        } else {
            (self.error_count as f64 / self.event_count as f64) * 100.0
        }
    }

    /// Check if stream is healthy (low error rate, recent activity)
    pub fn is_healthy(&self) -> bool {
        self.error_rate() < 5.0 && self.uptime_percentage > 95.0
    }
}

// Implement the Event trait for GenericEvent
impl crate::traits::Event for GenericEvent {
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn crate::traits::Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> crate::error::EventResult<serde_json::Value> {
        Ok(serde_json::json!({
            "metadata": self.metadata,
            "data": self.data
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Event;
    use serde_json::json;

    #[test]
    fn test_event_kind_display() {
        assert_eq!(EventKind::Transaction.to_string(), "transaction");
        assert_eq!(
            EventKind::Custom("my-event".to_string()).to_string(),
            "my-event"
        );
    }

    #[test]
    fn test_event_metadata_creation() {
        let metadata = EventMetadata::new(
            "test-id".to_string(),
            EventKind::Transaction,
            "test-source".to_string(),
        );

        assert_eq!(metadata.id, "test-id");
        assert_eq!(metadata.kind, EventKind::Transaction);
        assert_eq!(metadata.source, "test-source");
        assert!(metadata.custom.is_empty());
    }

    #[test]
    fn test_event_metadata_custom_fields() {
        let metadata = EventMetadata::new(
            "test-id".to_string(),
            EventKind::Transaction,
            "test-source".to_string(),
        )
        .with_custom("priority".to_string(), "high")
        .with_custom("retry_count".to_string(), 3);

        assert_eq!(
            metadata.get_custom::<String>("priority"),
            Some("high".to_string())
        );
        assert_eq!(metadata.get_custom::<i32>("retry_count"), Some(3));
        assert_eq!(metadata.get_custom::<String>("missing"), None);
    }

    #[test]
    fn test_generic_event_creation() {
        let event = GenericEvent::new(
            "test-event".to_string(),
            EventKind::Swap,
            json!({"amount": 1000, "token": "USDC"}),
        );

        assert_eq!(event.metadata.id, "test-event");
        assert_eq!(event.metadata.kind, EventKind::Swap);
        assert_eq!(event.data["amount"], 1000);
        assert_eq!(event.data["token"], "USDC");
    }

    #[test]
    fn test_generic_event_data_extraction() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct SwapData {
            amount: u64,
            token: String,
        }

        let event = GenericEvent::new(
            "test-event".to_string(),
            EventKind::Swap,
            json!({"amount": 1000, "token": "USDC"}),
        );

        let swap_data: SwapData = event.extract_data().unwrap();
        assert_eq!(
            swap_data,
            SwapData {
                amount: 1000,
                token: "USDC".to_string()
            }
        );
    }

    #[test]
    fn test_stream_info_lifecycle() {
        let mut stream = StreamInfo::new(
            "stream-1".to_string(),
            "Test Stream".to_string(),
            "websocket".to_string(),
            "wss://api.example.com".to_string(),
        );

        assert!(!stream.active);
        assert_eq!(stream.metrics.event_count, 0);

        stream.set_active(true);
        assert!(stream.active);
        assert_eq!(stream.metrics.connection_count, 1);

        stream.update_last_event();
        assert!(stream.last_event_at.is_some());
        assert_eq!(stream.metrics.event_count, 1);

        stream.record_error();
        assert_eq!(stream.metrics.error_count, 1);
    }

    #[test]
    fn test_stream_metrics_error_rate() {
        let metrics = StreamMetrics {
            event_count: 100,
            error_count: 5,
            ..Default::default()
        };

        assert_eq!(metrics.error_rate(), 5.0);
        assert!(!metrics.is_healthy()); // Low uptime
    }

    #[cfg(feature = "solana")]
    #[test]
    fn test_solana_chain_data() {
        use solana_sdk::pubkey::Pubkey;

        let chain_data = ChainData::Solana {
            slot: 123456,
            signature: Some("test-signature".to_string()),
            program_id: Some(Pubkey::new_unique()),
            instruction_index: Some(0),
            block_time: Some(1234567890),
            protocol_data: None,
        };

        assert_eq!(chain_data.chain_id(), "solana");
        assert_eq!(chain_data.block_id(), "123456");
        assert_eq!(
            chain_data.transaction_id(),
            Some("test-signature".to_string())
        );
    }

    #[test]
    fn test_generic_chain_data() {
        let chain_data = ChainData::Generic {
            chain_id: "custom-chain".to_string(),
            block_id: "block-abc".to_string(),
            transaction_id: Some("tx-def".to_string()),
            data: HashMap::new(),
        };

        assert_eq!(chain_data.chain_id(), "custom-chain");
        assert_eq!(chain_data.block_id(), "block-abc");
        assert_eq!(chain_data.transaction_id(), Some("tx-def".to_string()));
    }

    // Additional comprehensive tests for 100% coverage

    #[test]
    fn test_event_kind_display_all_variants() {
        assert_eq!(EventKind::Transaction.to_string(), "transaction");
        assert_eq!(EventKind::Block.to_string(), "block");
        assert_eq!(EventKind::Contract.to_string(), "contract");
        assert_eq!(EventKind::Transfer.to_string(), "transfer");
        assert_eq!(EventKind::Swap.to_string(), "swap");
        assert_eq!(EventKind::Liquidity.to_string(), "liquidity");
        assert_eq!(EventKind::Price.to_string(), "price");
        assert_eq!(EventKind::External.to_string(), "external");
        assert_eq!(
            EventKind::Custom("custom-event".to_string()).to_string(),
            "custom-event"
        );
        assert_eq!(EventKind::Custom("".to_string()).to_string(), "");
    }

    #[test]
    fn test_event_kind_default() {
        assert_eq!(EventKind::default(), EventKind::Transaction);
    }

    #[test]
    fn test_event_kind_serialization() {
        let transaction = EventKind::Transaction;
        let json = serde_json::to_string(&transaction).unwrap();
        let deserialized: EventKind = serde_json::from_str(&json).unwrap();
        assert_eq!(transaction, deserialized);

        let custom = EventKind::Custom("test".to_string());
        let json = serde_json::to_string(&custom).unwrap();
        let deserialized: EventKind = serde_json::from_str(&json).unwrap();
        assert_eq!(custom, deserialized);
    }

    #[test]
    fn test_event_metadata_with_timestamp() {
        let timestamp = DateTime::from_timestamp(1234567890, 0).unwrap();
        let metadata = EventMetadata::with_timestamp(
            "test-id".to_string(),
            EventKind::Block,
            "rpc-source".to_string(),
            timestamp,
        );

        assert_eq!(metadata.id, "test-id");
        assert_eq!(metadata.kind, EventKind::Block);
        assert_eq!(metadata.source, "rpc-source");
        assert_eq!(metadata.timestamp, timestamp);
        assert!(metadata.received_at > timestamp);
        assert!(metadata.chain_data.is_none());
        assert!(metadata.custom.is_empty());
    }

    #[test]
    fn test_event_metadata_with_chain_data() {
        let chain_data = ChainData::Generic {
            chain_id: "test-chain".to_string(),
            block_id: "123".to_string(),
            transaction_id: None,
            data: HashMap::new(),
        };

        let metadata = EventMetadata::new(
            "test-id".to_string(),
            EventKind::Transaction,
            "test-source".to_string(),
        )
        .with_chain_data(chain_data.clone());

        assert_eq!(metadata.chain_data, Some(chain_data));
    }

    #[test]
    fn test_event_metadata_with_custom_serialization_error() {
        // Test a type that fails to serialize
        struct NonSerializable;

        impl serde::Serialize for NonSerializable {
            fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                use serde::ser::Error;
                Err(S::Error::custom("intentional serialization failure"))
            }
        }

        let metadata = EventMetadata::new(
            "test-id".to_string(),
            EventKind::Transaction,
            "test-source".to_string(),
        )
        .with_custom("invalid".to_string(), NonSerializable);

        // The custom field should not be added if serialization fails
        assert!(!metadata.custom.contains_key("invalid"));
    }

    #[test]
    fn test_event_metadata_get_custom_invalid_type() {
        let metadata = EventMetadata::new(
            "test-id".to_string(),
            EventKind::Transaction,
            "test-source".to_string(),
        )
        .with_custom("number".to_string(), 42);

        // Trying to get as wrong type should return None
        assert_eq!(metadata.get_custom::<String>("number"), None);
    }

    #[test]
    fn test_event_metadata_default() {
        let metadata = EventMetadata::default();
        assert_eq!(metadata.id, "");
        assert_eq!(metadata.kind, EventKind::Transaction);
        assert_eq!(metadata.source, "");
        assert!(metadata.chain_data.is_none());
        assert!(metadata.custom.is_empty());
    }

    #[test]
    fn test_generic_chain_data_with_no_transaction() {
        let chain_data = ChainData::Generic {
            chain_id: "test-chain".to_string(),
            block_id: "block-123".to_string(),
            transaction_id: None,
            data: HashMap::new(),
        };

        assert_eq!(chain_data.chain_id(), "test-chain");
        assert_eq!(chain_data.block_id(), "block-123");
        assert_eq!(chain_data.transaction_id(), None);
    }

    #[cfg(feature = "evm")]
    #[test]
    fn test_evm_chain_data() {
        let chain_data = ChainData::Evm {
            block_number: 12345,
            transaction_hash: Some("0xabc123".to_string()),
            contract_address: Some("0xdef456".to_string()),
            log_index: Some(1),
            chain_id: 1,
        };

        assert_eq!(chain_data.chain_id(), "1");
        assert_eq!(chain_data.block_id(), "12345");
        assert_eq!(chain_data.transaction_id(), Some("0xabc123".to_string()));
    }

    #[cfg(feature = "evm")]
    #[test]
    fn test_evm_chain_data_no_transaction() {
        let chain_data = ChainData::Evm {
            block_number: 12345,
            transaction_hash: None,
            contract_address: None,
            log_index: None,
            chain_id: 137,
        };

        assert_eq!(chain_data.chain_id(), "137");
        assert_eq!(chain_data.block_id(), "12345");
        assert_eq!(chain_data.transaction_id(), None);
    }

    #[cfg(feature = "solana")]
    #[test]
    fn test_solana_chain_data_no_signature() {
        let chain_data = ChainData::Solana {
            slot: 98765,
            signature: None,
            program_id: None,
            instruction_index: None,
            block_time: None,
            protocol_data: None,
        };

        assert_eq!(chain_data.chain_id(), "solana");
        assert_eq!(chain_data.block_id(), "98765");
        assert_eq!(chain_data.transaction_id(), None);
    }

    #[test]
    fn test_generic_event_with_source() {
        let event = GenericEvent::with_source(
            "test-id".to_string(),
            EventKind::Price,
            "price-feed".to_string(),
            json!({"symbol": "BTC", "price": 50000}),
        );

        assert_eq!(event.metadata.id, "test-id");
        assert_eq!(event.metadata.kind, EventKind::Price);
        assert_eq!(event.metadata.source, "price-feed");
        assert_eq!(event.data["symbol"], "BTC");
        assert_eq!(event.data["price"], 50000);
    }

    #[test]
    fn test_generic_event_with_metadata() {
        let metadata = EventMetadata::new(
            "custom-id".to_string(),
            EventKind::Liquidity,
            "dex-source".to_string(),
        );
        let event = GenericEvent::with_metadata(
            metadata.clone(),
            json!({"pool": "ETH/USDC", "amount": 1000}),
        );

        assert_eq!(event.metadata, metadata);
        assert_eq!(event.data["pool"], "ETH/USDC");
        assert_eq!(event.data["amount"], 1000);
    }

    #[test]
    fn test_generic_event_extract_data_error() {
        #[derive(Deserialize)]
        struct InvalidData {
            #[allow(dead_code)]
            required_field: String,
        }

        let event = GenericEvent::new(
            "test-id".to_string(),
            EventKind::Transaction,
            json!({"wrong_field": "value"}),
        );

        let result = event.extract_data::<InvalidData>();
        assert!(result.is_err());
    }

    #[test]
    fn test_generic_event_update_data_success() {
        #[derive(Serialize)]
        struct NewData {
            field: String,
            value: i32,
        }

        let mut event = GenericEvent::new(
            "test-id".to_string(),
            EventKind::Contract,
            json!({"old": "data"}),
        );

        let new_data = NewData {
            field: "test".to_string(),
            value: 42,
        };

        let result = event.update_data(new_data);
        assert!(result.is_ok());
        assert_eq!(event.data["field"], "test");
        assert_eq!(event.data["value"], 42);
    }

    #[test]
    fn test_generic_event_update_data_error() {
        struct NonSerializable;

        impl serde::Serialize for NonSerializable {
            fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                use serde::ser::Error;
                Err(S::Error::custom("intentional serialization failure"))
            }
        }

        let mut event = GenericEvent::new(
            "test-id".to_string(),
            EventKind::Contract,
            json!({"old": "data"}),
        );

        let result = event.update_data(NonSerializable);
        assert!(result.is_err());
    }

    #[test]
    fn test_stream_info_new() {
        let stream = StreamInfo::new(
            "stream-1".to_string(),
            "Test Stream".to_string(),
            "rpc".to_string(),
            "https://api.test.com".to_string(),
        );

        assert_eq!(stream.id, "stream-1");
        assert_eq!(stream.name, "Test Stream");
        assert_eq!(stream.source_type, "rpc");
        assert_eq!(stream.endpoint, "https://api.test.com");
        assert!(!stream.active);
        assert!(stream.last_event_at.is_none());
        assert_eq!(stream.metrics, StreamMetrics::default());
    }

    #[test]
    fn test_stream_info_set_active_false() {
        let mut stream = StreamInfo::new(
            "stream-1".to_string(),
            "Test Stream".to_string(),
            "websocket".to_string(),
            "wss://api.test.com".to_string(),
        );

        stream.set_active(false);
        assert!(!stream.active);
        assert_eq!(stream.metrics.connection_count, 0);
    }

    #[test]
    fn test_stream_info_multiple_activations() {
        let mut stream = StreamInfo::new(
            "stream-1".to_string(),
            "Test Stream".to_string(),
            "websocket".to_string(),
            "wss://api.test.com".to_string(),
        );

        stream.set_active(true);
        stream.set_active(true);
        stream.set_active(false);
        stream.set_active(true);

        assert!(stream.active);
        assert_eq!(stream.metrics.connection_count, 3);
    }

    #[test]
    fn test_stream_metrics_default() {
        let metrics = StreamMetrics::default();
        assert_eq!(metrics.event_count, 0);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.connection_count, 0);
        assert_eq!(metrics.avg_processing_time_us, 0);
        assert_eq!(metrics.uptime_percentage, 0.0);
    }

    #[test]
    fn test_stream_metrics_error_rate_zero_events() {
        let metrics = StreamMetrics {
            event_count: 0,
            error_count: 5,
            ..Default::default()
        };

        assert_eq!(metrics.error_rate(), 0.0);
    }

    #[test]
    fn test_stream_metrics_error_rate_with_events() {
        let metrics = StreamMetrics {
            event_count: 200,
            error_count: 10,
            ..Default::default()
        };

        assert_eq!(metrics.error_rate(), 5.0);
    }

    #[test]
    fn test_stream_metrics_is_healthy_true() {
        let metrics = StreamMetrics {
            event_count: 1000,
            error_count: 10, // 1% error rate
            uptime_percentage: 99.5,
            ..Default::default()
        };

        assert!(metrics.is_healthy());
    }

    #[test]
    fn test_stream_metrics_is_healthy_false_high_error_rate() {
        let metrics = StreamMetrics {
            event_count: 100,
            error_count: 10, // 10% error rate
            uptime_percentage: 99.0,
            ..Default::default()
        };

        assert!(!metrics.is_healthy());
    }

    #[test]
    fn test_stream_metrics_is_healthy_false_low_uptime() {
        let metrics = StreamMetrics {
            event_count: 1000,
            error_count: 10, // 1% error rate
            uptime_percentage: 90.0,
            ..Default::default()
        };

        assert!(!metrics.is_healthy());
    }

    #[test]
    fn test_generic_event_trait_implementations() {
        let event = GenericEvent::new(
            "trait-test".to_string(),
            EventKind::External,
            json!({"test": "data"}),
        );

        // Test Event trait methods
        assert_eq!(event.id(), "trait-test");
        assert_eq!(event.kind(), &EventKind::External);
        assert_eq!(event.metadata().id, "trait-test");

        // Test as_any
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<GenericEvent>().is_some());

        // Test clone_boxed
        let boxed_clone = event.clone_boxed();
        assert_eq!(boxed_clone.id(), "trait-test");

        // Test to_json
        let json_result = event.to_json();
        assert!(json_result.is_ok());
        let json_value = json_result.unwrap();
        assert_eq!(json_value["metadata"]["id"], "trait-test");
        assert_eq!(json_value["data"]["test"], "data");
    }

    #[test]
    fn test_generic_event_trait_mutable_metadata() {
        let mut event = GenericEvent::new(
            "mutable-test".to_string(),
            EventKind::Transfer,
            json!({"amount": 100}),
        );

        // Test metadata_mut
        let metadata_mut = event.metadata_mut();
        metadata_mut.source = "updated-source".to_string();

        assert_eq!(event.metadata.source, "updated-source");

        // Test as_any_mut
        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<GenericEvent>().is_some());
    }

    #[test]
    fn test_event_kind_hash_and_eq() {
        let kind1 = EventKind::Transaction;
        let kind2 = EventKind::Transaction;
        let kind3 = EventKind::Block;
        let custom1 = EventKind::Custom("test".to_string());
        let custom2 = EventKind::Custom("test".to_string());
        let custom3 = EventKind::Custom("different".to_string());

        assert_eq!(kind1, kind2);
        assert_ne!(kind1, kind3);
        assert_eq!(custom1, custom2);
        assert_ne!(custom1, custom3);

        // Test in HashMap to verify Hash implementation
        let mut map = HashMap::new();
        map.insert(kind1, "transaction");
        map.insert(custom1, "custom");

        assert_eq!(map.get(&kind2), Some(&"transaction"));
        assert_eq!(map.get(&custom2), Some(&"custom"));
    }

    #[test]
    fn test_stream_info_multiple_events_and_errors() {
        let mut stream = StreamInfo::new(
            "multi-test".to_string(),
            "Multi Test Stream".to_string(),
            "kafka".to_string(),
            "kafka://localhost:9092".to_string(),
        );

        // Process multiple events
        for _ in 0..5 {
            stream.update_last_event();
        }

        // Record multiple errors
        for _ in 0..2 {
            stream.record_error();
        }

        assert_eq!(stream.metrics.event_count, 5);
        assert_eq!(stream.metrics.error_count, 2);
        assert!(stream.last_event_at.is_some());
    }

    #[test]
    fn test_event_metadata_complex_custom_data() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct ComplexData {
            nested: HashMap<String, i32>,
            array: Vec<String>,
        }

        let mut nested = HashMap::new();
        nested.insert("key1".to_string(), 42);
        nested.insert("key2".to_string(), 24);

        let complex_data = ComplexData {
            nested,
            array: vec!["a".to_string(), "b".to_string()],
        };

        let metadata = EventMetadata::new(
            "complex-test".to_string(),
            EventKind::Custom("complex".to_string()),
            "test-source".to_string(),
        )
        .with_custom("complex_field".to_string(), &complex_data);

        let retrieved: Option<ComplexData> = metadata.get_custom("complex_field");
        assert_eq!(retrieved, Some(complex_data));
    }
}
