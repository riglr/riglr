//! BONK protocol event definitions and parsing functionality.
//!
//! This module provides comprehensive support for parsing and handling BONK protocol events,
//! including trade events, pool creation events, and related transaction parsing.

// Known clippy false positive on enum definitions - see https://github.com/rust-lang/rust-clippy/issues/7477
#![allow(clippy::pattern_type_mismatch)]

use crate::{
    error::{Error as ParseError, ParseResult},
    events::{
        common::{
            parse_swap_amounts, safe_get_account, validate_account_count, validate_data_length,
        },
        factory::{InnerInstructionParseParams, InstructionParseParams, SolanaTransactionInput},
        parser_types::{GenericEventParseConfig, GenericEventParser, ProtocolParser},
    },
    solana_metadata::SolanaEventMetadata,
    types::{EventType, ProtocolType},
};
use borsh::{BorshDeserialize, BorshSerialize};
use core::any::Any;
use riglr_events_core::{
    error::EventResult,
    traits::{EventParser as CoreEventParser, ParserInfo},
    Event,
};
use serde::{Deserialize, Serialize};
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use std::collections::HashMap;

// ================================================================================================
// Discriminator constants
// ================================================================================================

/// Discriminator constants for BONK events and instructions
pub mod discriminators {
    /// String discriminator for trade events
    pub const TRADE_EVENT: &str = "bonk_trade_event";

    /// String discriminator for pool create events
    pub const POOL_CREATE_EVENT: &str = "bonk_pool_create_event";

    /// Byte discriminator for trade events (16 bytes)
    pub const TRADE_EVENT_BYTES: [u8; 16] = [
        0x42, 0x6F, 0x6E, 0x6B, 0x54, 0x72, 0x61, 0x64, 0x65, 0x45, 0x76, 0x65, 0x6E, 0x74, 0x00,
        0x00,
    ];

    /// Byte discriminator for pool create events (16 bytes)
    pub const POOL_CREATE_EVENT_BYTES: [u8; 16] = [
        0x42, 0x6F, 0x6E, 0x6B, 0x50, 0x6F, 0x6F, 0x6C, 0x43, 0x72, 0x65, 0x61, 0x74, 0x65, 0x00,
        0x00,
    ];

    /// Instruction discriminator for buy exact in
    pub const BUY_EXACT_IN_IX: [u8; 8] = [0x42, 0x75, 0x79, 0x49, 0x6E, 0x00, 0x00, 0x00];

    /// Instruction discriminator for buy exact out
    pub const BUY_EXACT_OUT_IX: [u8; 8] = [0x42, 0x75, 0x79, 0x4F, 0x75, 0x74, 0x00, 0x00];

    /// Instruction discriminator for sell exact in
    pub const SELL_EXACT_IN_IX: [u8; 8] = [0x53, 0x65, 0x6C, 0x6C, 0x49, 0x6E, 0x00, 0x00];

    /// Instruction discriminator for sell exact out
    pub const SELL_EXACT_OUT_IX: [u8; 8] = [0x53, 0x65, 0x6C, 0x6C, 0x4F, 0x75, 0x74, 0x00];

    /// Instruction discriminator for initialize
    pub const INITIALIZE_IX: [u8; 8] = [0x49, 0x6E, 0x69, 0x74, 0x00, 0x00, 0x00, 0x00];

    /// Instruction discriminator for migrate to AMM
    pub const MIGRATE_TO_AMM_IX: [u8; 8] = [0x4D, 0x69, 0x67, 0x41, 0x4D, 0x4D, 0x00, 0x00];

    /// Instruction discriminator for migrate to `CPSwap`
    pub const MIGRATE_TO_CPSWAP_IX: [u8; 8] = [0x4D, 0x69, 0x67, 0x43, 0x50, 0x00, 0x00, 0x00];
}

/// Bonk program ID
pub const BONK_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("bonksoHKfNJJ8Wo8ZJjpw7dHGePNxS2z2WE5GxUPdSo");

// ================================================================================================
// BONK Protocol Data Types and Structures
// ================================================================================================

/// Direction of a trade operation in the BONK protocol
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    BorshDeserialize,
    BorshSerialize,
)]
pub enum TradeDirection {
    /// Buy operation - purchasing tokens
    #[default]
    Buy,
    /// Sell operation - selling tokens
    Sell,
}

/// Current status of a liquidity pool in the BONK protocol
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    BorshDeserialize,
    BorshSerialize,
)]
pub enum PoolStatus {
    /// Initial funding phase where liquidity is being raised
    #[default]
    Fund,
    /// Migration phase where pool is transitioning to DEX
    Migrate,
    /// Active trading phase where tokens can be traded
    Trade,
}

