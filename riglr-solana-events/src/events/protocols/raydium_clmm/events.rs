use riglr_events_core::{Event, EventKind, EventMetadata};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::any::Any;

/// Raydium CLMM swap event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmSwapEvent {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Amount of token0 in the swap
    pub amount0: u64,
    /// Amount of token1 in the swap
    pub amount1: u64,
    /// Square root of price multiplied by 2^64
    pub sqrt_price_x64: u128,
    /// Liquidity amount for the pool
    pub liquidity: u128,
    /// Current tick of the pool
    pub tick_current: i32,

    // Account keys
    /// Account that pays for the transaction
    pub payer: Pubkey,
    /// Pool state account
    pub pool_state: Pubkey,
    /// Input token account
    pub input_token_account: Pubkey,
    /// Output token account
    pub output_token_account: Pubkey,
    /// Input vault account
    pub input_vault: Pubkey,
    /// Output vault account
    pub output_vault: Pubkey,
    /// Token mint for token0
    pub token_mint0: Pubkey,
    /// Token mint for token1
    pub token_mint1: Pubkey,
}

/// Raydium CLMM swap V2 event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmSwapV2Event {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Amount of token0 in the swap
    pub amount0: u64,
    /// Amount of token1 in the swap
    pub amount1: u64,
    /// Square root of price multiplied by 2^64
    pub sqrt_price_x64: u128,
    /// Liquidity amount for the pool
    pub liquidity: u128,
    /// Current tick of the pool
    pub tick_current: i32,
    /// Whether base token is the input
    pub is_base_input: bool,

    // Account keys
    /// Account that pays for the transaction
    pub payer: Pubkey,
    /// Pool state account
    pub pool_state: Pubkey,
    /// Input token account
    pub input_token_account: Pubkey,
    /// Output token account
    pub output_token_account: Pubkey,
    /// Input vault account
    pub input_vault: Pubkey,
    /// Output vault account
    pub output_vault: Pubkey,
    /// Token mint for token0
    pub token_mint0: Pubkey,
    /// Token mint for token1
    pub token_mint1: Pubkey,
}

/// Raydium CLMM create pool event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmCreatePoolEvent {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Square root of price multiplied by 2^64
    pub sqrt_price_x64: u128,
    /// Current tick of the pool
    pub tick_current: i32,
    /// Observation index for the pool
    pub observation_index: u16,

    // Account keys
    /// Account that creates the pool
    pub pool_creator: Pubkey,
    /// Pool state account
    pub pool_state: Pubkey,
    /// Token mint for token0
    pub token_mint0: Pubkey,
    /// Token mint for token1
    pub token_mint1: Pubkey,
    /// Vault account for token0
    pub token_vault0: Pubkey,
    /// Vault account for token1
    pub token_vault1: Pubkey,
}

/// Raydium CLMM open position V2 event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmOpenPositionV2Event {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Lower tick index for the position
    pub tick_lower_index: i32,
    /// Upper tick index for the position
    pub tick_upper_index: i32,
    /// Start index for lower tick array
    pub tick_array_lower_start_index: i32,
    /// Start index for upper tick array
    pub tick_array_upper_start_index: i32,
    /// Liquidity amount for the position
    pub liquidity: u128,
    /// Maximum amount of token0 to use
    pub amount0_max: u64,
    /// Maximum amount of token1 to use
    pub amount1_max: u64,
    /// Whether to include metadata
    pub with_metadata: bool,
    /// Optional base flag
    pub base_flag: Option<bool>,

    // Account keys
    /// Account that pays for the transaction
    pub payer: Pubkey,
    /// Owner of the position NFT
    pub position_nft_owner: Pubkey,
    /// Mint account for the position NFT
    pub position_nft_mint: Pubkey,
    /// Token account for the position NFT
    pub position_nft_account: Pubkey,
    /// Metadata account for the NFT
    pub metadata_account: Pubkey,
    /// Pool state account
    pub pool_state: Pubkey,
}

/// Raydium CLMM close position event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmClosePositionEvent {
    /// Event metadata
    pub metadata: EventMetadata,

    // Account keys
    /// Owner of the NFT
    pub nft_owner: Pubkey,
    /// Mint account for the position NFT
    pub position_nft_mint: Pubkey,
    /// Token account for the position NFT
    pub position_nft_account: Pubkey,
    /// Personal position account
    pub personal_position: Pubkey,
}

