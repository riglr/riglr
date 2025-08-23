// Events now implement Event trait directly
#[cfg(test)]
use crate::events::protocols::bonk::types::{ConstantCurve, FixedCurve, LinearCurve};
use crate::events::protocols::bonk::types::{
    CurveParams, MintParams, PoolStatus, TradeDirection, VestingParams,
};
use crate::solana_metadata::SolanaEventMetadata;
use crate::types::EventMetadata;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

// Import Event trait from riglr-events-core
use riglr_events_core::{Event, EventKind};
use std::any::Any;

/// Trade event
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Default,
)]
pub struct BonkTradeEvent {
    /// Event metadata
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: EventMetadata,
    /// Public key of the pool state account
    pub pool_state: Pubkey,
    /// Total amount of base tokens available for sale
    pub total_base_sell: u64,
    /// Virtual base token reserves used for price calculations
    pub virtual_base: u64,
    /// Virtual quote token reserves used for price calculations
    pub virtual_quote: u64,
    /// Real base token reserves before the trade
    pub real_base_before: u64,
    /// Real quote token reserves before the trade
    pub real_quote_before: u64,
    /// Real base token reserves after the trade
    pub real_base_after: u64,
    /// Real quote token reserves after the trade
    pub real_quote_after: u64,
    /// Amount of tokens being traded in
    pub amount_in: u64,
    /// Amount of tokens being traded out
    pub amount_out: u64,
    /// Fee charged by the protocol
    pub protocol_fee: u64,
    /// Fee charged by the platform
    pub platform_fee: u64,
    /// Fee shared with stakeholders
    pub share_fee: u64,
    /// Direction of the trade (buy or sell)
    pub trade_direction: TradeDirection,
    /// Current status of the pool
    pub pool_status: PoolStatus,
    /// Minimum amount expected to receive from the trade
    #[serde(skip)]
    pub minimum_amount_out: u64,
    /// Maximum amount willing to spend on the trade
    #[serde(skip)]
    pub maximum_amount_in: u64,
    /// Rate of the share fee as a percentage
    #[serde(skip)]
    pub share_fee_rate: u64,
    /// Public key of the account paying for the transaction
    #[serde(skip)]
    pub payer: Pubkey,
    /// Public key of the user's base token account
    #[serde(skip)]
    pub user_base_token: Pubkey,
    /// Public key of the user's quote token account
    #[serde(skip)]
    pub user_quote_token: Pubkey,
    /// Public key of the pool's base token vault
    #[serde(skip)]
    pub base_vault: Pubkey,
    /// Public key of the pool's quote token vault
    #[serde(skip)]
    pub quote_vault: Pubkey,
    /// Public key of the base token mint
    #[serde(skip)]
    pub base_token_mint: Pubkey,
    /// Public key of the quote token mint
    #[serde(skip)]
    pub quote_token_mint: Pubkey,
    /// Whether this is a developer token creation trade
    #[serde(skip)]
    pub is_dev_create_token_trade: bool,
    /// Whether the trade was executed by a bot
    #[serde(skip)]
    pub is_bot: bool,
}

// Macro to generate UnifiedEvent implementation

// New Event trait implementation
impl Event for BonkTradeEvent {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        &self.metadata.core.kind
    }

    fn metadata(&self) -> &riglr_events_core::EventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut riglr_events_core::EventMetadata {
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
        serde_json::to_value(self).map_err(riglr_events_core::error::EventError::Serialization)
    }
}

/// Create pool event
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Default,
)]
pub struct BonkPoolCreateEvent {
    /// Event metadata
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: EventMetadata,
    /// Public key of the pool state account
    pub pool_state: Pubkey,
    /// Public key of the pool creator
    pub creator: Pubkey,
    /// Public key of the pool configuration
    pub config: Pubkey,
    /// Parameters for the base token mint
    pub base_mint_param: MintParams,
    /// Parameters for the bonding curve
    pub curve_param: CurveParams,
    /// Parameters for token vesting
    pub vesting_param: VestingParams,
    /// Public key of the account paying for the transaction
    #[serde(skip)]
    pub payer: Pubkey,
    /// Public key of the base token mint
    #[serde(skip)]
    pub base_mint: Pubkey,
    /// Public key of the quote token mint
    #[serde(skip)]
    pub quote_mint: Pubkey,
    /// Public key of the base token vault
    #[serde(skip)]
    pub base_vault: Pubkey,
    /// Public key of the quote token vault
    #[serde(skip)]
    pub quote_vault: Pubkey,
    /// Public key of the global configuration account
    #[serde(skip)]
    pub global_config: Pubkey,
    /// Public key of the platform configuration account
    #[serde(skip)]
    pub platform_config: Pubkey,
}

