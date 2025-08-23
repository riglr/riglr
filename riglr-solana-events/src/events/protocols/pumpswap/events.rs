use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

// UnifiedEvent trait removed - events now implement Event trait directly

// Import new Event trait from riglr-events-core
use riglr_events_core::{Event, EventKind, EventMetadata as CoreEventMetadata};
// Import the SolanaEventMetadata which is the correct type for Solana events
use crate::types::EventMetadata;
use std::any::Any;

/// Buy event
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize, Default,
)]
pub struct PumpSwapBuyEvent {
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: EventMetadata,
    /// Block timestamp when the event occurred
    pub timestamp: i64,
    /// Amount of base tokens received from the swap
    pub base_amount_out: u64,
    /// Maximum amount of quote tokens willing to spend
    pub max_quote_amount_in: u64,
    /// User's base token balance after the swap
    pub user_base_token_reserves: u64,
    /// User's quote token balance after the swap
    pub user_quote_token_reserves: u64,
    /// Pool's base token reserves after the swap
    pub pool_base_token_reserves: u64,
    /// Pool's quote token reserves after the swap
    pub pool_quote_token_reserves: u64,
    /// Actual amount of quote tokens spent in the swap
    pub quote_amount_in: u64,
    /// Liquidity provider fee in basis points
    pub lp_fee_basis_points: u64,
    /// Liquidity provider fee amount
    pub lp_fee: u64,
    /// Protocol fee in basis points
    pub protocol_fee_basis_points: u64,
    /// Protocol fee amount
    pub protocol_fee: u64,
    /// Quote amount including LP fees
    pub quote_amount_in_with_lp_fee: u64,
    /// User's quote amount input
    pub user_quote_amount_in: u64,
    /// Pool public key
    pub pool: Pubkey,
    /// User wallet public key
    pub user: Pubkey,
    /// User's base token account public key
    pub user_base_token_account: Pubkey,
    /// User's quote token account public key
    pub user_quote_token_account: Pubkey,
    /// Protocol fee recipient public key
    pub protocol_fee_recipient: Pubkey,
    /// Protocol fee recipient token account public key
    pub protocol_fee_recipient_token_account: Pubkey,
    /// Coin creator public key
    pub coin_creator: Pubkey,
    /// Coin creator fee in basis points
    pub coin_creator_fee_basis_points: u64,
    /// Coin creator fee amount
    pub coin_creator_fee: u64,
    /// Whether to track volume for this swap
    pub track_volume: bool,
    /// Total unclaimed tokens
    pub total_unclaimed_tokens: u64,
    /// Total claimed tokens
    pub total_claimed_tokens: u64,
    /// Current SOL volume
    pub current_sol_volume: u64,
    /// Last update timestamp
    pub last_update_timestamp: i64,
    /// Base token mint public key (excluded from serialization)
    #[serde(skip)]
    pub base_mint: Pubkey,
    /// Quote token mint public key (excluded from serialization)
    #[serde(skip)]
    pub quote_mint: Pubkey,
    /// Pool's base token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    /// Pool's quote token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
    /// Coin creator vault ATA public key (excluded from serialization)
    #[serde(skip)]
    pub coin_creator_vault_ata: Pubkey,
    /// Coin creator vault authority public key (excluded from serialization)
    #[serde(skip)]
    pub coin_creator_vault_authority: Pubkey,
}

// Use macro to generate UnifiedEvent implementation

// New Event trait implementation
impl Event for PumpSwapBuyEvent {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        &self.metadata.core.kind
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
        serde_json::to_value(self).map_err(riglr_events_core::error::EventError::Serialization)
    }
}

/// Sell event
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize, Default,
)]
pub struct PumpSwapSellEvent {
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: EventMetadata,
    /// Block timestamp when the event occurred
    pub timestamp: i64,
    /// Amount of base tokens being sold
    pub base_amount_in: u64,
    /// Minimum amount of quote tokens expected to receive
    pub min_quote_amount_out: u64,
    /// User's base token balance after the swap
    pub user_base_token_reserves: u64,
    /// User's quote token balance after the swap
    pub user_quote_token_reserves: u64,
    /// Pool's base token reserves after the swap
    pub pool_base_token_reserves: u64,
    /// Pool's quote token reserves after the swap
    pub pool_quote_token_reserves: u64,
    /// Actual amount of quote tokens received from the swap
    pub quote_amount_out: u64,
    /// Liquidity provider fee in basis points
    pub lp_fee_basis_points: u64,
    /// Liquidity provider fee amount
    pub lp_fee: u64,
    /// Protocol fee in basis points
    pub protocol_fee_basis_points: u64,
    /// Protocol fee amount
    pub protocol_fee: u64,
    /// Quote amount without LP fees
    pub quote_amount_out_without_lp_fee: u64,
    /// User's quote amount output
    pub user_quote_amount_out: u64,
    /// Pool public key
    pub pool: Pubkey,
    /// User wallet public key
    pub user: Pubkey,
    /// User's base token account public key
    pub user_base_token_account: Pubkey,
    /// User's quote token account public key
    pub user_quote_token_account: Pubkey,
    /// Protocol fee recipient public key
    pub protocol_fee_recipient: Pubkey,
    /// Protocol fee recipient token account public key
    pub protocol_fee_recipient_token_account: Pubkey,
    /// Coin creator public key
    pub coin_creator: Pubkey,
    /// Coin creator fee in basis points
    pub coin_creator_fee_basis_points: u64,
    /// Coin creator fee amount
    pub coin_creator_fee: u64,
    /// Base token mint public key (excluded from serialization)
    #[serde(skip)]
    pub base_mint: Pubkey,
    /// Quote token mint public key (excluded from serialization)
    #[serde(skip)]
    pub quote_mint: Pubkey,
    /// Pool's base token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    /// Pool's quote token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
    /// Coin creator vault ATA public key (excluded from serialization)
    #[serde(skip)]
    pub coin_creator_vault_ata: Pubkey,
    /// Coin creator vault authority public key (excluded from serialization)
    #[serde(skip)]
    pub coin_creator_vault_authority: Pubkey,
}

