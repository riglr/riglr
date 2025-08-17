use riglr_events_core::Event;
use std::collections::HashMap;

use borsh::BorshDeserialize;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;

use crate::types::{EventMetadata, EventType, ProtocolType};
use crate::events::{
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser},
    protocols::bonk::{discriminators, BonkPoolCreateEvent, BonkTradeEvent, TradeDirection},
};

/// Bonk program ID
pub const BONK_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("bonksoHKfNJJ8Wo8ZJjpw7dHGePNxS2z2WE5GxUPdSo");

/// Bonk event parser
pub struct BonkEventParser {
    inner: GenericEventParser,
}

impl Default for BonkEventParser {
    fn default() -> Self {
        // Configure all event types
        let configs = vec![
            GenericEventParseConfig {
                program_id: BONK_PROGRAM_ID,
                protocol_type: ProtocolType::Bonk,
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: discriminators::BUY_EXACT_IN_IX,
                event_type: EventType::BonkBuyExactIn,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_buy_exact_in_instruction,
            },
            GenericEventParseConfig {
                program_id: BONK_PROGRAM_ID,
                protocol_type: ProtocolType::Bonk,
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: discriminators::BUY_EXACT_OUT_IX,
                event_type: EventType::BonkBuyExactOut,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_buy_exact_out_instruction,
            },
            GenericEventParseConfig {
                program_id: BONK_PROGRAM_ID,
                protocol_type: ProtocolType::Bonk,
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: discriminators::SELL_EXACT_IN_IX,
                event_type: EventType::BonkSellExactIn,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_sell_exact_in_instruction,
            },
            GenericEventParseConfig {
                program_id: BONK_PROGRAM_ID,
                protocol_type: ProtocolType::Bonk,
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: discriminators::SELL_EXACT_OUT_IX,
                event_type: EventType::BonkSellExactOut,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_sell_exact_out_instruction,
            },
            GenericEventParseConfig {
                program_id: BONK_PROGRAM_ID,
                protocol_type: ProtocolType::Bonk,
                inner_instruction_discriminator: discriminators::POOL_CREATE_EVENT,
                instruction_discriminator: discriminators::INITIALIZE_IX,
                event_type: EventType::BonkInitialize,
                inner_instruction_parser: Self::parse_pool_create_inner_instruction,
                instruction_parser: Self::parse_initialize_instruction,
            },
            GenericEventParseConfig {
                program_id: BONK_PROGRAM_ID,
                protocol_type: ProtocolType::Bonk,
                inner_instruction_discriminator: discriminators::POOL_CREATE_EVENT,
                instruction_discriminator: discriminators::MIGRATE_TO_AMM_IX,
                event_type: EventType::BonkMigrateToAmm,
                inner_instruction_parser: Self::parse_pool_create_inner_instruction,
                instruction_parser: Self::parse_migrate_to_amm_instruction,
            },
            GenericEventParseConfig {
                program_id: BONK_PROGRAM_ID,
                protocol_type: ProtocolType::Bonk,
                inner_instruction_discriminator: discriminators::POOL_CREATE_EVENT,
                instruction_discriminator: discriminators::MIGRATE_TO_CPSWAP_IX,
                event_type: EventType::BonkMigrateToCpswap,
                inner_instruction_parser: Self::parse_pool_create_inner_instruction,
                instruction_parser: Self::parse_migrate_to_cpswap_instruction,
            },
        ];

        let inner = GenericEventParser::new(vec![BONK_PROGRAM_ID], configs);

        Self { inner }
    }
}

impl BonkEventParser {
    /// Creates a new Bonk event parser with configured discriminators and parsers
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse trade log event
    fn parse_trade_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn Event>> {
        // Parse the trade event using borsh deserialization
        if let Ok(event) = BonkTradeEvent::try_from_slice(data) {
            let mut metadata = metadata;
            metadata.set_id(format!("{}-{}", metadata.signature, event.pool_state));

            // Validate trade direction matches the event type
            if metadata.event_type == EventType::BonkBuyExactIn
                || metadata.event_type == EventType::BonkBuyExactOut
            {
                if event.trade_direction != TradeDirection::Buy {
                    return None;
                }
            } else if (metadata.event_type == EventType::BonkSellExactIn
                || metadata.event_type == EventType::BonkSellExactOut)
                && event.trade_direction != TradeDirection::Sell
            {
                return None;
            }

            Some(Box::new(BonkTradeEvent { metadata, ..event }))
        } else {
            None
        }
    }

    /// Parse pool create log event
    fn parse_pool_create_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn Event>> {
        // Parse the pool create event using borsh deserialization
        if let Ok(event) = BonkPoolCreateEvent::try_from_slice(data) {
            let mut metadata = metadata;
            metadata.set_id(format!("{}-{}", metadata.signature, event.pool_state));
            Some(Box::new(BonkPoolCreateEvent { metadata, ..event }))
        } else {
            None
        }
    }

