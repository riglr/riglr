use super::types::{OrcaLiquidityData, OrcaPositionData, OrcaSwapData};
use crate::metadata_helpers;
use crate::types::{EventMetadata, EventType, ProtocolType, TransferData};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;

// Import Event trait from riglr-events-core
use riglr_events_core::{Event, EventKind};

/// Parameters for creating Orca events
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventParameters {
    /// Unique identifier for the event
    pub id: String,
    /// Transaction signature
    pub signature: String,
    /// Solana slot number
    pub slot: u64,
    /// Block timestamp in seconds
    pub block_time: i64,
    /// Block timestamp in milliseconds
    pub block_time_ms: i64,
    /// Time when program received the transaction (in milliseconds)
    pub program_received_time_ms: i64,
    /// Instruction index within the transaction
    pub index: String,
}

impl EventParameters {
    /// Create new EventParameters
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
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    pub metadata: EventMetadata,
    /// Unique event identifier
    pub id: String,
    /// Event kind/classification
    pub kind: riglr_events_core::EventKind,
    /// Event timestamp
    pub timestamp: std::time::SystemTime,
    /// Time when event was received
    pub received_at: std::time::SystemTime,
    /// Source that generated this event
    pub source: String,
    /// Chain-specific data
    pub chain_data: Option<serde_json::Value>,
    /// Custom metadata
    pub custom: std::collections::HashMap<String, serde_json::Value>,
    /// Orca-specific swap data
    pub swap_data: OrcaSwapData,
    /// Associated token transfer data
    pub transfer_data: Vec<TransferData>,
}

impl OrcaSwapEvent {
    /// Creates a new OrcaSwapEvent with the provided parameters and swap data
    pub fn new(params: EventParameters, swap_data: OrcaSwapData) -> Self {
        let metadata = metadata_helpers::create_solana_metadata(
            params.id.clone(),
            riglr_events_core::EventKind::Swap,
            "solana-orca".to_string(),
            params.slot,
            Some(params.signature.clone()),
            None, // program_id will be set later if needed
            Some(params.index.parse().unwrap_or(0)),
            Some(params.block_time),
            ProtocolType::OrcaWhirlpool,
            EventType::Swap,
        );

        let solana_metadata = crate::solana_metadata::SolanaEventMetadata::new(
            params.signature,
            params.slot,
            EventType::Swap,
            ProtocolType::OrcaWhirlpool,
            params.index,
            params.program_received_time_ms,
            metadata,
        );

        Self {
            metadata: solana_metadata.clone(),
            id: params.id,
            kind: riglr_events_core::EventKind::Swap,
            timestamp: std::time::SystemTime::UNIX_EPOCH
                + std::time::Duration::from_secs(params.block_time as u64),
            received_at: std::time::SystemTime::UNIX_EPOCH
                + std::time::Duration::from_millis(params.program_received_time_ms as u64),
            source: "solana-orca".to_string(),
            chain_data: None,
            custom: HashMap::default(),
            swap_data,
            transfer_data: Vec::default(),
        }
    }

    /// Sets the transfer data for this swap event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

impl Default for OrcaSwapEvent {
    fn default() -> Self {
        Self {
            metadata: EventMetadata::default(),
            id: String::default(),
            kind: riglr_events_core::EventKind::Swap,
            timestamp: std::time::SystemTime::UNIX_EPOCH,
            received_at: std::time::SystemTime::UNIX_EPOCH,
            source: String::default(),
            chain_data: None,
            custom: HashMap::default(),
            swap_data: OrcaSwapData::default(),
            transfer_data: Vec::default(),
        }
    }
}

/// Orca position event (open/close)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaPositionEvent {
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    pub metadata: EventMetadata,
    /// Unique event identifier
    pub id: String,
    /// Event kind/classification
    pub kind: riglr_events_core::EventKind,
    /// Event timestamp
    pub timestamp: std::time::SystemTime,
    /// Time when event was received
    pub received_at: std::time::SystemTime,
    /// Source that generated this event
    pub source: String,
    /// Chain-specific data
    pub chain_data: Option<serde_json::Value>,
    /// Custom metadata
    pub custom: std::collections::HashMap<String, serde_json::Value>,
    /// Orca-specific position data
    pub position_data: OrcaPositionData,
    /// Whether the position is being opened (true) or closed (false)
    pub is_open: bool,
    /// Associated token transfer data
    pub transfer_data: Vec<TransferData>,
}

