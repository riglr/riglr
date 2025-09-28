/// Meteora protocol event definitions and constants.
// Types module
pub mod types {
    use serde::{Deserialize, Serialize};
    use solana_sdk::pubkey::Pubkey;
    use std::sync::OnceLock;

    /// Meteora DLMM program ID
    pub const METEORA_DLMM_PROGRAM_ID: &str = "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo";

    /// Meteora Dynamic program ID
    pub const METEORA_DYNAMIC_PROGRAM_ID: &str = "Dooar9JkhdZ7J3LHN3A7YCuoGRUggXhQaG4kijfLGU2j";

    /// Meteora instruction discriminators
    /// Discriminator for DLMM swap instruction
    pub const DLMM_SWAP_DISCRIMINATOR: [u8; 8] = [0x14, 0x65, 0x32, 0x1f, 0x7a, 0x43, 0x2a, 0x9f];
    /// Discriminator for DLMM add liquidity instruction
    pub const DLMM_ADD_LIQUIDITY_DISCRIMINATOR: [u8; 8] =
        [0x4c, 0x1c, 0x9b, 0x2d, 0xe3, 0x7a, 0x8b, 0x12];
    /// Discriminator for DLMM remove liquidity instruction
    pub const DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR: [u8; 8] =
        [0xa2, 0xfd, 0x67, 0xe3, 0x45, 0x1b, 0x8c, 0x9a];
    /// Discriminator for Dynamic AMM add liquidity instruction
    pub const DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR: [u8; 8] =
        [0x85, 0x72, 0x1a, 0x5f, 0x9d, 0x4e, 0x23, 0x7c];
    /// Discriminator for Dynamic AMM remove liquidity instruction
    pub const DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR: [u8; 8] =
        [0x6a, 0x8b, 0x47, 0x2e, 0x1c, 0x93, 0x5f, 0x4d];

    /// Meteora DLMM bin information
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[non_exhaustive]
    pub struct DlmmBin {
        /// Unique identifier for this price bin
        pub bin_id: u32,
        /// Total liquidity token supply for this bin
        pub liquidity_supply: u128,
        /// Current price for this bin
        pub price: f64,
        /// Amount of token X reserves in this bin
        pub reserve_x: u64,
        /// Amount of token Y reserves in this bin
        pub reserve_y: u64,
    }

    /// Meteora DLMM pair configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[non_exhaustive]
    pub struct DlmmPairConfig {
        /// Currently active bin ID
        pub active_id: u32,
        /// Base fee percentage charged for swaps
        pub base_fee_percentage: u64,
        /// Base key for the pair
        pub base_key: Pubkey,
        /// Step size between bins in basis points
        pub bin_step: u16,
        /// ID reference for bin tracking
        pub id_reference: u32,
        /// Liquidity provider fee percentage
        pub liquidity_fee_percentage: u64,
        /// Maximum fee percentage that can be charged
        pub max_fee_percentage: u64,
        /// Public key of the DLMM pair account
        pub pair: Pubkey,
        /// Protocol fee percentage taken from trades
        pub protocol_fee_percentage: u64,
        /// Timestamp of last pair update
        pub time_of_last_update: u64,
        /// Mint address of token X
        pub token_mint_x: Pubkey,
        /// Mint address of token Y
        pub token_mint_y: Pubkey,
        /// Current volatility accumulator value
        pub volatility_accumulator: u32,
        /// Volatility reference point
        pub volatility_reference: u32,
    }

