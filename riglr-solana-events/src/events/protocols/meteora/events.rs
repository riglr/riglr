use std::any::Any;
use serde::{Deserialize, Serialize};
use crate::{
    events::core::UnifiedEvent,
    types::{EventType, ProtocolType, TransferData, SwapData},
};
use super::types::{MeteoraSwapData, MeteoraLiquidityData, MeteoraDynamicLiquidityData};

/// Meteora DLMM swap event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraSwapEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub swap_data: MeteoraSwapData,
    pub transfer_data: Vec<TransferData>,
}

impl MeteoraSwapEvent {
    pub fn new(
        id: String,
        signature: String,
        slot: u64,
        block_time: i64,
        block_time_ms: i64,
        program_received_time_ms: i64,
        index: String,
        swap_data: MeteoraSwapData,
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

impl UnifiedEvent for MeteoraSwapEvent {
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
            if self.swap_data.swap_for_y {
                self.swap_data.token_mint_x = swap_data.input_mint;
                self.swap_data.token_mint_y = swap_data.output_mint;
            } else {
                self.swap_data.token_mint_x = swap_data.output_mint;
                self.swap_data.token_mint_y = swap_data.input_mint;
            }
            self.swap_data.amount_in = swap_data.amount_in;
            self.swap_data.actual_amount_out = swap_data.amount_out;
        }
    }

    fn index(&self) -> String {
        self.index.clone()
    }

    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::MeteoraDlmm
    }
}

/// Meteora DLMM liquidity event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraLiquidityEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub liquidity_data: MeteoraLiquidityData,
    pub transfer_data: Vec<TransferData>,
}

impl UnifiedEvent for MeteoraLiquidityEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn event_type(&self) -> EventType {
        if self.liquidity_data.is_add {
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
        ProtocolType::MeteoraDlmm
    }
}

/// Meteora Dynamic AMM liquidity event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDynamicLiquidityEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub liquidity_data: MeteoraDynamicLiquidityData,
    pub transfer_data: Vec<TransferData>,
}

impl UnifiedEvent for MeteoraDynamicLiquidityEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn event_type(&self) -> EventType {
        if self.liquidity_data.is_deposit {
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
        ProtocolType::Other("MeteoraDynamic".to_string())
    }
}