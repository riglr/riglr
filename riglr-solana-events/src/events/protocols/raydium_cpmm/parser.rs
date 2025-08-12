use riglr_events_core::Event;
use std::collections::HashMap;

use borsh::BorshDeserialize;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;

use crate::types::{EventMetadata, EventType, ProtocolType};
use crate::events::{
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser},
    protocols::raydium_cpmm::{discriminators, RaydiumCpmmDepositEvent, RaydiumCpmmSwapEvent},
};

/// Raydium CPMM program ID
pub const RAYDIUM_CPMM_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");

/// Raydium CPMM event parser
pub struct RaydiumCpmmEventParser {
    inner: GenericEventParser,
}

impl Default for RaydiumCpmmEventParser {
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
        Self { inner }
    }
}

impl RaydiumCpmmEventParser {
    /// Creates a new Raydium CPMM event parser with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse swap log event
    fn parse_swap_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn Event>> {
        // Parse the swap event using borsh deserialization
        if let Ok(event) = RaydiumCpmmSwapEvent::try_from_slice(data) {
            let mut metadata = metadata;
            metadata.set_id(format!("{}-{}-swap", metadata.signature, event.pool_state));
            Some(Box::new(RaydiumCpmmSwapEvent { metadata: metadata.core, ..event }))
        } else {
            None
        }
    }

    /// Parse deposit log event
    fn parse_deposit_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn Event>> {
        // Parse the deposit event using borsh deserialization
        if let Ok(event) = RaydiumCpmmDepositEvent::try_from_slice(data) {
            let mut metadata = metadata;
            metadata.set_id(format!(
                "{}-{}-deposit",
                metadata.signature, event.pool_state
            ));
            Some(Box::new(RaydiumCpmmDepositEvent { metadata: metadata.core, ..event }))
        } else {
            None
        }
    }

    /// Parse swap base input instruction event
    fn parse_swap_base_input_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn Event>> {
        if data.len() < 16 || accounts.len() < 10 {
            return None;
        }

        let amount_in = u64::from_le_bytes(data[0..8].try_into().ok()?);
        let minimum_amount_out = u64::from_le_bytes(data[8..16].try_into().ok()?);

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-swap-{}-{}",
            metadata.signature, accounts[0], amount_in, minimum_amount_out
        ));

        Some(Box::new(RaydiumCpmmSwapEvent {
            metadata: metadata.core,
            pool_state: accounts[0],
            payer: accounts[1],
            input_token_account: accounts[2],
            output_token_account: accounts[3],
            input_vault: accounts[4],
            output_vault: accounts[5],
            input_token_mint: accounts[6],
            output_token_mint: accounts[7],
            amount_in,
            amount_out: 0, // Will be filled by log parsing
            ..Default::default()
        }))
    }

    /// Parse swap base output instruction event
    fn parse_swap_base_output_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn Event>> {
        if data.len() < 16 || accounts.len() < 10 {
            return None;
        }

        let max_amount_in = u64::from_le_bytes(data[0..8].try_into().ok()?);
        let amount_out = u64::from_le_bytes(data[8..16].try_into().ok()?);

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-swap-{}-{}",
            metadata.signature, accounts[0], max_amount_in, amount_out
        ));

        Some(Box::new(RaydiumCpmmSwapEvent {
            metadata: metadata.core,
            pool_state: accounts[0],
            payer: accounts[1],
            input_token_account: accounts[2],
            output_token_account: accounts[3],
            input_vault: accounts[4],
            output_vault: accounts[5],
            input_token_mint: accounts[6],
            output_token_mint: accounts[7],
            amount_in: 0, // Will be filled by log parsing
            amount_out,
            ..Default::default()
        }))
    }

    /// Parse deposit instruction event
    fn parse_deposit_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn Event>> {
        if data.len() < 24 || accounts.len() < 8 {
            return None;
        }

        let lp_token_amount = u64::from_le_bytes(data[0..8].try_into().ok()?);
        let token_0_amount = u64::from_le_bytes(data[8..16].try_into().ok()?);
        let token_1_amount = u64::from_le_bytes(data[16..24].try_into().ok()?);

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-deposit-{}",
            metadata.signature, accounts[0], lp_token_amount
        ));

        Some(Box::new(RaydiumCpmmDepositEvent {
            metadata: metadata.core,
            pool_state: accounts[0],
            user: accounts[1],
            lp_token_amount,
            token_0_amount,
            token_1_amount,
        }))
    }
}

#[async_trait::async_trait]
impl EventParser for RaydiumCpmmEventParser {
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner.inner_instruction_configs()
    }
    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.inner.instruction_configs()
    }
    fn parse_events_from_inner_instruction(
        &self,
        inner_instruction: &UiCompiledInstruction,
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn Event>> {
        self.inner.parse_events_from_inner_instruction(
            inner_instruction,
            signature,
            slot,
            block_time,
            program_received_time_ms,
            index,
        )
    }

    fn parse_events_from_instruction(
        &self,
        instruction: &CompiledInstruction,
        accounts: &[Pubkey],
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn Event>> {
        self.inner.parse_events_from_instruction(
            instruction,
            accounts,
            signature,
            slot,
            block_time,
            program_received_time_ms,
            index,
        )
    }

    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.inner.should_handle(program_id)
    }

    fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.inner.supported_program_ids()
    }
}