// New Event trait implementation
impl Event for BonkPoolCreateEvent {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        &self.metadata.core.kind
    }

    fn metadata(&self) -> &riglr_events_core::EventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut riglr_events_core::EventMetadata {
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
        serde_json::to_value(self).map_err(riglr_events_core::error::EventError::Serialization)
    }
}

/// Event discriminator constants
pub mod discriminators {
    /// String identifier for Bonk trade events
    pub const TRADE_EVENT: &str = "bonk_trade_event";
    /// String identifier for Bonk pool creation events
    pub const POOL_CREATE_EVENT: &str = "bonk_pool_create_event";

    /// Byte array discriminator for trade events, used for efficient on-chain parsing
    pub const TRADE_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x0e, 0x11, 0xa9, 0xd2, 0xbe, 0x8b, 0x72,
        0xb4,
    ];
    /// Byte array discriminator for pool creation events, used for efficient on-chain parsing
    pub const POOL_CREATE_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x1a, 0x11, 0xa9, 0xd2, 0xbe, 0x8b, 0x72,
        0x00,
    ];

    /// Instruction discriminator for buy exact in operations
    pub const BUY_EXACT_IN_IX: &[u8] = &[64, 198, 72, 0, 130, 84, 226, 13];
    /// Instruction discriminator for buy exact out operations
    pub const BUY_EXACT_OUT_IX: &[u8] = &[121, 45, 19, 239, 92, 248, 10, 80];
    /// Instruction discriminator for sell exact in operations
    pub const SELL_EXACT_IN_IX: &[u8] = &[155, 233, 77, 24, 217, 193, 201, 60];
    /// Instruction discriminator for sell exact out operations
    pub const SELL_EXACT_OUT_IX: &[u8] = &[17, 199, 138, 195, 225, 36, 94, 131];
    /// Instruction discriminator for pool initialization
    pub const INITIALIZE_IX: &[u8] = &[175, 175, 109, 31, 13, 152, 155, 237];
    /// Instruction discriminator for migrating to AMM
    pub const MIGRATE_TO_AMM_IX: &[u8] = &[139, 100, 87, 4, 218, 242, 121, 178];
    /// Instruction discriminator for migrating to constant product swap
    pub const MIGRATE_TO_CPSWAP_IX: &[u8] = &[211, 229, 52, 34, 83, 117, 96, 198];
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event_metadata() -> EventMetadata {
        use crate::types::{EventType, ProtocolType};

        let core = riglr_events_core::EventMetadata::new(
            "test-id".to_string(),
            EventKind::Swap,
            "test-source".to_string(),
        );

        SolanaEventMetadata::new(
            "test-signature".to_string(),
            12345,
            EventType::Swap,
            ProtocolType::Bonk,
            "0".to_string(),
            1640995200000,
            core,
        )
    }

    fn create_test_pubkey() -> Pubkey {
        Pubkey::new_unique()
    }

    fn create_test_mint_params() -> MintParams {
        MintParams {
            decimals: 9,
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            uri: "https://example.com/token.json".to_string(),
        }
    }

    fn create_test_curve_params() -> CurveParams {
        CurveParams {
            constant_curve: Some(ConstantCurve {
                supply: 1000000,
                total_base_sell: 500000,
                total_quote_fund_raising: 10000,
                migrate_type: 1,
            }),
            fixed_curve: None,
            linear_curve: None,
        }
    }

    fn create_test_vesting_params() -> VestingParams {
        VestingParams {
            total_locked_amount: 100000,
            cliff_period: 86400,
            unlock_period: 2592000,
        }
    }

    #[test]
    fn test_bonk_trade_event_default() {
        let event = BonkTradeEvent::default();
        assert_eq!(event.pool_state, Pubkey::default());
        assert_eq!(event.total_base_sell, 0);
        assert_eq!(event.virtual_base, 0);
        assert_eq!(event.virtual_quote, 0);
        assert_eq!(event.real_base_before, 0);
        assert_eq!(event.real_quote_before, 0);
        assert_eq!(event.real_base_after, 0);
        assert_eq!(event.real_quote_after, 0);
        assert_eq!(event.amount_in, 0);
        assert_eq!(event.amount_out, 0);
        assert_eq!(event.protocol_fee, 0);
        assert_eq!(event.platform_fee, 0);
        assert_eq!(event.share_fee, 0);
        assert_eq!(event.trade_direction, TradeDirection::default());
        assert_eq!(event.pool_status, PoolStatus::default());
        assert_eq!(event.minimum_amount_out, 0);
        assert_eq!(event.maximum_amount_in, 0);
        assert_eq!(event.share_fee_rate, 0);
        assert_eq!(event.payer, Pubkey::default());
        assert_eq!(event.user_base_token, Pubkey::default());
        assert_eq!(event.user_quote_token, Pubkey::default());
        assert_eq!(event.base_vault, Pubkey::default());
        assert_eq!(event.quote_vault, Pubkey::default());
        assert_eq!(event.base_token_mint, Pubkey::default());
        assert_eq!(event.quote_token_mint, Pubkey::default());
        assert!(!event.is_dev_create_token_trade);
        assert!(!event.is_bot);
        assert_eq!(event.metadata, EventMetadata::default());
    }

    #[test]
    fn test_bonk_trade_event_clone() {
        let mut event = BonkTradeEvent::default();
        event.metadata = create_test_event_metadata();
        event.pool_state = create_test_pubkey();
        event.total_base_sell = 1000;
        event.trade_direction = TradeDirection::Sell;
        event.pool_status = PoolStatus::Trade;
        event.is_bot = true;

        let cloned = event.clone();
        assert_eq!(event, cloned);
        assert_eq!(event.metadata.id, cloned.metadata.id);
        assert_eq!(event.pool_state, cloned.pool_state);
        assert_eq!(event.total_base_sell, cloned.total_base_sell);
        assert_eq!(event.trade_direction, cloned.trade_direction);
        assert_eq!(event.pool_status, cloned.pool_status);
        assert_eq!(event.is_bot, cloned.is_bot);
    }

    #[test]
    fn test_bonk_trade_event_serialization() {
        let mut event = BonkTradeEvent::default();
        event.pool_state = create_test_pubkey();
        event.total_base_sell = 1000;
        event.virtual_base = 500;
        event.virtual_quote = 2000;
        event.trade_direction = TradeDirection::Buy;
        event.pool_status = PoolStatus::Migrate;

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: BonkTradeEvent = serde_json::from_str(&serialized).unwrap();

        // Note: metadata field is skipped in serialization, so we only compare other fields
        assert_eq!(event.pool_state, deserialized.pool_state);
        assert_eq!(event.total_base_sell, deserialized.total_base_sell);
        assert_eq!(event.virtual_base, deserialized.virtual_base);
        assert_eq!(event.virtual_quote, deserialized.virtual_quote);
        assert_eq!(event.trade_direction, deserialized.trade_direction);
        assert_eq!(event.pool_status, deserialized.pool_status);
    }

    #[test]
    fn test_bonk_trade_event_event_trait_id() {
        let mut event = BonkTradeEvent::default();
        event.metadata = create_test_event_metadata();

        assert_eq!(event.id(), "test-id");
    }

    #[test]
    fn test_bonk_trade_event_event_trait_kind() {
        let mut event = BonkTradeEvent::default();
        event.metadata = create_test_event_metadata();

        assert_eq!(event.kind(), &EventKind::Swap);
    }

    #[test]
    fn test_bonk_trade_event_event_trait_metadata() {
        let mut event = BonkTradeEvent::default();
        let metadata = create_test_event_metadata();
        event.metadata = metadata;

        assert_eq!(event.metadata(), &create_test_event_metadata().core);
    }

    #[test]
    fn test_bonk_trade_event_event_trait_metadata_mut() {
        let mut event = BonkTradeEvent::default();
        event.metadata = create_test_event_metadata();

        let metadata_mut = event.metadata_mut();
        metadata_mut.source = "updated-source".to_string();

        assert_eq!(event.metadata.source, "updated-source");
    }

    #[test]
    fn test_bonk_trade_event_event_trait_as_any() {
        let mut event = BonkTradeEvent::default();
        event.metadata = create_test_event_metadata();

        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<BonkTradeEvent>().is_some());
    }

    #[test]
    fn test_bonk_trade_event_event_trait_as_any_mut() {
        let mut event = BonkTradeEvent::default();
        event.metadata = create_test_event_metadata();

        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<BonkTradeEvent>().is_some());
    }

    #[test]
    fn test_bonk_trade_event_event_trait_clone_boxed() {
        let mut event = BonkTradeEvent::default();
        event.metadata = create_test_event_metadata();
        event.total_base_sell = 1500;

        let boxed_clone = event.clone_boxed();
        assert_eq!(boxed_clone.id(), "test-id");

        // Downcast to verify it's the same type and data
        let downcasted = boxed_clone
            .as_any()
            .downcast_ref::<BonkTradeEvent>()
            .unwrap();
        assert_eq!(downcasted.total_base_sell, 1500);
    }

    #[test]
    fn test_bonk_trade_event_event_trait_to_json_success() {
        let mut event = BonkTradeEvent::default();
        event.metadata = create_test_event_metadata();
        event.pool_state = create_test_pubkey();
        event.total_base_sell = 2000;

        let json_result = event.to_json();
        assert!(json_result.is_ok());

        let json_value = json_result.unwrap();
        assert_eq!(json_value["total_base_sell"], 2000);
        assert!(json_value["pool_state"].is_string());
    }

    #[test]
    fn test_bonk_pool_create_event_default() {
        let event = BonkPoolCreateEvent::default();
        assert_eq!(event.pool_state, Pubkey::default());
        assert_eq!(event.creator, Pubkey::default());
        assert_eq!(event.config, Pubkey::default());
        assert_eq!(event.base_mint_param, MintParams::default());
        assert_eq!(event.curve_param, CurveParams::default());
        assert_eq!(event.vesting_param, VestingParams::default());
        assert_eq!(event.payer, Pubkey::default());
        assert_eq!(event.base_mint, Pubkey::default());
        assert_eq!(event.quote_mint, Pubkey::default());
        assert_eq!(event.base_vault, Pubkey::default());
        assert_eq!(event.quote_vault, Pubkey::default());
        assert_eq!(event.global_config, Pubkey::default());
        assert_eq!(event.platform_config, Pubkey::default());
        assert_eq!(event.metadata, EventMetadata::default());
    }

    #[test]
    fn test_bonk_pool_create_event_clone() {
        let mut event = BonkPoolCreateEvent::default();
        event.metadata = create_test_event_metadata();
        event.pool_state = create_test_pubkey();
        event.creator = create_test_pubkey();
        event.base_mint_param = create_test_mint_params();
        event.curve_param = create_test_curve_params();
        event.vesting_param = create_test_vesting_params();

        let cloned = event.clone();
        assert_eq!(event, cloned);
        assert_eq!(event.metadata.id, cloned.metadata.id);
        assert_eq!(event.pool_state, cloned.pool_state);
        assert_eq!(event.creator, cloned.creator);
        assert_eq!(event.base_mint_param, cloned.base_mint_param);
        assert_eq!(event.curve_param, cloned.curve_param);
        assert_eq!(event.vesting_param, cloned.vesting_param);
    }

    #[test]
    fn test_bonk_pool_create_event_serialization() {
        let mut event = BonkPoolCreateEvent::default();
        event.pool_state = create_test_pubkey();
        event.creator = create_test_pubkey();
        event.config = create_test_pubkey();
        event.base_mint_param = create_test_mint_params();
        event.curve_param = create_test_curve_params();
        event.vesting_param = create_test_vesting_params();

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: BonkPoolCreateEvent = serde_json::from_str(&serialized).unwrap();

        // Note: metadata field is skipped in serialization, so we only compare other fields
        assert_eq!(event.pool_state, deserialized.pool_state);
        assert_eq!(event.creator, deserialized.creator);
        assert_eq!(event.config, deserialized.config);
        assert_eq!(event.base_mint_param, deserialized.base_mint_param);
        assert_eq!(event.curve_param, deserialized.curve_param);
        assert_eq!(event.vesting_param, deserialized.vesting_param);
    }

    #[test]
    fn test_bonk_pool_create_event_event_trait_id() {
        let mut event = BonkPoolCreateEvent::default();
        event.metadata = create_test_event_metadata();

        assert_eq!(event.id(), "test-id");
    }

    #[test]
    fn test_bonk_pool_create_event_event_trait_kind() {
        let mut event = BonkPoolCreateEvent::default();
        event.metadata = create_test_event_metadata();

        assert_eq!(event.kind(), &EventKind::Swap);
    }

    #[test]
    fn test_bonk_pool_create_event_event_trait_metadata() {
        let mut event = BonkPoolCreateEvent::default();
        let metadata = create_test_event_metadata();
        event.metadata = metadata;

        assert_eq!(event.metadata(), &create_test_event_metadata().core);
    }

    #[test]
    fn test_bonk_pool_create_event_event_trait_metadata_mut() {
        let mut event = BonkPoolCreateEvent::default();
        event.metadata = create_test_event_metadata();

        let metadata_mut = event.metadata_mut();
        metadata_mut.source = "pool-source".to_string();

        assert_eq!(event.metadata.source, "pool-source");
    }

    #[test]
    fn test_bonk_pool_create_event_event_trait_as_any() {
        let mut event = BonkPoolCreateEvent::default();
        event.metadata = create_test_event_metadata();

        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<BonkPoolCreateEvent>().is_some());
    }

    #[test]
    fn test_bonk_pool_create_event_event_trait_as_any_mut() {
        let mut event = BonkPoolCreateEvent::default();
        event.metadata = create_test_event_metadata();

        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<BonkPoolCreateEvent>().is_some());
    }

    #[test]
    fn test_bonk_pool_create_event_event_trait_clone_boxed() {
        let mut event = BonkPoolCreateEvent::default();
        event.metadata = create_test_event_metadata();
        event.creator = create_test_pubkey();

        let boxed_clone = event.clone_boxed();
        assert_eq!(boxed_clone.id(), "test-id");

        // Downcast to verify it's the same type and data
        let downcasted = boxed_clone
            .as_any()
            .downcast_ref::<BonkPoolCreateEvent>()
            .unwrap();
        assert_eq!(downcasted.creator, event.creator);
    }

    #[test]
    fn test_bonk_pool_create_event_event_trait_to_json_success() {
        let mut event = BonkPoolCreateEvent::default();
        event.metadata = create_test_event_metadata();
        event.pool_state = create_test_pubkey();
        event.creator = create_test_pubkey();
        event.base_mint_param = create_test_mint_params();

        let json_result = event.to_json();
        assert!(json_result.is_ok());

        let json_value = json_result.unwrap();
        assert!(json_value["pool_state"].is_string());
        assert!(json_value["creator"].is_string());
        assert_eq!(json_value["base_mint_param"]["name"], "Test Token");
        assert_eq!(json_value["base_mint_param"]["symbol"], "TEST");
        assert_eq!(json_value["base_mint_param"]["decimals"], 9);
    }

    #[test]
    fn test_bonk_trade_event_with_all_trade_directions() {
        let mut buy_event = BonkTradeEvent::default();
        buy_event.trade_direction = TradeDirection::Buy;
        assert_eq!(buy_event.trade_direction, TradeDirection::Buy);

        let mut sell_event = BonkTradeEvent::default();
        sell_event.trade_direction = TradeDirection::Sell;
        assert_eq!(sell_event.trade_direction, TradeDirection::Sell);
    }

    #[test]
    fn test_bonk_trade_event_with_all_pool_statuses() {
        let mut fund_event = BonkTradeEvent::default();
        fund_event.pool_status = PoolStatus::Fund;
        assert_eq!(fund_event.pool_status, PoolStatus::Fund);

        let mut migrate_event = BonkTradeEvent::default();
        migrate_event.pool_status = PoolStatus::Migrate;
        assert_eq!(migrate_event.pool_status, PoolStatus::Migrate);

        let mut trade_event = BonkTradeEvent::default();
        trade_event.pool_status = PoolStatus::Trade;
        assert_eq!(trade_event.pool_status, PoolStatus::Trade);
    }

    #[test]
    fn test_bonk_trade_event_with_maximum_values() {
        let mut event = BonkTradeEvent::default();
        event.total_base_sell = u64::MAX;
        event.virtual_base = u64::MAX;
        event.virtual_quote = u64::MAX;
        event.real_base_before = u64::MAX;
        event.real_quote_before = u64::MAX;
        event.real_base_after = u64::MAX;
        event.real_quote_after = u64::MAX;
        event.amount_in = u64::MAX;
        event.amount_out = u64::MAX;
        event.protocol_fee = u64::MAX;
        event.platform_fee = u64::MAX;
        event.share_fee = u64::MAX;
        event.minimum_amount_out = u64::MAX;
        event.maximum_amount_in = u64::MAX;
        event.share_fee_rate = u64::MAX;

        assert_eq!(event.total_base_sell, u64::MAX);
        assert_eq!(event.virtual_base, u64::MAX);
        assert_eq!(event.virtual_quote, u64::MAX);
        assert_eq!(event.real_base_before, u64::MAX);
        assert_eq!(event.real_quote_before, u64::MAX);
        assert_eq!(event.real_base_after, u64::MAX);
        assert_eq!(event.real_quote_after, u64::MAX);
        assert_eq!(event.amount_in, u64::MAX);
        assert_eq!(event.amount_out, u64::MAX);
        assert_eq!(event.protocol_fee, u64::MAX);
        assert_eq!(event.platform_fee, u64::MAX);
        assert_eq!(event.share_fee, u64::MAX);
        assert_eq!(event.minimum_amount_out, u64::MAX);
        assert_eq!(event.maximum_amount_in, u64::MAX);
        assert_eq!(event.share_fee_rate, u64::MAX);
    }

    #[test]
    fn test_bonk_trade_event_boolean_flags() {
        let mut event = BonkTradeEvent::default();

        // Test default values
        assert!(!event.is_dev_create_token_trade);
        assert!(!event.is_bot);

        // Test setting to true
        event.is_dev_create_token_trade = true;
        event.is_bot = true;
        assert!(event.is_dev_create_token_trade);
        assert!(event.is_bot);

        // Test setting back to false
        event.is_dev_create_token_trade = false;
        event.is_bot = false;
        assert!(!event.is_dev_create_token_trade);
        assert!(!event.is_bot);
    }

    #[test]
    fn test_bonk_pool_create_event_with_complex_curve_params() {
        let mut event = BonkPoolCreateEvent::default();

        // Test with all curve types
        event.curve_param = CurveParams {
            constant_curve: Some(ConstantCurve {
                supply: 1000000,
                total_base_sell: 500000,
                total_quote_fund_raising: 10000,
                migrate_type: 1,
            }),
            fixed_curve: Some(FixedCurve {
                supply: 2000000,
                total_quote_fund_raising: 20000,
                migrate_type: 2,
            }),
            linear_curve: Some(LinearCurve {
                supply: 3000000,
                total_quote_fund_raising: 30000,
                migrate_type: 3,
            }),
        };

        assert!(event.curve_param.constant_curve.is_some());
        assert!(event.curve_param.fixed_curve.is_some());
        assert!(event.curve_param.linear_curve.is_some());

        let constant = event.curve_param.constant_curve.as_ref().unwrap();
        assert_eq!(constant.supply, 1000000);
        assert_eq!(constant.total_base_sell, 500000);
        assert_eq!(constant.total_quote_fund_raising, 10000);
        assert_eq!(constant.migrate_type, 1);

        let fixed = event.curve_param.fixed_curve.as_ref().unwrap();
        assert_eq!(fixed.supply, 2000000);
        assert_eq!(fixed.total_quote_fund_raising, 20000);
        assert_eq!(fixed.migrate_type, 2);

        let linear = event.curve_param.linear_curve.as_ref().unwrap();
        assert_eq!(linear.supply, 3000000);
        assert_eq!(linear.total_quote_fund_raising, 30000);
        assert_eq!(linear.migrate_type, 3);
    }

    #[test]
    fn test_bonk_pool_create_event_with_empty_strings() {
        let mut event = BonkPoolCreateEvent::default();
        event.base_mint_param = MintParams {
            decimals: 0,
            ..Default::default()
        };

        assert_eq!(event.base_mint_param.decimals, 0);
        assert_eq!(event.base_mint_param.name, "");
        assert_eq!(event.base_mint_param.symbol, "");
        assert_eq!(event.base_mint_param.uri, "");
    }

    #[test]
    fn test_bonk_pool_create_event_with_vesting_params_edge_cases() {
        let mut event = BonkPoolCreateEvent::default();

        // Test with zero values
        event.vesting_param = VestingParams {
            total_locked_amount: 0,
            cliff_period: 0,
            unlock_period: 0,
        };

        assert_eq!(event.vesting_param.total_locked_amount, 0);
        assert_eq!(event.vesting_param.cliff_period, 0);
        assert_eq!(event.vesting_param.unlock_period, 0);

        // Test with maximum values
        event.vesting_param = VestingParams {
            total_locked_amount: u64::MAX,
            cliff_period: u64::MAX,
            unlock_period: u64::MAX,
        };

        assert_eq!(event.vesting_param.total_locked_amount, u64::MAX);
        assert_eq!(event.vesting_param.cliff_period, u64::MAX);
        assert_eq!(event.vesting_param.unlock_period, u64::MAX);
    }

    #[test]
    fn test_discriminators_constants() {
        assert_eq!(discriminators::TRADE_EVENT, "bonk_trade_event");
        assert_eq!(discriminators::POOL_CREATE_EVENT, "bonk_pool_create_event");

        assert_eq!(discriminators::TRADE_EVENT_BYTES.len(), 16);
        assert_eq!(discriminators::POOL_CREATE_EVENT_BYTES.len(), 16);

        assert_eq!(discriminators::BUY_EXACT_IN_IX.len(), 8);
        assert_eq!(discriminators::BUY_EXACT_OUT_IX.len(), 8);
        assert_eq!(discriminators::SELL_EXACT_IN_IX.len(), 8);
        assert_eq!(discriminators::SELL_EXACT_OUT_IX.len(), 8);
        assert_eq!(discriminators::INITIALIZE_IX.len(), 8);
        assert_eq!(discriminators::MIGRATE_TO_AMM_IX.len(), 8);
        assert_eq!(discriminators::MIGRATE_TO_CPSWAP_IX.len(), 8);
    }

    #[test]
    fn test_discriminators_byte_arrays_exact_values() {
        assert_eq!(
            discriminators::TRADE_EVENT_BYTES,
            &[
                0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x0e, 0x11, 0xa9, 0xd2, 0xbe, 0x8b,
                0x72, 0xb4
            ]
        );
        assert_eq!(
            discriminators::POOL_CREATE_EVENT_BYTES,
            &[
                0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x1a, 0x11, 0xa9, 0xd2, 0xbe, 0x8b,
                0x72, 0x00
            ]
        );
        assert_eq!(
            discriminators::BUY_EXACT_IN_IX,
            &[64, 198, 72, 0, 130, 84, 226, 13]
        );
        assert_eq!(
            discriminators::BUY_EXACT_OUT_IX,
            &[121, 45, 19, 239, 92, 248, 10, 80]
        );
        assert_eq!(
            discriminators::SELL_EXACT_IN_IX,
            &[155, 233, 77, 24, 217, 193, 201, 60]
        );
        assert_eq!(
            discriminators::SELL_EXACT_OUT_IX,
            &[17, 199, 138, 195, 225, 36, 94, 131]
        );
        assert_eq!(
            discriminators::INITIALIZE_IX,
            &[175, 175, 109, 31, 13, 152, 155, 237]
        );
        assert_eq!(
            discriminators::MIGRATE_TO_AMM_IX,
            &[139, 100, 87, 4, 218, 242, 121, 178]
        );
        assert_eq!(
            discriminators::MIGRATE_TO_CPSWAP_IX,
            &[211, 229, 52, 34, 83, 117, 96, 198]
        );
    }

    #[test]
    fn test_bonk_trade_event_eq_and_ne() {
        let event1 = BonkTradeEvent {
            metadata: create_test_event_metadata(),
            pool_state: create_test_pubkey(),
            total_base_sell: 1000,
            virtual_base: 500,
            virtual_quote: 2000,
            trade_direction: TradeDirection::Buy,
            pool_status: PoolStatus::Trade,
            is_bot: true,
            ..Default::default()
        };

        let event2 = BonkTradeEvent {
            metadata: create_test_event_metadata(),
            pool_state: event1.pool_state,
            total_base_sell: 1000,
            virtual_base: 500,
            virtual_quote: 2000,
            trade_direction: TradeDirection::Buy,
            pool_status: PoolStatus::Trade,
            is_bot: true,
            ..Default::default()
        };

        let event3 = BonkTradeEvent {
            metadata: create_test_event_metadata(),
            pool_state: event1.pool_state,
            total_base_sell: 2000, // Different value
            virtual_base: 500,
            virtual_quote: 2000,
            trade_direction: TradeDirection::Buy,
            pool_status: PoolStatus::Trade,
            is_bot: true,
            ..Default::default()
        };

        assert_eq!(event1, event2);
        assert_ne!(event1, event3);
    }

    #[test]
    fn test_bonk_pool_create_event_eq_and_ne() {
        let shared_metadata = create_test_event_metadata();
        let shared_pool_state = create_test_pubkey();
        let shared_creator = create_test_pubkey();
        let shared_config = create_test_pubkey();

        let event1 = BonkPoolCreateEvent {
            metadata: shared_metadata.clone(),
            pool_state: shared_pool_state,
            creator: shared_creator,
            config: shared_config,
            base_mint_param: create_test_mint_params(),
            curve_param: create_test_curve_params(),
            vesting_param: create_test_vesting_params(),
            ..Default::default()
        };

        let event2 = BonkPoolCreateEvent {
            metadata: shared_metadata.clone(),
            pool_state: shared_pool_state,
            creator: shared_creator,
            config: shared_config,
            base_mint_param: create_test_mint_params(),
            curve_param: create_test_curve_params(),
            vesting_param: create_test_vesting_params(),
            ..Default::default()
        };

        let event3 = BonkPoolCreateEvent {
            metadata: shared_metadata,
            pool_state: shared_pool_state,
            creator: create_test_pubkey(), // Different creator
            config: shared_config,
            base_mint_param: create_test_mint_params(),
            curve_param: create_test_curve_params(),
            vesting_param: create_test_vesting_params(),
            ..Default::default()
        };

        assert_eq!(event1, event2);
        assert_ne!(event1, event3);
    }

    #[test]
    fn test_bonk_trade_event_debug_format() {
        let event = BonkTradeEvent {
            metadata: create_test_event_metadata(),
            pool_state: create_test_pubkey(),
            total_base_sell: 1000,
            trade_direction: TradeDirection::Sell,
            pool_status: PoolStatus::Migrate,
            ..Default::default()
        };

        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("BonkTradeEvent"));
        assert!(debug_str.contains("total_base_sell: 1000"));
        assert!(debug_str.contains("Sell"));
        assert!(debug_str.contains("Migrate"));
    }

    #[test]
    fn test_bonk_pool_create_event_debug_format() {
        let event = BonkPoolCreateEvent {
            metadata: create_test_event_metadata(),
            pool_state: create_test_pubkey(),
            creator: create_test_pubkey(),
            base_mint_param: create_test_mint_params(),
            ..Default::default()
        };

        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("BonkPoolCreateEvent"));
        assert!(debug_str.contains("Test Token"));
        assert!(debug_str.contains("TEST"));
    }

    #[test]
    fn test_bonk_trade_event_partial_eq_trait_coverage() {
        let event1 = BonkTradeEvent::default();
        let event2 = BonkTradeEvent::default();

        // Test PartialEq implementation
        assert!(event1 == event2);
        assert!(!(event1 != event2));
    }

    #[test]
    fn test_bonk_pool_create_event_partial_eq_trait_coverage() {
        let event1 = BonkPoolCreateEvent::default();
        let event2 = BonkPoolCreateEvent::default();

        // Test PartialEq implementation
        assert!(event1 == event2);
        assert!(!(event1 != event2));
    }

    // Test cases to ensure 100% coverage of all possible branches and error paths

    #[test]
    fn test_event_metadata_with_different_kinds() {
        let mut event = BonkTradeEvent::default();

        // Test with different EventKind values
        let core = riglr_events_core::EventMetadata::new(
            "test-1".to_string(),
            EventKind::Transaction,
            "source".to_string(),
        );
        event.metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            12345,
            crate::types::EventType::Swap,
            crate::types::ProtocolType::Bonk,
            "0".to_string(),
            1640995200000,
            core,
        );
        assert_eq!(event.kind(), &EventKind::Transaction);

        event.metadata.core.kind = EventKind::Liquidity;
        assert_eq!(event.kind(), &EventKind::Liquidity);

        event.metadata.core.kind = EventKind::Custom("bonk-custom".to_string());
        assert_eq!(event.kind(), &EventKind::Custom("bonk-custom".to_string()));
    }

    #[test]
    fn test_event_metadata_modification_through_trait() {
        let mut trade_event = BonkTradeEvent::default();
        trade_event.metadata = create_test_event_metadata();

        let mut pool_event = BonkPoolCreateEvent::default();
        pool_event.metadata = create_test_event_metadata();

        // Test metadata modification through trait methods
        let trade_meta_mut = trade_event.metadata_mut();
        trade_meta_mut.id = "modified-trade-id".to_string();
        assert_eq!(trade_event.id(), "modified-trade-id");

        let pool_meta_mut = pool_event.metadata_mut();
        pool_meta_mut.id = "modified-pool-id".to_string();
        assert_eq!(pool_event.id(), "modified-pool-id");
    }

    #[test]
    fn test_cross_type_downcasting() {
        let trade_event = BonkTradeEvent::default();
        let pool_event = BonkPoolCreateEvent::default();

        // Test that downcasting to wrong type fails
        let trade_any = trade_event.as_any();
        assert!(trade_any.downcast_ref::<BonkPoolCreateEvent>().is_none());
        assert!(trade_any.downcast_ref::<BonkTradeEvent>().is_some());

        let pool_any = pool_event.as_any();
        assert!(pool_any.downcast_ref::<BonkTradeEvent>().is_none());
        assert!(pool_any.downcast_ref::<BonkPoolCreateEvent>().is_some());
    }

    #[test]
    fn test_serde_skip_fields_not_in_json() {
        let mut event = BonkTradeEvent::default();
        event.minimum_amount_out = 1000;
        event.maximum_amount_in = 2000;
        event.share_fee_rate = 500;
        event.payer = create_test_pubkey();
        event.is_dev_create_token_trade = true;
        event.is_bot = true;

        let json_value = serde_json::to_value(&event).unwrap();

        // These fields should not appear in JSON due to #[serde(skip)]
        assert!(json_value.get("minimum_amount_out").is_none());
        assert!(json_value.get("maximum_amount_in").is_none());
        assert!(json_value.get("share_fee_rate").is_none());
        assert!(json_value.get("payer").is_none());
        assert!(json_value.get("is_dev_create_token_trade").is_none());
        assert!(json_value.get("is_bot").is_none());
        assert!(json_value.get("metadata").is_none());

        // These fields should appear
        assert!(json_value.get("pool_state").is_some());
        assert!(json_value.get("total_base_sell").is_some());
        assert!(json_value.get("trade_direction").is_some());
    }

    #[test]
    fn test_pool_create_event_serde_skip_fields() {
        let mut event = BonkPoolCreateEvent::default();
        event.payer = create_test_pubkey();
        event.base_mint = create_test_pubkey();
        event.quote_mint = create_test_pubkey();
        event.global_config = create_test_pubkey();

        let json_value = serde_json::to_value(&event).unwrap();

        // These fields should not appear in JSON due to #[serde(skip)]
        assert!(json_value.get("payer").is_none());
        assert!(json_value.get("base_mint").is_none());
        assert!(json_value.get("quote_mint").is_none());
        assert!(json_value.get("global_config").is_none());
        assert!(json_value.get("metadata").is_none());

        // These fields should appear
        assert!(json_value.get("pool_state").is_some());
        assert!(json_value.get("creator").is_some());
        assert!(json_value.get("base_mint_param").is_some());
    }
}
