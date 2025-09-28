/// Pumpswap protocol event definitions and constants.
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::time::SystemTime;

use crate::error::Error as ParseError;
use crate::solana_metadata::SolanaEventMetadata;
use core::any::Any;

// ================================
// DISCRIMINATORS
// ================================

/// Event discriminator constants
pub mod discriminators {
    /// String identifier for `PumpSwap` buy events
    pub const BUY_EVENT: &str = "pumpswap_buy_event";
    /// String identifier for `PumpSwap` sell events
    pub const SELL_EVENT: &str = "pumpswap_sell_event";
    /// String identifier for `PumpSwap` create pool events
    pub const CREATE_POOL_EVENT: &str = "pumpswap_create_pool_event";
    /// String identifier for `PumpSwap` deposit events
    pub const DEPOSIT_EVENT: &str = "pumpswap_deposit_event";
    /// String identifier for `PumpSwap` withdraw events
    pub const WITHDRAW_EVENT: &str = "pumpswap_withdraw_event";

    /// Byte array discriminator for `PumpSwap` buy events
    pub const BUY_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x67, 0xf4, 0x52, 0x1f, 0x2c, 0xf5, 0x77,
        0x77,
    ];
    /// Byte array discriminator for `PumpSwap` sell events
    pub const SELL_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x3e, 0x2f, 0x37, 0x0a, 0xa5, 0x03, 0xdc,
        0x2a,
    ];
    /// Byte array discriminator for `PumpSwap` create pool events
    pub const CREATE_POOL_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0xb1, 0x31, 0x0c, 0xd2, 0xa0, 0x76, 0xa7,
        0x74,
    ];
    /// Byte array discriminator for `PumpSwap` deposit events
    pub const DEPOSIT_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x78, 0xf8, 0x3d, 0x53, 0x1f, 0x8e, 0x6b,
        0x90,
    ];
    /// Byte array discriminator for `PumpSwap` withdraw events
    pub const WITHDRAW_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x16, 0x09, 0x85, 0x1a, 0xa0, 0x2c, 0x47,
        0xc0,
    ];

    /// Instruction discriminator for `PumpSwap` buy operations
    pub const BUY_IX: &[u8] = &[102, 6, 61, 18, 1, 218, 235, 234];
    /// Instruction discriminator for `PumpSwap` sell operations
    pub const SELL_IX: &[u8] = &[51, 230, 133, 164, 1, 127, 131, 173];
    /// Instruction discriminator for `PumpSwap` create pool operations
    pub const CREATE_POOL_IX: &[u8] = &[233, 146, 209, 142, 207, 104, 64, 188];
    /// Instruction discriminator for `PumpSwap` deposit operations
    pub const DEPOSIT_IX: &[u8] = &[242, 35, 198, 137, 82, 225, 242, 182];
    /// Instruction discriminator for `PumpSwap` withdraw operations
    pub const WITHDRAW_IX: &[u8] = &[183, 18, 70, 156, 148, 109, 161, 34];
}

use riglr_events_core::{
    error::{EventError, EventResult},
    traits::EventFilter,
    Event, EventKind, EventMetadata as CoreEventMetadata,
};

// ================================
// EVENT DEFINITIONS
// ================================

/// Buy event
#[non_exhaustive]
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize, Default,
)]
pub struct PumpSwapBuyEvent {
    /// Amount of base tokens received from the swap
    pub base_amount_out: u64,
    /// Base token mint public key (excluded from serialization)
    #[serde(skip)]
    pub base_mint: Pubkey,
    /// Coin creator public key
    pub coin_creator: Pubkey,
    /// Coin creator fee amount
    pub coin_creator_fee: u64,
    /// Coin creator fee in basis points
    pub coin_creator_fee_basis_points: u64,
    /// Coin creator vault ATA public key (excluded from serialization)
    #[serde(skip)]
    pub coin_creator_vault_ata: Pubkey,
    /// Coin creator vault authority public key (excluded from serialization)
    #[serde(skip)]
    pub coin_creator_vault_authority: Pubkey,
    /// Current SOL volume
    pub current_sol_volume: u64,
    /// Last update timestamp
    pub last_update_timestamp: i64,
    /// Liquidity provider fee amount
    pub lp_fee: u64,
    /// Liquidity provider fee in basis points
    pub lp_fee_basis_points: u64,
    /// Maximum amount of quote tokens willing to spend
    pub max_quote_amount_in: u64,
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: SolanaEventMetadata,
    /// Pool public key
    pub pool: Pubkey,
    /// Pool's base token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    /// Pool's base token reserves after the swap
    pub pool_base_token_reserves: u64,
    /// Pool's quote token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
    /// Pool's quote token reserves after the swap
    pub pool_quote_token_reserves: u64,
    /// Protocol fee amount
    pub protocol_fee: u64,
    /// Protocol fee in basis points
    pub protocol_fee_basis_points: u64,
    /// Protocol fee recipient public key
    pub protocol_fee_recipient: Pubkey,
    /// Protocol fee recipient token account public key
    pub protocol_fee_recipient_token_account: Pubkey,
    /// Actual amount of quote tokens spent in the swap
    pub quote_amount_in: u64,
    /// Quote amount including LP fees
    pub quote_amount_in_with_lp_fee: u64,
    /// Quote token mint public key (excluded from serialization)
    #[serde(skip)]
    pub quote_mint: Pubkey,
    /// Block timestamp when the event occurred
    pub timestamp: i64,
    /// Total claimed tokens
    pub total_claimed_tokens: u64,
    /// Total unclaimed tokens
    pub total_unclaimed_tokens: u64,
    /// Whether to track volume for this swap
    pub track_volume: bool,
    /// User wallet public key
    pub user: Pubkey,
    /// User's base token account public key
    pub user_base_token_account: Pubkey,
    /// User's base token balance after the swap
    pub user_base_token_reserves: u64,
    /// User's quote amount input
    pub user_quote_amount_in: u64,
    /// User's quote token account public key
    pub user_quote_token_account: Pubkey,
    /// User's quote token balance after the swap
    pub user_quote_token_reserves: u64,
}

impl Event for PumpSwapBuyEvent {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    #[inline]
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    #[inline]
    fn kind(&self) -> &EventKind {
        static SWAP_KIND: EventKind = EventKind::Swap;
        &SWAP_KIND
    }

    #[inline]
    fn matches_filter(&self, filter: &dyn EventFilter) -> bool {
        filter.matches(self)
    }

    #[inline]
    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    #[inline]
    fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
    }

    #[inline]
    fn source(&self) -> &str {
        &self.metadata().source
    }

    #[inline]
    fn timestamp(&self) -> SystemTime {
        self.metadata().timestamp.into()
    }

    #[inline]
    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(EventError::Serialization)
    }
}