// Use macro to generate UnifiedEvent implementation

// New Event trait implementation
impl Event for PumpSwapSellEvent {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        &self.metadata.core.kind
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
        serde_json::to_value(self).map_err(riglr_events_core::error::EventError::Serialization)
    }
}

/// Create pool event
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize, Default,
)]
pub struct PumpSwapCreatePoolEvent {
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: EventMetadata,
    /// Block timestamp when the event occurred
    pub timestamp: i64,
    /// Pool index identifier
    pub index: u16,
    /// Pool creator public key
    pub creator: Pubkey,
    /// Base token mint public key
    pub base_mint: Pubkey,
    /// Quote token mint public key
    pub quote_mint: Pubkey,
    /// Base token decimal places
    pub base_mint_decimals: u8,
    /// Quote token decimal places
    pub quote_mint_decimals: u8,
    /// Amount of base tokens deposited
    pub base_amount_in: u64,
    /// Amount of quote tokens deposited
    pub quote_amount_in: u64,
    /// Pool's base token amount
    pub pool_base_amount: u64,
    /// Pool's quote token amount
    pub pool_quote_amount: u64,
    /// Minimum liquidity required
    pub minimum_liquidity: u64,
    /// Initial liquidity provided
    pub initial_liquidity: u64,
    /// LP tokens minted
    pub lp_token_amount_out: u64,
    /// Pool bump seed
    pub pool_bump: u8,
    /// Pool public key
    pub pool: Pubkey,
    /// LP token mint public key
    pub lp_mint: Pubkey,
    /// User's base token account public key
    pub user_base_token_account: Pubkey,
    /// User's quote token account public key
    pub user_quote_token_account: Pubkey,
    /// Coin creator public key
    pub coin_creator: Pubkey,
    /// User's pool token account public key (excluded from serialization)
    #[serde(skip)]
    pub user_pool_token_account: Pubkey,
    /// Pool's base token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    /// Pool's quote token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
}

// New Event trait implementation
impl Event for PumpSwapCreatePoolEvent {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        &self.metadata.core.kind
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
        serde_json::to_value(self).map_err(riglr_events_core::error::EventError::Serialization)
    }
}

/// Deposit event
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize, Default,
)]
pub struct PumpSwapDepositEvent {
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: EventMetadata,
    /// Block timestamp when the event occurred
    pub timestamp: i64,
    /// Amount of LP tokens minted
    pub lp_token_amount_out: u64,
    /// Maximum base token amount willing to deposit
    pub max_base_amount_in: u64,
    /// Maximum quote token amount willing to deposit
    pub max_quote_amount_in: u64,
    /// User's base token balance after deposit
    pub user_base_token_reserves: u64,
    /// User's quote token balance after deposit
    pub user_quote_token_reserves: u64,
    /// Pool's base token reserves after deposit
    pub pool_base_token_reserves: u64,
    /// Pool's quote token reserves after deposit
    pub pool_quote_token_reserves: u64,
    /// Actual base token amount deposited
    pub base_amount_in: u64,
    /// Actual quote token amount deposited
    pub quote_amount_in: u64,
    /// Total LP token supply after deposit
    pub lp_mint_supply: u64,
    /// Pool public key
    pub pool: Pubkey,
    /// User wallet public key
    pub user: Pubkey,
    /// User's base token account public key
    pub user_base_token_account: Pubkey,
    /// User's quote token account public key
    pub user_quote_token_account: Pubkey,
    /// User's pool token account public key
    pub user_pool_token_account: Pubkey,
    /// Base token mint public key (excluded from serialization)
    #[serde(skip)]
    pub base_mint: Pubkey,
    /// Quote token mint public key (excluded from serialization)
    #[serde(skip)]
    pub quote_mint: Pubkey,
    /// Pool's base token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    /// Pool's quote token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
}

// New Event trait implementation
impl Event for PumpSwapDepositEvent {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        &self.metadata.core.kind
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
        serde_json::to_value(self).map_err(riglr_events_core::error::EventError::Serialization)
    }
}

