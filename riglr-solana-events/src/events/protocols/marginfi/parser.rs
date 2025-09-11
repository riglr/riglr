use super::{
    events::{
        MarginFiBorrowEvent, MarginFiDepositEvent, MarginFiLiquidationEvent, MarginFiRepayEvent,
        MarginFiWithdrawEvent,
    },
    types::{
        marginfi_bank_program_id, marginfi_program_id, MarginFiBorrowData, MarginFiDepositData,
        MarginFiLiquidationData, MarginFiRepayData, MarginFiWithdrawData,
        MARGINFI_BORROW_DISCRIMINATOR, MARGINFI_DEPOSIT_DISCRIMINATOR,
        MARGINFI_LIQUIDATE_DISCRIMINATOR, MARGINFI_REPAY_DISCRIMINATOR,
        MARGINFI_WITHDRAW_DISCRIMINATOR,
    },
};
use crate::{
    error::ParseResult,
    events::core::EventParameters,
    events::{
        common::utils::{
            has_discriminator, parse_u64_le, safe_get_account, validate_account_count,
            validate_data_length,
        },
        factory::SolanaTransactionInput,
        parser_types::{GenericEventParseConfig, ProtocolParser},
    },
    solana_metadata::SolanaEventMetadata,
    types::{metadata_helpers, EventType, ProtocolType},
};

use riglr_events_core::{
    error::EventResult,
    traits::{EventParser, ParserInfo},
    Event,
};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// MarginFi event parser
#[derive(Debug)]
pub struct MarginFiEventParser {
    program_ids: Vec<Pubkey>,
    inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>,
    instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
}

impl MarginFiEventParser {
    /// Helper method to return inner instruction configs (for testing)
    pub fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner_instruction_configs.clone()
    }

    /// Helper method to return instruction configs (for testing)
    pub fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.instruction_configs.clone()
    }
}

// Implement the new core EventParser trait
#[async_trait::async_trait]
impl EventParser for MarginFiEventParser {
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
        ParserInfo::new("marginfi_parser".to_string(), "1.0.0".to_string())
            .with_kind(EventKind::Transaction)
            .with_format("solana_instruction".to_string())
    }
}

// Implement the legacy EventParser trait for backward compatibility
impl ProtocolParser for MarginFiEventParser {
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

impl MarginFiEventParser {
    fn parse_events_from_inner_instruction_impl(
        &self,
        params: &crate::events::factory::InnerInstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        let mut events = Vec::new();

        // For inner instructions, we'll use the data to identify the instruction type
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
        let mut events = Vec::new();

        // Check each discriminator
        for (discriminator, configs) in &self.instruction_configs {
            if has_discriminator(&params.instruction.data, discriminator) {
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

                    if let Ok(event) = (config.instruction_parser)(
                        &params.instruction.data,
                        params.accounts,
                        metadata,
                    ) {
                        events.push(event);
                    }
                }
            }
        }

        events
    }
}

impl Default for MarginFiEventParser {
    fn default() -> Self {
        let program_ids = vec![marginfi_program_id(), marginfi_bank_program_id()];

        let configs = vec![
            GenericEventParseConfig {
                program_id: marginfi_program_id(),
                protocol_type: ProtocolType::MarginFi,
                inner_instruction_discriminator: "lendingAccountDeposit",
                instruction_discriminator: &MARGINFI_DEPOSIT_DISCRIMINATOR,
                event_type: EventType::AddLiquidity,
                inner_instruction_parser: parse_marginfi_deposit_inner_instruction,
                instruction_parser: parse_marginfi_deposit_instruction,
            },
            GenericEventParseConfig {
                program_id: marginfi_program_id(),
                protocol_type: ProtocolType::MarginFi,
                inner_instruction_discriminator: "lendingAccountWithdraw",
                instruction_discriminator: &MARGINFI_WITHDRAW_DISCRIMINATOR,
                event_type: EventType::RemoveLiquidity,
                inner_instruction_parser: parse_marginfi_withdraw_inner_instruction,
                instruction_parser: parse_marginfi_withdraw_instruction,
            },
            GenericEventParseConfig {
                program_id: marginfi_program_id(),
                protocol_type: ProtocolType::MarginFi,
                inner_instruction_discriminator: "lendingAccountBorrow",
                instruction_discriminator: &MARGINFI_BORROW_DISCRIMINATOR,
                event_type: EventType::Borrow,
                inner_instruction_parser: parse_marginfi_borrow_inner_instruction,
                instruction_parser: parse_marginfi_borrow_instruction,
            },
            GenericEventParseConfig {
                program_id: marginfi_program_id(),
                protocol_type: ProtocolType::MarginFi,
                inner_instruction_discriminator: "lendingAccountRepay",
                instruction_discriminator: &MARGINFI_REPAY_DISCRIMINATOR,
                event_type: EventType::Repay,
                inner_instruction_parser: parse_marginfi_repay_inner_instruction,
                instruction_parser: parse_marginfi_repay_instruction,
            },
            GenericEventParseConfig {
                program_id: marginfi_program_id(),
                protocol_type: ProtocolType::MarginFi,
                inner_instruction_discriminator: "lendingAccountLiquidate",
                instruction_discriminator: &MARGINFI_LIQUIDATE_DISCRIMINATOR,
                event_type: EventType::Liquidate,
                inner_instruction_parser: parse_marginfi_liquidate_inner_instruction,
                instruction_parser: parse_marginfi_liquidate_instruction,
            },
        ];

        let mut inner_instruction_configs = HashMap::new();
        let mut instruction_configs = HashMap::new();

        for config in configs {
            inner_instruction_configs
                .entry(config.inner_instruction_discriminator)
                .or_insert_with(Vec::new)
                .push(config.clone());
            instruction_configs
                .entry(config.instruction_discriminator.to_vec())
                .or_insert_with(Vec::new)
                .push(config);
        }

        Self {
            program_ids,
            inner_instruction_configs,
            instruction_configs,
        }
    }
}