/// Sell event
#[non_exhaustive]
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize, Default,
)]
pub struct PumpSwapSellEvent {
    /// Amount of base tokens being sold
    pub base_amount_in: u64,
    /// Base token mint public key (excluded from serialization)
    #[serde(skip)]
    pub base_mint: Pubkey,
    /// Coin creator public key
    pub coin_creator: Pubkey,
    /// Coin creator fee amount
    pub coin_creator_fee: u64,
    /// Coin creator fee in basis points
    pub coin_creator_fee_basis_points: u64,
    /// Coin creator vault ATA public key (excluded from serialization)
    #[serde(skip)]
    pub coin_creator_vault_ata: Pubkey,
    /// Coin creator vault authority public key (excluded from serialization)
    #[serde(skip)]
    pub coin_creator_vault_authority: Pubkey,
    /// Liquidity provider fee amount
    pub lp_fee: u64,
    /// Liquidity provider fee in basis points
    pub lp_fee_basis_points: u64,
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: SolanaEventMetadata,
    /// Minimum amount of quote tokens expected to receive
    pub min_quote_amount_out: u64,
    /// Pool public key
    pub pool: Pubkey,
    /// Pool's base token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    /// Pool's base token reserves after the swap
    pub pool_base_token_reserves: u64,
    /// Pool's quote token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
    /// Pool's quote token reserves after the swap
    pub pool_quote_token_reserves: u64,
    /// Protocol fee amount
    pub protocol_fee: u64,
    /// Protocol fee in basis points
    pub protocol_fee_basis_points: u64,
    /// Protocol fee recipient public key
    pub protocol_fee_recipient: Pubkey,
    /// Protocol fee recipient token account public key
    pub protocol_fee_recipient_token_account: Pubkey,
    /// Actual amount of quote tokens received from the swap
    pub quote_amount_out: u64,
    /// Quote amount without LP fees
    pub quote_amount_out_without_lp_fee: u64,
    /// Quote token mint public key (excluded from serialization)
    #[serde(skip)]
    pub quote_mint: Pubkey,
    /// Block timestamp when the event occurred
    pub timestamp: i64,
    /// User wallet public key
    pub user: Pubkey,
    /// User's base token account public key
    pub user_base_token_account: Pubkey,
    /// User's base token balance after the swap
    pub user_base_token_reserves: u64,
    /// User's quote amount output
    pub user_quote_amount_out: u64,
    /// User's quote token account public key
    pub user_quote_token_account: Pubkey,
    /// User's quote token balance after the swap
    pub user_quote_token_reserves: u64,
}

impl Event for PumpSwapSellEvent {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    #[inline]
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    #[inline]
    fn kind(&self) -> &EventKind {
        static SWAP_KIND: EventKind = EventKind::Swap;
        &SWAP_KIND
    }

    #[inline]
    fn matches_filter(&self, filter: &dyn EventFilter) -> bool {
        filter.matches(self)
    }

    #[inline]
    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    #[inline]
    fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
    }

    #[inline]
    fn source(&self) -> &str {
        &self.metadata().source
    }

    #[inline]
    fn timestamp(&self) -> SystemTime {
        self.metadata().timestamp.into()
    }

    #[inline]
    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(EventError::Serialization)
    }
}

/// Create pool event
#[non_exhaustive]
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize, Default,
)]
pub struct PumpSwapCreatePoolEvent {
    /// Amount of base tokens deposited
    pub base_amount_in: u64,
    /// Base token mint public key
    pub base_mint: Pubkey,
    /// Base token decimal places
    pub base_mint_decimals: u8,
    /// Coin creator public key
    pub coin_creator: Pubkey,
    /// Pool creator public key
    pub creator: Pubkey,
    /// Pool index identifier
    pub index: u16,
    /// Initial liquidity provided
    pub initial_liquidity: u64,
    /// LP token mint public key
    pub lp_mint: Pubkey,
    /// LP tokens minted
    pub lp_token_amount_out: u64,
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: SolanaEventMetadata,
    /// Minimum liquidity required
    pub minimum_liquidity: u64,
    /// Pool public key
    pub pool: Pubkey,
    /// Pool's base token amount
    pub pool_base_amount: u64,
    /// Pool's base token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    /// Pool bump seed
    pub pool_bump: u8,
    /// Pool's quote token amount
    pub pool_quote_amount: u64,
    /// Pool's quote token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
    /// Amount of quote tokens deposited
    pub quote_amount_in: u64,
    /// Quote token mint public key
    pub quote_mint: Pubkey,
    /// Quote token decimal places
    pub quote_mint_decimals: u8,
    /// Block timestamp when the event occurred
    pub timestamp: i64,
    /// User's base token account public key
    pub user_base_token_account: Pubkey,
    /// User's pool token account public key (excluded from serialization)
    #[serde(skip)]
    pub user_pool_token_account: Pubkey,
    /// User's quote token account public key
    pub user_quote_token_account: Pubkey,
}

impl Event for PumpSwapCreatePoolEvent {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    #[inline]
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    #[inline]
    fn kind(&self) -> &EventKind {
        static CONTRACT_KIND: EventKind = EventKind::Contract;
        &CONTRACT_KIND
    }

    #[inline]
    fn matches_filter(&self, filter: &dyn EventFilter) -> bool {
        filter.matches(self)
    }

    #[inline]
    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    #[inline]
    fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
    }

    #[inline]
    fn source(&self) -> &str {
        &self.metadata().source
    }

    #[inline]
    fn timestamp(&self) -> SystemTime {
        self.metadata().timestamp.into()
    }

    #[inline]
    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(EventError::Serialization)
    }
}

/// Deposit event
#[non_exhaustive]
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize, Default,
)]
pub struct PumpSwapDepositEvent {
    /// Actual base token amount deposited
    pub base_amount_in: u64,
    /// Base token mint public key (excluded from serialization)
    #[serde(skip)]
    pub base_mint: Pubkey,
    /// Total LP token supply after deposit
    pub lp_mint_supply: u64,
    /// Amount of LP tokens minted
    pub lp_token_amount_out: u64,
    /// Maximum base token amount willing to deposit
    pub max_base_amount_in: u64,
    /// Maximum quote token amount willing to deposit
    pub max_quote_amount_in: u64,
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: SolanaEventMetadata,
    /// Pool public key
    pub pool: Pubkey,
    /// Pool's base token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    /// Pool's base token reserves after deposit
    pub pool_base_token_reserves: u64,
    /// Pool's quote token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
    /// Pool's quote token reserves after deposit
    pub pool_quote_token_reserves: u64,
    /// Actual quote token amount deposited
    pub quote_amount_in: u64,
    /// Quote token mint public key (excluded from serialization)
    #[serde(skip)]
    pub quote_mint: Pubkey,
    /// Block timestamp when the event occurred
    pub timestamp: i64,
    /// User wallet public key
    pub user: Pubkey,
    /// User's base token account public key
    pub user_base_token_account: Pubkey,
    /// User's base token balance after deposit
    pub user_base_token_reserves: u64,
    /// User's pool token account public key
    pub user_pool_token_account: Pubkey,
    /// User's quote token account public key
    pub user_quote_token_account: Pubkey,
    /// User's quote token balance after deposit
    pub user_quote_token_reserves: u64,
}