impl OrcaPositionEvent {
    /// Creates a new OrcaPositionEvent with the provided parameters and position data
    pub fn new(params: EventParameters, position_data: OrcaPositionData, is_open: bool) -> Self {
        let event_type = if is_open {
            EventType::OpenPosition
        } else {
            EventType::ClosePosition
        };

        let metadata = metadata_helpers::create_solana_metadata(
            params.id.clone(),
            riglr_events_core::EventKind::Custom("position".to_string()),
            "solana-orca".to_string(),
            params.slot,
            Some(params.signature.clone()),
            None, // program_id will be set later if needed
            Some(params.index.parse().unwrap_or(0)),
            Some(params.block_time),
            ProtocolType::OrcaWhirlpool,
            event_type.clone(),
        );

        let solana_metadata = crate::solana_metadata::SolanaEventMetadata::new(
            params.signature,
            params.slot,
            event_type,
            ProtocolType::OrcaWhirlpool,
            params.index,
            params.program_received_time_ms,
            metadata,
        );

        Self {
            metadata: solana_metadata.clone(),
            id: params.id,
            kind: riglr_events_core::EventKind::Custom("position".to_string()),
            timestamp: std::time::SystemTime::UNIX_EPOCH
                + std::time::Duration::from_secs(params.block_time as u64),
            received_at: std::time::SystemTime::UNIX_EPOCH
                + std::time::Duration::from_millis(params.program_received_time_ms as u64),
            source: "solana-orca".to_string(),
            chain_data: None,
            custom: HashMap::default(),
            position_data,
            is_open,
            transfer_data: Vec::default(),
        }
    }

    /// Sets the transfer data for this position event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

impl Default for OrcaPositionEvent {
    fn default() -> Self {
        Self {
            metadata: EventMetadata::default(),
            id: String::default(),
            kind: riglr_events_core::EventKind::Custom("position".to_string()),
            timestamp: std::time::SystemTime::UNIX_EPOCH,
            received_at: std::time::SystemTime::UNIX_EPOCH,
            source: String::default(),
            chain_data: None,
            custom: HashMap::default(),
            position_data: OrcaPositionData::default(),
            is_open: false,
            transfer_data: Vec::default(),
        }
    }
}

/// Orca liquidity event (increase/decrease)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaLiquidityEvent {
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    pub metadata: EventMetadata,
    /// Unique event identifier
    pub id: String,
    /// Event kind/classification
    pub kind: riglr_events_core::EventKind,
    /// Event timestamp
    pub timestamp: std::time::SystemTime,
    /// Time when event was received
    pub received_at: std::time::SystemTime,
    /// Source that generated this event
    pub source: String,
    /// Chain-specific data
    pub chain_data: Option<serde_json::Value>,
    /// Custom metadata
    pub custom: std::collections::HashMap<String, serde_json::Value>,
    /// Orca-specific liquidity data
    pub liquidity_data: OrcaLiquidityData,
    /// Associated token transfer data
    pub transfer_data: Vec<TransferData>,
}

impl OrcaLiquidityEvent {
    /// Creates a new OrcaLiquidityEvent with the provided parameters and liquidity data
    pub fn new(params: EventParameters, liquidity_data: OrcaLiquidityData) -> Self {
        let metadata = metadata_helpers::create_solana_metadata(
            params.id.clone(),
            riglr_events_core::EventKind::Custom("liquidity".to_string()),
            "solana-orca".to_string(),
            params.slot,
            Some(params.signature.clone()),
            None, // program_id will be set later if needed
            Some(params.index.parse().unwrap_or(0)),
            Some(params.block_time),
            ProtocolType::OrcaWhirlpool,
            EventType::AddLiquidity, // Default to AddLiquidity, can be changed later
        );

        let solana_metadata = crate::solana_metadata::SolanaEventMetadata::new(
            params.signature,
            params.slot,
            EventType::AddLiquidity,
            ProtocolType::OrcaWhirlpool,
            params.index,
            params.program_received_time_ms,
            metadata,
        );

        Self {
            metadata: solana_metadata.clone(),
            id: params.id,
            kind: riglr_events_core::EventKind::Custom("liquidity".to_string()),
            timestamp: std::time::SystemTime::UNIX_EPOCH
                + std::time::Duration::from_secs(params.block_time as u64),
            received_at: std::time::SystemTime::UNIX_EPOCH
                + std::time::Duration::from_millis(params.program_received_time_ms as u64),
            source: "solana-orca".to_string(),
            chain_data: None,
            custom: HashMap::default(),
            liquidity_data,
            transfer_data: Vec::default(),
        }
    }