// Parser functions for different MarginFi instruction types

fn parse_marginfi_deposit_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let deposit_data = parse_marginfi_deposit_data(data).ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat(
            "Failed to parse MarginFi deposit data".to_string(),
        )
    })?;

    Ok(Box::new(MarginFiDepositEvent::new(
        EventParameters::new(
            metadata.id().to_string(),
            metadata.signature.clone(),
            metadata.slot,
            0, // block_time - not available in SolanaEventMetadata
            0, // block_time_ms - not available in SolanaEventMetadata
            metadata.program_received_time_ms,
            metadata.index.clone(),
        ),
        deposit_data,
    )) as Box<dyn Event>)
}

fn parse_marginfi_deposit_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let deposit_data =
        parse_marginfi_deposit_data_from_instruction(data, accounts).ok_or_else(|| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse MarginFi deposit instruction data".to_string(),
            )
        })?;

    Ok(Box::new(MarginFiDepositEvent::new(
        EventParameters::new(
            metadata.id().to_string(),
            metadata.signature.clone(),
            metadata.slot,
            0, // block_time - not available in SolanaEventMetadata
            0, // block_time_ms - not available in SolanaEventMetadata
            metadata.program_received_time_ms,
            metadata.index.clone(),
        ),
        deposit_data,
    )) as Box<dyn Event>)
}

fn parse_marginfi_withdraw_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let withdraw_data = parse_marginfi_withdraw_data(data).ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat(
            "Failed to parse MarginFi withdraw data".to_string(),
        )
    })?;

    let params = EventParameters::new(
        metadata.id().to_string(),
        metadata.signature.clone(),
        metadata.slot,
        0, // block_time - not available in SolanaEventMetadata
        0, // block_time_ms - not available in SolanaEventMetadata
        metadata.program_received_time_ms,
        metadata.index.clone(),
    );

    Ok(Box::new(MarginFiWithdrawEvent::new(params, withdraw_data)) as Box<dyn Event>)
}

fn parse_marginfi_withdraw_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let withdraw_data =
        parse_marginfi_withdraw_data_from_instruction(data, accounts).ok_or_else(|| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse MarginFi withdraw instruction data".to_string(),
            )
        })?;

    let params = EventParameters::new(
        metadata.id().to_string(),
        metadata.signature.clone(),
        metadata.slot,
        0, // block_time - not available in SolanaEventMetadata
        0, // block_time_ms - not available in SolanaEventMetadata
        metadata.program_received_time_ms,
        metadata.index.clone(),
    );

    Ok(Box::new(MarginFiWithdrawEvent::new(params, withdraw_data)) as Box<dyn Event>)
}

fn parse_marginfi_borrow_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let borrow_data = parse_marginfi_borrow_data(data).ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat(
            "Failed to parse MarginFi borrow data".to_string(),
        )
    })?;

    let params = EventParameters::new(
        metadata.id().to_string(),
        metadata.signature.clone(),
        metadata.slot,
        0, // block_time - not available in SolanaEventMetadata
        0, // block_time_ms - not available in SolanaEventMetadata
        metadata.program_received_time_ms,
        metadata.index.clone(),
    );

    Ok(Box::new(MarginFiBorrowEvent::new(params, borrow_data)) as Box<dyn Event>)
}

fn parse_marginfi_borrow_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let borrow_data =
        parse_marginfi_borrow_data_from_instruction(data, accounts).ok_or_else(|| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse MarginFi borrow instruction data".to_string(),
            )
        })?;

    let params = EventParameters::new(
        metadata.id().to_string(),
        metadata.signature.clone(),
        metadata.slot,
        0, // block_time - not available in SolanaEventMetadata
        0, // block_time_ms - not available in SolanaEventMetadata
        metadata.program_received_time_ms,
        metadata.index.clone(),
    );

    Ok(Box::new(MarginFiBorrowEvent::new(params, borrow_data)) as Box<dyn Event>)
}

fn parse_marginfi_repay_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let repay_data = parse_marginfi_repay_data(data).ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat(
            "Failed to parse MarginFi repay data".to_string(),
        )
    })?;

    let params = EventParameters::new(
        metadata.id().to_string(),
        metadata.signature.clone(),
        metadata.slot,
        0, // block_time - not available in SolanaEventMetadata
        0, // block_time_ms - not available in SolanaEventMetadata
        metadata.program_received_time_ms,
        metadata.index.clone(),
    );

    Ok(Box::new(MarginFiRepayEvent::new(params, repay_data)) as Box<dyn Event>)
}

fn parse_marginfi_repay_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let repay_data =
        parse_marginfi_repay_data_from_instruction(data, accounts).ok_or_else(|| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse MarginFi repay instruction data".to_string(),
            )
        })?;

    let params = EventParameters::new(
        metadata.id().to_string(),
        metadata.signature.clone(),
        metadata.slot,
        0, // block_time - not available in SolanaEventMetadata
        0, // block_time_ms - not available in SolanaEventMetadata
        metadata.program_received_time_ms,
        metadata.index.clone(),
    );

    Ok(Box::new(MarginFiRepayEvent::new(params, repay_data)) as Box<dyn Event>)
}

fn parse_marginfi_liquidate_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidation_data = parse_marginfi_liquidate_data(data).ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat(
            "Failed to parse MarginFi liquidation data".to_string(),
        )
    })?;

    let params = EventParameters::new(
        metadata.id().to_string(),
        metadata.signature.clone(),
        metadata.slot,
        0, // block_time - not available in SolanaEventMetadata
        0, // block_time_ms - not available in SolanaEventMetadata
        metadata.program_received_time_ms,
        metadata.index.clone(),
    );

    Ok(Box::new(MarginFiLiquidationEvent::new(params, liquidation_data)) as Box<dyn Event>)
}

