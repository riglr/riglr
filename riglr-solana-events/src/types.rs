use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProtocolType {
    OrcaWhirlpool,
    MeteoraDlmm,
    MarginFi,
    Other(&'static str),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    Swap,
    AddLiquidity,
    RemoveLiquidity,
    Borrow,
    Repay,
    Liquidate,
    Transfer,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub protocol: ProtocolType,
    pub event_type: EventType,
    pub program_id: Pubkey,
    pub index: String,
    pub program_received_time_ms: i64,
}

impl EventMetadata {
    pub fn new(
        id: String,
        signature: String,
        slot: u64,
        block_time: i64,
        block_time_ms: i64,
        protocol: ProtocolType,
        event_type: EventType,
        program_id: Pubkey,
        index: String,
        program_received_time_ms: i64,
    ) -> Self {
        Self { id, signature, slot, block_time, block_time_ms, protocol, event_type, program_id, index, program_received_time_ms }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferData {
    pub source: Pubkey,
    pub destination: Pubkey,
    pub mint: Option<Pubkey>,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapData {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
}
