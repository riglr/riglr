use super::types::JupiterSwapData;
use crate::events::core::EventParameters;
use crate::solana_metadata::SolanaEventMetadata;
use crate::types::{metadata_helpers, EventType, ProtocolType, TransferData};
use borsh::BorshDeserialize;
use riglr_events_core::EventMetadata as CoreEventMetadata;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::any::Any;

// Import new Event trait from riglr-events-core
use riglr_events_core::{Event, EventKind};

// EventParameters is now imported from crate::events::core

/// Jupiter swap event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JupiterSwapEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Jupiter-specific swap data
    pub swap_data: JupiterSwapData,
    /// Associated token transfer data
    pub transfer_data: Vec<TransferData>,
}

impl JupiterSwapEvent {
    /// Creates a new JupiterSwapEvent with the provided parameters and swap data
    pub fn new(params: EventParameters, swap_data: JupiterSwapData) -> Self {
        let metadata = metadata_helpers::create_solana_metadata(
            params.id,
            params.signature,
            params.slot,
            params.block_time,
            ProtocolType::Jupiter,
            EventType::Swap,
            super::types::jupiter_v6_program_id(),
            params.index,
            params.program_received_time_ms,
        );

        Self {
            metadata,
            swap_data,
            transfer_data: Vec::default(),
        }
    }

    /// Adds transfer data to the swap event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

impl JupiterLiquidityEvent {
    /// Creates a new JupiterLiquidityEvent with the provided parameters
    pub fn new(params: EventParameters) -> Self {
        let metadata = metadata_helpers::create_solana_metadata(
            params.id,
            params.signature,
            params.slot,
            params.block_time,
            ProtocolType::Jupiter,
            EventType::AddLiquidity,
            super::types::jupiter_v6_program_id(),
            params.index,
            params.program_received_time_ms,
        );

        Self {
            metadata,
            user: solana_sdk::pubkey::Pubkey::default(),
            mint_a: solana_sdk::pubkey::Pubkey::default(),
            mint_b: solana_sdk::pubkey::Pubkey::default(),
            amount_a: 0,
            amount_b: 0,
            liquidity_amount: 0,
            is_remove: false,
            transfer_data: Vec::default(),
        }
    }

    /// Sets user account and returns self for method chaining
    pub fn with_user(mut self, user: solana_sdk::pubkey::Pubkey) -> Self {
        self.user = user;
        self
    }

    /// Sets token mints and returns self for method chaining
    pub fn with_mints(
        mut self,
        mint_a: solana_sdk::pubkey::Pubkey,
        mint_b: solana_sdk::pubkey::Pubkey,
    ) -> Self {
        self.mint_a = mint_a;
        self.mint_b = mint_b;
        self
    }

    /// Sets amounts and returns self for method chaining
    pub fn with_amounts(mut self, amount_a: u64, amount_b: u64, liquidity_amount: u64) -> Self {
        self.amount_a = amount_a;
        self.amount_b = amount_b;
        self.liquidity_amount = liquidity_amount;
        self
    }

    /// Sets removal flag and returns self for method chaining
    pub fn with_removal(mut self, is_remove: bool) -> Self {
        self.is_remove = is_remove;
        self
    }

    /// Adds transfer data to the event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

/// Jupiter liquidity provision event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JupiterLiquidityEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// User account providing/removing liquidity
    pub user: solana_sdk::pubkey::Pubkey,
    /// First token mint address
    pub mint_a: solana_sdk::pubkey::Pubkey,
    /// Second token mint address
    pub mint_b: solana_sdk::pubkey::Pubkey,
    /// Amount of token A
    pub amount_a: u64,
    /// Amount of token B
    pub amount_b: u64,
    /// Amount of liquidity tokens
    pub liquidity_amount: u64,
    /// Whether this is a liquidity removal operation
    pub is_remove: bool,
    /// Associated token transfer data
    pub transfer_data: Vec<TransferData>,
}

// UnifiedEvent implementation removed - now using Event trait

/// Jupiter swap event with borsh (for simple events)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
pub struct JupiterSwapBorshEvent {
    /// Event metadata (skipped during serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: SolanaEventMetadata,
    /// User account performing the swap
    pub user: Pubkey,
    /// Input token mint address
    pub input_mint: Pubkey,
    /// Output token mint address
    pub output_mint: Pubkey,
    /// Input token amount
    pub input_amount: u64,
    /// Output token amount received
    pub output_amount: u64,
    /// Slippage tolerance in basis points
    pub slippage_bps: u16,
    /// Platform fee in basis points
    pub platform_fee_bps: u8,
}

// JupiterSwapBorshEvent uses Event trait

// Event trait implementation for JupiterSwapEvent
impl Event for JupiterSwapEvent {
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