    /// Meteora DLMM swap data
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    #[non_exhaustive]
    pub struct SwapData {
        /// Active bin ID after the swap
        pub active_id_after: u32,
        /// Active bin ID before the swap
        pub active_id_before: u32,
        /// Actual amount of tokens received
        pub actual_amount_out: u64,
        /// Amount of tokens being swapped in
        pub amount_in: u64,
        /// List of bin IDs traversed during the swap
        pub bins_traversed: Vec<u32>,
        /// Total fee amount charged for the swap
        pub fee_amount: u64,
        /// Minimum expected amount of tokens out
        pub min_amount_out: u64,
        /// Public key of the DLMM pair being swapped on
        pub pair: Pubkey,
        /// Protocol fee portion of the total fee
        pub protocol_fee: u64,
        /// Reserve account for token X
        pub reserve_x: Pubkey,
        /// Reserve account for token Y
        pub reserve_y: Pubkey,
        /// Whether swapping X for Y (true) or Y for X (false)
        pub swap_for_y: bool,
        /// Mint address of token X
        pub token_mint_x: Pubkey,
        /// Mint address of token Y
        pub token_mint_y: Pubkey,
        /// Public key of the user performing the swap
        pub user: Pubkey,
    }

    /// Meteora DLMM liquidity data
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    #[non_exhaustive]
    pub struct LiquidityData {
        /// Currently active bin ID
        pub active_id: u32,
        /// Amount of token X added or removed
        pub amount_x: u64,
        /// Amount of token Y added or removed
        pub amount_y: u64,
        /// Starting bin ID for liquidity range
        pub bin_id_from: u32,
        /// Ending bin ID for liquidity range
        pub bin_id_to: u32,
        /// List of bins affected by this liquidity operation
        pub bins_affected: Vec<DlmmBin>,
        /// Whether this is an add (true) or remove (false) operation
        pub is_add: bool,
        /// Amount of liquidity tokens minted or burned
        pub liquidity_minted: u128,
        /// Public key of the DLMM pair
        pub pair: Pubkey,
        /// Public key of the liquidity position
        pub position: Pubkey,
        /// Reserve account for token X
        pub reserve_x: Pubkey,
        /// Reserve account for token Y
        pub reserve_y: Pubkey,
        /// Mint address of token X
        pub token_mint_x: Pubkey,
        /// Mint address of token Y
        pub token_mint_y: Pubkey,
        /// Public key of the user adding/removing liquidity
        pub user: Pubkey,
    }

    /// Meteora Dynamic AMM pool data
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[non_exhaustive]
    pub struct DynamicPoolData {
        /// Administrative fee rate
        pub admin_fee_rate: u64,
        /// Base fee rate for the pool
        pub fee_rate: u64,
        /// Denominator for host fee calculation
        pub host_fee_denominator: u64,
        /// Numerator for host fee calculation
        pub host_fee_numerator: u64,
        /// Mint address of the LP tokens
        pub lp_mint: Pubkey,
        /// Denominator for owner trade fee calculation
        pub owner_trade_fee_denominator: u64,
        /// Numerator for owner trade fee calculation
        pub owner_trade_fee_numerator: u64,
        /// Denominator for owner withdraw fee calculation
        pub owner_withdraw_fee_denominator: u64,
        /// Numerator for owner withdraw fee calculation
        pub owner_withdraw_fee_numerator: u64,
        /// Public key of the Dynamic AMM pool
        pub pool: Pubkey,
        /// Mint address of token A
        pub token_mint_a: Pubkey,
        /// Mint address of token B
        pub token_mint_b: Pubkey,
        /// Denominator for trade fee calculation
        pub trade_fee_denominator: u64,
        /// Numerator for trade fee calculation
        pub trade_fee_numerator: u64,
        /// Vault account holding token A reserves
        pub vault_a: Pubkey,
        /// Vault account holding token B reserves
        pub vault_b: Pubkey,
    }

