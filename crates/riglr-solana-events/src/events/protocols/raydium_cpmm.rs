/// Raydium CPMM event definitions and structures.
// Import section
use crate::solana_metadata::SolanaEventMetadata;
use borsh::{BorshDeserialize, BorshSerialize};
use core::any::Any;
use riglr_events_core::error::EventResult;
use riglr_events_core::traits::EventFilter;
use riglr_events_core::EventMetadata as CoreEventMetadata;
use riglr_events_core::{Event, EventKind};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::time::SystemTime;

// Parser-specific imports
use riglr_events_core::traits::{EventParser as CoreEventParser, ParserInfo};
use std::collections::HashMap;

use crate::error::ParseResult;
use crate::events::{
    factory::{InnerInstructionParseParams, InstructionParseParams, SolanaTransactionInput},
    parser_types::{GenericEventParseConfig, GenericEventParser, ProtocolParser},
};
use crate::types::{EventType, ProtocolType};

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

use solana_message::compiled_instruction::CompiledInstruction;
use solana_transaction_status::UiCompiledInstruction;

/// Raydium CPMM program ID
pub const RAYDIUM_CPMM_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");

/// Raydium CPMM Swap event
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[non_exhaustive]
pub struct SwapEvent {
    /// Amount of input tokens swapped
    pub amount_in: u64,
    /// Amount of output tokens received
    pub amount_out: u64,
    /// Public key of the input token account
    pub input_token_account: Pubkey,
    /// Public key of the input token mint
    pub input_token_mint: Pubkey,
    /// Public key of the input token vault
    pub input_vault: Pubkey,
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: SolanaEventMetadata,
    /// Public key of the output token account
    pub output_token_account: Pubkey,
    /// Public key of the output token mint
    pub output_token_mint: Pubkey,
    /// Public key of the output token vault
    pub output_vault: Pubkey,
    /// Public key of the account that initiated the swap
    pub payer: Pubkey,
    /// Public key of the CPMM pool state account
    pub pool_state: Pubkey,
    /// Trading fee amount
    pub trade_fee: u64,
    /// Transfer fee amount
    pub transfer_fee: u64,
}

// Event trait implementation for SwapEvent
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

    fn matches_filter(&self, _filter: &dyn EventFilter) -> bool
    where
        Self: Sized,
    {
        true
    }

    #[inline]
    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    #[inline]
    fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
    }

    fn source(&self) -> &'static str {
        "raydium_cpmm"
    }

    fn timestamp(&self) -> SystemTime {
        self.metadata.core.timestamp.into()
    }
    #[inline]
    fn to_json(&self) -> EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

/// Raydium CPMM Deposit event
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[non_exhaustive]
pub struct DepositEvent {
    /// Amount of LP tokens minted
    pub lp_token_amount: u64,
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: SolanaEventMetadata,
    /// Public key of the CPMM pool state account
    pub pool_state: Pubkey,
    /// Amount of token 0 deposited
    pub token_0_amount: u64,
    /// Amount of token 1 deposited
    pub token_1_amount: u64,
    /// Public key of the user depositing liquidity
    pub user: Pubkey,
}

// Event trait implementation for DepositEvent
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

    fn matches_filter(&self, _filter: &dyn EventFilter) -> bool
    where
        Self: Sized,
    {
        true
    }

    #[inline]
    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    #[inline]
    fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
    }

    fn source(&self) -> &'static str {
        "raydium_cpmm"
    }

    fn timestamp(&self) -> SystemTime {
        self.metadata.core.timestamp.into()
    }
    #[inline]
    fn to_json(&self) -> EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

