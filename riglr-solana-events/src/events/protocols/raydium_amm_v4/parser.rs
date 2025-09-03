use riglr_events_core::{
    error::EventResult,
    traits::{EventParser, ParserInfo},
    Event,
};
use std::collections::HashMap;

use solana_sdk::pubkey::Pubkey;

use crate::error::ParseResult;
use crate::events::{
    common::{
        read_u64_le, read_u8_le,
        utils::{
            parse_liquidity_amounts, safe_get_account, validate_account_count, validate_data_length,
        },
        EventType, ProtocolType,
    },
    core::traits::{EventParser as LegacyEventParser, GenericEventParseConfig, GenericEventParser},
    factory::SolanaTransactionInput,
    protocols::raydium_amm_v4::{
        discriminators, RaydiumAmmV4DepositEvent, RaydiumAmmV4Initialize2Event,
        RaydiumAmmV4SwapEvent, RaydiumAmmV4WithdrawEvent, RaydiumAmmV4WithdrawPnlEvent,
        SwapDirection,
    },
};
use crate::solana_metadata::SolanaEventMetadata;

type EventMetadata = SolanaEventMetadata;

/// Raydium AMM V4 program ID
pub const RAYDIUM_AMM_V4_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

/// Raydium AMM V4 event parser
pub struct RaydiumAmmV4EventParser {
    inner: GenericEventParser,
}

impl Default for RaydiumAmmV4EventParser {
    fn default() -> Self {
        let configs = vec![
            GenericEventParseConfig {
                program_id: RAYDIUM_AMM_V4_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumAmmV4,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::SWAP_BASE_IN,
                event_type: EventType::RaydiumAmmV4SwapBaseIn,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_swap_base_input_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_AMM_V4_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumAmmV4,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::SWAP_BASE_OUT,
                event_type: EventType::RaydiumAmmV4SwapBaseOut,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_swap_base_output_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_AMM_V4_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumAmmV4,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::DEPOSIT,
                event_type: EventType::RaydiumAmmV4Deposit,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_deposit_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_AMM_V4_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumAmmV4,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::INITIALIZE2,
                event_type: EventType::RaydiumAmmV4Initialize2,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_initialize2_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_AMM_V4_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumAmmV4,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::WITHDRAW,
                event_type: EventType::RaydiumAmmV4Withdraw,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_withdraw_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_AMM_V4_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumAmmV4,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::WITHDRAW_PNL,
                event_type: EventType::RaydiumAmmV4WithdrawPnl,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_withdraw_pnl_instruction,
            },
        ];

        let inner = GenericEventParser::new(vec![RAYDIUM_AMM_V4_PROGRAM_ID], configs);
        Self { inner }
    }
}

impl RaydiumAmmV4EventParser {
    /// Creates a new Raydium AMM V4 event parser with configured discriminators and parsers
    pub fn new() -> Self {
        Self::default()
    }

