use super::{
    events::JupiterSwapEvent,
    types::{
        jupiter_v6_program_id, JupiterSwapData, RoutePlan, SharedAccountsExactOutRouteData,
        SharedAccountsRouteData, EXACT_OUT_ROUTE_DISCRIMINATOR, ROUTE_DISCRIMINATOR,
    },
};
use crate::{
    error::ParseResult,
    events::core::EventParameters,
    events::{
        common::utils::{validate_account_count, validate_data_length},
        factory::SolanaTransactionInput,
        parser_types::{GenericEventParseConfig, ProtocolParser},
    },
    solana_metadata::SolanaEventMetadata,
    types::{metadata_helpers, EventType, ProtocolType},
};

use borsh::BorshDeserialize;
use riglr_events_core::{
    error::EventResult,
    traits::{EventParser, ParserInfo},
    Event,
};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// Jupiter event parser
#[derive(Debug)]
pub struct JupiterEventParser {
    program_ids: Vec<Pubkey>,
    inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>,
    instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
}

impl JupiterEventParser {
    /// Creates a new Jupiter event parser with default configurations for routing and exact-out routing
    pub fn new() -> Self {
        let program_ids = vec![jupiter_v6_program_id()];

        let configs = vec![
            GenericEventParseConfig {
                program_id: jupiter_v6_program_id(),
                protocol_type: ProtocolType::Other("Jupiter".to_string()),
                inner_instruction_discriminator: "swap",
                instruction_discriminator: &ROUTE_DISCRIMINATOR,
                event_type: EventType::Swap,
                inner_instruction_parser: parse_jupiter_swap_inner_instruction,
                instruction_parser: parse_jupiter_swap_instruction,
            },
            GenericEventParseConfig {
                program_id: jupiter_v6_program_id(),
                protocol_type: ProtocolType::Other("Jupiter".to_string()),
                inner_instruction_discriminator: "exactOutRoute",
                instruction_discriminator: &EXACT_OUT_ROUTE_DISCRIMINATOR,
                event_type: EventType::Swap,
                inner_instruction_parser: parse_jupiter_exact_out_inner_instruction,
                instruction_parser: parse_jupiter_exact_out_instruction,
            },
        ];

        let mut inner_instruction_configs = HashMap::default();
        let mut instruction_configs = HashMap::default();

        for config in configs {
            inner_instruction_configs
                .entry(config.inner_instruction_discriminator)
                .or_insert_with(Vec::default)
                .push(config.clone());
            instruction_configs
                .entry(config.instruction_discriminator.to_vec())
                .or_insert_with(Vec::default)
                .push(config);
        }

        Self {
            program_ids,
            inner_instruction_configs,
            instruction_configs,
        }
    }
}

// Implement the new core EventParser trait
#[async_trait::async_trait]
impl EventParser for JupiterEventParser {
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
                self.parse_events_from_inner_instruction_impl(&legacy_params)
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
                self.parse_events_from_instruction_impl(&legacy_params)
            }
        };
        Ok(events)
    }

    fn can_parse(&self, input: &Self::Input) -> bool {
        match input {
            SolanaTransactionInput::InnerInstruction(_) => true,
            SolanaTransactionInput::Instruction(params) => {
                // Check if the instruction matches our discriminators
                self.instruction_configs
                    .keys()
                    .any(|disc| params.instruction_data.starts_with(disc))
            }
        }
    }

    fn info(&self) -> ParserInfo {
        use riglr_events_core::EventKind;
        ParserInfo::new("jupiter_parser".to_string(), "1.0.0".to_string())
            .with_kind(EventKind::Swap)
            .with_format("solana_instruction".to_string())
    }
}

// Implement the legacy EventParser trait for backward compatibility
impl ProtocolParser for JupiterEventParser {
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner_instruction_configs.clone()
    }

    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.instruction_configs.clone()
    }

    fn parse_events_from_inner_instruction(
        &self,
        params: &crate::events::factory::InnerInstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        self.parse_events_from_inner_instruction_impl(params)
    }

    fn parse_events_from_instruction(
        &self,
        params: &crate::events::factory::InstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        self.parse_events_from_instruction_impl(params)
    }

    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.program_ids.contains(program_id)
    }

    fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.program_ids.clone()
    }
}

