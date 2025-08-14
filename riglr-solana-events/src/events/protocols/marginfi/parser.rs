use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;
use crate::{
    events::core::{EventParser, GenericEventParseConfig, UnifiedEvent},
    types::{EventMetadata, EventType, ProtocolType},
    events::common::utils::{has_discriminator, parse_u64_le},
};
use super::{
    events::{
        MarginFiDepositEvent, MarginFiWithdrawEvent, MarginFiBorrowEvent,
        MarginFiRepayEvent, MarginFiLiquidationEvent
    },
    types::{
        marginfi_program_id, marginfi_bank_program_id, MarginFiDepositData, MarginFiWithdrawData,
        MarginFiBorrowData, MarginFiRepayData, MarginFiLiquidationData,
        MARGINFI_DEPOSIT_DISCRIMINATOR, MARGINFI_WITHDRAW_DISCRIMINATOR,
        MARGINFI_BORROW_DISCRIMINATOR, MARGINFI_REPAY_DISCRIMINATOR,
        MARGINFI_LIQUIDATE_DISCRIMINATOR,
    },
};

/// MarginFi event parser
pub struct MarginFiEventParser {
    program_ids: Vec<Pubkey>,
    inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>,
    instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
}

impl MarginFiEventParser {
    pub fn new() -> Self {
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

#[async_trait::async_trait]
impl EventParser for MarginFiEventParser {
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner_instruction_configs.clone()
    }

    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.instruction_configs.clone()
    }

    fn parse_events_from_inner_instruction(
        &self,
        inner_instruction: &solana_transaction_status::UiCompiledInstruction,
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        let mut events = Vec::new();
        
        // For inner instructions, we'll use the data to identify the instruction type
        if let Ok(data) = bs58::decode(&inner_instruction.data).into_vec() {
            for (_, configs) in &self.inner_instruction_configs {
                for config in configs {
                    let metadata = EventMetadata::new(
                        format!("{}_{}", signature, index),
                        signature.to_string(),
                        slot,
                        block_time.unwrap_or(0),
                        block_time.unwrap_or(0) * 1000,
                        config.protocol_type.clone(),
                        config.event_type.clone(),
                        config.program_id,
                        index.clone(),
                        program_received_time_ms,
                    );
                    
                    if let Some(event) = (config.inner_instruction_parser)(&data, metadata) {
                        events.push(event);
                    }
                }
            }
        }
        
        events
    }

    fn parse_events_from_instruction(
        &self,
        instruction: &solana_sdk::instruction::CompiledInstruction,
        accounts: &[Pubkey],
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        let mut events = Vec::new();
        
        // Check each discriminator
        for (discriminator, configs) in &self.instruction_configs {
            if has_discriminator(&instruction.data, discriminator) {
                for config in configs {
                    let metadata = EventMetadata::new(
                        format!("{}_{}", signature, index),
                        signature.to_string(),
                        slot,
                        block_time.unwrap_or(0),
                        block_time.unwrap_or(0) * 1000,
                        config.protocol_type.clone(),
                        config.event_type.clone(),
                        config.program_id,
                        index.clone(),
                        program_received_time_ms,
                    );
                    
                    if let Some(event) = (config.instruction_parser)(&instruction.data, accounts, metadata) {
                        events.push(event);
                    }
                }
            }
        }
        
        events
    }

    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.program_ids.contains(program_id)
    }

    fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.program_ids.clone()
    }

}

impl Default for MarginFiEventParser {
    fn default() -> Self {
        Self::new()
    }
}

// Parser functions for different MarginFi instruction types

fn parse_marginfi_deposit_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_marginfi_deposit_data(data).map(|deposit_data| {
        Box::new(MarginFiDepositEvent::new(
            metadata.id,
            metadata.signature,
            metadata.slot,
            metadata.block_time,
            metadata.block_time_ms,
            metadata.program_received_time_ms,
            metadata.index,
            deposit_data,
        )) as Box<dyn UnifiedEvent>
    })
}

fn parse_marginfi_deposit_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_marginfi_deposit_data_from_instruction(data, accounts).map(|deposit_data| {
        Box::new(MarginFiDepositEvent::new(
            metadata.id,
            metadata.signature,
            metadata.slot,
            metadata.block_time,
            metadata.block_time_ms,
            metadata.program_received_time_ms,
            metadata.index,
            deposit_data,
        )) as Box<dyn UnifiedEvent>
    })
}

fn parse_marginfi_withdraw_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_marginfi_withdraw_data(data).map(|withdraw_data| {
        Box::new(MarginFiWithdrawEvent {
            id: metadata.id,
            signature: metadata.signature,
            slot: metadata.slot,
            block_time: metadata.block_time,
            block_time_ms: metadata.block_time_ms,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index,
            withdraw_data,
            transfer_data: Vec::new(),
        }) as Box<dyn UnifiedEvent>
    })
}

fn parse_marginfi_withdraw_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_marginfi_withdraw_data_from_instruction(data, accounts).map(|withdraw_data| {
        Box::new(MarginFiWithdrawEvent {
            id: metadata.id,
            signature: metadata.signature,
            slot: metadata.slot,
            block_time: metadata.block_time,
            block_time_ms: metadata.block_time_ms,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index,
            withdraw_data,
            transfer_data: Vec::new(),
        }) as Box<dyn UnifiedEvent>
    })
}