    /// Parse buy exact in instruction event
    fn parse_buy_exact_in_instruction(
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
            "{}-{}-{}-{}",
            metadata.signature, accounts[0], amount_in, minimum_amount_out
        ));

        Some(Box::new(BonkTradeEvent {
            metadata: metadata.core,
            pool_state: accounts[0],
            amount_in,
            minimum_amount_out,
            trade_direction: TradeDirection::Buy,
            payer: accounts[1],
            user_base_token: accounts[2],
            user_quote_token: accounts[3],
            base_vault: accounts[4],
            quote_vault: accounts[5],
            base_token_mint: accounts[6],
            quote_token_mint: accounts[7],
            ..Default::default()
        }))
    }

    /// Parse buy exact out instruction event
    fn parse_buy_exact_out_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn Event>> {
        if data.len() < 16 || accounts.len() < 10 {
            return None;
        }

        let maximum_amount_in = u64::from_le_bytes(data[0..8].try_into().ok()?);
        let amount_out = u64::from_le_bytes(data[8..16].try_into().ok()?);

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[0], maximum_amount_in, amount_out
        ));

        Some(Box::new(BonkTradeEvent {
            metadata: metadata.core,
            pool_state: accounts[0],
            amount_out,
            maximum_amount_in,
            trade_direction: TradeDirection::Buy,
            payer: accounts[1],
            user_base_token: accounts[2],
            user_quote_token: accounts[3],
            base_vault: accounts[4],
            quote_vault: accounts[5],
            base_token_mint: accounts[6],
            quote_token_mint: accounts[7],
            ..Default::default()
        }))
    }

    /// Parse sell exact in instruction event
    fn parse_sell_exact_in_instruction(
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
            "{}-{}-{}-{}",
            metadata.signature, accounts[0], amount_in, minimum_amount_out
        ));

        Some(Box::new(BonkTradeEvent {
            metadata: metadata.core,
            pool_state: accounts[0],
            amount_in,
            minimum_amount_out,
            trade_direction: TradeDirection::Sell,
            payer: accounts[1],
            user_base_token: accounts[2],
            user_quote_token: accounts[3],
            base_vault: accounts[4],
            quote_vault: accounts[5],
            base_token_mint: accounts[6],
            quote_token_mint: accounts[7],
            ..Default::default()
        }))
    }

    /// Parse sell exact out instruction event
    fn parse_sell_exact_out_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn Event>> {
        if data.len() < 16 || accounts.len() < 10 {
            return None;
        }

        let maximum_amount_in = u64::from_le_bytes(data[0..8].try_into().ok()?);
        let amount_out = u64::from_le_bytes(data[8..16].try_into().ok()?);

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[0], maximum_amount_in, amount_out
        ));

        Some(Box::new(BonkTradeEvent {
            metadata: metadata.core,
            pool_state: accounts[0],
            amount_out,
            maximum_amount_in,
            trade_direction: TradeDirection::Sell,
            payer: accounts[1],
            user_base_token: accounts[2],
            user_quote_token: accounts[3],
            base_vault: accounts[4],
            quote_vault: accounts[5],
            base_token_mint: accounts[6],
            quote_token_mint: accounts[7],
            ..Default::default()
        }))
    }

    /// Parse initialize instruction event
    fn parse_initialize_instruction(
        _data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn Event>> {
        if accounts.len() < 8 {
            return None;
        }

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}",
            metadata.signature, accounts[0], accounts[1]
        ));

        Some(Box::new(BonkPoolCreateEvent {
            metadata: metadata.core,
            pool_state: accounts[0],
            creator: accounts[1],
            config: accounts[2],
            payer: accounts[1],
            base_mint: accounts[3],
            quote_mint: accounts[4],
            base_vault: accounts[5],
            quote_vault: accounts[6],
            global_config: accounts[7],
            ..Default::default()
        }))
    }

    /// Parse migrate to AMM instruction event
    fn parse_migrate_to_amm_instruction(
        _data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn Event>> {
        if accounts.len() < 5 {
            return None;
        }

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-migrate-amm",
            metadata.signature, accounts[0]
        ));

        Some(Box::new(BonkPoolCreateEvent {
            metadata: metadata.core,
            pool_state: accounts[0],
            creator: accounts[1],
            config: accounts[2],
            base_mint: accounts[3],
            quote_mint: accounts[4],
            ..Default::default()
        }))
    }

    /// Parse migrate to CPSWAP instruction event
    fn parse_migrate_to_cpswap_instruction(
        _data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn Event>> {
        if accounts.len() < 5 {
            return None;
        }

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-migrate-cpswap",
            metadata.signature, accounts[0]
        ));

        Some(Box::new(BonkPoolCreateEvent {
            metadata: metadata.core,
            pool_state: accounts[0],
            creator: accounts[1],
            config: accounts[2],
            base_mint: accounts[3],
            quote_mint: accounts[4],
            ..Default::default()
        }))
    }
}

#[async_trait::async_trait]
impl EventParser for BonkEventParser {
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