impl JupiterEventParser {
    /// Helper method to return inner instruction configs (for testing)
    pub fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner_instruction_configs.clone()
    }

    /// Helper method to return instruction configs (for testing)
    pub fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.instruction_configs.clone()
    }

    fn parse_events_from_inner_instruction_impl(
        &self,
        params: &crate::events::factory::InnerInstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        let mut events = Vec::default();

        // For inner instructions, we'll use the data to identify the instruction type
        // since the program field isn't available in all Solana SDK versions
        if let Ok(data) = bs58::decode(&params.inner_instruction.data).into_vec() {
            for configs in self.inner_instruction_configs.values() {
                for config in configs {
                    let metadata = metadata_helpers::create_solana_metadata(
                        format!("{}_{}", params.signature, params.index),
                        params.signature.to_string(),
                        params.slot,
                        params.block_time.unwrap_or(0),
                        config.protocol_type.clone(),
                        config.event_type.clone(),
                        config.program_id,
                        params.index.clone(),
                        params.program_received_time_ms,
                    );

                    if let Ok(event) = (config.inner_instruction_parser)(&data, metadata) {
                        events.push(event);
                    }
                }
            }
        }

        events
    }

    fn parse_events_from_instruction_impl(
        &self,
        params: &crate::events::factory::InstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        let mut events = Vec::default();

        if let Some(configs) = self.instruction_configs.get(&params.instruction.data) {
            for config in configs {
                let metadata = metadata_helpers::create_solana_metadata(
                    format!("{}_{}", params.signature, params.index),
                    params.signature.to_string(),
                    params.slot,
                    params.block_time.unwrap_or(0),
                    config.protocol_type.clone(),
                    config.event_type.clone(),
                    config.program_id,
                    params.index.clone(),
                    params.program_received_time_ms,
                );

                if let Ok(event) =
                    (config.instruction_parser)(&params.instruction.data, params.accounts, metadata)
                {
                    events.push(event);
                }
            }
        }

        events
    }
}

impl Default for JupiterEventParser {
    fn default() -> Self {
        let program_ids = vec![jupiter_v6_program_id()];

        let configs = vec![
            GenericEventParseConfig {
                program_id: jupiter_v6_program_id(),
                protocol_type: ProtocolType::Other("Jupiter".to_string()),
                inner_instruction_discriminator: "swap",
                instruction_discriminator: &ROUTE_DISCRIMINATOR,
                event_type: EventType::Swap,
                inner_instruction_parser: parse_jupiter_swap_inner_instruction,
                instruction_parser: parse_jupiter_swap_instruction,
            },
            GenericEventParseConfig {
                program_id: jupiter_v6_program_id(),
                protocol_type: ProtocolType::Other("Jupiter".to_string()),
                inner_instruction_discriminator: "exactOutRoute",
                instruction_discriminator: &EXACT_OUT_ROUTE_DISCRIMINATOR,
                event_type: EventType::Swap,
                inner_instruction_parser: parse_jupiter_exact_out_inner_instruction,
                instruction_parser: parse_jupiter_exact_out_instruction,
            },
        ];

        let mut inner_instruction_configs = HashMap::default();
        let mut instruction_configs = HashMap::default();

        for config in configs {
            inner_instruction_configs
                .entry(config.inner_instruction_discriminator)
                .or_insert_with(Vec::default)
                .push(config.clone());
            instruction_configs
                .entry(config.instruction_discriminator.to_vec())
                .or_insert_with(Vec::default)
                .push(config);
        }

        Self {
            program_ids,
            inner_instruction_configs,
            instruction_configs,
        }
    }
}

// Parser functions for different Jupiter instruction types

