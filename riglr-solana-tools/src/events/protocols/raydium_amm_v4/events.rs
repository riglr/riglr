use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use crate::events::{
    common::{EventMetadata, EventType, ProtocolType},
    core::traits::UnifiedEvent,
};

/// Raydium AMM V4 swap event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumAmmV4SwapEvent {
    pub metadata: EventMetadata,
    pub amount_in: u64,
    pub amount_out: u64,
    pub direction: SwapDirection,
    
    // Account keys
    pub amm: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_open_orders: Pubkey,
    pub pool_coin_token_account: Pubkey,
    pub pool_pc_token_account: Pubkey,
    pub serum_program: Pubkey,
    pub serum_market: Pubkey,
    pub user_coin_token_account: Pubkey,
    pub user_pc_token_account: Pubkey,
    pub user_owner: Pubkey,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum SwapDirection {
    #[default]
    BaseIn,
    BaseOut,
}

crate::impl_unified_event!(RaydiumAmmV4SwapEvent);

/// Raydium AMM V4 deposit event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumAmmV4DepositEvent {
    pub metadata: EventMetadata,
    pub max_coin_amount: u64,
    pub max_pc_amount: u64,
    pub base_side: u64,
    
    // Account keys
    pub token_program: Pubkey,
    pub amm: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_open_orders: Pubkey,
    pub amm_target_orders: Pubkey,
    pub lp_mint_address: Pubkey,
    pub pool_coin_token_account: Pubkey,
    pub pool_pc_token_account: Pubkey,
    pub serum_market: Pubkey,
    pub user_coin_token_account: Pubkey,
    pub user_pc_token_account: Pubkey,
    pub user_lp_token_account: Pubkey,
    pub user_owner: Pubkey,
}

crate::impl_unified_event!(RaydiumAmmV4DepositEvent);

/// Raydium AMM V4 initialize2 event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumAmmV4Initialize2Event {
    pub metadata: EventMetadata,
    pub nonce: u8,
    pub open_time: u64,
    pub init_pc_amount: u64,
    pub init_coin_amount: u64,
    
    // Account keys
    pub amm: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_open_orders: Pubkey,
    pub lp_mint_address: Pubkey,
    pub coin_mint_address: Pubkey,
    pub pc_mint_address: Pubkey,
    pub pool_coin_token_account: Pubkey,
    pub pool_pc_token_account: Pubkey,
    pub pool_withdraw_queue: Pubkey,
    pub amm_target_orders: Pubkey,
    pub pool_lp_token_account: Pubkey,
    pub pool_temp_lp_token_account: Pubkey,
    pub serum_program: Pubkey,
    pub serum_market: Pubkey,
    pub user_wallet: Pubkey,
}

crate::impl_unified_event!(RaydiumAmmV4Initialize2Event);

/// Raydium AMM V4 withdraw event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumAmmV4WithdrawEvent {
    pub metadata: EventMetadata,
    pub amount: u64,
    
    // Account keys
    pub token_program: Pubkey,
    pub amm: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_open_orders: Pubkey,
    pub amm_target_orders: Pubkey,
    pub lp_mint_address: Pubkey,
    pub pool_coin_token_account: Pubkey,
    pub pool_pc_token_account: Pubkey,
    pub pool_withdraw_queue: Pubkey,
    pub pool_temp_lp_token_account: Pubkey,
    pub serum_program: Pubkey,
    pub serum_market: Pubkey,
    pub serum_coin_vault_account: Pubkey,
    pub serum_pc_vault_account: Pubkey,
    pub serum_vault_signer: Pubkey,
    pub user_lp_token_account: Pubkey,
    pub user_coin_token_account: Pubkey,
    pub user_pc_token_account: Pubkey,
    pub user_owner: Pubkey,
    pub serum_event_queue: Pubkey,
    pub serum_bids: Pubkey,
    pub serum_asks: Pubkey,
}

crate::impl_unified_event!(RaydiumAmmV4WithdrawEvent);

/// Raydium AMM V4 withdraw PNL event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumAmmV4WithdrawPnlEvent {
    pub metadata: EventMetadata,
    
    // Account keys
    pub token_program: Pubkey,
    pub amm: Pubkey,
    pub amm_config: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_open_orders: Pubkey,
    pub pool_coin_token_account: Pubkey,
    pub pool_pc_token_account: Pubkey,
    pub coin_pnl_token_account: Pubkey,
    pub pc_pnl_token_account: Pubkey,
    pub pnl_owner_account: Pubkey,
    pub amm_target_orders: Pubkey,
    pub serum_program: Pubkey,
    pub serum_market: Pubkey,
    pub serum_event_queue: Pubkey,
    pub serum_coin_vault_account: Pubkey,
    pub serum_pc_vault_account: Pubkey,
    pub serum_vault_signer: Pubkey,
}

crate::impl_unified_event!(RaydiumAmmV4WithdrawPnlEvent);