impl Event for PumpSwapDepositEvent {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    #[inline]
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    #[inline]
    fn kind(&self) -> &EventKind {
        static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
        &LIQUIDITY_KIND
    }

    #[inline]
    fn matches_filter(&self, filter: &dyn EventFilter) -> bool {
        filter.matches(self)
    }

    #[inline]
    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    #[inline]
    fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
    }

    #[inline]
    fn source(&self) -> &str {
        &self.metadata().source
    }

    #[inline]
    fn timestamp(&self) -> SystemTime {
        self.metadata().timestamp.into()
    }

    #[inline]
    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(EventError::Serialization)
    }
}

/// Withdraw event
#[non_exhaustive]
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize, Default,
)]
pub struct PumpSwapWithdrawEvent {
    /// Actual base token amount withdrawn
    pub base_amount_out: u64,
    /// Base token mint public key (excluded from serialization)
    #[serde(skip)]
    pub base_mint: Pubkey,
    /// Total LP token supply after withdrawal
    pub lp_mint_supply: u64,
    /// Amount of LP tokens burned
    pub lp_token_amount_in: u64,
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: SolanaEventMetadata,
    /// Minimum base token amount expected to receive
    pub min_base_amount_out: u64,
    /// Minimum quote token amount expected to receive
    pub min_quote_amount_out: u64,
    /// Pool public key
    pub pool: Pubkey,
    /// Pool's base token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    /// Pool's base token reserves after withdrawal
    pub pool_base_token_reserves: u64,
    /// Pool's quote token account public key (excluded from serialization)
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
    /// Pool's quote token reserves after withdrawal
    pub pool_quote_token_reserves: u64,
    /// Actual quote token amount withdrawn
    pub quote_amount_out: u64,
    /// Quote token mint public key (excluded from serialization)
    #[serde(skip)]
    pub quote_mint: Pubkey,
    /// Block timestamp when the event occurred
    pub timestamp: i64,
    /// User wallet public key
    pub user: Pubkey,
    /// User's base token account public key
    pub user_base_token_account: Pubkey,
    /// User's base token balance after withdrawal
    pub user_base_token_reserves: u64,
    /// User's pool token account public key
    pub user_pool_token_account: Pubkey,
    /// User's quote token account public key
    pub user_quote_token_account: Pubkey,
    /// User's quote token balance after withdrawal
    pub user_quote_token_reserves: u64,
}

impl Event for PumpSwapWithdrawEvent {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    #[inline]
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    #[inline]
    fn kind(&self) -> &EventKind {
        static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
        &LIQUIDITY_KIND
    }

    #[inline]
    fn matches_filter(&self, filter: &dyn EventFilter) -> bool {
        filter.matches(self)
    }

    #[inline]
    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    #[inline]
    fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
    }

    #[inline]
    fn source(&self) -> &str {
        &self.metadata().source
    }

    #[inline]
    fn timestamp(&self) -> SystemTime {
        self.metadata().timestamp.into()
    }

    #[inline]
    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(EventError::Serialization)
    }
}

// ================================
// PARSER DEFINITIONS
// ================================

use crate::error::ParseResult;
use crate::events::{
    common::read_u64_le,
    factory::{InnerInstructionParseParams, InstructionParseParams, SolanaTransactionInput},
    parser_types::{GenericEventParseConfig, GenericEventParser, ProtocolParser},
};
use crate::types::{EventType, ProtocolType};
use riglr_events_core::traits::{EventParser, ParserInfo};
use solana_message::compiled_instruction::CompiledInstruction;
use std::collections::HashMap;

/// `PumpSwap` program ID
pub const PUMPSWAP_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");

/// `PumpSwap` event parser
#[derive(Debug)]
pub struct PumpSwapEventParser {
    info: ParserInfo,
    inner: GenericEventParser,
}

impl Default for PumpSwapEventParser {
    fn default() -> Self {
        let configs = vec![
            GenericEventParseConfig {
                program_id: PUMPSWAP_PROGRAM_ID,
                protocol_type: ProtocolType::PumpSwap,
                inner_instruction_discriminator: discriminators::BUY_EVENT,
                instruction_discriminator: discriminators::BUY_IX,
                event_type: EventType::PumpSwapBuy,
                inner_instruction_parser: Self::parse_buy_inner_instruction,
                instruction_parser: Self::parse_buy_instruction,
            },
            GenericEventParseConfig {
                program_id: PUMPSWAP_PROGRAM_ID,
                protocol_type: ProtocolType::PumpSwap,
                inner_instruction_discriminator: discriminators::SELL_EVENT,
                instruction_discriminator: discriminators::SELL_IX,
                event_type: EventType::PumpSwapSell,
                inner_instruction_parser: Self::parse_sell_inner_instruction,
                instruction_parser: Self::parse_sell_instruction,
            },
            GenericEventParseConfig {
                program_id: PUMPSWAP_PROGRAM_ID,
                protocol_type: ProtocolType::PumpSwap,
                inner_instruction_discriminator: discriminators::CREATE_POOL_EVENT,
                instruction_discriminator: discriminators::CREATE_POOL_IX,
                event_type: EventType::PumpSwapCreatePool,
                inner_instruction_parser: Self::parse_create_pool_inner_instruction,
                instruction_parser: Self::parse_create_pool_instruction,
            },
            GenericEventParseConfig {
                program_id: PUMPSWAP_PROGRAM_ID,
                protocol_type: ProtocolType::PumpSwap,
                inner_instruction_discriminator: discriminators::DEPOSIT_EVENT,
                instruction_discriminator: discriminators::DEPOSIT_IX,
                event_type: EventType::PumpSwapDeposit,
                inner_instruction_parser: Self::parse_deposit_inner_instruction,
                instruction_parser: Self::parse_deposit_instruction,
            },
            GenericEventParseConfig {
                program_id: PUMPSWAP_PROGRAM_ID,
                protocol_type: ProtocolType::PumpSwap,
                inner_instruction_discriminator: discriminators::WITHDRAW_EVENT,
                instruction_discriminator: discriminators::WITHDRAW_IX,
                event_type: EventType::PumpSwapWithdraw,
                inner_instruction_parser: Self::parse_withdraw_inner_instruction,
                instruction_parser: Self::parse_withdraw_instruction,
            },
        ];

        let inner = GenericEventParser::new(vec![PUMPSWAP_PROGRAM_ID], configs);
        let info = ParserInfo::new("pumpswap_parser".to_owned(), "1.0.0".to_owned())
            .with_kind(riglr_events_core::EventKind::Custom(
                "pumpswap_buy".to_owned(),
            ))
            .with_kind(riglr_events_core::EventKind::Custom(
                "pumpswap_sell".to_owned(),
            ))
            .with_kind(riglr_events_core::EventKind::Custom(
                "pumpswap_create_pool".to_owned(),
            ))
            .with_format("solana_instruction".to_owned());

        Self { info, inner }
    }
}