/// Parameters for minting new tokens in the BONK protocol
#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize,
)]
pub struct MintParams {
    /// Number of decimal places for the token
    pub decimals: u8,
    /// Human-readable name of the token
    pub name: String,
    /// Short symbol identifier for the token
    pub symbol: String,
    /// URI pointing to token metadata
    pub uri: String,
}

/// Parameters for token vesting schedules in the BONK protocol
#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize,
)]
pub struct VestingParams {
    /// Duration before tokens begin to unlock
    pub cliff_period: u64,
    /// Total amount of tokens locked in vesting
    pub total_locked_amount: u64,
    /// Duration over which tokens are gradually unlocked
    pub unlock_period: u64,
}

/// Constant bonding curve parameters
#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize,
)]
pub struct ConstantCurve {
    /// Type of migration when curve is complete
    pub migrate_type: u8,
    /// Total supply of tokens on the curve
    pub supply: u64,
    /// Total base tokens available for selling
    pub total_base_sell: u64,
    /// Total quote tokens needed for fundraising
    pub total_quote_fund_raising: u64,
}

/// Fixed bonding curve parameters
#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize,
)]
pub struct FixedCurve {
    /// Type of migration when curve is complete
    pub migrate_type: u8,
    /// Total supply of tokens on the curve
    pub supply: u64,
    /// Total quote tokens needed for fundraising
    pub total_quote_fund_raising: u64,
}

/// Linear bonding curve parameters
#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize,
)]
pub struct LinearCurve {
    /// Type of migration when curve is complete
    pub migrate_type: u8,
    /// Total supply of tokens on the curve
    pub supply: u64,
    /// Total quote tokens needed for fundraising
    pub total_quote_fund_raising: u64,
}

/// Combined bonding curve parameters supporting different curve types
#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize,
)]
pub struct CurveParams {
    /// Optional constant curve configuration
    pub constant_curve: Option<ConstantCurve>,
    /// Optional fixed curve configuration
    pub fixed_curve: Option<FixedCurve>,
    /// Optional linear curve configuration
    pub linear_curve: Option<LinearCurve>,
}

// ================================================================================================
// BONK Protocol Event Types and Structures
// ================================================================================================

/// Trade event
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Default,
)]
pub struct TradeEvent {
    /// Amount of tokens being traded in
    pub amount_in: u64,
    /// Amount of tokens being traded out
    pub amount_out: u64,
    /// Base token mint address
    pub base_mint: Pubkey,
    /// Event metadata
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: SolanaEventMetadata,
    /// Fee charged by the platform
    pub platform_fee: u64,
    /// Pool's base token account
    pub pool_base_account: Pubkey,
    /// Pool's quote token account
    pub pool_quote_account: Pubkey,
    /// Public key of the pool state account
    pub pool_state: Pubkey,
    /// Fee charged by the protocol
    pub protocol_fee: u64,
    /// Quote token mint address
    pub quote_mint: Pubkey,
    /// Real base token reserves after the trade
    pub real_base_after: u64,
    /// Real base token reserves before the trade
    pub real_base_before: u64,
    /// Real quote token reserves after the trade
    pub real_quote_after: u64,
    /// Real quote token reserves before the trade
    pub real_quote_before: u64,
    /// Fee shared with stakeholders
    pub share_fee: u64,
    /// Signer authority for the trade
    pub signer: Pubkey,
    /// Timestamp when the trade occurred
    pub timestamp: u64,
    /// Total amount of base tokens available for sale
    pub total_base_sell: u64,
    /// Direction of the trade (buy/sell)
    pub trade_direction: TradeDirection,
    /// User's base token account
    pub user_base_account: Pubkey,
    /// User's quote token account
    pub user_quote_account: Pubkey,
    /// Virtual base token reserves used for price calculations
    pub virtual_base: u64,
    /// Virtual quote token reserves used for price calculations
    pub virtual_quote: u64,
}

impl Event for TradeEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
    fn id(&self) -> &str {
        &self.metadata.core.id
    }
    fn kind(&self) -> &riglr_events_core::EventKind {
        &self.metadata.core.kind
    }
    fn metadata(&self) -> &riglr_events_core::EventMetadata {
        &self.metadata.core
    }
    fn metadata_mut(
        &mut self,
    ) -> riglr_events_core::EventResult<&mut riglr_events_core::EventMetadata> {
        Ok(&mut self.metadata.core)
    }
    fn to_json(&self) -> riglr_events_core::EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(riglr_events_core::EventError::Serialization)
    }
}

