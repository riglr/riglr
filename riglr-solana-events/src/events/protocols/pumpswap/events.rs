use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

// UnifiedEvent trait removed - events now implement Event trait directly

// Import new Event trait from riglr-events-core
use riglr_events_core::{Event, EventKind, EventMetadata};
use std::any::Any;

/// Buy event
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
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

/// Sell event
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
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

/// Deposit event
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
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

/// Withdraw event
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
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
