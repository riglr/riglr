use anyhow::Result;
use solana_sdk::{
    instruction::CompiledInstruction, pubkey::Pubkey, transaction::VersionedTransaction,
};
use solana_transaction_status::{
    EncodedTransactionWithStatusMeta, UiCompiledInstruction, UiInnerInstructions, UiInstruction,
};
use std::fmt::Debug;
use std::{collections::HashMap, str::FromStr};

use crate::events::common::{
    parse_transfer_datas_from_next_instructions, SwapData, TransferData, EventMetadata, EventType, ProtocolType
};
use crate::events::common::utils::*;

/// Unified Event Interface - All protocol events must implement this trait
pub trait UnifiedEvent: Debug + Send + Sync {
    /// Get event ID
    fn id(&self) -> &str;

    /// Get event type
    fn event_type(&self) -> EventType;

    /// Get transaction signature
    fn signature(&self) -> &str;

    /// Get slot number
    fn slot(&self) -> u64;

    /// Get program received timestamp (milliseconds)
    fn program_received_time_ms(&self) -> i64;

    /// Processing time consumption (milliseconds)
    fn program_handle_time_consuming_ms(&self) -> i64;

    /// Set processing time consumption (milliseconds)
    fn set_program_handle_time_consuming_ms(&mut self, program_handle_time_consuming_ms: i64);

    /// Convert event to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;

    /// Convert event to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Clone the event
    fn clone_boxed(&self) -> Box<dyn UnifiedEvent>;

    /// Merge events (optional implementation)
    fn merge(&mut self, _other: Box<dyn UnifiedEvent>) {
        // Default implementation: no merging operation
    }

    /// Set transfer datas
    fn set_transfer_datas(
        &mut self,
        transfer_datas: Vec<TransferData>,
        swap_data: Option<SwapData>,
    );

    /// Get index
    fn index(&self) -> String;

    /// Get protocol type
    fn protocol_type(&self) -> ProtocolType;
}

