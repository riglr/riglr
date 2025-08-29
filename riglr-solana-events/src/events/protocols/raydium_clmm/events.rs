use crate::solana_metadata::SolanaEventMetadata;
use riglr_events_core::EventMetadata as CoreEventMetadata;
use riglr_events_core::{Event, EventKind};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::any::Any;

/// Raydium CLMM swap event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumClmmSwapEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
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
    pub metadata: SolanaEventMetadata,
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
    pub metadata: SolanaEventMetadata,
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
    pub metadata: SolanaEventMetadata,
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
    pub metadata: SolanaEventMetadata,

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
    pub metadata: SolanaEventMetadata,
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
    pub metadata: SolanaEventMetadata,
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
    pub metadata: SolanaEventMetadata,
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
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static SWAP_KIND: EventKind = EventKind::Swap;
        &SWAP_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        &mut self.metadata.core
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
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static SWAP_KIND: EventKind = EventKind::Swap;
        &SWAP_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        &mut self.metadata.core
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
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static CONTRACT_KIND: EventKind = EventKind::Contract;
        &CONTRACT_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        &mut self.metadata.core
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
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
        &LIQUIDITY_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        &mut self.metadata.core
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
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
        &LIQUIDITY_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        &mut self.metadata.core
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
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
        &LIQUIDITY_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        &mut self.metadata.core
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
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
        &LIQUIDITY_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        &mut self.metadata.core
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
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
        &LIQUIDITY_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        &mut self.metadata.core
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solana_metadata::SolanaEventMetadata;
    use crate::types::{EventType, ProtocolType};
    use riglr_events_core::EventKind;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    // Helper function to create test EventMetadata
    fn create_test_metadata(id: &str) -> SolanaEventMetadata {
        let core = riglr_events_core::EventMetadata::new(
            id.to_string(),
            EventKind::Swap,
            "test-protocol".to_string(),
        );

        SolanaEventMetadata::new(
            "test-signature".to_string(),
            12345,
            EventType::Swap,
            ProtocolType::RaydiumClmm,
            "0".to_string(),
            1640995200000,
            core,
        )
    }

    // Helper function to create test Pubkey
    fn create_test_pubkey() -> Pubkey {
        Pubkey::from_str("11111111111111111111111111111112").unwrap()
    }

    // Tests for RaydiumClmmSwapEvent

    #[test]
    fn test_raydium_clmm_swap_event_default() {
        let event = RaydiumClmmSwapEvent::default();
        assert_eq!(event.amount0, 0);
        assert_eq!(event.amount1, 0);
        assert_eq!(event.sqrt_price_x64, 0);
        assert_eq!(event.liquidity, 0);
        assert_eq!(event.tick_current, 0);
        assert_eq!(event.payer, Pubkey::default());
        assert_eq!(event.pool_state, Pubkey::default());
    }

    #[test]
    fn test_raydium_clmm_swap_event_with_data() {
        let metadata = create_test_metadata("test-swap");
        let payer = create_test_pubkey();
        let event = RaydiumClmmSwapEvent {
            metadata,
            amount0: 1000,
            amount1: 2000,
            sqrt_price_x64: u128::MAX,
            liquidity: u128::MAX,
            tick_current: -887272,
            payer,
            pool_state: create_test_pubkey(),
            input_token_account: create_test_pubkey(),
            output_token_account: create_test_pubkey(),
            input_vault: create_test_pubkey(),
            output_vault: create_test_pubkey(),
            token_mint0: create_test_pubkey(),
            token_mint1: create_test_pubkey(),
        };

        assert_eq!(event.amount0, 1000);
        assert_eq!(event.amount1, 2000);
        assert_eq!(event.sqrt_price_x64, u128::MAX);
        assert_eq!(event.liquidity, u128::MAX);
        assert_eq!(event.tick_current, -887272);
        assert_eq!(event.payer, payer);
    }

    #[test]
    fn test_raydium_clmm_swap_event_trait_implementation() {
        let metadata = create_test_metadata("test-id");
        let mut event = RaydiumClmmSwapEvent {
            metadata,
            ..Default::default()
        };

        // Test id()
        assert_eq!(event.id(), "test-id");

        // Test kind()
        assert_eq!(*event.kind(), EventKind::Swap);

        // Test metadata()
        let meta = event.metadata();
        assert_eq!(meta.kind, EventKind::Swap);
        assert_eq!(meta.source, "raydium-clmm");

        // Test as_any()
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<RaydiumClmmSwapEvent>().is_some());

        // Test as_any_mut()
        let any_mut_ref = event.as_any_mut();
        assert!(any_mut_ref.downcast_mut::<RaydiumClmmSwapEvent>().is_some());

        // Test clone_boxed()
        let boxed = event.clone_boxed();
        assert_eq!(boxed.id(), "test-id");

        // Test to_json()
        let json_result = event.to_json();
        assert!(json_result.is_ok());
        let json_value = json_result.unwrap();
        assert!(json_value.is_object());
    }

    #[test]
    fn test_raydium_clmm_swap_event_metadata_mut_should_work() {
        let mut event = RaydiumClmmSwapEvent::default();
        let metadata = event.metadata_mut();
        metadata.id = "test-clmm-swap-id".to_string();
        assert_eq!(event.metadata().id, "test-clmm-swap-id");
    }

    // Tests for RaydiumClmmSwapV2Event

    #[test]
    fn test_raydium_clmm_swap_v2_event_default() {
        let event = RaydiumClmmSwapV2Event::default();
        assert_eq!(event.amount0, 0);
        assert_eq!(event.amount1, 0);
        assert_eq!(event.sqrt_price_x64, 0);
        assert_eq!(event.liquidity, 0);
        assert_eq!(event.tick_current, 0);
        assert!(!event.is_base_input);
    }

    #[test]
    fn test_raydium_clmm_swap_v2_event_with_data() {
        let metadata = create_test_metadata("test-swap-v2");
        let event = RaydiumClmmSwapV2Event {
            metadata,
            amount0: 500,
            amount1: 1500,
            sqrt_price_x64: 12345678901234567890,
            liquidity: 9876543210987654321,
            tick_current: 443636,
            is_base_input: true,
            payer: create_test_pubkey(),
            pool_state: create_test_pubkey(),
            input_token_account: create_test_pubkey(),
            output_token_account: create_test_pubkey(),
            input_vault: create_test_pubkey(),
            output_vault: create_test_pubkey(),
            token_mint0: create_test_pubkey(),
            token_mint1: create_test_pubkey(),
        };

        assert_eq!(event.amount0, 500);
        assert_eq!(event.amount1, 1500);
        assert!(event.is_base_input);
        assert_eq!(event.tick_current, 443636);
    }

    #[test]
    fn test_raydium_clmm_swap_v2_event_trait_implementation() {
        let metadata = create_test_metadata("test-v2-id");
        let event = RaydiumClmmSwapV2Event {
            metadata,
            ..Default::default()
        };

        assert_eq!(event.id(), "test-v2-id");
        assert_eq!(*event.kind(), EventKind::Swap);

        let meta = event.metadata();
        assert_eq!(meta.kind, EventKind::Swap);
        assert_eq!(meta.source, "raydium-clmm");

        let json_result = event.to_json();
        assert!(json_result.is_ok());
    }

    #[test]
    fn test_raydium_clmm_swap_v2_event_metadata_mut_should_work() {
        let mut event = RaydiumClmmSwapV2Event::default();
        let metadata = event.metadata_mut();
        metadata.id = "test-clmm-swap-v2-id".to_string();
        assert_eq!(event.metadata().id, "test-clmm-swap-v2-id");
    }

    // Tests for RaydiumClmmCreatePoolEvent

    #[test]
    fn test_raydium_clmm_create_pool_event_default() {
        let event = RaydiumClmmCreatePoolEvent::default();
        assert_eq!(event.sqrt_price_x64, 0);
        assert_eq!(event.tick_current, 0);
        assert_eq!(event.observation_index, 0);
        assert_eq!(event.pool_creator, Pubkey::default());
    }

    #[test]
    fn test_raydium_clmm_create_pool_event_with_data() {
        let metadata = create_test_metadata("test-create-pool");
        let event = RaydiumClmmCreatePoolEvent {
            metadata,
            sqrt_price_x64: 79228162514264337593543950336,
            tick_current: 0,
            observation_index: 1000,
            pool_creator: create_test_pubkey(),
            pool_state: create_test_pubkey(),
            token_mint0: create_test_pubkey(),
            token_mint1: create_test_pubkey(),
            token_vault0: create_test_pubkey(),
            token_vault1: create_test_pubkey(),
        };

        assert_eq!(event.sqrt_price_x64, 79228162514264337593543950336);
        assert_eq!(event.observation_index, 1000);
    }

    #[test]
    fn test_raydium_clmm_create_pool_event_trait_implementation() {
        let metadata = create_test_metadata("test-pool-id");
        let event = RaydiumClmmCreatePoolEvent {
            metadata,
            ..Default::default()
        };

        assert_eq!(event.id(), "test-pool-id");
        assert_eq!(*event.kind(), EventKind::Contract);

        let meta = event.metadata();
        assert_eq!(meta.kind, EventKind::Contract);
        assert_eq!(meta.source, "raydium-clmm");

        let json_result = event.to_json();
        assert!(json_result.is_ok());
    }

    #[test]
    fn test_raydium_clmm_create_pool_event_metadata_mut_should_work() {
        let mut event = RaydiumClmmCreatePoolEvent::default();
        let metadata = event.metadata_mut();
        metadata.id = "test-clmm-create-pool-id".to_string();
        assert_eq!(event.metadata().id, "test-clmm-create-pool-id");
    }

    // Tests for RaydiumClmmOpenPositionV2Event

    #[test]
    fn test_raydium_clmm_open_position_v2_event_default() {
        let event = RaydiumClmmOpenPositionV2Event::default();
        assert_eq!(event.tick_lower_index, 0);
        assert_eq!(event.tick_upper_index, 0);
        assert_eq!(event.liquidity, 0);
        assert_eq!(event.amount0_max, 0);
        assert_eq!(event.amount1_max, 0);
        assert!(!event.with_metadata);
        assert_eq!(event.base_flag, None);
    }

    #[test]
    fn test_raydium_clmm_open_position_v2_event_with_data() {
        let metadata = create_test_metadata("test-open-position");
        let event = RaydiumClmmOpenPositionV2Event {
            metadata,
            tick_lower_index: -443636,
            tick_upper_index: 443636,
            tick_array_lower_start_index: -443636,
            tick_array_upper_start_index: 443636,
            liquidity: 1000000000000000000,
            amount0_max: 1000000,
            amount1_max: 2000000,
            with_metadata: true,
            base_flag: Some(true),
            payer: create_test_pubkey(),
            position_nft_owner: create_test_pubkey(),
            position_nft_mint: create_test_pubkey(),
            position_nft_account: create_test_pubkey(),
            metadata_account: create_test_pubkey(),
            pool_state: create_test_pubkey(),
        };

        assert_eq!(event.tick_lower_index, -443636);
        assert_eq!(event.tick_upper_index, 443636);
        assert!(event.with_metadata);
        assert_eq!(event.base_flag, Some(true));
    }

    #[test]
    fn test_raydium_clmm_open_position_v2_event_base_flag_none() {
        let event = RaydiumClmmOpenPositionV2Event {
            base_flag: None,
            ..Default::default()
        };
        assert_eq!(event.base_flag, None);
    }

    #[test]
    fn test_raydium_clmm_open_position_v2_event_base_flag_false() {
        let event = RaydiumClmmOpenPositionV2Event {
            base_flag: Some(false),
            ..Default::default()
        };
        assert_eq!(event.base_flag, Some(false));
    }

    #[test]
    fn test_raydium_clmm_open_position_v2_event_trait_implementation() {
        let metadata = create_test_metadata("test-position-id");
        let event = RaydiumClmmOpenPositionV2Event {
            metadata,
            ..Default::default()
        };

        assert_eq!(event.id(), "test-position-id");
        assert_eq!(*event.kind(), EventKind::Liquidity);

        let meta = event.metadata();
        assert_eq!(meta.kind, EventKind::Liquidity);
        assert_eq!(meta.source, "raydium-clmm");

        let json_result = event.to_json();
        assert!(json_result.is_ok());
    }

    #[test]
    fn test_raydium_clmm_open_position_v2_event_metadata_mut_should_work() {
        let mut event = RaydiumClmmOpenPositionV2Event::default();
        let metadata = event.metadata_mut();
        metadata.id = "test-clmm-open-position-v2-id".to_string();
        assert_eq!(event.metadata().id, "test-clmm-open-position-v2-id");
    }

    // Tests for RaydiumClmmClosePositionEvent

    #[test]
    fn test_raydium_clmm_close_position_event_default() {
        let event = RaydiumClmmClosePositionEvent::default();
        assert_eq!(event.nft_owner, Pubkey::default());
        assert_eq!(event.position_nft_mint, Pubkey::default());
        assert_eq!(event.position_nft_account, Pubkey::default());
        assert_eq!(event.personal_position, Pubkey::default());
    }

    #[test]
    fn test_raydium_clmm_close_position_event_with_data() {
        let metadata = create_test_metadata("test-close-position");
        let event = RaydiumClmmClosePositionEvent {
            metadata,
            nft_owner: create_test_pubkey(),
            position_nft_mint: create_test_pubkey(),
            position_nft_account: create_test_pubkey(),
            personal_position: create_test_pubkey(),
        };

        assert_eq!(event.nft_owner, create_test_pubkey());
    }

    #[test]
    fn test_raydium_clmm_close_position_event_trait_implementation() {
        let metadata = create_test_metadata("test-close-id");
        let event = RaydiumClmmClosePositionEvent {
            metadata,
            ..Default::default()
        };

        assert_eq!(event.id(), "test-close-id");
        assert_eq!(*event.kind(), EventKind::Liquidity);

        let meta = event.metadata();
        assert_eq!(meta.kind, EventKind::Liquidity);
        assert_eq!(meta.source, "raydium-clmm");

        let json_result = event.to_json();
        assert!(json_result.is_ok());
    }

    #[test]
    fn test_raydium_clmm_close_position_event_metadata_mut_should_work() {
        let mut event = RaydiumClmmClosePositionEvent::default();
        let metadata = event.metadata_mut();
        metadata.id = "test-clmm-close-position-id".to_string();
        assert_eq!(event.metadata().id, "test-clmm-close-position-id");
    }

    // Tests for RaydiumClmmIncreaseLiquidityV2Event

    #[test]
    fn test_raydium_clmm_increase_liquidity_v2_event_default() {
        let event = RaydiumClmmIncreaseLiquidityV2Event::default();
        assert_eq!(event.liquidity, 0);
        assert_eq!(event.amount0_max, 0);
        assert_eq!(event.amount1_max, 0);
        assert_eq!(event.base_flag, None);
    }

    #[test]
    fn test_raydium_clmm_increase_liquidity_v2_event_with_data() {
        let metadata = create_test_metadata("test-increase-liquidity");
        let event = RaydiumClmmIncreaseLiquidityV2Event {
            metadata,
            liquidity: 5000000000000000000,
            amount0_max: 3000000,
            amount1_max: 4000000,
            base_flag: Some(false),
            nft_owner: create_test_pubkey(),
            position_nft_account: create_test_pubkey(),
            pool_state: create_test_pubkey(),
        };

        assert_eq!(event.liquidity, 5000000000000000000);
        assert_eq!(event.base_flag, Some(false));
    }

    #[test]
    fn test_raydium_clmm_increase_liquidity_v2_event_trait_implementation() {
        let metadata = create_test_metadata("test-increase-id");
        let event = RaydiumClmmIncreaseLiquidityV2Event {
            metadata,
            ..Default::default()
        };

        assert_eq!(event.id(), "test-increase-id");
        assert_eq!(*event.kind(), EventKind::Liquidity);

        let meta = event.metadata();
        assert_eq!(meta.kind, EventKind::Liquidity);
        assert_eq!(meta.source, "raydium-clmm");

        let json_result = event.to_json();
        assert!(json_result.is_ok());
    }

    #[test]
    fn test_raydium_clmm_increase_liquidity_v2_event_metadata_mut_should_work() {
        let mut event = RaydiumClmmIncreaseLiquidityV2Event::default();
        let metadata = event.metadata_mut();
        metadata.id = "test-clmm-increase-liquidity-v2-id".to_string();
        assert_eq!(event.metadata().id, "test-clmm-increase-liquidity-v2-id");
    }

    // Tests for RaydiumClmmDecreaseLiquidityV2Event

    #[test]
    fn test_raydium_clmm_decrease_liquidity_v2_event_default() {
        let event = RaydiumClmmDecreaseLiquidityV2Event::default();
        assert_eq!(event.liquidity, 0);
        assert_eq!(event.amount0_min, 0);
        assert_eq!(event.amount1_min, 0);
    }

    #[test]
    fn test_raydium_clmm_decrease_liquidity_v2_event_with_data() {
        let metadata = create_test_metadata("test-decrease-liquidity");
        let event = RaydiumClmmDecreaseLiquidityV2Event {
            metadata,
            liquidity: 2000000000000000000,
            amount0_min: 100000,
            amount1_min: 200000,
            nft_owner: create_test_pubkey(),
            position_nft_account: create_test_pubkey(),
            pool_state: create_test_pubkey(),
        };

        assert_eq!(event.liquidity, 2000000000000000000);
        assert_eq!(event.amount0_min, 100000);
        assert_eq!(event.amount1_min, 200000);
    }

    #[test]
    fn test_raydium_clmm_decrease_liquidity_v2_event_trait_implementation() {
        let metadata = create_test_metadata("test-decrease-id");
        let event = RaydiumClmmDecreaseLiquidityV2Event {
            metadata,
            ..Default::default()
        };

        assert_eq!(event.id(), "test-decrease-id");
        assert_eq!(*event.kind(), EventKind::Liquidity);

        let meta = event.metadata();
        assert_eq!(meta.kind, EventKind::Liquidity);
        assert_eq!(meta.source, "raydium-clmm");

        let json_result = event.to_json();
        assert!(json_result.is_ok());
    }

    #[test]
    fn test_raydium_clmm_decrease_liquidity_v2_event_metadata_mut_should_work() {
        let mut event = RaydiumClmmDecreaseLiquidityV2Event::default();
        let metadata = event.metadata_mut();
        metadata.id = "test-clmm-decrease-liquidity-v2-id".to_string();
        assert_eq!(event.metadata().id, "test-clmm-decrease-liquidity-v2-id");
    }

    // Tests for RaydiumClmmOpenPositionWithToken22NftEvent

    #[test]
    fn test_raydium_clmm_open_position_with_token22_nft_event_default() {
        let event = RaydiumClmmOpenPositionWithToken22NftEvent::default();
        assert_eq!(event.tick_lower_index, 0);
        assert_eq!(event.tick_upper_index, 0);
        assert_eq!(event.liquidity, 0);
        assert_eq!(event.amount0_max, 0);
        assert_eq!(event.amount1_max, 0);
        assert!(!event.with_metadata);
        assert_eq!(event.base_flag, None);
    }

    #[test]
    fn test_raydium_clmm_open_position_with_token22_nft_event_with_data() {
        let metadata = create_test_metadata("test-token22-position");
        let event = RaydiumClmmOpenPositionWithToken22NftEvent {
            metadata,
            tick_lower_index: -887272,
            tick_upper_index: 887272,
            tick_array_lower_start_index: -887272,
            tick_array_upper_start_index: 887272,
            liquidity: u128::MAX,
            amount0_max: u64::MAX,
            amount1_max: u64::MAX,
            with_metadata: true,
            base_flag: Some(true),
            payer: create_test_pubkey(),
            position_nft_owner: create_test_pubkey(),
            position_nft_mint: create_test_pubkey(),
            position_nft_account: create_test_pubkey(),
            pool_state: create_test_pubkey(),
        };

        assert_eq!(event.tick_lower_index, -887272);
        assert_eq!(event.tick_upper_index, 887272);
        assert_eq!(event.liquidity, u128::MAX);
        assert_eq!(event.amount0_max, u64::MAX);
        assert_eq!(event.amount1_max, u64::MAX);
        assert!(event.with_metadata);
        assert_eq!(event.base_flag, Some(true));
    }

    #[test]
    fn test_raydium_clmm_open_position_with_token22_nft_event_trait_implementation() {
        let metadata = create_test_metadata("test-token22-id");
        let event = RaydiumClmmOpenPositionWithToken22NftEvent {
            metadata,
            ..Default::default()
        };

        assert_eq!(event.id(), "test-token22-id");
        assert_eq!(*event.kind(), EventKind::Liquidity);

        let meta = event.metadata();
        assert_eq!(meta.kind, EventKind::Liquidity);
        assert_eq!(meta.source, "raydium-clmm");

        let json_result = event.to_json();
        assert!(json_result.is_ok());
    }

    #[test]
    fn test_raydium_clmm_open_position_with_token22_nft_event_metadata_mut_should_work() {
        let mut event = RaydiumClmmOpenPositionWithToken22NftEvent::default();
        let metadata = event.metadata_mut();
        metadata.id = "test-clmm-open-position-token22-nft-id".to_string();
        assert_eq!(
            event.metadata().id,
            "test-clmm-open-position-token22-nft-id"
        );
    }

    // Test edge cases and boundary conditions

    #[test]
    fn test_extreme_tick_values() {
        let event = RaydiumClmmSwapEvent {
            tick_current: i32::MIN,
            ..Default::default()
        };
        assert_eq!(event.tick_current, i32::MIN);

        let event = RaydiumClmmSwapEvent {
            tick_current: i32::MAX,
            ..Default::default()
        };
        assert_eq!(event.tick_current, i32::MAX);
    }

    #[test]
    fn test_extreme_amounts() {
        let event = RaydiumClmmSwapEvent {
            amount0: u64::MAX,
            amount1: u64::MAX,
            ..Default::default()
        };
        assert_eq!(event.amount0, u64::MAX);
        assert_eq!(event.amount1, u64::MAX);
    }

    #[test]
    fn test_extreme_liquidity_values() {
        let event = RaydiumClmmSwapEvent {
            liquidity: u128::MAX,
            sqrt_price_x64: u128::MAX,
            ..Default::default()
        };
        assert_eq!(event.liquidity, u128::MAX);
        assert_eq!(event.sqrt_price_x64, u128::MAX);
    }

    #[test]
    fn test_clone_functionality() {
        let original = RaydiumClmmSwapEvent {
            amount0: 1000,
            amount1: 2000,
            tick_current: 100,
            ..Default::default()
        };

        let cloned = original.clone();
        assert_eq!(original.amount0, cloned.amount0);
        assert_eq!(original.amount1, cloned.amount1);
        assert_eq!(original.tick_current, cloned.tick_current);
    }

    #[test]
    fn test_serialization_deserialization() {
        let original = RaydiumClmmSwapV2Event {
            amount0: 1500,
            amount1: 2500,
            is_base_input: true,
            ..Default::default()
        };

        // Test serialization
        let serialized = serde_json::to_string(&original).unwrap();
        assert!(serialized.contains("1500"));
        assert!(serialized.contains("2500"));
        assert!(serialized.contains("true"));

        // Test deserialization
        let deserialized: RaydiumClmmSwapV2Event = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original.amount0, deserialized.amount0);
        assert_eq!(original.amount1, deserialized.amount1);
        assert_eq!(original.is_base_input, deserialized.is_base_input);
    }

    #[test]
    fn test_debug_formatting() {
        let event = RaydiumClmmCreatePoolEvent {
            observation_index: 42,
            ..Default::default()
        };

        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("RaydiumClmmCreatePoolEvent"));
        assert!(debug_str.contains("42"));
    }
}
#[cfg(test)]
mod clmm_metadata_fix_verification {
    use super::*;
    use riglr_events_core::Event;

    #[test]
    fn test_clmm_metadata_isolation() {
        let mut event1 = RaydiumClmmSwapEvent::default();
        let mut event2 = RaydiumClmmSwapEvent::default();

        event1.metadata.id = "clmm1".to_string();
        event2.metadata.id = "clmm2".to_string();

        assert_eq!(event1.id(), "clmm1");
        assert_eq!(event2.id(), "clmm2");

        event1.metadata.id = "modified".to_string();
        assert_eq!(event1.id(), "modified");
        assert_eq!(event2.id(), "clmm2");
    }

    #[test]
    fn test_clmm_metadata_mut_works() {
        let mut event = RaydiumClmmSwapEvent::default();
        let metadata_mut = event.metadata_mut();
        metadata_mut.id = "test_clmm".to_string();
        assert_eq!(event.id(), "test_clmm");
    }
}
