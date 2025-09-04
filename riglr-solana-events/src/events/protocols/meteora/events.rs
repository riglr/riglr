use super::types::{MeteoraDynamicLiquidityData, MeteoraLiquidityData, MeteoraSwapData};
use crate::solana_metadata::SolanaEventMetadata;
use crate::types::{metadata_helpers, EventType, ProtocolType, TransferData};
use riglr_events_core::EventMetadata as CoreEventMetadata;
use serde::{Deserialize, Serialize};
use std::any::Any;

// Import Event trait from riglr-events-core
use riglr_events_core::{Event, EventKind};

/// Parameters for creating event metadata, reducing function parameter count
#[derive(Debug, Clone, Default)]
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
    /// Time when the program received the event in milliseconds
    pub program_received_time_ms: i64,
    /// Event index within the transaction
    pub index: String,
}

impl EventParameters {
    /// Creates a new EventParameters instance with the provided values
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

/// Meteora DLMM swap event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MeteoraSwapEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Meteora swap-specific data
    pub swap_data: MeteoraSwapData,
    /// Token transfer data associated with the swap
    pub transfer_data: Vec<TransferData>,
}

impl MeteoraSwapEvent {
    /// Creates a new MeteoraSwapEvent with the provided parameters and swap data
    pub fn new(params: EventParameters, swap_data: MeteoraSwapData) -> Self {
        let metadata = metadata_helpers::create_solana_metadata(
            params.id,
            params.signature,
            params.slot,
            params.block_time,
            ProtocolType::MeteoraDlmm,
            EventType::Swap,
            super::types::meteora_dlmm_program_id(),
            params.index,
            params.program_received_time_ms,
        );

        Self {
            metadata,
            swap_data,
            transfer_data: Vec::new(),
        }
    }

    /// Sets the transfer data for this swap event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

// Event trait implementation
impl Event for MeteoraSwapEvent {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static SWAP_KIND: EventKind = EventKind::Swap;
        &SWAP_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        &mut self.metadata.core
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
        Ok(serde_json::to_value(self)?)
    }
}

/// Meteora DLMM liquidity event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MeteoraLiquidityEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Meteora liquidity-specific data
    pub liquidity_data: MeteoraLiquidityData,
    /// Token transfer data associated with the liquidity operation
    pub transfer_data: Vec<TransferData>,
}

impl MeteoraLiquidityEvent {
    /// Creates a new MeteoraLiquidityEvent with the provided parameters and liquidity data
    pub fn new(params: EventParameters, liquidity_data: MeteoraLiquidityData) -> Self {
        let metadata = metadata_helpers::create_solana_metadata(
            params.id,
            params.signature,
            params.slot,
            params.block_time,
            ProtocolType::MeteoraDlmm,
            EventType::AddLiquidity,
            super::types::meteora_dlmm_program_id(),
            params.index,
            params.program_received_time_ms,
        );

        Self {
            metadata,
            liquidity_data,
            transfer_data: Vec::new(),
        }
    }

    /// Sets the transfer data for this liquidity event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

// Event trait implementation for MeteoraLiquidityEvent
impl Event for MeteoraLiquidityEvent {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
        &LIQUIDITY_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        &mut self.metadata.core
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
        Ok(serde_json::to_value(self)?)
    }
}

/// Meteora Dynamic AMM liquidity event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MeteoraDynamicLiquidityEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Meteora dynamic liquidity-specific data
    pub liquidity_data: MeteoraDynamicLiquidityData,
    /// Token transfer data associated with the liquidity operation
    pub transfer_data: Vec<TransferData>,
}

impl MeteoraDynamicLiquidityEvent {
    /// Creates a new MeteoraDynamicLiquidityEvent with the provided parameters and liquidity data
    pub fn new(params: EventParameters, liquidity_data: MeteoraDynamicLiquidityData) -> Self {
        let metadata = metadata_helpers::create_solana_metadata(
            params.id,
            params.signature,
            params.slot,
            params.block_time,
            ProtocolType::MeteoraDlmm, // Using MeteoraDlmm for consistency
            EventType::AddLiquidity,
            super::types::meteora_dynamic_program_id(),
            params.index,
            params.program_received_time_ms,
        );

        Self {
            metadata,
            liquidity_data,
            transfer_data: Vec::new(),
        }
    }

