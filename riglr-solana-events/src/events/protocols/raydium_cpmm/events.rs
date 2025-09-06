use crate::solana_metadata::SolanaEventMetadata;
use borsh::{BorshDeserialize, BorshSerialize};
use riglr_events_core::error::EventResult;
use riglr_events_core::EventMetadata as CoreEventMetadata;
use riglr_events_core::{Event, EventKind};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::any::Any;
// UnifiedEvent trait removed - events now implement Event trait directly

/// Raydium CPMM Swap event
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct RaydiumCpmmSwapEvent {
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: SolanaEventMetadata,
    /// Public key of the CPMM pool state account
    pub pool_state: Pubkey,
    /// Public key of the account that initiated the swap
    pub payer: Pubkey,
    /// Public key of the input token account
    pub input_token_account: Pubkey,
    /// Public key of the output token account
    pub output_token_account: Pubkey,
    /// Public key of the input token vault
    pub input_vault: Pubkey,
    /// Public key of the output token vault
    pub output_vault: Pubkey,
    /// Public key of the input token mint
    pub input_token_mint: Pubkey,
    /// Public key of the output token mint
    pub output_token_mint: Pubkey,
    /// Amount of input tokens swapped
    pub amount_in: u64,
    /// Amount of output tokens received
    pub amount_out: u64,
    /// Trading fee amount
    pub trade_fee: u64,
    /// Transfer fee amount
    pub transfer_fee: u64,
}

// Event trait implementation for RaydiumCpmmSwapEvent
// Event trait implementation for RaydiumCpmmSwapEvent
impl Event for RaydiumCpmmSwapEvent {
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

    fn to_json(&self) -> EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

/// Raydium CPMM Deposit event
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct RaydiumCpmmDepositEvent {
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: SolanaEventMetadata,
    /// Public key of the CPMM pool state account
    pub pool_state: Pubkey,
    /// Public key of the user depositing liquidity
    pub user: Pubkey,
    /// Amount of LP tokens minted
    pub lp_token_amount: u64,
    /// Amount of token 0 deposited
    pub token_0_amount: u64,
    /// Amount of token 1 deposited
    pub token_1_amount: u64,
}

// Event trait implementation for RaydiumCpmmDepositEvent
impl Event for RaydiumCpmmDepositEvent {
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

    fn to_json(&self) -> EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

// Custom Default implementations with correct EventKind
impl Default for RaydiumCpmmSwapEvent {
    fn default() -> Self {
        use chrono::{DateTime, Utc};
        use riglr_events_core::EventMetadata;
        use std::collections::HashMap;

        // Use a fixed timestamp for reproducible tests
        let fixed_timestamp = DateTime::from_timestamp(0, 0).unwrap_or_else(Utc::now);
        let core = EventMetadata {
            id: String::default(),
            kind: EventKind::Swap,
            timestamp: fixed_timestamp,
            received_at: fixed_timestamp, // Use same fixed timestamp for received_at
            source: "solana".to_string(),
            chain_data: None,
            custom: HashMap::new(),
        };

        let metadata = SolanaEventMetadata::new(
            String::default(),                       // signature
            0,                                       // slot
            crate::types::EventType::Swap,           // event_type
            crate::types::ProtocolType::RaydiumCpmm, // protocol_type
            String::default(),                       // index
            0,                                       // program_received_time_ms
            core,
        );

        Self {
            metadata,
            pool_state: Pubkey::default(),
            payer: Pubkey::default(),
            input_token_account: Pubkey::default(),
            output_token_account: Pubkey::default(),
            input_vault: Pubkey::default(),
            output_vault: Pubkey::default(),
            input_token_mint: Pubkey::default(),
            output_token_mint: Pubkey::default(),
            amount_in: 0,
            amount_out: 0,
            trade_fee: 0,
            transfer_fee: 0,
        }
    }
}

impl Default for RaydiumCpmmDepositEvent {
    fn default() -> Self {
        use chrono::{DateTime, Utc};
        use riglr_events_core::EventMetadata;
        use std::collections::HashMap;

        // Use a fixed timestamp for reproducible tests
        let fixed_timestamp = DateTime::from_timestamp(0, 0).unwrap_or_else(Utc::now);
        let core = EventMetadata {
            id: String::default(),
            kind: EventKind::Liquidity,
            timestamp: fixed_timestamp,
            received_at: fixed_timestamp, // Use same fixed timestamp for received_at
            source: "solana".to_string(),
            chain_data: None,
            custom: HashMap::new(),
        };

        let metadata = SolanaEventMetadata::new(
            String::default(),                       // signature
            0,                                       // slot
            crate::types::EventType::AddLiquidity,   // event_type
            crate::types::ProtocolType::RaydiumCpmm, // protocol_type
            String::default(),                       // index
            0,                                       // program_received_time_ms
            core,
        );

        Self {
            metadata,
            pool_state: Pubkey::default(),
            user: Pubkey::default(),
            lp_token_amount: 0,
            token_0_amount: 0,
            token_1_amount: 0,
        }
    }
}

/// Event discriminators module
pub mod discriminators {
    /// String identifier for swap events
    pub const SWAP_EVENT: &str = "raydium_cpmm_swap_event";
    /// String identifier for deposit events
    pub const DEPOSIT_EVENT: &str = "raydium_cpmm_deposit_event";