fn parse_marginfi_borrow_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_marginfi_borrow_data(data).map(|borrow_data| {
        Box::new(MarginFiBorrowEvent {
            id: metadata.id,
            signature: metadata.signature,
            slot: metadata.slot,
            block_time: metadata.block_time,
            block_time_ms: metadata.block_time_ms,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index,
            borrow_data,
            transfer_data: Vec::new(),
        }) as Box<dyn UnifiedEvent>
    })
}

fn parse_marginfi_borrow_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_marginfi_borrow_data_from_instruction(data, accounts).map(|borrow_data| {
        Box::new(MarginFiBorrowEvent {
            id: metadata.id,
            signature: metadata.signature,
            slot: metadata.slot,
            block_time: metadata.block_time,
            block_time_ms: metadata.block_time_ms,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index,
            borrow_data,
            transfer_data: Vec::new(),
        }) as Box<dyn UnifiedEvent>
    })
}

fn parse_marginfi_repay_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_marginfi_repay_data(data).map(|repay_data| {
        Box::new(MarginFiRepayEvent {
            id: metadata.id,
            signature: metadata.signature,
            slot: metadata.slot,
            block_time: metadata.block_time,
            block_time_ms: metadata.block_time_ms,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index,
            repay_data,
            transfer_data: Vec::new(),
        }) as Box<dyn UnifiedEvent>
    })
}

fn parse_marginfi_repay_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_marginfi_repay_data_from_instruction(data, accounts).map(|repay_data| {
        Box::new(MarginFiRepayEvent {
            id: metadata.id,
            signature: metadata.signature,
            slot: metadata.slot,
            block_time: metadata.block_time,
            block_time_ms: metadata.block_time_ms,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index,
            repay_data,
            transfer_data: Vec::new(),
        }) as Box<dyn UnifiedEvent>
    })
}

fn parse_marginfi_liquidate_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_marginfi_liquidate_data(data).map(|liquidation_data| {
        Box::new(MarginFiLiquidationEvent {
            id: metadata.id,
            signature: metadata.signature,
            slot: metadata.slot,
            block_time: metadata.block_time,
            block_time_ms: metadata.block_time_ms,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index,
            liquidation_data,
            transfer_data: Vec::new(),
        }) as Box<dyn UnifiedEvent>
    })
}

fn parse_marginfi_liquidate_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_marginfi_liquidate_data_from_instruction(data, accounts).map(|liquidation_data| {
        Box::new(MarginFiLiquidationEvent {
            id: metadata.id,
            signature: metadata.signature,
            slot: metadata.slot,
            block_time: metadata.block_time,
            block_time_ms: metadata.block_time_ms,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index,
            liquidation_data,
            transfer_data: Vec::new(),
        }) as Box<dyn UnifiedEvent>
    })
}

// Data parsing helpers

fn parse_marginfi_deposit_data(data: &[u8]) -> Option<MarginFiDepositData> {
    if data.len() < 16 {
        return None;
    }

    let offset = 8; // Skip discriminator
    let amount = parse_u64_le(&data[offset..offset + 8]).ok()?;
    
    Some(MarginFiDepositData {
        marginfi_group: Pubkey::default(), // Would need to extract from accounts
        marginfi_account: Pubkey::default(), // Would need to extract from accounts
        signer: Pubkey::default(), // Would need to extract from accounts
        bank: Pubkey::default(), // Would need to extract from accounts
        token_account: Pubkey::default(), // Would need to extract from accounts
        bank_liquidity_vault: Pubkey::default(), // Would need to extract from accounts
        token_program: Pubkey::default(), // Would need to extract from accounts
        amount,
    })
}

fn parse_marginfi_deposit_data_from_instruction(data: &[u8], accounts: &[Pubkey]) -> Option<MarginFiDepositData> {
    let mut deposit_data = parse_marginfi_deposit_data(data)?;
    
    // Extract accounts (typical MarginFi deposit instruction layout)
    if accounts.len() >= 8 {
        deposit_data.marginfi_group = accounts[1];
        deposit_data.marginfi_account = accounts[2];
        deposit_data.signer = accounts[0];
        deposit_data.bank = accounts[3];
        deposit_data.token_account = accounts[4];
        deposit_data.bank_liquidity_vault = accounts[5];
        deposit_data.token_program = accounts[6];
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

fn parse_marginfi_withdraw_data_from_instruction(data: &[u8], accounts: &[Pubkey]) -> Option<MarginFiWithdrawData> {
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

fn parse_marginfi_borrow_data_from_instruction(data: &[u8], accounts: &[Pubkey]) -> Option<MarginFiBorrowData> {
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

fn parse_marginfi_repay_data_from_instruction(data: &[u8], accounts: &[Pubkey]) -> Option<MarginFiRepayData> {
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

fn parse_marginfi_liquidate_data_from_instruction(data: &[u8], accounts: &[Pubkey]) -> Option<MarginFiLiquidationData> {
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