    /// Sets the transfer data for this dynamic liquidity event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

// Event trait implementation for MeteoraDynamicLiquidityEvent
impl Event for MeteoraDynamicLiquidityEvent {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
        &LIQUIDITY_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        &mut self.metadata.core
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
        Ok(serde_json::to_value(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    // Helper function to create test EventParameters
    fn create_test_event_parameters() -> EventParameters {
        EventParameters {
            id: "test_id".to_string(),
            signature: "test_signature".to_string(),
            slot: 12345,
            block_time: 1234567890,
            block_time_ms: 1234567890000,
            program_received_time_ms: 1234567890001,
            index: "test_index".to_string(),
        }
    }

    // Helper function to create test MeteoraSwapData
    fn create_test_meteora_swap_data() -> MeteoraSwapData {
        MeteoraSwapData {
            pair: Pubkey::from_str("11111111111111111111111111111112").unwrap(),
            user: Pubkey::from_str("11111111111111111111111111111113").unwrap(),
            token_mint_x: Pubkey::from_str("11111111111111111111111111111114").unwrap(),
            token_mint_y: Pubkey::from_str("11111111111111111111111111111115").unwrap(),
            reserve_x: Pubkey::from_str("11111111111111111111111111111116").unwrap(),
            reserve_y: Pubkey::from_str("11111111111111111111111111111117").unwrap(),
            amount_in: 1000,
            min_amount_out: 950,
            actual_amount_out: 980,
            swap_for_y: true,
            active_id_before: 8388608,
            active_id_after: 8388609,
            fee_amount: 10,
            protocol_fee: 5,
            bins_traversed: vec![8388608, 8388609],
        }
    }

    // Helper function to create test MeteoraLiquidityData
    fn create_test_meteora_liquidity_data() -> MeteoraLiquidityData {
        MeteoraLiquidityData {
            pair: Pubkey::from_str("11111111111111111111111111111112").unwrap(),
            user: Pubkey::from_str("11111111111111111111111111111113").unwrap(),
            position: Pubkey::from_str("11111111111111111111111111111114").unwrap(),
            token_mint_x: Pubkey::from_str("11111111111111111111111111111115").unwrap(),
            token_mint_y: Pubkey::from_str("11111111111111111111111111111116").unwrap(),
            reserve_x: Pubkey::from_str("11111111111111111111111111111117").unwrap(),
            reserve_y: Pubkey::from_str("11111111111111111111111111111118").unwrap(),
            bin_id_from: 100,
            bin_id_to: 105,
            amount_x: 1000,
            amount_y: 2000,
            liquidity_minted: 50000,
            active_id: 102,
            is_add: true,
            bins_affected: vec![],
        }
    }

    // Helper function to create test MeteoraDynamicLiquidityData
    fn create_test_meteora_dynamic_liquidity_data() -> MeteoraDynamicLiquidityData {
        MeteoraDynamicLiquidityData {
            pool: Pubkey::from_str("11111111111111111111111111111112").unwrap(),
            user: Pubkey::from_str("11111111111111111111111111111113").unwrap(),
            token_mint_a: Pubkey::from_str("11111111111111111111111111111114").unwrap(),
            token_mint_b: Pubkey::from_str("11111111111111111111111111111115").unwrap(),
            vault_a: Pubkey::from_str("11111111111111111111111111111116").unwrap(),
            vault_b: Pubkey::from_str("11111111111111111111111111111117").unwrap(),
            lp_mint: Pubkey::from_str("11111111111111111111111111111118").unwrap(),
            pool_token_amount: 1000,
            token_a_amount: 500,
            token_b_amount: 750,
            minimum_pool_token_amount: 900,
            maximum_token_a_amount: 550,
            maximum_token_b_amount: 800,
            is_deposit: true,
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
                mint: None, // SOL transfer
                amount: 500,
            },
        ]
    }

    // EventParameters tests
    #[test]
    fn test_event_parameters_new_when_valid_inputs_should_create_correctly() {
        let params = EventParameters::new(
            "test_id".to_string(),
            "test_sig".to_string(),
            12345,
            1234567890,
            1234567890000,
            1234567890001,
            "test_index".to_string(),
        );

        assert_eq!(params.id, "test_id");
        assert_eq!(params.signature, "test_sig");
        assert_eq!(params.slot, 12345);
        assert_eq!(params.block_time, 1234567890);
        assert_eq!(params.block_time_ms, 1234567890000);
        assert_eq!(params.program_received_time_ms, 1234567890001);
        assert_eq!(params.index, "test_index");
    }

    #[test]
    fn test_event_parameters_new_when_empty_strings_should_create_correctly() {
        let params =
            EventParameters::new("".to_string(), "".to_string(), 0, 0, 0, 0, "".to_string());

        assert_eq!(params.id, "");
        assert_eq!(params.signature, "");
        assert_eq!(params.slot, 0);
        assert_eq!(params.block_time, 0);
        assert_eq!(params.block_time_ms, 0);
        assert_eq!(params.program_received_time_ms, 0);
        assert_eq!(params.index, "");
    }

    #[test]
    fn test_event_parameters_new_when_max_values_should_create_correctly() {
        let params = EventParameters::new(
            "max_id".to_string(),
            "max_sig".to_string(),
            u64::MAX,
            i64::MAX,
            i64::MAX,
            i64::MAX,
            "max_index".to_string(),
        );

        assert_eq!(params.id, "max_id");
        assert_eq!(params.signature, "max_sig");
        assert_eq!(params.slot, u64::MAX);
        assert_eq!(params.block_time, i64::MAX);
        assert_eq!(params.block_time_ms, i64::MAX);
        assert_eq!(params.program_received_time_ms, i64::MAX);
        assert_eq!(params.index, "max_index");
    }

    #[test]
    fn test_event_parameters_new_when_negative_times_should_create_correctly() {
        let params = EventParameters::new(
            "neg_id".to_string(),
            "neg_sig".to_string(),
            100,
            -1234567890,
            -1234567890000,
            -1234567890001,
            "neg_index".to_string(),
        );

        assert_eq!(params.block_time, -1234567890);
        assert_eq!(params.block_time_ms, -1234567890000);
        assert_eq!(params.program_received_time_ms, -1234567890001);
    }

    #[test]
    fn test_event_parameters_default_when_called_should_create_empty() {
        let params = EventParameters::default();

        assert_eq!(params.id, "");
        assert_eq!(params.signature, "");
        assert_eq!(params.slot, 0);
        assert_eq!(params.block_time, 0);
        assert_eq!(params.block_time_ms, 0);
        assert_eq!(params.program_received_time_ms, 0);
        assert_eq!(params.index, "");
    }

    // MeteoraSwapEvent tests
    #[test]
    fn test_meteora_swap_event_new_when_valid_params_should_create_correctly() {
        let params = create_test_event_parameters();
        let swap_data = create_test_meteora_swap_data();
        let event = MeteoraSwapEvent::new(params.clone(), swap_data.clone());

        assert_eq!(event.metadata.core.id, params.id);
        assert_eq!(event.metadata.signature, params.signature);
        assert_eq!(event.metadata.slot, params.slot);
        assert_eq!(event.metadata.index, params.index);
        assert_eq!(
            event.metadata.program_received_time_ms,
            params.program_received_time_ms
        );
        assert_eq!(event.swap_data.pair, swap_data.pair);
        assert_eq!(event.swap_data.amount_in, swap_data.amount_in);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_meteora_swap_event_with_transfer_data_when_valid_data_should_set_correctly() {
        let params = create_test_event_parameters();
        let swap_data = create_test_meteora_swap_data();
        let transfer_data = create_test_transfer_data();

        let event =
            MeteoraSwapEvent::new(params, swap_data).with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 2);
        assert_eq!(event.transfer_data[0].amount, 1000);
        assert_eq!(event.transfer_data[1].amount, 500);
        assert_eq!(
            event.transfer_data[0].mint,
            Some(Pubkey::from_str("11111111111111111111111111111114").unwrap())
        );
        assert_eq!(event.transfer_data[1].mint, None);
    }

    #[test]
    fn test_meteora_swap_event_with_transfer_data_when_empty_vec_should_set_empty() {
        let params = create_test_event_parameters();
        let swap_data = create_test_meteora_swap_data();

        let event = MeteoraSwapEvent::new(params, swap_data).with_transfer_data(vec![]);

        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_meteora_swap_event_default_when_called_should_create_empty() {
        let event = MeteoraSwapEvent::default();

        assert_eq!(event.metadata.core.id, "");
        assert_eq!(event.metadata.signature, "");
        assert_eq!(event.metadata.slot, 0);
        assert_eq!(event.metadata.index, "");
        assert_eq!(event.metadata.program_received_time_ms, 0);
        assert_eq!(event.swap_data.amount_in, 0);
        assert!(event.transfer_data.is_empty());
    }

    // MeteoraSwapEvent Event trait implementation tests
    #[test]
    fn test_meteora_swap_event_id_when_called_should_return_metadata_id() {
        let params = create_test_event_parameters();
        let swap_data = create_test_meteora_swap_data();
        let event = MeteoraSwapEvent::new(params, swap_data);

        // The id() method returns from metadata, not the struct field
        let result = event.id();
        assert_eq!(result, &event.metadata.core.id);
    }

    #[test]
    fn test_meteora_swap_event_kind_when_called_should_return_metadata_kind() {
        let params = create_test_event_parameters();
        let swap_data = create_test_meteora_swap_data();
        let event = MeteoraSwapEvent::new(params, swap_data);

        let result = event.kind();
        assert_eq!(result, &event.metadata.core.kind);
    }

    #[test]
    fn test_meteora_swap_event_metadata_when_called_should_return_metadata_ref() {
        let params = create_test_event_parameters();
        let swap_data = create_test_meteora_swap_data();
        let event = MeteoraSwapEvent::new(params, swap_data);

        let result = event.metadata();
        assert_eq!(result as *const _, &event.metadata.core as *const _);
    }

    #[test]
    fn test_meteora_swap_event_metadata_mut_when_called_should_return_mutable_ref() {
        let params = create_test_event_parameters();
        let swap_data = create_test_meteora_swap_data();
        let mut event = MeteoraSwapEvent::new(params, swap_data);

        let result = event.metadata_mut();
        assert_eq!(result as *mut _, &mut event.metadata.core as *mut _);
    }

    #[test]
    fn test_meteora_swap_event_as_any_when_called_should_return_any_ref() {
        let params = create_test_event_parameters();
        let swap_data = create_test_meteora_swap_data();
        let event = MeteoraSwapEvent::new(params, swap_data);

        let result = event.as_any();
        assert!(result.downcast_ref::<MeteoraSwapEvent>().is_some());
    }

    #[test]
    fn test_meteora_swap_event_as_any_mut_when_called_should_return_mutable_any_ref() {
        let params = create_test_event_parameters();
        let swap_data = create_test_meteora_swap_data();
        let mut event = MeteoraSwapEvent::new(params, swap_data);

        let result = event.as_any_mut();
        assert!(result.downcast_mut::<MeteoraSwapEvent>().is_some());
    }

    #[test]
    fn test_meteora_swap_event_clone_boxed_when_called_should_return_boxed_clone() {
        let params = create_test_event_parameters();
        let swap_data = create_test_meteora_swap_data();
        let event = MeteoraSwapEvent::new(params, swap_data);

        let boxed = event.clone_boxed();
        let cloned = boxed.as_any().downcast_ref::<MeteoraSwapEvent>().unwrap();

        assert_eq!(event.metadata.core.id, cloned.metadata.core.id);
        assert_eq!(event.metadata.signature, cloned.metadata.signature);
        assert_eq!(event.metadata.slot, cloned.metadata.slot);
    }

    #[test]
    fn test_meteora_swap_event_to_json_when_valid_event_should_serialize_successfully() {
        let params = create_test_event_parameters();
        let swap_data = create_test_meteora_swap_data();
        let event = MeteoraSwapEvent::new(params, swap_data);

        let result = event.to_json();
        assert!(result.is_ok());

        let json_value = result.unwrap();
        assert!(json_value.is_object());
        assert!(json_value.get("id").is_some());
        assert!(json_value.get("signature").is_some());
        assert!(json_value.get("swap_data").is_some());
    }

    // MeteoraLiquidityEvent tests
    #[test]
    fn test_meteora_liquidity_event_new_when_valid_params_should_create_correctly() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_liquidity_data();
        let event = MeteoraLiquidityEvent::new(params.clone(), liquidity_data.clone());

        assert_eq!(event.metadata.core.id, params.id);
        assert_eq!(event.metadata.signature, params.signature);
        assert_eq!(event.metadata.slot, params.slot);
        assert_eq!(event.metadata.index, params.index);
        assert_eq!(
            event.metadata.program_received_time_ms,
            params.program_received_time_ms
        );
        assert_eq!(event.liquidity_data.pair, liquidity_data.pair);
        assert_eq!(event.liquidity_data.amount_x, liquidity_data.amount_x);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_meteora_liquidity_event_with_transfer_data_when_valid_data_should_set_correctly() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_liquidity_data();
        let transfer_data = create_test_transfer_data();

        let event = MeteoraLiquidityEvent::new(params, liquidity_data)
            .with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 2);
        assert_eq!(event.transfer_data[0].amount, 1000);
        assert_eq!(event.transfer_data[1].amount, 500);
    }

