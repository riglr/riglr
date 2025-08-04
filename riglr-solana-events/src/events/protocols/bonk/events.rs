// UnifiedEvent trait removed - events now implement Event trait directly
use crate::events::common::EventMetadata;
use crate::events::protocols::bonk::types::{
    CurveParams, MintParams, PoolStatus, TradeDirection, VestingParams,
};
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

// Import new Event trait from riglr-events-core
use riglr_events_core::{Event, EventKind, EventMetadata as CoreEventMetadata};
use std::any::Any;

/// Trade event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct BonkTradeEvent {
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: EventMetadata,
    #[serde(skip)]
    #[borsh(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
    pub pool_state: Pubkey,
    pub total_base_sell: u64,
    pub virtual_base: u64,
    pub virtual_quote: u64,
    pub real_base_before: u64,
    pub real_quote_before: u64,
    pub real_base_after: u64,
    pub real_quote_after: u64,
    pub amount_in: u64,
    pub amount_out: u64,
    pub protocol_fee: u64,
    pub platform_fee: u64,
    pub share_fee: u64,
    pub trade_direction: TradeDirection,
    pub pool_status: PoolStatus,
    #[serde(skip)]
    pub minimum_amount_out: u64,
    #[serde(skip)]
    pub maximum_amount_in: u64,
    #[serde(skip)]
    pub share_fee_rate: u64,
    #[serde(skip)]
    pub payer: Pubkey,
    #[serde(skip)]
    pub user_base_token: Pubkey,
    #[serde(skip)]
    pub user_quote_token: Pubkey,
    #[serde(skip)]
    pub base_vault: Pubkey,
    #[serde(skip)]
    pub quote_vault: Pubkey,
    #[serde(skip)]
    pub base_token_mint: Pubkey,
    #[serde(skip)]
    pub quote_token_mint: Pubkey,
    #[serde(skip)]
    pub is_dev_create_token_trade: bool,
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
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            // Fallback: create EventKind from legacy EventType
            &EventKind::Swap // Bonk trades are swaps
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata.as_ref().unwrap_or_else(|| {
            // This should not happen in practice once migration is complete
            panic!("Core metadata not initialized for BonkTradeEvent")
        })
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        if self.core_metadata.is_none() {
            // Initialize core metadata from legacy metadata if needed
            self.core_metadata = Some(
                self.metadata
                    .to_core_metadata(self.metadata.event_type.to_event_kind(), "bonk".to_string()),
            );
        }
        self.core_metadata.as_mut().unwrap()
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct BonkPoolCreateEvent {
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: EventMetadata,
    #[serde(skip)]
    #[borsh(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
    pub pool_state: Pubkey,
    pub creator: Pubkey,
    pub config: Pubkey,
    pub base_mint_param: MintParams,
    pub curve_param: CurveParams,
    pub vesting_param: VestingParams,
    #[serde(skip)]
    pub payer: Pubkey,
    #[serde(skip)]
    pub base_mint: Pubkey,
    #[serde(skip)]
    pub quote_mint: Pubkey,
    #[serde(skip)]
    pub base_vault: Pubkey,
    #[serde(skip)]
    pub quote_vault: Pubkey,
    #[serde(skip)]
    pub global_config: Pubkey,
    #[serde(skip)]
    pub platform_config: Pubkey,
}

// New Event trait implementation
impl Event for BonkPoolCreateEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            // Fallback: create EventKind from legacy EventType
            &EventKind::Liquidity // Pool creation is liquidity-related
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata.as_ref().unwrap_or_else(|| {
            // This should not happen in practice once migration is complete
            panic!("Core metadata not initialized for BonkPoolCreateEvent")
        })
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        if self.core_metadata.is_none() {
            // Initialize core metadata from legacy metadata if needed
            self.core_metadata = Some(
                self.metadata
                    .to_core_metadata(self.metadata.event_type.to_event_kind(), "bonk".to_string()),
            );
        }
        self.core_metadata.as_mut().unwrap()
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
    // Event discriminators - more efficient byte arrays for comparison
    pub const TRADE_EVENT: &str = "bonk_trade_event";
    pub const POOL_CREATE_EVENT: &str = "bonk_pool_create_event";

    // Raw event discriminators as byte arrays for efficient parsing
    pub const TRADE_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x0e, 0x11, 0xa9, 0xd2, 0xbe, 0x8b, 0x72,
        0xb4,
    ];
    pub const POOL_CREATE_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x1a, 0x11, 0xa9, 0xd2, 0xbe, 0x8b, 0x72,
        0x00,
    ];

    // Instruction discriminators
    pub const BUY_EXACT_IN_IX: &[u8] = &[64, 198, 72, 0, 130, 84, 226, 13];
    pub const BUY_EXACT_OUT_IX: &[u8] = &[121, 45, 19, 239, 92, 248, 10, 80];
    pub const SELL_EXACT_IN_IX: &[u8] = &[155, 233, 77, 24, 217, 193, 201, 60];
    pub const SELL_EXACT_OUT_IX: &[u8] = &[17, 199, 138, 195, 225, 36, 94, 131];
    pub const INITIALIZE_IX: &[u8] = &[175, 175, 109, 31, 13, 152, 155, 237];
    pub const MIGRATE_TO_AMM_IX: &[u8] = &[139, 100, 87, 4, 218, 242, 121, 178];
    pub const MIGRATE_TO_CPSWAP_IX: &[u8] = &[211, 229, 52, 34, 83, 117, 96, 198];
}