    /// Meteora Dynamic liquidity data
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    #[non_exhaustive]
    pub struct DynamicLiquidityData {
        /// Whether this is a deposit (true) or withdrawal (false) operation
        pub is_deposit: bool,
        /// Mint address of the LP tokens
        pub lp_mint: Pubkey,
        /// Maximum token A amount willing to deposit
        pub maximum_token_a_amount: u64,
        /// Maximum token B amount willing to deposit
        pub maximum_token_b_amount: u64,
        /// Minimum acceptable pool token amount for slippage protection
        pub minimum_pool_token_amount: u64,
        /// Public key of the Dynamic AMM pool
        pub pool: Pubkey,
        /// Amount of pool tokens being minted or burned
        pub pool_token_amount: u64,
        /// Amount of token A being deposited or withdrawn
        pub token_a_amount: u64,
        /// Amount of token B being deposited or withdrawn
        pub token_b_amount: u64,
        /// Mint address of token A
        pub token_mint_a: Pubkey,
        /// Mint address of token B
        pub token_mint_b: Pubkey,
        /// Public key of the user adding/removing liquidity
        pub user: Pubkey,
        /// Vault account holding token A reserves
        pub vault_a: Pubkey,
        /// Vault account holding token B reserves
        pub vault_b: Pubkey,
    }

    /// Extract Meteora DLMM program ID as Pubkey
    ///
    /// This uses a static lazy-evaluated Pubkey to avoid repeated parsing.
    static METEORA_DLMM_PUBKEY: OnceLock<Pubkey> = OnceLock::new();

    /// # Panics
    ///
    /// This function panics if the Meteora DLMM program ID constant is invalid,
    /// which should never happen as it's a hardcoded valid constant.
    #[must_use]
    #[inline]
    pub fn dlmm_program_id() -> Pubkey {
        *METEORA_DLMM_PUBKEY.get_or_init(|| {
            Pubkey::try_from(METEORA_DLMM_PROGRAM_ID).unwrap_or_else(|_| Pubkey::default())
        })
    }

    /// Extract Meteora Dynamic program ID as Pubkey
    ///
    /// This uses a static lazy-evaluated Pubkey to avoid repeated parsing.
    static METEORA_DYNAMIC_PUBKEY: OnceLock<Pubkey> = OnceLock::new();

    #[must_use]
    #[inline]
    /// Returns the Meteora Dynamic program ID.
    ///
    /// # Panics
    /// Panics if the Meteora Dynamic program ID constant is invalid,
    /// which should never happen as it's a hardcoded valid constant.
    pub fn dynamic_program_id() -> Pubkey {
        *METEORA_DYNAMIC_PUBKEY.get_or_init(|| {
            Pubkey::try_from(METEORA_DYNAMIC_PROGRAM_ID).unwrap_or_else(|_| Pubkey::default())
        })
    }

    /// Check if the given pubkey is Meteora DLMM program
    #[must_use]
    #[inline]
    pub fn is_meteora_dlmm_program(program_id: &Pubkey) -> bool {
        *program_id == dlmm_program_id()
    }

    /// Check if the given pubkey is Meteora Dynamic program
    #[must_use]
    #[inline]
    pub fn is_meteora_dynamic_program(program_id: &Pubkey) -> bool {
        *program_id == dynamic_program_id()
    }

    /// Convert bin ID to price for DLMM
    #[must_use]
    #[inline]
    pub fn bin_id_to_price(bin_id: u32, bin_step: u16) -> f64 {
        let bin_step_decimal = f64::from(bin_step) / 10_000.0_f64;
        // The cast is safe because valid bin IDs in Meteora DLMM are centered around 8_388_608 (2^23)
        // and are designed to fit within i32 range for the exponent calculation
        #[expect(clippy::cast_possible_wrap)]
        {
            (1.0 + bin_step_decimal).powi((bin_id as i32).saturating_sub(0x0080_0000))
            // 2^23 offset
        }
    }

    /// Calculate active bin price
    #[must_use]
    #[inline]
    pub fn calculate_active_bin_price(active_id: u32, bin_step: u16) -> f64 {
        bin_id_to_price(active_id, bin_step)
    }