// Custom Default implementations with correct EventKind
impl Default for SwapEvent {
    #[inline]
    fn default() -> Self {
        use chrono::{DateTime, Utc};
        use riglr_events_core::EventMetadata;

        // Use a fixed timestamp for reproducible tests
        let fixed_timestamp = DateTime::from_timestamp(0, 0).unwrap_or_else(Utc::now);
        let mut core = EventMetadata::new(String::default(), EventKind::Swap, "solana".to_owned());
        core.timestamp = fixed_timestamp;
        core.received_at = fixed_timestamp; // Use same fixed timestamp for received_at

        let metadata = SolanaEventMetadata::new(
            String::default(),         // signature
            0,                         // slot
            EventType::Swap,           // event_type
            ProtocolType::RaydiumCpmm, // protocol_type
            String::default(),         // index
            0,                         // program_received_time_ms
            core,
        );

        Self {
            amount_in: 0,
            amount_out: 0,
            input_token_account: Pubkey::default(),
            input_token_mint: Pubkey::default(),
            input_vault: Pubkey::default(),
            metadata,
            output_token_account: Pubkey::default(),
            output_token_mint: Pubkey::default(),
            output_vault: Pubkey::default(),
            payer: Pubkey::default(),
            pool_state: Pubkey::default(),
            trade_fee: 0,
            transfer_fee: 0,
        }
    }
}
impl Default for DepositEvent {
    #[inline]
    fn default() -> Self {
        use chrono::{DateTime, Utc};
        use riglr_events_core::EventMetadata;

        // Use a fixed timestamp for reproducible tests
        let fixed_timestamp = DateTime::from_timestamp(0, 0).unwrap_or_else(Utc::now);
        let mut core =
            EventMetadata::new(String::default(), EventKind::Liquidity, "solana".to_owned());
        core.timestamp = fixed_timestamp;
        core.received_at = fixed_timestamp; // Use same fixed timestamp for received_at

        let metadata = SolanaEventMetadata::new(
            String::default(),         // signature
            0,                         // slot
            EventType::AddLiquidity,   // event_type
            ProtocolType::RaydiumCpmm, // protocol_type
            String::default(),         // index
            0,                         // program_received_time_ms
            core,
        );

        Self {
            lp_token_amount: 0,
            metadata,
            pool_state: Pubkey::default(),
            token_0_amount: 0,
            token_1_amount: 0,
            user: Pubkey::default(),
        }
    }
}

/// Raydium CPMM event parser
#[derive(Debug)]
pub struct EventParser {
    /// Parser information
    info: ParserInfo,
    /// Inner generic event parser
    inner: GenericEventParser,
}
impl Default for EventParser {
    #[inline]
    fn default() -> Self {
        let configs = vec![
            GenericEventParseConfig {
                program_id: RAYDIUM_CPMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumCpmm,
                inner_instruction_discriminator: discriminators::SWAP_EVENT,
                instruction_discriminator: discriminators::SWAP_BASE_INPUT_IX,
                event_type: EventType::RaydiumCpmmSwapBaseInput,
                inner_instruction_parser: Self::parse_swap_inner_instruction,
                instruction_parser: Self::parse_swap_base_input_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CPMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumCpmm,
                inner_instruction_discriminator: discriminators::SWAP_EVENT,
                instruction_discriminator: discriminators::SWAP_BASE_OUTPUT_IX,
                event_type: EventType::RaydiumCpmmSwapBaseOutput,
                inner_instruction_parser: Self::parse_swap_inner_instruction,
                instruction_parser: Self::parse_swap_base_output_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CPMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumCpmm,
                inner_instruction_discriminator: discriminators::DEPOSIT_EVENT,
                instruction_discriminator: discriminators::DEPOSIT_IX,
                event_type: EventType::RaydiumCpmmDeposit,
                inner_instruction_parser: Self::parse_deposit_inner_instruction,
                instruction_parser: Self::parse_deposit_instruction,
            },
        ];

        let inner = GenericEventParser::new(vec![RAYDIUM_CPMM_PROGRAM_ID], configs);
        let info = ParserInfo::new("raydium_cpmm_parser".to_owned(), "1.0.0".to_owned())
            .with_kind(riglr_events_core::EventKind::Custom(
                "raydium_cpmm_swap".to_owned(),
            ))
            .with_kind(riglr_events_core::EventKind::Custom(
                "raydium_cpmm_deposit".to_owned(),
            ))
            .with_format("solana_instruction".to_owned());
        Self { info, inner }
    }
}
impl EventParser {
    /// Create a new `EventParser`
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    fn parse_deposit_inner_instruction(
        data: &'_ [u8],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        use crate::error::Error as ParseError;

        // Parse the deposit event using borsh deserialization
        let event = DepositEvent::try_from_slice(data).map_err(|_| {
            ParseError::InvalidDataFormat("Failed to parse Raydium CPMM deposit event".to_owned())
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-deposit",
            metadata.signature, event.pool_state
        ));
        Ok(Box::new(DepositEvent { metadata, ..event }))
    }
    fn parse_deposit_instruction(
        data: &'_ [u8],
        accounts: &'_ [Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        use crate::error::Error as ParseError;

        if data.len() < 24 || accounts.len() < 8 {
            return Err(ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CPMM deposit instruction".to_owned(),
            ));
        }