fn parse_marginfi_liquidate_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidation_data = parse_marginfi_liquidate_data_from_instruction(data, accounts)
        .ok_or_else(|| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse MarginFi liquidation instruction data".to_string(),
            )
        })?;

    let params = EventParameters::new(
        metadata.id().to_string(),
        metadata.signature.clone(),
        metadata.slot,
        0, // block_time - not available in SolanaEventMetadata
        0, // block_time_ms - not available in SolanaEventMetadata
        metadata.program_received_time_ms,
        metadata.index.clone(),
    );

    Ok(Box::new(MarginFiLiquidationEvent::new(params, liquidation_data)) as Box<dyn Event>)
}

// Data parsing helpers

fn parse_marginfi_deposit_data(data: &[u8]) -> Option<MarginFiDepositData> {
    validate_data_length(data, 16, "MarginFi deposit data").ok()?;

    let offset = 8; // Skip discriminator
    let amount = parse_u64_le(&data[offset..offset + 8]).ok()?;

    Some(MarginFiDepositData {
        marginfi_group: Pubkey::default(), // Would need to extract from accounts
        marginfi_account: Pubkey::default(), // Would need to extract from accounts
        signer: Pubkey::default(),         // Would need to extract from accounts
        bank: Pubkey::default(),           // Would need to extract from accounts
        token_account: Pubkey::default(),  // Would need to extract from accounts
        bank_liquidity_vault: Pubkey::default(), // Would need to extract from accounts
        token_program: Pubkey::default(),  // Would need to extract from accounts
        amount,
    })
}

fn parse_marginfi_deposit_data_from_instruction(
    data: &[u8],
    accounts: &[Pubkey],
) -> Option<MarginFiDepositData> {
    let mut deposit_data = parse_marginfi_deposit_data(data)?;

    // Extract accounts (typical MarginFi deposit instruction layout)
    if validate_account_count(accounts, 8, "MarginFi deposit instruction").is_ok() {
        deposit_data.marginfi_group = safe_get_account(accounts, 1).unwrap_or_default();
        deposit_data.marginfi_account = safe_get_account(accounts, 2).unwrap_or_default();
        deposit_data.signer = safe_get_account(accounts, 0).unwrap_or_default();
        deposit_data.bank = safe_get_account(accounts, 3).unwrap_or_default();
        deposit_data.token_account = safe_get_account(accounts, 4).unwrap_or_default();
        deposit_data.bank_liquidity_vault = safe_get_account(accounts, 5).unwrap_or_default();
        deposit_data.token_program = safe_get_account(accounts, 6).unwrap_or_default();
    }

    Some(deposit_data)
}

fn parse_marginfi_withdraw_data(data: &[u8]) -> Option<MarginFiWithdrawData> {
    if data.len() < 17 {
        return None;
    }

    let mut offset = 8; // Skip discriminator
    let amount = parse_u64_le(&data[offset..offset + 8]).ok()?;
    offset += 8;
    let withdraw_all = data.get(offset)? != &0;

    Some(MarginFiWithdrawData {
        marginfi_group: Pubkey::default(),
        marginfi_account: Pubkey::default(),
        signer: Pubkey::default(),
        bank: Pubkey::default(),
        token_account: Pubkey::default(),
        bank_liquidity_vault: Pubkey::default(),
        bank_liquidity_vault_authority: Pubkey::default(),
        token_program: Pubkey::default(),
        amount,
        withdraw_all,
    })
}

fn parse_marginfi_withdraw_data_from_instruction(
    data: &[u8],
    accounts: &[Pubkey],
) -> Option<MarginFiWithdrawData> {
    let mut withdraw_data = parse_marginfi_withdraw_data(data)?;

    if accounts.len() >= 9 {
        withdraw_data.marginfi_group = accounts[1];
        withdraw_data.marginfi_account = accounts[2];
        withdraw_data.signer = accounts[0];
        withdraw_data.bank = accounts[3];
        withdraw_data.token_account = accounts[4];
        withdraw_data.bank_liquidity_vault = accounts[5];
        withdraw_data.bank_liquidity_vault_authority = accounts[6];
        withdraw_data.token_program = accounts[7];
    }

    Some(withdraw_data)
}

fn parse_marginfi_borrow_data(data: &[u8]) -> Option<MarginFiBorrowData> {
    if data.len() < 16 {
        return None;
    }

    let offset = 8; // Skip discriminator
    let amount = parse_u64_le(&data[offset..offset + 8]).ok()?;

    Some(MarginFiBorrowData {
        marginfi_group: Pubkey::default(),
        marginfi_account: Pubkey::default(),
        signer: Pubkey::default(),
        bank: Pubkey::default(),
        token_account: Pubkey::default(),
        bank_liquidity_vault: Pubkey::default(),
        bank_liquidity_vault_authority: Pubkey::default(),
        token_program: Pubkey::default(),
        amount,
    })
}

fn parse_marginfi_borrow_data_from_instruction(
    data: &[u8],
    accounts: &[Pubkey],
) -> Option<MarginFiBorrowData> {
    let mut borrow_data = parse_marginfi_borrow_data(data)?;

    if accounts.len() >= 9 {
        borrow_data.marginfi_group = accounts[1];
        borrow_data.marginfi_account = accounts[2];
        borrow_data.signer = accounts[0];
        borrow_data.bank = accounts[3];
        borrow_data.token_account = accounts[4];
        borrow_data.bank_liquidity_vault = accounts[5];
        borrow_data.bank_liquidity_vault_authority = accounts[6];
        borrow_data.token_program = accounts[7];
    }

    Some(borrow_data)
}