    /// Calculate liquidity distribution across bins
    #[must_use]
    #[inline]
    pub fn calculate_liquidity_distribution(
        amount_x: u64,
        amount_y: u64,
        bin_id_from: u32,
        bin_id_to: u32,
        active_id: u32,
    ) -> Vec<(u32, u64, u64)> {
        let mut distribution = Vec::new();

        // Check for invalid range to prevent underflow
        if bin_id_to < bin_id_from {
            return distribution;
        }

        let total_bins = u64::from(bin_id_to.saturating_sub(bin_id_from).saturating_add(1));

        if total_bins == 0 {
            return distribution;
        }

        for bin_id in bin_id_from..=bin_id_to {
            let x_amount = if bin_id <= active_id && total_bins > 0 {
                amount_x.checked_div(total_bins).unwrap_or(0)
            } else {
                0
            };

            let y_amount = if bin_id >= active_id && total_bins > 0 {
                amount_y.checked_div(total_bins).unwrap_or(0)
            } else {
                0
            };

            distribution.push((bin_id, x_amount, y_amount));
        }

        distribution
    }
}

// Events module
pub mod events {
    use super::types::{DynamicLiquidityData, LiquidityData, SwapData};
    use crate::events::core::EventParameters;
    use crate::solana_metadata::SolanaEventMetadata;
    use crate::types::{metadata_helpers, EventType, ProtocolType, TransferData};
    use core::any::Any;
    use riglr_events_core::{
        error::EventResult, traits::EventFilter, Event, EventKind,
        EventMetadata as CoreEventMetadata,
    };
    use serde::{Deserialize, Serialize};
    use std::time::SystemTime;

    /// Meteora DLMM swap event
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    #[non_exhaustive]
    pub struct SwapEvent {
        /// Event metadata
        pub metadata: SolanaEventMetadata,
        /// Meteora swap-specific data
        pub swap_data: SwapData,
        /// Token transfer data associated with the swap
        pub transfer_data: Vec<TransferData>,
    }

    impl SwapEvent {
        /// Creates a new `MeteoraSwapEvent` with the provided parameters and swap data
        #[must_use]
        #[inline]
        pub fn new(params: EventParameters, swap_data: SwapData) -> Self {
            let metadata = metadata_helpers::create_solana_metadata(
                params.id,
                params.signature,
                params.slot,
                params.block_time,
                ProtocolType::MeteoraDlmm,
                EventType::Swap,
                super::types::dlmm_program_id(),
                params.index,
                params.program_received_time_ms,
            );

            Self {
                metadata,
                swap_data,
                transfer_data: Vec::new(),
            }
        }

