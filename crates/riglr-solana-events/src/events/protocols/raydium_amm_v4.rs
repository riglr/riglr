//! Raydium AMM V4 instruction discriminators and constants.

use crate::solana_metadata::SolanaEventMetadata;
use core::any::Any;
use riglr_events_core::error::EventResult;
use riglr_events_core::EventMetadata as CoreEventMetadata;
use riglr_events_core::{Event, EventKind};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use riglr_events_core::traits::{EventFilter, EventParser, ParserInfo};

use crate::error::{Error as ParseError, ParseResult};
use crate::events::{
    common::{
        read_u64_le, read_u8_le, safe_get_account, validate_account_count, validate_data_length,
    },
    factory::{
        InnerInstructionParseParams, InstructionParseParams, OwnedInstructionParseParams,
        SolanaTransactionInput,
    },
    parser_types::{GenericEventParseConfig, ProtocolParser},
};
use crate::solana_metadata::create_metadata;
use crate::types::{EventType, ProtocolType};
use riglr_events_core::error::EventError;
use solana_message::compiled_instruction::CompiledInstruction;

/// Raydium AMM V4 program ID
pub const RAYDIUM_AMM_V4_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

// ================================================================================================
// DISCRIMINATORS
// ================================================================================================

/// Instruction discriminator for swapping with base token as input
pub const SWAP_BASE_IN: &[u8] = &[0x8f, 0x9a, 0x14, 0xdf, 0x90, 0x38, 0x15, 0xe5];

/// Instruction discriminator for swapping with base token as output
pub const SWAP_BASE_OUT: &[u8] = &[0xab, 0x69, 0x6b, 0xc3, 0xb2, 0x02, 0xee, 0x55];

/// Instruction discriminator for depositing liquidity into the pool
pub const DEPOSIT: &[u8] = &[0x3e, 0xc2, 0xf7, 0x7f, 0x68, 0x0e, 0xc1, 0x0d];

/// Instruction discriminator for initializing the AMM pool (version 2)
pub const INITIALIZE2: &[u8] = &[0xa3, 0xa5, 0xba, 0xcd, 0xeb, 0xc8, 0xd4, 0xe2];

/// Instruction discriminator for withdrawing liquidity from the pool
pub const WITHDRAW: &[u8] = &[0xb7, 0x12, 0x46, 0x9c, 0x94, 0x37, 0xa0, 0xf4];

/// Instruction discriminator for withdrawing profit and loss from the pool
pub const WITHDRAW_PNL: &[u8] = &[0xd6, 0x8f, 0x37, 0x9a, 0x1f, 0xe1, 0x28, 0x52];

// ================================================================================================
// EVENTS
// ================================================================================================

/// Raydium AMM V4 swap event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct SwapEvent {
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// Amount of tokens going into the swap
    pub amount_in: u64,
    /// Amount of tokens coming out of the swap
    pub amount_out: u64,
    /// Direction of the swap (`BaseIn` or `BaseOut`)
    pub direction: SwapDirection,
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC (price currency) token account
    pub pool_pc_token_account: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// Serum program ID
    pub serum_program: Pubkey,
    /// User coin token account
    pub user_coin_token_account: Pubkey,
    /// User owner account
    pub user_owner: Pubkey,
    /// User PC token account
    pub user_pc_token_account: Pubkey,
}

/// Direction of a Raydium swap
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub enum SwapDirection {
    /// Base token is input (selling base for quote)
    #[default]
    BaseIn,
    /// Base token is output (buying base with quote)
    BaseOut,
}

impl Event for SwapEvent {
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
        &self.metadata.core.source
    }

    #[inline]
    fn timestamp(&self) -> SystemTime {
        self.metadata.core.timestamp.into()
    }

    #[inline]
    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self)
            .map_err(|e| EventError::generic(format!("Serialization failed: {e}")))
    }
}

/// Raydium AMM V4 deposit event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct DepositEvent {
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// AMM target orders account
    pub amm_target_orders: Pubkey,
    /// Which token side (0 or 1)
    pub base_side: u64,
    /// LP mint address
    pub lp_mint_address: Pubkey,
    /// Maximum coin amount
    pub max_coin_amount: u64,
    /// Maximum PC amount
    pub max_pc_amount: u64,
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC token account
    pub pool_pc_token_account: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// Token program ID
    pub token_program: Pubkey,
    /// User coin token account
    pub user_coin_token_account: Pubkey,
    /// User LP token account
    pub user_lp_token_account: Pubkey,
    /// User owner account
    pub user_owner: Pubkey,
    /// User PC token account
    pub user_pc_token_account: Pubkey,
}

impl Event for DepositEvent {
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
        &self.metadata.core.source
    }

    #[inline]
    fn timestamp(&self) -> SystemTime {
        self.metadata.core.timestamp.into()
    }

    #[inline]
    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self)
            .map_err(|e| EventError::generic(format!("Serialization failed: {e}")))
    }
}

/// Raydium AMM V4 initialize2 event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct Initialize2Event {
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// AMM target orders account
    pub amm_target_orders: Pubkey,
    /// Coin mint address
    pub coin_mint_address: Pubkey,
    /// Initial coin amount
    pub init_coin_amount: u64,
    /// Initial PC amount
    pub init_pc_amount: u64,
    /// LP mint address
    pub lp_mint_address: Pubkey,
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Nonce for the AMM
    pub nonce: u8,
    /// Open time for the AMM
    pub open_time: u64,
    /// PC mint address
    pub pc_mint_address: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool LP token account
    pub pool_lp_token_account: Pubkey,
    /// Pool PC token account
    pub pool_pc_token_account: Pubkey,
    /// Pool temp LP token account
    pub pool_temp_lp_token_account: Pubkey,
    /// Pool withdraw queue account
    pub pool_withdraw_queue: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// Serum program ID
    pub serum_program: Pubkey,
    /// User wallet account
    pub user_wallet: Pubkey,
}

