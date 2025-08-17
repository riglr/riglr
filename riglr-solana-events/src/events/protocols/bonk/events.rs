// Events now implement Event trait directly
use crate::events::protocols::bonk::types::{
    CurveParams, MintParams, PoolStatus, TradeDirection, VestingParams,
};
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

// Import Event trait and EventMetadata from riglr-events-core
use riglr_events_core::{Event, EventKind, EventMetadata};
use std::any::Any;

/// Trade event
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
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