    /// Helper method to get inner instruction configs
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner.inner_instruction_configs()
    }

    /// Helper method to get instruction configs
    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.inner.instruction_configs()
    }

    /// Helper method to check if should handle program ID
    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.inner.should_handle(program_id)
    }

    /// Helper method to get supported program IDs
    fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.inner.supported_program_ids()
    }

    /// Empty parser for inner instructions
    ///
    /// Raydium AMM V4 does not emit events through inner instructions or program logs.
    /// All event data is encoded directly in the instruction data itself, which is
    /// parsed by the instruction_parser functions below. This is intentional and
    /// follows the protocol's design where all necessary information is available
    /// in the instruction parameters and accounts.
    ///
    /// This differs from protocols like Raydium CPMM which emit events through logs
    /// that need to be parsed from inner instructions.
    fn empty_parse(
        _data: &[u8],
        _metadata: crate::solana_metadata::SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        Err(crate::error::ParseError::InvalidInstructionType(
            "Raydium AMM V4 does not emit events through inner instructions".to_string(),
        ))
    }

    /// Parse swap base input instruction event
    fn parse_swap_base_input_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: crate::solana_metadata::SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        validate_data_length(data, 16, "RaydiumAmmV4 swap base input instruction")?;
        validate_account_count(accounts, 17, "RaydiumAmmV4 swap base input instruction")?;

        let amount_in = read_u64_le(data, 0).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read amount_in".to_string())
        })?;
        let minimum_amount_out = read_u64_le(data, 8).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to read minimum_amount_out".to_string(),
            )
        })?;

        let amm = safe_get_account(accounts, 1)?;
        let mut event_metadata = metadata;
        let signature =
            crate::metadata_helpers::get_signature(&event_metadata.core).unwrap_or("unknown");
        event_metadata.set_id(format!("{}-{}-swap-in-{}", signature, amm, amount_in));

        Ok(Box::new(RaydiumAmmV4SwapEvent {
            metadata: event_metadata,
            amount_in,
            amount_out: minimum_amount_out, // Use minimum as estimate, actual amount determined on-chain
            direction: SwapDirection::BaseIn,
            amm,
            amm_authority: safe_get_account(accounts, 2)?,
            amm_open_orders: safe_get_account(accounts, 3)?,
            pool_coin_token_account: safe_get_account(accounts, 6)?,
            pool_pc_token_account: safe_get_account(accounts, 7)?,
            serum_program: safe_get_account(accounts, 8)?,
            serum_market: safe_get_account(accounts, 9)?,
            user_coin_token_account: safe_get_account(accounts, 14)?,
            user_pc_token_account: safe_get_account(accounts, 15)?,
            user_owner: safe_get_account(accounts, 16)?,
        }))
    }

    /// Parse swap base output instruction event
    fn parse_swap_base_output_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: crate::solana_metadata::SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        validate_data_length(data, 16, "RaydiumAmmV4 swap base output instruction")?;
        validate_account_count(accounts, 17, "RaydiumAmmV4 swap base output instruction")?;

        let max_amount_in = read_u64_le(data, 0).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read max_amount_in".to_string())
        })?;
        let amount_out = read_u64_le(data, 8).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read amount_out".to_string())
        })?;

        let amm = safe_get_account(accounts, 1)?;
        let mut event_metadata = metadata;
        let signature =
            crate::metadata_helpers::get_signature(&event_metadata.core).unwrap_or("unknown");
        event_metadata.set_id(format!("{}-{}-swap-out-{}", signature, amm, amount_out));

        Ok(Box::new(RaydiumAmmV4SwapEvent {
            metadata: event_metadata,
            amount_in: max_amount_in, // Use maximum as estimate, actual amount determined on-chain
            amount_out,
            direction: SwapDirection::BaseOut,
            amm,
            amm_authority: safe_get_account(accounts, 2)?,
            amm_open_orders: safe_get_account(accounts, 3)?,
            pool_coin_token_account: safe_get_account(accounts, 6)?,
            pool_pc_token_account: safe_get_account(accounts, 7)?,
            serum_program: safe_get_account(accounts, 8)?,
            serum_market: safe_get_account(accounts, 9)?,
            user_coin_token_account: safe_get_account(accounts, 14)?,
            user_pc_token_account: safe_get_account(accounts, 15)?,
            user_owner: safe_get_account(accounts, 16)?,
        }))
    }

    /// Parse deposit instruction event
    fn parse_deposit_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: crate::solana_metadata::SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        validate_data_length(data, 24, "RaydiumAmmV4 deposit instruction")?;
        validate_account_count(accounts, 13, "RaydiumAmmV4 deposit instruction")?;

        let (max_coin_amount, max_pc_amount, base_side) = parse_liquidity_amounts(data)?;

        let amm = safe_get_account(accounts, 1)?;
        let mut event_metadata = metadata;
        let signature =
            crate::metadata_helpers::get_signature(&event_metadata.core).unwrap_or("unknown");
        event_metadata.set_id(format!(
            "{}-{}-deposit-{}-{}",
            signature, amm, max_coin_amount, max_pc_amount
        ));

        Ok(Box::new(RaydiumAmmV4DepositEvent {
            metadata: event_metadata,
            max_coin_amount,
            max_pc_amount,
            base_side,
            token_program: safe_get_account(accounts, 0)?,
            amm,
            amm_authority: safe_get_account(accounts, 2)?,
            amm_open_orders: safe_get_account(accounts, 3)?,
            amm_target_orders: safe_get_account(accounts, 4)?,
            lp_mint_address: safe_get_account(accounts, 5)?,
            pool_coin_token_account: safe_get_account(accounts, 6)?,
            pool_pc_token_account: safe_get_account(accounts, 7)?,
            serum_market: safe_get_account(accounts, 8)?,
            user_coin_token_account: safe_get_account(accounts, 9)?,
            user_pc_token_account: safe_get_account(accounts, 10)?,
            user_lp_token_account: safe_get_account(accounts, 11)?,
            user_owner: safe_get_account(accounts, 12)?,
        }))
    }

    /// Parse initialize2 instruction event
    fn parse_initialize2_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: crate::solana_metadata::SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 25 {
            return Err(crate::error::ParseError::not_enough_bytes(
                25,
                data.len(),
                0,
            ));
        }
        if accounts.len() < 21 {
            return Err(crate::error::ParseError::invalid_account_index(
                20,
                accounts.len() - 1,
            ));
        }

        let nonce = read_u8_le(data, 0).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read nonce".to_string())
        })?;
        let open_time = read_u64_le(data, 1).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read open_time".to_string())
        })?;
        let init_pc_amount = read_u64_le(data, 9).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read init_pc_amount".to_string())
        })?;
        let init_coin_amount = read_u64_le(data, 17).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to read init_coin_amount".to_string(),
            )
        })?;

        let mut event_metadata = metadata;
        let signature =
            crate::metadata_helpers::get_signature(&event_metadata.core).unwrap_or("unknown");
        event_metadata.set_id(format!(
            "{}-{}-init-{}-{}",
            signature, accounts[4], init_coin_amount, init_pc_amount
        ));

        Ok(Box::new(RaydiumAmmV4Initialize2Event {
            metadata: event_metadata,
            nonce,
            open_time,
            init_pc_amount,
            init_coin_amount,
            amm: accounts[4],
            amm_authority: accounts[5],
            amm_open_orders: accounts[6],
            lp_mint_address: accounts[7],
            coin_mint_address: accounts[8],
            pc_mint_address: accounts[9],
            pool_coin_token_account: accounts[10],
            pool_pc_token_account: accounts[11],
            pool_withdraw_queue: accounts[12],
            amm_target_orders: accounts[13],
            pool_lp_token_account: accounts[14],
            pool_temp_lp_token_account: accounts[15],
            serum_program: accounts[16],
            serum_market: accounts[17],
            user_wallet: accounts[18],
        }))
    }

    /// Parse withdraw instruction event
    fn parse_withdraw_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: crate::solana_metadata::SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 8 {
            return Err(crate::error::ParseError::not_enough_bytes(8, data.len(), 0));
        }
        if accounts.len() < 22 {
            return Err(crate::error::ParseError::invalid_account_index(
                21,
                accounts.len() - 1,
            ));
        }

        let amount = read_u64_le(data, 0).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read amount".to_string())
        })?;

        let mut event_metadata = metadata;
        let signature =
            crate::metadata_helpers::get_signature(&event_metadata.core).unwrap_or("unknown");
        event_metadata.set_id(format!("{}-{}-withdraw-{}", signature, accounts[1], amount));

        Ok(Box::new(RaydiumAmmV4WithdrawEvent {
            metadata: event_metadata,
            amount,
            token_program: accounts[0],
            amm: accounts[1],
            amm_authority: accounts[2],
            amm_open_orders: accounts[3],
            amm_target_orders: accounts[4],
            lp_mint_address: accounts[5],
            pool_coin_token_account: accounts[6],
            pool_pc_token_account: accounts[7],
            pool_withdraw_queue: accounts[8],
            pool_temp_lp_token_account: accounts[9],
            serum_program: accounts[10],
            serum_market: accounts[11],
            serum_coin_vault_account: accounts[12],
            serum_pc_vault_account: accounts[13],
            serum_vault_signer: accounts[14],
            user_lp_token_account: accounts[15],
            user_coin_token_account: accounts[16],
            user_pc_token_account: accounts[17],
            user_owner: accounts[18],
            serum_event_queue: accounts[19],
            serum_bids: accounts[20],
            serum_asks: accounts[21],
        }))
    }

    /// Parse withdraw PNL instruction event
    fn parse_withdraw_pnl_instruction(
        _data: &[u8],
        accounts: &[Pubkey],
        metadata: crate::solana_metadata::SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if accounts.len() < 17 {
            return Err(crate::error::ParseError::invalid_account_index(
                16,
                accounts.len() - 1,
            ));
        }

        let mut event_metadata = metadata;
        let signature =
            crate::metadata_helpers::get_signature(&event_metadata.core).unwrap_or("unknown");
        event_metadata.set_id(format!("{}-{}-withdraw-pnl", signature, accounts[1]));

        Ok(Box::new(RaydiumAmmV4WithdrawPnlEvent {
            metadata: event_metadata,
            token_program: accounts[0],
            amm: accounts[1],
            amm_config: accounts[2],
            amm_authority: accounts[3],
            amm_open_orders: accounts[4],
            pool_coin_token_account: accounts[5],
            pool_pc_token_account: accounts[6],
            coin_pnl_token_account: accounts[7],
            pc_pnl_token_account: accounts[8],
            pnl_owner_account: accounts[9],
            amm_target_orders: accounts[10],
            serum_program: accounts[11],
            serum_market: accounts[12],
            serum_event_queue: accounts[13],
            serum_coin_vault_account: accounts[14],
            serum_pc_vault_account: accounts[15],
            serum_vault_signer: accounts[16],
        }))
    }
}