    fn metadata_mut(&mut self) -> riglr_events_core::error::EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
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

// Event trait implementation for JupiterLiquidityEvent
impl Event for JupiterLiquidityEvent {
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

    fn metadata_mut(&mut self) -> riglr_events_core::error::EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
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

// New Event trait implementation for JupiterSwapBorshEvent
impl Event for JupiterSwapBorshEvent {
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

    fn metadata_mut(&mut self) -> riglr_events_core::error::EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
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
    use crate::events::protocols::jupiter::types::RoutePlan;

    fn create_test_pubkey() -> Pubkey {
        Pubkey::new_unique()
    }

    fn create_test_transfer_data() -> TransferData {
        TransferData {
            source: create_test_pubkey(),
            destination: create_test_pubkey(),
            mint: Some(create_test_pubkey()),
            amount: 1000,
        }
    }

    fn create_test_jupiter_swap_data() -> JupiterSwapData {
        JupiterSwapData {
            user: create_test_pubkey(),
            input_mint: create_test_pubkey(),
            output_mint: create_test_pubkey(),
            input_amount: 1000,
            output_amount: 900,
            price_impact_pct: Some("0.5".to_string()),
            platform_fee_bps: Some(10),
            route_plan: vec![RoutePlan {
                input_mint: create_test_pubkey(),
                output_mint: create_test_pubkey(),
                amount_in: 1000,
                amount_out: 900,
                dex_label: "Orca".to_string(),
            }],
        }
    }

    fn create_test_event_parameters() -> EventParameters {
        EventParameters {
            id: "test-id".to_string(),
            signature: "test-signature".to_string(),
            slot: 12345,
            block_time: 1640995200,
            block_time_ms: 1640995200000,
            program_received_time_ms: 1640995201000,
            index: "0".to_string(),
        }
    }

    // EventParameters tests
    #[test]
    fn test_event_parameters_new_should_create_with_all_fields() {
        let params = EventParameters::new(
            "test-id".to_string(),
            "test-signature".to_string(),
            12345,
            1640995200,
            1640995200000,
            1640995201000,
            "0".to_string(),
        );

        assert_eq!(params.id, "test-id");
        assert_eq!(params.signature, "test-signature");
        assert_eq!(params.slot, 12345);
        assert_eq!(params.block_time, 1640995200);
        assert_eq!(params.block_time_ms, 1640995200000);
        assert_eq!(params.program_received_time_ms, 1640995201000);
        assert_eq!(params.index, "0");
    }

    #[test]
    fn test_event_parameters_default_should_create_empty() {
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
    fn test_event_parameters_clone_should_create_identical_copy() {
        let params1 = create_test_event_parameters();
        let params2 = params1.clone();

        assert_eq!(params1.id, params2.id);
        assert_eq!(params1.signature, params2.signature);
        assert_eq!(params1.slot, params2.slot);
        assert_eq!(params1.block_time, params2.block_time);
        assert_eq!(params1.block_time_ms, params2.block_time_ms);
        assert_eq!(
            params1.program_received_time_ms,
            params2.program_received_time_ms
        );
        assert_eq!(params1.index, params2.index);
    }

    // JupiterSwapEvent tests
    #[test]
    fn test_jupiter_swap_event_new_should_create_with_parameters_and_swap_data() {
        let params = create_test_event_parameters();
        let swap_data = create_test_jupiter_swap_data();

        let event = JupiterSwapEvent::new(params.clone(), swap_data.clone());

        assert_eq!(event.metadata.core.id, params.id);
        assert_eq!(event.metadata.signature, params.signature);
        assert_eq!(event.metadata.slot, params.slot);
        assert_eq!(event.metadata.index, params.index);
        assert_eq!(
            event.metadata.program_received_time_ms,
            params.program_received_time_ms
        );
        assert_eq!(event.swap_data.user, swap_data.user);
        assert_eq!(event.transfer_data.len(), 0);
    }

    #[test]
    fn test_jupiter_swap_event_with_transfer_data_should_add_transfers() {
        let params = create_test_event_parameters();
        let swap_data = create_test_jupiter_swap_data();
        let transfer_data = vec![create_test_transfer_data()];

        let event =
            JupiterSwapEvent::new(params, swap_data).with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 1);
        assert_eq!(event.transfer_data[0].amount, transfer_data[0].amount);
    }