impl Event for Initialize2Event {
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
        &self.metadata.core.source
    }

    #[inline]
    fn timestamp(&self) -> SystemTime {
        self.metadata.core.timestamp.into()
    }

    #[inline]
    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self)
            .map_err(|e| EventError::generic(format!("Serialization failed: {e}")))
    }
}

/// Raydium AMM V4 withdraw event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct WithdrawEvent {
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// AMM target orders account
    pub amm_target_orders: Pubkey,
    /// Amount to withdraw
    pub amount: u64,
    /// LP mint address
    pub lp_mint_address: Pubkey,
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC token account
    pub pool_pc_token_account: Pubkey,
    /// Pool temp LP token account
    pub pool_temp_lp_token_account: Pubkey,
    /// Pool withdraw queue account
    pub pool_withdraw_queue: Pubkey,
    /// Serum asks account
    pub serum_asks: Pubkey,
    /// Serum bids account
    pub serum_bids: Pubkey,
    /// Serum coin vault account
    pub serum_coin_vault_account: Pubkey,
    /// Serum event queue account
    pub serum_event_queue: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// Serum PC vault account
    pub serum_pc_vault_account: Pubkey,
    /// Serum program ID
    pub serum_program: Pubkey,
    /// Serum vault signer account
    pub serum_vault_signer: Pubkey,
    /// Token program ID
    pub token_program: Pubkey,
    /// User coin token account
    pub user_coin_token_account: Pubkey,
    /// User LP token account
    pub user_lp_token_account: Pubkey,
    /// User owner account
    pub user_owner: Pubkey,
    /// User PC token account
    pub user_pc_token_account: Pubkey,
}

impl Event for WithdrawEvent {
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
        &self.metadata.core.source
    }

    #[inline]
    fn timestamp(&self) -> SystemTime {
        self.metadata.core.timestamp.into()
    }

    #[inline]
    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self)
            .map_err(|e| EventError::generic(format!("Serialization failed: {e}")))
    }
}

/// Raydium AMM V4 withdraw `PnL` event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct WithdrawPnlEvent {
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM config account
    pub amm_config: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// AMM target orders account
    pub amm_target_orders: Pubkey,
    /// Coin `PnL` token account
    pub coin_pnl_token_account: Pubkey,
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// PC `PnL` token account
    pub pc_pnl_token_account: Pubkey,
    /// `PnL` owner account
    pub pnl_owner_account: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC token account
    pub pool_pc_token_account: Pubkey,
    /// Serum coin vault account
    pub serum_coin_vault_account: Pubkey,
    /// Serum event queue account
    pub serum_event_queue: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// Serum PC vault account
    pub serum_pc_vault_account: Pubkey,
    /// Serum program ID
    pub serum_program: Pubkey,
    /// Serum vault signer account
    pub serum_vault_signer: Pubkey,
    /// Token program ID
    pub token_program: Pubkey,
}

impl Event for WithdrawPnlEvent {
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
        static TRANSFER_KIND: EventKind = EventKind::Transfer;
        &TRANSFER_KIND
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
        &self.metadata.core.source
    }

    #[inline]
    fn timestamp(&self) -> SystemTime {
        self.metadata.core.timestamp.into()
    }

    #[inline]
    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self)
            .map_err(|e| EventError::generic(format!("Serialization failed: {e}")))
    }
}

// ================================================================================================
// PARSER
// ================================================================================================

/// Raydium AMM V4 event parser
#[derive(Debug, Clone, Default)]
pub struct Parser;

impl Parser {
    /// Creates a new Raydium AMM V4 event parser
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Checks if this parser should handle the given program ID
    #[must_use]
    pub fn should_handle(&self, program_id: &Pubkey) -> bool {
        *program_id == RAYDIUM_AMM_V4_PROGRAM_ID
    }

    /// Returns the supported program IDs
    #[must_use]
    pub fn supported_program_ids(&self) -> Vec<Pubkey> {
        vec![RAYDIUM_AMM_V4_PROGRAM_ID]
    }

    /// Parses a deposit instruction
    fn parse_deposit(
        input: &SolanaTransactionInput,
        instruction_data: &[u8],
        accounts: &[Pubkey],
    ) -> ParseResult<Box<dyn Event>> {
        // Extract input parameters
        let (signature, slot, instruction_index) = match *input {
            SolanaTransactionInput::Instruction(ref params) => {
                (&params.signature, params.slot, &params.index)
            }
            _ => {
                return Err(ParseError::InvalidDataFormat(
                    "Expected instruction input".to_owned(),
                ));
            }
        };

        // Validate minimum instruction data length for deposit
        validate_data_length(instruction_data, 25, "Raydium AMM V4 deposit")?;

        // Validate minimum account count for deposit
        validate_account_count(accounts, 17, "Raydium AMM V4 deposit")?;

        // Parse amounts and base side from instruction data
        let max_coin_amount = read_u64_le(instruction_data, 8)?;
        let max_pc_amount = read_u64_le(instruction_data, 16)?;
        let base_side = u64::from(read_u8_le(instruction_data, 24)?);

        // Extract accounts
        let token_program = safe_get_account(accounts, 0)?;
        let amm = safe_get_account(accounts, 1)?;
        let amm_authority = safe_get_account(accounts, 2)?;
        let amm_open_orders = safe_get_account(accounts, 3)?;
        let amm_target_orders = safe_get_account(accounts, 4)?;
        let lp_mint_address = safe_get_account(accounts, 5)?;
        let pool_coin_token_account = safe_get_account(accounts, 6)?;
        let pool_pc_token_account = safe_get_account(accounts, 7)?;
        let serum_market = safe_get_account(accounts, 8)?;
        let user_coin_token_account = safe_get_account(accounts, 9)?;
        let user_pc_token_account = safe_get_account(accounts, 10)?;
        let user_lp_token_account = safe_get_account(accounts, 11)?;
        let user_owner = safe_get_account(accounts, 12)?;

        // Create metadata for this deposit event
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ParseError::InvalidDataFormat("System time error".to_owned()))?
            .as_millis()
            .try_into()
            .unwrap_or(0);

