use std::any::Any;
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use crate::{
    events::core::UnifiedEvent,
    events::common::EventMetadata,
    types::{EventType, ProtocolType, TransferData, SwapData},
    impl_unified_event,
};
use super::types::JupiterSwapData;

/// Jupiter swap event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterSwapEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub swap_data: JupiterSwapData,
    pub transfer_data: Vec<TransferData>,
}

impl JupiterSwapEvent {
    pub fn new(
        id: String,
        signature: String,
        slot: u64,
        block_time: i64,
        block_time_ms: i64,
        program_received_time_ms: i64,
        index: String,
        swap_data: JupiterSwapData,
    ) -> Self {
        Self {
            id,
            signature,
            slot,
            block_time,
            block_time_ms,
            program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index,
            swap_data,
            transfer_data: Vec::new(),
        }
    }

    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

impl UnifiedEvent for JupiterSwapEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn event_type(&self) -> EventType {
        EventType::Swap
    }

    fn signature(&self) -> &str {
        &self.signature
    }

    fn slot(&self) -> u64 {
        self.slot
    }

    fn program_received_time_ms(&self) -> i64 {
        self.program_received_time_ms
    }

    fn program_handle_time_consuming_ms(&self) -> i64 {
        self.program_handle_time_consuming_ms
    }

    fn set_program_handle_time_consuming_ms(&mut self, program_handle_time_consuming_ms: i64) {
        self.program_handle_time_consuming_ms = program_handle_time_consuming_ms;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn UnifiedEvent> {
        Box::new(self.clone())
    }

    fn set_transfer_data(&mut self, transfer_data: Vec<TransferData>, swap_data: Option<SwapData>) {
        self.transfer_data = transfer_data;
        if let Some(swap_data) = swap_data {
            self.swap_data.input_mint = swap_data.input_mint;
            self.swap_data.output_mint = swap_data.output_mint;
            self.swap_data.input_amount = swap_data.amount_in;
            self.swap_data.output_amount = swap_data.amount_out;
        }
    }

    fn index(&self) -> String {
        self.index.clone()
    }

    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::Other("Jupiter".to_string())
    }

}

/// Jupiter liquidity provision event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterLiquidityEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub user: solana_sdk::pubkey::Pubkey,
    pub mint_a: solana_sdk::pubkey::Pubkey,
    pub mint_b: solana_sdk::pubkey::Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub liquidity_amount: u64,
    pub is_remove: bool,
    pub transfer_data: Vec<TransferData>,
}

impl UnifiedEvent for JupiterLiquidityEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn event_type(&self) -> EventType {
        if self.is_remove {
            EventType::RemoveLiquidity
        } else {
            EventType::AddLiquidity
        }
    }

    fn signature(&self) -> &str {
        &self.signature
    }

    fn slot(&self) -> u64 {
        self.slot
    }

    fn program_received_time_ms(&self) -> i64 {
        self.program_received_time_ms
    }

    fn program_handle_time_consuming_ms(&self) -> i64 {
        self.program_handle_time_consuming_ms
    }

    fn set_program_handle_time_consuming_ms(&mut self, program_handle_time_consuming_ms: i64) {
        self.program_handle_time_consuming_ms = program_handle_time_consuming_ms;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn UnifiedEvent> {
        Box::new(self.clone())
    }

    fn set_transfer_data(&mut self, transfer_data: Vec<TransferData>, _swap_data: Option<SwapData>) {
        self.transfer_data = transfer_data;
    }

    fn index(&self) -> String {
        self.index.clone()
    }

    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::Other("Jupiter".to_string())
    }

}

/// Jupiter swap event with borsh (for simple events)
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct JupiterSwapBorshEvent {
    #[serde(skip)]
    pub metadata: EventMetadata,
    pub user: Pubkey,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub input_amount: u64,
    pub output_amount: u64,
    pub slippage_bps: u16,
    pub platform_fee_bps: u8,
}

// Implement UnifiedEvent for the borsh event
impl_unified_event!(JupiterSwapBorshEvent);