/// Pool creation event
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Default,
)]
pub struct PoolCreateEvent {
    /// Base token mint address
    pub base_mint: Pubkey,
    /// Pool creator's public key
    pub creator: Pubkey,
    /// Bonding curve parameters
    pub curve_params: CurveParams,
    /// Fee recipient account
    pub fee_recipient: Pubkey,
    /// Event metadata
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: SolanaEventMetadata,
    /// Parameters for token minting
    pub mint_params: MintParams,
    /// Public key of the pool state account
    pub pool_state: Pubkey,
    /// Initial pool status
    pub pool_status: PoolStatus,
    /// Quote token mint address
    pub quote_mint: Pubkey,
    /// Timestamp when pool was created
    pub timestamp: u64,
    /// Total supply of base tokens
    pub total_base_supply: u64,
    /// Parameters for token vesting
    pub vesting_params: VestingParams,
}

// ================================================================================================
// Event implementations
// ================================================================================================

impl Event for PoolCreateEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
    fn id(&self) -> &str {
        &self.metadata.core.id
    }
    fn kind(&self) -> &riglr_events_core::EventKind {
        &self.metadata.core.kind
    }
    fn metadata(&self) -> &riglr_events_core::EventMetadata {
        &self.metadata.core
    }
    fn metadata_mut(
        &mut self,
    ) -> riglr_events_core::EventResult<&mut riglr_events_core::EventMetadata> {
        Ok(&mut self.metadata.core)
    }
    fn to_json(&self) -> riglr_events_core::EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(riglr_events_core::EventError::Serialization)
    }
}

// ================================================================================================
// BONK Protocol Transaction and Log Parsing Functionality
// ================================================================================================

/// Type alias for Solana event metadata
type EventMetadata = SolanaEventMetadata;

/// Bonk event parser
#[derive(Debug)]
pub struct EventParser {
    /// Parser information
    info: ParserInfo,
    /// Inner generic event parser that handles the core parsing logic
    inner: GenericEventParser,
}

impl Default for EventParser {
    #[inline]
    fn default() -> Self {
        // Configure all event types
        let configs = vec![
            GenericEventParseConfig {
                program_id: BONK_PROGRAM_ID,
                protocol_type: ProtocolType::Bonk,
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: &discriminators::BUY_EXACT_IN_IX,
                event_type: EventType::BonkBuyExactIn,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_buy_exact_in_instruction,
            },
            GenericEventParseConfig {
                program_id: BONK_PROGRAM_ID,
                protocol_type: ProtocolType::Bonk,
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: &discriminators::BUY_EXACT_OUT_IX,
                event_type: EventType::BonkBuyExactOut,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_buy_exact_out_instruction,
            },
            GenericEventParseConfig {
                program_id: BONK_PROGRAM_ID,
                protocol_type: ProtocolType::Bonk,
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: &discriminators::SELL_EXACT_IN_IX,
                event_type: EventType::BonkSellExactIn,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_sell_exact_in_instruction,
            },
            GenericEventParseConfig {
                program_id: BONK_PROGRAM_ID,
                protocol_type: ProtocolType::Bonk,
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: &discriminators::SELL_EXACT_OUT_IX,
                event_type: EventType::BonkSellExactOut,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_sell_exact_out_instruction,
            },
            GenericEventParseConfig {
                program_id: BONK_PROGRAM_ID,
                protocol_type: ProtocolType::Bonk,
                inner_instruction_discriminator: discriminators::POOL_CREATE_EVENT,
                instruction_discriminator: &discriminators::INITIALIZE_IX,
                event_type: EventType::BonkInitialize,
                inner_instruction_parser: Self::parse_pool_create_inner_instruction,
                instruction_parser: Self::parse_initialize_instruction,
            },
            GenericEventParseConfig {
                program_id: BONK_PROGRAM_ID,
                protocol_type: ProtocolType::Bonk,
                inner_instruction_discriminator: discriminators::POOL_CREATE_EVENT,
                instruction_discriminator: &discriminators::MIGRATE_TO_AMM_IX,
                event_type: EventType::BonkMigrateToAmm,
                inner_instruction_parser: Self::parse_pool_create_inner_instruction,
                instruction_parser: Self::parse_migrate_to_amm_instruction,
            },
            GenericEventParseConfig {
                program_id: BONK_PROGRAM_ID,
                protocol_type: ProtocolType::Bonk,
                inner_instruction_discriminator: discriminators::POOL_CREATE_EVENT,
                instruction_discriminator: &discriminators::MIGRATE_TO_CPSWAP_IX,
                event_type: EventType::BonkMigrateToCpswap,
                inner_instruction_parser: Self::parse_pool_create_inner_instruction,
                instruction_parser: Self::parse_migrate_to_cpswap_instruction,
            },
        ];

        let inner = GenericEventParser::new(vec![BONK_PROGRAM_ID], configs);
        let info = ParserInfo::new("bonk_parser".to_owned(), "1.0.0".to_owned())
            .with_kind(riglr_events_core::EventKind::Custom(
                "bonk_trade".to_owned(),
            ))
            .with_kind(riglr_events_core::EventKind::Custom(
                "bonk_initialize".to_owned(),
            ))
            .with_kind(riglr_events_core::EventKind::Custom(
                "bonk_migrate".to_owned(),
            ))
            .with_format("solana_instruction".to_owned());
        Self { info, inner }
    }
}

