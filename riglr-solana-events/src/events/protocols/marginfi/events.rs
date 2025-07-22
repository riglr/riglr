use std::any::Any;
use serde::{Deserialize, Serialize};
use crate::{
    events::core::UnifiedEvent,
    types::{EventType, ProtocolType, TransferData, SwapData},
};
use super::types::{
    MarginFiDepositData, MarginFiWithdrawData, MarginFiBorrowData, 
    MarginFiRepayData, MarginFiLiquidationData
};

/// MarginFi deposit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiDepositEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub deposit_data: MarginFiDepositData,
    pub transfer_data: Vec<TransferData>,
}

impl MarginFiDepositEvent {
    pub fn new(
        id: String,
        signature: String,
        slot: u64,
        block_time: i64,
        block_time_ms: i64,
        program_received_time_ms: i64,
        index: String,
        deposit_data: MarginFiDepositData,
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
            deposit_data,
            transfer_data: Vec::new(),
        }
    }

    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

impl UnifiedEvent for MarginFiDepositEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn event_type(&self) -> EventType {
        EventType::AddLiquidity
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
        ProtocolType::MarginFi
    }

}

/// MarginFi withdraw event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiWithdrawEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub withdraw_data: MarginFiWithdrawData,
    pub transfer_data: Vec<TransferData>,
}

impl UnifiedEvent for MarginFiWithdrawEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn event_type(&self) -> EventType {
        EventType::RemoveLiquidity
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
        ProtocolType::MarginFi
    }

}

/// MarginFi borrow event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiBorrowEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub borrow_data: MarginFiBorrowData,
    pub transfer_data: Vec<TransferData>,
}

impl UnifiedEvent for MarginFiBorrowEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn event_type(&self) -> EventType {
        EventType::Borrow
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
        ProtocolType::MarginFi
    }

}

/// MarginFi repay event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiRepayEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub repay_data: MarginFiRepayData,
    pub transfer_data: Vec<TransferData>,
}

impl UnifiedEvent for MarginFiRepayEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn event_type(&self) -> EventType {
        EventType::Repay
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
        ProtocolType::MarginFi
    }

}

/// MarginFi liquidation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiLiquidationEvent {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub index: String,
    pub liquidation_data: MarginFiLiquidationData,
    pub transfer_data: Vec<TransferData>,
}

impl UnifiedEvent for MarginFiLiquidationEvent {
    fn id(&self) -> &str {
        &self.id
    }

    fn event_type(&self) -> EventType {
        EventType::Liquidate
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
        ProtocolType::MarginFi
    }

}