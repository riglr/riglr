use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use crate::events::{
    common::{EventMetadata, EventType, ProtocolType},
    core::traits::UnifiedEvent,
};

/// Raydium CLMM swap event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmSwapEvent {
    pub metadata: EventMetadata,
    pub amount0: u64,
    pub amount1: u64,
    pub sqrt_price_x64: u128,
    pub liquidity: u128,
    pub tick_current: i32,
    
    // Account keys
    pub payer: Pubkey,
    pub pool_state: Pubkey,
    pub input_token_account: Pubkey,
    pub output_token_account: Pubkey,
    pub input_vault: Pubkey,
    pub output_vault: Pubkey,
    pub token_mint0: Pubkey,
    pub token_mint1: Pubkey,
}

crate::impl_unified_event!(RaydiumClmmSwapEvent);

/// Raydium CLMM swap V2 event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmSwapV2Event {
    pub metadata: EventMetadata,
    pub amount0: u64,
    pub amount1: u64,
    pub sqrt_price_x64: u128,
    pub liquidity: u128,
    pub tick_current: i32,
    pub is_base_input: bool,
    
    // Account keys
    pub payer: Pubkey,
    pub pool_state: Pubkey,
    pub input_token_account: Pubkey,
    pub output_token_account: Pubkey,
    pub input_vault: Pubkey,
    pub output_vault: Pubkey,
    pub token_mint0: Pubkey,
    pub token_mint1: Pubkey,
}

crate::impl_unified_event!(RaydiumClmmSwapV2Event);

/// Raydium CLMM create pool event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmCreatePoolEvent {
    pub metadata: EventMetadata,
    pub sqrt_price_x64: u128,
    pub tick_current: i32,
    pub observation_index: u16,
    
    // Account keys
    pub pool_creator: Pubkey,
    pub pool_state: Pubkey,
    pub token_mint0: Pubkey,
    pub token_mint1: Pubkey,
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
}

crate::impl_unified_event!(RaydiumClmmCreatePoolEvent);

/// Raydium CLMM open position V2 event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmOpenPositionV2Event {
    pub metadata: EventMetadata,
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub tick_array_lower_start_index: i32,
    pub tick_array_upper_start_index: i32,
    pub liquidity: u128,
    pub amount0_max: u64,
    pub amount1_max: u64,
    pub with_metadata: bool,
    pub base_flag: Option<bool>,
    
    // Account keys
    pub payer: Pubkey,
    pub position_nft_owner: Pubkey,
    pub position_nft_mint: Pubkey,
    pub position_nft_account: Pubkey,
    pub metadata_account: Pubkey,
    pub pool_state: Pubkey,
}

crate::impl_unified_event!(RaydiumClmmOpenPositionV2Event);

/// Raydium CLMM close position event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmClosePositionEvent {
    pub metadata: EventMetadata,
    
    // Account keys
    pub nft_owner: Pubkey,
    pub position_nft_mint: Pubkey,
    pub position_nft_account: Pubkey,
    pub personal_position: Pubkey,
}

crate::impl_unified_event!(RaydiumClmmClosePositionEvent);

/// Raydium CLMM increase liquidity V2 event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmIncreaseLiquidityV2Event {
    pub metadata: EventMetadata,
    pub liquidity: u128,
    pub amount0_max: u64,
    pub amount1_max: u64,
    pub base_flag: Option<bool>,
    
    // Account keys
    pub nft_owner: Pubkey,
    pub position_nft_account: Pubkey,
    pub pool_state: Pubkey,
}

crate::impl_unified_event!(RaydiumClmmIncreaseLiquidityV2Event);

/// Raydium CLMM decrease liquidity V2 event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmDecreaseLiquidityV2Event {
    pub metadata: EventMetadata,
    pub liquidity: u128,
    pub amount0_min: u64,
    pub amount1_min: u64,
    
    // Account keys
    pub nft_owner: Pubkey,
    pub position_nft_account: Pubkey,
    pub pool_state: Pubkey,
}

crate::impl_unified_event!(RaydiumClmmDecreaseLiquidityV2Event);

/// Raydium CLMM open position with Token-22 NFT event  
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmOpenPositionWithToken22NftEvent {
    pub metadata: EventMetadata,
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub tick_array_lower_start_index: i32,
    pub tick_array_upper_start_index: i32,
    pub liquidity: u128,
    pub amount0_max: u64,
    pub amount1_max: u64,
    pub with_metadata: bool,
    pub base_flag: Option<bool>,
    
    // Account keys
    pub payer: Pubkey,
    pub position_nft_owner: Pubkey,
    pub position_nft_mint: Pubkey,
    pub position_nft_account: Pubkey,
    pub pool_state: Pubkey,
}

crate::impl_unified_event!(RaydiumClmmOpenPositionWithToken22NftEvent);