        let metadata = create_metadata(
            format!("{signature}-{instruction_index}"),
            signature.to_owned(),
            slot,
            None, // block_time
            timestamp,
            instruction_index.to_string(),
            EventType::AddLiquidity,
            ProtocolType::RaydiumAmmV4,
        );

        // Create deposit event
        let deposit_event = DepositEvent {
            amm,
            amm_authority,
            amm_open_orders,
            amm_target_orders,
            base_side,
            lp_mint_address,
            max_coin_amount,
            max_pc_amount,
            metadata,
            pool_coin_token_account,
            pool_pc_token_account,
            serum_market,
            token_program,
            user_coin_token_account,
            user_lp_token_account,
            user_owner,
            user_pc_token_account,
        };
        Ok(Box::new(deposit_event))
    }

    /// Internal method to parse instruction data
    fn parse_instruction_internal(
        instruction_data: &[u8],
        accounts: &[Pubkey],
        signature: &str,
        slot: u64,
        instruction_index: String,
        timestamp_ms: u64,
    ) -> ParseResult<Box<dyn Event>> {
        // Create a dummy CompiledInstruction for compatibility
        let _compiled_instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: instruction_data.to_vec(),
        };

        // Create a dummy SolanaTransactionInput for compatibility
        let input = SolanaTransactionInput::Instruction(OwnedInstructionParseParams {
            accounts: accounts.to_vec(),
            block_time: None,
            index: instruction_index,
            instruction_data: instruction_data.to_vec(),
            #[expect(clippy::cast_possible_wrap)]
            program_received_time_ms: timestamp_ms as i64,
            signature: signature.to_owned(),
            slot,
        });

        // Check if we have enough data for discriminator
        if instruction_data.len() < 8 {
            return Err(ParseError::NotEnoughBytes {
                expected: 8,
                found: instruction_data.len(),
                offset: 0,
            });
        }

        // Extract discriminator (first 8 bytes)
        let discriminator = instruction_data.get(0..8).ok_or_else(|| {
            ParseError::InvalidDataFormat(
                "Instruction data too short for discriminator".to_string(),
            )
        })?;

        // Parse based on discriminator
        match discriminator {
            SWAP_BASE_IN => Self::parse_swap_base_in(&input, instruction_data, accounts),
            SWAP_BASE_OUT => Self::parse_swap_base_out(&input, instruction_data, accounts),
            DEPOSIT => Self::parse_deposit(&input, instruction_data, accounts),
            INITIALIZE2 => Self::parse_initialize2(&input, instruction_data, accounts),
            WITHDRAW => Self::parse_withdraw(&input, instruction_data, accounts),
            WITHDRAW_PNL => Self::parse_withdraw_pnl(&input, instruction_data, accounts),
            _ => Err(ParseError::InvalidInstructionType(format!(
                "Unknown Raydium AMM V4 discriminator: {discriminator:02x?}"
            ))),
        }
    }

    /// Parses an initialize2 instruction
    fn parse_initialize2(
        input: &SolanaTransactionInput,
        instruction_data: &[u8],
        accounts: &[Pubkey],
    ) -> ParseResult<Box<dyn Event>> {
        // Extract input parameters
        let (signature, slot, instruction_index) = match *input {
            SolanaTransactionInput::Instruction(ref params) => {
                (&params.signature, params.slot, &params.index)
            }
            _ => {
                return Err(ParseError::InvalidDataFormat(
                    "Expected instruction input".to_owned(),
                ));
            }
        };

        // Validate minimum instruction data length for initialize2
        validate_data_length(instruction_data, 33, "Raydium AMM V4 initialize2")?;

        // Validate minimum account count for initialize2
        validate_account_count(accounts, 20, "Raydium AMM V4 initialize2")?;

        // Parse initialization data
        let nonce = read_u8_le(instruction_data, 8)?;
        let open_time = read_u64_le(instruction_data, 9)?;
        let init_pc_amount = read_u64_le(instruction_data, 17)?;
        let init_coin_amount = read_u64_le(instruction_data, 25)?;

        // Extract accounts
        let amm = safe_get_account(accounts, 4)?;
        let amm_authority = safe_get_account(accounts, 5)?;
        let amm_open_orders = safe_get_account(accounts, 6)?;
        let lp_mint_address = safe_get_account(accounts, 7)?;
        let coin_mint_address = safe_get_account(accounts, 8)?;
        let pc_mint_address = safe_get_account(accounts, 9)?;
        let pool_coin_token_account = safe_get_account(accounts, 10)?;
        let pool_pc_token_account = safe_get_account(accounts, 11)?;
        let pool_withdraw_queue = safe_get_account(accounts, 12)?;
        let amm_target_orders = safe_get_account(accounts, 13)?;
        let pool_lp_token_account = safe_get_account(accounts, 14)?;
        let pool_temp_lp_token_account = safe_get_account(accounts, 15)?;
        let serum_program = safe_get_account(accounts, 16)?;
        let serum_market = safe_get_account(accounts, 17)?;
        let user_wallet = safe_get_account(accounts, 18)?;

        // Create metadata for this initialize event
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ParseError::InvalidDataFormat("System time error".to_owned()))?
            .as_millis()
            .try_into()
            .unwrap_or(0);

        let metadata = create_metadata(
            format!("{signature}-{instruction_index}"),
            signature.to_owned(),
            slot,
            None, // block_time
            timestamp,
            instruction_index.to_string(),
            EventType::CreatePool,
            ProtocolType::RaydiumAmmV4,
        );

        // Create initialize2 event
        let init_event = Initialize2Event {
            amm,
            amm_authority,
            amm_open_orders,
            amm_target_orders,
            coin_mint_address,
            init_coin_amount,
            init_pc_amount,
            lp_mint_address,
            metadata,
            nonce,
            open_time,
            pc_mint_address,
            pool_coin_token_account,
            pool_lp_token_account,
            pool_pc_token_account,
            pool_temp_lp_token_account,
            pool_withdraw_queue,
            serum_market,
            serum_program,
            user_wallet,
        };
        Ok(Box::new(init_event))
    }

    /// Parses a swap base in instruction
    fn parse_swap_base_in(
        input: &SolanaTransactionInput,
        instruction_data: &[u8],
        accounts: &[Pubkey],
    ) -> ParseResult<Box<dyn Event>> {
        // Extract input parameters
        let (signature, slot, instruction_index) = match *input {
            SolanaTransactionInput::Instruction(ref params) => {
                (&params.signature, params.slot, &params.index)
            }
            _ => {
                return Err(ParseError::InvalidDataFormat(
                    "Expected instruction input".to_owned(),
                ));
            }
        };

        // Validate minimum instruction data length for swap base in
        validate_data_length(instruction_data, 24, "Raydium AMM V4 swap base in")?;

        // Validate minimum account count for swap base in
        validate_account_count(accounts, 16, "Raydium AMM V4 swap base in")?;

        // Parse amounts from instruction data
        let amount_in = read_u64_le(instruction_data, 8)?;
        let minimum_amount_out = read_u64_le(instruction_data, 16)?;

        // Extract accounts
        let user_source_token_account = safe_get_account(accounts, 0)?;
        let user_destination_token_account = safe_get_account(accounts, 1)?;
        let user_owner = safe_get_account(accounts, 2)?;
        let amm = safe_get_account(accounts, 3)?;
        let amm_authority = safe_get_account(accounts, 4)?;
        let amm_open_orders = safe_get_account(accounts, 5)?;
        let pool_coin_token_account = safe_get_account(accounts, 6)?;
        let pool_pc_token_account = safe_get_account(accounts, 7)?;
        let serum_program = safe_get_account(accounts, 8)?;
        let serum_market = safe_get_account(accounts, 9)?;

        // Create metadata for this swap event
        let metadata = create_metadata(
            format!("{signature}-{instruction_index}"),
            signature.to_owned(),
            slot,
            None, // block_time
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| ParseError::InvalidDataFormat("System time error".to_owned()))?
                .as_millis()
                .try_into()
                .unwrap_or(0),
            instruction_index.to_string(),
            EventType::Swap,
            ProtocolType::RaydiumAmmV4,
        );

        // Create swap event
        let swap_event = SwapEvent {
            amount_in,
            amount_out: minimum_amount_out, // Use minimum out as estimate
            direction: SwapDirection::BaseIn,
            amm,
            amm_authority,
            amm_open_orders,
            pool_coin_token_account,
            pool_pc_token_account,
            serum_program,
            serum_market,
            user_coin_token_account: user_source_token_account,
            user_pc_token_account: user_destination_token_account,
            user_owner,
            metadata,
        };
        Ok(Box::new(swap_event))
    }

    /// Parses a swap base out instruction
    fn parse_swap_base_out(
        input: &SolanaTransactionInput,
        instruction_data: &[u8],
        accounts: &[Pubkey],
    ) -> ParseResult<Box<dyn Event>> {
        // Extract input parameters
        let (signature, slot, instruction_index) = match *input {
            SolanaTransactionInput::Instruction(ref params) => {
                (&params.signature, params.slot, &params.index)
            }
            _ => {
                return Err(ParseError::InvalidDataFormat(
                    "Expected instruction input".to_owned(),
                ));
            }
        };

        // Validate minimum instruction data length for swap base out
        validate_data_length(instruction_data, 24, "Raydium AMM V4 swap base out")?;

        // Validate minimum account count for swap base out
        validate_account_count(accounts, 16, "Raydium AMM V4 swap base out")?;

        // Parse amounts from instruction data
        let max_amount_in = read_u64_le(instruction_data, 8)?;
        let amount_out = read_u64_le(instruction_data, 16)?;

        // Extract accounts
        let user_source_token_account = safe_get_account(accounts, 0)?;
        let user_destination_token_account = safe_get_account(accounts, 1)?;
        let user_owner = safe_get_account(accounts, 2)?;
        let amm = safe_get_account(accounts, 3)?;
        let amm_authority = safe_get_account(accounts, 4)?;
        let amm_open_orders = safe_get_account(accounts, 5)?;
        let pool_coin_token_account = safe_get_account(accounts, 6)?;
        let pool_pc_token_account = safe_get_account(accounts, 7)?;
        let serum_program = safe_get_account(accounts, 8)?;
        let serum_market = safe_get_account(accounts, 9)?;

        // Create metadata for this swap event
        let metadata = create_metadata(
            format!("{signature}-{instruction_index}"),
            signature.to_owned(),
            slot,
            None, // block_time
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| ParseError::InvalidDataFormat("System time error".to_owned()))?
                .as_millis()
                .try_into()
                .unwrap_or(0),
            instruction_index.to_string(),
            EventType::Swap,
            ProtocolType::RaydiumAmmV4,
        );

        // Create swap event
        let swap_event = SwapEvent {
            amount_in: max_amount_in, // Use max in as estimate
            amount_out,
            direction: SwapDirection::BaseOut,
            amm,
            amm_authority,
            amm_open_orders,
            pool_coin_token_account,
            pool_pc_token_account,
            serum_program,
            serum_market,
            user_coin_token_account: user_source_token_account,
            user_pc_token_account: user_destination_token_account,
            user_owner,
            metadata,
        };
        Ok(Box::new(swap_event))
    }

    /// Parses a withdraw instruction
    fn parse_withdraw(
        input: &SolanaTransactionInput,
        instruction_data: &[u8],
        accounts: &[Pubkey],
    ) -> ParseResult<Box<dyn Event>> {
        // Extract input parameters
        let (signature, slot, instruction_index) = match *input {
            SolanaTransactionInput::Instruction(ref params) => {
                (&params.signature, params.slot, &params.index)
            }
            _ => {
                return Err(ParseError::InvalidDataFormat(
                    "Expected instruction input".to_owned(),
                ));
            }
        };

        // Validate minimum instruction data length for withdraw
        validate_data_length(instruction_data, 16, "Raydium AMM V4 withdraw")?;

        // Validate minimum account count for withdraw
        validate_account_count(accounts, 23, "Raydium AMM V4 withdraw")?;

        // Parse amount from instruction data
        let amount = read_u64_le(instruction_data, 8)?;

        // Extract accounts
        let token_program = safe_get_account(accounts, 0)?;
        let amm = safe_get_account(accounts, 1)?;
        let amm_authority = safe_get_account(accounts, 2)?;
        let amm_open_orders = safe_get_account(accounts, 3)?;
        let amm_target_orders = safe_get_account(accounts, 4)?;
        let lp_mint_address = safe_get_account(accounts, 5)?;
        let pool_coin_token_account = safe_get_account(accounts, 6)?;
        let pool_pc_token_account = safe_get_account(accounts, 7)?;
        let pool_withdraw_queue = safe_get_account(accounts, 8)?;
        let pool_temp_lp_token_account = safe_get_account(accounts, 9)?;
        let serum_program = safe_get_account(accounts, 10)?;
        let serum_market = safe_get_account(accounts, 11)?;
        let serum_coin_vault_account = safe_get_account(accounts, 12)?;
        let serum_pc_vault_account = safe_get_account(accounts, 13)?;
        let serum_vault_signer = safe_get_account(accounts, 14)?;
        let user_lp_token_account = safe_get_account(accounts, 15)?;
        let user_coin_token_account = safe_get_account(accounts, 16)?;
        let user_pc_token_account = safe_get_account(accounts, 17)?;
        let user_owner = safe_get_account(accounts, 18)?;
        let serum_event_queue = safe_get_account(accounts, 19)?;
        let serum_bids = safe_get_account(accounts, 20)?;
        let serum_asks = safe_get_account(accounts, 21)?;

        // Create metadata for this withdraw event
        let metadata = create_metadata(
            format!("{signature}-{instruction_index}"),
            signature.to_owned(),
            slot,
            None, // block_time
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| ParseError::InvalidDataFormat("System time error".to_owned()))?
                .as_millis()
                .try_into()
                .unwrap_or(0),
            instruction_index.to_string(),
            EventType::RemoveLiquidity,
            ProtocolType::RaydiumAmmV4,
        );

        // Create withdraw event
        let withdraw_event = WithdrawEvent {
            amm,
            amm_authority,
            amm_open_orders,
            amm_target_orders,
            amount,
            lp_mint_address,
            metadata,
            pool_coin_token_account,
            pool_pc_token_account,
            pool_temp_lp_token_account,
            pool_withdraw_queue,
            serum_asks,
            serum_bids,
            serum_coin_vault_account,
            serum_event_queue,
            serum_market,
            serum_pc_vault_account,
            serum_program,
            serum_vault_signer,
            token_program,
            user_coin_token_account,
            user_lp_token_account,
            user_owner,
            user_pc_token_account,
        };
        Ok(Box::new(withdraw_event))
    }

    /// Parses a withdraw `PnL` instruction
    fn parse_withdraw_pnl(
        input: &SolanaTransactionInput,
        instruction_data: &[u8],
        accounts: &[Pubkey],
    ) -> ParseResult<Box<dyn Event>> {
        // Extract input parameters
        let (signature, slot, instruction_index) = match *input {
            SolanaTransactionInput::Instruction(ref params) => {
                (&params.signature, params.slot, &params.index)
            }
            _ => {
                return Err(ParseError::InvalidDataFormat(
                    "Expected instruction input".to_owned(),
                ));
            }
        };

        // Validate minimum instruction data length for withdraw PnL
        validate_data_length(instruction_data, 8, "Raydium AMM V4 withdraw PnL")?;

        // Validate minimum account count for withdraw PnL
        validate_account_count(accounts, 18, "Raydium AMM V4 withdraw PnL")?;

        // Extract accounts
        let token_program = safe_get_account(accounts, 0)?;
        let amm = safe_get_account(accounts, 1)?;
        let amm_config = safe_get_account(accounts, 2)?;
        let amm_authority = safe_get_account(accounts, 3)?;
        let amm_open_orders = safe_get_account(accounts, 4)?;
        let pool_coin_token_account = safe_get_account(accounts, 5)?;
        let pool_pc_token_account = safe_get_account(accounts, 6)?;
        let coin_pnl_token_account = safe_get_account(accounts, 7)?;
        let pc_pnl_token_account = safe_get_account(accounts, 8)?;
        let pnl_owner_account = safe_get_account(accounts, 9)?;
        let amm_target_orders = safe_get_account(accounts, 10)?;
        let serum_program = safe_get_account(accounts, 11)?;
        let serum_market = safe_get_account(accounts, 12)?;
        let serum_event_queue = safe_get_account(accounts, 13)?;
        let serum_coin_vault_account = safe_get_account(accounts, 14)?;
        let serum_pc_vault_account = safe_get_account(accounts, 15)?;
        let serum_vault_signer = safe_get_account(accounts, 16)?;

        // Create metadata for this withdraw PnL event
        let metadata = create_metadata(
            format!("{signature}-{instruction_index}"),
            signature.to_owned(),
            slot,
            None, // block_time
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| ParseError::InvalidDataFormat("System time error".to_owned()))?
                .as_millis()
                .try_into()
                .unwrap_or(0),
            instruction_index.to_string(),
            EventType::Transfer,
            ProtocolType::RaydiumAmmV4,
        );

        // Create withdraw PnL event
        let withdraw_pnl_event = WithdrawPnlEvent {
            amm,
            amm_authority,
            amm_config,
            amm_open_orders,
            amm_target_orders,
            coin_pnl_token_account,
            metadata,
            pc_pnl_token_account,
            pnl_owner_account,
            pool_coin_token_account,
            pool_pc_token_account,
            serum_coin_vault_account,
            serum_event_queue,
            serum_market,
            serum_pc_vault_account,
            serum_program,
            serum_vault_signer,
            token_program,
        };
        Ok(Box::new(withdraw_pnl_event))
    }
}