fn parse_marginfi_repay_data(data: &[u8]) -> Option<MarginFiRepayData> {
    if data.len() < 17 {
        return None;
    }

    let mut offset = 8; // Skip discriminator
    let amount = parse_u64_le(&data[offset..offset + 8]).ok()?;
    offset += 8;
    let repay_all = data.get(offset)? != &0;

    Some(MarginFiRepayData {
        marginfi_group: Pubkey::default(),
        marginfi_account: Pubkey::default(),
        signer: Pubkey::default(),
        bank: Pubkey::default(),
        token_account: Pubkey::default(),
        bank_liquidity_vault: Pubkey::default(),
        token_program: Pubkey::default(),
        amount,
        repay_all,
    })
}

fn parse_marginfi_repay_data_from_instruction(
    data: &[u8],
    accounts: &[Pubkey],
) -> Option<MarginFiRepayData> {
    let mut repay_data = parse_marginfi_repay_data(data)?;

    if accounts.len() >= 8 {
        repay_data.marginfi_group = accounts[1];
        repay_data.marginfi_account = accounts[2];
        repay_data.signer = accounts[0];
        repay_data.bank = accounts[3];
        repay_data.token_account = accounts[4];
        repay_data.bank_liquidity_vault = accounts[5];
        repay_data.token_program = accounts[6];
    }

    Some(repay_data)
}

fn parse_marginfi_liquidate_data(data: &[u8]) -> Option<MarginFiLiquidationData> {
    if data.len() < 24 {
        return None;
    }

    let mut offset = 8; // Skip discriminator
    let asset_amount = parse_u64_le(&data[offset..offset + 8]).ok()?;
    offset += 8;
    let liab_amount = parse_u64_le(&data[offset..offset + 8]).ok()?;

    Some(MarginFiLiquidationData {
        marginfi_group: Pubkey::default(),
        asset_bank: Pubkey::default(),
        liab_bank: Pubkey::default(),
        liquidatee_marginfi_account: Pubkey::default(),
        liquidator_marginfi_account: Pubkey::default(),
        liquidator: Pubkey::default(),
        asset_bank_liquidity_vault: Pubkey::default(),
        liab_bank_liquidity_vault: Pubkey::default(),
        liquidator_token_account: Pubkey::default(),
        token_program: Pubkey::default(),
        asset_amount,
        liab_amount,
    })
}