fn parse_jupiter_swap_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let swap_data = parse_jupiter_swap_with_borsh(data)?;

    Ok(Box::new(JupiterSwapEvent::new(
        EventParameters::new(
            metadata.id().to_string(),
            metadata.signature.clone(),
            metadata.slot,
            0, // block_time - not available in SolanaEventMetadata
            0, // block_time_ms - not available in SolanaEventMetadata
            metadata.program_received_time_ms,
            metadata.index.clone(),
        ),
        swap_data,
    )) as Box<dyn Event>)
}

fn parse_jupiter_swap_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    // Validate minimum account count for Jupiter swap instructions
    validate_account_count(accounts, 1, "Jupiter swap instruction")?;

    let swap_data = parse_jupiter_swap_with_borsh(data)?;

    Ok(Box::new(JupiterSwapEvent::new(
        EventParameters::new(
            metadata.id().to_string(),
            metadata.signature.clone(),
            metadata.slot,
            0, // block_time - not available in SolanaEventMetadata
            0, // block_time_ms - not available in SolanaEventMetadata
            metadata.program_received_time_ms,
            metadata.index.clone(),
        ),
        swap_data,
    )) as Box<dyn Event>)
}

fn parse_jupiter_exact_out_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let swap_data = parse_jupiter_exact_out_with_borsh(data)?;

    Ok(Box::new(JupiterSwapEvent::new(
        EventParameters::new(
            metadata.id().to_string(),
            metadata.signature.clone(),
            metadata.slot,
            0, // block_time - not available in SolanaEventMetadata
            0, // block_time_ms - not available in SolanaEventMetadata
            metadata.program_received_time_ms,
            metadata.index.clone(),
        ),
        swap_data,
    )) as Box<dyn Event>)
}

fn parse_jupiter_exact_out_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    // Validate minimum account count for Jupiter exact out instructions
    validate_account_count(accounts, 1, "Jupiter exact out instruction")?;

    let swap_data = parse_jupiter_exact_out_with_borsh(data)?;

    Ok(Box::new(JupiterSwapEvent::new(
        EventParameters::new(
            metadata.id().to_string(),
            metadata.signature.clone(),
            metadata.slot,
            0, // block_time - not available in SolanaEventMetadata
            0, // block_time_ms - not available in SolanaEventMetadata
            metadata.program_received_time_ms,
            metadata.index.clone(),
        ),
        swap_data,
    )) as Box<dyn Event>)
}

// Data parsing helpers