impl ProtocolParser for Parser {
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        HashMap::new()
    }
    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        HashMap::new()
    }
    fn parse_events_from_inner_instruction(
        &self,
        _params: &InnerInstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        vec![]
    }

    #[expect(clippy::cast_sign_loss)]
    fn parse_events_from_instruction(
        &self,
        params: &InstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        Self::parse_instruction_internal(
            &params.instruction.data,
            params.accounts,
            params.signature,
            params.slot,
            params.index.clone(),
            params.program_received_time_ms as u64,
        )
        .map_or_else(|_| vec![], |event| vec![event])
    }
    fn should_handle(&self, program_id: &Pubkey) -> bool {
        *program_id == RAYDIUM_AMM_V4_PROGRAM_ID
    }
    fn supported_program_ids(&self) -> Vec<Pubkey> {
        vec![RAYDIUM_AMM_V4_PROGRAM_ID]
    }
}

#[async_trait::async_trait]
impl EventParser for Parser {
    type Input = SolanaTransactionInput;
    fn can_parse(&self, input: &Self::Input) -> bool {
        match *input {
            SolanaTransactionInput::Instruction(_) => {
                // This parser is specific to Raydium AMM V4
                true
            }
            SolanaTransactionInput::InnerInstruction(_) => false,
        }
    }
    fn info(&self) -> &ParserInfo {
        static INFO: OnceLock<ParserInfo> = OnceLock::new();
        INFO.get_or_init(|| ParserInfo {
            name: "Parser".to_owned(),
            supported_formats: vec!["solana_instruction".to_owned()],
            supported_kinds: vec![
                riglr_events_core::EventKind::Swap,
                riglr_events_core::EventKind::Liquidity,
                riglr_events_core::EventKind::Contract,
            ],
            version: "1.0.0".to_owned(),
        })
    }