/// Raydium CLMM increase liquidity V2 event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmIncreaseLiquidityV2Event {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Amount of liquidity to add
    pub liquidity: u128,
    /// Maximum amount of token0 to use
    pub amount0_max: u64,
    /// Maximum amount of token1 to use
    pub amount1_max: u64,
    /// Optional base flag
    pub base_flag: Option<bool>,

    // Account keys
    /// Owner of the NFT
    pub nft_owner: Pubkey,
    /// Token account for the position NFT
    pub position_nft_account: Pubkey,
    /// Pool state account
    pub pool_state: Pubkey,
}

/// Raydium CLMM decrease liquidity V2 event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmDecreaseLiquidityV2Event {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Amount of liquidity to remove
    pub liquidity: u128,
    /// Minimum amount of token0 to receive
    pub amount0_min: u64,
    /// Minimum amount of token1 to receive
    pub amount1_min: u64,

    // Account keys
    /// Owner of the NFT
    pub nft_owner: Pubkey,
    /// Token account for the position NFT
    pub position_nft_account: Pubkey,
    /// Pool state account
    pub pool_state: Pubkey,
}

/// Raydium CLMM open position with Token-22 NFT event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmOpenPositionWithToken22NftEvent {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Lower tick index for the position
    pub tick_lower_index: i32,
    /// Upper tick index for the position
    pub tick_upper_index: i32,
    /// Start index for lower tick array
    pub tick_array_lower_start_index: i32,
    /// Start index for upper tick array
    pub tick_array_upper_start_index: i32,
    /// Liquidity amount for the position
    pub liquidity: u128,
    /// Maximum amount of token0 to use
    pub amount0_max: u64,
    /// Maximum amount of token1 to use
    pub amount1_max: u64,
    /// Whether to include metadata
    pub with_metadata: bool,
    /// Optional base flag
    pub base_flag: Option<bool>,

    // Account keys
    /// Account that pays for the transaction
    pub payer: Pubkey,
    /// Owner of the position NFT
    pub position_nft_owner: Pubkey,
    /// Mint account for the position NFT
    pub position_nft_mint: Pubkey,
    /// Token account for the position NFT
    pub position_nft_account: Pubkey,
    /// Pool state account
    pub pool_state: Pubkey,
}

// Event trait implementations

impl Event for RaydiumClmmSwapEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        static SWAP_KIND: EventKind = EventKind::Swap;
        &SWAP_KIND
    }

    fn metadata(&self) -> &EventMetadata {
        use std::sync::OnceLock;
        static METADATA_CACHE: OnceLock<EventMetadata> = OnceLock::new();

        METADATA_CACHE.get_or_init(|| {
            EventMetadata::new(
                String::default(),
                EventKind::Swap,
                "raydium-clmm".to_string(),
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

impl Event for RaydiumClmmSwapV2Event {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        static SWAP_KIND: EventKind = EventKind::Swap;
        &SWAP_KIND
    }

    fn metadata(&self) -> &EventMetadata {
        use std::sync::OnceLock;
        static METADATA_CACHE: OnceLock<EventMetadata> = OnceLock::new();

        METADATA_CACHE.get_or_init(|| {
            EventMetadata::new(
                String::default(),
                EventKind::Swap,
                "raydium-clmm".to_string(),
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

impl Event for RaydiumClmmCreatePoolEvent {
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
                "raydium-clmm".to_string(),
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

impl Event for RaydiumClmmOpenPositionV2Event {
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
                "raydium-clmm".to_string(),
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

impl Event for RaydiumClmmClosePositionEvent {
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
                "raydium-clmm".to_string(),
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

impl Event for RaydiumClmmIncreaseLiquidityV2Event {
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
                "raydium-clmm".to_string(),
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

impl Event for RaydiumClmmDecreaseLiquidityV2Event {
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
                "raydium-clmm".to_string(),
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

impl Event for RaydiumClmmOpenPositionWithToken22NftEvent {
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
                "raydium-clmm".to_string(),
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