impl PumpSwapEventParser {
    /// Creates a new `PumpSwap` event parser with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse buy log event as static method
    fn parse_buy_inner_instruction(
        data: &'_ [u8],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        let mut event = PumpSwapBuyEvent::try_from_slice(data).map_err(|_| {
            ParseError::InvalidDataFormat("Failed to parse PumpSwap buy event".to_owned())
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-buy", metadata.signature, event.pool));
        event.metadata = metadata;
        Ok(Box::new(event))
    }

    /// Parse create pool log event as static method
    fn parse_create_pool_inner_instruction(
        data: &'_ [u8],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        let mut event = PumpSwapCreatePoolEvent::try_from_slice(data).map_err(|_| {
            ParseError::InvalidDataFormat("Failed to parse PumpSwap create pool event".to_owned())
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-create", metadata.signature, event.pool));
        event.metadata = metadata;
        Ok(Box::new(event))
    }

    /// Parse deposit log event as static method
    fn parse_deposit_inner_instruction(
        data: &'_ [u8],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        let mut event = PumpSwapDepositEvent::try_from_slice(data).map_err(|_| {
            ParseError::InvalidDataFormat("Failed to parse PumpSwap deposit event".to_owned())
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-deposit", metadata.signature, event.pool));
        event.metadata = metadata;
        Ok(Box::new(event))
    }

    /// Parse sell log event as static method
    fn parse_sell_inner_instruction(
        data: &'_ [u8],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        let mut event = PumpSwapSellEvent::try_from_slice(data).map_err(|_| {
            ParseError::InvalidDataFormat("Failed to parse PumpSwap sell event".to_owned())
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-sell", metadata.signature, event.pool));
        event.metadata = metadata;
        Ok(Box::new(event))
    }

    /// Parse buy instruction event as static method
    fn parse_buy_instruction(
        data: &'_ [u8],
        accounts: &'_ [Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        use crate::events::common::{
            get_account_or_default, parse_swap_amounts, safe_get_account, validate_account_count,
            validate_data_length,
        };

        validate_data_length(data, 16, "PumpSwap buy instruction")?;
        validate_account_count(accounts, 11, "PumpSwap buy instruction")?;

        let (base_amount_out, max_quote_amount_in) = parse_swap_amounts(data)?;

        let pool = safe_get_account(accounts, 0)?;
        let user = safe_get_account(accounts, 1)?;
        let base_mint = safe_get_account(accounts, 3)?;
        let quote_mint = safe_get_account(accounts, 4)?;
        let user_base_token_account = safe_get_account(accounts, 5)?;
        let user_quote_token_account = safe_get_account(accounts, 6)?;
        let pool_base_token_account = safe_get_account(accounts, 7)?;
        let pool_quote_token_account = safe_get_account(accounts, 8)?;

        let protocol_fee_recipient = get_account_or_default(accounts, 9);
        let protocol_fee_recipient_token_account = get_account_or_default(accounts, 10);
        let coin_creator_vault_ata = Pubkey::default();
        let coin_creator_vault_authority = Pubkey::default();

