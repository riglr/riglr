// use borsh::BorshDeserialize; // Not needed for simplified implementation
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::events::common::EventMetadata;
use crate::impl_unified_event;

/// Raydium CPMM Swap event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaydiumCpmmSwapEvent {
    #[serde(skip)]
    pub metadata: EventMetadata,
    pub pool_state: Pubkey,
    pub payer: Pubkey,
    pub input_token_account: Pubkey,
    pub output_token_account: Pubkey,
    pub input_vault: Pubkey,
    pub output_vault: Pubkey,
    pub input_token_mint: Pubkey,
    pub output_token_mint: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub trade_fee: u64,
    pub transfer_fee: u64,
}

impl_unified_event!(
    RaydiumCpmmSwapEvent,
    pool_state,
    payer,
    input_token_account,
    output_token_account,
    input_vault,
    output_vault,
    input_token_mint,
    output_token_mint,
    amount_in,
    amount_out,
    trade_fee,
    transfer_fee
);

/// Raydium CPMM Deposit event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaydiumCpmmDepositEvent {
    #[serde(skip)]
    pub metadata: EventMetadata,
    pub pool_state: Pubkey,
    pub user: Pubkey,
    pub lp_token_amount: u64,
    pub token_0_amount: u64,
    pub token_1_amount: u64,
}

impl_unified_event!(
    RaydiumCpmmDepositEvent,
    pool_state,
    user,
    lp_token_amount,
    token_0_amount,
    token_1_amount
);

/// Event discriminator constants
pub mod discriminators {
    // Event discriminators
    pub const SWAP_EVENT: &str = "0xe445a52e51cb9a1d0a11a9d2be8b72b1";
    pub const DEPOSIT_EVENT: &str = "0xe445a52e51cb9a1d0b11a9d2be8b72b2";
    
    // Instruction discriminators
    pub const SWAP_BASE_INPUT_IX: &[u8] = &[143, 190, 90, 218, 196, 30, 51, 222];
    pub const SWAP_BASE_OUTPUT_IX: &[u8] = &[55, 217, 98, 86, 163, 74, 180, 175];
    pub const DEPOSIT_IX: &[u8] = &[242, 35, 198, 137, 82, 225, 242, 182];
    pub const INITIALIZE_IX: &[u8] = &[175, 175, 109, 31, 13, 152, 155, 237];
    pub const WITHDRAW_IX: &[u8] = &[183, 18, 70, 156, 148, 109, 161, 34];
}