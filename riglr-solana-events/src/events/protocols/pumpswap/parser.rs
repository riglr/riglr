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
    common::read_u64_le,
    factory::SolanaTransactionInput,
    parser_types::{GenericEventParseConfig, GenericEventParser, LegacyEventParser},
    protocols::pumpswap::{
        discriminators, PumpSwapBuyEvent, PumpSwapCreatePoolEvent, PumpSwapDepositEvent,
        PumpSwapSellEvent, PumpSwapWithdrawEvent,
    },
};
use crate::solana_metadata::SolanaEventMetadata;
use crate::types::{EventType, ProtocolType};

// Removed unused type alias

/// PumpSwap program ID
pub const PUMPSWAP_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");

/// PumpSwap event parser
pub struct PumpSwapEventParser {
    inner: GenericEventParser,
}

impl Default for PumpSwapEventParser {
    fn default() -> Self {
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
}

impl PumpSwapEventParser {
    /// Creates a new PumpSwap event parser with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    // Removed unused helper methods - functionality available through trait implementation

    /// Parse buy log event as static method
    fn parse_buy_inner_instruction(
        data: &[u8],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        // Parse the buy event using borsh deserialization
        let mut event = PumpSwapBuyEvent::try_from_slice(data).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse PumpSwap buy event".to_string(),
            )
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-buy", metadata.signature, event.pool));
        event.metadata = metadata;
        Ok(Box::new(event))
    }

    /// Parse sell log event as static method
    fn parse_sell_inner_instruction(
        data: &[u8],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        // Parse the sell event using borsh deserialization
        let mut event = PumpSwapSellEvent::try_from_slice(data).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse PumpSwap sell event".to_string(),
            )
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-sell", metadata.signature, event.pool));
        event.metadata = metadata;
        Ok(Box::new(event))
    }

    /// Parse create pool log event as static method
    fn parse_create_pool_inner_instruction(
        data: &[u8],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        // Parse the create pool event using borsh deserialization
        let mut event = PumpSwapCreatePoolEvent::try_from_slice(data).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse PumpSwap create pool event".to_string(),
            )
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-create", metadata.signature, event.pool));
        event.metadata = metadata;
        Ok(Box::new(event))
    }

    /// Parse deposit log event as static method
    fn parse_deposit_inner_instruction(
        data: &[u8],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        // Parse the deposit event using borsh deserialization
        let mut event = PumpSwapDepositEvent::try_from_slice(data).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse PumpSwap deposit event".to_string(),
            )
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-deposit", metadata.signature, event.pool));
        event.metadata = metadata;
        Ok(Box::new(event))
    }

    /// Parse withdraw log event as static method
    fn parse_withdraw_inner_instruction(
        data: &[u8],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        // Parse the withdraw event using borsh deserialization
        let mut event = PumpSwapWithdrawEvent::try_from_slice(data).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse PumpSwap withdraw event".to_string(),
            )
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-withdraw", metadata.signature, event.pool));
        event.metadata = metadata;
        Ok(Box::new(event))
    }

    /// Parse buy instruction event as static method
    fn parse_buy_instruction(
        data: &[u8],
        accounts: &[Pubkey],
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
        let protocol_fee_recipient = safe_get_account(accounts, 9)?;
        let protocol_fee_recipient_token_account = safe_get_account(accounts, 10)?;
        let coin_creator_vault_ata = get_account_or_default(accounts, 17);
        let coin_creator_vault_authority = get_account_or_default(accounts, 18);

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, user, pool, base_amount_out
        ));

        let event = PumpSwapBuyEvent {
            metadata,
            timestamp: 0,
            base_amount_out,
            max_quote_amount_in,
            user_base_token_reserves: 0,
            user_quote_token_reserves: 0,
            pool_base_token_reserves: 0,
            pool_quote_token_reserves: 0,
            quote_amount_in: 0,
            lp_fee_basis_points: 0,
            lp_fee: 0,
            protocol_fee_basis_points: 0,
            protocol_fee: 0,
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
        };
        Ok(Box::new(event))
    }

    /// Parse sell instruction event as static method
    fn parse_sell_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 16 || accounts.len() < 11 {
            return Err(crate::error::ParseError::InvalidDataFormat(
                "Insufficient data or accounts for PumpSwap sell instruction".to_string(),
            ));
        }

        let base_amount_in = read_u64_le(data, 0).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read base_amount_in".to_string())
        })?;
        let min_quote_amount_out = read_u64_le(data, 8).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to read min_quote_amount_out".to_string(),
            )
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[1], accounts[0], base_amount_in
        ));

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
            pool: accounts[0],
            user: accounts[1],
            user_base_token_account: accounts[5],
            user_quote_token_account: accounts[6],
            protocol_fee_recipient: accounts[9],
            protocol_fee_recipient_token_account: accounts[10],
            coin_creator: Pubkey::default(),
            coin_creator_fee_basis_points: 0,
            coin_creator_fee: 0,
            base_mint: accounts[3],
            quote_mint: accounts[4],
            pool_base_token_account: accounts[7],
            pool_quote_token_account: accounts[8],
            coin_creator_vault_ata: accounts.get(17).copied().unwrap_or_default(),
            coin_creator_vault_authority: accounts.get(18).copied().unwrap_or_default(),
        };
        Ok(Box::new(event))
    }

    /// Parse create pool instruction event as static method
    fn parse_create_pool_instruction(
        data: &[u8],
        accounts: &[Pubkey],
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

    /// Parse deposit instruction event as static method
    fn parse_deposit_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 24 || accounts.len() < 11 {
            return Err(crate::error::ParseError::InvalidDataFormat(
                "Insufficient data or accounts for PumpSwap deposit instruction".to_string(),
            ));
        }

        let lp_token_amount_out = u64::from_le_bytes(data[0..8].try_into().map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to read lp_token_amount_out".to_string(),
            )
        })?);
        let max_base_amount_in = u64::from_le_bytes(data[8..16].try_into().map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to read max_base_amount_in".to_string(),
            )
        })?);
        let max_quote_amount_in = u64::from_le_bytes(data[16..24].try_into().map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to read max_quote_amount_in".to_string(),
            )
        })?);

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[0], accounts[2], lp_token_amount_out
        ));

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
            pool: accounts[0],
            user: accounts[2],
            user_base_token_account: accounts[6],
            user_quote_token_account: accounts[7],
            user_pool_token_account: accounts[8],
            base_mint: accounts[3],
            quote_mint: accounts[4],
            pool_base_token_account: accounts[9],
            pool_quote_token_account: accounts[10],
        };
        Ok(Box::new(event))
    }

    /// Parse withdraw instruction event as static method
    fn parse_withdraw_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 24 || accounts.len() < 11 {
            return Err(crate::error::ParseError::InvalidDataFormat(
                "Insufficient data or accounts for PumpSwap withdraw instruction".to_string(),
            ));
        }

        let lp_token_amount_in = u64::from_le_bytes(data[0..8].try_into().map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to read lp_token_amount_in".to_string(),
            )
        })?);
        let min_base_amount_out = u64::from_le_bytes(data[8..16].try_into().map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to read min_base_amount_out".to_string(),
            )
        })?);
        let min_quote_amount_out = u64::from_le_bytes(data[16..24].try_into().map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to read min_quote_amount_out".to_string(),
            )
        })?);

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[0], accounts[2], lp_token_amount_in
        ));

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
            pool: accounts[0],
            user: accounts[2],
            user_base_token_account: accounts[6],
            user_quote_token_account: accounts[7],
            user_pool_token_account: accounts[8],
            base_mint: accounts[3],
            quote_mint: accounts[4],
            pool_base_token_account: accounts[9],
            pool_quote_token_account: accounts[10],
        };
        Ok(Box::new(event))
    }
}