// Implement the new core EventParser trait
#[async_trait::async_trait]
impl EventParser for RaydiumAmmV4EventParser {
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
        ParserInfo::new("raydium_amm_v4_parser".to_string(), "1.0.0".to_string())
            .with_kind(EventKind::Custom("raydium_amm_v4_swap".to_string()))
            .with_kind(EventKind::Custom("raydium_amm_v4_deposit".to_string()))
            .with_kind(EventKind::Custom("raydium_amm_v4_withdraw".to_string()))
            .with_kind(EventKind::Custom("raydium_amm_v4_initialize".to_string()))
            .with_format("solana_instruction".to_string())
    }
}

// Keep legacy implementation for backward compatibility
#[async_trait::async_trait]
impl LegacyEventParser for RaydiumAmmV4EventParser {
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
    use crate::events::common::{EventType, ProtocolType};
    use solana_message::compiled_instruction::CompiledInstruction;
    use solana_transaction_status::UiCompiledInstruction;
    use std::str::FromStr;

    fn create_test_metadata() -> crate::solana_metadata::SolanaEventMetadata {
        let core = crate::metadata_helpers::create_core_metadata(
            "test-signature-test-index".to_string(),
            riglr_events_core::EventKind::Transaction,
            "solana".to_string(),
            Some(1640995200),
        );

        crate::solana_metadata::SolanaEventMetadata::new(
            "test-signature-test-index".to_string(),
            100,
            EventType::RaydiumAmmV4SwapBaseIn,
            ProtocolType::RaydiumAmmV4,
            "test-index".to_string(),
            1640995200000,
            core,
        )
    }