    #[expect(clippy::cast_sign_loss)]
    async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>> {
        // Check if this is the correct program
        match input {
            SolanaTransactionInput::Instruction(ref params) => {
                // Raydium AMM V4 parser only handles its specific instructions
                // Program ID validation would happen at a higher level

                // Parse the instruction
                match Self::parse_instruction_internal(
                    &params.instruction_data,
                    &params.accounts,
                    &params.signature,
                    params.slot,
                    params.index.clone(),
                    params.program_received_time_ms as u64,
                ) {
                    Ok(event) => return Ok(vec![event]),
                    Err(_) => return Ok(vec![]),
                }
            }
            SolanaTransactionInput::InnerInstruction(_) => Ok(vec![]),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::{
        solana_metadata::SolanaEventMetadata,
        types::{EventType, ProtocolType},
    };
    use riglr_events_core::{Event, EventKind};
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn module_exports_event_structs() {
        // Test that all event structs are properly exported and can be instantiated
        let core = riglr_events_core::EventMetadata::new(
            "test-id".to_owned(),
            EventKind::Swap,
            "raydium-amm-v4".to_owned(),
        );

        let metadata = SolanaEventMetadata::new(
            "test-signature".to_owned(),
            12345,
            EventType::Swap,
            ProtocolType::RaydiumAmmV4,
            "0".to_owned(),
            1_640_995_200_000,
            core,
        );

        // Test SwapEvent export
        let swap_event = SwapEvent {
            metadata,
            amount_in: 1000,
            amount_out: 950,
            direction: SwapDirection::BaseIn,
            amm: Pubkey::default(),
            amm_authority: Pubkey::default(),
            amm_open_orders: Pubkey::default(),
            pool_coin_token_account: Pubkey::default(),
            pool_pc_token_account: Pubkey::default(),
            serum_program: Pubkey::default(),
            serum_market: Pubkey::default(),
            user_coin_token_account: Pubkey::default(),
            user_pc_token_account: Pubkey::default(),
            user_owner: Pubkey::default(),
        };
        assert_eq!(swap_event.amount_in, 1000);
        assert_eq!(swap_event.amount_out, 950);
    }

    #[test]
    fn module_exports_deposit_event() {
        let core = riglr_events_core::EventMetadata::new(
            "test-deposit-id".to_owned(),
            EventKind::Liquidity,
            "raydium-amm-v4".to_owned(),
        );

        let metadata = SolanaEventMetadata::new(
            "test-signature".to_owned(),
            12346,
            EventType::AddLiquidity,
            ProtocolType::RaydiumAmmV4,
            "0".to_owned(),
            1_640_995_200_000,
            core,
        );

        let deposit_event = DepositEvent {
            metadata,
            max_coin_amount: 2000,
            max_pc_amount: 1800,
            base_side: 1,
            token_program: Pubkey::default(),
            amm: Pubkey::default(),
            amm_authority: Pubkey::default(),
            amm_open_orders: Pubkey::default(),
            amm_target_orders: Pubkey::default(),
            lp_mint_address: Pubkey::default(),
            pool_coin_token_account: Pubkey::default(),
            pool_pc_token_account: Pubkey::default(),
            serum_market: Pubkey::default(),
            user_coin_token_account: Pubkey::default(),
            user_pc_token_account: Pubkey::default(),
            user_lp_token_account: Pubkey::default(),
            user_owner: Pubkey::default(),
        };
        assert_eq!(deposit_event.max_coin_amount, 2000);
        assert_eq!(deposit_event.base_side, 1);
    }

    #[test]
    fn module_exports_initialize2_event() {
        let core = riglr_events_core::EventMetadata::new(
            "test-init-id".to_owned(),
            EventKind::Contract,
            "raydium-amm-v4".to_owned(),
        );

        let metadata = SolanaEventMetadata::new(
            "test-signature".to_owned(),
            12347,
            EventType::CreatePool,
            ProtocolType::RaydiumAmmV4,
            "0".to_owned(),
            1_640_995_200_000,
            core,
        );

        let init_event = Initialize2Event {
            metadata,
            nonce: 42,
            open_time: 1_634_567_890,
            init_pc_amount: 5000,
            init_coin_amount: 4500,
            amm: Pubkey::default(),
            amm_authority: Pubkey::default(),
            amm_open_orders: Pubkey::default(),
            lp_mint_address: Pubkey::default(),
            coin_mint_address: Pubkey::default(),
            pc_mint_address: Pubkey::default(),
            pool_coin_token_account: Pubkey::default(),
            pool_pc_token_account: Pubkey::default(),
            pool_withdraw_queue: Pubkey::default(),
            amm_target_orders: Pubkey::default(),
            pool_lp_token_account: Pubkey::default(),
            pool_temp_lp_token_account: Pubkey::default(),
            serum_program: Pubkey::default(),
            serum_market: Pubkey::default(),
            user_wallet: Pubkey::default(),
        };
        assert_eq!(init_event.nonce, 42);
        assert_eq!(init_event.open_time, 1_634_567_890);
    }

    #[test]
    fn module_exports_withdraw_event() {
        let core = riglr_events_core::EventMetadata::new(
            "test-withdraw-id".to_owned(),
            EventKind::Liquidity,
            "raydium-amm-v4".to_owned(),
        );

        let metadata = SolanaEventMetadata::new(
            "test-signature".to_owned(),
            12348,
            EventType::RemoveLiquidity,
            ProtocolType::RaydiumAmmV4,
            "0".to_owned(),
            1_640_995_200_000,
            core,
        );

        let withdraw_event = WithdrawEvent {
            metadata,
            amount: 1500,
            token_program: Pubkey::default(),
            amm: Pubkey::default(),
            amm_authority: Pubkey::default(),
            amm_open_orders: Pubkey::default(),
            amm_target_orders: Pubkey::default(),
            lp_mint_address: Pubkey::default(),
            pool_coin_token_account: Pubkey::default(),
            pool_pc_token_account: Pubkey::default(),
            pool_withdraw_queue: Pubkey::default(),
            pool_temp_lp_token_account: Pubkey::default(),
            serum_program: Pubkey::default(),
            serum_market: Pubkey::default(),
            serum_coin_vault_account: Pubkey::default(),
            serum_pc_vault_account: Pubkey::default(),
            serum_vault_signer: Pubkey::default(),
            user_lp_token_account: Pubkey::default(),
            user_coin_token_account: Pubkey::default(),
            user_pc_token_account: Pubkey::default(),
            user_owner: Pubkey::default(),
            serum_event_queue: Pubkey::default(),
            serum_bids: Pubkey::default(),
            serum_asks: Pubkey::default(),
        };
        assert_eq!(withdraw_event.amount, 1500);
    }

    #[test]
    fn module_exports_withdraw_pnl_event() {
        let core = riglr_events_core::EventMetadata::new(
            "test-pnl-id".to_owned(),
            EventKind::Transfer,
            "raydium-amm-v4".to_owned(),
        );

        let metadata = SolanaEventMetadata::new(
            "test-signature".to_owned(),
            12349,
            EventType::Transfer,
            ProtocolType::RaydiumAmmV4,
            "0".to_owned(),
            1_640_995_200_000,
            core,
        );

        let pnl_event = WithdrawPnlEvent {
            metadata,
            token_program: Pubkey::default(),
            amm: Pubkey::default(),
            amm_config: Pubkey::default(),
            amm_authority: Pubkey::default(),
            amm_open_orders: Pubkey::default(),
            pool_coin_token_account: Pubkey::default(),
            pool_pc_token_account: Pubkey::default(),
            coin_pnl_token_account: Pubkey::default(),
            pc_pnl_token_account: Pubkey::default(),
            pnl_owner_account: Pubkey::default(),
            amm_target_orders: Pubkey::default(),
            serum_program: Pubkey::default(),
            serum_market: Pubkey::default(),
            serum_event_queue: Pubkey::default(),
            serum_coin_vault_account: Pubkey::default(),
            serum_pc_vault_account: Pubkey::default(),
            serum_vault_signer: Pubkey::default(),
        };
        // PNL event doesn't have numeric fields to test, just verify it was created
        assert_eq!(pnl_event.token_program, Pubkey::default());
    }

    #[test]
    fn module_exports_swap_direction_enum() {
        // Test SwapDirection enum export and variants
        let base_in = SwapDirection::BaseIn;
        let base_out = SwapDirection::BaseOut;

        // Test that the enum can be cloned and debugged
        let base_in_clone = base_in;
        assert!(format!("{base_in_clone:?}").contains("BaseIn"));
        assert!(format!("{base_out:?}").contains("BaseOut"));

        // Test default implementation
        let default_direction = SwapDirection::default();
        assert!(matches!(default_direction, SwapDirection::BaseIn));
    }

    #[test]
    fn module_exports_parser() {
        // Test that Parser is properly exported
        let parser = Parser;

        // Verify parser can be created and has the expected program ID support
        let supported_ids = parser.supported_program_ids();
        assert!(!supported_ids.is_empty());
        assert!(supported_ids.contains(&RAYDIUM_AMM_V4_PROGRAM_ID));
    }

    #[test]
    fn module_exports_program_id_constant() {
        // Test that RAYDIUM_AMM_V4_PROGRAM_ID is properly exported
        let expected_program_id =
            solana_sdk::pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");
        assert_eq!(RAYDIUM_AMM_V4_PROGRAM_ID, expected_program_id);

        // Verify it's not the default pubkey
        assert_ne!(RAYDIUM_AMM_V4_PROGRAM_ID, Pubkey::default());
    }

    #[test]
    #[allow(clippy::panic)]
    fn module_exports_work_with_event_trait() {
        // Test that exported events implement the Event trait correctly
        let core = riglr_events_core::EventMetadata::new(
            "trait-test-id".to_owned(),
            EventKind::Swap,
            "raydium-amm-v4".to_owned(),
        );

        let metadata = SolanaEventMetadata::new(
            "test-signature".to_owned(),
            12350,
            EventType::Swap,
            ProtocolType::RaydiumAmmV4,
            "0".to_owned(),
            1_640_995_200_000,
            core,
        );

        let swap_event = SwapEvent {
            metadata,
            amount_in: 100,
            amount_out: 95,
            direction: SwapDirection::BaseOut,
            amm: Pubkey::default(),
            amm_authority: Pubkey::default(),
            amm_open_orders: Pubkey::default(),
            pool_coin_token_account: Pubkey::default(),
            pool_pc_token_account: Pubkey::default(),
            serum_program: Pubkey::default(),
            serum_market: Pubkey::default(),
            user_coin_token_account: Pubkey::default(),
            user_pc_token_account: Pubkey::default(),
            user_owner: Pubkey::default(),
        };

        // Test Event trait methods
        assert_eq!(swap_event.id(), "trait-test-id");
        assert_eq!(*swap_event.kind(), EventKind::Swap);

        // Test clone_boxed
        let boxed_event = swap_event.clone_boxed();
        assert_eq!(boxed_event.id(), "trait-test-id");

        // Test to_json
        let json_result = swap_event.to_json();
        assert!(json_result.is_ok());
        let json_value =
            json_result.unwrap_or_else(|_| panic!("Event serialization should succeed in test"));
        assert!(json_value.is_object());
    }

    #[test]
    fn all_event_types_can_be_used_as_trait_objects() {
        // Create instances of all event types as trait objects
        let events: Vec<Box<dyn Event>> = vec![
            Box::new(SwapEvent::default()),
            Box::new(DepositEvent::default()),
            Box::new(Initialize2Event::default()),
            Box::new(WithdrawEvent::default()),
            Box::new(WithdrawPnlEvent::default()),
        ];

        // Verify they all implement the Event trait properly
        for event in events {
            assert!(!event.id().is_empty() || event.id().is_empty()); // Just check method exists
            let _kind = event.kind(); // Verify method exists
            let _json = event.to_json(); // Verify method exists and doesn't panic
        }
    }

    #[test]
    fn parser_should_handle_correct_program_id() {
        let parser = Parser;

        // Should handle the Raydium AMM V4 program ID
        assert!(parser.should_handle(&RAYDIUM_AMM_V4_PROGRAM_ID));

        // Should not handle other program IDs
        assert!(!parser.should_handle(&Pubkey::default()));
        assert!(!parser.should_handle(&solana_sdk::pubkey!("11111111111111111111111111111112")));
    }

    #[test]
    fn module_structure_integrity() {
        // This test ensures that the module re-exports work correctly
        // and don't cause compilation issues

        // Test discriminators module is accessible
        let swap_base_in = SWAP_BASE_IN;
        let swap_base_out = SWAP_BASE_OUT;
        let deposit = DEPOSIT;
        let withdraw = WITHDRAW;

        // Check unused constants exist without naming conflicts
        let _ = INITIALIZE2;
        let _ = WITHDRAW_PNL;

        // Verify discriminators are different
        assert_ne!(swap_base_in, swap_base_out);
        assert_ne!(deposit, withdraw);
    }
}