    /// Sets the transfer data for this liquidity event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

impl Default for OrcaLiquidityEvent {
    fn default() -> Self {
        Self {
            metadata: EventMetadata::default(),
            id: String::default(),
            kind: riglr_events_core::EventKind::Custom("liquidity".to_string()),
            timestamp: std::time::SystemTime::UNIX_EPOCH,
            received_at: std::time::SystemTime::UNIX_EPOCH,
            source: String::default(),
            chain_data: None,
            custom: HashMap::default(),
            liquidity_data: OrcaLiquidityData::default(),
            transfer_data: Vec::default(),
        }
    }
}

// Event trait implementation for OrcaSwapEvent
impl Event for OrcaSwapEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &EventKind {
        &self.kind
    }

    fn metadata(&self) -> &riglr_events_core::EventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut riglr_events_core::EventMetadata {
        &mut self.metadata.core
    }

    fn timestamp(&self) -> std::time::SystemTime {
        self.timestamp
    }

    fn source(&self) -> &str {
        &self.source
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
        serde_json::to_value(self).map_err(|e| {
            riglr_events_core::error::EventError::generic(format!("Serialization failed: {}", e))
        })
    }
}

// Event trait implementation for OrcaPositionEvent
impl Event for OrcaPositionEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &EventKind {
        &self.kind
    }

    fn metadata(&self) -> &riglr_events_core::EventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut riglr_events_core::EventMetadata {
        &mut self.metadata.core
    }

    fn timestamp(&self) -> std::time::SystemTime {
        self.timestamp
    }

    fn source(&self) -> &str {
        &self.source
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
        serde_json::to_value(self).map_err(|e| {
            riglr_events_core::error::EventError::generic(format!("Serialization failed: {}", e))
        })
    }
}