/// Withdraw event
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize, Default,
)]
pub struct PumpSwapWithdrawEvent {
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: EventMetadata,
    /// Block timestamp when the event occurred
    pub timestamp: i64,
    /// Amount of LP tokens burned
    pub lp_token_amount_in: u64,
    /// Minimum base token amount expected to receive
    pub min_base_amount_out: u64,
    /// Minimum quote token amount expected to receive
    pub min_quote_amount_out: u64,
    /// User's base token balance after withdrawal
    pub user_base_token_reserves: u64,
    /// User's quote token balance after withdrawal
    pub user_quote_token_reserves: u64,
    /// Pool's base token reserves after withdrawal
    pub pool_base_token_reserves: u64,
    /// Pool's quote token reserves after withdrawal
    pub pool_quote_token_reserves: u64,
    /// Actual base token amount withdrawn
    pub base_amount_out: u64,
    /// Actual quote token amount withdrawn
    pub quote_amount_out: u64,
    /// Total LP token supply after withdrawal
    pub lp_mint_supply: u64,
    /// Pool public key
    pub pool: Pubkey,
    /// User wallet public key
    pub user: Pubkey,
    /// User's base token account public key
    pub user_base_token_account: Pubkey,
    /// User's quote token account public key
    pub user_quote_token_account: Pubkey,
    /// User's pool token account public key
    pub user_pool_token_account: Pubkey,
    /// Base token mint public key (excluded from serialization)
    #[serde(skip)]
    pub base_mint: Pubkey,
    /// Quote token mint public key (excluded from serialization)
    #[serde(skip)]
    pub quote_mint: Pubkey,
    /// Pool's base token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    /// Pool's quote token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
}

// New Event trait implementation
impl Event for PumpSwapWithdrawEvent {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        &self.metadata.core.kind
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
        serde_json::to_value(self).map_err(riglr_events_core::error::EventError::Serialization)
    }
}

/// Event discriminator constants
pub mod discriminators {
    // Event discriminators - more efficient string identifiers
    /// String identifier for PumpSwap buy events
    pub const BUY_EVENT: &str = "pumpswap_buy_event";
    /// String identifier for PumpSwap sell events
    pub const SELL_EVENT: &str = "pumpswap_sell_event";
    /// String identifier for PumpSwap create pool events
    pub const CREATE_POOL_EVENT: &str = "pumpswap_create_pool_event";
    /// String identifier for PumpSwap deposit events
    pub const DEPOSIT_EVENT: &str = "pumpswap_deposit_event";
    /// String identifier for PumpSwap withdraw events
    pub const WITHDRAW_EVENT: &str = "pumpswap_withdraw_event";