        let event = PumpSwapBuyEvent {
            metadata,
            base_amount_out,
            max_quote_amount_in,
            quote_amount_in_with_lp_fee: 0,
            user_quote_amount_in: 0,
            pool,
            user,
            user_base_token_account,
            user_quote_token_account,
            protocol_fee_recipient,
            protocol_fee_recipient_token_account,
            coin_creator: Pubkey::default(),
            coin_creator_fee_basis_points: 0,
            coin_creator_fee: 0,
            track_volume: false,
            total_unclaimed_tokens: 0,
            total_claimed_tokens: 0,
            current_sol_volume: 0,
            last_update_timestamp: 0,
            base_mint,
            quote_mint,
            pool_base_token_account,
            pool_quote_token_account,
            coin_creator_vault_ata,
            coin_creator_vault_authority,
            lp_fee: 0,
            lp_fee_basis_points: 0,
            pool_base_token_reserves: 0,
            pool_quote_token_reserves: 0,
            protocol_fee: 0,
            protocol_fee_basis_points: 0,
            quote_amount_in: 0,
            timestamp: 0,
            user_base_token_reserves: 0,
            user_quote_token_reserves: 0,
        };
        Ok(Box::new(event))
    }

    /// Parse withdraw log event as static method
    fn parse_withdraw_inner_instruction(
        data: &'_ [u8],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        let mut event = PumpSwapWithdrawEvent::try_from_slice(data).map_err(|_| {
            ParseError::InvalidDataFormat("Failed to parse PumpSwap withdraw event".to_owned())
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-withdraw", metadata.signature, event.pool));
        event.metadata = metadata;
        Ok(Box::new(event))
    }

    /// Parse create pool instruction event as static method
    fn parse_create_pool_instruction(
        data: &'_ [u8],
        accounts: &'_ [Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        use crate::events::common::{
            read_pubkey, read_u16_le, read_u64_le, safe_get_account, validate_account_count,
            validate_data_length,
        };

        validate_data_length(data, 18, "PumpSwap create pool instruction")?;
        validate_account_count(accounts, 11, "PumpSwap create pool instruction")?;

        let index = read_u16_le(data, 0)?;
        let base_amount_in = read_u64_le(data, 2)?;
        let quote_amount_in = read_u64_le(data, 10)?;
        let coin_creator = if data.len() >= 50 {
            read_pubkey(data, 18)?
        } else {
            Pubkey::default()
        };

        let pool = safe_get_account(accounts, 0)?;
        let creator = safe_get_account(accounts, 2)?;
        let base_mint = safe_get_account(accounts, 3)?;
        let quote_mint = safe_get_account(accounts, 4)?;
        let lp_mint = safe_get_account(accounts, 5)?;
        let user_base_token_account = safe_get_account(accounts, 6)?;
        let user_quote_token_account = safe_get_account(accounts, 7)?;
        let user_pool_token_account = safe_get_account(accounts, 8)?;
        let pool_base_token_account = safe_get_account(accounts, 9)?;
        let pool_quote_token_account = safe_get_account(accounts, 10)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, pool, creator, base_amount_in
        ));

        let event = PumpSwapCreatePoolEvent {
            metadata,
            timestamp: 0,
            index,
            creator,
            base_mint,
            quote_mint,
            base_mint_decimals: 0,
            quote_mint_decimals: 0,
            base_amount_in,
            quote_amount_in,
            pool_base_amount: 0,
            pool_quote_amount: 0,
            minimum_liquidity: 0,
            initial_liquidity: 0,
            lp_token_amount_out: 0,
            pool_bump: 0,
            pool,
            lp_mint,
            user_base_token_account,
            user_quote_token_account,
            coin_creator,
            user_pool_token_account,
            pool_base_token_account,
            pool_quote_token_account,
        };
        Ok(Box::new(event))
    }

    /// Parse sell instruction event as static method
    fn parse_sell_instruction(
        data: &'_ [u8],
        accounts: &'_ [Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 16 || accounts.len() < 11 {
            return Err(ParseError::InvalidDataFormat(
                "Insufficient data or accounts for PumpSwap sell instruction".to_owned(),
            ));
        }

        let base_amount_in = read_u64_le(data, 0).map_err(|_| {
            ParseError::InvalidDataFormat("Failed to read base_amount_in".to_owned())
        })?;
        let min_quote_amount_out = read_u64_le(data, 8).map_err(|_| {
            ParseError::InvalidDataFormat("Failed to read min_quote_amount_out".to_owned())
        })?;

        let mut metadata = metadata;
        let user_for_id = accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing user account for ID generation".to_string())
        })?;
        let pool_for_id = accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool account for ID generation".to_string())
        })?;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, user_for_id, pool_for_id, base_amount_in
        ));

        let pool = *accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool account at index 0".to_string())
        })?;
        let user = *accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing user account at index 1".to_string())
        })?;
        let base_mint = *accounts.get(3).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing base_mint account at index 3".to_string())
        })?;
        let quote_mint = *accounts.get(4).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing quote_mint account at index 4".to_string())
        })?;
        let user_base_token_account = *accounts.get(5).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing user_base_token_account at index 5".to_string())
        })?;
        let user_quote_token_account = *accounts.get(6).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing user_quote_token_account at index 6".to_string())
        })?;
        let pool_base_token_account = *accounts.get(7).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool_base_token_account at index 7".to_string())
        })?;
        let pool_quote_token_account = *accounts.get(8).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool_quote_token_account at index 8".to_string())
        })?;
        let protocol_fee_recipient = *accounts.get(9).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing protocol_fee_recipient at index 9".to_string())
        })?;
        let protocol_fee_recipient_token_account = *accounts.get(10).ok_or_else(|| {
            ParseError::InvalidDataFormat(
                "Missing protocol_fee_recipient_token_account at index 10".to_string(),
            )
        })?;

        let event = PumpSwapSellEvent {
            metadata,
            timestamp: 0,
            base_amount_in,
            min_quote_amount_out,
            user_base_token_reserves: 0,
            user_quote_token_reserves: 0,
            pool_base_token_reserves: 0,
            pool_quote_token_reserves: 0,
            quote_amount_out: 0,
            lp_fee_basis_points: 0,
            lp_fee: 0,
            protocol_fee_basis_points: 0,
            protocol_fee: 0,
            quote_amount_out_without_lp_fee: 0,
            user_quote_amount_out: 0,
            pool,
            user,
            user_base_token_account,
            user_quote_token_account,
            protocol_fee_recipient,
            protocol_fee_recipient_token_account,
            coin_creator: Pubkey::default(),
            coin_creator_fee_basis_points: 0,
            coin_creator_fee: 0,
            base_mint,
            quote_mint,
            pool_base_token_account,
            pool_quote_token_account,
            coin_creator_vault_ata: accounts.get(17).copied().unwrap_or_default(),
            coin_creator_vault_authority: accounts.get(18).copied().unwrap_or_default(),
        };
        Ok(Box::new(event))
    }

    /// Parse deposit instruction event as static method
    #[expect(clippy::too_many_lines)]
    fn parse_deposit_instruction(
        data: &'_ [u8],
        accounts: &'_ [Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 24 || accounts.len() < 11 {
            return Err(ParseError::InvalidDataFormat(
                "Insufficient data or accounts for PumpSwap deposit instruction".to_owned(),
            ));
        }

        let lp_token_amount_out = u64::from_le_bytes(
            data.get(0..8)
                .ok_or_else(|| {
                    ParseError::InvalidDataFormat(
                        "Insufficient data for lp_token_amount_out".to_string(),
                    )
                })?
                .try_into()
                .map_err(|_| {
                    ParseError::InvalidDataFormat("Failed to read lp_token_amount_out".to_owned())
                })?,
        );
        let max_base_amount_in = u64::from_le_bytes(
            data.get(8..16)
                .ok_or_else(|| {
                    ParseError::InvalidDataFormat(
                        "Insufficient data for max_base_amount_in".to_string(),
                    )
                })?
                .try_into()
                .map_err(|_| {
                    ParseError::InvalidDataFormat("Failed to read max_base_amount_in".to_owned())
                })?,
        );
        let max_quote_amount_in = u64::from_le_bytes(
            data.get(16..24)
                .ok_or_else(|| {
                    ParseError::InvalidDataFormat(
                        "Insufficient data for max_quote_amount_in".to_string(),
                    )
                })?
                .try_into()
                .map_err(|_| {
                    ParseError::InvalidDataFormat("Failed to read max_quote_amount_in".to_owned())
                })?,
        );

        let mut metadata = metadata;
        let pool_for_id = accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool account for ID generation".to_string())
        })?;
        let user_for_id = accounts.get(2).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing user account for ID generation".to_string())
        })?;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, pool_for_id, user_for_id, lp_token_amount_out
        ));

        let pool = *accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool account at index 0".to_string())
        })?;
        let user = *accounts.get(2).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing user account at index 2".to_string())
        })?;
        let base_mint = *accounts.get(3).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing base_mint account at index 3".to_string())
        })?;
        let quote_mint = *accounts.get(4).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing quote_mint account at index 4".to_string())
        })?;
        let user_base_token_account = *accounts.get(6).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing user_base_token_account at index 6".to_string())
        })?;
        let user_quote_token_account = *accounts.get(7).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing user_quote_token_account at index 7".to_string())
        })?;
        let user_pool_token_account = *accounts.get(8).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing user_pool_token_account at index 8".to_string())
        })?;
        let pool_base_token_account = *accounts.get(9).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool_base_token_account at index 9".to_string())
        })?;
        let pool_quote_token_account = *accounts.get(10).ok_or_else(|| {
            ParseError::InvalidDataFormat(
                "Missing pool_quote_token_account at index 10".to_string(),
            )
        })?;

        let event = PumpSwapDepositEvent {
            metadata,
            timestamp: 0,
            lp_token_amount_out,
            max_base_amount_in,
            max_quote_amount_in,
            user_base_token_reserves: 0,
            user_quote_token_reserves: 0,
            pool_base_token_reserves: 0,
            pool_quote_token_reserves: 0,
            base_amount_in: 0,
            quote_amount_in: 0,
            lp_mint_supply: 0,
            pool,
            user,
            user_base_token_account,
            user_quote_token_account,
            user_pool_token_account,
            base_mint,
            quote_mint,
            pool_base_token_account,
            pool_quote_token_account,
        };
        Ok(Box::new(event))
    }

    /// Parse withdraw instruction event as static method
    #[expect(clippy::too_many_lines)]
    fn parse_withdraw_instruction(
        data: &'_ [u8],
        accounts: &'_ [Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 24 || accounts.len() < 11 {
            return Err(ParseError::InvalidDataFormat(
                "Insufficient data or accounts for PumpSwap withdraw instruction".to_owned(),
            ));
        }

        let lp_token_amount_in = u64::from_le_bytes(
            data.get(0..8)
                .ok_or_else(|| {
                    ParseError::InvalidDataFormat(
                        "Insufficient data for lp_token_amount_in".to_string(),
                    )
                })?
                .try_into()
                .map_err(|_| {
                    ParseError::InvalidDataFormat("Failed to read lp_token_amount_in".to_owned())
                })?,
        );
        let min_base_amount_out = u64::from_le_bytes(
            data.get(8..16)
                .ok_or_else(|| {
                    ParseError::InvalidDataFormat(
                        "Insufficient data for min_base_amount_out".to_string(),
                    )
                })?
                .try_into()
                .map_err(|_| {
                    ParseError::InvalidDataFormat("Failed to read min_base_amount_out".to_owned())
                })?,
        );
        let min_quote_amount_out = u64::from_le_bytes(
            data.get(16..24)
                .ok_or_else(|| {
                    ParseError::InvalidDataFormat(
                        "Insufficient data for min_quote_amount_out".to_string(),
                    )
                })?
                .try_into()
                .map_err(|_| {
                    ParseError::InvalidDataFormat("Failed to read min_quote_amount_out".to_owned())
                })?,
        );

        let mut metadata = metadata;
        let pool_for_id = accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool account for ID generation".to_string())
        })?;
        let user_for_id = accounts.get(2).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing user account for ID generation".to_string())
        })?;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, pool_for_id, user_for_id, lp_token_amount_in
        ));

        let pool = *accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool account at index 0".to_string())
        })?;
        let user = *accounts.get(2).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing user account at index 2".to_string())
        })?;
        let base_mint = *accounts.get(3).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing base_mint account at index 3".to_string())
        })?;
        let quote_mint = *accounts.get(4).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing quote_mint account at index 4".to_string())
        })?;
        let user_base_token_account = *accounts.get(6).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing user_base_token_account at index 6".to_string())
        })?;
        let user_quote_token_account = *accounts.get(7).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing user_quote_token_account at index 7".to_string())
        })?;
        let user_pool_token_account = *accounts.get(8).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing user_pool_token_account at index 8".to_string())
        })?;
        let pool_base_token_account = *accounts.get(9).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool_base_token_account at index 9".to_string())
        })?;
        let pool_quote_token_account = *accounts.get(10).ok_or_else(|| {
            ParseError::InvalidDataFormat(
                "Missing pool_quote_token_account at index 10".to_string(),
            )
        })?;

        let event = PumpSwapWithdrawEvent {
            metadata,
            timestamp: 0,
            lp_token_amount_in,
            min_base_amount_out,
            min_quote_amount_out,
            user_base_token_reserves: 0,
            user_quote_token_reserves: 0,
            pool_base_token_reserves: 0,
            pool_quote_token_reserves: 0,
            base_amount_out: 0,
            quote_amount_out: 0,
            lp_mint_supply: 0,
            pool,
            user,
            user_base_token_account,
            user_quote_token_account,
            user_pool_token_account,
            base_mint,
            quote_mint,
            pool_base_token_account,
            pool_quote_token_account,
        };
        Ok(Box::new(event))
    }
}