    /// Byte array discriminator for swap events
    pub const SWAP_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x0a, 0x11, 0xa9, 0xd2, 0xbe, 0x8b, 0x72,
        0xb1,
    ];
    /// Byte array discriminator for deposit events
    pub const DEPOSIT_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x0b, 0x11, 0xa9, 0xd2, 0xbe, 0x8b, 0x72,
        0xb2,
    ];

    /// Instruction discriminator for swap with base input
    pub const SWAP_BASE_INPUT_IX: &[u8] = &[143, 190, 90, 218, 196, 30, 51, 222];
    /// Instruction discriminator for swap with base output
    pub const SWAP_BASE_OUTPUT_IX: &[u8] = &[55, 217, 98, 86, 163, 74, 180, 175];
    /// Instruction discriminator for deposit operations
    pub const DEPOSIT_IX: &[u8] = &[242, 35, 198, 137, 82, 225, 242, 182];
    /// Instruction discriminator for pool initialization
    pub const INITIALIZE_IX: &[u8] = &[175, 175, 109, 31, 13, 152, 155, 237];
    /// Instruction discriminator for withdraw operations
    pub const WITHDRAW_IX: &[u8] = &[183, 18, 70, 156, 148, 109, 161, 34];
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solana_metadata::SolanaEventMetadata;
    use riglr_events_core::{Event, EventKind};

    // Helper function to create a test Pubkey
    fn test_pubkey() -> Pubkey {
        Pubkey::new_unique()
    }

    // Test RaydiumCpmmSwapEvent struct and implementations
    #[test]
    fn test_raydium_cpmm_swap_event_new_default() {
        let event = RaydiumCpmmSwapEvent::default();

        assert_eq!(event.pool_state, Pubkey::default());
        assert_eq!(event.payer, Pubkey::default());
        assert_eq!(event.input_token_account, Pubkey::default());
        assert_eq!(event.output_token_account, Pubkey::default());
        assert_eq!(event.input_vault, Pubkey::default());
        assert_eq!(event.output_vault, Pubkey::default());
        assert_eq!(event.input_token_mint, Pubkey::default());
        assert_eq!(event.output_token_mint, Pubkey::default());
        assert_eq!(event.amount_in, 0);
        assert_eq!(event.amount_out, 0);
        assert_eq!(event.trade_fee, 0);
        assert_eq!(event.transfer_fee, 0);
    }