impl EventParser {
    /// Creates a new BONK event parser
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse buy exact in instruction
    fn parse_buy_exact_in_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        match validate_data_length(data, 16, "Bonk buy exact in instruction") {
            Ok(()) => {}
            Err(error) => return Err(error),
        }
        match validate_account_count(accounts, 10, "Bonk buy exact in instruction") {
            Ok(()) => {}
            Err(error) => return Err(error),
        }

        let (amount_in, minimum_amount_out) = parse_swap_amounts(data)?;

        let pool_state = safe_get_account(accounts, 0)?;
        let payer = safe_get_account(accounts, 1)?;
        let user_base_token = safe_get_account(accounts, 2)?;
        let user_quote_token = safe_get_account(accounts, 3)?;
        let base_vault = safe_get_account(accounts, 4)?;
        let quote_vault = safe_get_account(accounts, 5)?;
        let base_token_mint = safe_get_account(accounts, 6)?;
        let quote_token_mint = safe_get_account(accounts, 7)?;

        let mut local_metadata = metadata;
        local_metadata.core.id = format!(
            "{}-{}-{}-{}",
            local_metadata.signature, pool_state, amount_in, minimum_amount_out
        );

        Ok(Box::new(TradeEvent {
            metadata: local_metadata,
            pool_state,
            amount_in,
            amount_out: minimum_amount_out,
            trade_direction: TradeDirection::Buy,
            timestamp: 0,
            signer: payer,
            user_base_account: user_base_token,
            user_quote_account: user_quote_token,
            pool_base_account: base_vault,
            pool_quote_account: quote_vault,
            base_mint: base_token_mint,
            quote_mint: quote_token_mint,
            ..Default::default()
        }))
    }

    /// Parse buy exact out instruction
    fn parse_buy_exact_out_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        match validate_data_length(data, 16, "Bonk buy exact out instruction") {
            Ok(()) => {}
            Err(error) => return Err(error),
        }
        match validate_account_count(accounts, 10, "Bonk buy exact out instruction") {
            Ok(()) => {}
            Err(error) => return Err(error),
        }

        let (maximum_amount_in, amount_out) = parse_swap_amounts(data)?;

        let pool_state = safe_get_account(accounts, 0)?;
        let payer = safe_get_account(accounts, 1)?;
        let user_base_token = safe_get_account(accounts, 2)?;
        let user_quote_token = safe_get_account(accounts, 3)?;
        let base_vault = safe_get_account(accounts, 4)?;
        let quote_vault = safe_get_account(accounts, 5)?;
        let base_token_mint = safe_get_account(accounts, 6)?;
        let quote_token_mint = safe_get_account(accounts, 7)?;

        let local_metadata = metadata;

        Ok(Box::new(TradeEvent {
            metadata: local_metadata,
            pool_state,
            amount_in: maximum_amount_in,
            amount_out,
            trade_direction: TradeDirection::Buy,
            timestamp: 0,
            signer: payer,
            user_base_account: user_base_token,
            user_quote_account: user_quote_token,
            pool_base_account: base_vault,
            pool_quote_account: quote_vault,
            base_mint: base_token_mint,
            quote_mint: quote_token_mint,
            ..Default::default()
        }))
    }

    /// Parse initialize instruction
    fn parse_initialize_instruction(
        _data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        match validate_account_count(accounts, 8, "Bonk initialize instruction") {
            Ok(()) => {}
            Err(error) => return Err(error),
        }

        let pool_state = safe_get_account(accounts, 0)?;
        let creator = safe_get_account(accounts, 1)?;
        let base_mint = safe_get_account(accounts, 3)?;
        let quote_mint = safe_get_account(accounts, 4)?;

        let local_metadata = metadata;

        Ok(Box::new(PoolCreateEvent {
            metadata: local_metadata,
            pool_state,
            creator,
            base_mint,
            quote_mint,
            ..Default::default()
        }))
    }

    /// Parse migrate to AMM instruction
    fn parse_migrate_to_amm_instruction(
        _data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        match validate_account_count(accounts, 5, "migrate to AMM instruction") {
            Ok(()) => {}
            Err(error) => return Err(error),
        }

        let mut local_metadata = metadata;
        local_metadata.core.id = format!(
            "{}-{}",
            local_metadata.signature,
            safe_get_account(accounts, 0)?
        );

        let pool_state = safe_get_account(accounts, 0)?;
        let creator = safe_get_account(accounts, 1)?;
        let base_mint = safe_get_account(accounts, 3)?;
        let quote_mint = safe_get_account(accounts, 4)?;

        Ok(Box::new(PoolCreateEvent {
            metadata: local_metadata,
            pool_state,
            creator,
            base_mint,
            quote_mint,
            ..Default::default()
        }))
    }

    /// Parse migrate to CPSWAP instruction
    fn parse_migrate_to_cpswap_instruction(
        _data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        match validate_account_count(accounts, 5, "migrate to CPSWAP instruction") {
            Ok(()) => {}
            Err(error) => return Err(error),
        }

        let mut local_metadata = metadata;
        local_metadata.core.id = format!(
            "{}-{}",
            local_metadata.signature,
            safe_get_account(accounts, 0)?
        );

        let pool_state = safe_get_account(accounts, 0)?;
        let creator = safe_get_account(accounts, 1)?;
        let base_mint = safe_get_account(accounts, 3)?;
        let quote_mint = safe_get_account(accounts, 4)?;

        Ok(Box::new(PoolCreateEvent {
            metadata: local_metadata,
            pool_state,
            creator,
            base_mint,
            quote_mint,
            ..Default::default()
        }))
    }

    /// Parse pool create inner instruction data
    fn parse_pool_create_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        let event = match PoolCreateEvent::try_from_slice(data) {
            Ok(parsed_event) => parsed_event,
            Err(error) => return Err(ParseError::BorshError(error.to_string())),
        };

        let local_metadata = metadata;

        Ok(Box::new(PoolCreateEvent {
            metadata: local_metadata,
            ..event
        }))
    }

    /// Parse sell exact in instruction
    fn parse_sell_exact_in_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        match validate_data_length(data, 16, "sell exact in instruction") {
            Ok(()) => {}
            Err(error) => return Err(error),
        }
        match validate_account_count(accounts, 10, "sell exact in instruction") {
            Ok(()) => {}
            Err(error) => return Err(error),
        }

        let (amount_in, minimum_amount_out) = parse_swap_amounts(data)?;

        let mut local_metadata = metadata;
        local_metadata.core.id = format!(
            "{}-{}-{}-{}",
            local_metadata.signature,
            safe_get_account(accounts, 0)?,
            amount_in,
            minimum_amount_out
        );

        let pool_state = safe_get_account(accounts, 0)?;
        let signer = safe_get_account(accounts, 1)?;
        let user_base_account = safe_get_account(accounts, 2)?;
        let user_quote_account = safe_get_account(accounts, 3)?;
        let pool_base_account = safe_get_account(accounts, 4)?;
        let pool_quote_account = safe_get_account(accounts, 5)?;
        let base_mint = safe_get_account(accounts, 6)?;
        let quote_mint = safe_get_account(accounts, 7)?;

        Ok(Box::new(TradeEvent {
            metadata: local_metadata,
            pool_state,
            amount_in,
            amount_out: minimum_amount_out,
            trade_direction: TradeDirection::Sell,
            timestamp: 0,
            signer,
            user_base_account,
            user_quote_account,
            pool_base_account,
            pool_quote_account,
            base_mint,
            quote_mint,
            ..Default::default()
        }))
    }

    /// Parse sell exact out instruction
    fn parse_sell_exact_out_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        match validate_data_length(data, 16, "sell exact out instruction") {
            Ok(()) => {}
            Err(error) => return Err(error),
        }
        match validate_account_count(accounts, 10, "sell exact out instruction") {
            Ok(()) => {}
            Err(error) => return Err(error),
        }

        let (maximum_amount_in, amount_out) = parse_swap_amounts(data)?;

        let mut local_metadata = metadata;
        local_metadata.core.id = format!(
            "{}-{}-{}-{}",
            local_metadata.signature,
            safe_get_account(accounts, 0)?,
            maximum_amount_in,
            amount_out
        );

        let pool_state = safe_get_account(accounts, 0)?;
        let signer = safe_get_account(accounts, 1)?;
        let user_base_account = safe_get_account(accounts, 2)?;
        let user_quote_account = safe_get_account(accounts, 3)?;
        let pool_base_account = safe_get_account(accounts, 4)?;
        let pool_quote_account = safe_get_account(accounts, 5)?;
        let base_mint = safe_get_account(accounts, 6)?;
        let quote_mint = safe_get_account(accounts, 7)?;

        Ok(Box::new(TradeEvent {
            metadata: local_metadata,
            pool_state,
            amount_in: maximum_amount_in,
            amount_out,
            trade_direction: TradeDirection::Sell,
            timestamp: 0,
            signer,
            user_base_account,
            user_quote_account,
            pool_base_account,
            pool_quote_account,
            base_mint,
            quote_mint,
            ..Default::default()
        }))
    }

    /// Parse trade inner instruction data
    fn parse_trade_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        let event = match TradeEvent::try_from_slice(data) {
            Ok(mut parsed_event) => {
                parsed_event.metadata = metadata.clone();
                parsed_event
            }
            Err(error) => return Err(ParseError::BorshError(error.to_string())),
        };

        let local_metadata = metadata;

        // Validate trade direction matches event type
        if (local_metadata.event_type == EventType::BonkBuyExactIn
            || local_metadata.event_type == EventType::BonkBuyExactOut)
            && event.trade_direction != TradeDirection::Buy
        {
            return Err(ParseError::InvalidDataFormat(
                "Trade direction does not match expected buy direction".to_owned(),
            ));
        } else if (local_metadata.event_type == EventType::BonkSellExactIn
            || local_metadata.event_type == EventType::BonkSellExactOut)
            && event.trade_direction != TradeDirection::Sell
        {
            return Err(ParseError::InvalidDataFormat(
                "Trade direction does not match expected sell direction".to_owned(),
            ));
        }

        Ok(Box::new(TradeEvent {
            metadata: local_metadata,
            ..event
        }))
    }
}

