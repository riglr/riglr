use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::events::common::EventMetadata;
// UnifiedEvent trait removed - events now implement Event trait directly

// Import new Event trait from riglr-events-core
use riglr_events_core::{Event, EventKind, EventMetadata as CoreEventMetadata};
use std::any::Any;

/// Buy event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpSwapBuyEvent {
    #[serde(skip)]
    pub metadata: EventMetadata,
    #[serde(skip)]
    #[borsh(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
    pub timestamp: i64,
    pub base_amount_out: u64,
    pub max_quote_amount_in: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub quote_amount_in: u64,
    pub lp_fee_basis_points: u64,
    pub lp_fee: u64,
    pub protocol_fee_basis_points: u64,
    pub protocol_fee: u64,
    pub quote_amount_in_with_lp_fee: u64,
    pub user_quote_amount_in: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub protocol_fee_recipient: Pubkey,
    pub protocol_fee_recipient_token_account: Pubkey,
    pub coin_creator: Pubkey,
    pub coin_creator_fee_basis_points: u64,
    pub coin_creator_fee: u64,
    pub track_volume: bool,
    pub total_unclaimed_tokens: u64,
    pub total_claimed_tokens: u64,
    pub current_sol_volume: u64,
    pub last_update_timestamp: i64,
    #[serde(skip)]
    pub base_mint: Pubkey,
    #[serde(skip)]
    pub quote_mint: Pubkey,
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
    #[serde(skip)]
    pub coin_creator_vault_ata: Pubkey,
    #[serde(skip)]
    pub coin_creator_vault_authority: Pubkey,
}

// Use macro to generate UnifiedEvent implementation

// New Event trait implementation
impl Event for PumpSwapBuyEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            &EventKind::Swap // PumpSwap buy is a swap
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata
            .as_ref()
            .unwrap_or_else(|| panic!("Core metadata not initialized for PumpSwapBuyEvent"))
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        if self.core_metadata.is_none() {
            self.core_metadata = Some(self.metadata.to_core_metadata(
                self.metadata.event_type.to_event_kind(),
                "pumpswap".to_string(),
            ));
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

/// Sell event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpSwapSellEvent {
    #[serde(skip)]
    pub metadata: EventMetadata,
    #[serde(skip)]
    #[borsh(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
    pub timestamp: i64,
    pub base_amount_in: u64,
    pub min_quote_amount_out: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub quote_amount_out: u64,
    pub lp_fee_basis_points: u64,
    pub lp_fee: u64,
    pub protocol_fee_basis_points: u64,
    pub protocol_fee: u64,
    pub quote_amount_out_without_lp_fee: u64,
    pub user_quote_amount_out: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub protocol_fee_recipient: Pubkey,
    pub protocol_fee_recipient_token_account: Pubkey,
    pub coin_creator: Pubkey,
    pub coin_creator_fee_basis_points: u64,
    pub coin_creator_fee: u64,
    #[serde(skip)]
    pub base_mint: Pubkey,
    #[serde(skip)]
    pub quote_mint: Pubkey,
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
    #[serde(skip)]
    pub coin_creator_vault_ata: Pubkey,
    #[serde(skip)]
    pub coin_creator_vault_authority: Pubkey,
}

// Use macro to generate UnifiedEvent implementation