// Implement the new core EventParser trait
#[async_trait::async_trait]
impl EventParser for PumpSwapEventParser {
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
        ParserInfo::new("pumpswap_parser".to_string(), "1.0.0".to_string())
            .with_kind(EventKind::Custom("pumpswap_buy".to_string()))
            .with_kind(EventKind::Custom("pumpswap_sell".to_string()))
            .with_kind(EventKind::Custom("pumpswap_create_pool".to_string()))
            .with_kind(EventKind::Custom("pumpswap_deposit".to_string()))
            .with_kind(EventKind::Custom("pumpswap_withdraw".to_string()))
            .with_format("solana_instruction".to_string())
    }
}

// Keep legacy implementation for backward compatibility
#[async_trait::async_trait]
impl LegacyEventParser for PumpSwapEventParser {
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner.inner_instruction_configs()
    }
    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.inner.instruction_configs()
    }
    fn parse_events_from_inner_instruction(
        &self,
        params: &crate::events::factory::InnerInstructionParseParams,
    ) -> Vec<Box<dyn Event>> {
        self.inner.parse_events_from_inner_instruction(params)
    }

    fn parse_events_from_instruction(
        &self,
        params: &crate::events::factory::InstructionParseParams,
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
    use crate::solana_metadata::SolanaEventMetadata;
    use crate::types::{EventType, ProtocolType};
    use riglr_events_core::{EventKind, EventMetadata as CoreEventMetadata};
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    fn create_test_metadata() -> SolanaEventMetadata {
        let core = CoreEventMetadata::new(
            "test_id".to_string(),
            EventKind::Transaction,
            "test".to_string(),
        );

        SolanaEventMetadata::new(
            "test_signature".to_string(),
            100,
            EventType::PumpSwapBuy,
            ProtocolType::PumpSwap,
            "0".to_string(),
            1640995200000,
            core,
        )
    }

    fn create_test_accounts() -> Vec<Pubkey> {
        (0..20).map(|_| Pubkey::new_unique()).collect()
    }

    #[test]
    fn test_pumpswap_program_id_constant() {
        let expected = Pubkey::from_str("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA").unwrap();
        assert_eq!(PUMPSWAP_PROGRAM_ID, expected);
    }

    #[test]
    fn test_parser_default() {
        let parser = PumpSwapEventParser::default();
        assert!(parser.should_handle(&PUMPSWAP_PROGRAM_ID));
        assert_eq!(parser.supported_program_ids(), vec![PUMPSWAP_PROGRAM_ID]);
    }

    #[test]
    fn test_parser_should_handle_valid_program_id() {
        let parser = PumpSwapEventParser::default();
        assert!(parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    }

    #[test]
    fn test_parser_should_not_handle_invalid_program_id() {
        let parser = PumpSwapEventParser::default();
        let random_id = Pubkey::new_unique();
        assert!(!parser.should_handle(&random_id));
    }

    #[test]
    fn test_parser_inner_instruction_configs() {
        let parser = PumpSwapEventParser::default();
        let configs = parser.inner_instruction_configs();
        assert!(!configs.is_empty());
    }

    #[test]
    fn test_parser_instruction_configs() {
        let parser = PumpSwapEventParser::default();
        let configs = parser.instruction_configs();
        assert!(!configs.is_empty());
    }

    #[test]
    fn test_parse_buy_inner_instruction_when_valid_data_should_return_ok() {
        let metadata = create_test_metadata();

        // Create valid borsh-serialized buy event data
        let test_event = PumpSwapBuyEvent {
            metadata: create_test_metadata(),
            timestamp: 0,
            base_amount_out: 0,
            max_quote_amount_in: 0,
            user_base_token_reserves: 0,
            user_quote_token_reserves: 0,
            pool_base_token_reserves: 0,
            pool_quote_token_reserves: 0,
            quote_amount_in: 0,
            lp_fee_basis_points: 0,
            lp_fee: 0,
            protocol_fee_basis_points: 0,
            protocol_fee: 0,
            quote_amount_in_with_lp_fee: 0,
            user_quote_amount_in: 0,
            pool: Pubkey::new_unique(),
            user: Pubkey::default(),
            user_base_token_account: Pubkey::default(),
            user_quote_token_account: Pubkey::default(),
            protocol_fee_recipient: Pubkey::default(),
            protocol_fee_recipient_token_account: Pubkey::default(),
            coin_creator: Pubkey::default(),
            coin_creator_fee_basis_points: 0,
            coin_creator_fee: 0,
            track_volume: false,
            total_unclaimed_tokens: 0,
            total_claimed_tokens: 0,
            current_sol_volume: 0,
            last_update_timestamp: 0,
            base_mint: Pubkey::default(),
            quote_mint: Pubkey::default(),
            pool_base_token_account: Pubkey::default(),
            pool_quote_token_account: Pubkey::default(),
            coin_creator_vault_ata: Pubkey::default(),
            coin_creator_vault_authority: Pubkey::default(),
        };
        let data = borsh::to_vec(&test_event).unwrap();

        let result = PumpSwapEventParser::parse_buy_inner_instruction(&data, metadata);
        assert!(result.is_ok());

        let event = result.unwrap();
        assert_eq!(event.kind(), &EventKind::Transaction);
    }

    #[test]
    fn test_parse_buy_inner_instruction_when_invalid_data_should_return_err() {
        let metadata = create_test_metadata();
        let invalid_data = vec![0x00, 0x01, 0x02]; // Invalid borsh data

        let result = PumpSwapEventParser::parse_buy_inner_instruction(&invalid_data, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_buy_inner_instruction_when_empty_data_should_return_err() {
        let metadata = create_test_metadata();
        let empty_data = vec![];

        let result = PumpSwapEventParser::parse_buy_inner_instruction(&empty_data, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sell_inner_instruction_when_valid_data_should_return_ok() {
        let metadata = create_test_metadata();

        let test_event = PumpSwapSellEvent {
            metadata: create_test_metadata(),
            timestamp: 0,
            base_amount_in: 0,
            min_quote_amount_out: 0,
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
            pool: Pubkey::new_unique(),
            user: Pubkey::default(),
            user_base_token_account: Pubkey::default(),
            user_quote_token_account: Pubkey::default(),
            protocol_fee_recipient: Pubkey::default(),
            protocol_fee_recipient_token_account: Pubkey::default(),
            coin_creator: Pubkey::default(),
            coin_creator_fee_basis_points: 0,
            coin_creator_fee: 0,
            base_mint: Pubkey::default(),
            quote_mint: Pubkey::default(),
            pool_base_token_account: Pubkey::default(),
            pool_quote_token_account: Pubkey::default(),
            coin_creator_vault_ata: Pubkey::default(),
            coin_creator_vault_authority: Pubkey::default(),
        };
        let data = borsh::to_vec(&test_event).unwrap();

        let result = PumpSwapEventParser::parse_sell_inner_instruction(&data, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_sell_inner_instruction_when_invalid_data_should_return_err() {
        let metadata = create_test_metadata();
        let invalid_data = vec![0xFF, 0xEE, 0xDD];

        let result = PumpSwapEventParser::parse_sell_inner_instruction(&invalid_data, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_create_pool_inner_instruction_when_valid_data_should_return_ok() {
        let metadata = create_test_metadata();

        let test_event = PumpSwapCreatePoolEvent {
            metadata: create_test_metadata(),
            timestamp: 0,
            index: 0,
            creator: Pubkey::default(),
            base_mint: Pubkey::default(),
            quote_mint: Pubkey::default(),
            base_mint_decimals: 0,
            quote_mint_decimals: 0,
            base_amount_in: 0,
            quote_amount_in: 0,
            pool_base_amount: 0,
            pool_quote_amount: 0,
            minimum_liquidity: 0,
            initial_liquidity: 0,
            lp_token_amount_out: 0,
            pool_bump: 0,
            pool: Pubkey::new_unique(),
            lp_mint: Pubkey::default(),
            user_base_token_account: Pubkey::default(),
            user_quote_token_account: Pubkey::default(),
            coin_creator: Pubkey::default(),
            user_pool_token_account: Pubkey::default(),
            pool_base_token_account: Pubkey::default(),
            pool_quote_token_account: Pubkey::default(),
        };
        let data = borsh::to_vec(&test_event).unwrap();

        let result = PumpSwapEventParser::parse_create_pool_inner_instruction(&data, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_create_pool_inner_instruction_when_invalid_data_should_return_err() {
        let metadata = create_test_metadata();
        let invalid_data = vec![0x11, 0x22];

        let result = PumpSwapEventParser::parse_create_pool_inner_instruction(&invalid_data, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_deposit_inner_instruction_when_valid_data_should_return_ok() {
        let metadata = create_test_metadata();

        let test_event = PumpSwapDepositEvent {
            metadata: create_test_metadata(),
            timestamp: 0,
            lp_token_amount_out: 0,
            max_base_amount_in: 0,
            max_quote_amount_in: 0,
            user_base_token_reserves: 0,
            user_quote_token_reserves: 0,
            pool_base_token_reserves: 0,
            pool_quote_token_reserves: 0,
            base_amount_in: 0,
            quote_amount_in: 0,
            lp_mint_supply: 0,
            pool: Pubkey::new_unique(),
            user: Pubkey::default(),
            user_base_token_account: Pubkey::default(),
            user_quote_token_account: Pubkey::default(),
            user_pool_token_account: Pubkey::default(),
            base_mint: Pubkey::default(),
            quote_mint: Pubkey::default(),
            pool_base_token_account: Pubkey::default(),
            pool_quote_token_account: Pubkey::default(),
        };
        let data = borsh::to_vec(&test_event).unwrap();

        let result = PumpSwapEventParser::parse_deposit_inner_instruction(&data, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_deposit_inner_instruction_when_invalid_data_should_return_err() {
        let metadata = create_test_metadata();
        let invalid_data = vec![0xAA, 0xBB, 0xCC];

        let result = PumpSwapEventParser::parse_deposit_inner_instruction(&invalid_data, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_withdraw_inner_instruction_when_valid_data_should_return_ok() {
        let metadata = create_test_metadata();

        let test_event = PumpSwapWithdrawEvent {
            metadata: create_test_metadata(),
            timestamp: 0,
            lp_token_amount_in: 0,
            min_base_amount_out: 0,
            min_quote_amount_out: 0,
            user_base_token_reserves: 0,
            user_quote_token_reserves: 0,
            pool_base_token_reserves: 0,
            pool_quote_token_reserves: 0,
            base_amount_out: 0,
            quote_amount_out: 0,
            lp_mint_supply: 0,
            pool: Pubkey::new_unique(),
            user: Pubkey::default(),
            user_base_token_account: Pubkey::default(),
            user_quote_token_account: Pubkey::default(),
            user_pool_token_account: Pubkey::default(),
            base_mint: Pubkey::default(),
            quote_mint: Pubkey::default(),
            pool_base_token_account: Pubkey::default(),
            pool_quote_token_account: Pubkey::default(),
        };
        let data = borsh::to_vec(&test_event).unwrap();

        let result = PumpSwapEventParser::parse_withdraw_inner_instruction(&data, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_withdraw_inner_instruction_when_invalid_data_should_return_err() {
        let metadata = create_test_metadata();
        let invalid_data = vec![0x33, 0x44, 0x55];

        let result = PumpSwapEventParser::parse_withdraw_inner_instruction(&invalid_data, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_buy_instruction_when_valid_data_should_return_ok() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();

        // Create valid instruction data: 16 bytes minimum
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&1000u64.to_le_bytes()); // base_amount_out
        data[8..16].copy_from_slice(&2000u64.to_le_bytes()); // max_quote_amount_in

        let result = PumpSwapEventParser::parse_buy_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());

        let event = result.unwrap();
        let buy_event = event.as_any().downcast_ref::<PumpSwapBuyEvent>().unwrap();
        assert_eq!(buy_event.base_amount_out, 1000);
        assert_eq!(buy_event.max_quote_amount_in, 2000);
        assert_eq!(buy_event.pool, accounts[0]);
        assert_eq!(buy_event.user, accounts[1]);
    }

    #[test]
    fn test_parse_buy_instruction_when_insufficient_data_length_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let short_data = vec![0u8; 15]; // Less than 16 bytes required

        let result = PumpSwapEventParser::parse_buy_instruction(&short_data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_buy_instruction_when_insufficient_accounts_should_return_err() {
        let metadata = create_test_metadata();
        let data = vec![0u8; 16];
        let short_accounts = vec![Pubkey::new_unique(); 10]; // Less than 11 required

        let result = PumpSwapEventParser::parse_buy_instruction(&data, &short_accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_buy_instruction_when_empty_data_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let empty_data = vec![];

        let result = PumpSwapEventParser::parse_buy_instruction(&empty_data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_buy_instruction_with_optional_accounts() {
        let metadata = create_test_metadata();
        let mut accounts = create_test_accounts();
        accounts.truncate(19); // Remove last account to test get() returns Some

        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&500u64.to_le_bytes());
        data[8..16].copy_from_slice(&1500u64.to_le_bytes());

        let result = PumpSwapEventParser::parse_buy_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());

        let event = result.unwrap();
        let buy_event = event.as_any().downcast_ref::<PumpSwapBuyEvent>().unwrap();
        assert_eq!(buy_event.coin_creator_vault_ata, accounts[17]);
        assert_eq!(buy_event.coin_creator_vault_authority, accounts[18]);
    }

    #[test]
    fn test_parse_buy_instruction_with_missing_optional_accounts() {
        let metadata = create_test_metadata();
        let mut accounts = create_test_accounts();
        accounts.truncate(17); // Remove optional accounts to test get() returns None -> default

        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&500u64.to_le_bytes());
        data[8..16].copy_from_slice(&1500u64.to_le_bytes());

        let result = PumpSwapEventParser::parse_buy_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());

        let event = result.unwrap();
        let buy_event = event.as_any().downcast_ref::<PumpSwapBuyEvent>().unwrap();
        assert_eq!(buy_event.coin_creator_vault_ata, Pubkey::default());
        assert_eq!(buy_event.coin_creator_vault_authority, Pubkey::default());
    }

    #[test]
    fn test_parse_sell_instruction_when_valid_data_should_return_ok() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();

        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&1500u64.to_le_bytes()); // base_amount_in
        data[8..16].copy_from_slice(&800u64.to_le_bytes()); // min_quote_amount_out

        let result = PumpSwapEventParser::parse_sell_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());

        let event = result.unwrap();
        let sell_event = event.as_any().downcast_ref::<PumpSwapSellEvent>().unwrap();
        assert_eq!(sell_event.base_amount_in, 1500);
        assert_eq!(sell_event.min_quote_amount_out, 800);
        assert_eq!(sell_event.pool, accounts[0]);
        assert_eq!(sell_event.user, accounts[1]);
    }

    #[test]
    fn test_parse_sell_instruction_when_insufficient_data_length_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let short_data = vec![0u8; 15];

        let result = PumpSwapEventParser::parse_sell_instruction(&short_data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sell_instruction_when_insufficient_accounts_should_return_err() {
        let metadata = create_test_metadata();
        let data = vec![0u8; 16];
        let short_accounts = vec![Pubkey::new_unique(); 10];

        let result = PumpSwapEventParser::parse_sell_instruction(&data, &short_accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_create_pool_instruction_when_valid_data_should_return_ok() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();

        let mut data = vec![0u8; 50]; // Include coin_creator data
        data[0..2].copy_from_slice(&42u16.to_le_bytes()); // index
        data[2..10].copy_from_slice(&5000u64.to_le_bytes()); // base_amount_in
        data[10..18].copy_from_slice(&3000u64.to_le_bytes()); // quote_amount_in
        let coin_creator = Pubkey::new_unique();
        data[18..50].copy_from_slice(&coin_creator.to_bytes()); // coin_creator

        let result = PumpSwapEventParser::parse_create_pool_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());

        let event = result.unwrap();
        let create_event = event
            .as_any()
            .downcast_ref::<PumpSwapCreatePoolEvent>()
            .unwrap();
        assert_eq!(create_event.index, 42);
        assert_eq!(create_event.base_amount_in, 5000);
        assert_eq!(create_event.quote_amount_in, 3000);
        assert_eq!(create_event.coin_creator, coin_creator);
        assert_eq!(create_event.pool, accounts[0]);
        assert_eq!(create_event.creator, accounts[2]);
    }

    #[test]
    fn test_parse_create_pool_instruction_when_insufficient_data_length_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let short_data = vec![0u8; 17]; // Less than 18 bytes required

        let result = PumpSwapEventParser::parse_create_pool_instruction(&short_data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_create_pool_instruction_when_insufficient_accounts_should_return_err() {
        let metadata = create_test_metadata();
        let data = vec![0u8; 18];
        let short_accounts = vec![Pubkey::new_unique(); 10];

        let result = PumpSwapEventParser::parse_create_pool_instruction(&data, &short_accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_create_pool_instruction_without_coin_creator_data() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();

        let mut data = vec![0u8; 18]; // Minimum length without coin_creator
        data[0..2].copy_from_slice(&10u16.to_le_bytes());
        data[2..10].copy_from_slice(&1000u64.to_le_bytes());
        data[10..18].copy_from_slice(&2000u64.to_le_bytes());

        let result = PumpSwapEventParser::parse_create_pool_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());

        let event = result.unwrap();
        let create_event = event
            .as_any()
            .downcast_ref::<PumpSwapCreatePoolEvent>()
            .unwrap();
        assert_eq!(create_event.coin_creator, Pubkey::default());
    }

    #[test]
    fn test_parse_create_pool_instruction_with_invalid_coin_creator_bytes() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();

        let mut data = vec![0u8; 49]; // Not enough bytes for coin_creator (need 50)
        data[0..2].copy_from_slice(&10u16.to_le_bytes());
        data[2..10].copy_from_slice(&1000u64.to_le_bytes());
        data[10..18].copy_from_slice(&2000u64.to_le_bytes());

        let result = PumpSwapEventParser::parse_create_pool_instruction(&data, &accounts, metadata);
        assert!(result.is_err()); // Should fail due to invalid try_into for coin_creator
    }

    #[test]
    fn test_parse_deposit_instruction_when_valid_data_should_return_ok() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();

        let mut data = vec![0u8; 24];
        data[0..8].copy_from_slice(&100u64.to_le_bytes()); // lp_token_amount_out
        data[8..16].copy_from_slice(&500u64.to_le_bytes()); // max_base_amount_in
        data[16..24].copy_from_slice(&300u64.to_le_bytes()); // max_quote_amount_in

        let result = PumpSwapEventParser::parse_deposit_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());

        let event = result.unwrap();
        let deposit_event = event
            .as_any()
            .downcast_ref::<PumpSwapDepositEvent>()
            .unwrap();
        assert_eq!(deposit_event.lp_token_amount_out, 100);
        assert_eq!(deposit_event.max_base_amount_in, 500);
        assert_eq!(deposit_event.max_quote_amount_in, 300);
        assert_eq!(deposit_event.pool, accounts[0]);
        assert_eq!(deposit_event.user, accounts[2]);
    }

    #[test]
    fn test_parse_deposit_instruction_when_insufficient_data_length_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let short_data = vec![0u8; 23]; // Less than 24 bytes required

        let result = PumpSwapEventParser::parse_deposit_instruction(&short_data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_deposit_instruction_when_insufficient_accounts_should_return_err() {
        let metadata = create_test_metadata();
        let data = vec![0u8; 24];
        let short_accounts = vec![Pubkey::new_unique(); 10];

        let result = PumpSwapEventParser::parse_deposit_instruction(&data, &short_accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_withdraw_instruction_when_valid_data_should_return_ok() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();

        let mut data = vec![0u8; 24];
        data[0..8].copy_from_slice(&150u64.to_le_bytes()); // lp_token_amount_in
        data[8..16].copy_from_slice(&600u64.to_le_bytes()); // min_base_amount_out
        data[16..24].copy_from_slice(&400u64.to_le_bytes()); // min_quote_amount_out

        let result = PumpSwapEventParser::parse_withdraw_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());

        let event = result.unwrap();
        let withdraw_event = event
            .as_any()
            .downcast_ref::<PumpSwapWithdrawEvent>()
            .unwrap();
        assert_eq!(withdraw_event.lp_token_amount_in, 150);
        assert_eq!(withdraw_event.min_base_amount_out, 600);
        assert_eq!(withdraw_event.min_quote_amount_out, 400);
        assert_eq!(withdraw_event.pool, accounts[0]);
        assert_eq!(withdraw_event.user, accounts[2]);
    }

    #[test]
    fn test_parse_withdraw_instruction_when_insufficient_data_length_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let short_data = vec![0u8; 23];

        let result = PumpSwapEventParser::parse_withdraw_instruction(&short_data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_withdraw_instruction_when_insufficient_accounts_should_return_err() {
        let metadata = create_test_metadata();
        let data = vec![0u8; 24];
        let short_accounts = vec![Pubkey::new_unique(); 10];

        let result = PumpSwapEventParser::parse_withdraw_instruction(&data, &short_accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_events_from_inner_instruction() {
        let parser = PumpSwapEventParser::default();
        let inner_instruction = solana_transaction_status::UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: "test_data".to_string(),
            stack_height: Some(1),
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 100,
            block_time: Some(1640995200),
            program_received_time_ms: 1640995200000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        // Should return empty vector since we don't have valid discriminator data
        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_events_from_instruction() {
        let parser = PumpSwapEventParser::default();
        let instruction = solana_message::compiled_instruction::CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: vec![1, 2, 3],
        };
        let accounts = vec![
            PUMPSWAP_PROGRAM_ID,
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 100,
            block_time: Some(1640995200),
            program_received_time_ms: 1640995200000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        // Should return empty vector since we don't have valid discriminator data
        assert!(events.is_empty());
    }

    #[test]
    fn test_metadata_id_generation_for_buy_inner_instruction() {
        let mut metadata = create_test_metadata();
        metadata.signature = "abc123".to_string();

        let test_event = PumpSwapBuyEvent {
            metadata: crate::solana_metadata::SolanaEventMetadata::default(),
            timestamp: 0,
            base_amount_out: 0,
            max_quote_amount_in: 0,
            user_base_token_reserves: 0,
            user_quote_token_reserves: 0,
            pool_base_token_reserves: 0,
            pool_quote_token_reserves: 0,
            quote_amount_in: 0,
            lp_fee_basis_points: 0,
            lp_fee: 0,
            protocol_fee_basis_points: 0,
            protocol_fee: 0,
            quote_amount_in_with_lp_fee: 0,
            user_quote_amount_in: 0,
            pool: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
            user: Pubkey::default(),
            user_base_token_account: Pubkey::default(),
            user_quote_token_account: Pubkey::default(),
            protocol_fee_recipient: Pubkey::default(),
            protocol_fee_recipient_token_account: Pubkey::default(),
            coin_creator: Pubkey::default(),
            coin_creator_fee_basis_points: 0,
            coin_creator_fee: 0,
            track_volume: false,
            total_unclaimed_tokens: 0,
            total_claimed_tokens: 0,
            current_sol_volume: 0,
            last_update_timestamp: 0,
            base_mint: Pubkey::default(),
            quote_mint: Pubkey::default(),
            pool_base_token_account: Pubkey::default(),
            pool_quote_token_account: Pubkey::default(),
            coin_creator_vault_ata: Pubkey::default(),
            coin_creator_vault_authority: Pubkey::default(),
        };
        let data = borsh::to_vec(&test_event).unwrap();

        let result = PumpSwapEventParser::parse_buy_inner_instruction(&data, metadata);
        assert!(result.is_ok());

        let event = result.unwrap();
        let expected_id = format!("abc123-{}-buy", test_event.pool);
        assert_eq!(event.id(), expected_id);
    }

    #[test]
    fn test_metadata_id_generation_for_sell_inner_instruction() {
        let mut metadata = create_test_metadata();
        metadata.signature = "def456".to_string();

        let test_event = PumpSwapSellEvent {
            metadata: create_test_metadata(),
            timestamp: 0,
            base_amount_in: 0,
            min_quote_amount_out: 0,
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
            pool: Pubkey::from_str("22222222222222222222222222222222").unwrap(),
            user: Pubkey::default(),
            user_base_token_account: Pubkey::default(),
            user_quote_token_account: Pubkey::default(),
            protocol_fee_recipient: Pubkey::default(),
            protocol_fee_recipient_token_account: Pubkey::default(),
            coin_creator: Pubkey::default(),
            coin_creator_fee_basis_points: 0,
            coin_creator_fee: 0,
            base_mint: Pubkey::default(),
            quote_mint: Pubkey::default(),
            pool_base_token_account: Pubkey::default(),
            pool_quote_token_account: Pubkey::default(),
            coin_creator_vault_ata: Pubkey::default(),
            coin_creator_vault_authority: Pubkey::default(),
        };
        let data = borsh::to_vec(&test_event).unwrap();

        let result = PumpSwapEventParser::parse_sell_inner_instruction(&data, metadata);
        assert!(result.is_ok());

        let event = result.unwrap();
        let expected_id = format!("def456-{}-sell", test_event.pool);
        assert_eq!(event.id(), expected_id);
    }

    #[test]
    fn test_metadata_id_generation_for_buy_instruction() {
        let mut metadata = create_test_metadata();
        metadata.signature = "ghi789".to_string();
        let accounts = create_test_accounts();

        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&1000u64.to_le_bytes()); // base_amount_out
        data[8..16].copy_from_slice(&2000u64.to_le_bytes()); // max_quote_amount_in

        let result = PumpSwapEventParser::parse_buy_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());

        let event = result.unwrap();
        let expected_id = format!("ghi789-{}-{}-1000", accounts[1], accounts[0]);
        assert_eq!(event.id(), expected_id);
    }

    #[test]
    fn test_invalid_byte_conversion_in_create_pool_instruction() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();

        // Create data that will fail u16::from_le_bytes conversion
        let data = vec![0u8; 1]; // Only 1 byte, need 2 for u16

        let result = PumpSwapEventParser::parse_create_pool_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_byte_conversion_in_deposit_instruction() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();

        // Create data that will fail u64::from_le_bytes conversion
        let data = vec![0u8; 7]; // Only 7 bytes, need 8 for u64

        let result = PumpSwapEventParser::parse_deposit_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_byte_conversion_in_withdraw_instruction() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();

        // Create data that will fail u64::from_le_bytes conversion
        let data = vec![0u8; 15]; // Only 15 bytes, need at least 24

        let result = PumpSwapEventParser::parse_withdraw_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_u64_le_failure_in_buy_instruction() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();

        // Create data that has enough length but will fail read_u64_le due to malformed data
        // read_u64_le expects valid u64 at specific offsets
        let _data = [0xFFu8; 16];
        // Deliberately corrupt the data at offset 0 to make read_u64_le fail
        // This is harder to trigger since read_u64_le is quite permissive,
        // but we can test with empty slice which should fail bounds check

        // Actually, let's test by creating a scenario where the data slice is too short for read_u64_le
        // by providing exactly 16 bytes but making read_u64_le fail on bounds
        let short_data = vec![0u8; 7]; // Less than 8 bytes needed for first u64

        let result = PumpSwapEventParser::parse_buy_instruction(&short_data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_u64_le_failure_in_sell_instruction() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();

        let short_data = vec![0u8; 7]; // Less than 8 bytes needed for first u64

        let result = PumpSwapEventParser::parse_sell_instruction(&short_data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_event_parser_trait_delegation() {
        let parser = PumpSwapEventParser::default();

        // Test that all EventParser trait methods properly delegate to inner
        let inner_configs = parser.inner_instruction_configs();
        let instruction_configs = parser.instruction_configs();
        let supported_ids = parser.supported_program_ids();
        let should_handle = parser.should_handle(&PUMPSWAP_PROGRAM_ID);

        assert!(!inner_configs.is_empty());
        assert!(!instruction_configs.is_empty());
        assert_eq!(supported_ids, vec![PUMPSWAP_PROGRAM_ID]);
        assert!(should_handle);
    }

    // ========== ADDITIONAL COMPREHENSIVE TESTS FOR TASK 4 ==========

    // Test ParseError conditions for insufficient data lengths

    #[test]
    fn test_parse_buy_instruction_when_data_exactly_15_bytes_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let data = vec![0u8; 15]; // One byte short of required 16

        let result = PumpSwapEventParser::parse_buy_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Insufficient data"));
        }
    }

    #[test]
    fn test_parse_sell_instruction_when_data_exactly_15_bytes_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let data = vec![0u8; 15]; // One byte short of required 16

        let result = PumpSwapEventParser::parse_sell_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Insufficient data"));
        }
    }

    #[test]
    fn test_parse_create_pool_instruction_when_data_exactly_17_bytes_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let data = vec![0u8; 17]; // One byte short of required 18

        let result = PumpSwapEventParser::parse_create_pool_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Insufficient data"));
        }
    }

    #[test]
    fn test_parse_deposit_instruction_when_data_exactly_23_bytes_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let data = vec![0u8; 23]; // One byte short of required 24

        let result = PumpSwapEventParser::parse_deposit_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Insufficient data"));
        }
    }

    #[test]
    fn test_parse_withdraw_instruction_when_data_exactly_23_bytes_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let data = vec![0u8; 23]; // One byte short of required 24

        let result = PumpSwapEventParser::parse_withdraw_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Insufficient data"));
        }
    }

    // Test ParseError conditions for insufficient accounts

    #[test]
    fn test_parse_buy_instruction_when_accounts_exactly_10_should_return_err() {
        let metadata = create_test_metadata();
        let data = vec![0u8; 16];
        let accounts = vec![Pubkey::new_unique(); 10]; // One account short of required 11

        let result = PumpSwapEventParser::parse_buy_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Insufficient accounts"));
        }
    }

    #[test]
    fn test_parse_sell_instruction_when_accounts_exactly_10_should_return_err() {
        let metadata = create_test_metadata();
        let data = vec![0u8; 16];
        let accounts = vec![Pubkey::new_unique(); 10]; // One account short of required 11

        let result = PumpSwapEventParser::parse_sell_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Insufficient data"));
        }
    }

    #[test]
    fn test_parse_create_pool_instruction_when_accounts_exactly_10_should_return_err() {
        let metadata = create_test_metadata();
        let data = vec![0u8; 18];
        let accounts = vec![Pubkey::new_unique(); 10]; // One account short of required 11

        let result = PumpSwapEventParser::parse_create_pool_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Insufficient accounts"));
        }
    }

    #[test]
    fn test_parse_deposit_instruction_when_accounts_exactly_10_should_return_err() {
        let metadata = create_test_metadata();
        let data = vec![0u8; 24];
        let accounts = vec![Pubkey::new_unique(); 10]; // One account short of required 11

        let result = PumpSwapEventParser::parse_deposit_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Insufficient data"));
        }
    }

    #[test]
    fn test_parse_withdraw_instruction_when_accounts_exactly_10_should_return_err() {
        let metadata = create_test_metadata();
        let data = vec![0u8; 24];
        let accounts = vec![Pubkey::new_unique(); 10]; // One account short of required 11

        let result = PumpSwapEventParser::parse_withdraw_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Insufficient data"));
        }
    }

    // Test with invalid data format scenarios

    #[test]
    fn test_parse_inner_instructions_when_corrupted_borsh_data_should_return_err() {
        let metadata = create_test_metadata();
        // Create completely invalid borsh data
        let invalid_data = vec![0xFF; 100];

        // Test all inner instruction parsers
        let buy_result =
            PumpSwapEventParser::parse_buy_inner_instruction(&invalid_data, metadata.clone());
        assert!(buy_result.is_err());

        let sell_result =
            PumpSwapEventParser::parse_sell_inner_instruction(&invalid_data, metadata.clone());
        assert!(sell_result.is_err());

        let create_result = PumpSwapEventParser::parse_create_pool_inner_instruction(
            &invalid_data,
            metadata.clone(),
        );
        assert!(create_result.is_err());

        let deposit_result =
            PumpSwapEventParser::parse_deposit_inner_instruction(&invalid_data, metadata.clone());
        assert!(deposit_result.is_err());

        let withdraw_result =
            PumpSwapEventParser::parse_withdraw_inner_instruction(&invalid_data, metadata);
        assert!(withdraw_result.is_err());
    }

    // Test edge case values - zero amounts

    #[test]
    fn test_parse_buy_instruction_with_zero_amounts_should_succeed() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let data = vec![0u8; 16]; // Both amounts are zero

        let result = PumpSwapEventParser::parse_buy_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let buy_event = event.as_any().downcast_ref::<PumpSwapBuyEvent>().unwrap();
        assert_eq!(buy_event.base_amount_out, 0);
        assert_eq!(buy_event.max_quote_amount_in, 0);
    }

    #[test]
    fn test_parse_sell_instruction_with_zero_amounts_should_succeed() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let data = vec![0u8; 16]; // Both amounts are zero

        let result = PumpSwapEventParser::parse_sell_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let sell_event = event.as_any().downcast_ref::<PumpSwapSellEvent>().unwrap();
        assert_eq!(sell_event.base_amount_in, 0);
        assert_eq!(sell_event.min_quote_amount_out, 0);
    }

    // Test edge case values - maximum amounts

    #[test]
    fn test_parse_buy_instruction_with_max_amounts_should_succeed() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&u64::MAX.to_le_bytes());
        data[8..16].copy_from_slice(&u64::MAX.to_le_bytes());

        let result = PumpSwapEventParser::parse_buy_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let buy_event = event.as_any().downcast_ref::<PumpSwapBuyEvent>().unwrap();
        assert_eq!(buy_event.base_amount_out, u64::MAX);
        assert_eq!(buy_event.max_quote_amount_in, u64::MAX);
    }

    #[test]
    fn test_parse_sell_instruction_with_max_amounts_should_succeed() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&u64::MAX.to_le_bytes());
        data[8..16].copy_from_slice(&u64::MAX.to_le_bytes());

        let result = PumpSwapEventParser::parse_sell_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let sell_event = event.as_any().downcast_ref::<PumpSwapSellEvent>().unwrap();
        assert_eq!(sell_event.base_amount_in, u64::MAX);
        assert_eq!(sell_event.min_quote_amount_out, u64::MAX);
    }

    #[test]
    fn test_parse_create_pool_instruction_with_max_values_should_succeed() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let mut data = vec![0u8; 50];
        data[0..2].copy_from_slice(&u16::MAX.to_le_bytes()); // max index
        data[2..10].copy_from_slice(&u64::MAX.to_le_bytes()); // max base_amount_in
        data[10..18].copy_from_slice(&u64::MAX.to_le_bytes()); // max quote_amount_in

        let result = PumpSwapEventParser::parse_create_pool_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let create_event = event
            .as_any()
            .downcast_ref::<PumpSwapCreatePoolEvent>()
            .unwrap();
        assert_eq!(create_event.index, u16::MAX);
        assert_eq!(create_event.base_amount_in, u64::MAX);
        assert_eq!(create_event.quote_amount_in, u64::MAX);
    }

    #[test]
    fn test_parse_deposit_instruction_with_max_amounts_should_succeed() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let mut data = vec![0u8; 24];
        data[0..8].copy_from_slice(&u64::MAX.to_le_bytes());
        data[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
        data[16..24].copy_from_slice(&u64::MAX.to_le_bytes());

        let result = PumpSwapEventParser::parse_deposit_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let deposit_event = event
            .as_any()
            .downcast_ref::<PumpSwapDepositEvent>()
            .unwrap();
        assert_eq!(deposit_event.lp_token_amount_out, u64::MAX);
        assert_eq!(deposit_event.max_base_amount_in, u64::MAX);
        assert_eq!(deposit_event.max_quote_amount_in, u64::MAX);
    }

    #[test]
    fn test_parse_withdraw_instruction_with_max_amounts_should_succeed() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let mut data = vec![0u8; 24];
        data[0..8].copy_from_slice(&u64::MAX.to_le_bytes());
        data[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
        data[16..24].copy_from_slice(&u64::MAX.to_le_bytes());

        let result = PumpSwapEventParser::parse_withdraw_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let withdraw_event = event
            .as_any()
            .downcast_ref::<PumpSwapWithdrawEvent>()
            .unwrap();
        assert_eq!(withdraw_event.lp_token_amount_in, u64::MAX);
        assert_eq!(withdraw_event.min_base_amount_out, u64::MAX);
        assert_eq!(withdraw_event.min_quote_amount_out, u64::MAX);
    }

    // Test with exact minimum required data and accounts

    #[test]
    fn test_parse_instructions_with_exact_minimum_requirements_should_succeed() {
        let metadata = create_test_metadata();

        // Test buy instruction with exactly 16 bytes and 11 accounts
        let accounts = vec![Pubkey::new_unique(); 11];
        let data = vec![0u8; 16];
        let result = PumpSwapEventParser::parse_buy_instruction(&data, &accounts, metadata.clone());
        assert!(result.is_ok());

        // Test sell instruction with exactly 16 bytes and 11 accounts
        let result = PumpSwapEventParser::parse_sell_instruction(&data, &accounts, metadata.clone());
        assert!(result.is_ok());

        // Test create pool instruction with exactly 18 bytes and 11 accounts
        let data_18 = vec![0u8; 18];
        let result = PumpSwapEventParser::parse_create_pool_instruction(&data_18, &accounts, metadata.clone());
        assert!(result.is_ok());

        // Test deposit/withdraw instructions with exactly 24 bytes and 11 accounts
        let data_24 = vec![0u8; 24];
        let result = PumpSwapEventParser::parse_deposit_instruction(&data_24, &accounts, metadata.clone());
        assert!(result.is_ok());

        let result = PumpSwapEventParser::parse_withdraw_instruction(&data_24, &accounts, metadata);
        assert!(result.is_ok());
    }

    // Test with more than required data and accounts (should still work)

    #[test]
    fn test_parse_instructions_with_excess_data_and_accounts_should_succeed() {
        let metadata = create_test_metadata();
        let accounts = vec![Pubkey::new_unique(); 25]; // More than required 11

        // Test with excess data
        let data = vec![0u8; 32]; // More than required 16
        let result = PumpSwapEventParser::parse_buy_instruction(&data, &accounts, metadata.clone());
        assert!(result.is_ok());

        let result = PumpSwapEventParser::parse_sell_instruction(&data, &accounts, metadata.clone());
        assert!(result.is_ok());

        let data_large = vec![0u8; 100]; // Much more than required
        let result = PumpSwapEventParser::parse_create_pool_instruction(&data_large, &accounts, metadata.clone());
        assert!(result.is_ok());

        let result = PumpSwapEventParser::parse_deposit_instruction(&data_large, &accounts, metadata.clone());
        assert!(result.is_ok());

        let result = PumpSwapEventParser::parse_withdraw_instruction(&data_large, &accounts, metadata);
        assert!(result.is_ok());
    }

    // Test error handling for try_into failures

    #[test]
    fn test_parse_deposit_instruction_corrupted_byte_conversion_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();

        // Create data that has correct length but corrupted at specific offsets
        let _data = [0u8; 24];
        // Corrupt the first 8 bytes by making it shorter - simulate try_into failure
        // Actually, since try_into won't fail with proper slice length,
        // we test with insufficient data length which is caught earlier
        let short_data = vec![0u8; 7]; // Less than 8 bytes for first u64

        let result = PumpSwapEventParser::parse_deposit_instruction(&short_data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_withdraw_instruction_corrupted_byte_conversion_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();

        let short_data = vec![0u8; 15]; // Between 8 and 16 bytes - will fail on second u64

        let result = PumpSwapEventParser::parse_withdraw_instruction(&short_data, &accounts, metadata);
        assert!(result.is_err());
    }

    // Test edge cases with default pubkeys

    #[test]
    fn test_parse_instructions_with_default_pubkeys_should_succeed() {
        let metadata = create_test_metadata();
        let accounts = vec![Pubkey::default(); 25]; // All default pubkeys

        let data = vec![0u8; 16];
        let result = PumpSwapEventParser::parse_buy_instruction(&data, &accounts, metadata.clone());
        assert!(result.is_ok());

        let event = result.unwrap();
        let buy_event = event.as_any().downcast_ref::<PumpSwapBuyEvent>().unwrap();
        assert_eq!(buy_event.pool, Pubkey::default());
        assert_eq!(buy_event.user, Pubkey::default());
    }

    // Test ID generation with extreme values

    #[test]
    fn test_instruction_id_generation_with_max_values() {
        let mut metadata = create_test_metadata();
        metadata.signature = "extremely_long_signature_".repeat(10); // Very long signature

        let accounts = create_test_accounts();
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&u64::MAX.to_le_bytes());
        data[8..16].copy_from_slice(&u64::MAX.to_le_bytes());

        let result = PumpSwapEventParser::parse_buy_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        // ID should contain max values without causing panic
        assert!(event.id().contains(&u64::MAX.to_string()));
    }

    // Test parser with empty string signature and index

    #[test]
    fn test_parse_instruction_with_empty_signature_should_succeed() {
        let mut metadata = create_test_metadata();
        metadata.signature = "".to_string();

        let accounts = create_test_accounts();
        let data = vec![0u8; 16];

        let result = PumpSwapEventParser::parse_buy_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
    }

    // Test specific error message validation for read_u64_le failures

    #[test]
    fn test_parse_sell_instruction_read_u64_le_error_messages() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();

        // Create data that will cause read_u64_le to fail at different offsets
        let data_7_bytes = vec![0u8; 7]; // Fails on first read_u64_le
        let result = PumpSwapEventParser::parse_sell_instruction(&data_7_bytes, &accounts, metadata.clone());
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Failed to read"));
        }

        let data_15_bytes = vec![0u8; 15]; // Fails on second read_u64_le
        let result = PumpSwapEventParser::parse_sell_instruction(&data_15_bytes, &accounts, metadata);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Failed to read"));
        }
    }

    // Test empty and minimal cases

    #[test]
    fn test_parse_all_instructions_with_empty_data_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = create_test_accounts();
        let empty_data = vec![];

        // All instruction parsers should fail with empty data
        assert!(PumpSwapEventParser::parse_buy_instruction(
            &empty_data,
            &accounts,
            metadata.clone()
        )
        .is_err());
        assert!(PumpSwapEventParser::parse_sell_instruction(
            &empty_data,
            &accounts,
            metadata.clone()
        )
        .is_err());
        assert!(PumpSwapEventParser::parse_create_pool_instruction(
            &empty_data,
            &accounts,
            metadata.clone()
        )
        .is_err());
        assert!(PumpSwapEventParser::parse_deposit_instruction(
            &empty_data,
            &accounts,
            metadata.clone()
        )
        .is_err());
        assert!(
            PumpSwapEventParser::parse_withdraw_instruction(&empty_data, &accounts, metadata)
                .is_err()
        );
    }

    #[test]
    fn test_parse_all_instructions_with_empty_accounts_should_return_err() {
        let metadata = create_test_metadata();
        let empty_accounts = vec![];
        let data = vec![0u8; 24];

        // All instruction parsers should fail with empty accounts
        assert!(PumpSwapEventParser::parse_buy_instruction(
            &data,
            &empty_accounts,
            metadata.clone()
        )
        .is_err());
        assert!(PumpSwapEventParser::parse_sell_instruction(
            &data,
            &empty_accounts,
            metadata.clone()
        )
        .is_err());
        assert!(PumpSwapEventParser::parse_create_pool_instruction(
            &data,
            &empty_accounts,
            metadata.clone()
        )
        .is_err());
        assert!(PumpSwapEventParser::parse_deposit_instruction(
            &data,
            &empty_accounts,
            metadata.clone()
        )
        .is_err());
        assert!(
            PumpSwapEventParser::parse_withdraw_instruction(&data, &empty_accounts, metadata)
                .is_err()
        );
    }

    // Test should_handle with edge cases

    #[test]
    fn test_should_handle_with_default_pubkey_should_return_false() {
        let parser = PumpSwapEventParser::default();
        let default_pubkey = Pubkey::default();

        let result = parser.should_handle(&default_pubkey);

        assert!(!result);
    }
}