// ================================================================================================
// Backward Compatibility Re-exports
// ================================================================================================

/// Event types module for backward compatibility
pub mod events {
    pub use super::{discriminators, PoolCreateEvent, TradeEvent};
}

/// Parser module for backward compatibility
pub mod parser {
    pub use super::{EventParser as Parser, BONK_PROGRAM_ID};
}

// Implement the core EventParser trait for compatibility with riglr-events-core
#[async_trait::async_trait]
impl CoreEventParser for EventParser {
    type Input = SolanaTransactionInput;

    #[inline]
    fn can_parse(&self, input: &Self::Input) -> bool {
        match *input {
            SolanaTransactionInput::InnerInstruction(_)
            | SolanaTransactionInput::Instruction(_) => true,
        }
    }

    #[inline]
    fn info(&self) -> &ParserInfo {
        &self.info
    }

    #[inline]
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
                    index: params.index,
                    program_received_time_ms: params.program_received_time_ms,
                };
                self.parse_events_from_inner_instruction(&legacy_params)
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
                    index: params.index,
                    program_received_time_ms: params.program_received_time_ms,
                };
                self.parse_events_from_instruction(&legacy_params)
            }
        };
        Ok(events)
    }
}

impl ProtocolParser for EventParser {
    #[inline]
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner.inner_instruction_configs()
    }

    #[inline]
    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.inner.instruction_configs()
    }

    #[inline]
    fn parse_events_from_inner_instruction(
        &self,
        params: &InnerInstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        self.inner.parse_events_from_inner_instruction(params)
    }

    #[inline]
    fn parse_events_from_instruction(
        &self,
        params: &InstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        self.inner.parse_events_from_instruction(params)
    }

    #[inline]
    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.inner.should_handle(program_id)
    }

    #[inline]
    fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.inner.supported_program_ids()
    }
}