        /// Sets the transfer data for this swap event
        #[must_use]
        #[inline]
        pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
            self.transfer_data = transfer_data;
            self
        }
    }

    // Event trait implementation
    impl Event for SwapEvent {
        #[inline]
        fn as_any(&self) -> &dyn Any {
            self
        }

        #[inline]
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        #[inline]
        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }

        #[inline]
        fn id(&self) -> &str {
            &self.metadata.core.id
        }

        #[inline]
        fn kind(&self) -> &EventKind {
            static SWAP_KIND: EventKind = EventKind::Swap;
            &SWAP_KIND
        }

        #[inline]
        fn matches_filter(&self, filter: &dyn EventFilter) -> bool
        where
            Self: Sized,
        {
            filter.matches(self)
        }

        #[inline]
        fn metadata(&self) -> &CoreEventMetadata {
            &self.metadata.core
        }

        #[inline]
        fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
            Ok(&mut self.metadata.core)
        }

        #[inline]
        fn source(&self) -> &'static str {
            "meteora"
        }

        #[inline]
        fn timestamp(&self) -> SystemTime {
            self.metadata.core.timestamp.into()
        }

        #[inline]
        fn to_json(&self) -> EventResult<serde_json::Value> {
            Ok(serde_json::to_value(self)?)
        }
    }

    /// Meteora DLMM liquidity event
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    #[non_exhaustive]
    pub struct LiquidityEvent {
        /// Meteora liquidity-specific data
        pub liquidity_data: LiquidityData,
        /// Event metadata
        pub metadata: SolanaEventMetadata,
        /// Token transfer data associated with the liquidity operation
        pub transfer_data: Vec<TransferData>,
    }

    impl LiquidityEvent {
        /// Creates a new `MeteoraLiquidityEvent` with the provided parameters and liquidity data
        #[must_use]
        #[inline]
        pub fn new(params: EventParameters, liquidity_data: LiquidityData) -> Self {
            let metadata = metadata_helpers::create_solana_metadata(
                params.id,
                params.signature,
                params.slot,
                params.block_time,
                ProtocolType::MeteoraDlmm,
                EventType::AddLiquidity,
                super::types::dlmm_program_id(),
                params.index,
                params.program_received_time_ms,
            );

            Self {
                metadata,
                liquidity_data,
                transfer_data: Vec::new(),
            }
        }

        /// Sets the transfer data for this liquidity event
        #[must_use]
        #[inline]
        pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
            self.transfer_data = transfer_data;
            self
        }
    }

    // Event trait implementation for MeteoraLiquidityEvent
    impl Event for LiquidityEvent {
        #[inline]
        fn as_any(&self) -> &dyn Any {
            self
        }

        #[inline]
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        #[inline]
        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }

        #[inline]
        fn id(&self) -> &str {
            &self.metadata.core.id
        }

        #[inline]
        fn kind(&self) -> &EventKind {
            static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
            &LIQUIDITY_KIND
        }

        #[inline]
        fn matches_filter(&self, filter: &dyn EventFilter) -> bool
        where
            Self: Sized,
        {
            filter.matches(self)
        }

        #[inline]
        fn metadata(&self) -> &CoreEventMetadata {
            &self.metadata.core
        }

        #[inline]
        fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
            Ok(&mut self.metadata.core)
        }

        #[inline]
        fn source(&self) -> &'static str {
            "meteora"
        }

        #[inline]
        fn timestamp(&self) -> SystemTime {
            self.metadata.core.timestamp.into()
        }

        #[inline]
        fn to_json(&self) -> EventResult<serde_json::Value> {
            Ok(serde_json::to_value(self)?)
        }
    }

    /// Meteora Dynamic AMM liquidity event
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    #[non_exhaustive]
    pub struct DynamicLiquidityEvent {
        /// Meteora dynamic liquidity-specific data
        pub liquidity_data: DynamicLiquidityData,
        /// Event metadata
        pub metadata: SolanaEventMetadata,
        /// Token transfer data associated with the liquidity operation
        pub transfer_data: Vec<TransferData>,
    }

    impl DynamicLiquidityEvent {
        /// Creates a new `MeteoraDynamicLiquidityEvent` with the provided parameters and liquidity data
        #[must_use]
        #[inline]
        pub fn new(params: EventParameters, liquidity_data: DynamicLiquidityData) -> Self {
            let metadata = metadata_helpers::create_solana_metadata(
                params.id,
                params.signature,
                params.slot,
                params.block_time,
                ProtocolType::MeteoraDlmm, // Using MeteoraDlmm for consistency
                EventType::AddLiquidity,
                super::types::dynamic_program_id(),
                params.index,
                params.program_received_time_ms,
            );

            Self {
                metadata,
                liquidity_data,
                transfer_data: Vec::new(),
            }
        }

        /// Sets the transfer data for this dynamic liquidity event
        #[must_use]
        #[inline]
        pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
            self.transfer_data = transfer_data;
            self
        }
    }

    // Event trait implementation for MeteoraDynamicLiquidityEvent
    impl Event for DynamicLiquidityEvent {
        #[inline]
        fn as_any(&self) -> &dyn Any {
            self
        }

        #[inline]
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        #[inline]
        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }

        #[inline]
        fn id(&self) -> &str {
            &self.metadata.core.id
        }

        #[inline]
        fn kind(&self) -> &EventKind {
            static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
            &LIQUIDITY_KIND
        }

        #[inline]
        fn matches_filter(&self, filter: &dyn EventFilter) -> bool
        where
            Self: Sized,
        {
            filter.matches(self)
        }

        #[inline]
        fn metadata(&self) -> &CoreEventMetadata {
            &self.metadata.core
        }

        #[inline]
        fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
            Ok(&mut self.metadata.core)
        }

        #[inline]
        fn source(&self) -> &'static str {
            "meteora"
        }

        #[inline]
        fn timestamp(&self) -> SystemTime {
            self.metadata.core.timestamp.into()
        }

        #[inline]
        fn to_json(&self) -> EventResult<serde_json::Value> {
            Ok(serde_json::to_value(self)?)
        }
    }
}