    #[test]
    fn test_jupiter_swap_event_with_empty_transfer_data_should_have_empty_vec() {
        let params = create_test_event_parameters();
        let swap_data = create_test_jupiter_swap_data();

        let event = JupiterSwapEvent::new(params, swap_data).with_transfer_data(vec![]);

        assert_eq!(event.transfer_data.len(), 0);
    }

    #[test]
    fn test_jupiter_swap_event_default_should_create_empty() {
        let event = JupiterSwapEvent::default();

        assert_eq!(event.metadata.core.id, "");
        assert_eq!(event.metadata.signature, "");
        assert_eq!(event.metadata.slot, 0);
        assert_eq!(event.metadata.index, "");
        assert_eq!(event.metadata.program_received_time_ms, 0);
        assert_eq!(event.transfer_data.len(), 0);
    }

    #[test]
    fn test_jupiter_swap_event_clone_should_create_identical_copy() {
        let params = create_test_event_parameters();
        let swap_data = create_test_jupiter_swap_data();
        let transfer_data = vec![create_test_transfer_data()];

        let event1 = JupiterSwapEvent::new(params, swap_data).with_transfer_data(transfer_data);
        let event2 = event1.clone();

        assert_eq!(event1.metadata.core.id, event2.metadata.core.id);
        assert_eq!(event1.metadata.signature, event2.metadata.signature);
        assert_eq!(event1.metadata.slot, event2.metadata.slot);
        assert_eq!(event1.transfer_data.len(), event2.transfer_data.len());
    }

    // JupiterLiquidityEvent tests
    #[test]
    fn test_jupiter_liquidity_event_default_should_create_empty() {
        let event = JupiterLiquidityEvent::default();

        assert_eq!(event.metadata.core.id, "");
        assert_eq!(event.metadata.signature, "");
        assert_eq!(event.metadata.slot, 0);
        assert_eq!(event.metadata.index, "");
        assert_eq!(event.metadata.program_received_time_ms, 0);
        assert_eq!(event.user, Pubkey::default());
        assert_eq!(event.mint_a, Pubkey::default());
        assert_eq!(event.mint_b, Pubkey::default());
        assert_eq!(event.amount_a, 0);
        assert_eq!(event.amount_b, 0);
        assert_eq!(event.liquidity_amount, 0);
        assert!(!event.is_remove);
        assert_eq!(event.transfer_data.len(), 0);
    }

    #[test]
    fn test_jupiter_liquidity_event_clone_should_create_identical_copy() {
        let params = create_test_event_parameters();
        let event1 = JupiterLiquidityEvent::new(params)
            .with_user(create_test_pubkey())
            .with_removal(true)
            .with_amounts(1000, 2000, 3000);

        let event2 = event1.clone();

        assert_eq!(event1.metadata.core.id, event2.metadata.core.id);
        assert_eq!(event1.user, event2.user);
        assert_eq!(event1.is_remove, event2.is_remove);
        assert_eq!(event1.amount_a, event2.amount_a);
    }

    // JupiterSwapBorshEvent tests
    #[test]
    fn test_jupiter_swap_borsh_event_default_should_create_empty() {
        let event = JupiterSwapBorshEvent::default();

        assert_eq!(event.user, Pubkey::default());
        assert_eq!(event.input_mint, Pubkey::default());
        assert_eq!(event.output_mint, Pubkey::default());
        assert_eq!(event.input_amount, 0);
        assert_eq!(event.output_amount, 0);
        assert_eq!(event.slippage_bps, 0);
        assert_eq!(event.platform_fee_bps, 0);
    }

    #[test]
    fn test_jupiter_swap_borsh_event_clone_should_create_identical_copy() {
        let mut event1 = JupiterSwapBorshEvent::default();
        event1.user = create_test_pubkey();
        event1.input_amount = 1000;
        event1.output_amount = 900;
        event1.slippage_bps = 50;
        event1.platform_fee_bps = 10;

        let event2 = event1.clone();

        assert_eq!(event1.user, event2.user);
        assert_eq!(event1.input_amount, event2.input_amount);
        assert_eq!(event1.output_amount, event2.output_amount);
        assert_eq!(event1.slippage_bps, event2.slippage_bps);
        assert_eq!(event1.platform_fee_bps, event2.platform_fee_bps);
    }