        let lp_token_amount = u64::from_le_bytes(
            data.get(0..8)
                .ok_or_else(|| {
                    ParseError::InvalidDataFormat("Data too short for lp_token_amount".to_owned())
                })?
                .try_into()
                .map_err(|_| {
                    ParseError::InvalidDataFormat("Failed to read lp_token_amount".to_owned())
                })?,
        );
        let token_0_amount = u64::from_le_bytes(
            data.get(8..16)
                .ok_or_else(|| {
                    ParseError::InvalidDataFormat("Data too short for token_0_amount".to_owned())
                })?
                .try_into()
                .map_err(|_| {
                    ParseError::InvalidDataFormat("Failed to read token_0_amount".to_owned())
                })?,
        );
        let token_1_amount = u64::from_le_bytes(
            data.get(16..24)
                .ok_or_else(|| {
                    ParseError::InvalidDataFormat("Data too short for token_1_amount".to_owned())
                })?
                .try_into()
                .map_err(|_| {
                    ParseError::InvalidDataFormat("Failed to read token_1_amount".to_owned())
                })?,
        );

        let mut metadata = metadata;
        let pool_account = accounts
            .first()
            .ok_or_else(|| ParseError::InvalidDataFormat("Missing pool account".to_owned()))?;
        metadata.set_id(format!(
            "{}-{}-deposit-{}",
            metadata.signature, pool_account, lp_token_amount
        ));

        let user_account = accounts
            .get(1)
            .ok_or_else(|| ParseError::InvalidDataFormat("Missing user account".to_owned()))?;