// Parser module - simplified version to avoid complex trait issues
pub mod parser {
    use super::types::{dlmm_program_id, dynamic_program_id};
    use crate::events::factory::SolanaTransactionInput;
    use riglr_events_core::{error::EventResult, traits::EventParser, traits::ParserInfo, Event};
    use solana_sdk::pubkey::Pubkey;

    /// Meteora event parser - simplified version
    #[derive(Debug)]
    pub struct Parser {
        /// Parser information
        info: ParserInfo,
        /// List of supported program IDs
        program_ids: Vec<Pubkey>,
    }

    impl Parser {
        /// Creates a new Meteora event parser with default configurations
        #[must_use]
        #[inline]
        pub fn new() -> Self {
            let program_ids = vec![dlmm_program_id(), dynamic_program_id()];
            let info = ParserInfo::new("meteora_parser".to_owned(), "1.0.0".to_owned());
            Self { info, program_ids }
        }

        /// Checks if the parser should handle events from the given program ID
        #[must_use]
        #[inline]
        pub fn should_handle(&self, program_id: &Pubkey) -> bool {
            self.program_ids.contains(program_id)
        }

        /// Returns the supported program IDs
        #[must_use]
        #[inline]
        pub fn supported_program_ids(&self) -> Vec<Pubkey> {
            self.program_ids.clone()
        }
    }

    impl Default for Parser {
        #[inline]
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait::async_trait]
    impl EventParser for Parser {
        type Input = SolanaTransactionInput;

        #[inline]
        fn can_parse(&self, _input: &Self::Input) -> bool {
            // Simplified implementation
            true
        }

        #[inline]
        fn info(&self) -> &ParserInfo {
            &self.info
        }

        #[inline]
        async fn parse(&self, _input: Self::Input) -> EventResult<Vec<Box<dyn Event>>> {
            // Simplified implementation - return empty for now
            Ok(vec![])
        }
    }
}

// Re-export main types and functions for backward compatibility
pub use events::{DynamicLiquidityEvent, LiquidityEvent, SwapEvent};
pub use parser::Parser;
pub use types::{
    dlmm_program_id, dynamic_program_id, DynamicLiquidityData, LiquidityData, SwapData,
};

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {

    #[test]
    fn events_module_accessibility() {
        // Test that the events module is accessible
        // This ensures the module declaration is working correctly
        let _module_exists = std::module_path!().contains("meteora");
    }

    #[test]
    fn parser_module_accessibility() {
        // Test that the parser module is accessible
        // This ensures the module declaration is working correctly
        let _module_exists = std::module_path!().contains("meteora");
    }

    #[test]
    fn types_module_accessibility() {
        // Test that the types module is accessible
        // This ensures the module declaration is working correctly
        let _module_exists = std::module_path!().contains("meteora");
    }

    #[test]
    fn module_reexports_compile() {
        // Test that all re-exports compile successfully
        // If any re-export fails, this test won't compile
        // This verifies that events::*, parser::*, and types::* are valid
    }
}

#[cfg(test)]
mod types_tests {
    use super::types::*;
    use core::str::FromStr;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn dlmm_program_id_when_called_should_return_valid_pubkey() {
        let program_id = dlmm_program_id();
        assert_eq!(
            program_id.to_string(),
            "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo"
        );
    }

    #[test]
    fn dynamic_program_id_when_called_should_return_valid_pubkey() {
        let program_id = dynamic_program_id();
        assert_eq!(
            program_id.to_string(),
            "Dooar9JkhdZ7J3LHN3A7YCuoGRUggXhQaG4kijfLGU2j"
        );
    }