    // Raw event discriminators as byte arrays for efficient parsing
    /// Byte array discriminator for PumpSwap buy events
    pub const BUY_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x67, 0xf4, 0x52, 0x1f, 0x2c, 0xf5, 0x77,
        0x77,
    ];
    /// Byte array discriminator for PumpSwap sell events
    pub const SELL_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x3e, 0x2f, 0x37, 0x0a, 0xa5, 0x03, 0xdc,
        0x2a,
    ];
    /// Byte array discriminator for PumpSwap create pool events
    pub const CREATE_POOL_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0xb1, 0x31, 0x0c, 0xd2, 0xa0, 0x76, 0xa7,
        0x74,
    ];
    /// Byte array discriminator for PumpSwap deposit events
    pub const DEPOSIT_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x78, 0xf8, 0x3d, 0x53, 0x1f, 0x8e, 0x6b,
        0x90,
    ];
    /// Byte array discriminator for PumpSwap withdraw events
    pub const WITHDRAW_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x16, 0x09, 0x85, 0x1a, 0xa0, 0x2c, 0x47,
        0xc0,
    ];

    // Instruction discriminators
    /// Instruction discriminator for PumpSwap buy operations
    pub const BUY_IX: &[u8] = &[102, 6, 61, 18, 1, 218, 235, 234];
    /// Instruction discriminator for PumpSwap sell operations
    pub const SELL_IX: &[u8] = &[51, 230, 133, 164, 1, 127, 131, 173];
    /// Instruction discriminator for PumpSwap create pool operations
    pub const CREATE_POOL_IX: &[u8] = &[233, 146, 209, 142, 207, 104, 64, 188];
    /// Instruction discriminator for PumpSwap deposit operations
    pub const DEPOSIT_IX: &[u8] = &[242, 35, 198, 137, 82, 225, 242, 182];
    /// Instruction discriminator for PumpSwap withdraw operations
    pub const WITHDRAW_IX: &[u8] = &[183, 18, 70, 156, 148, 109, 161, 34];
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_events_core::{EventKind, EventMetadata as CoreEventMetadata};
    use serde_json;

    // Helper function to create test metadata
    fn create_test_metadata() -> CoreEventMetadata {
        use chrono::DateTime;
        use riglr_events_core::types::ChainData;
        use std::collections::HashMap;

        let timestamp = DateTime::from_timestamp(1234567890, 0).unwrap();
        let chain_data = ChainData::Solana {
            slot: 100,
            signature: Some("test_sig".to_string()),
            program_id: Some(Pubkey::default()),
            instruction_index: Some(0),
            block_time: Some(1234567890),
            protocol_data: None,
        };

        CoreEventMetadata {
            id: "test_id".to_string(),
            kind: EventKind::Swap,
            timestamp,
            received_at: timestamp,
            source: "solana-test".to_string(),
            chain_data: Some(chain_data),
            custom: HashMap::new(),
        }
    }

    // Helper function to create test pubkey
    fn create_test_pubkey() -> Pubkey {
        Pubkey::new_from_array([1; 32])
    }

    #[test]
    fn test_pumpswap_buy_event_default() {
        let event = PumpSwapBuyEvent::default();
        assert_eq!(event.timestamp, 0);
        assert_eq!(event.base_amount_out, 0);
        assert_eq!(event.max_quote_amount_in, 0);
        assert!(!event.track_volume);
        assert_eq!(event.pool, Pubkey::default());
        assert_eq!(event.user, Pubkey::default());
    }

    #[test]
    fn test_pumpswap_buy_event_with_values() {
        let mut event = PumpSwapBuyEvent::default();
        event.metadata.core = create_test_metadata();
        event.timestamp = 1234567890;
        event.base_amount_out = 1000;
        event.max_quote_amount_in = 2000;
        event.track_volume = true;
        event.pool = create_test_pubkey();
        event.user = create_test_pubkey();

        assert_eq!(event.timestamp, 1234567890);
        assert_eq!(event.base_amount_out, 1000);
        assert_eq!(event.max_quote_amount_in, 2000);
        assert!(event.track_volume);
        assert_eq!(event.pool, create_test_pubkey());
        assert_eq!(event.user, create_test_pubkey());
    }

    #[test]
    fn test_pumpswap_buy_event_event_trait_id() {
        let mut event = PumpSwapBuyEvent::default();
        event.metadata.core = create_test_metadata();
        assert_eq!(event.id(), "test_id");
    }

    #[test]
    fn test_pumpswap_buy_event_event_trait_kind() {
        let mut event = PumpSwapBuyEvent::default();
        event.metadata.core = create_test_metadata();
        assert_eq!(event.kind(), &EventKind::Swap);
    }

    #[test]
    fn test_pumpswap_buy_event_event_trait_metadata() {
        let mut event = PumpSwapBuyEvent::default();
        event.metadata.core = create_test_metadata();
        let metadata = event.metadata();
        assert_eq!(metadata.id, "test_id");
        assert_eq!(metadata.kind, EventKind::Swap);
    }

    #[test]
    fn test_pumpswap_buy_event_event_trait_metadata_mut() {
        let mut event = PumpSwapBuyEvent::default();
        event.metadata.core = create_test_metadata();
        let metadata_mut = event.metadata_mut();
        metadata_mut.id = "new_id".to_string();
        assert_eq!(event.metadata.core.id, "new_id");
    }

    #[test]
    fn test_pumpswap_buy_event_event_trait_as_any() {
        let event = PumpSwapBuyEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<PumpSwapBuyEvent>().is_some());
    }

    #[test]
    fn test_pumpswap_buy_event_event_trait_as_any_mut() {
        let mut event = PumpSwapBuyEvent::default();
        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<PumpSwapBuyEvent>().is_some());
    }

    #[test]
    fn test_pumpswap_buy_event_event_trait_clone_boxed() {
        let event = PumpSwapBuyEvent::default();
        let cloned = event.clone_boxed();
        assert!(cloned.as_any().downcast_ref::<PumpSwapBuyEvent>().is_some());
    }

    #[test]
    fn test_pumpswap_buy_event_event_trait_to_json() {
        let event = PumpSwapBuyEvent::default();
        let json_result = event.to_json();
        assert!(json_result.is_ok());
        let json_value = json_result.unwrap();
        assert!(json_value.is_object());
    }

    #[test]
    fn test_pumpswap_sell_event_default() {
        let event = PumpSwapSellEvent::default();
        assert_eq!(event.timestamp, 0);
        assert_eq!(event.base_amount_in, 0);
        assert_eq!(event.min_quote_amount_out, 0);
        assert_eq!(event.pool, Pubkey::default());
        assert_eq!(event.user, Pubkey::default());
    }

    #[test]
    fn test_pumpswap_sell_event_with_values() {
        let mut event = PumpSwapSellEvent::default();
        event.metadata.core = create_test_metadata();
        event.timestamp = 1234567890;
        event.base_amount_in = 1000;
        event.min_quote_amount_out = 2000;
        event.pool = create_test_pubkey();
        event.user = create_test_pubkey();

        assert_eq!(event.timestamp, 1234567890);
        assert_eq!(event.base_amount_in, 1000);
        assert_eq!(event.min_quote_amount_out, 2000);
        assert_eq!(event.pool, create_test_pubkey());
        assert_eq!(event.user, create_test_pubkey());
    }

    #[test]
    fn test_pumpswap_sell_event_event_trait_id() {
        let mut event = PumpSwapSellEvent::default();
        event.metadata.core = create_test_metadata();
        assert_eq!(event.id(), "test_id");
    }

    #[test]
    fn test_pumpswap_sell_event_event_trait_kind() {
        let mut event = PumpSwapSellEvent::default();
        event.metadata.core = create_test_metadata();
        assert_eq!(event.kind(), &EventKind::Swap);
    }

    #[test]
    fn test_pumpswap_sell_event_event_trait_metadata() {
        let mut event = PumpSwapSellEvent::default();
        event.metadata.core = create_test_metadata();
        let metadata = event.metadata();
        assert_eq!(metadata.id, "test_id");
        assert_eq!(metadata.kind, EventKind::Swap);
    }

    #[test]
    fn test_pumpswap_sell_event_event_trait_metadata_mut() {
        let mut event = PumpSwapSellEvent::default();
        event.metadata.core = create_test_metadata();
        let metadata_mut = event.metadata_mut();
        metadata_mut.id = "new_id".to_string();
        assert_eq!(event.metadata.core.id, "new_id");
    }

    #[test]
    fn test_pumpswap_sell_event_event_trait_as_any() {
        let event = PumpSwapSellEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<PumpSwapSellEvent>().is_some());
    }

    #[test]
    fn test_pumpswap_sell_event_event_trait_as_any_mut() {
        let mut event = PumpSwapSellEvent::default();
        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<PumpSwapSellEvent>().is_some());
    }

    #[test]
    fn test_pumpswap_sell_event_event_trait_clone_boxed() {
        let event = PumpSwapSellEvent::default();
        let cloned = event.clone_boxed();
        assert!(cloned
            .as_any()
            .downcast_ref::<PumpSwapSellEvent>()
            .is_some());
    }

    #[test]
    fn test_pumpswap_sell_event_event_trait_to_json() {
        let event = PumpSwapSellEvent::default();
        let json_result = event.to_json();
        assert!(json_result.is_ok());
        let json_value = json_result.unwrap();
        assert!(json_value.is_object());
    }

    #[test]
    fn test_pumpswap_create_pool_event_default() {
        let event = PumpSwapCreatePoolEvent::default();
        assert_eq!(event.timestamp, 0);
        assert_eq!(event.index, 0);
        assert_eq!(event.creator, Pubkey::default());
        assert_eq!(event.base_mint, Pubkey::default());
        assert_eq!(event.quote_mint, Pubkey::default());
        assert_eq!(event.base_mint_decimals, 0);
        assert_eq!(event.quote_mint_decimals, 0);
    }

    #[test]
    fn test_pumpswap_create_pool_event_with_values() {
        let mut event = PumpSwapCreatePoolEvent::default();
        event.metadata.core = create_test_metadata();
        event.timestamp = 1234567890;
        event.index = 42;
        event.creator = create_test_pubkey();
        event.base_mint = create_test_pubkey();
        event.quote_mint = create_test_pubkey();
        event.base_mint_decimals = 9;
        event.quote_mint_decimals = 6;

        assert_eq!(event.timestamp, 1234567890);
        assert_eq!(event.index, 42);
        assert_eq!(event.creator, create_test_pubkey());
        assert_eq!(event.base_mint, create_test_pubkey());
        assert_eq!(event.quote_mint, create_test_pubkey());
        assert_eq!(event.base_mint_decimals, 9);
        assert_eq!(event.quote_mint_decimals, 6);
    }

    #[test]
    fn test_pumpswap_create_pool_event_event_trait_id() {
        let mut event = PumpSwapCreatePoolEvent::default();
        event.metadata.core = create_test_metadata();
        assert_eq!(event.id(), "test_id");
    }

    #[test]
    fn test_pumpswap_create_pool_event_event_trait_kind() {
        let mut event = PumpSwapCreatePoolEvent::default();
        event.metadata.core = create_test_metadata();
        assert_eq!(event.kind(), &EventKind::Swap);
    }

    #[test]
    fn test_pumpswap_create_pool_event_event_trait_metadata() {
        let mut event = PumpSwapCreatePoolEvent::default();
        event.metadata.core = create_test_metadata();
        let metadata = event.metadata();
        assert_eq!(metadata.id, "test_id");
        assert_eq!(metadata.kind, EventKind::Swap);
    }

    #[test]
    fn test_pumpswap_create_pool_event_event_trait_metadata_mut() {
        let mut event = PumpSwapCreatePoolEvent::default();
        event.metadata.core = create_test_metadata();
        let metadata_mut = event.metadata_mut();
        metadata_mut.id = "new_id".to_string();
        assert_eq!(event.metadata.core.id, "new_id");
    }

    #[test]
    fn test_pumpswap_create_pool_event_event_trait_as_any() {
        let event = PumpSwapCreatePoolEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<PumpSwapCreatePoolEvent>().is_some());
    }

    #[test]
    fn test_pumpswap_create_pool_event_event_trait_as_any_mut() {
        let mut event = PumpSwapCreatePoolEvent::default();
        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<PumpSwapCreatePoolEvent>().is_some());
    }

    #[test]
    fn test_pumpswap_create_pool_event_event_trait_clone_boxed() {
        let event = PumpSwapCreatePoolEvent::default();
        let cloned = event.clone_boxed();
        assert!(cloned
            .as_any()
            .downcast_ref::<PumpSwapCreatePoolEvent>()
            .is_some());
    }

    #[test]
    fn test_pumpswap_create_pool_event_event_trait_to_json() {
        let event = PumpSwapCreatePoolEvent::default();
        let json_result = event.to_json();
        assert!(json_result.is_ok());
        let json_value = json_result.unwrap();
        assert!(json_value.is_object());
    }

    #[test]
    fn test_pumpswap_deposit_event_default() {
        let event = PumpSwapDepositEvent::default();
        assert_eq!(event.timestamp, 0);
        assert_eq!(event.lp_token_amount_out, 0);
        assert_eq!(event.max_base_amount_in, 0);
        assert_eq!(event.max_quote_amount_in, 0);
        assert_eq!(event.pool, Pubkey::default());
        assert_eq!(event.user, Pubkey::default());
    }

    #[test]
    fn test_pumpswap_deposit_event_with_values() {
        let mut event = PumpSwapDepositEvent::default();
        event.metadata.core = create_test_metadata();
        event.timestamp = 1234567890;
        event.lp_token_amount_out = 1000;
        event.max_base_amount_in = 2000;
        event.max_quote_amount_in = 3000;
        event.pool = create_test_pubkey();
        event.user = create_test_pubkey();

        assert_eq!(event.timestamp, 1234567890);
        assert_eq!(event.lp_token_amount_out, 1000);
        assert_eq!(event.max_base_amount_in, 2000);
        assert_eq!(event.max_quote_amount_in, 3000);
        assert_eq!(event.pool, create_test_pubkey());
        assert_eq!(event.user, create_test_pubkey());
    }

    #[test]
    fn test_pumpswap_deposit_event_event_trait_id() {
        let mut event = PumpSwapDepositEvent::default();
        event.metadata.core = create_test_metadata();
        assert_eq!(event.id(), "test_id");
    }

    #[test]
    fn test_pumpswap_deposit_event_event_trait_kind() {
        let mut event = PumpSwapDepositEvent::default();
        event.metadata.core = create_test_metadata();
        assert_eq!(event.kind(), &EventKind::Swap);
    }

    #[test]
    fn test_pumpswap_deposit_event_event_trait_metadata() {
        let mut event = PumpSwapDepositEvent::default();
        event.metadata.core = create_test_metadata();
        let metadata = event.metadata();
        assert_eq!(metadata.id, "test_id");
        assert_eq!(metadata.kind, EventKind::Swap);
    }

    #[test]
    fn test_pumpswap_deposit_event_event_trait_metadata_mut() {
        let mut event = PumpSwapDepositEvent::default();
        event.metadata.core = create_test_metadata();
        let metadata_mut = event.metadata_mut();
        metadata_mut.id = "new_id".to_string();
        assert_eq!(event.metadata.core.id, "new_id");
    }

    #[test]
    fn test_pumpswap_deposit_event_event_trait_as_any() {
        let event = PumpSwapDepositEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<PumpSwapDepositEvent>().is_some());
    }

    #[test]
    fn test_pumpswap_deposit_event_event_trait_as_any_mut() {
        let mut event = PumpSwapDepositEvent::default();
        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<PumpSwapDepositEvent>().is_some());
    }

    #[test]
    fn test_pumpswap_deposit_event_event_trait_clone_boxed() {
        let event = PumpSwapDepositEvent::default();
        let cloned = event.clone_boxed();
        assert!(cloned
            .as_any()
            .downcast_ref::<PumpSwapDepositEvent>()
            .is_some());
    }

    #[test]
    fn test_pumpswap_deposit_event_event_trait_to_json() {
        let event = PumpSwapDepositEvent::default();
        let json_result = event.to_json();
        assert!(json_result.is_ok());
        let json_value = json_result.unwrap();
        assert!(json_value.is_object());
    }

    #[test]
    fn test_pumpswap_withdraw_event_default() {
        let event = PumpSwapWithdrawEvent::default();
        assert_eq!(event.timestamp, 0);
        assert_eq!(event.lp_token_amount_in, 0);
        assert_eq!(event.min_base_amount_out, 0);
        assert_eq!(event.min_quote_amount_out, 0);
        assert_eq!(event.pool, Pubkey::default());
        assert_eq!(event.user, Pubkey::default());
    }

    #[test]
    fn test_pumpswap_withdraw_event_with_values() {
        let mut event = PumpSwapWithdrawEvent::default();
        event.metadata.core = create_test_metadata();
        event.timestamp = 1234567890;
        event.lp_token_amount_in = 1000;
        event.min_base_amount_out = 2000;
        event.min_quote_amount_out = 3000;
        event.pool = create_test_pubkey();
        event.user = create_test_pubkey();

        assert_eq!(event.timestamp, 1234567890);
        assert_eq!(event.lp_token_amount_in, 1000);
        assert_eq!(event.min_base_amount_out, 2000);
        assert_eq!(event.min_quote_amount_out, 3000);
        assert_eq!(event.pool, create_test_pubkey());
        assert_eq!(event.user, create_test_pubkey());
    }

    #[test]
    fn test_pumpswap_withdraw_event_event_trait_id() {
        let mut event = PumpSwapWithdrawEvent::default();
        event.metadata.core = create_test_metadata();
        assert_eq!(event.id(), "test_id");
    }

    #[test]
    fn test_pumpswap_withdraw_event_event_trait_kind() {
        let mut event = PumpSwapWithdrawEvent::default();
        event.metadata.core = create_test_metadata();
        assert_eq!(event.kind(), &EventKind::Swap);
    }

    #[test]
    fn test_pumpswap_withdraw_event_event_trait_metadata() {
        let mut event = PumpSwapWithdrawEvent::default();
        event.metadata.core = create_test_metadata();
        let metadata = event.metadata();
        assert_eq!(metadata.id, "test_id");
        assert_eq!(metadata.kind, EventKind::Swap);
    }

    #[test]
    fn test_pumpswap_withdraw_event_event_trait_metadata_mut() {
        let mut event = PumpSwapWithdrawEvent::default();
        event.metadata.core = create_test_metadata();
        let metadata_mut = event.metadata_mut();
        metadata_mut.id = "new_id".to_string();
        assert_eq!(event.metadata.core.id, "new_id");
    }

    #[test]
    fn test_pumpswap_withdraw_event_event_trait_as_any() {
        let event = PumpSwapWithdrawEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<PumpSwapWithdrawEvent>().is_some());
    }

    #[test]
    fn test_pumpswap_withdraw_event_event_trait_as_any_mut() {
        let mut event = PumpSwapWithdrawEvent::default();
        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<PumpSwapWithdrawEvent>().is_some());
    }

    #[test]
    fn test_pumpswap_withdraw_event_event_trait_clone_boxed() {
        let event = PumpSwapWithdrawEvent::default();
        let cloned = event.clone_boxed();
        assert!(cloned
            .as_any()
            .downcast_ref::<PumpSwapWithdrawEvent>()
            .is_some());
    }

    #[test]
    fn test_pumpswap_withdraw_event_event_trait_to_json() {
        let event = PumpSwapWithdrawEvent::default();
        let json_result = event.to_json();
        assert!(json_result.is_ok());
        let json_value = json_result.unwrap();
        assert!(json_value.is_object());
    }

    #[test]
    fn test_discriminators_string_constants() {
        assert_eq!(discriminators::BUY_EVENT, "pumpswap_buy_event");
        assert_eq!(discriminators::SELL_EVENT, "pumpswap_sell_event");
        assert_eq!(
            discriminators::CREATE_POOL_EVENT,
            "pumpswap_create_pool_event"
        );
        assert_eq!(discriminators::DEPOSIT_EVENT, "pumpswap_deposit_event");
        assert_eq!(discriminators::WITHDRAW_EVENT, "pumpswap_withdraw_event");
    }

    #[test]
    fn test_discriminators_byte_arrays() {
        assert_eq!(
            discriminators::BUY_EVENT_BYTES,
            &[
                0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x67, 0xf4, 0x52, 0x1f, 0x2c, 0xf5,
                0x77, 0x77
            ]
        );
        assert_eq!(
            discriminators::SELL_EVENT_BYTES,
            &[
                0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x3e, 0x2f, 0x37, 0x0a, 0xa5, 0x03,
                0xdc, 0x2a
            ]
        );
        assert_eq!(
            discriminators::CREATE_POOL_EVENT_BYTES,
            &[
                0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0xb1, 0x31, 0x0c, 0xd2, 0xa0, 0x76,
                0xa7, 0x74
            ]
        );
        assert_eq!(
            discriminators::DEPOSIT_EVENT_BYTES,
            &[
                0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x78, 0xf8, 0x3d, 0x53, 0x1f, 0x8e,
                0x6b, 0x90
            ]
        );
        assert_eq!(
            discriminators::WITHDRAW_EVENT_BYTES,
            &[
                0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x16, 0x09, 0x85, 0x1a, 0xa0, 0x2c,
                0x47, 0xc0
            ]
        );
    }

    #[test]
    fn test_discriminators_instruction_arrays() {
        assert_eq!(discriminators::BUY_IX, &[102, 6, 61, 18, 1, 218, 235, 234]);
        assert_eq!(
            discriminators::SELL_IX,
            &[51, 230, 133, 164, 1, 127, 131, 173]
        );
        assert_eq!(
            discriminators::CREATE_POOL_IX,
            &[233, 146, 209, 142, 207, 104, 64, 188]
        );
        assert_eq!(
            discriminators::DEPOSIT_IX,
            &[242, 35, 198, 137, 82, 225, 242, 182]
        );
        assert_eq!(
            discriminators::WITHDRAW_IX,
            &[183, 18, 70, 156, 148, 109, 161, 34]
        );
    }

    #[test]
    fn test_discriminators_byte_array_lengths() {
        assert_eq!(discriminators::BUY_EVENT_BYTES.len(), 16);
        assert_eq!(discriminators::SELL_EVENT_BYTES.len(), 16);
        assert_eq!(discriminators::CREATE_POOL_EVENT_BYTES.len(), 16);
        assert_eq!(discriminators::DEPOSIT_EVENT_BYTES.len(), 16);
        assert_eq!(discriminators::WITHDRAW_EVENT_BYTES.len(), 16);
    }

    #[test]
    fn test_discriminators_instruction_array_lengths() {
        assert_eq!(discriminators::BUY_IX.len(), 8);
        assert_eq!(discriminators::SELL_IX.len(), 8);
        assert_eq!(discriminators::CREATE_POOL_IX.len(), 8);
        assert_eq!(discriminators::DEPOSIT_IX.len(), 8);
        assert_eq!(discriminators::WITHDRAW_IX.len(), 8);
    }

    #[test]
    fn test_event_equality() {
        let event1 = PumpSwapBuyEvent::default();
        let event2 = PumpSwapBuyEvent::default();
        assert_eq!(event1, event2);

        let event3 = PumpSwapSellEvent::default();
        let event4 = PumpSwapSellEvent::default();
        assert_eq!(event3, event4);

        let event5 = PumpSwapCreatePoolEvent::default();
        let event6 = PumpSwapCreatePoolEvent::default();
        assert_eq!(event5, event6);

        let event7 = PumpSwapDepositEvent::default();
        let event8 = PumpSwapDepositEvent::default();
        assert_eq!(event7, event8);

        let event9 = PumpSwapWithdrawEvent::default();
        let event10 = PumpSwapWithdrawEvent::default();
        assert_eq!(event9, event10);
    }

    #[test]
    fn test_event_clone() {
        let event1 = PumpSwapBuyEvent::default();
        let event2 = event1.clone();
        assert_eq!(event1, event2);

        let event3 = PumpSwapSellEvent::default();
        let event4 = event3.clone();
        assert_eq!(event3, event4);

        let event5 = PumpSwapCreatePoolEvent::default();
        let event6 = event5.clone();
        assert_eq!(event5, event6);

        let event7 = PumpSwapDepositEvent::default();
        let event8 = event7.clone();
        assert_eq!(event7, event8);

        let event9 = PumpSwapWithdrawEvent::default();
        let event10 = event9.clone();
        assert_eq!(event9, event10);
    }

    #[test]
    fn test_event_debug() {
        let event = PumpSwapBuyEvent::default();
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("PumpSwapBuyEvent"));
    }

    #[test]
    fn test_event_serialization() {
        let event = PumpSwapBuyEvent::default();
        let serialized = serde_json::to_string(&event);
        assert!(serialized.is_ok());

        let deserialized: Result<PumpSwapBuyEvent, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_event_max_values() {
        let mut event = PumpSwapBuyEvent::default();
        event.base_amount_out = u64::MAX;
        event.max_quote_amount_in = u64::MAX;
        event.user_base_token_reserves = u64::MAX;
        event.user_quote_token_reserves = u64::MAX;

        assert_eq!(event.base_amount_out, u64::MAX);
        assert_eq!(event.max_quote_amount_in, u64::MAX);
        assert_eq!(event.user_base_token_reserves, u64::MAX);
        assert_eq!(event.user_quote_token_reserves, u64::MAX);
    }

    #[test]
    fn test_event_negative_timestamp() {
        let mut event = PumpSwapBuyEvent::default();
        event.timestamp = -1000;
        assert_eq!(event.timestamp, -1000);
    }

    #[test]
    fn test_discriminators_uniqueness() {
        // Ensure all event discriminators are unique
        let discriminators = vec![
            discriminators::BUY_EVENT_BYTES,
            discriminators::SELL_EVENT_BYTES,
            discriminators::CREATE_POOL_EVENT_BYTES,
            discriminators::DEPOSIT_EVENT_BYTES,
            discriminators::WITHDRAW_EVENT_BYTES,
        ];

        for (i, disc1) in discriminators.iter().enumerate() {
            for (j, disc2) in discriminators.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        disc1, disc2,
                        "Discriminators at indices {} and {} are the same",
                        i, j
                    );
                }
            }
        }
    }

    #[test]
    fn test_instruction_discriminators_uniqueness() {
        // Ensure all instruction discriminators are unique
        let instructions = vec![
            discriminators::BUY_IX,
            discriminators::SELL_IX,
            discriminators::CREATE_POOL_IX,
            discriminators::DEPOSIT_IX,
            discriminators::WITHDRAW_IX,
        ];

        for (i, ix1) in instructions.iter().enumerate() {
            for (j, ix2) in instructions.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        ix1, ix2,
                        "Instruction discriminators at indices {} and {} are the same",
                        i, j
                    );
                }
            }
        }
    }
}
