use crate::events::common::EventMetadata;
use riglr_events_core::{Event, EventKind, EventMetadata as CoreEventMetadata};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::any::Any;

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

// Event trait implementations

impl Event for RaydiumAmmV4SwapEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        static SWAP_KIND: EventKind = EventKind::Swap;
        &SWAP_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        // Convert our metadata to core metadata and cache it (placeholder implementation)
        use std::sync::OnceLock;
        static METADATA_CACHE: OnceLock<CoreEventMetadata> = OnceLock::new();

        METADATA_CACHE.get_or_init(|| {
            CoreEventMetadata::new(String::new(), EventKind::Swap, "raydium-amm-v4".to_string())
        })
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        unimplemented!("Mutable metadata not yet implemented")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

impl Event for RaydiumAmmV4DepositEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
        &LIQUIDITY_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        use std::sync::OnceLock;
        static METADATA_CACHE: OnceLock<CoreEventMetadata> = OnceLock::new();

        METADATA_CACHE.get_or_init(|| {
            CoreEventMetadata::new(
                String::new(),
                EventKind::Liquidity,
                "raydium-amm-v4".to_string(),
            )
        })
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        unimplemented!("Mutable metadata not yet implemented")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

impl Event for RaydiumAmmV4Initialize2Event {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        static CONTRACT_KIND: EventKind = EventKind::Contract;
        &CONTRACT_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        use std::sync::OnceLock;
        static METADATA_CACHE: OnceLock<CoreEventMetadata> = OnceLock::new();

        METADATA_CACHE.get_or_init(|| {
            CoreEventMetadata::new(
                String::new(),
                EventKind::Contract,
                "raydium-amm-v4".to_string(),
            )
        })
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        unimplemented!("Mutable metadata not yet implemented")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

impl Event for RaydiumAmmV4WithdrawEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
        &LIQUIDITY_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        use std::sync::OnceLock;
        static METADATA_CACHE: OnceLock<CoreEventMetadata> = OnceLock::new();

        METADATA_CACHE.get_or_init(|| {
            CoreEventMetadata::new(
                String::new(),
                EventKind::Liquidity,
                "raydium-amm-v4".to_string(),
            )
        })
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        unimplemented!("Mutable metadata not yet implemented")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

impl Event for RaydiumAmmV4WithdrawPnlEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        static TRANSFER_KIND: EventKind = EventKind::Transfer;
        &TRANSFER_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        use std::sync::OnceLock;
        static METADATA_CACHE: OnceLock<CoreEventMetadata> = OnceLock::new();

        METADATA_CACHE.get_or_init(|| {
            CoreEventMetadata::new(
                String::new(),
                EventKind::Transfer,
                "raydium-amm-v4".to_string(),
            )
        })
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        unimplemented!("Mutable metadata not yet implemented")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}
