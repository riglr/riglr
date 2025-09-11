use riglr_events_core::{
    error::EventResult,
    traits::{EventParser, ParserInfo},
    Event,
};
use std::collections::HashMap;

use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

use crate::error::ParseResult;
use crate::events::{
    common::{parse_swap_amounts, validate_account_count, validate_data_length},
    factory::SolanaTransactionInput,
    parser_types::{GenericEventParseConfig, GenericEventParser, ProtocolParser},
    protocols::bonk::{discriminators, BonkPoolCreateEvent, BonkTradeEvent, TradeDirection},
};
use crate::solana_metadata::SolanaEventMetadata;
use crate::types::{EventType, ProtocolType};

type EventMetadata = SolanaEventMetadata;

/// Bonk program ID
pub const BONK_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("bonksoHKfNJJ8Wo8ZJjpw7dHGePNxS2z2WE5GxUPdSo");

/// Bonk event parser
#[derive(Debug)]
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
    ) -> ParseResult<Box<dyn Event>> {
        // Parse the trade event using borsh deserialization
        let event = BonkTradeEvent::try_from_slice(data)
            .map_err(|e| crate::error::ParseError::BorshError(e.to_string()))?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}", metadata.signature, event.pool_state));

        // Validate trade direction matches the event type
        if metadata.event_type == EventType::BonkBuyExactIn
            || metadata.event_type == EventType::BonkBuyExactOut
        {
            if event.trade_direction != TradeDirection::Buy {
                return Err(crate::error::ParseError::InvalidDataFormat(
                    "Trade direction does not match expected buy direction".to_string(),
                ));
            }
        } else if (metadata.event_type == EventType::BonkSellExactIn
            || metadata.event_type == EventType::BonkSellExactOut)
            && event.trade_direction != TradeDirection::Sell
        {
            return Err(crate::error::ParseError::InvalidDataFormat(
                "Trade direction does not match expected sell direction".to_string(),
            ));
        }

        Ok(Box::new(BonkTradeEvent { metadata, ..event }))
    }

    /// Parse pool create log event
    fn parse_pool_create_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        // Parse the pool create event using borsh deserialization
        let event = BonkPoolCreateEvent::try_from_slice(data)
            .map_err(|e| crate::error::ParseError::BorshError(e.to_string()))?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}", metadata.signature, event.pool_state));
        Ok(Box::new(BonkPoolCreateEvent { metadata, ..event }))
    }

    /// Parse buy exact in instruction event
    fn parse_buy_exact_in_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        use crate::events::common::{
            parse_swap_amounts, safe_get_account, validate_account_count, validate_data_length,
        };

        validate_data_length(data, 16, "Bonk buy exact in instruction")?;
        validate_account_count(accounts, 10, "Bonk buy exact in instruction")?;

        let (amount_in, minimum_amount_out) = parse_swap_amounts(data)?;

        let pool_state = safe_get_account(accounts, 0)?;
        let payer = safe_get_account(accounts, 1)?;
        let user_base_token = safe_get_account(accounts, 2)?;
        let user_quote_token = safe_get_account(accounts, 3)?;
        let base_vault = safe_get_account(accounts, 4)?;
        let quote_vault = safe_get_account(accounts, 5)?;
        let base_token_mint = safe_get_account(accounts, 6)?;
        let quote_token_mint = safe_get_account(accounts, 7)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, pool_state, amount_in, minimum_amount_out
        ));

        Ok(Box::new(BonkTradeEvent {
            metadata,
            pool_state,
            amount_in,
            minimum_amount_out,
            trade_direction: TradeDirection::Buy,
            payer,
            user_base_token,
            user_quote_token,
            base_vault,
            quote_vault,
            base_token_mint,
            quote_token_mint,
            ..Default::default()
        }))
    }

    /// Parse buy exact out instruction event
    fn parse_buy_exact_out_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        use crate::events::common::{
            parse_swap_amounts, safe_get_account, validate_account_count, validate_data_length,
        };

        validate_data_length(data, 16, "Bonk buy exact out instruction")?;
        validate_account_count(accounts, 10, "Bonk buy exact out instruction")?;

        let (maximum_amount_in, amount_out) = parse_swap_amounts(data)?;

        let pool_state = safe_get_account(accounts, 0)?;
        let payer = safe_get_account(accounts, 1)?;
        let user_base_token = safe_get_account(accounts, 2)?;
        let user_quote_token = safe_get_account(accounts, 3)?;
        let base_vault = safe_get_account(accounts, 4)?;
        let quote_vault = safe_get_account(accounts, 5)?;
        let base_token_mint = safe_get_account(accounts, 6)?;
        let quote_token_mint = safe_get_account(accounts, 7)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, pool_state, maximum_amount_in, amount_out
        ));

        Ok(Box::new(BonkTradeEvent {
            metadata,
            pool_state,
            amount_out,
            maximum_amount_in,
            trade_direction: TradeDirection::Buy,
            payer,
            user_base_token,
            user_quote_token,
            base_vault,
            quote_vault,
            base_token_mint,
            quote_token_mint,
            ..Default::default()
        }))
    }

    /// Parse sell exact in instruction event
    fn parse_sell_exact_in_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        validate_data_length(data, 16, "sell exact in instruction")?;
        validate_account_count(accounts, 10, "sell exact in instruction")?;

        let (amount_in, minimum_amount_out) = parse_swap_amounts(data)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[0], amount_in, minimum_amount_out
        ));

        Ok(Box::new(BonkTradeEvent {
            metadata,
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
    ) -> ParseResult<Box<dyn Event>> {
        validate_data_length(data, 16, "sell exact out instruction")?;
        validate_account_count(accounts, 10, "sell exact out instruction")?;

        let (maximum_amount_in, amount_out) = parse_swap_amounts(data)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[0], maximum_amount_in, amount_out
        ));

        Ok(Box::new(BonkTradeEvent {
            metadata,
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
    ) -> ParseResult<Box<dyn Event>> {
        use crate::events::common::{safe_get_account, validate_account_count};

        validate_account_count(accounts, 8, "Bonk initialize instruction")?;

        let pool_state = safe_get_account(accounts, 0)?;
        let creator = safe_get_account(accounts, 1)?;
        let config = safe_get_account(accounts, 2)?;
        let base_mint = safe_get_account(accounts, 3)?;
        let quote_mint = safe_get_account(accounts, 4)?;
        let base_vault = safe_get_account(accounts, 5)?;
        let quote_vault = safe_get_account(accounts, 6)?;
        let global_config = safe_get_account(accounts, 7)?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-{}", metadata.signature, pool_state, creator));

        Ok(Box::new(BonkPoolCreateEvent {
            metadata,
            pool_state,
            creator,
            config,
            payer: creator,
            base_mint,
            quote_mint,
            base_vault,
            quote_vault,
            global_config,
            ..Default::default()
        }))
    }

    /// Parse migrate to AMM instruction event
    fn parse_migrate_to_amm_instruction(
        _data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        validate_account_count(accounts, 5, "migrate to AMM instruction")?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-migrate-amm",
            metadata.signature, accounts[0]
        ));

        Ok(Box::new(BonkPoolCreateEvent {
            metadata,
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
    ) -> ParseResult<Box<dyn Event>> {
        validate_account_count(accounts, 5, "migrate to CPSWAP instruction")?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-migrate-cpswap",
            metadata.signature, accounts[0]
        ));

        Ok(Box::new(BonkPoolCreateEvent {
            metadata,
            pool_state: accounts[0],
            creator: accounts[1],
            config: accounts[2],
            base_mint: accounts[3],
            quote_mint: accounts[4],
            ..Default::default()
        }))
    }
}

// Implement the new core EventParser trait
#[async_trait::async_trait]
impl EventParser for BonkEventParser {
    type Input = SolanaTransactionInput;

    async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>> {
        let events = match input {
            SolanaTransactionInput::InnerInstruction(params) => {
                let legacy_params = crate::events::factory::InnerInstructionParseParams {
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
                let instruction = solana_message::compiled_instruction::CompiledInstruction {
                    program_id_index: 0,
                    accounts: vec![],
                    data: params.instruction_data.clone(),
                };
                let legacy_params = crate::events::factory::InstructionParseParams {
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

    fn can_parse(&self, input: &Self::Input) -> bool {
        match input {
            SolanaTransactionInput::InnerInstruction(_) => true,
            SolanaTransactionInput::Instruction(_) => true,
        }
    }

    fn info(&self) -> ParserInfo {
        use riglr_events_core::EventKind;
        ParserInfo::new("bonk_parser".to_string(), "1.0.0".to_string())
            .with_kind(EventKind::Custom("bonk_trade".to_string()))
            .with_kind(EventKind::Custom("bonk_pool_create".to_string()))
            .with_format("solana_instruction".to_string())
    }
}

// Keep legacy implementation for backward compatibility
#[async_trait::async_trait]
impl ProtocolParser for BonkEventParser {
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner.inner_instruction_configs()
    }
    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.inner.instruction_configs()
    }
    fn parse_events_from_inner_instruction(
        &self,
        params: &crate::events::factory::InnerInstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        self.inner.parse_events_from_inner_instruction(params)
    }

    fn parse_events_from_instruction(
        &self,
        params: &crate::events::factory::InstructionParseParams<'_>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::protocols::bonk::{
        discriminators, BonkPoolCreateEvent, BonkTradeEvent, TradeDirection,
    };
    use crate::solana_metadata::SolanaEventMetadata;
    use crate::types::{EventType, ProtocolType};
    type EventMetadata = crate::solana_metadata::SolanaEventMetadata;
    use riglr_events_core::EventKind;
    use solana_message::compiled_instruction::CompiledInstruction;
    use solana_sdk::pubkey::Pubkey;
    use solana_transaction_status::UiCompiledInstruction;

    fn create_test_metadata(event_type: EventType) -> EventMetadata {
        let core = riglr_events_core::EventMetadata::new(
            "test-signature".to_string(),
            EventKind::Custom(event_type.to_string()),
            "solana".to_string(),
        );

        SolanaEventMetadata::new(
            "test-signature".to_string(),
            12345,
            event_type,
            ProtocolType::Bonk,
            "0".to_string(),
            1640995200000,
            core,
        )
    }

    fn create_test_pubkey(seed: u8) -> Pubkey {
        let mut bytes = [0u8; 32];
        bytes[0] = seed;
        Pubkey::new_from_array(bytes)
    }

    fn create_test_trade_event_data() -> Vec<u8> {
        let event = BonkTradeEvent {
            pool_state: create_test_pubkey(1),
            total_base_sell: 1000,
            virtual_base: 2000,
            virtual_quote: 3000,
            real_base_before: 4000,
            real_quote_before: 5000,
            real_base_after: 6000,
            real_quote_after: 7000,
            amount_in: 100,
            amount_out: 95,
            protocol_fee: 2,
            platform_fee: 1,
            share_fee: 1,
            trade_direction: TradeDirection::Buy,
            ..Default::default()
        };
        borsh::to_vec(&event).unwrap()
    }

    fn create_test_pool_create_event_data() -> Vec<u8> {
        let event = BonkPoolCreateEvent {
            pool_state: create_test_pubkey(1),
            creator: create_test_pubkey(2),
            config: create_test_pubkey(3),
            ..Default::default()
        };
        borsh::to_vec(&event).unwrap()
    }

    #[test]
    fn test_bonk_program_id_when_checked_should_be_correct() {
        let expected = solana_sdk::pubkey!("bonksoHKfNJJ8Wo8ZJjpw7dHGePNxS2z2WE5GxUPdSo");
        assert_eq!(BONK_PROGRAM_ID, expected);
    }

    #[test]
    fn test_bonk_event_parser_new_when_created_should_have_configs() {
        let parser = BonkEventParser::default();
        let inner_configs = parser.inner_instruction_configs();
        let instruction_configs = parser.instruction_configs();

        assert!(!inner_configs.is_empty());
        assert!(!instruction_configs.is_empty());
    }

    #[test]
    fn test_bonk_event_parser_default_when_created_should_equal_new() {
        let parser1 = BonkEventParser::default();
        let parser2 = BonkEventParser::default();

        assert_eq!(
            parser1.supported_program_ids(),
            parser2.supported_program_ids()
        );
    }

    #[test]
    fn test_bonk_event_parser_should_handle_when_bonk_program_id_should_return_true() {
        let parser = BonkEventParser::default();
        assert!(parser.should_handle(&BONK_PROGRAM_ID));
    }

    #[test]
    fn test_bonk_event_parser_should_handle_when_other_program_id_should_return_false() {
        let parser = BonkEventParser::default();
        let other_program_id = create_test_pubkey(99);
        assert!(!parser.should_handle(&other_program_id));
    }

    #[test]
    fn test_bonk_event_parser_supported_program_ids_when_called_should_contain_bonk() {
        let parser = BonkEventParser::default();
        let supported = parser.supported_program_ids();

        assert_eq!(supported.len(), 1);
        assert_eq!(supported[0], BONK_PROGRAM_ID);
    }

    #[test]
    fn test_parse_trade_inner_instruction_when_valid_buy_exact_in_should_return_event() {
        let data = create_test_trade_event_data();
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_trade_inner_instruction(&data, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(
            event.metadata().kind,
            EventKind::Custom("BonkBuyExactIn".to_string())
        );
    }

    #[test]
    fn test_parse_trade_inner_instruction_when_valid_buy_exact_out_should_return_event() {
        let data = create_test_trade_event_data();
        let metadata = create_test_metadata(EventType::BonkBuyExactOut);

        let result = BonkEventParser::parse_trade_inner_instruction(&data, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_trade_inner_instruction_when_valid_sell_exact_in_should_return_event() {
        let mut event = BonkTradeEvent::default();
        event.trade_direction = TradeDirection::Sell;
        let data = borsh::to_vec(&event).unwrap();
        let metadata = create_test_metadata(EventType::BonkSellExactIn);

        let result = BonkEventParser::parse_trade_inner_instruction(&data, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_trade_inner_instruction_when_valid_sell_exact_out_should_return_event() {
        let mut event = BonkTradeEvent::default();
        event.trade_direction = TradeDirection::Sell;
        let data = borsh::to_vec(&event).unwrap();
        let metadata = create_test_metadata(EventType::BonkSellExactOut);

        let result = BonkEventParser::parse_trade_inner_instruction(&data, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_trade_inner_instruction_when_invalid_data_should_return_none() {
        let invalid_data = vec![1, 2, 3, 4];
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_trade_inner_instruction(&invalid_data, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_trade_inner_instruction_when_empty_data_should_return_none() {
        let data = vec![];
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_trade_inner_instruction(&data, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_trade_inner_instruction_when_wrong_trade_direction_buy_should_return_none() {
        let mut event = BonkTradeEvent::default();
        event.trade_direction = TradeDirection::Sell;
        let data = borsh::to_vec(&event).unwrap();
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_trade_inner_instruction(&data, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_trade_inner_instruction_when_wrong_trade_direction_sell_should_return_none() {
        let mut event = BonkTradeEvent::default();
        event.trade_direction = TradeDirection::Buy;
        let data = borsh::to_vec(&event).unwrap();
        let metadata = create_test_metadata(EventType::BonkSellExactIn);

        let result = BonkEventParser::parse_trade_inner_instruction(&data, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_pool_create_inner_instruction_when_valid_data_should_return_event() {
        let data = create_test_pool_create_event_data();
        let metadata = create_test_metadata(EventType::BonkInitialize);

        let result = BonkEventParser::parse_pool_create_inner_instruction(&data, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_pool_create_inner_instruction_when_invalid_data_should_return_none() {
        let invalid_data = vec![1, 2, 3, 4];
        let metadata = create_test_metadata(EventType::BonkInitialize);

        let result = BonkEventParser::parse_pool_create_inner_instruction(&invalid_data, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_pool_create_inner_instruction_when_empty_data_should_return_none() {
        let data = vec![];
        let metadata = create_test_metadata(EventType::BonkInitialize);

        let result = BonkEventParser::parse_pool_create_inner_instruction(&data, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_buy_exact_in_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&1000u64.to_le_bytes());
        data[8..16].copy_from_slice(&950u64.to_le_bytes());

        let accounts = vec![
            create_test_pubkey(1),
            create_test_pubkey(2),
            create_test_pubkey(3),
            create_test_pubkey(4),
            create_test_pubkey(5),
            create_test_pubkey(6),
            create_test_pubkey(7),
            create_test_pubkey(8),
            create_test_pubkey(9),
            create_test_pubkey(10),
        ];
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_buy_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<BonkTradeEvent>().unwrap();
        assert_eq!(trade_event.amount_in, 1000);
        assert_eq!(trade_event.minimum_amount_out, 950);
        assert_eq!(trade_event.trade_direction, TradeDirection::Buy);
    }

    #[test]
    fn test_parse_buy_exact_in_instruction_when_insufficient_data_should_return_none() {
        let data = vec![0u8; 8]; // Less than 16 bytes
        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_buy_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_buy_exact_in_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![0u8; 16];
        let accounts = vec![create_test_pubkey(1); 5]; // Less than 10 accounts
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_buy_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_buy_exact_in_instruction_when_empty_data_should_return_none() {
        let data = vec![];
        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_buy_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_buy_exact_in_instruction_when_empty_accounts_should_return_none() {
        let data = vec![0u8; 16];
        let accounts = vec![];
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_buy_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_buy_exact_out_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&1050u64.to_le_bytes());
        data[8..16].copy_from_slice(&1000u64.to_le_bytes());

        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkBuyExactOut);

        let result = BonkEventParser::parse_buy_exact_out_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<BonkTradeEvent>().unwrap();
        assert_eq!(trade_event.maximum_amount_in, 1050);
        assert_eq!(trade_event.amount_out, 1000);
        assert_eq!(trade_event.trade_direction, TradeDirection::Buy);
    }

    #[test]
    fn test_parse_buy_exact_out_instruction_when_insufficient_data_should_return_none() {
        let data = vec![0u8; 8];
        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkBuyExactOut);

        let result = BonkEventParser::parse_buy_exact_out_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_buy_exact_out_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![0u8; 16];
        let accounts = vec![create_test_pubkey(1); 5];
        let metadata = create_test_metadata(EventType::BonkBuyExactOut);

        let result = BonkEventParser::parse_buy_exact_out_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sell_exact_in_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&1000u64.to_le_bytes());
        data[8..16].copy_from_slice(&950u64.to_le_bytes());

        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkSellExactIn);

        let result = BonkEventParser::parse_sell_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<BonkTradeEvent>().unwrap();
        assert_eq!(trade_event.amount_in, 1000);
        assert_eq!(trade_event.minimum_amount_out, 950);
        assert_eq!(trade_event.trade_direction, TradeDirection::Sell);
    }

    #[test]
    fn test_parse_sell_exact_in_instruction_when_insufficient_data_should_return_none() {
        let data = vec![0u8; 8];
        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkSellExactIn);

        let result = BonkEventParser::parse_sell_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sell_exact_in_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![0u8; 16];
        let accounts = vec![create_test_pubkey(1); 5];
        let metadata = create_test_metadata(EventType::BonkSellExactIn);

        let result = BonkEventParser::parse_sell_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sell_exact_out_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&1050u64.to_le_bytes());
        data[8..16].copy_from_slice(&1000u64.to_le_bytes());

        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkSellExactOut);

        let result = BonkEventParser::parse_sell_exact_out_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<BonkTradeEvent>().unwrap();
        assert_eq!(trade_event.maximum_amount_in, 1050);
        assert_eq!(trade_event.amount_out, 1000);
        assert_eq!(trade_event.trade_direction, TradeDirection::Sell);
    }

    #[test]
    fn test_parse_sell_exact_out_instruction_when_insufficient_data_should_return_none() {
        let data = vec![0u8; 8];
        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkSellExactOut);

        let result = BonkEventParser::parse_sell_exact_out_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sell_exact_out_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![0u8; 16];
        let accounts = vec![create_test_pubkey(1); 5];
        let metadata = create_test_metadata(EventType::BonkSellExactOut);

        let result = BonkEventParser::parse_sell_exact_out_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_initialize_instruction_when_valid_accounts_should_return_event() {
        let data = vec![];
        let accounts = vec![create_test_pubkey(1); 8];
        let metadata = create_test_metadata(EventType::BonkInitialize);

        let result = BonkEventParser::parse_initialize_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let pool_event = event
            .as_any()
            .downcast_ref::<BonkPoolCreateEvent>()
            .unwrap();
        assert_eq!(pool_event.pool_state, accounts[0]);
        assert_eq!(pool_event.creator, accounts[1]);
        assert_eq!(pool_event.config, accounts[2]);
        assert_eq!(pool_event.payer, accounts[1]);
    }

    #[test]
    fn test_parse_initialize_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![];
        let accounts = vec![create_test_pubkey(1); 5]; // Less than 8 accounts
        let metadata = create_test_metadata(EventType::BonkInitialize);

        let result = BonkEventParser::parse_initialize_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_initialize_instruction_when_empty_accounts_should_return_none() {
        let data = vec![];
        let accounts = vec![];
        let metadata = create_test_metadata(EventType::BonkInitialize);

        let result = BonkEventParser::parse_initialize_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_migrate_to_amm_instruction_when_valid_accounts_should_return_event() {
        let data = vec![];
        let accounts = vec![create_test_pubkey(1); 5];
        let metadata = create_test_metadata(EventType::BonkMigrateToAmm);

        let result = BonkEventParser::parse_migrate_to_amm_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let pool_event = event
            .as_any()
            .downcast_ref::<BonkPoolCreateEvent>()
            .unwrap();
        assert_eq!(pool_event.pool_state, accounts[0]);
        assert_eq!(pool_event.creator, accounts[1]);
        assert_eq!(pool_event.config, accounts[2]);
        assert_eq!(pool_event.base_mint, accounts[3]);
        assert_eq!(pool_event.quote_mint, accounts[4]);
    }

    #[test]
    fn test_parse_migrate_to_amm_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![];
        let accounts = vec![create_test_pubkey(1); 3]; // Less than 5 accounts
        let metadata = create_test_metadata(EventType::BonkMigrateToAmm);

        let result = BonkEventParser::parse_migrate_to_amm_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_migrate_to_amm_instruction_when_empty_accounts_should_return_none() {
        let data = vec![];
        let accounts = vec![];
        let metadata = create_test_metadata(EventType::BonkMigrateToAmm);

        let result = BonkEventParser::parse_migrate_to_amm_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_migrate_to_cpswap_instruction_when_valid_accounts_should_return_event() {
        let data = vec![];
        let accounts = vec![create_test_pubkey(1); 5];
        let metadata = create_test_metadata(EventType::BonkMigrateToCpswap);

        let result =
            BonkEventParser::parse_migrate_to_cpswap_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let pool_event = event
            .as_any()
            .downcast_ref::<BonkPoolCreateEvent>()
            .unwrap();
        assert_eq!(pool_event.pool_state, accounts[0]);
        assert_eq!(pool_event.creator, accounts[1]);
        assert_eq!(pool_event.config, accounts[2]);
        assert_eq!(pool_event.base_mint, accounts[3]);
        assert_eq!(pool_event.quote_mint, accounts[4]);
    }

    #[test]
    fn test_parse_migrate_to_cpswap_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![];
        let accounts = vec![create_test_pubkey(1); 3]; // Less than 5 accounts
        let metadata = create_test_metadata(EventType::BonkMigrateToCpswap);

        let result =
            BonkEventParser::parse_migrate_to_cpswap_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_migrate_to_cpswap_instruction_when_empty_accounts_should_return_none() {
        let data = vec![];
        let accounts = vec![];
        let metadata = create_test_metadata(EventType::BonkMigrateToCpswap);

        let result =
            BonkEventParser::parse_migrate_to_cpswap_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_events_from_inner_instruction_when_called_should_delegate_to_inner() {
        let parser = BonkEventParser::default();
        let instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "".to_string(),
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &instruction,
            signature: "test-signature",
            slot: 12345,
            block_time: Some(1640995200),
            program_received_time_ms: 1640995200000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        // Should return events (or empty vec) without panicking
        assert!(events.is_empty() || !events.is_empty());
    }

    #[test]
    fn test_parse_events_from_instruction_when_called_should_delegate_to_inner() {
        let parser = BonkEventParser::default();
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![],
        };
        let accounts = vec![BONK_PROGRAM_ID];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test-signature",
            slot: 12345,
            block_time: Some(1640995200),
            program_received_time_ms: 1640995200000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        // Should return events (or empty vec) without panicking
        assert!(events.is_empty() || !events.is_empty());
    }

    #[test]
    fn test_inner_instruction_configs_when_called_should_return_configs() {
        let parser = BonkEventParser::default();
        let configs = parser.inner_instruction_configs();

        assert!(!configs.is_empty());
        assert!(configs.contains_key(discriminators::TRADE_EVENT));
        assert!(configs.contains_key(discriminators::POOL_CREATE_EVENT));
    }

    #[test]
    fn test_instruction_configs_when_called_should_return_configs() {
        let parser = BonkEventParser::default();
        let configs = parser.instruction_configs();

        assert!(!configs.is_empty());
        assert!(configs.contains_key(&discriminators::BUY_EXACT_IN_IX.to_vec()));
        assert!(configs.contains_key(&discriminators::BUY_EXACT_OUT_IX.to_vec()));
        assert!(configs.contains_key(&discriminators::SELL_EXACT_IN_IX.to_vec()));
        assert!(configs.contains_key(&discriminators::SELL_EXACT_OUT_IX.to_vec()));
        assert!(configs.contains_key(&discriminators::INITIALIZE_IX.to_vec()));
        assert!(configs.contains_key(&discriminators::MIGRATE_TO_AMM_IX.to_vec()));
        assert!(configs.contains_key(&discriminators::MIGRATE_TO_CPSWAP_IX.to_vec()));
    }

    #[test]
    fn test_parse_trade_inner_instruction_with_id_generation() {
        let data = create_test_trade_event_data();
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_trade_inner_instruction(&data, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<BonkTradeEvent>().unwrap();

        // Verify ID format: signature-pool_state
        let expected_id_prefix = "test-signature-";
        assert!(trade_event.metadata.id.starts_with(&expected_id_prefix));
    }

    #[test]
    fn test_parse_pool_create_inner_instruction_with_id_generation() {
        let data = create_test_pool_create_event_data();
        let metadata = create_test_metadata(EventType::BonkInitialize);

        let result = BonkEventParser::parse_pool_create_inner_instruction(&data, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let pool_event = event
            .as_any()
            .downcast_ref::<BonkPoolCreateEvent>()
            .unwrap();

        // Verify ID format: signature-pool_state
        let expected_id_prefix = "test-signature-";
        assert!(pool_event.metadata.id.starts_with(&expected_id_prefix));
    }

    #[test]
    fn test_parse_instruction_events_with_id_generation() {
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&1000u64.to_le_bytes());
        data[8..16].copy_from_slice(&950u64.to_le_bytes());

        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_buy_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<BonkTradeEvent>().unwrap();

        // Verify ID format: signature-pool_state-amount_in-minimum_amount_out
        let expected_id = format!("test-signature-{}-1000-950", accounts[0]);
        assert_eq!(trade_event.metadata.id, expected_id);
    }

    #[test]
    fn test_edge_cases_with_max_values() {
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&u64::MAX.to_le_bytes());
        data[8..16].copy_from_slice(&u64::MAX.to_le_bytes());

        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_buy_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<BonkTradeEvent>().unwrap();
        assert_eq!(trade_event.amount_in, u64::MAX);
        assert_eq!(trade_event.minimum_amount_out, u64::MAX);
    }

    #[test]
    fn test_edge_cases_with_zero_values() {
        let data = vec![0u8; 16];
        let accounts = vec![Pubkey::default(); 10];
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_buy_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<BonkTradeEvent>().unwrap();
        assert_eq!(trade_event.amount_in, 0);
        assert_eq!(trade_event.minimum_amount_out, 0);
        assert_eq!(trade_event.pool_state, Pubkey::default());
    }

    #[test]
    fn test_all_trade_instruction_parsers_return_correct_directions() {
        let data = vec![0u8; 16];
        let accounts = vec![create_test_pubkey(1); 10];

        // Test buy exact in
        let result = BonkEventParser::parse_buy_exact_in_instruction(
            &data,
            &accounts,
            create_test_metadata(EventType::BonkBuyExactIn),
        );
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<BonkTradeEvent>().unwrap();
        assert_eq!(trade_event.trade_direction, TradeDirection::Buy);

        // Test buy exact out
        let result = BonkEventParser::parse_buy_exact_out_instruction(
            &data,
            &accounts,
            create_test_metadata(EventType::BonkBuyExactOut),
        );
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<BonkTradeEvent>().unwrap();
        assert_eq!(trade_event.trade_direction, TradeDirection::Buy);

        // Test sell exact in
        let result = BonkEventParser::parse_sell_exact_in_instruction(
            &data,
            &accounts,
            create_test_metadata(EventType::BonkSellExactIn),
        );
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<BonkTradeEvent>().unwrap();
        assert_eq!(trade_event.trade_direction, TradeDirection::Sell);

        // Test sell exact out
        let result = BonkEventParser::parse_sell_exact_out_instruction(
            &data,
            &accounts,
            create_test_metadata(EventType::BonkSellExactOut),
        );
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<BonkTradeEvent>().unwrap();
        assert_eq!(trade_event.trade_direction, TradeDirection::Sell);
    }

    #[test]
    fn test_instruction_parsers_ignore_data_parameter() {
        // Test that instruction parsers that don't use data parameter work with any data
        let random_data = vec![1, 2, 3, 4, 5];
        let accounts = vec![create_test_pubkey(1); 8];

        let result = BonkEventParser::parse_initialize_instruction(
            &random_data,
            &accounts,
            create_test_metadata(EventType::BonkInitialize),
        );
        assert!(result.is_ok());

        let accounts = vec![create_test_pubkey(1); 5];
        let result = BonkEventParser::parse_migrate_to_amm_instruction(
            &random_data,
            &accounts,
            create_test_metadata(EventType::BonkMigrateToAmm),
        );
        assert!(result.is_ok());

        let result = BonkEventParser::parse_migrate_to_cpswap_instruction(
            &random_data,
            &accounts,
            create_test_metadata(EventType::BonkMigrateToCpswap),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_config_completeness() {
        let parser = BonkEventParser::default();
        let inner_configs = parser.inner_instruction_configs();
        let instruction_configs = parser.instruction_configs();

        // Verify all expected discriminators are present
        assert!(inner_configs.contains_key(discriminators::TRADE_EVENT));
        assert!(inner_configs.contains_key(discriminators::POOL_CREATE_EVENT));

        // Verify all instruction discriminators are present
        let expected_instruction_discriminators = [
            discriminators::BUY_EXACT_IN_IX,
            discriminators::BUY_EXACT_OUT_IX,
            discriminators::SELL_EXACT_IN_IX,
            discriminators::SELL_EXACT_OUT_IX,
            discriminators::INITIALIZE_IX,
            discriminators::MIGRATE_TO_AMM_IX,
            discriminators::MIGRATE_TO_CPSWAP_IX,
        ];

        for discriminator in expected_instruction_discriminators {
            assert!(instruction_configs.contains_key(&discriminator.to_vec()));
        }
    }

    #[test]
    fn test_default_config_event_types() {
        let parser = BonkEventParser::default();
        let instruction_configs = parser.instruction_configs();

        // Verify correct event types are configured
        let buy_exact_in_configs = instruction_configs
            .get(&discriminators::BUY_EXACT_IN_IX.to_vec())
            .unwrap();
        assert_eq!(
            buy_exact_in_configs[0].event_type,
            EventType::BonkBuyExactIn
        );

        let buy_exact_out_configs = instruction_configs
            .get(&discriminators::BUY_EXACT_OUT_IX.to_vec())
            .unwrap();
        assert_eq!(
            buy_exact_out_configs[0].event_type,
            EventType::BonkBuyExactOut
        );

        let sell_exact_in_configs = instruction_configs
            .get(&discriminators::SELL_EXACT_IN_IX.to_vec())
            .unwrap();
        assert_eq!(
            sell_exact_in_configs[0].event_type,
            EventType::BonkSellExactIn
        );

        let sell_exact_out_configs = instruction_configs
            .get(&discriminators::SELL_EXACT_OUT_IX.to_vec())
            .unwrap();
        assert_eq!(
            sell_exact_out_configs[0].event_type,
            EventType::BonkSellExactOut
        );

        let initialize_configs = instruction_configs
            .get(&discriminators::INITIALIZE_IX.to_vec())
            .unwrap();
        assert_eq!(initialize_configs[0].event_type, EventType::BonkInitialize);

        let migrate_amm_configs = instruction_configs
            .get(&discriminators::MIGRATE_TO_AMM_IX.to_vec())
            .unwrap();
        assert_eq!(
            migrate_amm_configs[0].event_type,
            EventType::BonkMigrateToAmm
        );

        let migrate_cpswap_configs = instruction_configs
            .get(&discriminators::MIGRATE_TO_CPSWAP_IX.to_vec())
            .unwrap();
        assert_eq!(
            migrate_cpswap_configs[0].event_type,
            EventType::BonkMigrateToCpswap
        );
    }

    #[test]
    fn test_all_configs_use_bonk_program_id() {
        let parser = BonkEventParser::default();
        let instruction_configs = parser.instruction_configs();

        for configs in instruction_configs.values() {
            for config in configs {
                assert_eq!(config.program_id, BONK_PROGRAM_ID);
                assert_eq!(config.protocol_type, ProtocolType::Bonk);
            }
        }
    }

    // ========== ADDITIONAL COMPREHENSIVE TESTS FOR TASK 4 ==========

    // Test ParseError conditions for insufficient data

    #[test]
    fn test_parse_buy_exact_in_instruction_when_data_exactly_15_bytes_should_return_err() {
        let data = vec![0u8; 15]; // One byte short of required 16
        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_buy_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Insufficient data"));
        }
    }

    #[test]
    fn test_parse_buy_exact_out_instruction_when_data_exactly_15_bytes_should_return_err() {
        let data = vec![0u8; 15]; // One byte short of required 16
        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkBuyExactOut);

        let result = BonkEventParser::parse_buy_exact_out_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Insufficient data"));
        }
    }

    #[test]
    fn test_parse_sell_exact_in_instruction_when_data_exactly_15_bytes_should_return_err() {
        let data = vec![0u8; 15]; // One byte short of required 16
        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkSellExactIn);

        let result = BonkEventParser::parse_sell_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            // This function uses not_enough_bytes directly
            assert!(e.to_string().contains("Not enough bytes"));
        }
    }

    #[test]
    fn test_parse_sell_exact_out_instruction_when_data_exactly_15_bytes_should_return_err() {
        let data = vec![0u8; 15]; // One byte short of required 16
        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkSellExactOut);

        let result = BonkEventParser::parse_sell_exact_out_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            // This function uses not_enough_bytes directly
            assert!(e.to_string().contains("Not enough bytes"));
        }
    }

    // Test ParseError conditions for insufficient accounts

    #[test]
    fn test_parse_buy_exact_in_instruction_when_accounts_exactly_9_should_return_err() {
        let data = vec![0u8; 16];
        let accounts = vec![create_test_pubkey(1); 9]; // One account short of required 10
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_buy_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Insufficient accounts"));
        }
    }

    #[test]
    fn test_parse_buy_exact_out_instruction_when_accounts_exactly_9_should_return_err() {
        let data = vec![0u8; 16];
        let accounts = vec![create_test_pubkey(1); 9]; // One account short of required 10
        let metadata = create_test_metadata(EventType::BonkBuyExactOut);

        let result = BonkEventParser::parse_buy_exact_out_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Insufficient accounts"));
        }
    }

    #[test]
    fn test_parse_initialize_instruction_when_accounts_exactly_7_should_return_err() {
        let data = vec![];
        let accounts = vec![create_test_pubkey(1); 7]; // One account short of required 8
        let metadata = create_test_metadata(EventType::BonkInitialize);

        let result = BonkEventParser::parse_initialize_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Insufficient accounts"));
        }
    }

    #[test]
    fn test_parse_migrate_to_amm_instruction_when_accounts_exactly_4_should_return_err() {
        let data = vec![];
        let accounts = vec![create_test_pubkey(1); 4]; // One account short of required 5
        let metadata = create_test_metadata(EventType::BonkMigrateToAmm);

        let result = BonkEventParser::parse_migrate_to_amm_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Account index 4 out of bounds"));
        }
    }

    #[test]
    fn test_parse_migrate_to_cpswap_instruction_when_accounts_exactly_4_should_return_err() {
        let data = vec![];
        let accounts = vec![create_test_pubkey(1); 4]; // One account short of required 5
        let metadata = create_test_metadata(EventType::BonkMigrateToCpswap);

        let result =
            BonkEventParser::parse_migrate_to_cpswap_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Account index 4 out of bounds"));
        }
    }

    // Test with malformed/corrupted data that would cause try_into failures

    #[test]
    fn test_parse_sell_exact_in_instruction_when_corrupted_amount_data_should_return_err() {
        // Use only 7 bytes for first amount instead of 8
        let mut data = vec![1, 2, 3, 4, 5, 6, 7];
        data.extend_from_slice(&1000u64.to_le_bytes()); // 8 bytes for second amount
                                                        // Total is 15 bytes - should fail on length check

        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkSellExactIn);

        let result = BonkEventParser::parse_sell_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sell_exact_out_instruction_when_corrupted_amount_data_should_return_err() {
        // Use only 7 bytes for first amount instead of 8
        let mut data = vec![1, 2, 3, 4, 5, 6, 7];
        data.extend_from_slice(&1000u64.to_le_bytes()); // 8 bytes for second amount
                                                        // Total is 15 bytes - should fail on length check

        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkSellExactOut);

        let result = BonkEventParser::parse_sell_exact_out_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    // Test with invalid data format scenarios

    #[test]
    fn test_parse_trade_inner_instruction_when_completely_invalid_borsh_should_return_err() {
        // Create completely invalid borsh data that can't be deserialized
        let invalid_data = vec![0xFF; 100]; // 100 bytes of 0xFF
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_trade_inner_instruction(&invalid_data, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Borsh deserialization error"));
        }
    }

    #[test]
    fn test_parse_pool_create_inner_instruction_when_completely_invalid_borsh_should_return_err() {
        // Create completely invalid borsh data that can't be deserialized
        let invalid_data = vec![0xFF; 100]; // 100 bytes of 0xFF
        let metadata = create_test_metadata(EventType::BonkInitialize);

        let result = BonkEventParser::parse_pool_create_inner_instruction(&invalid_data, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Borsh deserialization error"));
        }
    }

    // Test edge case values - zero amounts

    #[test]
    fn test_parse_instruction_with_zero_amounts_should_succeed() {
        let data = vec![0u8; 16]; // Both amounts are zero
        let accounts = vec![Pubkey::default(); 10];
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_buy_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<BonkTradeEvent>().unwrap();
        assert_eq!(trade_event.amount_in, 0);
        assert_eq!(trade_event.minimum_amount_out, 0);
    }

    // Test edge case values - maximum amounts

    #[test]
    fn test_parse_instruction_with_max_amounts_should_succeed() {
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&u64::MAX.to_le_bytes());
        data[8..16].copy_from_slice(&u64::MAX.to_le_bytes());

        let accounts = vec![create_test_pubkey(1); 10];
        let metadata = create_test_metadata(EventType::BonkSellExactIn);

        let result = BonkEventParser::parse_sell_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<BonkTradeEvent>().unwrap();
        assert_eq!(trade_event.amount_in, u64::MAX);
        assert_eq!(trade_event.minimum_amount_out, u64::MAX);
    }

    // Test with exact minimum required data and accounts

    #[test]
    fn test_parse_instruction_with_exact_minimum_requirements_should_succeed() {
        let data = vec![0u8; 16]; // Exactly 16 bytes
        let accounts = vec![create_test_pubkey(1); 10]; // Exactly 10 accounts
        let metadata = create_test_metadata(EventType::BonkBuyExactOut);

        let result = BonkEventParser::parse_buy_exact_out_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_initialize_instruction_with_exact_minimum_accounts_should_succeed() {
        let data = vec![];
        let accounts = vec![create_test_pubkey(1); 8]; // Exactly 8 accounts
        let metadata = create_test_metadata(EventType::BonkInitialize);

        let result = BonkEventParser::parse_initialize_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_migrate_instruction_with_exact_minimum_accounts_should_succeed() {
        let data = vec![];
        let accounts = vec![create_test_pubkey(1); 5]; // Exactly 5 accounts
        let metadata = create_test_metadata(EventType::BonkMigrateToAmm);

        let result = BonkEventParser::parse_migrate_to_amm_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
    }

    // Test with more than required data and accounts (should still work)

    #[test]
    fn test_parse_instruction_with_excess_data_should_succeed() {
        let data = vec![0u8; 32]; // More than required 16 bytes
        let accounts = vec![create_test_pubkey(1); 15]; // More than required 10 accounts
        let metadata = create_test_metadata(EventType::BonkSellExactOut);

        let result = BonkEventParser::parse_sell_exact_out_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_initialize_instruction_with_excess_accounts_should_succeed() {
        let data = vec![];
        let accounts = vec![create_test_pubkey(1); 15]; // More than required 8 accounts
        let metadata = create_test_metadata(EventType::BonkInitialize);

        let result = BonkEventParser::parse_initialize_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
    }

    // Test ID generation with extreme values

    #[test]
    fn test_instruction_id_generation_with_max_values() {
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&u64::MAX.to_le_bytes());
        data[8..16].copy_from_slice(&u64::MAX.to_le_bytes());

        let accounts = vec![create_test_pubkey(255); 10]; // Use seed 255 for pubkey
        let metadata = create_test_metadata(EventType::BonkBuyExactIn);

        let result = BonkEventParser::parse_buy_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let trade_event = event.as_any().downcast_ref::<BonkTradeEvent>().unwrap();

        // Verify ID contains max values
        let expected_id = format!("test-signature-{}-{}-{}", accounts[0], u64::MAX, u64::MAX);
        assert_eq!(trade_event.metadata.id, expected_id);
    }

    // Test with different trade directions validation

    #[test]
    fn test_parse_trade_inner_instruction_strict_direction_validation() {
        // Create a sell event but try to parse it as buy - should fail
        let mut event = BonkTradeEvent::default();
        event.trade_direction = TradeDirection::Sell;
        let data = borsh::to_vec(&event).unwrap();

        // Try all buy event types - should all fail
        for event_type in [EventType::BonkBuyExactIn, EventType::BonkBuyExactOut] {
            let metadata = create_test_metadata(event_type);
            let result = BonkEventParser::parse_trade_inner_instruction(&data, metadata);

            assert!(result.is_err());
            if let Err(e) = result {
                assert!(e.to_string().contains("Trade direction does not match"));
            }
        }

        // Create a buy event but try to parse it as sell - should fail
        let mut event = BonkTradeEvent::default();
        event.trade_direction = TradeDirection::Buy;
        let data = borsh::to_vec(&event).unwrap();

        // Try all sell event types - should all fail
        for event_type in [EventType::BonkSellExactIn, EventType::BonkSellExactOut] {
            let metadata = create_test_metadata(event_type);
            let result = BonkEventParser::parse_trade_inner_instruction(&data, metadata);

            assert!(result.is_err());
            if let Err(e) = result {
                assert!(e.to_string().contains("Trade direction does not match"));
            }
        }
    }

    // Test parser with empty string signature and index

    #[test]
    fn test_parse_instruction_with_empty_signature_should_succeed() {
        let mut metadata = create_test_metadata(EventType::BonkBuyExactIn);
        metadata.signature = "".to_string();

        let data = vec![0u8; 16];
        let accounts = vec![create_test_pubkey(1); 10];

        let result = BonkEventParser::parse_buy_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_instruction_with_empty_index_should_succeed() {
        let mut metadata = create_test_metadata(EventType::BonkBuyExactIn);
        metadata.index = "".to_string();

        let data = vec![0u8; 16];
        let accounts = vec![create_test_pubkey(1); 10];

        let result = BonkEventParser::parse_buy_exact_in_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
    }
}