    #[test]
    fn test_jupiter_swap_borsh_event_partial_eq_when_equal_should_return_true() {
        let event1 = JupiterSwapBorshEvent {
            metadata: SolanaEventMetadata::default(),
            user: create_test_pubkey(),
            input_mint: create_test_pubkey(),
            output_mint: create_test_pubkey(),
            input_amount: 1000,
            output_amount: 900,
            slippage_bps: 50,
            platform_fee_bps: 10,
        };
        let event2 = event1.clone();

        assert_eq!(event1, event2);
    }

    #[test]
    fn test_jupiter_swap_borsh_event_partial_eq_when_different_should_return_false() {
        let mut event1 = JupiterSwapBorshEvent::default();
        event1.input_amount = 1000;

        let mut event2 = event1.clone();
        event2.input_amount = 2000;

        assert_ne!(event1, event2);
    }

    // Event trait implementation tests for JupiterSwapEvent
    #[test]
    fn test_jupiter_swap_event_id_should_return_event_id() {
        let params = create_test_event_parameters();
        let swap_data = create_test_jupiter_swap_data();
        let event = JupiterSwapEvent::new(params, swap_data);

        assert_eq!(event.id(), "test-id");
    }

    #[test]
    fn test_jupiter_swap_event_kind_should_return_swap() {
        let params = create_test_event_parameters();
        let swap_data = create_test_jupiter_swap_data();
        let event = JupiterSwapEvent::new(params, swap_data);

        assert_eq!(event.kind(), &EventKind::Swap);
    }

    #[test]
    fn test_jupiter_swap_event_metadata_should_return_default_metadata() {
        let params = create_test_event_parameters();
        let swap_data = create_test_jupiter_swap_data();
        let event = JupiterSwapEvent::new(params, swap_data);

        let metadata = event.metadata();
        assert_eq!(metadata.kind, EventKind::Swap);
        assert!(metadata.source.contains("solana"));
    }

    #[test]
    fn test_jupiter_swap_event_metadata_mut_should_work() {
        let params = create_test_event_parameters();
        let swap_data = create_test_jupiter_swap_data();
        let mut event = JupiterSwapEvent::new(params, swap_data);

        let metadata = event.metadata_mut().unwrap();
        // Verify we can access mutable metadata without panic
        assert_eq!(metadata.kind, EventKind::Swap);
    }

    #[test]
    fn test_jupiter_swap_event_as_any_should_return_self() {
        let params = create_test_event_parameters();
        let swap_data = create_test_jupiter_swap_data();
        let event = JupiterSwapEvent::new(params, swap_data);

        let any = event.as_any();
        assert!(any.downcast_ref::<JupiterSwapEvent>().is_some());
    }

    #[test]
    fn test_jupiter_swap_event_as_any_mut_should_return_mutable_self() {
        let params = create_test_event_parameters();
        let swap_data = create_test_jupiter_swap_data();
        let mut event = JupiterSwapEvent::new(params, swap_data);

        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<JupiterSwapEvent>().is_some());
    }

    #[test]
    fn test_jupiter_swap_event_clone_boxed_should_return_boxed_clone() {
        let params = create_test_event_parameters();
        let swap_data = create_test_jupiter_swap_data();
        let event = JupiterSwapEvent::new(params, swap_data);

        let boxed_clone = event.clone_boxed();
        assert_eq!(boxed_clone.id(), "test-id");
        assert_eq!(boxed_clone.kind(), &EventKind::Swap);
    }

    // Event trait implementation tests for JupiterLiquidityEvent
    #[test]
    fn test_jupiter_liquidity_event_id_should_return_event_id() {
        let params = EventParameters {
            id: "liquidity-test-id".to_string(),
            signature: "test-sig".to_string(),
            slot: 123,
            block_time: 0,
            block_time_ms: 0,
            program_received_time_ms: 0,
            index: "0".to_string(),
        };
        let event = JupiterLiquidityEvent::new(params);

        assert_eq!(event.id(), "liquidity-test-id");
    }

    #[test]
    fn test_jupiter_liquidity_event_kind_should_return_liquidity() {
        let params = create_test_event_parameters();
        let event = JupiterLiquidityEvent::new(params);

        assert_eq!(event.kind(), &EventKind::Liquidity);
    }