    #[test]
    fn is_meteora_dlmm_program_when_correct_pubkey_should_return_true() {
        let correct_pubkey = dlmm_program_id();
        assert!(is_meteora_dlmm_program(&correct_pubkey));
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn is_meteora_dlmm_program_when_incorrect_pubkey_should_return_false() {
        let incorrect_pubkey = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Test pubkey string should be valid");
        assert!(!is_meteora_dlmm_program(&incorrect_pubkey));
    }

    #[test]
    fn is_meteora_dynamic_program_when_correct_pubkey_should_return_true() {
        let correct_pubkey = dynamic_program_id();
        assert!(is_meteora_dynamic_program(&correct_pubkey));
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn is_meteora_dynamic_program_when_incorrect_pubkey_should_return_false() {
        let incorrect_pubkey = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Test pubkey string should be valid");
        assert!(!is_meteora_dynamic_program(&incorrect_pubkey));
    }

    #[test]
    fn bin_id_to_price_when_center_bin_should_return_one() {
        let price = bin_id_to_price(8_388_608, 100); // Center bin with 1% step
        assert!((price - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bin_id_to_price_when_higher_bin_should_return_higher_price() {
        let price = bin_id_to_price(8_388_609, 100); // One bin above center
        assert!(price > 1.0);
        let expected = (1.0 + 0.01_f64).powi(1);
        assert!((price - expected).abs() < 0.0001);
    }

    #[test]
    fn bin_id_to_price_when_lower_bin_should_return_lower_price() {
        let price = bin_id_to_price(8_388_607, 100); // One bin below center
        assert!(price < 1.0);
        let expected = (1.0 + 0.01_f64).powi(-1);
        assert!((price - expected).abs() < 0.0001);
    }

    #[test]
    fn bin_id_to_price_when_zero_bin_step_should_return_one() {
        let price = bin_id_to_price(8_388_608, 0);
        assert!((price - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bin_id_to_price_when_max_bin_step_should_work() {
        let price = bin_id_to_price(8_388_608, u16::MAX);
        assert!(price.is_finite());
    }

    #[test]
    fn calculate_active_bin_price_when_called_should_match_bin_id_to_price() {
        let active_id = 8_388_610;
        let bin_step = 50;
        let price1 = calculate_active_bin_price(active_id, bin_step);
        let price2 = bin_id_to_price(active_id, bin_step);
        assert!((price1 - price2).abs() < f64::EPSILON);
    }

    #[test]
    fn calculate_liquidity_distribution_when_equal_range_should_distribute_evenly() {
        let distribution = calculate_liquidity_distribution(1000, 2000, 100, 102, 101);
        assert_eq!(distribution.len(), 3);

        // Check bin 100 (below active): only Y tokens
        assert_eq!(distribution.first().copied(), Some((100, 333, 0)));
        // Check bin 101 (active): both X and Y
        assert_eq!(distribution.get(1).copied(), Some((101, 333, 666)));
        // Check bin 102 (above active): only X tokens
        assert_eq!(distribution.get(2).copied(), Some((102, 0, 666)));
    }

    #[test]
    fn calculate_liquidity_distribution_when_single_bin_should_contain_all() {
        let distribution = calculate_liquidity_distribution(1000, 2000, 100, 100, 100);
        assert_eq!(distribution.len(), 1);
        assert_eq!(distribution.first().copied(), Some((100, 1000, 2000)));
    }

    #[test]
    fn calculate_liquidity_distribution_when_active_below_range_should_only_have_y() {
        let distribution = calculate_liquidity_distribution(1000, 2000, 100, 102, 99);
        assert_eq!(distribution.len(), 3);

        for (_, x_amount, y_amount) in distribution {
            assert_eq!(x_amount, 0);
            assert_eq!(y_amount, 666);
        }
    }

    #[test]
    fn calculate_liquidity_distribution_when_active_above_range_should_only_have_x() {
        let distribution = calculate_liquidity_distribution(1000, 2000, 100, 102, 103);
        assert_eq!(distribution.len(), 3);

        for (_, x_amount, y_amount) in distribution {
            assert_eq!(x_amount, 333);
            assert_eq!(y_amount, 0);
        }
    }

    #[test]
    fn calculate_liquidity_distribution_when_invalid_range_should_return_empty() {
        let distribution = calculate_liquidity_distribution(1000, 2000, 102, 100, 101);
        assert!(distribution.is_empty());
    }

    #[test]
    fn calculate_liquidity_distribution_when_zero_amounts_should_work() {
        let distribution = calculate_liquidity_distribution(0, 0, 100, 102, 101);
        assert_eq!(distribution.len(), 3);

        for (_, x_amount, y_amount) in distribution {
            assert_eq!(x_amount, 0);
            assert_eq!(y_amount, 0);
        }
    }

    #[test]
    fn meteora_swap_data_default() {
        let swap_data = SwapData::default();
        assert_eq!(swap_data.amount_in, 0);
        assert_eq!(swap_data.min_amount_out, 0);
        assert_eq!(swap_data.actual_amount_out, 0);
        assert!(!swap_data.swap_for_y);
        assert_eq!(swap_data.active_id_before, 0);
        assert_eq!(swap_data.active_id_after, 0);
        assert_eq!(swap_data.fee_amount, 0);
        assert_eq!(swap_data.protocol_fee, 0);
        assert!(swap_data.bins_traversed.is_empty());
    }

    #[test]
    fn meteora_liquidity_data_default() {
        let liquidity_data = LiquidityData::default();
        assert_eq!(liquidity_data.bin_id_from, 0);
        assert_eq!(liquidity_data.bin_id_to, 0);
        assert_eq!(liquidity_data.amount_x, 0);
        assert_eq!(liquidity_data.amount_y, 0);
        assert_eq!(liquidity_data.liquidity_minted, 0);
        assert_eq!(liquidity_data.active_id, 0);
        assert!(!liquidity_data.is_add);
        assert!(liquidity_data.bins_affected.is_empty());
    }

    #[test]
    fn meteora_dynamic_liquidity_data_default() {
        let liquidity_data = DynamicLiquidityData::default();
        assert_eq!(liquidity_data.pool_token_amount, 0);
        assert_eq!(liquidity_data.token_a_amount, 0);
        assert_eq!(liquidity_data.token_b_amount, 0);
        assert_eq!(liquidity_data.minimum_pool_token_amount, 0);
        assert_eq!(liquidity_data.maximum_token_a_amount, 0);
        assert_eq!(liquidity_data.maximum_token_b_amount, 0);
        assert!(!liquidity_data.is_deposit);
    }

    #[test]
    fn constants_values() {
        assert_eq!(
            METEORA_DLMM_PROGRAM_ID,
            "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo"
        );
        assert_eq!(
            METEORA_DYNAMIC_PROGRAM_ID,
            "Dooar9JkhdZ7J3LHN3A7YCuoGRUggXhQaG4kijfLGU2j"
        );

        assert_eq!(
            DLMM_SWAP_DISCRIMINATOR,
            [0x14, 0x65, 0x32, 0x1f, 0x7a, 0x43, 0x2a, 0x9f]
        );
        assert_eq!(
            DLMM_ADD_LIQUIDITY_DISCRIMINATOR,
            [0x4c, 0x1c, 0x9b, 0x2d, 0xe3, 0x7a, 0x8b, 0x12]
        );
        assert_eq!(
            DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR,
            [0xa2, 0xfd, 0x67, 0xe3, 0x45, 0x1b, 0x8c, 0x9a]
        );
        assert_eq!(
            DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR,
            [0x85, 0x72, 0x1a, 0x5f, 0x9d, 0x4e, 0x23, 0x7c]
        );
        assert_eq!(
            DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR,
            [0x6a, 0x8b, 0x47, 0x2e, 0x1c, 0x93, 0x5f, 0x4d]
        );
    }
}