    #[test]
    fn test_raydium_cpmm_swap_event_with_values() {
        let pool_state = test_pubkey();
        let payer = test_pubkey();
        let input_token_account = test_pubkey();
        let output_token_account = test_pubkey();
        let input_vault = test_pubkey();
        let output_vault = test_pubkey();
        let input_token_mint = test_pubkey();
        let output_token_mint = test_pubkey();

        let event = RaydiumCpmmSwapEvent {
            metadata: SolanaEventMetadata::default(),
            pool_state,
            payer,
            input_token_account,
            output_token_account,
            input_vault,
            output_vault,
            input_token_mint,
            output_token_mint,
            amount_in: 1000,
            amount_out: 950,
            trade_fee: 30,
            transfer_fee: 20,
        };

        assert_eq!(event.pool_state, pool_state);
        assert_eq!(event.payer, payer);
        assert_eq!(event.input_token_account, input_token_account);
        assert_eq!(event.output_token_account, output_token_account);
        assert_eq!(event.input_vault, input_vault);
        assert_eq!(event.output_vault, output_vault);
        assert_eq!(event.input_token_mint, input_token_mint);
        assert_eq!(event.output_token_mint, output_token_mint);
        assert_eq!(event.amount_in, 1000);
        assert_eq!(event.amount_out, 950);
        assert_eq!(event.trade_fee, 30);
        assert_eq!(event.transfer_fee, 20);
    }

    #[test]
    fn test_raydium_cpmm_swap_event_clone() {
        let event = RaydiumCpmmSwapEvent {
            metadata: SolanaEventMetadata::default(),
            pool_state: test_pubkey(),
            payer: test_pubkey(),
            input_token_account: test_pubkey(),
            output_token_account: test_pubkey(),
            input_vault: test_pubkey(),
            output_vault: test_pubkey(),
            input_token_mint: test_pubkey(),
            output_token_mint: test_pubkey(),
            amount_in: 1000,
            amount_out: 950,
            trade_fee: 30,
            transfer_fee: 20,
        };

        let cloned_event = event.clone();
        assert_eq!(event, cloned_event);
    }

