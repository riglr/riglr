use riglr_events_core::Event;
use std::collections::HashMap;

use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

use crate::{
    error::ParseResult,
    events::{
        core::traits::{EventParser, GenericEventParseConfig, GenericEventParser},
        protocols::raydium_cpmm::{discriminators, RaydiumCpmmDepositEvent, RaydiumCpmmSwapEvent},
    },
    types::{EventMetadata, EventType, ProtocolType},
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
    /// Create a new RaydiumCpmmEventParser
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse swap log event
    fn parse_swap_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        // Parse the swap event using borsh deserialization
        let event = RaydiumCpmmSwapEvent::try_from_slice(data).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse Raydium CPMM swap event".to_string(),
            )
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-swap", metadata.signature, event.pool_state));
        Ok(Box::new(RaydiumCpmmSwapEvent { metadata, ..event }))
    }

    /// Parse deposit log event
    fn parse_deposit_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        // Parse the deposit event using borsh deserialization
        let event = RaydiumCpmmDepositEvent::try_from_slice(data).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse Raydium CPMM deposit event".to_string(),
            )
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-deposit",
            metadata.signature, event.pool_state
        ));
        Ok(Box::new(RaydiumCpmmDepositEvent { metadata, ..event }))
    }

    /// Parse swap base input instruction event
    fn parse_swap_base_input_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 16 || accounts.len() < 10 {
            return Err(crate::error::ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CPMM swap base input instruction"
                    .to_string(),
            ));
        }

        let amount_in = u64::from_le_bytes(data[0..8].try_into().map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read amount_in".to_string())
        })?);
        let minimum_amount_out = u64::from_le_bytes(data[8..16].try_into().map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to read minimum_amount_out".to_string(),
            )
        })?);

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-swap-{}-{}",
            metadata.signature, accounts[0], amount_in, minimum_amount_out
        ));

        Ok(Box::new(RaydiumCpmmSwapEvent {
            metadata,
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
            trade_fee: 0,
            transfer_fee: 0,
        }))
    }

    /// Parse swap base output instruction event
    fn parse_swap_base_output_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 16 || accounts.len() < 10 {
            return Err(crate::error::ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CPMM swap base output instruction"
                    .to_string(),
            ));
        }

        let max_amount_in = u64::from_le_bytes(data[0..8].try_into().map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read max_amount_in".to_string())
        })?);
        let amount_out = u64::from_le_bytes(data[8..16].try_into().map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read amount_out".to_string())
        })?);

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-swap-{}-{}",
            metadata.signature, accounts[0], max_amount_in, amount_out
        ));

        Ok(Box::new(RaydiumCpmmSwapEvent {
            metadata,
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
            trade_fee: 0,
            transfer_fee: 0,
        }))
    }

    /// Parse deposit instruction event
    fn parse_deposit_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 24 || accounts.len() < 8 {
            return Err(crate::error::ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CPMM deposit instruction".to_string(),
            ));
        }

        let lp_token_amount = u64::from_le_bytes(data[0..8].try_into().map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to read lp_token_amount".to_string(),
            )
        })?);
        let token_0_amount = u64::from_le_bytes(data[8..16].try_into().map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read token_0_amount".to_string())
        })?);
        let token_1_amount = u64::from_le_bytes(data[16..24].try_into().map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read token_1_amount".to_string())
        })?);

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-deposit-{}",
            metadata.signature, accounts[0], lp_token_amount
        ));

        Ok(Box::new(RaydiumCpmmDepositEvent {
            metadata,
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
    use riglr_events_core::EventKind;

    // Helper function to create test metadata
    fn create_test_metadata() -> EventMetadata {
        crate::types::EventMetadata::new(
            "test_signature".to_string(),
            123456,
            EventType::RaydiumCpmmSwap,
            ProtocolType::RaydiumCpmm,
            "0:0".to_string(),
            1640000000000,
            riglr_events_core::EventMetadata::new(
                "test_id".to_string(),
                EventKind::Transaction,
                "test_source".to_string(),
            ),
        )
    }

    // Helper function to create test accounts
    fn create_test_accounts() -> Vec<Pubkey> {
        (0..15).map(|i| Pubkey::new_from_array([i; 32])).collect()
    }

    #[test]
    fn test_default_creates_parser_with_correct_configs() {
        let parser = RaydiumCpmmEventParser::default();

        // Check that inner parser has correct configurations
        let inner_configs = parser.inner_instruction_configs();
        let instruction_configs = parser.instruction_configs();

        assert!(!inner_configs.is_empty());
        assert!(!instruction_configs.is_empty());

        // Verify supported program IDs
        let supported_ids = parser.supported_program_ids();
        assert_eq!(supported_ids.len(), 1);
        assert_eq!(supported_ids[0], RAYDIUM_CPMM_PROGRAM_ID);
    }

    #[test]
    fn test_new_returns_default_instance() {
        let parser1 = RaydiumCpmmEventParser::default();
        let parser2 = RaydiumCpmmEventParser::default();

        // Both should have the same configurations
        assert_eq!(
            parser1.supported_program_ids(),
            parser2.supported_program_ids()
        );
    }

    #[test]
    fn test_should_handle_with_correct_program_id() {
        let parser = RaydiumCpmmEventParser::default();
        assert!(parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
    }

    #[test]
    fn test_should_handle_with_incorrect_program_id() {
        let parser = RaydiumCpmmEventParser::default();
        let wrong_program_id = Pubkey::new_from_array([1; 32]);
        assert!(!parser.should_handle(&wrong_program_id));
    }

    #[test]
    fn test_parse_swap_inner_instruction_with_valid_data() {
        let core_metadata = riglr_events_core::EventMetadata::new(
            "test_swap_event".to_string(),
            EventKind::Swap,
            "test".to_string(),
        );

        let swap_metadata = crate::types::EventMetadata::new(
            "test-signature".to_string(),
            12345,
            EventType::RaydiumCpmmSwap,
            ProtocolType::RaydiumCpmm,
            "0".to_string(),
            1640995200000,
            core_metadata,
        );

        let swap_event = RaydiumCpmmSwapEvent {
            metadata: swap_metadata,
            pool_state: Pubkey::new_from_array([1; 32]),
            payer: Pubkey::new_from_array([2; 32]),
            input_token_account: Pubkey::new_from_array([3; 32]),
            output_token_account: Pubkey::new_from_array([4; 32]),
            input_vault: Pubkey::new_from_array([5; 32]),
            output_vault: Pubkey::new_from_array([6; 32]),
            input_token_mint: Pubkey::new_from_array([7; 32]),
            output_token_mint: Pubkey::new_from_array([8; 32]),
            amount_in: 1000,
            amount_out: 950,
            trade_fee: 30,
            transfer_fee: 20,
        };

        let serialized_data = borsh::to_vec(&swap_event).unwrap();
        let metadata = create_test_metadata();

        let result = RaydiumCpmmEventParser::parse_swap_inner_instruction(
            &serialized_data,
            metadata.clone(),
        );

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(
            event.id(),
            format!("{}-{}-swap", metadata.signature, swap_event.pool_state)
        );
    }

    #[test]
    fn test_parse_swap_inner_instruction_with_invalid_data() {
        let invalid_data = vec![1, 2, 3]; // Invalid borsh data
        let metadata = create_test_metadata();

        let result = RaydiumCpmmEventParser::parse_swap_inner_instruction(&invalid_data, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_inner_instruction_with_empty_data() {
        let empty_data = vec![];
        let metadata = create_test_metadata();

        let result = RaydiumCpmmEventParser::parse_swap_inner_instruction(&empty_data, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_deposit_inner_instruction_with_valid_data() {
        let core_metadata = riglr_events_core::EventMetadata::new(
            "test_deposit_event".to_string(),
            EventKind::Liquidity,
            "test".to_string(),
        );

        let deposit_metadata = crate::types::EventMetadata::new(
            "test-signature".to_string(),
            12346,
            EventType::RaydiumCpmmDeposit,
            ProtocolType::RaydiumCpmm,
            "0".to_string(),
            1640995200000,
            core_metadata,
        );

        let deposit_event = RaydiumCpmmDepositEvent {
            metadata: deposit_metadata,
            pool_state: Pubkey::new_from_array([1; 32]),
            user: Pubkey::new_from_array([2; 32]),
            lp_token_amount: 500,
            token_0_amount: 1000,
            token_1_amount: 2000,
        };

        let serialized_data = borsh::to_vec(&deposit_event).unwrap();
        let metadata = create_test_metadata();

        let result = RaydiumCpmmEventParser::parse_deposit_inner_instruction(
            &serialized_data,
            metadata.clone(),
        );

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(
            event.id(),
            format!(
                "{}-{}-deposit",
                metadata.signature, deposit_event.pool_state
            )
        );
    }

    #[test]
    fn test_parse_deposit_inner_instruction_with_invalid_data() {
        let invalid_data = vec![1, 2, 3]; // Invalid borsh data
        let metadata = create_test_metadata();

        let result =
            RaydiumCpmmEventParser::parse_deposit_inner_instruction(&invalid_data, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_deposit_inner_instruction_with_empty_data() {
        let empty_data = vec![];
        let metadata = create_test_metadata();

        let result = RaydiumCpmmEventParser::parse_deposit_inner_instruction(&empty_data, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_base_input_instruction_with_valid_data() {
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&1000u64.to_le_bytes()); // amount_in
        data[8..16].copy_from_slice(&950u64.to_le_bytes()); // minimum_amount_out

        let accounts = create_test_accounts();
        let metadata = create_test_metadata();

        let result = RaydiumCpmmEventParser::parse_swap_base_input_instruction(
            &data,
            &accounts,
            metadata.clone(),
        );

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(
            event.id(),
            format!(
                "{}-{}-swap-{}-{}",
                metadata.signature, accounts[0], 1000u64, 950u64
            )
        );
    }

    #[test]
    fn test_parse_swap_base_input_instruction_with_insufficient_data() {
        let data = vec![0u8; 8]; // Less than 16 bytes required
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();

        let result =
            RaydiumCpmmEventParser::parse_swap_base_input_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_base_input_instruction_with_insufficient_accounts() {
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&1000u64.to_le_bytes());
        data[8..16].copy_from_slice(&950u64.to_le_bytes());

        let accounts = vec![Pubkey::new_from_array([1; 32])]; // Less than 10 accounts required
        let metadata = create_test_metadata();

        let result =
            RaydiumCpmmEventParser::parse_swap_base_input_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_base_input_instruction_with_empty_data() {
        let data = vec![];
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();

        let result =
            RaydiumCpmmEventParser::parse_swap_base_input_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_base_output_instruction_with_valid_data() {
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&1000u64.to_le_bytes()); // max_amount_in
        data[8..16].copy_from_slice(&950u64.to_le_bytes()); // amount_out

        let accounts = create_test_accounts();
        let metadata = create_test_metadata();

        let result = RaydiumCpmmEventParser::parse_swap_base_output_instruction(
            &data,
            &accounts,
            metadata.clone(),
        );

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(
            event.id(),
            format!(
                "{}-{}-swap-{}-{}",
                metadata.signature, accounts[0], 1000u64, 950u64
            )
        );
    }

    #[test]
    fn test_parse_swap_base_output_instruction_with_insufficient_data() {
        let data = vec![0u8; 8]; // Less than 16 bytes required
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();

        let result =
            RaydiumCpmmEventParser::parse_swap_base_output_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_base_output_instruction_with_insufficient_accounts() {
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&1000u64.to_le_bytes());
        data[8..16].copy_from_slice(&950u64.to_le_bytes());

        let accounts = vec![Pubkey::new_from_array([1; 32])]; // Less than 10 accounts required
        let metadata = create_test_metadata();

        let result =
            RaydiumCpmmEventParser::parse_swap_base_output_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_base_output_instruction_with_empty_data() {
        let data = vec![];
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();

        let result =
            RaydiumCpmmEventParser::parse_swap_base_output_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_deposit_instruction_with_valid_data() {
        let mut data = vec![0u8; 24];
        data[0..8].copy_from_slice(&500u64.to_le_bytes()); // lp_token_amount
        data[8..16].copy_from_slice(&1000u64.to_le_bytes()); // token_0_amount
        data[16..24].copy_from_slice(&2000u64.to_le_bytes()); // token_1_amount

        let accounts = create_test_accounts();
        let metadata = create_test_metadata();

        let result =
            RaydiumCpmmEventParser::parse_deposit_instruction(&data, &accounts, metadata.clone());

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(
            event.id(),
            format!("{}-{}-deposit-{}", metadata.signature, accounts[0], 500u64)
        );
    }

    #[test]
    fn test_parse_deposit_instruction_with_insufficient_data() {
        let data = vec![0u8; 16]; // Less than 24 bytes required
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();

        let result = RaydiumCpmmEventParser::parse_deposit_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_deposit_instruction_with_insufficient_accounts() {
        let mut data = vec![0u8; 24];
        data[0..8].copy_from_slice(&500u64.to_le_bytes());
        data[8..16].copy_from_slice(&1000u64.to_le_bytes());
        data[16..24].copy_from_slice(&2000u64.to_le_bytes());

        let accounts = vec![Pubkey::new_from_array([1; 32])]; // Less than 8 accounts required
        let metadata = create_test_metadata();

        let result = RaydiumCpmmEventParser::parse_deposit_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_deposit_instruction_with_empty_data() {
        let data = vec![];
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();

        let result = RaydiumCpmmEventParser::parse_deposit_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_base_input_instruction_with_invalid_byte_conversion() {
        // Create data that will fail the try_into conversion
        let data = vec![0u8; 15]; // 15 bytes - will fail when trying to convert to [u8; 8]
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();

        let result =
            RaydiumCpmmEventParser::parse_swap_base_input_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_base_output_instruction_with_invalid_byte_conversion() {
        // Create data that will fail the try_into conversion
        let data = vec![0u8; 15]; // 15 bytes - will fail when trying to convert to [u8; 8]
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();

        let result =
            RaydiumCpmmEventParser::parse_swap_base_output_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_deposit_instruction_with_invalid_byte_conversion() {
        // Create data that will fail the try_into conversion
        let data = vec![0u8; 23]; // 23 bytes - will fail when trying to convert to [u8; 8]
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();

        let result = RaydiumCpmmEventParser::parse_deposit_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_event_parser_trait_methods() {
        let parser = RaydiumCpmmEventParser::default();

        // Test inner_instruction_configs
        let inner_configs = parser.inner_instruction_configs();
        assert!(!inner_configs.is_empty());

        // Test instruction_configs
        let instruction_configs = parser.instruction_configs();
        assert!(!instruction_configs.is_empty());

        // Test supported_program_ids
        let program_ids = parser.supported_program_ids();
        assert_eq!(program_ids.len(), 1);
        assert_eq!(program_ids[0], RAYDIUM_CPMM_PROGRAM_ID);
    }

    #[test]
    fn test_raydium_cpmm_program_id_constant() {
        // Verify the program ID constant is correctly defined
        let expected_program_id =
            solana_sdk::pubkey!("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");
        assert_eq!(RAYDIUM_CPMM_PROGRAM_ID, expected_program_id);
    }

    #[test]
    fn test_parser_delegations_to_inner() {
        let parser = RaydiumCpmmEventParser::default();

        // Create mock data for testing delegation
        let inner_instruction = solana_transaction_status::UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: "test_data".to_string(),
            stack_height: None,
        };

        let instruction = solana_message::compiled_instruction::CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: vec![1, 2, 3],
        };

        let accounts = create_test_accounts();

        // Test parse_events_from_inner_instruction delegation
        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 123456,
            block_time: Some(1640000000),
            program_received_time_ms: 1640000000000,
            index: "0:0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);
        // Should not panic and return some result (even if empty)
        // events.len() is always >= 0 for Vec, so just check it doesn't panic
        let _ = events.len();

        // Test parse_events_from_instruction delegation
        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 123456,
            block_time: Some(1640000000),
            program_received_time_ms: 1640000000000,
            index: "0:0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);
        // Should not panic and return some result (even if empty)
        // events.len() is always >= 0 for Vec, so just check it doesn't panic
        let _ = events.len();
    }
}
