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
            id: String::new(),
            kind: EventKind::default(),
            timestamp: now,
            received_at: now,
            source: String::new(),
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
}
