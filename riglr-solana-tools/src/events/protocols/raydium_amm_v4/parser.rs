use std::collections::HashMap;

use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;

use crate::events::{
    common::{EventMetadata, EventType, ProtocolType, read_u64_le, read_u8_le},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
    protocols::raydium_amm_v4::{
        discriminators, RaydiumAmmV4SwapEvent, RaydiumAmmV4DepositEvent,
        RaydiumAmmV4Initialize2Event, RaydiumAmmV4WithdrawEvent, 
        RaydiumAmmV4WithdrawPnlEvent, SwapDirection,
    },
};

/// Raydium AMM V4 program ID
pub const RAYDIUM_AMM_V4_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

/// Raydium AMM V4 event parser
pub struct RaydiumAmmV4EventParser {
    inner: GenericEventParser,
}

impl Default for RaydiumAmmV4EventParser {
    fn default() -> Self {
        Self::new()
    }
}

impl RaydiumAmmV4EventParser {
    pub fn new() -> Self {
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
    fn empty_parse(_data: &[u8], _metadata: EventMetadata) -> Option<Box<dyn UnifiedEvent>> {
        None
    }

    /// Parse swap base input instruction event  
    fn parse_swap_base_input_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 17 {
            return None;
        }

        let amount_in = read_u64_le(data, 0)?;
        let minimum_amount_out = read_u64_le(data, 8)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-swap-in-{}",
            metadata.signature, accounts[1], amount_in
        ));

        Some(Box::new(RaydiumAmmV4SwapEvent {
            metadata,
            amount_in,
            amount_out: minimum_amount_out, // Use minimum as estimate, actual amount determined on-chain
            direction: SwapDirection::BaseIn,
            amm: accounts[1],
            amm_authority: accounts[2],
            amm_open_orders: accounts[3],
            pool_coin_token_account: accounts[6],
            pool_pc_token_account: accounts[7],
            serum_program: accounts[8],
            serum_market: accounts[9],
            user_coin_token_account: accounts[14],
            user_pc_token_account: accounts[15],
            user_owner: accounts[16],
        }))
    }

    /// Parse swap base output instruction event
    fn parse_swap_base_output_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 17 {
            return None;
        }

        let max_amount_in = read_u64_le(data, 0)?;
        let amount_out = read_u64_le(data, 8)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-swap-out-{}",
            metadata.signature, accounts[1], amount_out
        ));

        Some(Box::new(RaydiumAmmV4SwapEvent {
            metadata,
            amount_in: max_amount_in, // Use maximum as estimate, actual amount determined on-chain
            amount_out,
            direction: SwapDirection::BaseOut,
            amm: accounts[1],
            amm_authority: accounts[2],
            amm_open_orders: accounts[3],
            pool_coin_token_account: accounts[6],
            pool_pc_token_account: accounts[7],
            serum_program: accounts[8],
            serum_market: accounts[9],
            user_coin_token_account: accounts[14],
            user_pc_token_account: accounts[15],
            user_owner: accounts[16],
        }))
    }

    /// Parse deposit instruction event
    fn parse_deposit_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 24 || accounts.len() < 13 {
            return None;
        }

        let max_coin_amount = read_u64_le(data, 0)?;
        let max_pc_amount = read_u64_le(data, 8)?;
        let base_side = read_u64_le(data, 16)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-deposit-{}-{}",
            metadata.signature, accounts[1], max_coin_amount, max_pc_amount
        ));

        Some(Box::new(RaydiumAmmV4DepositEvent {
            metadata,
            max_coin_amount,
            max_pc_amount,
            base_side,
            token_program: accounts[0],
            amm: accounts[1],
            amm_authority: accounts[2],
            amm_open_orders: accounts[3],
            amm_target_orders: accounts[4],
            lp_mint_address: accounts[5],
            pool_coin_token_account: accounts[6],
            pool_pc_token_account: accounts[7],
            serum_market: accounts[8],
            user_coin_token_account: accounts[9],
            user_pc_token_account: accounts[10],
            user_lp_token_account: accounts[11],
            user_owner: accounts[12],
        }))
    }

    /// Parse initialize2 instruction event
    fn parse_initialize2_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 25 || accounts.len() < 21 {
            return None;
        }

        let nonce = read_u8_le(data, 0)?;
        let open_time = read_u64_le(data, 1)?;
        let init_pc_amount = read_u64_le(data, 9)?;
        let init_coin_amount = read_u64_le(data, 17)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-init-{}-{}",
            metadata.signature, accounts[4], init_coin_amount, init_pc_amount
        ));

        Some(Box::new(RaydiumAmmV4Initialize2Event {
            metadata,
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
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 8 || accounts.len() < 22 {
            return None;
        }

        let amount = read_u64_le(data, 0)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-withdraw-{}",
            metadata.signature, accounts[1], amount
        ));

        Some(Box::new(RaydiumAmmV4WithdrawEvent {
            metadata,
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
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if accounts.len() < 17 {
            return None;
        }

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-withdraw-pnl",
            metadata.signature, accounts[1]
        ));

        Some(Box::new(RaydiumAmmV4WithdrawPnlEvent {
            metadata,
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

impl EventParser for RaydiumAmmV4EventParser {
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