/// Parse Jupiter swap data using borsh deserialization
fn parse_jupiter_swap_with_borsh(data: &[u8]) -> ParseResult<JupiterSwapData> {
    // Validate minimum data length (discriminator + instruction data)
    validate_data_length(data, 8, "Jupiter swap discriminator")?;

    let instruction_data = &data[8..];

    // Try to deserialize as SharedAccountsRouteData
    let route_data = SharedAccountsRouteData::try_from_slice(instruction_data).map_err(|e| {
        crate::error::ParseError::InvalidDataFormat(format!(
            "Failed to deserialize SharedAccountsRouteData: {}",
            e
        ))
    })?;

    // Extract first and last swap info for simplified event data
    let first_step = route_data.route_plan.first().ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat(
            "Empty route plan in Jupiter swap data".to_string(),
        )
    })?;

    let last_step = route_data.route_plan.last().ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat(
            "Empty route plan in Jupiter swap data".to_string(),
        )
    })?;

    Ok(JupiterSwapData {
        user: Pubkey::default(), // Will be set from accounts
        input_mint: first_step.swap.source_token,
        output_mint: last_step.swap.destination_token,
        input_amount: route_data.in_amount,
        output_amount: route_data.quoted_out_amount,
        price_impact_pct: None,
        platform_fee_bps: Some(route_data.platform_fee_bps as u32),
        route_plan: route_data
            .route_plan
            .into_iter()
            .map(|step| RoutePlan {
                input_mint: step.swap.source_token,
                output_mint: step.swap.destination_token,
                amount_in: 0,  // Not available in this format
                amount_out: 0, // Not available in this format
                dex_label: format!("DEX {}", step.percent),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solana_metadata::SolanaEventMetadata;
    use crate::types::{EventType, ProtocolType};
    use solana_message::compiled_instruction::CompiledInstruction;
    use solana_transaction_status::UiCompiledInstruction;
    use std::str::FromStr;

    fn create_test_metadata() -> SolanaEventMetadata {
        metadata_helpers::create_solana_metadata(
            "test_id".to_string(),
            "test_signature".to_string(),
            12345,
            1640995200,
            ProtocolType::Other("Jupiter".to_string()),
            EventType::Swap,
            jupiter_v6_program_id(),
            "0".to_string(),
            1640995200000,
        )
    }

    fn create_test_pubkey() -> Pubkey {
        Pubkey::from_str("11111111111111111111111111111112").unwrap()
    }

    #[test]
    fn test_new_should_create_parser_with_correct_configs() {
        let parser = JupiterEventParser::default();

        assert_eq!(parser.program_ids.len(), 1);
        assert_eq!(parser.program_ids[0], jupiter_v6_program_id());

        assert_eq!(parser.inner_instruction_configs.len(), 2);
        assert!(parser.inner_instruction_configs.contains_key("swap"));
        assert!(parser
            .inner_instruction_configs
            .contains_key("exactOutRoute"));

        assert_eq!(parser.instruction_configs.len(), 2);
        assert!(parser
            .instruction_configs
            .contains_key(&ROUTE_DISCRIMINATOR.to_vec()));
        assert!(parser
            .instruction_configs
            .contains_key(&EXACT_OUT_ROUTE_DISCRIMINATOR.to_vec()));
    }

    #[test]
    fn test_default_should_create_same_as_new() {
        let parser_new = JupiterEventParser::default();
        let parser_default = JupiterEventParser::default();

        assert_eq!(parser_new.program_ids, parser_default.program_ids);
        assert_eq!(
            parser_new.inner_instruction_configs.len(),
            parser_default.inner_instruction_configs.len()
        );
        assert_eq!(
            parser_new.instruction_configs.len(),
            parser_default.instruction_configs.len()
        );
    }

    #[test]
    fn test_inner_instruction_configs_should_return_clone() {
        let parser = JupiterEventParser::default();
        let configs = parser.inner_instruction_configs();

        assert_eq!(configs.len(), 2);
        assert!(configs.contains_key("swap"));
        assert!(configs.contains_key("exactOutRoute"));
    }

    #[test]
    fn test_instruction_configs_should_return_clone() {
        let parser = JupiterEventParser::default();
        let configs = parser.instruction_configs();

        assert_eq!(configs.len(), 2);
        assert!(configs.contains_key(&ROUTE_DISCRIMINATOR.to_vec()));
        assert!(configs.contains_key(&EXACT_OUT_ROUTE_DISCRIMINATOR.to_vec()));
    }

    #[test]
    fn test_should_handle_when_program_id_matches_should_return_true() {
        let parser = JupiterEventParser::default();
        let result = parser.should_handle(&jupiter_v6_program_id());

        assert!(result);
    }

    #[test]
    fn test_should_handle_when_program_id_does_not_match_should_return_false() {
        let parser = JupiterEventParser::default();
        let other_program_id = create_test_pubkey();
        let result = parser.should_handle(&other_program_id);

        assert!(!result);
    }

    #[test]
    fn test_supported_program_ids_should_return_clone() {
        let parser = JupiterEventParser::default();
        let program_ids = parser.supported_program_ids();

        assert_eq!(program_ids.len(), 1);
        assert_eq!(program_ids[0], jupiter_v6_program_id());
    }

    #[test]
    fn test_parse_events_from_inner_instruction_when_invalid_data_should_return_empty() {
        let parser = JupiterEventParser::default();
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "invalid_data".to_string(),
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1640995200),
            program_received_time_ms: 1640995200000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_events_from_inner_instruction_when_valid_data_but_no_match_should_return_empty() {
        let parser = JupiterEventParser::default();
        let valid_data = bs58::encode(vec![1, 2, 3, 4]).into_string();
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: valid_data,
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1640995200),
            program_received_time_ms: 1640995200000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_events_from_inner_instruction_when_block_time_none_should_use_zero() {
        let parser = JupiterEventParser::default();
        let valid_data = bs58::encode(vec![1, 2, 3, 4]).into_string();
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: valid_data,
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: None, // block_time is None
            program_received_time_ms: 1640995200000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        // Should not panic and return empty events (since data doesn't match any parser)
        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_events_from_instruction_when_no_matching_config_should_return_empty() {
        let parser = JupiterEventParser::default();
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![1, 2, 3, 4], // Non-matching discriminator
        };
        let accounts = vec![create_test_pubkey()];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1640995200),
            program_received_time_ms: 1640995200000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_events_from_instruction_when_matching_config_but_parser_fails_should_return_empty(
    ) {
        let parser = JupiterEventParser::default();
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: ROUTE_DISCRIMINATOR.to_vec(), // Matching discriminator but invalid data
        };
        let accounts = vec![create_test_pubkey()];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1640995200),
            program_received_time_ms: 1640995200000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_events_from_instruction_when_block_time_none_should_use_zero() {
        let parser = JupiterEventParser::default();
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: ROUTE_DISCRIMINATOR.to_vec(),
        };
        let accounts = vec![create_test_pubkey()];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 12345,
            block_time: None, // block_time is None
            program_received_time_ms: 1640995200000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        // Should not panic and return empty events (since data doesn't match any parser)
        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_jupiter_swap_inner_instruction_when_valid_data_should_return_event() {
        let metadata = create_test_metadata();

        // Create minimal valid data for testing (discriminator + some data)
        let data = vec![0u8; 16]; // 8 bytes discriminator + 8 bytes data

        let result = parse_jupiter_swap_inner_instruction(&data, metadata);

        // Should return Err since data doesn't match expected format
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_swap_inner_instruction_when_invalid_data_should_return_err() {
        let metadata = create_test_metadata();
        let data = vec![1, 2, 3]; // Invalid data

        let result = parse_jupiter_swap_inner_instruction(&data, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_swap_instruction_when_valid_data_should_return_event() {
        let metadata = create_test_metadata();
        let accounts = vec![create_test_pubkey()];

        // Create minimal valid data for testing
        let data = vec![0u8; 16];

        let result = parse_jupiter_swap_instruction(&data, &accounts, metadata);

        // Should return Err since data doesn't match expected format
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_swap_instruction_when_invalid_data_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = vec![create_test_pubkey()];
        let data = vec![1, 2, 3]; // Invalid data

        let result = parse_jupiter_swap_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_exact_out_inner_instruction_when_valid_data_should_return_event() {
        let metadata = create_test_metadata();

        // Create minimal valid data for testing
        let data = vec![0u8; 16];

        let result = parse_jupiter_exact_out_inner_instruction(&data, metadata);

        // Should return Err since data doesn't match expected format
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_exact_out_inner_instruction_when_invalid_data_should_return_err() {
        let metadata = create_test_metadata();
        let data = vec![1, 2, 3]; // Invalid data

        let result = parse_jupiter_exact_out_inner_instruction(&data, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_exact_out_instruction_when_valid_data_should_return_event() {
        let metadata = create_test_metadata();
        let accounts = vec![create_test_pubkey()];

        // Create minimal valid data for testing
        let data = vec![0u8; 16];

        let result = parse_jupiter_exact_out_instruction(&data, &accounts, metadata);

        // Should return Err since data doesn't match expected format
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_exact_out_instruction_when_invalid_data_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = vec![create_test_pubkey()];
        let data = vec![1, 2, 3]; // Invalid data

        let result = parse_jupiter_exact_out_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_swap_with_borsh_when_data_too_short_should_return_err() {
        let data = vec![1, 2, 3]; // Less than 8 bytes

        let result = parse_jupiter_swap_with_borsh(&data);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_swap_with_borsh_when_data_exactly_8_bytes_should_return_err() {
        let data = vec![0u8; 8]; // Exactly 8 bytes (only discriminator)

        let result = parse_jupiter_swap_with_borsh(&data);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_swap_with_borsh_when_invalid_borsh_data_should_return_err() {
        let mut data = vec![0u8; 8]; // Discriminator
        data.extend_from_slice(&[1, 2, 3, 4]); // Invalid borsh data

        let result = parse_jupiter_swap_with_borsh(&data);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_exact_out_with_borsh_when_data_too_short_should_return_err() {
        let data = vec![1, 2, 3]; // Less than 8 bytes

        let result = parse_jupiter_exact_out_with_borsh(&data);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_exact_out_with_borsh_when_data_exactly_8_bytes_should_return_err() {
        let data = vec![0u8; 8]; // Exactly 8 bytes (only discriminator)

        let result = parse_jupiter_exact_out_with_borsh(&data);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_exact_out_with_borsh_when_invalid_borsh_data_should_return_err() {
        let mut data = vec![0u8; 8]; // Discriminator
        data.extend_from_slice(&[1, 2, 3, 4]); // Invalid borsh data

        let result = parse_jupiter_exact_out_with_borsh(&data);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_events_from_inner_instruction_multiple_configs_should_process_all() {
        let parser = JupiterEventParser::default();

        // Verify that both swap and exactOutRoute configs are present
        assert!(parser.inner_instruction_configs.contains_key("swap"));
        assert!(parser
            .inner_instruction_configs
            .contains_key("exactOutRoute"));

        let valid_data = bs58::encode(vec![1, 2, 3, 4]).into_string();
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: valid_data,
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1640995200),
            program_received_time_ms: 1640995200000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        // Both configs should be processed, but return empty since data doesn't match
        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_events_from_instruction_multiple_configs_should_process_all() {
        let parser = JupiterEventParser::default();
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: ROUTE_DISCRIMINATOR.to_vec(),
        };
        let accounts = vec![create_test_pubkey()];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1640995200),
            program_received_time_ms: 1640995200000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        // Should process all matching configs but return empty since parser fails
        assert!(events.is_empty());
    }

    // ========== NEW COMPREHENSIVE TESTS FOR TASK 4 ==========

    #[test]
    fn test_parse_jupiter_swap_inner_instruction_when_data_too_short_should_return_err() {
        let metadata = create_test_metadata();
        let data = vec![1, 2, 3]; // Less than 8 bytes (discriminator size)

        let result = parse_jupiter_swap_inner_instruction(&data, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Failed to parse Jupiter swap data"));
        }
    }

    #[test]
    fn test_parse_jupiter_swap_inner_instruction_when_empty_data_should_return_err() {
        let metadata = create_test_metadata();
        let data = vec![];

        let result = parse_jupiter_swap_inner_instruction(&data, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Failed to parse Jupiter swap data"));
        }
    }

    #[test]
    fn test_parse_jupiter_swap_instruction_when_data_too_short_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = vec![create_test_pubkey()];
        let data = vec![1, 2]; // Very short data

        let result = parse_jupiter_swap_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e
                .to_string()
                .contains("Failed to parse Jupiter swap instruction data"));
        }
    }

    #[test]
    fn test_parse_jupiter_swap_instruction_when_empty_accounts_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = vec![];
        let data = vec![0u8; 16];

        let result = parse_jupiter_swap_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_exact_out_inner_instruction_when_data_too_short_should_return_err() {
        let metadata = create_test_metadata();
        let data = vec![1, 2, 3, 4]; // Less than 8 bytes

        let result = parse_jupiter_exact_out_inner_instruction(&data, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e
                .to_string()
                .contains("Failed to parse Jupiter exact out inner instruction data"));
        }
    }

    #[test]
    fn test_parse_jupiter_exact_out_instruction_when_data_too_short_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = vec![create_test_pubkey()];
        let data = vec![]; // Empty data

        let result = parse_jupiter_exact_out_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e
                .to_string()
                .contains("Failed to parse Jupiter exact out instruction data"));
        }
    }

    #[test]
    fn test_parse_jupiter_exact_out_instruction_when_empty_accounts_should_return_err() {
        let metadata = create_test_metadata();
        let accounts = vec![];
        let data = vec![0u8; 16];

        let result = parse_jupiter_exact_out_instruction(&data, &accounts, metadata);

        assert!(result.is_err());
    }

    // Edge case tests with max/min values

    #[test]
    fn test_parse_jupiter_swap_with_borsh_when_data_exactly_discriminator_size_should_return_err() {
        let data = vec![0u8; 8]; // Exactly discriminator size, no instruction data

        let result = parse_jupiter_swap_with_borsh(&data);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_exact_out_with_borsh_when_data_exactly_discriminator_size_should_return_err(
    ) {
        let data = vec![0u8; 8]; // Exactly discriminator size, no instruction data

        let result = parse_jupiter_exact_out_with_borsh(&data);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_swap_with_borsh_when_malformed_borsh_data_should_return_err() {
        let mut data = vec![0u8; 8]; // Discriminator
        data.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // Invalid borsh data

        let result = parse_jupiter_swap_with_borsh(&data);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jupiter_exact_out_with_borsh_when_malformed_borsh_data_should_return_err() {
        let mut data = vec![0u8; 8]; // Discriminator
        data.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // Invalid borsh data

        let result = parse_jupiter_exact_out_with_borsh(&data);

        assert!(result.is_err());
    }

    // Tests for extremely large values (edge cases)

    #[test]
    fn test_parse_events_from_inner_instruction_when_extremely_large_slot_should_not_panic() {
        let parser = JupiterEventParser::default();
        let valid_data = bs58::encode(vec![1, 2, 3, 4]).into_string();
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: valid_data,
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: u64::MAX,                     // Extremely large slot
            block_time: Some(i64::MAX),         // Extremely large block time
            program_received_time_ms: i64::MAX, // Extremely large program received time
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        // Should not panic and return empty events
        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_events_from_instruction_when_extremely_large_slot_should_not_panic() {
        let parser = JupiterEventParser::default();
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![1, 2, 3, 4], // Non-matching discriminator
        };
        let accounts = vec![create_test_pubkey()];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: u64::MAX,                     // Extremely large slot
            block_time: Some(i64::MAX),         // Extremely large block time
            program_received_time_ms: i64::MAX, // Extremely large program received time
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        // Should not panic and return empty events
        assert!(events.is_empty());
    }

    // Test with negative block times (edge case)

    #[test]
    fn test_parse_events_from_inner_instruction_when_negative_block_time_should_work() {
        let parser = JupiterEventParser::default();
        let valid_data = bs58::encode(vec![1, 2, 3, 4]).into_string();
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: valid_data,
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(-1640995200), // Negative block time
            program_received_time_ms: 1640995200000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        // Should not panic and return empty events
        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_events_from_instruction_when_negative_block_time_should_work() {
        let parser = JupiterEventParser::default();
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![1, 2, 3, 4],
        };
        let accounts = vec![create_test_pubkey()];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(-1640995200), // Negative block time
            program_received_time_ms: 1640995200000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        // Should not panic and return empty events
        assert!(events.is_empty());
    }

    // Tests with extremely long signatures and indices

    #[test]
    fn test_parse_events_with_extremely_long_signature_should_not_panic() {
        let parser = JupiterEventParser::default();
        let long_signature = "a".repeat(1000); // Very long signature
        let valid_data = bs58::encode(vec![1, 2, 3, 4]).into_string();
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: valid_data,
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: &long_signature,
            slot: 12345,
            block_time: Some(1640995200),
            program_received_time_ms: 1640995200000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        // Should not panic
        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_events_with_extremely_long_index_should_not_panic() {
        let parser = JupiterEventParser::default();
        let long_index = "x".repeat(500); // Very long index
        let valid_data = bs58::encode(vec![1, 2, 3, 4]).into_string();
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: valid_data,
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1640995200),
            program_received_time_ms: 1640995200000,
            index: long_index,
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        // Should not panic
        assert!(events.is_empty());
    }

    // Test with empty/zero values

    #[test]
    fn test_parse_events_from_inner_instruction_when_zero_slot_should_work() {
        let parser = JupiterEventParser::default();
        let valid_data = bs58::encode(vec![1, 2, 3, 4]).into_string();
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: valid_data,
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 0,                     // Zero slot
            block_time: Some(0),         // Zero block time
            program_received_time_ms: 0, // Zero program received time
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        // Should work without issues
        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_events_from_instruction_when_zero_slot_should_work() {
        let parser = JupiterEventParser::default();
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![1, 2, 3, 4],
        };
        let accounts = vec![create_test_pubkey()];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 0,                     // Zero slot
            block_time: Some(0),         // Zero block time
            program_received_time_ms: 0, // Zero program received time
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        // Should work without issues
        assert!(events.is_empty());
    }

    // Test with invalid base58 data

    #[test]
    fn test_parse_events_from_inner_instruction_when_invalid_base58_should_return_empty() {
        let parser = JupiterEventParser::default();
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "invalid_base58_0OIl".to_string(), // Invalid base58
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1640995200),
            program_received_time_ms: 1640995200000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        // Should handle gracefully and return empty
        assert!(events.is_empty());
    }

    // Test parser configuration edge cases

    #[test]
    fn test_should_handle_when_default_pubkey_should_return_false() {
        let parser = JupiterEventParser::default();
        let default_pubkey = Pubkey::default();

        let result = parser.should_handle(&default_pubkey);

        assert!(!result);
    }

    #[test]
    fn test_inner_instruction_configs_should_not_be_empty() {
        let parser = JupiterEventParser::default();
        let configs = parser.inner_instruction_configs();

        assert!(!configs.is_empty());
        assert!(configs.len() >= 2); // Should have at least swap and exactOutRoute
    }

    #[test]
    fn test_instruction_configs_should_not_be_empty() {
        let parser = JupiterEventParser::default();
        let configs = parser.instruction_configs();

        assert!(!configs.is_empty());
        assert!(configs.len() >= 2); // Should have at least 2 discriminators
    }

    #[test]
    fn test_supported_program_ids_should_contain_jupiter_v6() {
        let parser = JupiterEventParser::default();
        let program_ids = parser.supported_program_ids();

        assert!(!program_ids.is_empty());
        assert!(program_ids.contains(&jupiter_v6_program_id()));
    }
}

/// Parse Jupiter exact out swap data using borsh deserialization
fn parse_jupiter_exact_out_with_borsh(data: &[u8]) -> ParseResult<JupiterSwapData> {
    // Validate minimum data length (discriminator + instruction data)
    validate_data_length(data, 8, "Jupiter exact out swap discriminator")?;

    let instruction_data = &data[8..];

    // Try to deserialize as SharedAccountsExactOutRouteData
    let route_data =
        SharedAccountsExactOutRouteData::try_from_slice(instruction_data).map_err(|e| {
            crate::error::ParseError::InvalidDataFormat(format!(
                "Failed to deserialize SharedAccountsExactOutRouteData: {}",
                e
            ))
        })?;

    // Extract first and last swap info for simplified event data
    let first_step = route_data.route_plan.first().ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat(
            "Empty route plan in Jupiter exact out swap data".to_string(),
        )
    })?;

    let last_step = route_data.route_plan.last().ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat(
            "Empty route plan in Jupiter exact out swap data".to_string(),
        )
    })?;

    Ok(JupiterSwapData {
        user: Pubkey::default(), // Will be set from accounts
        input_mint: first_step.swap.source_token,
        output_mint: last_step.swap.destination_token,
        input_amount: route_data.quoted_in_amount,
        output_amount: route_data.out_amount,
        price_impact_pct: None,
        platform_fee_bps: Some(route_data.platform_fee_bps as u32),
        route_plan: route_data
            .route_plan
            .into_iter()
            .map(|step| RoutePlan {
                input_mint: step.swap.source_token,
                output_mint: step.swap.destination_token,
                amount_in: 0,  // Not available in this format
                amount_out: 0, // Not available in this format
                dex_label: format!("DEX {}", step.percent),
            })
            .collect(),
    })
}