    #[test]
    fn test_jupiter_liquidity_event_metadata_should_return_default_metadata() {
        let params = create_test_event_parameters();
        let event = JupiterLiquidityEvent::new(params);

        let metadata = event.metadata();
        assert_eq!(metadata.kind, EventKind::Liquidity);
        assert!(metadata.source.contains("solana"));
    }

    #[test]
    fn test_jupiter_liquidity_event_metadata_mut_should_work() {
        let params = create_test_event_parameters();
        let mut event = JupiterLiquidityEvent::new(params);

        let metadata = event.metadata_mut().unwrap();
        // Verify we can access mutable metadata without panic
        assert_eq!(metadata.kind, EventKind::Liquidity);
    }

    #[test]
    fn test_jupiter_liquidity_event_as_any_should_return_self() {
        let params = create_test_event_parameters();
        let event = JupiterLiquidityEvent::new(params);

        let any = event.as_any();
        assert!(any.downcast_ref::<JupiterLiquidityEvent>().is_some());
    }

    #[test]
    fn test_jupiter_liquidity_event_as_any_mut_should_return_mutable_self() {
        let params = create_test_event_parameters();
        let mut event = JupiterLiquidityEvent::new(params);

        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<JupiterLiquidityEvent>().is_some());
    }

    #[test]
    fn test_jupiter_liquidity_event_clone_boxed_should_return_boxed_clone() {
        let params = EventParameters {
            id: "liquidity-test-id".to_string(),
            signature: "test-sig".to_string(),
            slot: 123,
            block_time: 0,
            block_time_ms: 0,
            program_received_time_ms: 0,
            index: "0".to_string(),
        };
        let event = JupiterLiquidityEvent::new(params);

        let boxed_clone = event.clone_boxed();
        assert_eq!(boxed_clone.id(), "liquidity-test-id");
        assert_eq!(boxed_clone.kind(), &EventKind::Liquidity);
    }

    // Event trait implementation tests for JupiterSwapBorshEvent
    #[test]
    fn test_jupiter_swap_borsh_event_id_should_return_metadata_id() {
        let event = JupiterSwapBorshEvent::default();
        // Default metadata has empty id
        assert_eq!(event.id(), "");
    }

    #[test]
    fn test_jupiter_swap_borsh_event_kind_should_return_swap() {
        let event = JupiterSwapBorshEvent::default();

        assert_eq!(event.kind(), &EventKind::Swap);
    }

    #[test]
    fn test_jupiter_swap_borsh_event_metadata_should_return_default_metadata() {
        let event = JupiterSwapBorshEvent::default();

        let metadata = event.metadata();
        assert_eq!(metadata.kind, EventKind::Swap);
        assert!(metadata.source.contains("solana"));
    }

    #[test]
    fn test_jupiter_swap_borsh_event_metadata_mut_should_work() {
        let mut event = JupiterSwapBorshEvent::default();

        let metadata = event.metadata_mut().unwrap();
        // Verify we can access mutable metadata without panic
        assert_eq!(metadata.kind, EventKind::Swap);
    }

    #[test]
    fn test_jupiter_swap_borsh_event_as_any_should_return_self() {
        let event = JupiterSwapBorshEvent::default();

        let any = event.as_any();
        assert!(any.downcast_ref::<JupiterSwapBorshEvent>().is_some());
    }

    #[test]
    fn test_jupiter_swap_borsh_event_as_any_mut_should_return_mutable_self() {
        let mut event = JupiterSwapBorshEvent::default();

        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<JupiterSwapBorshEvent>().is_some());
    }

    #[test]
    fn test_jupiter_swap_borsh_event_clone_boxed_should_return_boxed_clone() {
        let mut event = JupiterSwapBorshEvent::default();
        event.input_amount = 1000;

        let boxed_clone = event.clone_boxed();
        assert_eq!(boxed_clone.id(), "");
        assert_eq!(boxed_clone.kind(), &EventKind::Swap);
    }

    // Edge case tests
    #[test]
    fn test_event_parameters_with_empty_strings_should_work() {
        let params =
            EventParameters::new("".to_string(), "".to_string(), 0, 0, 0, 0, "".to_string());

        assert_eq!(params.id, "");
        assert_eq!(params.signature, "");
        assert_eq!(params.index, "");
    }

    #[test]
    fn test_event_parameters_with_max_values_should_work() {
        let params = EventParameters::new(
            "very-long-id".repeat(100),
            "very-long-signature".repeat(100),
            u64::MAX,
            i64::MAX,
            i64::MAX,
            i64::MAX,
            "max-index".to_string(),
        );

        assert!(params.id.len() > 1000);
        assert_eq!(params.slot, u64::MAX);
        assert_eq!(params.block_time, i64::MAX);
    }