    fn create_test_accounts(count: usize) -> Vec<Pubkey> {
        (0..count)
            .map(|i| {
                Pubkey::from_str(&format!("1111111111111111111111111111111{:02}", i + 12))
                    .unwrap_or_else(|_| Pubkey::default())
            })
            .collect()
    }

    #[test]
    fn test_raydium_amm_v4_program_id_constant() {
        let expected_program_id =
            Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap();
        assert_eq!(RAYDIUM_AMM_V4_PROGRAM_ID, expected_program_id);
    }

    #[test]
    fn test_new_creates_parser_with_default() {
        let parser = RaydiumAmmV4EventParser::default();
        let configs = parser.instruction_configs();
        assert!(!configs.is_empty());

        // Check that all expected discriminators are present
        let expected_discriminators = vec![
            discriminators::SWAP_BASE_IN,
            discriminators::SWAP_BASE_OUT,
            discriminators::DEPOSIT,
            discriminators::INITIALIZE2,
            discriminators::WITHDRAW,
            discriminators::WITHDRAW_PNL,
        ];

        for discriminator in expected_discriminators {
            assert!(configs.contains_key(discriminator));
        }
    }

    #[test]
    fn test_default_creates_parser_with_all_configs() {
        let parser = RaydiumAmmV4EventParser::default();
        let configs = parser.instruction_configs();

        // Should have 6 configurations (one for each instruction type)
        assert_eq!(configs.len(), 6);

        // Check that supported program IDs include the Raydium AMM V4 program ID
        let supported_ids = parser.supported_program_ids();
        assert!(supported_ids.contains(&RAYDIUM_AMM_V4_PROGRAM_ID));
    }

    #[test]
    fn test_should_handle_returns_true_for_raydium_program_id() {
        let parser = RaydiumAmmV4EventParser::default();
        assert!(parser.should_handle(&RAYDIUM_AMM_V4_PROGRAM_ID));
    }

