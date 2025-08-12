use riglr_events_core::{Event, EventKind, EventMetadata};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::any::Any;

/// Raydium AMM V4 swap event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumAmmV4SwapEvent {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Amount of tokens going into the swap
    pub amount_in: u64,
    /// Amount of tokens coming out of the swap
    pub amount_out: u64,
    /// Direction of the swap (BaseIn or BaseOut)
    pub direction: SwapDirection,

    // Account keys
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC (price currency) token account
    pub pool_pc_token_account: Pubkey,
    /// Serum program account
    pub serum_program: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// User coin token account
    pub user_coin_token_account: Pubkey,
    /// User PC token account
    pub user_pc_token_account: Pubkey,
    /// User owner account
    pub user_owner: Pubkey,
}

/// Direction of the swap operation
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum SwapDirection {
    /// Swapping base token in for quote token out
    #[default]
    BaseIn,
    /// Swapping quote token in for base token out
    BaseOut,
}

/// Raydium AMM V4 deposit event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumAmmV4DepositEvent {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Maximum amount of coin tokens to deposit
    pub max_coin_amount: u64,
    /// Maximum amount of PC tokens to deposit
    pub max_pc_amount: u64,
    /// Base side identifier
    pub base_side: u64,

    // Account keys
    /// Token program account
    pub token_program: Pubkey,
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// AMM target orders account
    pub amm_target_orders: Pubkey,
    /// Liquidity provider mint address
    pub lp_mint_address: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC token account
    pub pool_pc_token_account: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// User coin token account
    pub user_coin_token_account: Pubkey,
    /// User PC token account
    pub user_pc_token_account: Pubkey,
    /// User liquidity provider token account
    pub user_lp_token_account: Pubkey,
    /// User owner account
    pub user_owner: Pubkey,
}

/// Raydium AMM V4 initialize2 event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumAmmV4Initialize2Event {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Nonce value for initialization
    pub nonce: u8,
    /// Time when the pool opens
    pub open_time: u64,
    /// Initial PC token amount
    pub init_pc_amount: u64,
    /// Initial coin token amount
    pub init_coin_amount: u64,

    // Account keys
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// Liquidity provider mint address
    pub lp_mint_address: Pubkey,
    /// Coin mint address
    pub coin_mint_address: Pubkey,
    /// PC mint address
    pub pc_mint_address: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC token account
    pub pool_pc_token_account: Pubkey,
    /// Pool withdrawal queue account
    pub pool_withdraw_queue: Pubkey,
    /// AMM target orders account
    pub amm_target_orders: Pubkey,
    /// Pool liquidity provider token account
    pub pool_lp_token_account: Pubkey,
    /// Pool temporary LP token account
    pub pool_temp_lp_token_account: Pubkey,
    /// Serum program account
    pub serum_program: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// User wallet account
    pub user_wallet: Pubkey,
}

/// Raydium AMM V4 withdraw event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumAmmV4WithdrawEvent {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Amount of tokens being withdrawn
    pub amount: u64,

    // Account keys
    /// Token program account
    pub token_program: Pubkey,
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// AMM target orders account
    pub amm_target_orders: Pubkey,
    /// Liquidity provider mint address
    pub lp_mint_address: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC token account
    pub pool_pc_token_account: Pubkey,
    /// Pool withdrawal queue account
    pub pool_withdraw_queue: Pubkey,
    /// Pool temporary LP token account
    pub pool_temp_lp_token_account: Pubkey,
    /// Serum program account
    pub serum_program: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// Serum coin vault account
    pub serum_coin_vault_account: Pubkey,
    /// Serum PC vault account
    pub serum_pc_vault_account: Pubkey,
    /// Serum vault signer account
    pub serum_vault_signer: Pubkey,
    /// User liquidity provider token account
    pub user_lp_token_account: Pubkey,
    /// User coin token account
    pub user_coin_token_account: Pubkey,
    /// User PC token account
    pub user_pc_token_account: Pubkey,
    /// User owner account
    pub user_owner: Pubkey,
    /// Serum event queue account
    pub serum_event_queue: Pubkey,
    /// Serum bids account
    pub serum_bids: Pubkey,
    /// Serum asks account
    pub serum_asks: Pubkey,
}

/// Raydium AMM V4 withdraw PNL event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumAmmV4WithdrawPnlEvent {
    /// Event metadata
    pub metadata: EventMetadata,

    // Account keys
    /// Token program account
    pub token_program: Pubkey,
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM configuration account
    pub amm_config: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC token account
    pub pool_pc_token_account: Pubkey,
    /// Coin PNL (Profit and Loss) token account
    pub coin_pnl_token_account: Pubkey,
    /// PC PNL token account
    pub pc_pnl_token_account: Pubkey,
    /// PNL owner account
    pub pnl_owner_account: Pubkey,
    /// AMM target orders account
    pub amm_target_orders: Pubkey,
    /// Serum program account
    pub serum_program: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// Serum event queue account
    pub serum_event_queue: Pubkey,
    /// Serum coin vault account
    pub serum_coin_vault_account: Pubkey,
    /// Serum PC vault account
    pub serum_pc_vault_account: Pubkey,
    /// Serum vault signer account
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

    fn metadata(&self) -> &EventMetadata {
        // Convert our metadata to core metadata and cache it (placeholder implementation)
        use std::sync::OnceLock;
        static METADATA_CACHE: OnceLock<EventMetadata> = OnceLock::new();

        METADATA_CACHE.get_or_init(|| {
            EventMetadata::new(String::default(), EventKind::Swap, "raydium-amm-v4".to_string())
        })
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
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

    fn metadata(&self) -> &EventMetadata {
        use std::sync::OnceLock;
        static METADATA_CACHE: OnceLock<EventMetadata> = OnceLock::new();

        METADATA_CACHE.get_or_init(|| {
            EventMetadata::new(
                String::default(),
                EventKind::Liquidity,
                "raydium-amm-v4".to_string(),
            )
        })
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
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

    fn metadata(&self) -> &EventMetadata {
        use std::sync::OnceLock;
        static METADATA_CACHE: OnceLock<EventMetadata> = OnceLock::new();

        METADATA_CACHE.get_or_init(|| {
            EventMetadata::new(
                String::default(),
                EventKind::Contract,
                "raydium-amm-v4".to_string(),
            )
        })
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
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

    fn metadata(&self) -> &EventMetadata {
        use std::sync::OnceLock;
        static METADATA_CACHE: OnceLock<EventMetadata> = OnceLock::new();

        METADATA_CACHE.get_or_init(|| {
            EventMetadata::new(
                String::default(),
                EventKind::Liquidity,
                "raydium-amm-v4".to_string(),
            )
        })
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
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

    fn metadata(&self) -> &EventMetadata {
        use std::sync::OnceLock;
        static METADATA_CACHE: OnceLock<EventMetadata> = OnceLock::new();

        METADATA_CACHE.get_or_init(|| {
            EventMetadata::new(
                String::default(),
                EventKind::Transfer,
                "raydium-amm-v4".to_string(),
            )
        })
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
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