    #[test]
    fn test_jupiter_swap_event_with_multiple_transfer_data_should_store_all() {
        let params = create_test_event_parameters();
        let swap_data = create_test_jupiter_swap_data();
        let transfer_data = vec![
            create_test_transfer_data(),
            create_test_transfer_data(),
            create_test_transfer_data(),
        ];

        let event = JupiterSwapEvent::new(params, swap_data).with_transfer_data(transfer_data);

        assert_eq!(event.transfer_data.len(), 3);
    }

    #[test]
    fn test_jupiter_swap_borsh_event_with_zero_amounts_should_work() {
        let mut event = JupiterSwapBorshEvent::default();
        event.input_amount = 0;
        event.output_amount = 0;
        event.slippage_bps = 0;
        event.platform_fee_bps = 0;

        assert_eq!(event.input_amount, 0);
        assert_eq!(event.output_amount, 0);
        assert_eq!(event.slippage_bps, 0);
        assert_eq!(event.platform_fee_bps, 0);
    }

    #[test]
    fn test_jupiter_swap_borsh_event_with_max_values_should_work() {
        let mut event = JupiterSwapBorshEvent::default();
        event.input_amount = u64::MAX;
        event.output_amount = u64::MAX;
        event.slippage_bps = u16::MAX;
        event.platform_fee_bps = u8::MAX;

        assert_eq!(event.input_amount, u64::MAX);
        assert_eq!(event.output_amount, u64::MAX);
        assert_eq!(event.slippage_bps, u16::MAX);
        assert_eq!(event.platform_fee_bps, u8::MAX);
    }

    #[test]
    fn test_jupiter_liquidity_event_with_max_amounts_should_work() {
        let mut event = JupiterLiquidityEvent::default();
        event.amount_a = u64::MAX;
        event.amount_b = u64::MAX;
        event.liquidity_amount = u64::MAX;

        assert_eq!(event.amount_a, u64::MAX);
        assert_eq!(event.amount_b, u64::MAX);
        assert_eq!(event.liquidity_amount, u64::MAX);
    }

    #[test]
    fn test_jupiter_liquidity_event_is_remove_toggle_should_work() {
        let mut event = JupiterLiquidityEvent::default();

        assert!(!event.is_remove);

        event.is_remove = true;
        assert!(event.is_remove);

        event.is_remove = false;
        assert!(!event.is_remove);
    }

    // Serialization/Deserialization tests
    #[test]
    fn test_jupiter_swap_event_serialization_should_work() {
        let params = create_test_event_parameters();
        let swap_data = create_test_jupiter_swap_data();
        let event = JupiterSwapEvent::new(params, swap_data);

        let serialized = serde_json::to_string(&event).expect("Failed to serialize");
        let deserialized: JupiterSwapEvent =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(event.metadata.core.id, deserialized.metadata.core.id);
        assert_eq!(event.metadata.signature, deserialized.metadata.signature);
        assert_eq!(event.metadata.slot, deserialized.metadata.slot);
    }

    #[test]
    fn test_jupiter_liquidity_event_serialization_should_work() {
        let params = EventParameters {
            id: "test-liquidity".to_string(),
            signature: "test-sig".to_string(),
            slot: 123,
            block_time: 0,
            block_time_ms: 0,
            program_received_time_ms: 0,
            index: "0".to_string(),
        };
        let event = JupiterLiquidityEvent::new(params)
            .with_amounts(1000, 0, 0)
            .with_removal(true);

        let serialized = serde_json::to_string(&event).expect("Failed to serialize");
        let deserialized: JupiterLiquidityEvent =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(event.metadata.core.id, deserialized.metadata.core.id);
        assert_eq!(event.amount_a, deserialized.amount_a);
        assert_eq!(event.is_remove, deserialized.is_remove);
    }

    #[test]
    fn test_jupiter_swap_borsh_event_serialization_should_work() {
        let mut event = JupiterSwapBorshEvent::default();
        event.input_amount = 1000;
        event.output_amount = 900;

        let serialized = serde_json::to_string(&event).expect("Failed to serialize");
        let deserialized: JupiterSwapBorshEvent =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(event.input_amount, deserialized.input_amount);
        assert_eq!(event.output_amount, deserialized.output_amount);
    }
}