// Event trait implementation for OrcaLiquidityEvent
impl Event for OrcaLiquidityEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &EventKind {
        &self.kind
    }

    fn metadata(&self) -> &riglr_events_core::EventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut riglr_events_core::EventMetadata {
        &mut self.metadata.core
    }

    fn timestamp(&self) -> std::time::SystemTime {
        self.timestamp
    }

    fn source(&self) -> &str {
        &self.source
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
        serde_json::to_value(self).map_err(|e| {
            riglr_events_core::error::EventError::generic(format!("Serialization failed: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::protocols::orca::types::PositionRewardInfo;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    // Helper function to create test EventParameters
    fn create_test_event_parameters() -> EventParameters {
        EventParameters::new(
            "test-id".to_string(),
            "test-signature".to_string(),
            12345,
            164099,
            164099000,
            164099001,
            "0".to_string(),
        )
    }

    // Helper function to create test OrcaSwapData
    fn create_test_swap_data() -> OrcaSwapData {
        OrcaSwapData {
            whirlpool: Pubkey::from_str("11111111111111111111111111111112").unwrap(),
            user: Pubkey::from_str("11111111111111111111111111111113").unwrap(),
            token_mint_a: Pubkey::from_str("11111111111111111111111111111114").unwrap(),
            token_mint_b: Pubkey::from_str("11111111111111111111111111111115").unwrap(),
            token_vault_a: Pubkey::from_str("11111111111111111111111111111116").unwrap(),
            token_vault_b: Pubkey::from_str("11111111111111111111111111111117").unwrap(),
            amount: 1000,
            amount_specified_is_input: true,
            a_to_b: true,
            sqrt_price_limit: 123456789,
            amount_in: 1000,
            amount_out: 950,
            fee_amount: 3,
            tick_current_index: 100,
            sqrt_price: 98765432109876543210,
            liquidity: 50000,
        }
    }

    // Helper function to create test OrcaPositionData
    fn create_test_position_data() -> OrcaPositionData {
        OrcaPositionData {
            whirlpool: Pubkey::from_str("11111111111111111111111111111112").unwrap(),
            position_mint: Pubkey::from_str("11111111111111111111111111111113").unwrap(),
            position: Pubkey::from_str("11111111111111111111111111111114").unwrap(),
            position_token_account: Pubkey::from_str("11111111111111111111111111111115").unwrap(),
            position_authority: Pubkey::from_str("11111111111111111111111111111116").unwrap(),
            tick_lower_index: -1000,
            tick_upper_index: 1000,
            liquidity: 25000,
            fee_growth_checkpoint_a: 123456789,
            fee_growth_checkpoint_b: 987654321,
            fee_owed_a: 100,
            fee_owed_b: 200,
            reward_infos: [PositionRewardInfo::default(); 3],
        }
    }

    // Helper function to create test OrcaLiquidityData
    fn create_test_liquidity_data() -> OrcaLiquidityData {
        OrcaLiquidityData {
            whirlpool: Pubkey::from_str("11111111111111111111111111111112").unwrap(),
            position: Pubkey::from_str("11111111111111111111111111111113").unwrap(),
            position_authority: Pubkey::from_str("11111111111111111111111111111114").unwrap(),
            token_mint_a: Pubkey::from_str("11111111111111111111111111111115").unwrap(),
            token_mint_b: Pubkey::from_str("11111111111111111111111111111116").unwrap(),
            token_vault_a: Pubkey::from_str("11111111111111111111111111111117").unwrap(),
            token_vault_b: Pubkey::from_str("11111111111111111111111111111118").unwrap(),
            tick_lower_index: -500,
            tick_upper_index: 500,
            liquidity_amount: 10000,
            token_max_a: 1000,
            token_max_b: 2000,
            token_actual_a: 950,
            token_actual_b: 1900,
            is_increase: true,
        }
    }

    // Helper function to create test TransferData
    fn create_test_transfer_data() -> Vec<TransferData> {
        vec![
            TransferData {
                source: Pubkey::from_str("11111111111111111111111111111112").unwrap(),
                destination: Pubkey::from_str("11111111111111111111111111111113").unwrap(),
                mint: Some(Pubkey::from_str("11111111111111111111111111111114").unwrap()),
                amount: 1000,
            },
            TransferData {
                source: Pubkey::from_str("11111111111111111111111111111115").unwrap(),
                destination: Pubkey::from_str("11111111111111111111111111111116").unwrap(),
                mint: None,
                amount: 500,
            },
        ]
    }

    // Tests for EventParameters
    #[test]
    fn test_event_parameters_new() {
        let params = EventParameters::new(
            "id1".to_string(),
            "sig1".to_string(),
            100,
            1000,
            1000000,
            1000001,
            "1".to_string(),
        );

        assert_eq!(params.id, "id1");
        assert_eq!(params.signature, "sig1");
        assert_eq!(params.slot, 100);
        assert_eq!(params.block_time, 1000);
        assert_eq!(params.block_time_ms, 1000000);
        assert_eq!(params.program_received_time_ms, 1000001);
        assert_eq!(params.index, "1");
    }

    #[test]
    fn test_event_parameters_default() {
        let params = EventParameters::default();

        assert_eq!(params.id, "");
        assert_eq!(params.signature, "");
        assert_eq!(params.slot, 0);
        assert_eq!(params.block_time, 0);
        assert_eq!(params.block_time_ms, 0);
        assert_eq!(params.program_received_time_ms, 0);
        assert_eq!(params.index, "");
    }

    #[test]
    fn test_event_parameters_debug() {
        let params = create_test_event_parameters();
        let debug_string = format!("{:?}", params);
        assert!(debug_string.contains("EventParameters"));
    }

    #[test]
    fn test_event_parameters_clone() {
        let params = create_test_event_parameters();
        let cloned = params.clone();

        assert_eq!(params.id, cloned.id);
        assert_eq!(params.signature, cloned.signature);
        assert_eq!(params.slot, cloned.slot);
        assert_eq!(params.block_time, cloned.block_time);
        assert_eq!(params.block_time_ms, cloned.block_time_ms);
        assert_eq!(
            params.program_received_time_ms,
            cloned.program_received_time_ms
        );
        assert_eq!(params.index, cloned.index);
    }

    // Tests for OrcaSwapEvent
    #[test]
    fn test_orca_swap_event_new() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = OrcaSwapEvent::new(params.clone(), swap_data.clone());

        assert_eq!(event.metadata.id(), &params.id);
        assert_eq!(event.swap_data.whirlpool, swap_data.whirlpool);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_swap_event_with_transfer_data() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let transfer_data = create_test_transfer_data();

        let event = OrcaSwapEvent::new(params, swap_data).with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 2);
        assert_eq!(event.transfer_data[0].amount, 1000);
        assert_eq!(event.transfer_data[1].amount, 500);
    }

    #[test]
    fn test_orca_swap_event_with_empty_transfer_data() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();

        let event = OrcaSwapEvent::new(params, swap_data).with_transfer_data(vec![]);

        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_swap_event_default() {
        let event = OrcaSwapEvent::default();

        assert_eq!(event.metadata.id(), "");
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_swap_event_clone() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let transfer_data = create_test_transfer_data();

        let event = OrcaSwapEvent::new(params, swap_data).with_transfer_data(transfer_data);
        let cloned = event.clone();

        assert_eq!(event.metadata.id(), cloned.metadata.id());
        assert_eq!(event.swap_data.whirlpool, cloned.swap_data.whirlpool);
        assert_eq!(event.transfer_data.len(), cloned.transfer_data.len());
    }

    #[test]
    fn test_orca_swap_event_serialization() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = OrcaSwapEvent::new(params, swap_data);

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: OrcaSwapEvent = serde_json::from_str(&serialized).unwrap();

        // Note: metadata field is skipped in serialization, so we only compare other fields
        assert_eq!(event.swap_data.whirlpool, deserialized.swap_data.whirlpool);
    }

    // Tests for OrcaPositionEvent
    #[test]
    fn test_orca_position_event_new_open() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = OrcaPositionEvent::new(params.clone(), position_data.clone(), true);

        assert_eq!(event.metadata.core.id, params.id);
        assert_eq!(event.position_data.whirlpool, position_data.whirlpool);
        assert!(event.is_open);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_position_event_new_close() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = OrcaPositionEvent::new(params, position_data, false);

        assert!(!event.is_open);
    }

    #[test]
    fn test_orca_position_event_with_transfer_data() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let transfer_data = create_test_transfer_data();

        let event = OrcaPositionEvent::new(params, position_data, true)
            .with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 2);
        assert_eq!(event.transfer_data[0].amount, 1000);
        assert_eq!(event.transfer_data[1].amount, 500);
    }

    #[test]
    fn test_orca_position_event_with_empty_transfer_data() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();

        let event = OrcaPositionEvent::new(params, position_data, true).with_transfer_data(vec![]);

        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_position_event_default() {
        let event = OrcaPositionEvent::default();

        assert_eq!(event.id(), "");
        assert_eq!(event.metadata.signature, "");
        assert_eq!(event.metadata.slot, 0);
        assert_eq!(
            crate::metadata_helpers::get_block_time(&event.metadata.core).unwrap_or(0),
            0
        );
        assert_eq!(
            crate::metadata_helpers::get_block_time(&event.metadata.core).unwrap_or(0) * 1000,
            0
        );
        assert_eq!(event.metadata.program_received_time_ms, 0);
        assert_eq!(event.metadata.index, "");
        assert!(!event.is_open);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_position_event_clone() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let transfer_data = create_test_transfer_data();

        let event =
            OrcaPositionEvent::new(params, position_data, true).with_transfer_data(transfer_data);
        let cloned = event.clone();

        assert_eq!(event.id(), cloned.id());
        assert_eq!(event.is_open, cloned.is_open);
        assert_eq!(
            event.position_data.whirlpool,
            cloned.position_data.whirlpool
        );
        assert_eq!(event.transfer_data.len(), cloned.transfer_data.len());
    }

    #[test]
    fn test_orca_position_event_serialization() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = OrcaPositionEvent::new(params, position_data, true);

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: OrcaPositionEvent = serde_json::from_str(&serialized).unwrap();

        // Note: metadata field is skipped in serialization, so we only compare other fields
        assert_eq!(event.is_open, deserialized.is_open);
        assert_eq!(
            event.position_data.whirlpool,
            deserialized.position_data.whirlpool
        );
    }

    // Tests for OrcaLiquidityEvent
    #[test]
    fn test_orca_liquidity_event_new() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let event = OrcaLiquidityEvent::new(params.clone(), liquidity_data.clone());

        assert_eq!(event.id(), &params.id);
        assert_eq!(event.metadata.signature, params.signature);
        assert_eq!(event.metadata.slot, params.slot);
        assert_eq!(
            crate::metadata_helpers::get_block_time(&event.metadata.core).unwrap_or(0),
            params.block_time
        );
        assert_eq!(
            crate::metadata_helpers::get_block_time(&event.metadata.core).unwrap_or(0) * 1000,
            params.block_time_ms
        );
        assert_eq!(
            event.metadata.program_received_time_ms,
            params.program_received_time_ms
        );
        assert_eq!(event.metadata.index, params.index);
        assert_eq!(event.liquidity_data.whirlpool, liquidity_data.whirlpool);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_liquidity_event_with_transfer_data() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let transfer_data = create_test_transfer_data();

        let event = OrcaLiquidityEvent::new(params, liquidity_data)
            .with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 2);
        assert_eq!(event.transfer_data[0].amount, 1000);
        assert_eq!(event.transfer_data[1].amount, 500);
    }

    #[test]
    fn test_orca_liquidity_event_with_empty_transfer_data() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();

        let event = OrcaLiquidityEvent::new(params, liquidity_data).with_transfer_data(vec![]);

        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_liquidity_event_default() {
        let event = OrcaLiquidityEvent::default();

        assert_eq!(event.id(), "");
        assert_eq!(event.metadata.signature, "");
        assert_eq!(event.metadata.slot, 0);
        assert_eq!(
            crate::metadata_helpers::get_block_time(&event.metadata.core).unwrap_or(0),
            0
        );
        assert_eq!(
            crate::metadata_helpers::get_block_time(&event.metadata.core).unwrap_or(0) * 1000,
            0
        );
        assert_eq!(event.metadata.program_received_time_ms, 0);
        assert_eq!(event.metadata.index, "");
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_liquidity_event_clone() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let transfer_data = create_test_transfer_data();

        let event =
            OrcaLiquidityEvent::new(params, liquidity_data).with_transfer_data(transfer_data);
        let cloned = event.clone();

        assert_eq!(event.id(), cloned.id());
        assert_eq!(
            event.liquidity_data.whirlpool,
            cloned.liquidity_data.whirlpool
        );
        assert_eq!(event.transfer_data.len(), cloned.transfer_data.len());
    }

    #[test]
    fn test_orca_liquidity_event_serialization() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let event = OrcaLiquidityEvent::new(params, liquidity_data);

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: OrcaLiquidityEvent = serde_json::from_str(&serialized).unwrap();

        // Note: metadata field is skipped in serialization, so we only compare other fields
        assert_eq!(
            event.liquidity_data.whirlpool,
            deserialized.liquidity_data.whirlpool
        );
    }

    // Tests for Event trait implementation on OrcaSwapEvent
    #[test]
    fn test_orca_swap_event_trait_id() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = OrcaSwapEvent::new(params, swap_data);

        assert_eq!(event.id(), event.metadata.id());
    }

    #[test]
    fn test_orca_swap_event_trait_kind() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = OrcaSwapEvent::new(params, swap_data);

        assert_eq!(event.kind(), event.metadata.kind());
    }

    #[test]
    fn test_orca_swap_event_trait_metadata() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = OrcaSwapEvent::new(params, swap_data);

        let metadata = event.metadata();
        assert_eq!(metadata, &event.metadata.core);
    }

    #[test]
    fn test_orca_swap_event_trait_metadata_mut() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let mut event = OrcaSwapEvent::new(params, swap_data);

        let metadata_mut = event.metadata_mut();
        metadata_mut.id = "updated-swap-id".to_string();

        assert_eq!(event.metadata.id(), "updated-swap-id");
    }

    #[test]
    fn test_orca_swap_event_trait_as_any() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = OrcaSwapEvent::new(params, swap_data);

        let any = event.as_any();
        assert!(any.downcast_ref::<OrcaSwapEvent>().is_some());
    }

    #[test]
    fn test_orca_swap_event_trait_as_any_mut() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let mut event = OrcaSwapEvent::new(params, swap_data);

        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<OrcaSwapEvent>().is_some());
    }

    #[test]
    fn test_orca_swap_event_trait_clone_boxed() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = OrcaSwapEvent::new(params, swap_data);

        let boxed = event.clone_boxed();
        let downcast = boxed.as_any().downcast_ref::<OrcaSwapEvent>().unwrap();
        assert_eq!(event.id(), downcast.id());
    }

    #[test]
    fn test_orca_swap_event_trait_to_json() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = OrcaSwapEvent::new(params, swap_data);

        let json_result = event.to_json();
        assert!(json_result.is_ok());
        let json_value = json_result.unwrap();
        assert!(json_value.is_object());
    }

    // Tests for Event trait implementation on OrcaPositionEvent
    #[test]
    fn test_orca_position_event_trait_id() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = OrcaPositionEvent::new(params, position_data, true);

        assert_eq!(event.id(), event.metadata.id());
    }

    #[test]
    fn test_orca_position_event_trait_kind() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = OrcaPositionEvent::new(params, position_data, true);

        assert_eq!(event.kind(), event.metadata.kind());
    }

    #[test]
    fn test_orca_position_event_trait_metadata() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = OrcaPositionEvent::new(params, position_data, true);

        let metadata = event.metadata();
        assert_eq!(metadata, &event.metadata.core);
    }

    #[test]
    fn test_orca_position_event_trait_metadata_mut() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let mut event = OrcaPositionEvent::new(params, position_data, true);

        let metadata_mut = event.metadata_mut();
        metadata_mut.id = "updated-position-id".to_string();

        assert_eq!(event.metadata.id(), "updated-position-id");
    }

    #[test]
    fn test_orca_position_event_trait_as_any() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = OrcaPositionEvent::new(params, position_data, true);

        let any = event.as_any();
        assert!(any.downcast_ref::<OrcaPositionEvent>().is_some());
    }

    #[test]
    fn test_orca_position_event_trait_as_any_mut() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let mut event = OrcaPositionEvent::new(params, position_data, true);

        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<OrcaPositionEvent>().is_some());
    }

    #[test]
    fn test_orca_position_event_trait_clone_boxed() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = OrcaPositionEvent::new(params, position_data, true);

        let boxed = event.clone_boxed();
        let downcast = boxed.as_any().downcast_ref::<OrcaPositionEvent>().unwrap();
        assert_eq!(event.id(), downcast.id());
    }

    #[test]
    fn test_orca_position_event_trait_to_json() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = OrcaPositionEvent::new(params, position_data, true);

        let json_result = event.to_json();
        assert!(json_result.is_ok());
        let json_value = json_result.unwrap();
        assert!(json_value.is_object());
    }

    // Tests for Event trait implementation on OrcaLiquidityEvent
    #[test]
    fn test_orca_liquidity_event_trait_id() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let event = OrcaLiquidityEvent::new(params, liquidity_data);

        assert_eq!(event.id(), event.metadata.id());
    }

    #[test]
    fn test_orca_liquidity_event_trait_kind() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let event = OrcaLiquidityEvent::new(params, liquidity_data);

        assert_eq!(event.kind(), event.metadata.kind());
    }

    #[test]
    fn test_orca_liquidity_event_trait_metadata() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let event = OrcaLiquidityEvent::new(params, liquidity_data);

        let metadata = event.metadata();
        assert_eq!(metadata, &event.metadata.core);
    }

    #[test]
    fn test_orca_liquidity_event_trait_metadata_mut() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let mut event = OrcaLiquidityEvent::new(params, liquidity_data);

        let metadata_mut = event.metadata_mut();
        metadata_mut.id = "updated-liquidity-id".to_string();

        assert_eq!(event.metadata.id(), "updated-liquidity-id");
    }

    #[test]
    fn test_orca_liquidity_event_trait_as_any() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let event = OrcaLiquidityEvent::new(params, liquidity_data);

        let any = event.as_any();
        assert!(any.downcast_ref::<OrcaLiquidityEvent>().is_some());
    }

    #[test]
    fn test_orca_liquidity_event_trait_as_any_mut() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let mut event = OrcaLiquidityEvent::new(params, liquidity_data);

        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<OrcaLiquidityEvent>().is_some());
    }

    #[test]
    fn test_orca_liquidity_event_trait_clone_boxed() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let event = OrcaLiquidityEvent::new(params, liquidity_data);

        let boxed = event.clone_boxed();
        let downcast = boxed.as_any().downcast_ref::<OrcaLiquidityEvent>().unwrap();
        assert_eq!(event.id(), downcast.id());
    }

    #[test]
    fn test_orca_liquidity_event_trait_to_json() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let event = OrcaLiquidityEvent::new(params, liquidity_data);

        let json_result = event.to_json();
        assert!(json_result.is_ok());
        let json_value = json_result.unwrap();
        assert!(json_value.is_object());
    }

    // Edge case tests
    #[test]
    fn test_event_parameters_with_empty_strings() {
        let params =
            EventParameters::new("".to_string(), "".to_string(), 0, 0, 0, 0, "".to_string());

        assert_eq!(params.id, "");
        assert_eq!(params.signature, "");
        assert_eq!(params.index, "");
    }

    #[test]
    fn test_event_parameters_with_max_values() {
        let params = EventParameters::new(
            "max-id".to_string(),
            "max-signature".to_string(),
            u64::MAX,
            i64::MAX,
            i64::MAX,
            i64::MAX,
            "max-index".to_string(),
        );

        assert_eq!(params.slot, u64::MAX);
        assert_eq!(params.block_time, i64::MAX);
        assert_eq!(params.block_time_ms, i64::MAX);
        assert_eq!(params.program_received_time_ms, i64::MAX);
    }

    #[test]
    fn test_event_parameters_with_negative_values() {
        let params = EventParameters::new(
            "neg-id".to_string(),
            "neg-signature".to_string(),
            0,
            i64::MIN,
            i64::MIN,
            i64::MIN,
            "neg-index".to_string(),
        );

        assert_eq!(params.block_time, i64::MIN);
        assert_eq!(params.block_time_ms, i64::MIN);
        assert_eq!(params.program_received_time_ms, i64::MIN);
    }

    #[test]
    fn test_orca_swap_event_with_multiple_transfers() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let mut transfer_data = create_test_transfer_data();

        // Add more transfers
        for i in 3..10 {
            transfer_data.push(TransferData {
                source: Pubkey::from_str("11111111111111111111111111111112").unwrap(),
                destination: Pubkey::from_str("11111111111111111111111111111113").unwrap(),
                mint: None,
                amount: i * 100,
            });
        }

        let event = OrcaSwapEvent::new(params, swap_data).with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), transfer_data.len());
    }

    #[test]
    fn test_debug_format_implementations() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let position_data = create_test_position_data();
        let liquidity_data = create_test_liquidity_data();

        let swap_event = OrcaSwapEvent::new(params.clone(), swap_data);
        let position_event = OrcaPositionEvent::new(params.clone(), position_data, true);
        let liquidity_event = OrcaLiquidityEvent::new(params, liquidity_data);

        // Test that debug formatting works
        let swap_debug = format!("{:?}", swap_event);
        let position_debug = format!("{:?}", position_event);
        let liquidity_debug = format!("{:?}", liquidity_event);

        assert!(swap_debug.contains("OrcaSwapEvent"));
        assert!(position_debug.contains("OrcaPositionEvent"));
        assert!(liquidity_debug.contains("OrcaLiquidityEvent"));
    }

    // Test chaining methods
    #[test]
    fn test_method_chaining() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let transfer_data = create_test_transfer_data();

        let event = OrcaSwapEvent::new(params, swap_data)
            .with_transfer_data(transfer_data.clone())
            .with_transfer_data(vec![]); // Chain multiple calls

        assert!(event.transfer_data.is_empty()); // Last call should override
    }

    // Test that metadata field is excluded from serialization
    #[test]
    fn test_metadata_field_excluded_from_serialization() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = OrcaSwapEvent::new(params, swap_data);

        let serialized = serde_json::to_string(&event).unwrap();
        assert!(!serialized.contains("metadata"));
    }
}