        Ok(Box::new(DepositEvent {
            lp_token_amount,
            metadata,
            pool_state: *pool_account,
            token_0_amount,
            token_1_amount,
            user: *user_account,
        }))
    }
    fn parse_swap_base_input_instruction(
        data: &'_ [u8],
        accounts: &'_ [Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        use crate::error::Error as ParseError;

        if data.len() < 16 || accounts.len() < 10 {
            return Err(ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CPMM swap base input instruction"
                    .to_owned(),
            ));
        }

        let amount_in = u64::from_le_bytes(
            data.get(0..8)
                .ok_or_else(|| {
                    ParseError::InvalidDataFormat("Data too short for amount_in".to_owned())
                })?
                .try_into()
                .map_err(|_| {
                    ParseError::InvalidDataFormat("Failed to read amount_in".to_owned())
                })?,
        );
        let minimum_amount_out = u64::from_le_bytes(
            data.get(8..16)
                .ok_or_else(|| {
                    ParseError::InvalidDataFormat(
                        "Data too short for minimum_amount_out".to_owned(),
                    )
                })?
                .try_into()
                .map_err(|_| {
                    ParseError::InvalidDataFormat("Failed to read minimum_amount_out".to_owned())
                })?,
        );

        let mut metadata = metadata;
        let pool_account = accounts
            .first()
            .ok_or_else(|| ParseError::InvalidDataFormat("Missing pool account".to_owned()))?;
        metadata.set_id(format!(
            "{}-{}-swap-{}-{}",
            metadata.signature, pool_account, amount_in, minimum_amount_out
        ));

        let payer = accounts
            .get(1)
            .ok_or_else(|| ParseError::InvalidDataFormat("Missing payer account".to_owned()))?;
        let input_token_account = accounts.get(2).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing input token account".to_owned())
        })?;
        let output_token_account = accounts.get(3).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing output token account".to_owned())
        })?;
        let input_vault = accounts
            .get(4)
            .ok_or_else(|| ParseError::InvalidDataFormat("Missing input vault".to_owned()))?;
        let output_vault = accounts
            .get(5)
            .ok_or_else(|| ParseError::InvalidDataFormat("Missing output vault".to_owned()))?;
        let input_token_mint = accounts
            .get(6)
            .ok_or_else(|| ParseError::InvalidDataFormat("Missing input token mint".to_owned()))?;
        let output_token_mint = accounts
            .get(7)
            .ok_or_else(|| ParseError::InvalidDataFormat("Missing output token mint".to_owned()))?;

        Ok(Box::new(SwapEvent {
            amount_in,
            amount_out: 0, // Will be filled by log parsing
            input_token_account: *input_token_account,
            input_token_mint: *input_token_mint,
            input_vault: *input_vault,
            metadata,
            output_token_account: *output_token_account,
            output_token_mint: *output_token_mint,
            output_vault: *output_vault,
            payer: *payer,
            pool_state: *pool_account,
            trade_fee: 0,
            transfer_fee: 0,
        }))
    }
    fn parse_swap_inner_instruction(
        data: &'_ [u8],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        use crate::error::Error as ParseError;

        // Parse the swap event using borsh deserialization
        let event = SwapEvent::try_from_slice(data).map_err(|_| {
            ParseError::InvalidDataFormat("Failed to parse Raydium CPMM swap event".to_owned())
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-swap", metadata.signature, event.pool_state));
        Ok(Box::new(SwapEvent { metadata, ..event }))
    }
    fn parse_swap_base_output_instruction(
        data: &'_ [u8],
        accounts: &'_ [Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        use crate::error::Error as ParseError;

        if data.len() < 16 || accounts.len() < 10 {
            return Err(ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CPMM swap base output instruction"
                    .to_owned(),
            ));
        }

        let max_amount_in = u64::from_le_bytes(
            data.get(0..8)
                .ok_or_else(|| {
                    ParseError::InvalidDataFormat("Data too short for max_amount_in".to_owned())
                })?
                .try_into()
                .map_err(|_| {
                    ParseError::InvalidDataFormat("Failed to read max_amount_in".to_owned())
                })?,
        );
        let amount_out = u64::from_le_bytes(
            data.get(8..16)
                .ok_or_else(|| {
                    ParseError::InvalidDataFormat("Data too short for amount_out".to_owned())
                })?
                .try_into()
                .map_err(|_| {
                    ParseError::InvalidDataFormat("Failed to read amount_out".to_owned())
                })?,
        );

        let mut metadata = metadata;
        let pool_account = accounts
            .first()
            .ok_or_else(|| ParseError::InvalidDataFormat("Missing pool account".to_owned()))?;
        metadata.set_id(format!(
            "{}-{}-swap-{}-{}",
            metadata.signature, pool_account, max_amount_in, amount_out
        ));

        let payer = accounts
            .get(1)
            .ok_or_else(|| ParseError::InvalidDataFormat("Missing payer account".to_owned()))?;
        let input_token_account = accounts.get(2).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing input token account".to_owned())
        })?;
        let output_token_account = accounts.get(3).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing output token account".to_owned())
        })?;
        let input_vault = accounts
            .get(4)
            .ok_or_else(|| ParseError::InvalidDataFormat("Missing input vault".to_owned()))?;
        let output_vault = accounts
            .get(5)
            .ok_or_else(|| ParseError::InvalidDataFormat("Missing output vault".to_owned()))?;
        let input_token_mint = accounts
            .get(6)
            .ok_or_else(|| ParseError::InvalidDataFormat("Missing input token mint".to_owned()))?;
        let output_token_mint = accounts
            .get(7)
            .ok_or_else(|| ParseError::InvalidDataFormat("Missing output token mint".to_owned()))?;

        Ok(Box::new(SwapEvent {
            amount_in: 0, // Will be filled by log parsing
            amount_out,
            input_token_account: *input_token_account,
            input_token_mint: *input_token_mint,
            input_vault: *input_vault,
            metadata,
            output_token_account: *output_token_account,
            output_token_mint: *output_token_mint,
            output_vault: *output_vault,
            payer: *payer,
            pool_state: *pool_account,
            trade_fee: 0,
            transfer_fee: 0,
        }))
    }
}