impl ProtocolParser for PumpSwapEventParser {
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner.inner_instruction_configs()
    }
    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.inner.instruction_configs()
    }
    fn parse_events_from_inner_instruction(
        &self,
        params: &InnerInstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        self.inner.parse_events_from_inner_instruction(params)
    }
    fn parse_events_from_instruction(
        &self,
        params: &InstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        self.inner.parse_events_from_instruction(params)
    }
    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.inner.should_handle(program_id)
    }
    fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.inner.supported_program_ids()
    }
}

#[async_trait::async_trait]
impl EventParser for PumpSwapEventParser {
    type Input = SolanaTransactionInput;
    fn can_parse(&self, input: &Self::Input) -> bool {
        match *input {
            SolanaTransactionInput::InnerInstruction(_)
            | SolanaTransactionInput::Instruction(_) => true,
        }
    }
    fn info(&self) -> &ParserInfo {
        &self.info
    }

    async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>> {
        let events = match input {
            SolanaTransactionInput::InnerInstruction(params) => {
                let legacy_params = InnerInstructionParseParams {
                    inner_instruction: &solana_transaction_status::UiCompiledInstruction {
                        program_id_index: 0,
                        accounts: vec![],
                        data: params.inner_instruction_data.clone(),
                        stack_height: Some(1),
                    },
                    signature: &params.signature,
                    slot: params.slot,
                    block_time: params.block_time,
                    program_received_time_ms: params.program_received_time_ms,
                    index: params.index.clone(),
                };
                self.inner
                    .parse_events_from_inner_instruction(&legacy_params)
            }
            SolanaTransactionInput::Instruction(params) => {
                let instruction = CompiledInstruction {
                    program_id_index: 0,
                    accounts: vec![],
                    data: params.instruction_data.clone(),
                };
                let legacy_params = InstructionParseParams {
                    instruction: &instruction,
                    accounts: &params.accounts,
                    signature: &params.signature,
                    slot: params.slot,
                    block_time: params.block_time,
                    program_received_time_ms: params.program_received_time_ms,
                    index: params.index.clone(),
                };
                self.inner.parse_events_from_instruction(&legacy_params)
            }
        };
        Ok(events)
    }
}