    #[test]
    fn test_should_handle_returns_false_for_other_program_id() {
        let parser = RaydiumAmmV4EventParser::default();
        let other_program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        assert!(!parser.should_handle(&other_program_id));
    }

    #[test]
    fn test_empty_parse_returns_none() {
        let data = vec![1, 2, 3, 4];
        let metadata = create_test_metadata();
        let result = RaydiumAmmV4EventParser::empty_parse(&data, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_parse_with_empty_data_returns_none() {
        let data = vec![];
        let metadata = create_test_metadata();
        let result = RaydiumAmmV4EventParser::empty_parse(&data, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_base_input_instruction_when_valid_should_return_event() {
        let data = vec![
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // amount_in: 4096
            0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // minimum_amount_out: 2048
        ];
        let accounts = create_test_accounts(17);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_swap_base_input_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();

        // Downcast to check specific event type
        let swap_event = event
            .as_any()
            .downcast_ref::<RaydiumAmmV4SwapEvent>()
            .unwrap();
        assert_eq!(swap_event.amount_in, 4096);
        assert_eq!(swap_event.amount_out, 2048);
        assert!(matches!(swap_event.direction, SwapDirection::BaseIn));
        assert_eq!(swap_event.amm, accounts[1]);
        assert_eq!(swap_event.user_owner, accounts[16]);
    }

    #[test]
    fn test_parse_swap_base_input_instruction_when_insufficient_data_should_return_none() {
        let data = vec![0x00, 0x10, 0x00]; // Only 3 bytes instead of required 16
        let accounts = create_test_accounts(17);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_swap_base_input_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_base_input_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        let accounts = create_test_accounts(16); // Only 16 accounts instead of required 17
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_swap_base_input_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_base_input_instruction_when_empty_data_should_return_none() {
        let data = vec![];
        let accounts = create_test_accounts(17);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_swap_base_input_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_base_input_instruction_when_max_values_should_work() {
        let data = vec![
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // amount_in: u64::MAX
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // minimum_amount_out: u64::MAX
        ];
        let accounts = create_test_accounts(17);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_swap_base_input_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let swap_event = event
            .as_any()
            .downcast_ref::<RaydiumAmmV4SwapEvent>()
            .unwrap();
        assert_eq!(swap_event.amount_in, u64::MAX);
        assert_eq!(swap_event.amount_out, u64::MAX);
    }

    #[test]
    fn test_parse_swap_base_output_instruction_when_valid_should_return_event() {
        let data = vec![
            0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // max_amount_in: 8192
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // amount_out: 4096
        ];
        let accounts = create_test_accounts(17);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_swap_base_output_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let swap_event = event
            .as_any()
            .downcast_ref::<RaydiumAmmV4SwapEvent>()
            .unwrap();
        assert_eq!(swap_event.amount_in, 8192);
        assert_eq!(swap_event.amount_out, 4096);
        assert!(matches!(swap_event.direction, SwapDirection::BaseOut));
        assert_eq!(swap_event.amm, accounts[1]);
    }

    #[test]
    fn test_parse_swap_base_output_instruction_when_insufficient_data_should_return_none() {
        let data = vec![0x00, 0x20]; // Only 2 bytes instead of required 16
        let accounts = create_test_accounts(17);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_swap_base_output_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_base_output_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![
            0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        let accounts = create_test_accounts(10); // Only 10 accounts instead of required 17
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_swap_base_output_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_deposit_instruction_when_valid_should_return_event() {
        let data = vec![
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // max_coin_amount: 4096
            0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // max_pc_amount: 2048
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // base_side: 1
        ];
        let accounts = create_test_accounts(13);
        let metadata = create_test_metadata();

        let result = RaydiumAmmV4EventParser::parse_deposit_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let deposit_event = event
            .as_any()
            .downcast_ref::<RaydiumAmmV4DepositEvent>()
            .unwrap();
        assert_eq!(deposit_event.max_coin_amount, 4096);
        assert_eq!(deposit_event.max_pc_amount, 2048);
        assert_eq!(deposit_event.base_side, 1);
        assert_eq!(deposit_event.amm, accounts[1]);
        assert_eq!(deposit_event.user_owner, accounts[12]);
    }

    #[test]
    fn test_parse_deposit_instruction_when_insufficient_data_should_return_none() {
        let data = vec![0x00, 0x10]; // Only 2 bytes instead of required 24
        let accounts = create_test_accounts(13);
        let metadata = create_test_metadata();

        let result = RaydiumAmmV4EventParser::parse_deposit_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_deposit_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let accounts = create_test_accounts(12); // Only 12 accounts instead of required 13
        let metadata = create_test_metadata();

        let result = RaydiumAmmV4EventParser::parse_deposit_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_deposit_instruction_when_zero_amounts_should_work() {
        let data = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // max_coin_amount: 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // max_pc_amount: 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // base_side: 0
        ];
        let accounts = create_test_accounts(13);
        let metadata = create_test_metadata();

        let result = RaydiumAmmV4EventParser::parse_deposit_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let deposit_event = event
            .as_any()
            .downcast_ref::<RaydiumAmmV4DepositEvent>()
            .unwrap();
        assert_eq!(deposit_event.max_coin_amount, 0);
        assert_eq!(deposit_event.max_pc_amount, 0);
        assert_eq!(deposit_event.base_side, 0);
    }

    #[test]
    fn test_parse_initialize2_instruction_when_valid_should_return_event() {
        let data = vec![
            0x05, // nonce: 5
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // open_time: 4096
            0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // init_pc_amount: 2048
            0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // init_coin_amount: 1024
        ];
        let accounts = create_test_accounts(21);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_initialize2_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let init_event = event
            .as_any()
            .downcast_ref::<RaydiumAmmV4Initialize2Event>()
            .unwrap();
        assert_eq!(init_event.nonce, 5);
        assert_eq!(init_event.open_time, 4096);
        assert_eq!(init_event.init_pc_amount, 2048);
        assert_eq!(init_event.init_coin_amount, 1024);
        assert_eq!(init_event.amm, accounts[4]);
        assert_eq!(init_event.user_wallet, accounts[18]);
    }

    #[test]
    fn test_parse_initialize2_instruction_when_insufficient_data_should_return_none() {
        let data = vec![0x05, 0x00, 0x10]; // Only 3 bytes instead of required 25
        let accounts = create_test_accounts(21);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_initialize2_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_initialize2_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![
            0x05, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let accounts = create_test_accounts(20); // Only 20 accounts instead of required 21
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_initialize2_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_initialize2_instruction_when_max_nonce_should_work() {
        let data = vec![
            0xFF, // nonce: 255 (u8::MAX)
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // open_time: u64::MAX
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // init_pc_amount: u64::MAX
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // init_coin_amount: u64::MAX
        ];
        let accounts = create_test_accounts(21);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_initialize2_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let init_event = event
            .as_any()
            .downcast_ref::<RaydiumAmmV4Initialize2Event>()
            .unwrap();
        assert_eq!(init_event.nonce, 255);
        assert_eq!(init_event.open_time, u64::MAX);
        assert_eq!(init_event.init_pc_amount, u64::MAX);
        assert_eq!(init_event.init_coin_amount, u64::MAX);
    }

    #[test]
    fn test_parse_withdraw_instruction_when_valid_should_return_event() {
        let data = vec![
            0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // amount: 8192
        ];
        let accounts = create_test_accounts(22);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_withdraw_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let withdraw_event = event
            .as_any()
            .downcast_ref::<RaydiumAmmV4WithdrawEvent>()
            .unwrap();
        assert_eq!(withdraw_event.amount, 8192);
        assert_eq!(withdraw_event.token_program, accounts[0]);
        assert_eq!(withdraw_event.amm, accounts[1]);
        assert_eq!(withdraw_event.user_owner, accounts[18]);
        assert_eq!(withdraw_event.serum_asks, accounts[21]);
    }

    #[test]
    fn test_parse_withdraw_instruction_when_insufficient_data_should_return_none() {
        let data = vec![0x00, 0x20]; // Only 2 bytes instead of required 8
        let accounts = create_test_accounts(22);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_withdraw_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_withdraw_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let accounts = create_test_accounts(21); // Only 21 accounts instead of required 22
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_withdraw_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_withdraw_instruction_when_zero_amount_should_work() {
        let data = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // amount: 0
        ];
        let accounts = create_test_accounts(22);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_withdraw_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let withdraw_event = event
            .as_any()
            .downcast_ref::<RaydiumAmmV4WithdrawEvent>()
            .unwrap();
        assert_eq!(withdraw_event.amount, 0);
    }

    #[test]
    fn test_parse_withdraw_instruction_when_empty_data_should_return_none() {
        let data = vec![];
        let accounts = create_test_accounts(22);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_withdraw_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_withdraw_pnl_instruction_when_valid_should_return_event() {
        let data = vec![0x01, 0x02, 0x03]; // Data is not used for withdraw PNL, any data works
        let accounts = create_test_accounts(17);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_withdraw_pnl_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let pnl_event = event
            .as_any()
            .downcast_ref::<RaydiumAmmV4WithdrawPnlEvent>()
            .unwrap();
        assert_eq!(pnl_event.token_program, accounts[0]);
        assert_eq!(pnl_event.amm, accounts[1]);
        assert_eq!(pnl_event.amm_config, accounts[2]);
        assert_eq!(pnl_event.serum_vault_signer, accounts[16]);
    }

    #[test]
    fn test_parse_withdraw_pnl_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![0x01, 0x02, 0x03];
        let accounts = create_test_accounts(16); // Only 16 accounts instead of required 17
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_withdraw_pnl_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_withdraw_pnl_instruction_when_empty_data_should_work() {
        let data = vec![]; // Empty data should work since data is not used
        let accounts = create_test_accounts(17);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_withdraw_pnl_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_withdraw_pnl_instruction_when_empty_accounts_should_return_none() {
        let data = vec![0x01, 0x02, 0x03];
        let accounts = vec![]; // Empty accounts
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_withdraw_pnl_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_events_from_inner_instruction_delegates_to_inner() {
        let parser = RaydiumAmmV4EventParser::default();

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: "test".to_string(),
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test-signature",
            slot: 100,
            block_time: Some(1640995200),
            program_received_time_ms: 1640995200000,
            index: "test-index".to_string(),
        };
        let result = parser.parse_events_from_inner_instruction(&params);

        // Should return empty vector since inner instructions aren't handled
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_events_from_instruction_delegates_to_inner() {
        let parser = RaydiumAmmV4EventParser::default();

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: vec![1, 2, 3, 4],
        };
        let accounts = create_test_accounts(3);

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test-signature",
            slot: 100,
            block_time: Some(1640995200),
            program_received_time_ms: 1640995200000,
            index: "test-index".to_string(),
        };
        let result = parser.parse_events_from_instruction(&params);

        // Should return empty vector since instruction doesn't match any discriminator
        assert!(result.is_empty());
    }

    #[test]
    fn test_inner_instruction_configs_delegates_to_inner() {
        let parser = RaydiumAmmV4EventParser::default();
        let configs = parser.inner_instruction_configs();

        // Should be empty since Raydium AMM V4 doesn't use inner instruction events
        assert!(configs.is_empty());
    }

    #[test]
    fn test_instruction_configs_returns_all_discriminators() {
        let parser = RaydiumAmmV4EventParser::default();
        let configs = parser.instruction_configs();

        // Should have exactly 6 configurations
        assert_eq!(configs.len(), 6);

        // Check each expected discriminator exists
        assert!(configs.contains_key(discriminators::SWAP_BASE_IN));
        assert!(configs.contains_key(discriminators::SWAP_BASE_OUT));
        assert!(configs.contains_key(discriminators::DEPOSIT));
        assert!(configs.contains_key(discriminators::INITIALIZE2));
        assert!(configs.contains_key(discriminators::WITHDRAW));
        assert!(configs.contains_key(discriminators::WITHDRAW_PNL));
    }

    #[test]
    fn test_supported_program_ids_contains_raydium_program_id() {
        let parser = RaydiumAmmV4EventParser::default();
        let supported_ids = parser.supported_program_ids();

        assert_eq!(supported_ids.len(), 1);
        assert_eq!(supported_ids[0], RAYDIUM_AMM_V4_PROGRAM_ID);
    }

    #[test]
    fn test_metadata_id_generation_for_swap_base_in() {
        let data = vec![
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        let accounts = create_test_accounts(17);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_swap_base_input_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let swap_event = event
            .as_any()
            .downcast_ref::<RaydiumAmmV4SwapEvent>()
            .unwrap();

        // Check that ID was generated with expected format
        let expected_id_prefix = format!("test-signature-{}-swap-in-", accounts[1]);
        assert!(swap_event.metadata.id.starts_with(&expected_id_prefix));
    }

    #[test]
    fn test_metadata_id_generation_for_swap_base_out() {
        let data = vec![
            0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        let accounts = create_test_accounts(17);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_swap_base_output_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let swap_event = event
            .as_any()
            .downcast_ref::<RaydiumAmmV4SwapEvent>()
            .unwrap();

        // Check that ID was generated with expected format
        let expected_id_prefix = format!("test-signature-{}-swap-out-", accounts[1]);
        assert!(swap_event.metadata.id.starts_with(&expected_id_prefix));
    }

    #[test]
    fn test_metadata_id_generation_for_deposit() {
        let data = vec![
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let accounts = create_test_accounts(13);
        let metadata = create_test_metadata();

        let result = RaydiumAmmV4EventParser::parse_deposit_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let deposit_event = event
            .as_any()
            .downcast_ref::<RaydiumAmmV4DepositEvent>()
            .unwrap();

        // Check that ID was generated with expected format
        let expected_id_prefix = format!("test-signature-{}-deposit-", accounts[1]);
        assert!(deposit_event.metadata.id.starts_with(&expected_id_prefix));
    }

    #[test]
    fn test_metadata_id_generation_for_initialize2() {
        let data = vec![
            0x05, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let accounts = create_test_accounts(21);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_initialize2_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let init_event = event
            .as_any()
            .downcast_ref::<RaydiumAmmV4Initialize2Event>()
            .unwrap();

        // Check that ID was generated with expected format
        let expected_id_prefix = format!("test-signature-{}-init-", accounts[4]);
        assert!(init_event.metadata.id.starts_with(&expected_id_prefix));
    }

    #[test]
    fn test_metadata_id_generation_for_withdraw() {
        let data = vec![0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let accounts = create_test_accounts(22);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_withdraw_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let withdraw_event = event
            .as_any()
            .downcast_ref::<RaydiumAmmV4WithdrawEvent>()
            .unwrap();

        // Check that ID was generated with expected format
        let expected_id_prefix = format!("test-signature-{}-withdraw-", accounts[1]);
        assert!(withdraw_event.metadata.id.starts_with(&expected_id_prefix));
    }

    #[test]
    fn test_metadata_id_generation_for_withdraw_pnl() {
        let data = vec![0x01, 0x02, 0x03];
        let accounts = create_test_accounts(17);
        let metadata = create_test_metadata();

        let result =
            RaydiumAmmV4EventParser::parse_withdraw_pnl_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
        let event = result.unwrap();
        let pnl_event = event
            .as_any()
            .downcast_ref::<RaydiumAmmV4WithdrawPnlEvent>()
            .unwrap();

        // Check that ID was generated with expected format
        let expected_id = format!("test-signature-{}-withdraw-pnl", accounts[1]);
        assert_eq!(pnl_event.metadata.id, expected_id);
    }

    #[test]
    fn test_all_parser_functions_handle_boundary_conditions() {
        let metadata = create_test_metadata();

        // Test with minimum required data/accounts
        let min_swap_data = vec![0u8; 16];
        let min_swap_accounts = create_test_accounts(17);
        assert!(RaydiumAmmV4EventParser::parse_swap_base_input_instruction(
            &min_swap_data,
            &min_swap_accounts,
            metadata.clone()
        )
        .is_ok());

        let min_deposit_data = vec![0u8; 24];
        let min_deposit_accounts = create_test_accounts(13);
        assert!(RaydiumAmmV4EventParser::parse_deposit_instruction(
            &min_deposit_data,
            &min_deposit_accounts,
            metadata.clone()
        )
        .is_ok());

        let min_init_data = vec![0u8; 25];
        let min_init_accounts = create_test_accounts(21);
        assert!(RaydiumAmmV4EventParser::parse_initialize2_instruction(
            &min_init_data,
            &min_init_accounts,
            metadata.clone()
        )
        .is_ok());

        let min_withdraw_data = vec![0u8; 8];
        let min_withdraw_accounts = create_test_accounts(22);
        assert!(RaydiumAmmV4EventParser::parse_withdraw_instruction(
            &min_withdraw_data,
            &min_withdraw_accounts,
            metadata.clone()
        )
        .is_ok());

        let min_pnl_accounts = create_test_accounts(17);
        assert!(RaydiumAmmV4EventParser::parse_withdraw_pnl_instruction(
            &[],
            &min_pnl_accounts,
            metadata
        )
        .is_ok());
    }
}