// Implement the new core EventParser trait
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
                    inner_instruction: &UiCompiledInstruction {
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

// Keep legacy implementation for backward compatibility

#[async_trait::async_trait]
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use riglr_events_core::{Event, EventKind};
    use std::collections::HashMap;

    #[test]
    fn module_re_exports_events() {
        // Test that events module items are re-exported correctly
        let _swap_event = SwapEvent::default();
        let _deposit_event = DepositEvent::default();
    }

    #[test]
    fn module_re_exports_parser() {
        // Test that parser module items are re-exported correctly
        let _parser = EventParser::new();
        let _: &Pubkey = &RAYDIUM_CPMM_PROGRAM_ID;
    }

    #[test]
    fn module_re_exports_discriminators() {
        // Test that discriminator constants are re-exported correctly
        let swap_event_str = discriminators::SWAP_EVENT;
        let deposit_event_str = discriminators::DEPOSIT_EVENT;
        let swap_event_bytes = discriminators::SWAP_EVENT_BYTES;
        let deposit_event_bytes = discriminators::DEPOSIT_EVENT_BYTES;
        let swap_base_input_ix = discriminators::SWAP_BASE_INPUT_IX;
        let swap_base_output_ix = discriminators::SWAP_BASE_OUTPUT_IX;
        let deposit_ix = discriminators::DEPOSIT_IX;
        let initialize_ix = discriminators::INITIALIZE_IX;
        let withdraw_ix = discriminators::WITHDRAW_IX;

        // Verify discriminator values
        assert_eq!(swap_event_str, "raydium_cpmm_swap_event");
        assert_eq!(deposit_event_str, "raydium_cpmm_deposit_event");
        assert_eq!(swap_event_bytes.len(), 16);
        assert_eq!(deposit_event_bytes.len(), 16);
        assert_eq!(swap_base_input_ix.len(), 8);
        assert_eq!(swap_base_output_ix.len(), 8);
        assert_eq!(deposit_ix.len(), 8);
        assert_eq!(initialize_ix.len(), 8);
        assert_eq!(withdraw_ix.len(), 8);
    }

    #[test]
    fn raydium_cpmm_swap_event_default() {
        // Test SwapEvent default implementation
        let event = SwapEvent::default();
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
    fn raydium_cpmm_swap_event_trait_implementations() {
        // Test Event trait implementation for SwapEvent
        let event = SwapEvent::default();

        // Test id method
        assert_eq!(event.id(), "");

        // Test kind method
        assert_eq!(*event.kind(), EventKind::Swap);

        // Test metadata method
        let metadata = event.metadata();
        assert_eq!(metadata.id, "");
        assert_eq!(metadata.kind, EventKind::Swap);
        assert_eq!(metadata.source, "solana");
        assert!(metadata.chain_data.is_none());
        assert_eq!(metadata.custom, HashMap::default());

        // Test as_any method
        let any_ref = event.as_any();
        assert!(any_ref.is::<SwapEvent>());

        // Test clone_boxed method
        let boxed_event = event.clone_boxed();
        assert!(boxed_event.as_any().is::<SwapEvent>());

        // Test clone trait
        let cloned_event = event.clone();
        assert_eq!(cloned_event, event);
    }

    #[test]
    // Test validates error recovery path
    fn raydium_cpmm_swap_event_metadata_mut_should_work() {
        // Test that metadata_mut works correctly
        let mut event = SwapEvent::default();
        let metadata = event
            .metadata_mut()
            .expect("Failed to get mutable metadata for swap event");
        metadata.id = "test-cpmm-swap-id".to_owned();
        assert_eq!(event.metadata().id, "test-cpmm-swap-id");
    }

    #[test]
    fn raydium_cpmm_swap_event_as_any_mut() {
        // Test as_any_mut method
        let mut event = SwapEvent::default();
        let any_mut_ref = event.as_any_mut();
        assert!(any_mut_ref.is::<SwapEvent>());
    }

    #[test]
    fn raydium_cpmm_deposit_event_default() {
        // Test DepositEvent default implementation
        let event = DepositEvent::default();
        assert_eq!(event.pool_state, Pubkey::default());
        assert_eq!(event.user, Pubkey::default());
        assert_eq!(event.lp_token_amount, 0);
        assert_eq!(event.token_0_amount, 0);
        assert_eq!(event.token_1_amount, 0);
    }

    #[test]
    fn raydium_cpmm_deposit_event_trait_implementations() {
        // Test Event trait implementation for DepositEvent
        let event = DepositEvent::default();

        // Test id method
        assert_eq!(event.id(), "");

        // Test kind method
        assert_eq!(*event.kind(), EventKind::Liquidity);

        // Test metadata method
        let metadata = event.metadata();
        assert_eq!(metadata.id, "");
        assert_eq!(metadata.kind, EventKind::Liquidity);
        assert_eq!(metadata.source, "solana");
        assert!(metadata.chain_data.is_none());
        assert_eq!(metadata.custom, HashMap::default());

        // Test as_any method
        let any_ref = event.as_any();
        assert!(any_ref.is::<DepositEvent>());

        // Test clone_boxed method
        let boxed_event = event.clone_boxed();
        assert!(boxed_event.as_any().is::<DepositEvent>());

        // Test clone trait
        let cloned_event = event.clone();
        assert_eq!(cloned_event, event);
    }

    #[test]
    // Test validates error recovery path
    fn raydium_cpmm_deposit_event_metadata_mut_should_work() {
        // Test that metadata_mut works correctly
        let mut event = DepositEvent::default();
        let metadata = event
            .metadata_mut()
            .expect("Failed to get mutable metadata for deposit event");
        metadata.id = "test-cpmm-deposit-id".to_owned();
        assert_eq!(event.metadata().id, "test-cpmm-deposit-id");
    }

    #[test]
    fn raydium_cpmm_deposit_event_as_any_mut() {
        // Test as_any_mut method
        let mut event = DepositEvent::default();
        let any_mut_ref = event.as_any_mut();
        assert!(any_mut_ref.is::<DepositEvent>());
    }

    #[test]
    fn raydium_cpmm_event_parser_new() {
        // Test EventParser::new()
        let parser = EventParser::new();

        // Test should_handle method
        assert!(parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
        assert!(!parser.should_handle(&Pubkey::default()));

        // Test supported_program_ids method
        let supported_ids = parser.supported_program_ids();
        assert!(supported_ids.contains(&RAYDIUM_CPMM_PROGRAM_ID));
        assert_eq!(supported_ids.len(), 1);
    }

    #[test]
    fn raydium_cpmm_event_parser_default() {
        // Test EventParser::default()
        let parser = EventParser::default();

        // Test that default and new produce equivalent results
        let new_parser = EventParser::new();
        assert_eq!(
            parser.supported_program_ids(),
            new_parser.supported_program_ids()
        );
    }

    #[test]
    fn raydium_cpmm_event_parser_configs() {
        // Test parser configuration methods
        let parser = EventParser::new();

        // Test inner_instruction_configs
        let inner_configs = parser.inner_instruction_configs();
        assert!(inner_configs.contains_key(discriminators::SWAP_EVENT));
        assert!(inner_configs.contains_key(discriminators::DEPOSIT_EVENT));

        // Test instruction_configs
        let instruction_configs = parser.instruction_configs();
        assert!(instruction_configs.contains_key(discriminators::SWAP_BASE_INPUT_IX));
        assert!(instruction_configs.contains_key(discriminators::SWAP_BASE_OUTPUT_IX));
        assert!(instruction_configs.contains_key(discriminators::DEPOSIT_IX));
    }

    #[test]
    fn program_id_constant() {
        // Test that the RAYDIUM_CPMM_PROGRAM_ID constant is correct
        let expected_program_id =
            solana_sdk::pubkey!("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");
        assert_eq!(RAYDIUM_CPMM_PROGRAM_ID, expected_program_id);
    }

    #[test]
    fn swap_event_with_custom_values() {
        // Test SwapEvent with custom values
        let pool_state = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let input_token_account = Pubkey::new_unique();
        let output_token_account = Pubkey::new_unique();
        let input_vault = Pubkey::new_unique();
        let output_vault = Pubkey::new_unique();
        let input_token_mint = Pubkey::new_unique();
        let output_token_mint = Pubkey::new_unique();

        let event = SwapEvent {
            pool_state,
            payer,
            input_token_account,
            output_token_account,
            input_vault,
            output_vault,
            input_token_mint,
            output_token_mint,
            amount_in: 1000,
            amount_out: 900,
            trade_fee: 10,
            transfer_fee: 5,
            ..Default::default()
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
        assert_eq!(event.amount_out, 900);
        assert_eq!(event.trade_fee, 10);
        assert_eq!(event.transfer_fee, 5);
    }

    #[test]
    fn deposit_event_with_custom_values() {
        // Test DepositEvent with custom values
        let pool_state = Pubkey::new_unique();
        let user = Pubkey::new_unique();

        let event = DepositEvent {
            pool_state,
            user,
            lp_token_amount: 1000,
            token_0_amount: 500,
            token_1_amount: 600,
            ..Default::default()
        };

        assert_eq!(event.pool_state, pool_state);
        assert_eq!(event.user, user);
        assert_eq!(event.lp_token_amount, 1000);
        assert_eq!(event.token_0_amount, 500);
        assert_eq!(event.token_1_amount, 600);
    }

    #[test]
    fn event_partial_eq() {
        // Test PartialEq implementation for events
        let event1 = SwapEvent::default();
        let event2 = SwapEvent::default();
        assert_eq!(event1, event2);

        let event3 = SwapEvent {
            amount_in: 100,
            ..Default::default()
        };
        assert_ne!(event1, event3);

        let deposit_event1 = DepositEvent::default();
        let deposit_event2 = DepositEvent::default();
        assert_eq!(deposit_event1, deposit_event2);

        let deposit_event3 = DepositEvent {
            lp_token_amount: 100,
            ..Default::default()
        };
        assert_ne!(deposit_event1, deposit_event3);
    }

    #[test]
    fn event_debug_format() {
        // Test Debug implementation for events
        let swap_event = SwapEvent::default();
        let debug_str = format!("{swap_event:?}");
        assert!(debug_str.contains("SwapEvent"));

        let deposit_event = DepositEvent::default();
        let debug_str = format!("{deposit_event:?}");
        assert!(debug_str.contains("DepositEvent"));
    }
}
