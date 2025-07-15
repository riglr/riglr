// use borsh::BorshDeserialize; // Not needed for simplified implementation
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::events::common::EventMetadata;
use crate::impl_unified_event;

/// Buy event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PumpSwapBuyEvent {
    #[serde(skip)]
    pub metadata: EventMetadata,
    pub timestamp: i64,
    pub base_amount_out: u64,
    pub max_quote_amount_in: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub quote_amount_in: u64,
    pub lp_fee_basis_points: u64,
    pub lp_fee: u64,
    pub protocol_fee_basis_points: u64,
    pub protocol_fee: u64,
    pub quote_amount_in_with_lp_fee: u64,
    pub user_quote_amount_in: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub protocol_fee_recipient: Pubkey,
    pub protocol_fee_recipient_token_account: Pubkey,
    pub coin_creator: Pubkey,
    pub coin_creator_fee_basis_points: u64,
    pub coin_creator_fee: u64,
    pub track_volume: bool,
    pub total_unclaimed_tokens: u64,
    pub total_claimed_tokens: u64,
    pub current_sol_volume: u64,
    pub last_update_timestamp: i64,
    #[serde(skip)]
    pub base_mint: Pubkey,
    #[serde(skip)]
    pub quote_mint: Pubkey,
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
    #[serde(skip)]
    pub coin_creator_vault_ata: Pubkey,
    #[serde(skip)]
    pub coin_creator_vault_authority: Pubkey,
}

// Use macro to generate UnifiedEvent implementation, specifying fields to merge
impl_unified_event!(
    PumpSwapBuyEvent,
    timestamp,
    base_amount_out,
    max_quote_amount_in,
    user_base_token_reserves,
    user_quote_token_reserves,
    pool_base_token_reserves,
    pool_quote_token_reserves,
    quote_amount_in,
    lp_fee_basis_points,
    lp_fee,
    protocol_fee_basis_points,
    protocol_fee,
    quote_amount_in_with_lp_fee,
    user_quote_amount_in,
    pool,
    user,
    user_base_token_account,
    user_quote_token_account,
    protocol_fee_recipient,
    protocol_fee_recipient_token_account,
    coin_creator,
    coin_creator_fee_basis_points,
    coin_creator_fee
);

/// Sell event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PumpSwapSellEvent {
    #[serde(skip)]
    pub metadata: EventMetadata,
    pub timestamp: i64,
    pub base_amount_in: u64,
    pub min_quote_amount_out: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub quote_amount_out: u64,
    pub lp_fee_basis_points: u64,
    pub lp_fee: u64,
    pub protocol_fee_basis_points: u64,
    pub protocol_fee: u64,
    pub quote_amount_out_without_lp_fee: u64,
    pub user_quote_amount_out: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub protocol_fee_recipient: Pubkey,
    pub protocol_fee_recipient_token_account: Pubkey,
    pub coin_creator: Pubkey,
    pub coin_creator_fee_basis_points: u64,
    pub coin_creator_fee: u64,
    #[serde(skip)]
    pub base_mint: Pubkey,
    #[serde(skip)]
    pub quote_mint: Pubkey,
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
    #[serde(skip)]
    pub coin_creator_vault_ata: Pubkey,
    #[serde(skip)]
    pub coin_creator_vault_authority: Pubkey,
}

// Use macro to generate UnifiedEvent implementation, specifying fields to merge
impl_unified_event!(
    PumpSwapSellEvent,
    timestamp,
    base_amount_in,
    min_quote_amount_out,
    user_base_token_reserves,
    user_quote_token_reserves,
    pool_base_token_reserves,
    pool_quote_token_reserves,
    quote_amount_out,
    lp_fee_basis_points,
    lp_fee,
    protocol_fee_basis_points,
    protocol_fee,
    quote_amount_out_without_lp_fee,
    user_quote_amount_out,
    pool,
    user,
    user_base_token_account,
    user_quote_token_account,
    protocol_fee_recipient,
    protocol_fee_recipient_token_account,
    coin_creator,
    coin_creator_fee_basis_points,
    coin_creator_fee
);

