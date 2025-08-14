use std::any::Any;
use serde::{Deserialize, Serialize};
use crate::{
    events::core::UnifiedEvent,
    types::{EventType, ProtocolType, TransferData, SwapData},
};
use super::types::{OrcaSwapData, OrcaPositionData, OrcaLiquidityData};

/// Orca Whirlpool swap event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaSwapEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub swap_data: OrcaSwapData,
    pub transfer_data: Vec<TransferData>,
}

impl OrcaSwapEvent {
    pub fn new(
        id: String,
        signature: String,
        slot: u64,
        block_time: i64,
        block_time_ms: i64,
        program_received_time_ms: i64,
        index: String,
        swap_data: OrcaSwapData,
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

impl UnifiedEvent for OrcaSwapEvent {
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
            if self.swap_data.a_to_b {
                self.swap_data.token_mint_a = swap_data.input_mint;
                self.swap_data.token_mint_b = swap_data.output_mint;
            } else {
                self.swap_data.token_mint_a = swap_data.output_mint;
                self.swap_data.token_mint_b = swap_data.input_mint;
            }
            self.swap_data.amount_in = swap_data.amount_in;
            self.swap_data.amount_out = swap_data.amount_out;
        }
    }

    fn index(&self) -> String {
        self.index.clone()
    }

    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::OrcaWhirlpool
    }
}

/// Orca position event (open/close)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaPositionEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub position_data: OrcaPositionData,
    pub is_open: bool,
    pub transfer_data: Vec<TransferData>,
}

impl UnifiedEvent for OrcaPositionEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn event_type(&self) -> EventType {
        if self.is_open {
            EventType::CreatePool
        } else {
            EventType::Unknown
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
        ProtocolType::OrcaWhirlpool
    }
}

/// Orca liquidity event (increase/decrease)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaLiquidityEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub liquidity_data: OrcaLiquidityData,
    pub transfer_data: Vec<TransferData>,
}

impl UnifiedEvent for OrcaLiquidityEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn event_type(&self) -> EventType {
        if self.liquidity_data.is_increase {
            EventType::AddLiquidity
        } else {
            EventType::RemoveLiquidity
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
        ProtocolType::OrcaWhirlpool
    }
}