/// Event Parser trait - defines the core methods for event parsing
#[async_trait::async_trait]
pub trait EventParser: Send + Sync {
    /// Get inner instruction parsing configurations
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>>;
    /// Get instruction parsing configurations
    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>>;
    /// Parse event data from inner instruction
    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_inner_instruction(
        &self,
        inner_instruction: &UiCompiledInstruction,
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>>;

    /// Parse event data from instruction
    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_instruction(
        &self,
        instruction: &CompiledInstruction,
        accounts: &[Pubkey],
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>>;

    /// Parse instruction events from VersionedTransaction
    #[allow(clippy::too_many_arguments)]
    async fn parse_instruction_events_from_versioned_transaction(
        &self,
        transaction: &VersionedTransaction,
        signature: &str,
        slot: Option<u64>,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        accounts: &[Pubkey],
        inner_instructions: &[UiInnerInstructions],
    ) -> Result<Vec<Box<dyn UnifiedEvent>>> {
        // Pre-allocate capacity to avoid dynamic expansion
        let mut instruction_events = Vec::with_capacity(16);
        // Get transaction instructions and accounts
        let compiled_instructions = transaction.message.instructions();
        let mut accounts: Vec<Pubkey> = accounts.to_vec();

        // Check if transaction contains programs
        let has_program = accounts.iter().any(|account| self.should_handle(account));
        if has_program {
            // Parse each instruction
            for (index, instruction) in compiled_instructions.iter().enumerate() {
                if let Some(program_id) = accounts.get(instruction.program_id_index as usize) {
                    if self.should_handle(program_id) {
                        let max_idx = instruction.accounts.iter().max().unwrap_or(&0);
                        // Fill accounts (using Pubkey::default())
                        if *max_idx as usize > accounts.len() {
                            for _i in accounts.len()..*max_idx as usize {
                                accounts.push(Pubkey::default());
                            }
                        }
                        if let Ok(mut events) = self
                            .parse_instruction(
                                instruction,
                                &accounts,
                                signature,
                                slot,
                                block_time,
                                program_received_time_ms,
                                format!("{index}"),
                            )
                            .await
                        {
                            if !events.is_empty() {
                                if let Some(inn) =
                                    inner_instructions.iter().find(|inner_instruction| {
                                        inner_instruction.index == index as u8
                                    })
                                {
                                    events.iter_mut().for_each(|event| {
                                        let (transfer_datas, swap_data) =
                                            parse_transfer_datas_from_next_instructions(
                                                event.clone_boxed(),
                                                inn,
                                                -1_i8,
                                                &accounts,
                                            );
                                        event.set_transfer_datas(transfer_datas, swap_data);
                                    });
                                }
                                instruction_events.extend(events);
                            }
                        }
                    }
                }
            }
        }
        Ok(instruction_events)
    }

    async fn parse_versioned_transaction(
        &self,
        versioned_tx: &VersionedTransaction,
        signature: &str,
        slot: Option<u64>,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        bot_wallet: Option<Pubkey>,
    ) -> Result<Vec<Box<dyn UnifiedEvent>>> {
        let accounts: Vec<Pubkey> = versioned_tx.message.static_account_keys().to_vec();
        let events = self
            .parse_instruction_events_from_versioned_transaction(
                versioned_tx,
                signature,
                slot,
                block_time,
                program_received_time_ms,
                &accounts,
                &[],
            )
            .await
            .unwrap_or_else(|_e| vec![]);
        Ok(self.process_events(events, bot_wallet))
    }

    async fn parse_transaction(
        &self,
        tx: EncodedTransactionWithStatusMeta,
        signature: &str,
        slot: Option<u64>,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        bot_wallet: Option<Pubkey>,
    ) -> Result<Vec<Box<dyn UnifiedEvent>>> {
        let transaction = tx.transaction;
        // Check transaction metadata
        let meta =
            tx.meta.as_ref().ok_or_else(|| anyhow::anyhow!("Missing transaction metadata"))?;

        let mut address_table_lookups: Vec<Pubkey> = vec![];
        let mut inner_instructions: Vec<UiInnerInstructions> = vec![];
        if meta.err.is_none() {
            // Correctly handle OptionSerializer type
            if let solana_transaction_status::option_serializer::OptionSerializer::Some(
                meta_inner_instructions,
            ) = &meta.inner_instructions
            {
                inner_instructions = meta_inner_instructions.clone();
            }
            if let solana_transaction_status::option_serializer::OptionSerializer::Some(
                loaded_addresses,
            ) = &meta.loaded_addresses
            {
                for lookup in &loaded_addresses.writable {
                    if let Ok(pubkey) = Pubkey::from_str(lookup) {
                        address_table_lookups.push(pubkey);
                    }
                }
                for lookup in &loaded_addresses.readonly {
                    if let Ok(pubkey) = Pubkey::from_str(lookup) {
                        address_table_lookups.push(pubkey);
                    }
                }
            }
        }
        let mut accounts: Vec<Pubkey> = vec![];

        // Pre-allocate capacity to avoid dynamic expansion
        let mut instruction_events = Vec::with_capacity(16);

        // Parse instruction events
        if let Some(versioned_tx) = transaction.decode() {
            accounts = versioned_tx.message.static_account_keys().to_vec();
            accounts.extend(address_table_lookups.clone());

            instruction_events = self
                .parse_instruction_events_from_versioned_transaction(
                    &versioned_tx,
                    signature,
                    slot,
                    block_time,
                    program_received_time_ms,
                    &accounts,
                    &inner_instructions,
                )
                .await
                .unwrap_or_else(|_e| vec![]);
        } else {
            accounts.extend(address_table_lookups.clone());
        }

        // Parse inner instruction events
        // Pre-allocate capacity to avoid dynamic expansion
        let mut inner_instruction_events = Vec::with_capacity(8);
        // Check if transaction succeeded
        if meta.err.is_none() {
            for inner_instruction in inner_instructions {
                for (index, instruction) in inner_instruction.instructions.iter().enumerate() {
                    if let UiInstruction::Compiled(compiled) = instruction {
                        // Parse nested instructions
                        let compiled_instruction = CompiledInstruction {
                            program_id_index: compiled.program_id_index,
                            accounts: compiled.accounts.clone(),
                            data: bs58::decode(compiled.data.clone())
                                .into_vec()
                                .unwrap_or_else(|_| vec![]),
                        };
                        if let Ok(mut events) = self
                            .parse_instruction(
                                &compiled_instruction,
                                &accounts,
                                signature,
                                slot,
                                block_time,
                                program_received_time_ms,
                                format!("{}.{}", inner_instruction.index, index),
                            )
                            .await
                        {
                            if !events.is_empty() {
                                events.iter_mut().for_each(|event| {
                                    let (transfer_datas, swap_data) =
                                        parse_transfer_datas_from_next_instructions(
                                            event.clone_boxed(),
                                            &inner_instruction,
                                            index as i8,
                                            &accounts,
                                        );
                                    event.set_transfer_datas(transfer_datas, swap_data);
                                });
                                instruction_events.extend(events);
                            }
                        }
                        if let Ok(mut events) = self
                            .parse_inner_instruction(
                                compiled,
                                signature,
                                slot,
                                block_time,
                                program_received_time_ms,
                                format!("{}.{}", inner_instruction.index, index),
                            )
                            .await
                        {
                            if !events.is_empty() {
                                events.iter_mut().for_each(|event| {
                                    let (transfer_datas, swap_data) =
                                        parse_transfer_datas_from_next_instructions(
                                            event.clone_boxed(),
                                            &inner_instruction,
                                            index as i8,
                                            &accounts,
                                        );
                                    event.set_transfer_datas(transfer_datas, swap_data);
                                });
                                inner_instruction_events.extend(events);
                            }
                        }
                    }
                }
            }
        }

        // Merge instruction and inner instruction events
        if !instruction_events.is_empty() && !inner_instruction_events.is_empty() {
            for instruction_event in &mut instruction_events {
                for inner_instruction_event in &inner_instruction_events {
                    if instruction_event.id() == inner_instruction_event.id() {
                        let i_index = instruction_event.index();
                        let in_index = inner_instruction_event.index();
                        if !i_index.contains(".") && in_index.contains(".") {
                            let in_index_parts: Vec<&str> = in_index.split(".").collect();
                            if !in_index_parts.is_empty() && in_index_parts[0] == i_index {
                                instruction_event.merge(inner_instruction_event.clone_boxed());
                                break;
                            }
                        } else if i_index.contains(".") && in_index.contains(".") {
                            // Nested instructions
                            let i_index_parts: Vec<&str> = i_index.split(".").collect();
                            let in_index_parts: Vec<&str> = in_index.split(".").collect();

                            if !i_index_parts.is_empty()
                                && !in_index_parts.is_empty()
                                && i_index_parts[0] == in_index_parts[0]
                            {
                                let i_index_child_index = i_index_parts
                                    .get(1)
                                    .and_then(|s| s.parse::<u32>().ok())
                                    .unwrap_or(0);
                                let in_index_child_index = in_index_parts
                                    .get(1)
                                    .and_then(|s| s.parse::<u32>().ok())
                                    .unwrap_or(0);
                                if in_index_child_index > i_index_child_index {
                                    instruction_event.merge(inner_instruction_event.clone_boxed());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        let result = self.process_events(instruction_events, bot_wallet);
        Ok(result)
    }

    fn process_events(
        &self,
        mut events: Vec<Box<dyn UnifiedEvent>>,
        _bot_wallet: Option<Pubkey>,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        for event in &mut events {
            let now = chrono::Utc::now().timestamp_millis();
            event.set_program_handle_time_consuming_ms(now - event.program_received_time_ms());
        }
        events
    }

    async fn parse_inner_instruction(
        &self,
        instruction: &UiCompiledInstruction,
        signature: &str,
        slot: Option<u64>,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Result<Vec<Box<dyn UnifiedEvent>>> {
        let slot = slot.unwrap_or(0);
        let events = self.parse_events_from_inner_instruction(
            instruction,
            signature,
            slot,
            block_time,
            program_received_time_ms,
            index,
        );
        Ok(events)
    }

    #[allow(clippy::too_many_arguments)]
    async fn parse_instruction(
        &self,
        instruction: &CompiledInstruction,
        accounts: &[Pubkey],
        signature: &str,
        slot: Option<u64>,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Result<Vec<Box<dyn UnifiedEvent>>> {
        let slot = slot.unwrap_or(0);
        let events = self.parse_events_from_instruction(
            instruction,
            accounts,
            signature,
            slot,
            block_time,
            program_received_time_ms,
            index,
        );
        Ok(events)
    }

    /// Check if this program ID should be handled
    fn should_handle(&self, program_id: &Pubkey) -> bool;

    /// Get supported program ID list
    fn supported_program_ids(&self) -> Vec<Pubkey>;
}

// Implement Clone for Box<dyn UnifiedEvent>
impl Clone for Box<dyn UnifiedEvent> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

/// Generic event parser configuration
#[derive(Debug, Clone)]
pub struct GenericEventParseConfig {
    pub program_id: Pubkey,
    pub protocol_type: ProtocolType,
    pub inner_instruction_discriminator: &'static str,
    pub instruction_discriminator: &'static [u8],
    pub event_type: EventType,
    pub inner_instruction_parser: InnerInstructionEventParser,
    pub instruction_parser: InstructionEventParser,
}

/// Inner instruction event parser
pub type InnerInstructionEventParser =
    fn(data: &[u8], metadata: EventMetadata) -> Option<Box<dyn UnifiedEvent>>;

/// Instruction event parser
pub type InstructionEventParser =
    fn(data: &[u8], accounts: &[Pubkey], metadata: EventMetadata) -> Option<Box<dyn UnifiedEvent>>;

/// Generic event parser base class
pub struct GenericEventParser {
    pub program_ids: Vec<Pubkey>,
    pub inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>,
    pub instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
}

impl GenericEventParser {
    /// Create new generic event parser
    pub fn new(program_ids: Vec<Pubkey>, configs: Vec<GenericEventParseConfig>) -> Self {
        // Pre-allocate capacity to avoid dynamic expansion
        let mut inner_instruction_configs = HashMap::with_capacity(configs.len());
        let mut instruction_configs = HashMap::with_capacity(configs.len());

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

        Self { program_ids, inner_instruction_configs, instruction_configs }
    }

    /// Generic inner instruction parsing method
    #[allow(clippy::too_many_arguments)]
    fn parse_inner_instruction_event(
        &self,
        config: &GenericEventParseConfig,
        data: &[u8],
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Option<Box<dyn UnifiedEvent>> {
        let block_time_ms = block_time.unwrap_or(0) * 1000;
        let metadata = EventMetadata::new(
            signature.to_string(),
            signature.to_string(),
            slot,
            block_time.unwrap_or(0),
            block_time_ms,
            config.protocol_type.clone(),
            config.event_type.clone(),
            config.program_id,
            index,
            program_received_time_ms,
        );
        (config.inner_instruction_parser)(data, metadata)
    }

    /// Generic instruction parsing method
    #[allow(clippy::too_many_arguments)]
    fn parse_instruction_event(
        &self,
        config: &GenericEventParseConfig,
        data: &[u8],
        account_pubkeys: &[Pubkey],
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Option<Box<dyn UnifiedEvent>> {
        let block_time_ms = block_time.unwrap_or(0) * 1000;
        let metadata = EventMetadata::new(
            signature.to_string(),
            signature.to_string(),
            slot,
            block_time.unwrap_or(0),
            block_time_ms,
            config.protocol_type.clone(),
            config.event_type.clone(),
            config.program_id,
            index,
            program_received_time_ms,
        );
        (config.instruction_parser)(data, account_pubkeys, metadata)
    }
}

#[async_trait::async_trait]
impl EventParser for GenericEventParser {
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner_instruction_configs.clone()
    }
    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.instruction_configs.clone()
    }
    /// Parse event data from inner instruction
    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_inner_instruction(
        &self,
        inner_instruction: &UiCompiledInstruction,
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        let inner_instruction_data = inner_instruction.data.clone();
        let inner_instruction_data_decoded =
            bs58::decode(inner_instruction_data).into_vec().unwrap_or_else(|_| vec![]);
        if inner_instruction_data_decoded.len() < 16 {
            return Vec::new();
        }
        let inner_instruction_data_decoded_str =
            format!("0x{}", hex::encode(&inner_instruction_data_decoded));
        let data = &inner_instruction_data_decoded[16..];
        let mut events = Vec::new();
        for (disc, configs) in &self.inner_instruction_configs {
            if discriminator_matches(&inner_instruction_data_decoded_str, disc) {
                for config in configs {
                    if let Some(event) = self.parse_inner_instruction_event(
                        config,
                        data,
                        signature,
                        slot,
                        block_time,
                        program_received_time_ms,
                        index.clone(),
                    ) {
                        events.push(event);
                    }
                }
            }
        }
        events
    }

    /// Parse events from instruction
    #[allow(clippy::too_many_arguments)]
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
        let program_id = accounts[instruction.program_id_index as usize];
        if !self.should_handle(&program_id) {
            return Vec::new();
        }
        let mut events = Vec::new();
        for (disc, configs) in &self.instruction_configs {
            if instruction.data.len() < disc.len() {
                continue;
            }
            let discriminator = &instruction.data[..disc.len()];
            let data = &instruction.data[disc.len()..];
            if discriminator == disc {
                // Validate account indices
                if !validate_account_indices(&instruction.accounts, accounts.len()) {
                    continue;
                }

                let account_pubkeys: Vec<Pubkey> =
                    instruction.accounts.iter().map(|&idx| accounts[idx as usize]).collect();
                for config in configs {
                    if config.program_id != program_id {
                        continue;
                    }
                    if let Some(event) = self.parse_instruction_event(
                        config,
                        data,
                        &account_pubkeys,
                        signature,
                        slot,
                        block_time,
                        program_received_time_ms,
                        index.clone(),
                    ) {
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