// New Event trait implementation
impl Event for PumpSwapSellEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            &EventKind::Swap // PumpSwap sell is a swap
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata
            .as_ref()
            .unwrap_or_else(|| panic!("Core metadata not initialized for PumpSwapSellEvent"))
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        if self.core_metadata.is_none() {
            self.core_metadata = Some(self.metadata.to_core_metadata(
                self.metadata.event_type.to_event_kind(),
                "pumpswap".to_string(),
            ));
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
pub struct PumpSwapCreatePoolEvent {
    #[serde(skip)]
    pub metadata: EventMetadata,
    #[serde(skip)]
    #[borsh(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
    pub timestamp: i64,
    pub index: u16,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_mint_decimals: u8,
    pub quote_mint_decimals: u8,
    pub base_amount_in: u64,
    pub quote_amount_in: u64,
    pub pool_base_amount: u64,
    pub pool_quote_amount: u64,
    pub minimum_liquidity: u64,
    pub initial_liquidity: u64,
    pub lp_token_amount_out: u64,
    pub pool_bump: u8,
    pub pool: Pubkey,
    pub lp_mint: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub coin_creator: Pubkey,
    #[serde(skip)]
    pub user_pool_token_account: Pubkey,
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
}

// New Event trait implementation
impl Event for PumpSwapCreatePoolEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            &EventKind::Liquidity // Pool creation is liquidity-related
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata
            .as_ref()
            .unwrap_or_else(|| panic!("Core metadata not initialized for PumpSwapCreatePoolEvent"))
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        if self.core_metadata.is_none() {
            self.core_metadata = Some(self.metadata.to_core_metadata(
                self.metadata.event_type.to_event_kind(),
                "pumpswap".to_string(),
            ));
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

/// Deposit event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpSwapDepositEvent {
    #[serde(skip)]
    pub metadata: EventMetadata,
    #[serde(skip)]
    #[borsh(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
    pub timestamp: i64,
    pub lp_token_amount_out: u64,
    pub max_base_amount_in: u64,
    pub max_quote_amount_in: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub base_amount_in: u64,
    pub quote_amount_in: u64,
    pub lp_mint_supply: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub user_pool_token_account: Pubkey,
    #[serde(skip)]
    pub base_mint: Pubkey,
    #[serde(skip)]
    pub quote_mint: Pubkey,
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
}

// New Event trait implementation
impl Event for PumpSwapDepositEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            &EventKind::Liquidity // Deposit is liquidity-related
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata
            .as_ref()
            .unwrap_or_else(|| panic!("Core metadata not initialized for PumpSwapDepositEvent"))
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        if self.core_metadata.is_none() {
            self.core_metadata = Some(self.metadata.to_core_metadata(
                self.metadata.event_type.to_event_kind(),
                "pumpswap".to_string(),
            ));
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

/// Withdraw event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpSwapWithdrawEvent {
    #[serde(skip)]
    pub metadata: EventMetadata,
    #[serde(skip)]
    #[borsh(skip)]
    pub core_metadata: Option<CoreEventMetadata>,
    pub timestamp: i64,
    pub lp_token_amount_in: u64,
    pub min_base_amount_out: u64,
    pub min_quote_amount_out: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub base_amount_out: u64,
    pub quote_amount_out: u64,
    pub lp_mint_supply: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub user_pool_token_account: Pubkey,
    #[serde(skip)]
    pub base_mint: Pubkey,
    #[serde(skip)]
    pub quote_mint: Pubkey,
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
}

// New Event trait implementation
impl Event for PumpSwapWithdrawEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        if let Some(ref core_metadata) = self.core_metadata {
            &core_metadata.kind
        } else {
            &EventKind::Liquidity // Withdraw is liquidity-related
        }
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.core_metadata
            .as_ref()
            .unwrap_or_else(|| panic!("Core metadata not initialized for PumpSwapWithdrawEvent"))
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        if self.core_metadata.is_none() {
            self.core_metadata = Some(self.metadata.to_core_metadata(
                self.metadata.event_type.to_event_kind(),
                "pumpswap".to_string(),
            ));
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
    // Event discriminators - more efficient string identifiers
    pub const BUY_EVENT: &str = "pumpswap_buy_event";
    pub const SELL_EVENT: &str = "pumpswap_sell_event";
    pub const CREATE_POOL_EVENT: &str = "pumpswap_create_pool_event";
    pub const DEPOSIT_EVENT: &str = "pumpswap_deposit_event";
    pub const WITHDRAW_EVENT: &str = "pumpswap_withdraw_event";

    // Raw event discriminators as byte arrays for efficient parsing
    pub const BUY_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x67, 0xf4, 0x52, 0x1f, 0x2c, 0xf5, 0x77,
        0x77,
    ];
    pub const SELL_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x3e, 0x2f, 0x37, 0x0a, 0xa5, 0x03, 0xdc,
        0x2a,
    ];
    pub const CREATE_POOL_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0xb1, 0x31, 0x0c, 0xd2, 0xa0, 0x76, 0xa7,
        0x74,
    ];
    pub const DEPOSIT_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x78, 0xf8, 0x3d, 0x53, 0x1f, 0x8e, 0x6b,
        0x90,
    ];
    pub const WITHDRAW_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x16, 0x09, 0x85, 0x1a, 0xa0, 0x2c, 0x47,
        0xc0,
    ];

    // Instruction discriminators
    pub const BUY_IX: &[u8] = &[102, 6, 61, 18, 1, 218, 235, 234];
    pub const SELL_IX: &[u8] = &[51, 230, 133, 164, 1, 127, 131, 173];
    pub const CREATE_POOL_IX: &[u8] = &[233, 146, 209, 142, 207, 104, 64, 188];
    pub const DEPOSIT_IX: &[u8] = &[242, 35, 198, 137, 82, 225, 242, 182];
    pub const WITHDRAW_IX: &[u8] = &[183, 18, 70, 156, 148, 109, 161, 34];
}