fn parse_marginfi_liquidate_data_from_instruction(
    data: &[u8],
    accounts: &[Pubkey],
) -> Option<MarginFiLiquidationData> {
    let mut liquidation_data = parse_marginfi_liquidate_data(data)?;

    if accounts.len() >= 12 {
        liquidation_data.marginfi_group = accounts[1];
        liquidation_data.asset_bank = accounts[2];
        liquidation_data.liab_bank = accounts[3];
        liquidation_data.liquidatee_marginfi_account = accounts[4];
        liquidation_data.liquidator_marginfi_account = accounts[5];
        liquidation_data.liquidator = accounts[0];
        liquidation_data.asset_bank_liquidity_vault = accounts[6];
        liquidation_data.liab_bank_liquidity_vault = accounts[7];
        liquidation_data.liquidator_token_account = accounts[8];
        liquidation_data.token_program = accounts[9];
    }

    Some(liquidation_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventType, ProtocolType};
    use solana_message::compiled_instruction::CompiledInstruction;
    use solana_transaction_status::UiCompiledInstruction;

    fn create_test_metadata() -> SolanaEventMetadata {
        use riglr_events_core::EventKind;

        let core = riglr_events_core::EventMetadata::new(
            "test_id".to_string(),
            EventKind::Transaction,
            "solana".to_string(),
        );

        SolanaEventMetadata::new(
            "test_signature".to_string(),
            12345,
            EventType::AddLiquidity,
            ProtocolType::MarginFi,
            "0".to_string(),
            1000,
            core,
        )
    }

    fn create_test_accounts() -> Vec<Pubkey> {
        vec![
            Pubkey::new_unique(), // 0 - signer/liquidator
            Pubkey::new_unique(), // 1 - marginfi_group
            Pubkey::new_unique(), // 2 - marginfi_account/asset_bank
            Pubkey::new_unique(), // 3 - bank/liab_bank
            Pubkey::new_unique(), // 4 - token_account/liquidatee_marginfi_account
            Pubkey::new_unique(), // 5 - bank_liquidity_vault/liquidator_marginfi_account
            Pubkey::new_unique(), // 6 - bank_liquidity_vault_authority/asset_bank_liquidity_vault
            Pubkey::new_unique(), // 7 - token_program/liab_bank_liquidity_vault
            Pubkey::new_unique(), // 8 - liquidator_token_account
            Pubkey::new_unique(), // 9 - token_program
            Pubkey::new_unique(), // 10
            Pubkey::new_unique(), // 11
        ]
    }

    // Tests for MarginFiEventParser default implementation
    #[test]
    fn test_marginfi_event_parser_default_creates_correct_program_ids() {
        let parser = MarginFiEventParser::default();
        let expected_ids = vec![marginfi_program_id(), marginfi_bank_program_id()];
        assert_eq!(parser.program_ids, expected_ids);
    }

    #[test]
    fn test_marginfi_event_parser_default_creates_inner_instruction_configs() {
        let parser = MarginFiEventParser::default();
        assert!(parser
            .inner_instruction_configs
            .contains_key("lendingAccountDeposit"));
        assert!(parser
            .inner_instruction_configs
            .contains_key("lendingAccountWithdraw"));
        assert!(parser
            .inner_instruction_configs
            .contains_key("lendingAccountBorrow"));
        assert!(parser
            .inner_instruction_configs
            .contains_key("lendingAccountRepay"));
        assert!(parser
            .inner_instruction_configs
            .contains_key("lendingAccountLiquidate"));
    }

    #[test]
    fn test_marginfi_event_parser_default_creates_instruction_configs() {
        let parser = MarginFiEventParser::default();
        assert_eq!(parser.instruction_configs.len(), 5);
        assert!(parser
            .instruction_configs
            .contains_key(&MARGINFI_DEPOSIT_DISCRIMINATOR.to_vec()));
        assert!(parser
            .instruction_configs
            .contains_key(&MARGINFI_WITHDRAW_DISCRIMINATOR.to_vec()));
        assert!(parser
            .instruction_configs
            .contains_key(&MARGINFI_BORROW_DISCRIMINATOR.to_vec()));
        assert!(parser
            .instruction_configs
            .contains_key(&MARGINFI_REPAY_DISCRIMINATOR.to_vec()));
        assert!(parser
            .instruction_configs
            .contains_key(&MARGINFI_LIQUIDATE_DISCRIMINATOR.to_vec()));
    }

    // Tests for EventParser trait implementation
    #[test]
    fn test_inner_instruction_configs_returns_cloned_configs() {
        let parser = MarginFiEventParser::default();
        let configs = parser.inner_instruction_configs();
        assert_eq!(configs.len(), parser.inner_instruction_configs.len());
    }

    #[test]
    fn test_instruction_configs_returns_cloned_configs() {
        let parser = MarginFiEventParser::default();
        let configs = parser.instruction_configs();
        assert_eq!(configs.len(), parser.instruction_configs.len());
    }

    #[test]
    fn test_should_handle_returns_true_for_supported_program_id() {
        let parser = MarginFiEventParser::default();
        assert!(parser.should_handle(&marginfi_program_id()));
        assert!(parser.should_handle(&marginfi_bank_program_id()));
    }

    #[test]
    fn test_should_handle_returns_false_for_unsupported_program_id() {
        let parser = MarginFiEventParser::default();
        let unsupported_id = Pubkey::new_unique();
        assert!(!parser.should_handle(&unsupported_id));
    }

    #[test]
    fn test_supported_program_ids_returns_cloned_ids() {
        let parser = MarginFiEventParser::default();
        let ids = parser.supported_program_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&marginfi_program_id()));
        assert!(ids.contains(&marginfi_bank_program_id()));
    }

    #[test]
    fn test_parse_events_from_inner_instruction_with_invalid_data() {
        let parser = MarginFiEventParser::default();
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "invalid_base58".to_string(),
            stack_height: Some(1),
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_sig",
            slot: 12345,
            block_time: Some(1000),
            program_received_time_ms: 2000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_events_from_inner_instruction_with_valid_data_no_matches() {
        let parser = MarginFiEventParser::default();
        let valid_data = bs58::encode(&[1, 2, 3, 4]).into_string();
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: valid_data,
            stack_height: Some(1),
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_sig",
            slot: 12345,
            block_time: Some(1000),
            program_received_time_ms: 2000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_events_from_instruction_with_no_matching_discriminator() {
        let parser = MarginFiEventParser::default();
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![99, 99, 99, 99], // Non-matching discriminator
        };
        let accounts = create_test_accounts();

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_sig",
            slot: 12345,
            block_time: Some(1000),
            program_received_time_ms: 2000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_events_from_instruction_with_matching_discriminator() {
        let parser = MarginFiEventParser::default();
        let mut data = MARGINFI_DEPOSIT_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2, 3, 4, 5, 6, 7],
            data,
        };
        let accounts = create_test_accounts();

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_sig",
            slot: 12345,
            block_time: Some(1000),
            program_received_time_ms: 2000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert_eq!(events.len(), 1);
    }

    // Tests for deposit data parsing
    #[test]
    fn test_parse_marginfi_deposit_data_with_insufficient_length() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7]; // Only 8 bytes, need 16
        let result = parse_marginfi_deposit_data(&data);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_marginfi_deposit_data_with_valid_data() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount
        let result = parse_marginfi_deposit_data(&data);
        assert!(result.is_some());
        let deposit_data = result.unwrap();
        assert_eq!(deposit_data.amount, 1000);
    }

    #[test]
    fn test_parse_marginfi_deposit_data_from_instruction_with_insufficient_accounts() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount
        let accounts = vec![Pubkey::new_unique()]; // Only 1 account, need 8
        let result = parse_marginfi_deposit_data_from_instruction(&data, &accounts);
        assert!(result.is_some());
        let deposit_data = result.unwrap();
        assert_eq!(deposit_data.amount, 1000);
        assert_eq!(deposit_data.marginfi_group, Pubkey::default());
    }

    #[test]
    fn test_parse_marginfi_deposit_data_from_instruction_with_sufficient_accounts() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount
        let accounts = create_test_accounts();
        let result = parse_marginfi_deposit_data_from_instruction(&data, &accounts);
        assert!(result.is_some());
        let deposit_data = result.unwrap();
        assert_eq!(deposit_data.amount, 1000);
        assert_eq!(deposit_data.signer, accounts[0]);
        assert_eq!(deposit_data.marginfi_group, accounts[1]);
        assert_eq!(deposit_data.marginfi_account, accounts[2]);
        assert_eq!(deposit_data.bank, accounts[3]);
        assert_eq!(deposit_data.token_account, accounts[4]);
        assert_eq!(deposit_data.bank_liquidity_vault, accounts[5]);
        assert_eq!(deposit_data.token_program, accounts[6]);
    }

    #[test]
    fn test_parse_marginfi_deposit_data_from_instruction_with_invalid_base_data() {
        let data = vec![0, 1, 2]; // Too short for base parsing
        let accounts = create_test_accounts();
        let result = parse_marginfi_deposit_data_from_instruction(&data, &accounts);
        assert!(result.is_none());
    }

    // Tests for withdraw data parsing
    #[test]
    fn test_parse_marginfi_withdraw_data_with_insufficient_length() {
        let data = vec![0; 15]; // Only 15 bytes, need 17
        let result = parse_marginfi_withdraw_data(&data);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_marginfi_withdraw_data_with_valid_data_withdraw_all_false() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&2000u64.to_le_bytes()); // amount
        data.push(0); // withdraw_all = false
        let result = parse_marginfi_withdraw_data(&data);
        assert!(result.is_some());
        let withdraw_data = result.unwrap();
        assert_eq!(withdraw_data.amount, 2000);
        assert!(!withdraw_data.withdraw_all);
    }

    #[test]
    fn test_parse_marginfi_withdraw_data_with_valid_data_withdraw_all_true() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&3000u64.to_le_bytes()); // amount
        data.push(1); // withdraw_all = true
        let result = parse_marginfi_withdraw_data(&data);
        assert!(result.is_some());
        let withdraw_data = result.unwrap();
        assert_eq!(withdraw_data.amount, 3000);
        assert!(withdraw_data.withdraw_all);
    }

    #[test]
    fn test_parse_marginfi_withdraw_data_from_instruction_with_insufficient_accounts() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&2000u64.to_le_bytes()); // amount
        data.push(0); // withdraw_all = false
        let accounts = vec![Pubkey::new_unique()]; // Only 1 account, need 9
        let result = parse_marginfi_withdraw_data_from_instruction(&data, &accounts);
        assert!(result.is_some());
        let withdraw_data = result.unwrap();
        assert_eq!(withdraw_data.amount, 2000);
        assert_eq!(withdraw_data.marginfi_group, Pubkey::default());
    }

    #[test]
    fn test_parse_marginfi_withdraw_data_from_instruction_with_sufficient_accounts() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&2000u64.to_le_bytes()); // amount
        data.push(0); // withdraw_all = false
        let accounts = create_test_accounts();
        let result = parse_marginfi_withdraw_data_from_instruction(&data, &accounts);
        assert!(result.is_some());
        let withdraw_data = result.unwrap();
        assert_eq!(withdraw_data.amount, 2000);
        assert_eq!(withdraw_data.signer, accounts[0]);
        assert_eq!(withdraw_data.marginfi_group, accounts[1]);
        assert_eq!(withdraw_data.marginfi_account, accounts[2]);
        assert_eq!(withdraw_data.bank, accounts[3]);
        assert_eq!(withdraw_data.token_account, accounts[4]);
        assert_eq!(withdraw_data.bank_liquidity_vault, accounts[5]);
        assert_eq!(withdraw_data.bank_liquidity_vault_authority, accounts[6]);
        assert_eq!(withdraw_data.token_program, accounts[7]);
    }

    #[test]
    fn test_parse_marginfi_withdraw_data_from_instruction_with_invalid_base_data() {
        let data = vec![0, 1, 2]; // Too short for base parsing
        let accounts = create_test_accounts();
        let result = parse_marginfi_withdraw_data_from_instruction(&data, &accounts);
        assert!(result.is_none());
    }

    // Tests for borrow data parsing
    #[test]
    fn test_parse_marginfi_borrow_data_with_insufficient_length() {
        let data = vec![0; 15]; // Only 15 bytes, need 16
        let result = parse_marginfi_borrow_data(&data);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_marginfi_borrow_data_with_valid_data() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&5000u64.to_le_bytes()); // amount
        let result = parse_marginfi_borrow_data(&data);
        assert!(result.is_some());
        let borrow_data = result.unwrap();
        assert_eq!(borrow_data.amount, 5000);
    }

    #[test]
    fn test_parse_marginfi_borrow_data_from_instruction_with_insufficient_accounts() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&5000u64.to_le_bytes()); // amount
        let accounts = vec![Pubkey::new_unique()]; // Only 1 account, need 9
        let result = parse_marginfi_borrow_data_from_instruction(&data, &accounts);
        assert!(result.is_some());
        let borrow_data = result.unwrap();
        assert_eq!(borrow_data.amount, 5000);
        assert_eq!(borrow_data.marginfi_group, Pubkey::default());
    }

    #[test]
    fn test_parse_marginfi_borrow_data_from_instruction_with_sufficient_accounts() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&5000u64.to_le_bytes()); // amount
        let accounts = create_test_accounts();
        let result = parse_marginfi_borrow_data_from_instruction(&data, &accounts);
        assert!(result.is_some());
        let borrow_data = result.unwrap();
        assert_eq!(borrow_data.amount, 5000);
        assert_eq!(borrow_data.signer, accounts[0]);
        assert_eq!(borrow_data.marginfi_group, accounts[1]);
        assert_eq!(borrow_data.marginfi_account, accounts[2]);
        assert_eq!(borrow_data.bank, accounts[3]);
        assert_eq!(borrow_data.token_account, accounts[4]);
        assert_eq!(borrow_data.bank_liquidity_vault, accounts[5]);
        assert_eq!(borrow_data.bank_liquidity_vault_authority, accounts[6]);
        assert_eq!(borrow_data.token_program, accounts[7]);
    }

    #[test]
    fn test_parse_marginfi_borrow_data_from_instruction_with_invalid_base_data() {
        let data = vec![0, 1, 2]; // Too short for base parsing
        let accounts = create_test_accounts();
        let result = parse_marginfi_borrow_data_from_instruction(&data, &accounts);
        assert!(result.is_none());
    }

    // Tests for repay data parsing
    #[test]
    fn test_parse_marginfi_repay_data_with_insufficient_length() {
        let data = vec![0; 16]; // Only 16 bytes, need 17
        let result = parse_marginfi_repay_data(&data);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_marginfi_repay_data_with_valid_data_repay_all_false() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&6000u64.to_le_bytes()); // amount
        data.push(0); // repay_all = false
        let result = parse_marginfi_repay_data(&data);
        assert!(result.is_some());
        let repay_data = result.unwrap();
        assert_eq!(repay_data.amount, 6000);
        assert!(!repay_data.repay_all);
    }

    #[test]
    fn test_parse_marginfi_repay_data_with_valid_data_repay_all_true() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&7000u64.to_le_bytes()); // amount
        data.push(1); // repay_all = true
        let result = parse_marginfi_repay_data(&data);
        assert!(result.is_some());
        let repay_data = result.unwrap();
        assert_eq!(repay_data.amount, 7000);
        assert!(repay_data.repay_all);
    }

    #[test]
    fn test_parse_marginfi_repay_data_from_instruction_with_insufficient_accounts() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&6000u64.to_le_bytes()); // amount
        data.push(0); // repay_all = false
        let accounts = vec![Pubkey::new_unique()]; // Only 1 account, need 8
        let result = parse_marginfi_repay_data_from_instruction(&data, &accounts);
        assert!(result.is_some());
        let repay_data = result.unwrap();
        assert_eq!(repay_data.amount, 6000);
        assert_eq!(repay_data.marginfi_group, Pubkey::default());
    }

    #[test]
    fn test_parse_marginfi_repay_data_from_instruction_with_sufficient_accounts() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&6000u64.to_le_bytes()); // amount
        data.push(0); // repay_all = false
        let accounts = create_test_accounts();
        let result = parse_marginfi_repay_data_from_instruction(&data, &accounts);
        assert!(result.is_some());
        let repay_data = result.unwrap();
        assert_eq!(repay_data.amount, 6000);
        assert_eq!(repay_data.signer, accounts[0]);
        assert_eq!(repay_data.marginfi_group, accounts[1]);
        assert_eq!(repay_data.marginfi_account, accounts[2]);
        assert_eq!(repay_data.bank, accounts[3]);
        assert_eq!(repay_data.token_account, accounts[4]);
        assert_eq!(repay_data.bank_liquidity_vault, accounts[5]);
        assert_eq!(repay_data.token_program, accounts[6]);
    }

    #[test]
    fn test_parse_marginfi_repay_data_from_instruction_with_invalid_base_data() {
        let data = vec![0, 1, 2]; // Too short for base parsing
        let accounts = create_test_accounts();
        let result = parse_marginfi_repay_data_from_instruction(&data, &accounts);
        assert!(result.is_none());
    }

    // Tests for liquidation data parsing
    #[test]
    fn test_parse_marginfi_liquidate_data_with_insufficient_length() {
        let data = vec![0; 23]; // Only 23 bytes, need 24
        let result = parse_marginfi_liquidate_data(&data);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_marginfi_liquidate_data_with_valid_data() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&8000u64.to_le_bytes()); // asset_amount
        data.extend_from_slice(&9000u64.to_le_bytes()); // liab_amount
        let result = parse_marginfi_liquidate_data(&data);
        assert!(result.is_some());
        let liquidation_data = result.unwrap();
        assert_eq!(liquidation_data.asset_amount, 8000);
        assert_eq!(liquidation_data.liab_amount, 9000);
    }

    #[test]
    fn test_parse_marginfi_liquidate_data_from_instruction_with_insufficient_accounts() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&8000u64.to_le_bytes()); // asset_amount
        data.extend_from_slice(&9000u64.to_le_bytes()); // liab_amount
        let accounts = vec![Pubkey::new_unique()]; // Only 1 account, need 12
        let result = parse_marginfi_liquidate_data_from_instruction(&data, &accounts);
        assert!(result.is_some());
        let liquidation_data = result.unwrap();
        assert_eq!(liquidation_data.asset_amount, 8000);
        assert_eq!(liquidation_data.liab_amount, 9000);
        assert_eq!(liquidation_data.marginfi_group, Pubkey::default());
    }

    #[test]
    fn test_parse_marginfi_liquidate_data_from_instruction_with_sufficient_accounts() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&8000u64.to_le_bytes()); // asset_amount
        data.extend_from_slice(&9000u64.to_le_bytes()); // liab_amount
        let accounts = create_test_accounts();
        let result = parse_marginfi_liquidate_data_from_instruction(&data, &accounts);
        assert!(result.is_some());
        let liquidation_data = result.unwrap();
        assert_eq!(liquidation_data.asset_amount, 8000);
        assert_eq!(liquidation_data.liab_amount, 9000);
        assert_eq!(liquidation_data.liquidator, accounts[0]);
        assert_eq!(liquidation_data.marginfi_group, accounts[1]);
        assert_eq!(liquidation_data.asset_bank, accounts[2]);
        assert_eq!(liquidation_data.liab_bank, accounts[3]);
        assert_eq!(liquidation_data.liquidatee_marginfi_account, accounts[4]);
        assert_eq!(liquidation_data.liquidator_marginfi_account, accounts[5]);
        assert_eq!(liquidation_data.asset_bank_liquidity_vault, accounts[6]);
        assert_eq!(liquidation_data.liab_bank_liquidity_vault, accounts[7]);
        assert_eq!(liquidation_data.liquidator_token_account, accounts[8]);
        assert_eq!(liquidation_data.token_program, accounts[9]);
    }

    #[test]
    fn test_parse_marginfi_liquidate_data_from_instruction_with_invalid_base_data() {
        let data = vec![0, 1, 2]; // Too short for base parsing
        let accounts = create_test_accounts();
        let result = parse_marginfi_liquidate_data_from_instruction(&data, &accounts);
        assert!(result.is_none());
    }

    // Tests for instruction parser functions
    #[test]
    fn test_parse_marginfi_deposit_inner_instruction_with_valid_data() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount
        let metadata = create_test_metadata();
        let result = parse_marginfi_deposit_inner_instruction(&data, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_marginfi_deposit_inner_instruction_with_invalid_data() {
        let data = vec![0, 1, 2]; // Too short
        let metadata = create_test_metadata();
        let result = parse_marginfi_deposit_inner_instruction(&data, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_marginfi_deposit_instruction_with_valid_data() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let result = parse_marginfi_deposit_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_marginfi_deposit_instruction_with_invalid_data() {
        let data = vec![0, 1, 2]; // Too short
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let result = parse_marginfi_deposit_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_marginfi_withdraw_inner_instruction_with_valid_data() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&2000u64.to_le_bytes()); // amount
        data.push(0); // withdraw_all = false
        let metadata = create_test_metadata();
        let result = parse_marginfi_withdraw_inner_instruction(&data, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_marginfi_withdraw_inner_instruction_with_invalid_data() {
        let data = vec![0, 1, 2]; // Too short
        let metadata = create_test_metadata();
        let result = parse_marginfi_withdraw_inner_instruction(&data, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_marginfi_withdraw_instruction_with_valid_data() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&2000u64.to_le_bytes()); // amount
        data.push(0); // withdraw_all = false
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let result = parse_marginfi_withdraw_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_marginfi_withdraw_instruction_with_invalid_data() {
        let data = vec![0, 1, 2]; // Too short
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let result = parse_marginfi_withdraw_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_marginfi_borrow_inner_instruction_with_valid_data() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&5000u64.to_le_bytes()); // amount
        let metadata = create_test_metadata();
        let result = parse_marginfi_borrow_inner_instruction(&data, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_marginfi_borrow_inner_instruction_with_invalid_data() {
        let data = vec![0, 1, 2]; // Too short
        let metadata = create_test_metadata();
        let result = parse_marginfi_borrow_inner_instruction(&data, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_marginfi_borrow_instruction_with_valid_data() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&5000u64.to_le_bytes()); // amount
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let result = parse_marginfi_borrow_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_marginfi_borrow_instruction_with_invalid_data() {
        let data = vec![0, 1, 2]; // Too short
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let result = parse_marginfi_borrow_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_marginfi_repay_inner_instruction_with_valid_data() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&6000u64.to_le_bytes()); // amount
        data.push(0); // repay_all = false
        let metadata = create_test_metadata();
        let result = parse_marginfi_repay_inner_instruction(&data, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_marginfi_repay_inner_instruction_with_invalid_data() {
        let data = vec![0, 1, 2]; // Too short
        let metadata = create_test_metadata();
        let result = parse_marginfi_repay_inner_instruction(&data, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_marginfi_repay_instruction_with_valid_data() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&6000u64.to_le_bytes()); // amount
        data.push(0); // repay_all = false
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let result = parse_marginfi_repay_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_marginfi_repay_instruction_with_invalid_data() {
        let data = vec![0, 1, 2]; // Too short
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let result = parse_marginfi_repay_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_marginfi_liquidate_inner_instruction_with_valid_data() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&8000u64.to_le_bytes()); // asset_amount
        data.extend_from_slice(&9000u64.to_le_bytes()); // liab_amount
        let metadata = create_test_metadata();
        let result = parse_marginfi_liquidate_inner_instruction(&data, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_marginfi_liquidate_inner_instruction_with_invalid_data() {
        let data = vec![0, 1, 2]; // Too short
        let metadata = create_test_metadata();
        let result = parse_marginfi_liquidate_inner_instruction(&data, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_marginfi_liquidate_instruction_with_valid_data() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&8000u64.to_le_bytes()); // asset_amount
        data.extend_from_slice(&9000u64.to_le_bytes()); // liab_amount
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let result = parse_marginfi_liquidate_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_marginfi_liquidate_instruction_with_invalid_data() {
        let data = vec![0, 1, 2]; // Too short
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let result = parse_marginfi_liquidate_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    // Edge case tests for data parsing with boundary conditions
    #[test]
    fn test_parse_marginfi_withdraw_data_with_exact_minimum_length() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&u64::MAX.to_le_bytes()); // max amount
        data.push(255); // any non-zero value for withdraw_all = true
        let result = parse_marginfi_withdraw_data(&data);
        assert!(result.is_some());
        let withdraw_data = result.unwrap();
        assert_eq!(withdraw_data.amount, u64::MAX);
        assert!(withdraw_data.withdraw_all);
    }

    #[test]
    fn test_parse_marginfi_repay_data_with_exact_minimum_length() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&u64::MAX.to_le_bytes()); // max amount
        data.push(255); // any non-zero value for repay_all = true
        let result = parse_marginfi_repay_data(&data);
        assert!(result.is_some());
        let repay_data = result.unwrap();
        assert_eq!(repay_data.amount, u64::MAX);
        assert!(repay_data.repay_all);
    }

    #[test]
    fn test_parse_marginfi_liquidate_data_with_exact_minimum_length() {
        let mut data = vec![0; 8]; // discriminator
        data.extend_from_slice(&u64::MAX.to_le_bytes()); // max asset_amount
        data.extend_from_slice(&0u64.to_le_bytes()); // min liab_amount
        let result = parse_marginfi_liquidate_data(&data);
        assert!(result.is_some());
        let liquidation_data = result.unwrap();
        assert_eq!(liquidation_data.asset_amount, u64::MAX);
        assert_eq!(liquidation_data.liab_amount, 0);
    }

    // Test parse_events_from_inner_instruction with None block_time
    #[test]
    fn test_parse_events_from_inner_instruction_with_none_block_time() {
        let parser = MarginFiEventParser::default();
        let valid_data = bs58::encode(&[1, 2, 3, 4]).into_string();
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: valid_data,
            stack_height: Some(1),
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_sig",
            slot: 12345,
            block_time: None, // None block_time
            program_received_time_ms: 2000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert!(events.is_empty());
    }

    // Test parse_events_from_instruction with None block_time
    #[test]
    fn test_parse_events_from_instruction_with_none_block_time() {
        let parser = MarginFiEventParser::default();
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![99, 99, 99, 99], // Non-matching discriminator
        };
        let accounts = create_test_accounts();

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_sig",
            slot: 12345,
            block_time: None, // None block_time
            program_received_time_ms: 2000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert!(events.is_empty());
    }
}
