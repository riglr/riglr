use std::collections::HashMap;

use borsh::BorshDeserialize;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;

use crate::events::{
    common::{read_u64_le, EventMetadata, EventType, ProtocolType},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
    protocols::pumpswap::{
        discriminators, PumpSwapBuyEvent, PumpSwapCreatePoolEvent, PumpSwapDepositEvent,
        PumpSwapSellEvent, PumpSwapWithdrawEvent,
    },
};

/// PumpSwap program ID
pub const PUMPSWAP_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");

/// PumpSwap event parser
pub struct PumpSwapEventParser {
    inner: GenericEventParser,
}

impl Default for PumpSwapEventParser {
    fn default() -> Self {
        Self::new()
    }
}

impl PumpSwapEventParser {
    pub fn new() -> Self {
        // Configure all event types
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

        Self { inner }
    }

    /// Parse buy log event
    fn parse_buy_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        // Parse the buy event using borsh deserialization
        if let Ok(event) = PumpSwapBuyEvent::try_from_slice(data) {
            let mut metadata = metadata;
            metadata.set_id(format!("{}-{}-buy", metadata.signature, event.pool));
            Some(Box::new(PumpSwapBuyEvent { metadata, ..event }))
        } else {
            None
        }
    }

    /// Parse sell log event
    fn parse_sell_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        // Parse the sell event using borsh deserialization
        if let Ok(event) = PumpSwapSellEvent::try_from_slice(data) {
            let mut metadata = metadata;
            metadata.set_id(format!("{}-{}-sell", metadata.signature, event.pool));
            Some(Box::new(PumpSwapSellEvent { metadata, ..event }))
        } else {
            None
        }
    }

    /// Parse create pool log event
    fn parse_create_pool_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        // Parse the create pool event using borsh deserialization
        if let Ok(event) = PumpSwapCreatePoolEvent::try_from_slice(data) {
            let mut metadata = metadata;
            metadata.set_id(format!("{}-{}-create", metadata.signature, event.pool));
            Some(Box::new(PumpSwapCreatePoolEvent { metadata, ..event }))
        } else {
            None
        }
    }

    /// Parse deposit log event
    fn parse_deposit_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        // Parse the deposit event using borsh deserialization
        if let Ok(event) = PumpSwapDepositEvent::try_from_slice(data) {
            let mut metadata = metadata;
            metadata.set_id(format!("{}-{}-deposit", metadata.signature, event.pool));
            Some(Box::new(PumpSwapDepositEvent { metadata, ..event }))
        } else {
            None
        }
    }

    /// Parse withdraw log event
    fn parse_withdraw_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        // Parse the withdraw event using borsh deserialization
        if let Ok(event) = PumpSwapWithdrawEvent::try_from_slice(data) {
            let mut metadata = metadata;
            metadata.set_id(format!("{}-{}-withdraw", metadata.signature, event.pool));
            Some(Box::new(PumpSwapWithdrawEvent { metadata, ..event }))
        } else {
            None
        }
    }

    /// Parse buy instruction event
    fn parse_buy_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 11 {
            return None;
        }

        let base_amount_out = read_u64_le(data, 0).ok()?;
        let max_quote_amount_in = read_u64_le(data, 8).ok()?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[1], accounts[0], base_amount_out
        ));

        Some(Box::new(PumpSwapBuyEvent {
            metadata,
            base_amount_out,
            max_quote_amount_in,
            pool: accounts[0],
            user: accounts[1],
            base_mint: accounts[3],
            quote_mint: accounts[4],
            user_base_token_account: accounts[5],
            user_quote_token_account: accounts[6],
            pool_base_token_account: accounts[7],
            pool_quote_token_account: accounts[8],
            protocol_fee_recipient: accounts[9],
            protocol_fee_recipient_token_account: accounts[10],
            coin_creator_vault_ata: accounts.get(17).copied().unwrap_or_default(),
            coin_creator_vault_authority: accounts.get(18).copied().unwrap_or_default(),
            ..Default::default()
        }))
    }

    /// Parse sell instruction event
    fn parse_sell_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 11 {
            return None;
        }

        let base_amount_in = read_u64_le(data, 0).ok()?;
        let min_quote_amount_out = read_u64_le(data, 8).ok()?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[1], accounts[0], base_amount_in
        ));

        Some(Box::new(PumpSwapSellEvent {
            metadata,
            base_amount_in,
            min_quote_amount_out,
            pool: accounts[0],
            user: accounts[1],
            base_mint: accounts[3],
            quote_mint: accounts[4],
            user_base_token_account: accounts[5],
            user_quote_token_account: accounts[6],
            pool_base_token_account: accounts[7],
            pool_quote_token_account: accounts[8],
            protocol_fee_recipient: accounts[9],
            protocol_fee_recipient_token_account: accounts[10],
            coin_creator_vault_ata: accounts.get(17).copied().unwrap_or_default(),
            coin_creator_vault_authority: accounts.get(18).copied().unwrap_or_default(),
            ..Default::default()
        }))
    }

    /// Parse create pool instruction event
    fn parse_create_pool_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 18 || accounts.len() < 11 {
            return None;
        }

        let index = u16::from_le_bytes(data[0..2].try_into().ok()?);
        let base_amount_in = u64::from_le_bytes(data[2..10].try_into().ok()?);
        let quote_amount_in = u64::from_le_bytes(data[10..18].try_into().ok()?);
        let coin_creator = if data.len() >= 50 {
            Pubkey::new_from_array(data[18..50].try_into().ok()?)
        } else {
            Pubkey::default()
        };

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[0], accounts[2], base_amount_in
        ));

        Some(Box::new(PumpSwapCreatePoolEvent {
            metadata,
            index,
            base_amount_in,
            quote_amount_in,
            pool: accounts[0],
            creator: accounts[2],
            base_mint: accounts[3],
            quote_mint: accounts[4],
            lp_mint: accounts[5],
            user_base_token_account: accounts[6],
            user_quote_token_account: accounts[7],
            user_pool_token_account: accounts[8],
            pool_base_token_account: accounts[9],
            pool_quote_token_account: accounts[10],
            coin_creator,
            ..Default::default()
        }))
    }

    /// Parse deposit instruction event
    fn parse_deposit_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 24 || accounts.len() < 11 {
            return None;
        }

        let lp_token_amount_out = u64::from_le_bytes(data[0..8].try_into().ok()?);
        let max_base_amount_in = u64::from_le_bytes(data[8..16].try_into().ok()?);
        let max_quote_amount_in = u64::from_le_bytes(data[16..24].try_into().ok()?);

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[0], accounts[2], lp_token_amount_out
        ));

        Some(Box::new(PumpSwapDepositEvent {
            metadata,
            lp_token_amount_out,
            max_base_amount_in,
            max_quote_amount_in,
            pool: accounts[0],
            user: accounts[2],
            base_mint: accounts[3],
            quote_mint: accounts[4],
            user_base_token_account: accounts[6],
            user_quote_token_account: accounts[7],
            user_pool_token_account: accounts[8],
            pool_base_token_account: accounts[9],
            pool_quote_token_account: accounts[10],
            ..Default::default()
        }))
    }

    /// Parse withdraw instruction event
    fn parse_withdraw_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 24 || accounts.len() < 11 {
            return None;
        }

        let lp_token_amount_in = u64::from_le_bytes(data[0..8].try_into().ok()?);
        let min_base_amount_out = u64::from_le_bytes(data[8..16].try_into().ok()?);
        let min_quote_amount_out = u64::from_le_bytes(data[16..24].try_into().ok()?);

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[0], accounts[2], lp_token_amount_in
        ));

        Some(Box::new(PumpSwapWithdrawEvent {
            metadata,
            lp_token_amount_in,
            min_base_amount_out,
            min_quote_amount_out,
            pool: accounts[0],
            user: accounts[2],
            base_mint: accounts[3],
            quote_mint: accounts[4],
            user_base_token_account: accounts[6],
            user_quote_token_account: accounts[7],
            user_pool_token_account: accounts[8],
            pool_base_token_account: accounts[9],
            pool_quote_token_account: accounts[10],
            ..Default::default()
        }))
    }
}

#[async_trait::async_trait]
impl EventParser for PumpSwapEventParser {
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
    ) -> Vec<Box<dyn UnifiedEvent>> {
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
    ) -> Vec<Box<dyn UnifiedEvent>> {
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