// ================================
// TESTS (consolidated from both files)
// ================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use core::str::FromStr as _;
    use riglr_events_core::{EventKind, EventMetadata as CoreEventMetadata};

    #[test]
    fn module_exports_events() {
        let _buy_event = PumpSwapBuyEvent::default();
        let _sell_event = PumpSwapSellEvent::default();
        let _create_pool_event = PumpSwapCreatePoolEvent::default();
        let _deposit_event = PumpSwapDepositEvent::default();
        let _withdraw_event = PumpSwapWithdrawEvent::default();
    }

    #[test]
    fn module_exports_parser() {
        let _parser = PumpSwapEventParser::new();
        let _default_parser = PumpSwapEventParser::default();
    }

    #[test]
    fn module_exports_discriminators() {
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
    fn module_exports_discriminator_bytes() {
        assert_eq!(discriminators::BUY_EVENT_BYTES.len(), 16);
        assert_eq!(discriminators::SELL_EVENT_BYTES.len(), 16);
        assert_eq!(discriminators::CREATE_POOL_EVENT_BYTES.len(), 16);
        assert_eq!(discriminators::DEPOSIT_EVENT_BYTES.len(), 16);
        assert_eq!(discriminators::WITHDRAW_EVENT_BYTES.len(), 16);
    }

    #[test]
    fn module_exports_instruction_discriminators() {
        assert_eq!(discriminators::BUY_IX.len(), 8);
        assert_eq!(discriminators::SELL_IX.len(), 8);
        assert_eq!(discriminators::CREATE_POOL_IX.len(), 8);
        assert_eq!(discriminators::DEPOSIT_IX.len(), 8);
        assert_eq!(discriminators::WITHDRAW_IX.len(), 8);
    }

    #[test]
    fn module_exports_program_id() {
        let program_id = PUMPSWAP_PROGRAM_ID;
        assert_ne!(program_id, Pubkey::default());

        // Test constant is safe
        let expected = Pubkey::from_str("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA").unwrap();
        assert_eq!(program_id, expected);
    }

    #[test]
    fn buy_event_implements_event_trait() {
        let mut event = PumpSwapBuyEvent::default();

        let _id = event.id();
        let _kind = event.kind();
        let _metadata = event.metadata();
        let _metadata_mut = event.metadata_mut();
        let _any = event.as_any();
        let _any_mut = event.as_any_mut();
        let _cloned = event.clone_boxed();

        let _json_result = event.to_json();
    }

    #[test]
    fn sell_event_implements_event_trait() {
        let mut event = PumpSwapSellEvent::default();

        let _id = event.id();
        let _kind = event.kind();
        let _metadata = event.metadata();
        let _metadata_mut = event.metadata_mut();
        let _any = event.as_any();
        let _any_mut = event.as_any_mut();
        let _cloned = event.clone_boxed();

        let _json_result = event.to_json();
    }

    #[test]
    fn create_pool_event_implements_event_trait() {
        let mut event = PumpSwapCreatePoolEvent::default();

        let _id = event.id();
        let _kind = event.kind();
        let _metadata = event.metadata();
        let _metadata_mut = event.metadata_mut();
        let _any = event.as_any();
        let _any_mut = event.as_any_mut();
        let _cloned = event.clone_boxed();

        let _json_result = event.to_json();
    }

    #[test]
    fn deposit_event_implements_event_trait() {
        let mut event = PumpSwapDepositEvent::default();

        let _id = event.id();
        let _kind = event.kind();
        let _metadata = event.metadata();
        let _metadata_mut = event.metadata_mut();
        let _any = event.as_any();
        let _any_mut = event.as_any_mut();
        let _cloned = event.clone_boxed();

        let _json_result = event.to_json();
    }

    #[test]
    fn withdraw_event_implements_event_trait() {
        let mut event = PumpSwapWithdrawEvent::default();

        let _id = event.id();
        let _kind = event.kind();
        let _metadata = event.metadata();
        let _metadata_mut = event.metadata_mut();
        let _any = event.as_any();
        let _any_mut = event.as_any_mut();
        let _cloned = event.clone_boxed();

        let _json_result = event.to_json();
    }

    #[test]
    fn parser_default_vs_new() {
        let parser1 = PumpSwapEventParser::default();
        let parser2 = PumpSwapEventParser::new();

        assert_eq!(
            parser1.supported_program_ids(),
            parser2.supported_program_ids()
        );

        assert!(parser1.should_handle(&PUMPSWAP_PROGRAM_ID));
        assert!(parser2.should_handle(&PUMPSWAP_PROGRAM_ID));

        let random_pubkey = Pubkey::new_unique();
        assert!(!parser1.should_handle(&random_pubkey));
        assert!(!parser2.should_handle(&random_pubkey));
    }

    #[test]
    fn event_structs_clone_and_debug() {
        let buy_event = PumpSwapBuyEvent::default();
        let _cloned_buy = buy_event.clone();
        let _debug_buy = format!("{buy_event:?}");

        let sell_event = PumpSwapSellEvent::default();
        let _cloned_sell = sell_event.clone();
        let _debug_sell = format!("{sell_event:?}");

        let create_pool_event = PumpSwapCreatePoolEvent::default();
        let _cloned_create_pool = create_pool_event.clone();
        let _debug_create_pool = format!("{create_pool_event:?}");

        let deposit_event = PumpSwapDepositEvent::default();
        let _cloned_deposit = deposit_event.clone();
        let _debug_deposit = format!("{deposit_event:?}");

        let withdraw_event = PumpSwapWithdrawEvent::default();
        let _cloned_withdraw = withdraw_event.clone();
        let _debug_withdraw = format!("{withdraw_event:?}");
    }

    #[test]
    fn event_structs_partial_eq() {
        // Create events with the same metadata to avoid timestamp differences
        let buy_event1 = PumpSwapBuyEvent::default();
        let buy_event2 = PumpSwapBuyEvent {
            metadata: buy_event1.metadata.clone(),
            ..Default::default()
        };
        assert_eq!(buy_event1, buy_event2);

        let sell_event1 = PumpSwapSellEvent::default();
        let sell_event2 = PumpSwapSellEvent {
            metadata: sell_event1.metadata.clone(),
            ..Default::default()
        };
        assert_eq!(sell_event1, sell_event2);

        let create_pool_event1 = PumpSwapCreatePoolEvent::default();
        let create_pool_event2 = PumpSwapCreatePoolEvent {
            metadata: create_pool_event1.metadata.clone(),
            ..Default::default()
        };
        assert_eq!(create_pool_event1, create_pool_event2);

        let deposit_event1 = PumpSwapDepositEvent::default();
        let deposit_event2 = PumpSwapDepositEvent {
            metadata: deposit_event1.metadata.clone(),
            ..Default::default()
        };
        assert_eq!(deposit_event1, deposit_event2);

        let withdraw_event1 = PumpSwapWithdrawEvent::default();
        let withdraw_event2 = PumpSwapWithdrawEvent {
            metadata: withdraw_event1.metadata.clone(),
            ..Default::default()
        };
        assert_eq!(withdraw_event1, withdraw_event2);
    }

    #[test]
    fn discriminators_are_unique() {
        let discriminators = [
            discriminators::BUY_EVENT,
            discriminators::SELL_EVENT,
            discriminators::CREATE_POOL_EVENT,
            discriminators::DEPOSIT_EVENT,
            discriminators::WITHDRAW_EVENT,
        ];

        for (i, disc1) in discriminators.iter().enumerate() {
            for (j, disc2) in discriminators.iter().enumerate() {
                if i != j {
                    assert_ne!(disc1, disc2, "Discriminators should be unique");
                }
            }
        }
    }

    #[test]
    fn byte_discriminators_are_unique() {
        let byte_discriminators = [
            discriminators::BUY_EVENT_BYTES,
            discriminators::SELL_EVENT_BYTES,
            discriminators::CREATE_POOL_EVENT_BYTES,
            discriminators::DEPOSIT_EVENT_BYTES,
            discriminators::WITHDRAW_EVENT_BYTES,
        ];

        for (i, disc1) in byte_discriminators.iter().enumerate() {
            for (j, disc2) in byte_discriminators.iter().enumerate() {
                if i != j {
                    assert_ne!(disc1, disc2, "Byte discriminators should be unique");
                }
            }
        }
    }

    #[test]
    fn instruction_discriminators_are_unique() {
        let instruction_discriminators = [
            discriminators::BUY_IX,
            discriminators::SELL_IX,
            discriminators::CREATE_POOL_IX,
            discriminators::DEPOSIT_IX,
            discriminators::WITHDRAW_IX,
        ];

        for (i, disc1) in instruction_discriminators.iter().enumerate() {
            for (j, disc2) in instruction_discriminators.iter().enumerate() {
                if i != j {
                    assert_ne!(disc1, disc2, "Instruction discriminators should be unique");
                }
            }
        }
    }

    #[test]
    fn parser_configs_accessible() {
        let parser = PumpSwapEventParser::new();

        let inner_configs = parser.inner_instruction_configs();
        assert!(
            !inner_configs.is_empty(),
            "Should have inner instruction configs"
        );

        let instruction_configs = parser.instruction_configs();
        assert!(
            !instruction_configs.is_empty(),
            "Should have instruction configs"
        );
    }

    #[test]
    fn supported_program_ids_contains_pumpswap() {
        let parser = PumpSwapEventParser::new();
        let supported = parser.supported_program_ids();

        assert!(
            supported.contains(&PUMPSWAP_PROGRAM_ID),
            "Should support PUMPSWAP_PROGRAM_ID"
        );
        assert!(
            !supported.is_empty(),
            "Should have at least one supported program ID"
        );
    }

    // Helper function to create test metadata
    fn create_test_metadata() -> CoreEventMetadata {
        use chrono::DateTime;
        use riglr_events_core::types::ChainData;

        // Test helper - expected to work
        let timestamp = DateTime::from_timestamp(1_234_567_890, 0)
            .expect("Failed to create DateTime from timestamp");
        let chain_data = ChainData::Solana {
            slot: 100,
            signature: Some("test_sig".to_string()),
            program_id: Some(Pubkey::default()),
            instruction_index: Some(0),
            block_time: Some(1_234_567_890),
            protocol_data: None,
        };

        let mut core_metadata = CoreEventMetadata::new(
            "test_id".to_string(),
            EventKind::Swap,
            "solana-test".to_string(),
        );
        core_metadata.timestamp = timestamp;
        core_metadata.received_at = timestamp;
        core_metadata.chain_data = Some(chain_data);
        core_metadata
    }

    fn create_test_pubkey() -> Pubkey {
        Pubkey::new_from_array([1; 32])
    }

    #[test]
    fn pumpswap_buy_event_default() {
        let event = PumpSwapBuyEvent::default();
        assert_eq!(event.timestamp, 0);
        assert_eq!(event.base_amount_out, 0);
        assert_eq!(event.max_quote_amount_in, 0);
        assert!(!event.track_volume);
        assert_eq!(event.pool, Pubkey::default());
        assert_eq!(event.user, Pubkey::default());
    }

    #[test]
    fn pumpswap_buy_event_with_values() {
        let mut event = PumpSwapBuyEvent::default();
        event.metadata.core = create_test_metadata();
        event.timestamp = 1_234_567_890;
        event.base_amount_out = 1000;
        event.max_quote_amount_in = 2000;
        event.track_volume = true;
        event.pool = create_test_pubkey();
        event.user = create_test_pubkey();

        assert_eq!(event.timestamp, 1_234_567_890);
        assert_eq!(event.base_amount_out, 1000);
        assert_eq!(event.max_quote_amount_in, 2000);
        assert!(event.track_volume);
        assert_eq!(event.pool, create_test_pubkey());
        assert_eq!(event.user, create_test_pubkey());
    }

    #[test]
    fn pumpswap_buy_event_event_trait_id() {
        let mut event = PumpSwapBuyEvent::default();
        event.metadata.core = create_test_metadata();
        assert_eq!(event.id(), "test_id");
    }

    #[test]
    fn pumpswap_buy_event_event_trait_kind() {
        let mut event = PumpSwapBuyEvent::default();
        event.metadata.core = create_test_metadata();
        assert_eq!(event.kind(), &EventKind::Swap);
    }

    #[test]
    fn pumpswap_buy_event_event_trait_metadata() {
        let mut event = PumpSwapBuyEvent::default();
        event.metadata.core = create_test_metadata();
        let metadata = event.metadata();
        assert_eq!(metadata.id, "test_id");
        assert_eq!(metadata.kind, EventKind::Swap);
    }

    #[test]
    fn pumpswap_buy_event_event_trait_metadata_mut() {
        let mut event = PumpSwapBuyEvent::default();
        event.metadata.core = create_test_metadata();
        // Test helper - expected to work
        let metadata_mut = event
            .metadata_mut()
            .expect("Failed to get mutable metadata reference");
        metadata_mut.id = "new_id".to_string();
        assert_eq!(event.metadata.core.id, "new_id");
    }

    #[test]
    fn pumpswap_buy_event_event_trait_as_any() {
        let event = PumpSwapBuyEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<PumpSwapBuyEvent>().is_some());
    }

    #[test]
    fn pumpswap_buy_event_event_trait_as_any_mut() {
        let mut event = PumpSwapBuyEvent::default();
        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<PumpSwapBuyEvent>().is_some());
    }

    #[test]
    fn pumpswap_buy_event_event_trait_clone_boxed() {
        let event = PumpSwapBuyEvent::default();
        let cloned = event.clone_boxed();
        assert!(cloned.as_any().downcast_ref::<PumpSwapBuyEvent>().is_some());
    }

    #[test]
    fn pumpswap_buy_event_event_trait_to_json() {
        let event = PumpSwapBuyEvent::default();
        let json_result = event.to_json();
        assert!(json_result.is_ok());
        // Test helper - expected to work
        let json_value = json_result.expect("Failed to convert event to JSON");
        assert!(json_value.is_object());
    }

    #[test]
    fn pumpswap_program_id_constant() {
        // Test constant is safe
        let expected = Pubkey::from_str("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA")
            .expect("Valid PumpSwap program ID constant");
        assert_eq!(PUMPSWAP_PROGRAM_ID, expected);
    }

    #[test]
    fn parser_default() {
        let parser = PumpSwapEventParser::default();
        assert!(parser.should_handle(&PUMPSWAP_PROGRAM_ID));
        assert_eq!(parser.supported_program_ids(), vec![PUMPSWAP_PROGRAM_ID]);
    }

    #[test]
    fn parser_should_handle_valid_program_id() {
        let parser = PumpSwapEventParser::default();
        assert!(parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    }

    #[test]
    fn parser_should_not_handle_invalid_program_id() {
        let parser = PumpSwapEventParser::default();
        let random_id = Pubkey::new_unique();
        assert!(!parser.should_handle(&random_id));
    }

    #[test]
    fn parser_inner_instruction_configs() {
        let parser = PumpSwapEventParser::default();
        let configs = parser.inner_instruction_configs();
        assert!(!configs.is_empty());
    }

    #[test]
    fn parser_instruction_configs() {
        let parser = PumpSwapEventParser::default();
        let configs = parser.instruction_configs();
        assert!(!configs.is_empty());
    }
}