    #[test]
    fn test_raydium_cpmm_swap_event_debug() {
        let event = RaydiumCpmmSwapEvent::default();
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("RaydiumCpmmSwapEvent"));
    }

    #[test]
    fn test_raydium_cpmm_swap_event_partial_eq() {
        let event1 = RaydiumCpmmSwapEvent::default();
        let event2 = RaydiumCpmmSwapEvent::default();
        let mut event3 = RaydiumCpmmSwapEvent::default();
        event3.amount_in = 100;

        assert_eq!(event1, event2);
        assert_ne!(event1, event3);
    }

    // Test Event trait implementation for RaydiumCpmmSwapEvent
    #[test]
    fn test_raydium_cpmm_swap_event_id() {
        let event = RaydiumCpmmSwapEvent::default();
        assert_eq!(event.id(), "");
    }

    #[test]
    fn test_raydium_cpmm_swap_event_kind() {
        let event = RaydiumCpmmSwapEvent::default();
        assert_eq!(event.kind(), &EventKind::Swap);
    }

    #[test]
    fn test_raydium_cpmm_swap_event_metadata() {
        let event = RaydiumCpmmSwapEvent::default();
        let metadata = event.metadata();

        assert_eq!(metadata.id, "");
        assert_eq!(metadata.kind, EventKind::Swap);
        assert_eq!(metadata.source, "solana");
        assert!(metadata.chain_data.is_none());
        assert!(metadata.custom.is_empty());
    }

    #[test]
    fn test_raydium_cpmm_swap_event_metadata_mut_should_work() {
        let mut event = RaydiumCpmmSwapEvent::default();
        let metadata = event.metadata_mut().unwrap();
        metadata.id = "test-cpmm-swap-event-id".to_string();
        assert_eq!(event.metadata().id, "test-cpmm-swap-event-id");
    }

    #[test]
    fn test_raydium_cpmm_swap_event_as_any() {
        let event = RaydiumCpmmSwapEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<RaydiumCpmmSwapEvent>().is_some());
    }

    #[test]
    fn test_raydium_cpmm_swap_event_as_any_mut() {
        let mut event = RaydiumCpmmSwapEvent::default();
        let any_mut_ref = event.as_any_mut();
        assert!(any_mut_ref.downcast_mut::<RaydiumCpmmSwapEvent>().is_some());
    }

    #[test]
    fn test_raydium_cpmm_swap_event_clone_boxed() {
        let event = RaydiumCpmmSwapEvent::default();
        let boxed_clone = event.clone_boxed();

        assert!(boxed_clone
            .as_any()
            .downcast_ref::<RaydiumCpmmSwapEvent>()
            .is_some());
        assert_eq!(boxed_clone.kind(), &EventKind::Swap);
    }

    // Test RaydiumCpmmDepositEvent struct and implementations
    #[test]
    fn test_raydium_cpmm_deposit_event_new_default() {
        let event = RaydiumCpmmDepositEvent::default();

        assert_eq!(event.pool_state, Pubkey::default());
        assert_eq!(event.user, Pubkey::default());
        assert_eq!(event.lp_token_amount, 0);
        assert_eq!(event.token_0_amount, 0);
        assert_eq!(event.token_1_amount, 0);
    }

    #[test]
    fn test_raydium_cpmm_deposit_event_with_values() {
        let pool_state = test_pubkey();
        let user = test_pubkey();

        let event = RaydiumCpmmDepositEvent {
            metadata: SolanaEventMetadata::default(),
            pool_state,
            user,
            lp_token_amount: 500,
            token_0_amount: 1000,
            token_1_amount: 2000,
        };

        assert_eq!(event.pool_state, pool_state);
        assert_eq!(event.user, user);
        assert_eq!(event.lp_token_amount, 500);
        assert_eq!(event.token_0_amount, 1000);
        assert_eq!(event.token_1_amount, 2000);
    }

    #[test]
    fn test_raydium_cpmm_deposit_event_clone() {
        let event = RaydiumCpmmDepositEvent {
            metadata: SolanaEventMetadata::default(),
            pool_state: test_pubkey(),
            user: test_pubkey(),
            lp_token_amount: 500,
            token_0_amount: 1000,
            token_1_amount: 2000,
        };

        let cloned_event = event.clone();
        assert_eq!(event, cloned_event);
    }

    #[test]
    fn test_raydium_cpmm_deposit_event_debug() {
        let event = RaydiumCpmmDepositEvent::default();
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("RaydiumCpmmDepositEvent"));
    }

    #[test]
    fn test_raydium_cpmm_deposit_event_partial_eq() {
        let event1 = RaydiumCpmmDepositEvent::default();
        let event2 = RaydiumCpmmDepositEvent::default();
        let mut event3 = RaydiumCpmmDepositEvent::default();
        event3.lp_token_amount = 100;

        assert_eq!(event1, event2);
        assert_ne!(event1, event3);
    }

    // Test Event trait implementation for RaydiumCpmmDepositEvent
    #[test]
    fn test_raydium_cpmm_deposit_event_id() {
        let event = RaydiumCpmmDepositEvent::default();
        assert_eq!(event.id(), "");
    }

    #[test]
    fn test_raydium_cpmm_deposit_event_kind() {
        let event = RaydiumCpmmDepositEvent::default();
        assert_eq!(event.kind(), &EventKind::Liquidity);
    }

    #[test]
    fn test_raydium_cpmm_deposit_event_metadata() {
        let event = RaydiumCpmmDepositEvent::default();
        let metadata = event.metadata();

        assert_eq!(metadata.id, "");
        assert_eq!(metadata.kind, EventKind::Liquidity);
        assert_eq!(metadata.source, "solana");
        assert!(metadata.chain_data.is_none());
        assert!(metadata.custom.is_empty());
    }

    #[test]
    fn test_raydium_cpmm_deposit_event_metadata_mut_should_work() {
        let mut event = RaydiumCpmmDepositEvent::default();
        let metadata = event.metadata_mut().unwrap();
        metadata.id = "test-cpmm-deposit-event-id".to_string();
        assert_eq!(event.metadata().id, "test-cpmm-deposit-event-id");
    }

    #[test]
    fn test_raydium_cpmm_deposit_event_as_any() {
        let event = RaydiumCpmmDepositEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<RaydiumCpmmDepositEvent>().is_some());
    }

    #[test]
    fn test_raydium_cpmm_deposit_event_as_any_mut() {
        let mut event = RaydiumCpmmDepositEvent::default();
        let any_mut_ref = event.as_any_mut();
        assert!(any_mut_ref
            .downcast_mut::<RaydiumCpmmDepositEvent>()
            .is_some());
    }

    #[test]
    fn test_raydium_cpmm_deposit_event_clone_boxed() {
        let event = RaydiumCpmmDepositEvent::default();
        let boxed_clone = event.clone_boxed();

        assert!(boxed_clone
            .as_any()
            .downcast_ref::<RaydiumCpmmDepositEvent>()
            .is_some());
        assert_eq!(boxed_clone.kind(), &EventKind::Liquidity);
    }

    // Test discriminators module constants
    #[test]
    fn test_discriminator_swap_event_string() {
        assert_eq!(discriminators::SWAP_EVENT, "raydium_cpmm_swap_event");
    }

    #[test]
    fn test_discriminator_deposit_event_string() {
        assert_eq!(discriminators::DEPOSIT_EVENT, "raydium_cpmm_deposit_event");
    }

    #[test]
    fn test_discriminator_swap_event_bytes() {
        let expected = &[
            0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x0a, 0x11, 0xa9, 0xd2, 0xbe, 0x8b,
            0x72, 0xb1,
        ];
        assert_eq!(discriminators::SWAP_EVENT_BYTES, expected);
        assert_eq!(discriminators::SWAP_EVENT_BYTES.len(), 16);
    }

    #[test]
    fn test_discriminator_deposit_event_bytes() {
        let expected = &[
            0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x0b, 0x11, 0xa9, 0xd2, 0xbe, 0x8b,
            0x72, 0xb2,
        ];
        assert_eq!(discriminators::DEPOSIT_EVENT_BYTES, expected);
        assert_eq!(discriminators::DEPOSIT_EVENT_BYTES.len(), 16);
    }

    #[test]
    fn test_discriminator_swap_base_input_ix() {
        let expected = &[143, 190, 90, 218, 196, 30, 51, 222];
        assert_eq!(discriminators::SWAP_BASE_INPUT_IX, expected);
        assert_eq!(discriminators::SWAP_BASE_INPUT_IX.len(), 8);
    }

    #[test]
    fn test_discriminator_swap_base_output_ix() {
        let expected = &[55, 217, 98, 86, 163, 74, 180, 175];
        assert_eq!(discriminators::SWAP_BASE_OUTPUT_IX, expected);
        assert_eq!(discriminators::SWAP_BASE_OUTPUT_IX.len(), 8);
    }

    #[test]
    fn test_discriminator_deposit_ix() {
        let expected = &[242, 35, 198, 137, 82, 225, 242, 182];
        assert_eq!(discriminators::DEPOSIT_IX, expected);
        assert_eq!(discriminators::DEPOSIT_IX.len(), 8);
    }

    #[test]
    fn test_discriminator_initialize_ix() {
        let expected = &[175, 175, 109, 31, 13, 152, 155, 237];
        assert_eq!(discriminators::INITIALIZE_IX, expected);
        assert_eq!(discriminators::INITIALIZE_IX.len(), 8);
    }

    #[test]
    fn test_discriminator_withdraw_ix() {
        let expected = &[183, 18, 70, 156, 148, 109, 161, 34];
        assert_eq!(discriminators::WITHDRAW_IX, expected);
        assert_eq!(discriminators::WITHDRAW_IX.len(), 8);
    }

    // Edge case tests with extreme values
    #[test]
    fn test_raydium_cpmm_swap_event_with_max_values() {
        let event = RaydiumCpmmSwapEvent {
            metadata: SolanaEventMetadata::default(),
            pool_state: test_pubkey(),
            payer: test_pubkey(),
            input_token_account: test_pubkey(),
            output_token_account: test_pubkey(),
            input_vault: test_pubkey(),
            output_vault: test_pubkey(),
            input_token_mint: test_pubkey(),
            output_token_mint: test_pubkey(),
            amount_in: u64::MAX,
            amount_out: u64::MAX,
            trade_fee: u64::MAX,
            transfer_fee: u64::MAX,
        };

        assert_eq!(event.amount_in, u64::MAX);
        assert_eq!(event.amount_out, u64::MAX);
        assert_eq!(event.trade_fee, u64::MAX);
        assert_eq!(event.transfer_fee, u64::MAX);
    }

    #[test]
    fn test_raydium_cpmm_deposit_event_with_max_values() {
        let event = RaydiumCpmmDepositEvent {
            metadata: SolanaEventMetadata::default(),
            pool_state: test_pubkey(),
            user: test_pubkey(),
            lp_token_amount: u64::MAX,
            token_0_amount: u64::MAX,
            token_1_amount: u64::MAX,
        };

        assert_eq!(event.lp_token_amount, u64::MAX);
        assert_eq!(event.token_0_amount, u64::MAX);
        assert_eq!(event.token_1_amount, u64::MAX);
    }

    // Test serialization/deserialization
    #[test]
    fn test_raydium_cpmm_swap_event_serialization() {
        let event = RaydiumCpmmSwapEvent {
            metadata: SolanaEventMetadata::default(),
            pool_state: test_pubkey(),
            payer: test_pubkey(),
            input_token_account: test_pubkey(),
            output_token_account: test_pubkey(),
            input_vault: test_pubkey(),
            output_vault: test_pubkey(),
            input_token_mint: test_pubkey(),
            output_token_mint: test_pubkey(),
            amount_in: 1000,
            amount_out: 950,
            trade_fee: 30,
            transfer_fee: 20,
        };

        // Test JSON serialization
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: RaydiumCpmmSwapEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event.pool_state, deserialized.pool_state);
        assert_eq!(event.amount_in, deserialized.amount_in);
        assert_eq!(event.amount_out, deserialized.amount_out);
    }

    #[test]
    fn test_raydium_cpmm_deposit_event_serialization() {
        let event = RaydiumCpmmDepositEvent {
            metadata: SolanaEventMetadata::default(),
            pool_state: test_pubkey(),
            user: test_pubkey(),
            lp_token_amount: 500,
            token_0_amount: 1000,
            token_1_amount: 2000,
        };

        // Test JSON serialization
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: RaydiumCpmmDepositEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event.pool_state, deserialized.pool_state);
        assert_eq!(event.lp_token_amount, deserialized.lp_token_amount);
        assert_eq!(event.token_0_amount, deserialized.token_0_amount);
        assert_eq!(event.token_1_amount, deserialized.token_1_amount);
    }

    // Test that discriminator bytes are different between events
    #[test]
    fn test_discriminator_bytes_are_different() {
        assert_ne!(
            discriminators::SWAP_EVENT_BYTES,
            discriminators::DEPOSIT_EVENT_BYTES
        );
    }

    // Test that instruction discriminators are all different
    #[test]
    fn test_instruction_discriminators_are_unique() {
        let discriminators = vec![
            discriminators::SWAP_BASE_INPUT_IX,
            discriminators::SWAP_BASE_OUTPUT_IX,
            discriminators::DEPOSIT_IX,
            discriminators::INITIALIZE_IX,
            discriminators::WITHDRAW_IX,
        ];

        for (i, disc1) in discriminators.iter().enumerate() {
            for (j, disc2) in discriminators.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        disc1, disc2,
                        "Discriminators at positions {} and {} are the same",
                        i, j
                    );
                }
            }
        }
    }
}
#[cfg(test)]
mod cpmm_metadata_fix_verification {
    use super::*;
    use riglr_events_core::Event;

    #[test]
    fn test_cpmm_metadata_isolation() {
        let mut event1 = RaydiumCpmmSwapEvent::default();
        let mut event2 = RaydiumCpmmSwapEvent::default();

        event1.metadata.id = "cpmm1".to_string();
        event2.metadata.id = "cpmm2".to_string();

        assert_eq!(event1.id(), "cpmm1");
        assert_eq!(event2.id(), "cpmm2");

        event1.metadata.id = "modified".to_string();
        assert_eq!(event1.id(), "modified");
        assert_eq!(event2.id(), "cpmm2");
    }

    #[test]
    fn test_cpmm_metadata_mut_works() {
        let mut event = RaydiumCpmmSwapEvent::default();
        let metadata_mut = event.metadata_mut().unwrap();
        metadata_mut.id = "test_cpmm".to_string();
        assert_eq!(event.id(), "test_cpmm");
    }
}