/// Types module for backward compatibility
pub mod types {
    pub use super::{
        ConstantCurve, CurveParams, FixedCurve, LinearCurve, MintParams, PoolStatus,
        TradeDirection, VestingParams,
    };
}

// ================================================================================================
// Test Module
// ================================================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::solana_metadata::SolanaEventMetadata;

    /// Type alias for Solana event metadata in tests
    type EventMetadata = SolanaEventMetadata;

    /// Create a test pubkey from a u8 value
    fn create_test_pubkey(value: u8) -> Pubkey {
        let mut bytes = [0_u8; 32];
        bytes[31] = value;
        Pubkey::new_from_array(bytes)
    }

    /// Create test event metadata with the given event type
    fn create_test_event_metadata(event_type: EventType) -> EventMetadata {
        use crate::metadata_helpers::create_core_metadata;
        let core = create_core_metadata(
            "test-signature".to_owned(),
            riglr_events_core::EventKind::Custom("bonk".to_owned()),
            "solana".to_owned(),
            Some(12345),
        );
        SolanaEventMetadata::new(
            "test-signature".to_owned(),
            12345,
            event_type,
            ProtocolType::Bonk,
            "0".to_owned(),
            12345,
            core,
        )
    }

    /// Create test `TradeEvent` with given trade direction
    fn create_test_bonk_trade_event(direction: TradeDirection) -> TradeEvent {
        TradeEvent {
            trade_direction: direction,
            ..Default::default()
        }
    }

    /// Create test borsh-serialized data for `TradeEvent`
    fn create_test_bonk_trade_event_data(direction: TradeDirection) -> Vec<u8> {
        let event = create_test_bonk_trade_event(direction);
        borsh::to_vec(&event).expect("Event serialization should not fail in test")
    }

    /// Create test borsh-serialized data for `PoolCreateEvent`
    fn create_test_bonk_pool_create_event_data() -> Vec<u8> {
        let event = PoolCreateEvent::default();
        borsh::to_vec(&event).expect("Event serialization should not fail in test")
    }

    #[test]
    fn bonk_program_id_when_checked_should_be_correct() {
        assert_eq!(
            BONK_PROGRAM_ID,
            solana_sdk::pubkey!("bonksoHKfNJJ8Wo8ZJjpw7dHGePNxS2z2WE5GxUPdSo")
        );
    }

    #[test]
    fn bonk_event_parser_new_when_created_should_have_configs() {
        let parser = EventParser::new();
        let instruction_configs = parser.instruction_configs();
        assert!(!instruction_configs.is_empty());
        let inner_configs = parser.inner_instruction_configs();
        assert!(!inner_configs.is_empty());
    }

    #[test]
    fn bonk_event_parser_default_when_created_should_equal_new() {
        let parser_new = EventParser::new();
        let parser_default = EventParser::default();

        let new_supported = parser_new.supported_program_ids();
        let default_supported = parser_default.supported_program_ids();
        assert_eq!(new_supported, default_supported);
    }

    #[test]
    fn bonk_event_parser_should_handle_when_bonk_program_id_should_return_true() {
        let parser = EventParser::new();
        assert!(parser.should_handle(&BONK_PROGRAM_ID));
    }

    #[test]
    fn bonk_event_parser_should_handle_when_other_program_id_should_return_false() {
        let parser = EventParser::new();
        let other_program_id = create_test_pubkey(1);
        assert!(!parser.should_handle(&other_program_id));
    }

    #[test]
    fn bonk_event_parser_supported_program_ids_when_called_should_contain_bonk() {
        let parser = EventParser::new();
        let supported = parser.supported_program_ids();
        assert_eq!(supported.len(), 1);
        assert_eq!(
            supported
                .first()
                .expect("Should have at least one program ID"),
            &BONK_PROGRAM_ID
        );
    }

    #[test]
    fn parse_trade_inner_instruction_when_valid_buy_exact_in_should_return_event() {
        let data = create_test_bonk_trade_event_data(TradeDirection::Buy);
        let metadata = create_test_event_metadata(EventType::BonkBuyExactIn);

        let result = EventParser::parse_trade_inner_instruction(&data, metadata);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(
            event.kind(),
            &riglr_events_core::EventKind::Custom("bonk".to_owned())
        );
    }

    #[test]
    fn parse_trade_inner_instruction_when_valid_buy_exact_out_should_return_event() {
        let data = create_test_bonk_trade_event_data(TradeDirection::Buy);
        let metadata = create_test_event_metadata(EventType::BonkBuyExactOut);

        let result = EventParser::parse_trade_inner_instruction(&data, metadata);
        result.unwrap();
    }

    #[test]
    fn parse_trade_inner_instruction_when_valid_sell_exact_in_should_return_event() {
        let data = create_test_bonk_trade_event_data(TradeDirection::Sell);
        let metadata = create_test_event_metadata(EventType::BonkSellExactIn);

        let result = EventParser::parse_trade_inner_instruction(&data, metadata);
        result.unwrap();
    }

    #[test]
    fn parse_trade_inner_instruction_when_valid_sell_exact_out_should_return_event() {
        let data = create_test_bonk_trade_event_data(TradeDirection::Sell);
        let metadata = create_test_event_metadata(EventType::BonkSellExactOut);

        let result = EventParser::parse_trade_inner_instruction(&data, metadata);
        result.unwrap();
    }

    #[test]
    fn parse_trade_inner_instruction_when_invalid_data_should_return_none() {
        let data = vec![1_u8, 2_u8, 3_u8]; // Invalid borsh data
        let metadata = create_test_event_metadata(EventType::BonkBuyExactIn);

        let result = EventParser::parse_trade_inner_instruction(&data, metadata);
        result.unwrap_err();
    }

    #[test]
    fn parse_trade_inner_instruction_when_empty_data_should_return_none() {
        let data = vec![];
        let metadata = create_test_event_metadata(EventType::BonkBuyExactIn);

        let result = EventParser::parse_trade_inner_instruction(&data, metadata);
        result.unwrap_err();
    }

    #[test]
    fn parse_trade_inner_instruction_when_wrong_trade_direction_buy_should_return_none() {
        let data = create_test_bonk_trade_event_data(TradeDirection::Sell); // Wrong direction
        let metadata = create_test_event_metadata(EventType::BonkBuyExactIn);

        let result = EventParser::parse_trade_inner_instruction(&data, metadata);
        result.unwrap_err();
    }

    #[test]
    fn parse_trade_inner_instruction_when_wrong_trade_direction_sell_should_return_none() {
        let data = create_test_bonk_trade_event_data(TradeDirection::Buy); // Wrong direction
        let metadata = create_test_event_metadata(EventType::BonkSellExactIn);

        let result = EventParser::parse_trade_inner_instruction(&data, metadata);
        result.unwrap_err();
    }

    #[test]
    fn parse_pool_create_inner_instruction_when_valid_data_should_return_event() {
        let data = create_test_bonk_pool_create_event_data();
        let metadata = create_test_event_metadata(EventType::BonkInitialize);

        let result = EventParser::parse_pool_create_inner_instruction(&data, metadata);
        result.unwrap();
    }

    #[test]
    fn parse_pool_create_inner_instruction_when_invalid_data_should_return_none() {
        let data = vec![1_u8, 2_u8, 3_u8]; // Invalid borsh data
        let metadata = create_test_event_metadata(EventType::BonkInitialize);

        let result = EventParser::parse_pool_create_inner_instruction(&data, metadata);
        result.unwrap_err();
    }

    #[test]
    fn parse_pool_create_inner_instruction_when_empty_data_should_return_none() {
        let data = vec![];
        let metadata = create_test_event_metadata(EventType::BonkInitialize);

        let result = EventParser::parse_pool_create_inner_instruction(&data, metadata);
        result.unwrap_err();
    }

    #[test]
    fn parse_buy_exact_in_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0_u8; 16];
        data.get_mut(0..8)
            .expect("Data buffer should be large enough")
            .copy_from_slice(&1000_u64.to_le_bytes());
        data.get_mut(8..16)
            .expect("Data buffer should be large enough")
            .copy_from_slice(&950_u64.to_le_bytes());
        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_event_metadata(EventType::BonkBuyExactIn);

        let event = EventParser::parse_buy_exact_in_instruction(&data, &accounts, metadata)
            .expect("Should successfully parse buy exact in instruction");

        let bonk_event = event.as_any().downcast_ref::<TradeEvent>().unwrap();
        assert_eq!(bonk_event.amount_in, 1000);
        assert_eq!(bonk_event.trade_direction, TradeDirection::Buy);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Create a test pubkey from a u8 value
    fn create_test_pubkey(value: u8) -> Pubkey {
        let mut bytes = [0_u8; 32];
        bytes[31] = value;
        Pubkey::new_from_array(bytes)
    }

    /// Create test event metadata with the given event type
    fn create_test_event_metadata(event_type: EventType) -> EventMetadata {
        use crate::metadata_helpers::create_core_metadata;
        let core = create_core_metadata(
            "test-signature".to_owned(),
            riglr_events_core::EventKind::Custom("bonk".to_owned()),
            "solana".to_owned(),
            Some(12345),
        );
        SolanaEventMetadata::new(
            "test-signature".to_owned(),
            12345,
            event_type,
            ProtocolType::Bonk,
            "0".to_owned(),
            12345,
            core,
        )
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn all_trade_instruction_parsers_return_correct_directions() {
        let data = vec![0_u8; 16];
        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_event_metadata(EventType::BonkBuyExactIn);

        // Test buy exact in
        let result =
            EventParser::parse_buy_exact_in_instruction(&data, &accounts, metadata.clone());
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<TradeEvent>().unwrap();
        assert_eq!(trade_event.trade_direction, TradeDirection::Buy);

        // Test buy exact out
        let result =
            EventParser::parse_buy_exact_out_instruction(&data, &accounts, metadata.clone());
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<TradeEvent>().unwrap();
        assert_eq!(trade_event.trade_direction, TradeDirection::Buy);

        // Test sell exact in
        let result =
            EventParser::parse_sell_exact_in_instruction(&data, &accounts, metadata.clone());
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<TradeEvent>().unwrap();
        assert_eq!(trade_event.trade_direction, TradeDirection::Sell);

        // Test sell exact out
        let result = EventParser::parse_sell_exact_out_instruction(&data, &accounts, metadata);
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<TradeEvent>().unwrap();
        assert_eq!(trade_event.trade_direction, TradeDirection::Sell);
    }
}