/// Create pool event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PumpSwapCreatePoolEvent {
    #[serde(skip)]
    pub metadata: EventMetadata,
    pub timestamp: i64,
    pub index: u16,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_mint_decimals: u8,
    pub quote_mint_decimals: u8,
    pub base_amount_in: u64,
    pub quote_amount_in: u64,
    pub pool_base_amount: u64,
    pub pool_quote_amount: u64,
    pub minimum_liquidity: u64,
    pub initial_liquidity: u64,
    pub lp_token_amount_out: u64,
    pub pool_bump: u8,
    pub pool: Pubkey,
    pub lp_mint: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub coin_creator: Pubkey,
    #[serde(skip)]
    pub user_pool_token_account: Pubkey,
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
}

impl_unified_event!(
    PumpSwapCreatePoolEvent,
    timestamp,
    index,
    creator,
    base_mint,
    quote_mint,
    base_mint_decimals,
    quote_mint_decimals,
    base_amount_in,
    quote_amount_in,
    pool_base_amount,
    pool_quote_amount,
    minimum_liquidity,
    initial_liquidity,
    lp_token_amount_out,
    pool_bump,
    pool,
    lp_mint,
    user_base_token_account,
    user_quote_token_account,
    coin_creator
);

/// Deposit event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PumpSwapDepositEvent {
    #[serde(skip)]
    pub metadata: EventMetadata,
    pub timestamp: i64,
    pub lp_token_amount_out: u64,
    pub max_base_amount_in: u64,
    pub max_quote_amount_in: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub base_amount_in: u64,
    pub quote_amount_in: u64,
    pub lp_mint_supply: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub user_pool_token_account: Pubkey,
    #[serde(skip)]
    pub base_mint: Pubkey,
    #[serde(skip)]
    pub quote_mint: Pubkey,
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
}

impl_unified_event!(
    PumpSwapDepositEvent,
    timestamp,
    lp_token_amount_out,
    max_base_amount_in,
    max_quote_amount_in,
    user_base_token_reserves,
    user_quote_token_reserves,
    pool_base_token_reserves,
    pool_quote_token_reserves,
    base_amount_in,
    quote_amount_in,
    lp_mint_supply,
    pool,
    user,
    user_base_token_account,
    user_quote_token_account,
    user_pool_token_account
);

/// Withdraw event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PumpSwapWithdrawEvent {
    #[serde(skip)]
    pub metadata: EventMetadata,
    pub timestamp: i64,
    pub lp_token_amount_in: u64,
    pub min_base_amount_out: u64,
    pub min_quote_amount_out: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub base_amount_out: u64,
    pub quote_amount_out: u64,
    pub lp_mint_supply: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub user_pool_token_account: Pubkey,
    #[serde(skip)]
    pub base_mint: Pubkey,
    #[serde(skip)]
    pub quote_mint: Pubkey,
    #[serde(skip)]
    pub pool_base_token_account: Pubkey,
    #[serde(skip)]
    pub pool_quote_token_account: Pubkey,
}

impl_unified_event!(
    PumpSwapWithdrawEvent,
    timestamp,
    lp_token_amount_in,
    min_base_amount_out,
    min_quote_amount_out,
    user_base_token_reserves,
    user_quote_token_reserves,
    pool_base_token_reserves,
    pool_quote_token_reserves,
    base_amount_out,
    quote_amount_out,
    lp_mint_supply,
    pool,
    user,
    user_base_token_account,
    user_quote_token_account,
    user_pool_token_account
);

/// Event discriminator constants
pub mod discriminators {
    // Event discriminators
    pub const BUY_EVENT: &str = "0xe445a52e51cb9a1d67f4521f2cf57777";
    pub const SELL_EVENT: &str = "0xe445a52e51cb9a1d3e2f370aa503dc2a";
    pub const CREATE_POOL_EVENT: &str = "0xe445a52e51cb9a1db1310cd2a076a774";
    pub const DEPOSIT_EVENT: &str = "0xe445a52e51cb9a1d78f83d531f8e6b90";
    pub const WITHDRAW_EVENT: &str = "0xe445a52e51cb9a1d1609851aa02c47c0";

    // Instruction discriminators
    pub const BUY_IX: &[u8] = &[102, 6, 61, 18, 1, 218, 235, 234];
    pub const SELL_IX: &[u8] = &[51, 230, 133, 164, 1, 127, 131, 173];
    pub const CREATE_POOL_IX: &[u8] = &[233, 146, 209, 142, 207, 104, 64, 188];
    pub const DEPOSIT_IX: &[u8] = &[242, 35, 198, 137, 82, 225, 242, 182];
    pub const WITHDRAW_IX: &[u8] = &[183, 18, 70, 156, 148, 109, 161, 34];
}