    #[test]
    fn test_meteora_liquidity_event_with_transfer_data_when_empty_vec_should_set_empty() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_liquidity_data();

        let event = MeteoraLiquidityEvent::new(params, liquidity_data).with_transfer_data(vec![]);

        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_meteora_liquidity_event_default_when_called_should_create_empty() {
        let event = MeteoraLiquidityEvent::default();

        assert_eq!(event.metadata.core.id, "");
        assert_eq!(event.metadata.signature, "");
        assert_eq!(event.metadata.slot, 0);
        assert_eq!(event.metadata.index, "");
        assert_eq!(event.metadata.program_received_time_ms, 0);
        assert_eq!(event.liquidity_data.amount_x, 0);
        assert!(event.transfer_data.is_empty());
    }

    // MeteoraLiquidityEvent Event trait implementation tests
    #[test]
    fn test_meteora_liquidity_event_id_when_called_should_return_metadata_id() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_liquidity_data();
        let event = MeteoraLiquidityEvent::new(params, liquidity_data);

        let result = event.id();
        assert_eq!(result, &event.metadata.core.id);
    }

    #[test]
    fn test_meteora_liquidity_event_kind_when_called_should_return_metadata_kind() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_liquidity_data();
        let event = MeteoraLiquidityEvent::new(params, liquidity_data);

        let result = event.kind();
        assert_eq!(result, &event.metadata.core.kind);
    }

    #[test]
    fn test_meteora_liquidity_event_metadata_when_called_should_return_metadata_ref() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_liquidity_data();
        let event = MeteoraLiquidityEvent::new(params, liquidity_data);

        let result = event.metadata();
        assert_eq!(result as *const _, &event.metadata.core as *const _);
    }

    #[test]
    fn test_meteora_liquidity_event_metadata_mut_when_called_should_return_mutable_ref() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_liquidity_data();
        let mut event = MeteoraLiquidityEvent::new(params, liquidity_data);

        let result = event.metadata_mut();
        assert_eq!(result as *mut _, &mut event.metadata.core as *mut _);
    }

    #[test]
    fn test_meteora_liquidity_event_as_any_when_called_should_return_any_ref() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_liquidity_data();
        let event = MeteoraLiquidityEvent::new(params, liquidity_data);

        let result = event.as_any();
        assert!(result.downcast_ref::<MeteoraLiquidityEvent>().is_some());
    }

    #[test]
    fn test_meteora_liquidity_event_as_any_mut_when_called_should_return_mutable_any_ref() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_liquidity_data();
        let mut event = MeteoraLiquidityEvent::new(params, liquidity_data);

        let result = event.as_any_mut();
        assert!(result.downcast_mut::<MeteoraLiquidityEvent>().is_some());
    }

    #[test]
    fn test_meteora_liquidity_event_clone_boxed_when_called_should_return_boxed_clone() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_liquidity_data();
        let event = MeteoraLiquidityEvent::new(params, liquidity_data);

        let boxed = event.clone_boxed();
        let cloned = boxed
            .as_any()
            .downcast_ref::<MeteoraLiquidityEvent>()
            .unwrap();

        assert_eq!(event.metadata.core.id, cloned.metadata.core.id);
        assert_eq!(event.metadata.signature, cloned.metadata.signature);
        assert_eq!(event.metadata.slot, cloned.metadata.slot);
    }

    #[test]
    fn test_meteora_liquidity_event_to_json_when_valid_event_should_serialize_successfully() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_liquidity_data();
        let event = MeteoraLiquidityEvent::new(params, liquidity_data);

        let result = event.to_json();
        assert!(result.is_ok());

        let json_value = result.unwrap();
        assert!(json_value.is_object());
        assert!(json_value.get("id").is_some());
        assert!(json_value.get("signature").is_some());
        assert!(json_value.get("liquidity_data").is_some());
    }

    // MeteoraDynamicLiquidityEvent tests
    #[test]
    fn test_meteora_dynamic_liquidity_event_new_when_valid_params_should_create_correctly() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_dynamic_liquidity_data();
        let event = MeteoraDynamicLiquidityEvent::new(params.clone(), liquidity_data.clone());

        assert_eq!(event.metadata.core.id, params.id);
        assert_eq!(event.metadata.signature, params.signature);
        assert_eq!(event.metadata.slot, params.slot);
        assert_eq!(event.metadata.index, params.index);
        assert_eq!(
            event.metadata.program_received_time_ms,
            params.program_received_time_ms
        );
        assert_eq!(event.liquidity_data.pool, liquidity_data.pool);
        assert_eq!(
            event.liquidity_data.pool_token_amount,
            liquidity_data.pool_token_amount
        );
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_meteora_dynamic_liquidity_event_with_transfer_data_when_valid_data_should_set_correctly(
    ) {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_dynamic_liquidity_data();
        let transfer_data = create_test_transfer_data();

        let event = MeteoraDynamicLiquidityEvent::new(params, liquidity_data)
            .with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 2);
        assert_eq!(event.transfer_data[0].amount, 1000);
        assert_eq!(event.transfer_data[1].amount, 500);
    }

    #[test]
    fn test_meteora_dynamic_liquidity_event_with_transfer_data_when_empty_vec_should_set_empty() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_dynamic_liquidity_data();

        let event =
            MeteoraDynamicLiquidityEvent::new(params, liquidity_data).with_transfer_data(vec![]);

        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_meteora_dynamic_liquidity_event_default_when_called_should_create_empty() {
        let event = MeteoraDynamicLiquidityEvent::default();

        assert_eq!(event.metadata.core.id, "");
        assert_eq!(event.metadata.signature, "");
        assert_eq!(event.metadata.slot, 0);
        assert_eq!(event.metadata.index, "");
        assert_eq!(event.metadata.program_received_time_ms, 0);
        assert_eq!(event.liquidity_data.pool_token_amount, 0);
        assert!(event.transfer_data.is_empty());
    }

    // MeteoraDynamicLiquidityEvent Event trait implementation tests
    #[test]
    fn test_meteora_dynamic_liquidity_event_id_when_called_should_return_metadata_id() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_dynamic_liquidity_data();
        let event = MeteoraDynamicLiquidityEvent::new(params, liquidity_data);

        let result = event.id();
        assert_eq!(result, &event.metadata.core.id);
    }

    #[test]
    fn test_meteora_dynamic_liquidity_event_kind_when_called_should_return_metadata_kind() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_dynamic_liquidity_data();
        let event = MeteoraDynamicLiquidityEvent::new(params, liquidity_data);

        let result = event.kind();
        assert_eq!(result, &event.metadata.core.kind);
    }

    #[test]
    fn test_meteora_dynamic_liquidity_event_metadata_when_called_should_return_metadata_ref() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_dynamic_liquidity_data();
        let event = MeteoraDynamicLiquidityEvent::new(params, liquidity_data);

        let result = event.metadata();
        assert_eq!(result as *const _, &event.metadata.core as *const _);
    }

    #[test]
    fn test_meteora_dynamic_liquidity_event_metadata_mut_when_called_should_return_mutable_ref() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_dynamic_liquidity_data();
        let mut event = MeteoraDynamicLiquidityEvent::new(params, liquidity_data);

        let result = event.metadata_mut();
        assert_eq!(result as *mut _, &mut event.metadata.core as *mut _);
    }

    #[test]
    fn test_meteora_dynamic_liquidity_event_as_any_when_called_should_return_any_ref() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_dynamic_liquidity_data();
        let event = MeteoraDynamicLiquidityEvent::new(params, liquidity_data);

        let result = event.as_any();
        assert!(result
            .downcast_ref::<MeteoraDynamicLiquidityEvent>()
            .is_some());
    }

    #[test]
    fn test_meteora_dynamic_liquidity_event_as_any_mut_when_called_should_return_mutable_any_ref() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_dynamic_liquidity_data();
        let mut event = MeteoraDynamicLiquidityEvent::new(params, liquidity_data);

        let result = event.as_any_mut();
        assert!(result
            .downcast_mut::<MeteoraDynamicLiquidityEvent>()
            .is_some());
    }

    #[test]
    fn test_meteora_dynamic_liquidity_event_clone_boxed_when_called_should_return_boxed_clone() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_dynamic_liquidity_data();
        let event = MeteoraDynamicLiquidityEvent::new(params, liquidity_data);

        let boxed = event.clone_boxed();
        let cloned = boxed
            .as_any()
            .downcast_ref::<MeteoraDynamicLiquidityEvent>()
            .unwrap();

        assert_eq!(event.metadata.core.id, cloned.metadata.core.id);
        assert_eq!(event.metadata.signature, cloned.metadata.signature);
        assert_eq!(event.metadata.slot, cloned.metadata.slot);
    }

    #[test]
    fn test_meteora_dynamic_liquidity_event_to_json_when_valid_event_should_serialize_successfully()
    {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_meteora_dynamic_liquidity_data();
        let event = MeteoraDynamicLiquidityEvent::new(params, liquidity_data);

        let result = event.to_json();
        assert!(result.is_ok());

        let json_value = result.unwrap();
        assert!(json_value.is_object());
        assert!(json_value.get("id").is_some());
        assert!(json_value.get("signature").is_some());
        assert!(json_value.get("liquidity_data").is_some());
    }

    // Edge case tests for all event types
    #[test]
    fn test_all_events_when_created_with_default_params_should_have_valid_metadata() {
        let params = EventParameters::default();
        let swap_data = MeteoraSwapData::default();
        let liquidity_data = MeteoraLiquidityData::default();
        let dynamic_liquidity_data = MeteoraDynamicLiquidityData::default();

        let swap_event = MeteoraSwapEvent::new(params.clone(), swap_data);
        let liquidity_event = MeteoraLiquidityEvent::new(params.clone(), liquidity_data);
        let dynamic_event = MeteoraDynamicLiquidityEvent::new(params, dynamic_liquidity_data);

        // Verify metadata is properly initialized
        assert_eq!(swap_event.metadata.core.id, "");
        assert_eq!(liquidity_event.metadata.core.id, "");
        assert_eq!(dynamic_event.metadata.core.id, "");
    }

    #[test]
    fn test_all_events_when_chaining_with_transfer_data_should_preserve_original_fields() {
        let params = create_test_event_parameters();
        let swap_data = create_test_meteora_swap_data();
        let liquidity_data = create_test_meteora_liquidity_data();
        let dynamic_liquidity_data = create_test_meteora_dynamic_liquidity_data();
        let transfer_data = create_test_transfer_data();

        let swap_event = MeteoraSwapEvent::new(params.clone(), swap_data.clone())
            .with_transfer_data(transfer_data.clone());
        let liquidity_event = MeteoraLiquidityEvent::new(params.clone(), liquidity_data.clone())
            .with_transfer_data(transfer_data.clone());
        let dynamic_event =
            MeteoraDynamicLiquidityEvent::new(params.clone(), dynamic_liquidity_data.clone())
                .with_transfer_data(transfer_data.clone());

        // Check that original fields are preserved after chaining
        assert_eq!(swap_event.metadata.core.id, params.id);
        assert_eq!(swap_event.swap_data.amount_in, swap_data.amount_in);
        assert_eq!(liquidity_event.metadata.core.id, params.id);
        assert_eq!(
            liquidity_event.liquidity_data.amount_x,
            liquidity_data.amount_x
        );
        assert_eq!(dynamic_event.metadata.core.id, params.id);
        assert_eq!(
            dynamic_event.liquidity_data.pool_token_amount,
            dynamic_liquidity_data.pool_token_amount
        );
    }

    #[test]
    fn test_all_events_serialization_when_valid_should_succeed() {
        let params = create_test_event_parameters();
        let swap_data = create_test_meteora_swap_data();
        let liquidity_data = create_test_meteora_liquidity_data();
        let dynamic_liquidity_data = create_test_meteora_dynamic_liquidity_data();

        let swap_event = MeteoraSwapEvent::new(params.clone(), swap_data);
        let liquidity_event = MeteoraLiquidityEvent::new(params.clone(), liquidity_data);
        let dynamic_event = MeteoraDynamicLiquidityEvent::new(params, dynamic_liquidity_data);

        // Test serialization
        let swap_json = serde_json::to_string(&swap_event);
        let liquidity_json = serde_json::to_string(&liquidity_event);
        let dynamic_json = serde_json::to_string(&dynamic_event);

        assert!(swap_json.is_ok());
        assert!(liquidity_json.is_ok());
        assert!(dynamic_json.is_ok());
    }

    #[test]
    fn test_all_events_deserialization_when_valid_json_should_succeed() {
        let params = create_test_event_parameters();
        let swap_data = create_test_meteora_swap_data();
        let liquidity_data = create_test_meteora_liquidity_data();
        let dynamic_liquidity_data = create_test_meteora_dynamic_liquidity_data();

        let swap_event = MeteoraSwapEvent::new(params.clone(), swap_data);
        let liquidity_event = MeteoraLiquidityEvent::new(params.clone(), liquidity_data);
        let dynamic_event = MeteoraDynamicLiquidityEvent::new(params, dynamic_liquidity_data);

        // Serialize then deserialize
        let swap_json = serde_json::to_string(&swap_event).unwrap();
        let liquidity_json = serde_json::to_string(&liquidity_event).unwrap();
        let dynamic_json = serde_json::to_string(&dynamic_event).unwrap();

        let deserialized_swap: Result<MeteoraSwapEvent, _> = serde_json::from_str(&swap_json);
        let deserialized_liquidity: Result<MeteoraLiquidityEvent, _> =
            serde_json::from_str(&liquidity_json);
        let deserialized_dynamic: Result<MeteoraDynamicLiquidityEvent, _> =
            serde_json::from_str(&dynamic_json);

        assert!(deserialized_swap.is_ok());
        assert!(deserialized_liquidity.is_ok());
        assert!(deserialized_dynamic.is_ok());

        let swap = deserialized_swap.unwrap();
        let liquidity = deserialized_liquidity.unwrap();
        let dynamic = deserialized_dynamic.unwrap();

        assert_eq!(swap.metadata.core.id, swap_event.metadata.core.id);
        assert_eq!(liquidity.metadata.core.id, liquidity_event.metadata.core.id);
        assert_eq!(dynamic.metadata.core.id, dynamic_event.metadata.core.id);
    }
}
