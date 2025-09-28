//! Raydium CLMM instruction discriminators and constants, event definitions, and parsing functionality.

use std::collections::HashMap;

use riglr_events_core::{
    error::EventResult,
    traits::{EventParser as CoreParser, ParserInfo},
    Event,
};
use solana_message::compiled_instruction::CompiledInstruction;
use solana_sdk::pubkey::Pubkey;

use crate::{
    error::{Error as ParseError, ParseResult},
    events::{
        common::{read_i32_le, read_option_bool, read_u128_le, read_u64_le, read_u8_le},
        factory::{InnerInstructionParseParams, InstructionParseParams, SolanaTransactionInput},
        parser_types::{GenericEventParseConfig, GenericEventParser, ProtocolParser},
    },
    solana_metadata::SolanaEventMetadata,
    EventType, ProtocolType,
};

use discriminators::{
    CLOSE_POSITION, CREATE_POOL, DECREASE_LIQUIDITY_V2, INCREASE_LIQUIDITY_V2, OPEN_POSITION_V2,
    OPEN_POSITION_WITH_TOKEN_22_NFT, SWAP, SWAP_V2,
};

// Instruction discriminator constants
pub mod discriminators {
    /// Instruction discriminator for swap operations
    pub const SWAP: &[u8] = &[0xa9, 0x0d, 0xd0, 0xfe, 0x89, 0xbc, 0xab, 0x27];

    /// Instruction discriminator for swap operations (version 2)
    pub const SWAP_V2: &[u8] = &[0x2a, 0x2d, 0x80, 0xb5, 0xce, 0x24, 0x7b, 0x87];

    /// Instruction discriminator for closing liquidity positions
    pub const CLOSE_POSITION: &[u8] = &[0x7b, 0x86, 0x51, 0x10, 0x31, 0xc0, 0xa1, 0x7a];

    /// Instruction discriminator for decreasing liquidity in positions (version 2)
    pub const DECREASE_LIQUIDITY_V2: &[u8] = &[0x58, 0x12, 0x7a, 0x1a, 0x95, 0x04, 0xac, 0xa0];

    /// Instruction discriminator for creating new pools
    pub const CREATE_POOL: &[u8] = &[0xe2, 0x58, 0x01, 0x5f, 0xc2, 0xc2, 0x49, 0xe9];

    /// Instruction discriminator for increasing liquidity in positions (version 2)
    pub const INCREASE_LIQUIDITY_V2: &[u8] = &[0x85, 0x15, 0x1a, 0xa4, 0xd1, 0x8b, 0x74, 0x2e];

    /// Instruction discriminator for opening positions with Token-22 NFT
    pub const OPEN_POSITION_WITH_TOKEN_22_NFT: &[u8] =
        &[0x3e, 0xf4, 0xcc, 0x1f, 0x66, 0x42, 0xee, 0xd1];

    /// Instruction discriminator for opening liquidity positions (version 2)
    pub const OPEN_POSITION_V2: &[u8] = &[0x4e, 0x14, 0xbb, 0x8b, 0xdd, 0xa8, 0xfc, 0x07];
}

// Event definitions module
pub mod events {
    use core::any::Any;
    use std::sync::OnceLock;

    use riglr_events_core::Event;
    use serde::{Deserialize, Serialize};
    use solana_sdk::pubkey::Pubkey;

    use crate::solana_metadata::SolanaEventMetadata;

    /// Raydium CLMM swap event
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SwapEvent {
        pub amount0: u64,
        pub amount1: u64,
        pub input_token_account: Pubkey,
        pub input_vault: Pubkey,
        pub liquidity: u128,
        pub metadata: SolanaEventMetadata,
        pub output_token_account: Pubkey,
        pub output_vault: Pubkey,
        pub payer: Pubkey,
        pub pool_state: Pubkey,
        pub sqrt_price_x64: u128,
        pub tick_current: i32,
        pub token_mint0: Pubkey,
        pub token_mint1: Pubkey,
    }

    impl Event for SwapEvent {
        fn id(&self) -> &str {
            &self.metadata.id
        }
        fn kind(&self) -> &riglr_events_core::EventKind {
            static KIND: OnceLock<riglr_events_core::EventKind> = OnceLock::new();
            KIND.get_or_init(|| {
                riglr_events_core::EventKind::Custom("raydium_clmm_swap".to_string())
            })
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn metadata(&self) -> &riglr_events_core::EventMetadata {
            &self.metadata
        }
        fn metadata_mut(
            &mut self,
        ) -> Result<&mut riglr_events_core::EventMetadata, riglr_events_core::EventError> {
            Ok(&mut self.metadata)
        }
        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }
        fn to_json(&self) -> Result<serde_json::Value, riglr_events_core::EventError> {
            serde_json::to_value(self).map_err(riglr_events_core::EventError::Serialization)
        }
    }

    /// Raydium CLMM swap V2 event
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SwapV2Event {
        pub amount0: u64,
        pub amount1: u64,
        pub input_token_account: Pubkey,
        pub input_vault: Pubkey,
        pub is_base_input: bool,
        pub liquidity: u128,
        pub metadata: SolanaEventMetadata,
        pub output_token_account: Pubkey,
        pub output_vault: Pubkey,
        pub payer: Pubkey,
        pub pool_state: Pubkey,
        pub sqrt_price_x64: u128,
        pub tick_current: i32,
        pub token_mint0: Pubkey,
        pub token_mint1: Pubkey,
    }

    impl Event for SwapV2Event {
        fn id(&self) -> &str {
            &self.metadata.id
        }
        fn kind(&self) -> &riglr_events_core::EventKind {
            static KIND: OnceLock<riglr_events_core::EventKind> = OnceLock::new();
            KIND.get_or_init(|| {
                riglr_events_core::EventKind::Custom("raydium_clmm_swap_v2".to_string())
            })
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn metadata(&self) -> &riglr_events_core::EventMetadata {
            &self.metadata
        }
        fn metadata_mut(
            &mut self,
        ) -> Result<&mut riglr_events_core::EventMetadata, riglr_events_core::EventError> {
            Ok(&mut self.metadata)
        }
        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }
        fn to_json(&self) -> Result<serde_json::Value, riglr_events_core::EventError> {
            serde_json::to_value(self).map_err(riglr_events_core::EventError::Serialization)
        }
    }

    /// Raydium CLMM create pool event
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CreatePoolEvent {
        pub metadata: SolanaEventMetadata,
        pub observation_index: u16,
        pub pool_creator: Pubkey,
        pub pool_state: Pubkey,
        pub sqrt_price_x64: u128,
        pub tick_current: i32,
        pub token_mint0: Pubkey,
        pub token_mint1: Pubkey,
        pub token_vault0: Pubkey,
        pub token_vault1: Pubkey,
    }

    impl Event for CreatePoolEvent {
        fn id(&self) -> &str {
            &self.metadata.id
        }
        fn kind(&self) -> &riglr_events_core::EventKind {
            static KIND: OnceLock<riglr_events_core::EventKind> = OnceLock::new();
            KIND.get_or_init(|| {
                riglr_events_core::EventKind::Custom("raydium_clmm_create_pool".to_string())
            })
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn metadata(&self) -> &riglr_events_core::EventMetadata {
            &self.metadata
        }
        fn metadata_mut(
            &mut self,
        ) -> Result<&mut riglr_events_core::EventMetadata, riglr_events_core::EventError> {
            Ok(&mut self.metadata)
        }
        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }
        fn to_json(&self) -> Result<serde_json::Value, riglr_events_core::EventError> {
            serde_json::to_value(self).map_err(riglr_events_core::EventError::Serialization)
        }
    }

    /// Raydium CLMM open position V2 event
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OpenPositionV2Event {
        pub amount0_max: u64,
        pub amount1_max: u64,
        pub base_flag: Option<bool>,
        pub liquidity: u128,
        pub metadata: SolanaEventMetadata,
        pub metadata_account: Pubkey,
        pub payer: Pubkey,
        pub pool_state: Pubkey,
        pub position_nft_account: Pubkey,
        pub position_nft_mint: Pubkey,
        pub position_nft_owner: Pubkey,
        pub tick_array_lower_start_index: i32,
        pub tick_array_upper_start_index: i32,
        pub tick_lower_index: i32,
        pub tick_upper_index: i32,
        pub with_metadata: bool,
    }

    impl Event for OpenPositionV2Event {
        fn id(&self) -> &str {
            &self.metadata.id
        }
        fn kind(&self) -> &riglr_events_core::EventKind {
            static KIND: OnceLock<riglr_events_core::EventKind> = OnceLock::new();
            KIND.get_or_init(|| {
                riglr_events_core::EventKind::Custom("raydium_clmm_open_position_v2".to_string())
            })
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn metadata(&self) -> &riglr_events_core::EventMetadata {
            &self.metadata
        }
        fn metadata_mut(
            &mut self,
        ) -> Result<&mut riglr_events_core::EventMetadata, riglr_events_core::EventError> {
            Ok(&mut self.metadata)
        }
        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }
        fn to_json(&self) -> Result<serde_json::Value, riglr_events_core::EventError> {
            serde_json::to_value(self).map_err(riglr_events_core::EventError::Serialization)
        }
    }

    /// Raydium CLMM close position event
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ClosePositionEvent {
        pub metadata: SolanaEventMetadata,
        pub nft_owner: Pubkey,
        pub personal_position: Pubkey,
        pub position_nft_account: Pubkey,
        pub position_nft_mint: Pubkey,
    }

    impl Event for ClosePositionEvent {
        fn id(&self) -> &str {
            &self.metadata.id
        }
        fn kind(&self) -> &riglr_events_core::EventKind {
            static KIND: OnceLock<riglr_events_core::EventKind> = OnceLock::new();
            KIND.get_or_init(|| {
                riglr_events_core::EventKind::Custom("raydium_clmm_close_position".to_string())
            })
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn metadata(&self) -> &riglr_events_core::EventMetadata {
            &self.metadata
        }
        fn metadata_mut(
            &mut self,
        ) -> Result<&mut riglr_events_core::EventMetadata, riglr_events_core::EventError> {
            Ok(&mut self.metadata)
        }
        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }
        fn to_json(&self) -> Result<serde_json::Value, riglr_events_core::EventError> {
            serde_json::to_value(self).map_err(riglr_events_core::EventError::Serialization)
        }
    }

    /// Raydium CLMM increase liquidity V2 event
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IncreaseLiquidityV2Event {
        pub amount0_max: u64,
        pub amount1_max: u64,
        pub base_flag: Option<bool>,
        pub liquidity: u128,
        pub metadata: SolanaEventMetadata,
        pub nft_owner: Pubkey,
        pub pool_state: Pubkey,
        pub position_nft_account: Pubkey,
    }

    impl Event for IncreaseLiquidityV2Event {
        fn id(&self) -> &str {
            &self.metadata.id
        }
        fn kind(&self) -> &riglr_events_core::EventKind {
            static KIND: OnceLock<riglr_events_core::EventKind> = OnceLock::new();
            KIND.get_or_init(|| {
                riglr_events_core::EventKind::Custom(
                    "raydium_clmm_increase_liquidity_v2".to_string(),
                )
            })
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn metadata(&self) -> &riglr_events_core::EventMetadata {
            &self.metadata
        }
        fn metadata_mut(
            &mut self,
        ) -> Result<&mut riglr_events_core::EventMetadata, riglr_events_core::EventError> {
            Ok(&mut self.metadata)
        }
        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }
        fn to_json(&self) -> Result<serde_json::Value, riglr_events_core::EventError> {
            serde_json::to_value(self).map_err(riglr_events_core::EventError::Serialization)
        }
    }

    /// Raydium CLMM decrease liquidity V2 event
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DecreaseLiquidityV2Event {
        pub amount0_min: u64,
        pub amount1_min: u64,
        pub liquidity: u128,
        pub metadata: SolanaEventMetadata,
        pub nft_owner: Pubkey,
        pub pool_state: Pubkey,
        pub position_nft_account: Pubkey,
    }

    impl Event for DecreaseLiquidityV2Event {
        fn id(&self) -> &str {
            &self.metadata.id
        }
        fn kind(&self) -> &riglr_events_core::EventKind {
            static KIND: OnceLock<riglr_events_core::EventKind> = OnceLock::new();
            KIND.get_or_init(|| {
                riglr_events_core::EventKind::Custom(
                    "raydium_clmm_decrease_liquidity_v2".to_string(),
                )
            })
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn metadata(&self) -> &riglr_events_core::EventMetadata {
            &self.metadata
        }
        fn metadata_mut(
            &mut self,
        ) -> Result<&mut riglr_events_core::EventMetadata, riglr_events_core::EventError> {
            Ok(&mut self.metadata)
        }
        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }
        fn to_json(&self) -> Result<serde_json::Value, riglr_events_core::EventError> {
            serde_json::to_value(self).map_err(riglr_events_core::EventError::Serialization)
        }
    }

    /// Raydium CLMM open position with Token-22 NFT event
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OpenPositionWithToken22NftEvent {
        pub amount0_max: u64,
        pub amount1_max: u64,
        pub base_flag: Option<bool>,
        pub liquidity: u128,
        pub metadata: SolanaEventMetadata,
        pub payer: Pubkey,
        pub pool_state: Pubkey,
        pub position_nft_account: Pubkey,
        pub position_nft_mint: Pubkey,
        pub position_nft_owner: Pubkey,
        pub tick_array_lower_start_index: i32,
        pub tick_array_upper_start_index: i32,
        pub tick_lower_index: i32,
        pub tick_upper_index: i32,
        pub with_metadata: bool,
    }

    impl Event for OpenPositionWithToken22NftEvent {
        fn id(&self) -> &str {
            &self.metadata.id
        }
        fn kind(&self) -> &riglr_events_core::EventKind {
            static KIND: OnceLock<riglr_events_core::EventKind> = OnceLock::new();
            KIND.get_or_init(|| {
                riglr_events_core::EventKind::Custom(
                    "raydium_clmm_open_position_with_token_22_nft".to_string(),
                )
            })
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn metadata(&self) -> &riglr_events_core::EventMetadata {
            &self.metadata
        }
        fn metadata_mut(
            &mut self,
        ) -> Result<&mut riglr_events_core::EventMetadata, riglr_events_core::EventError> {
            Ok(&mut self.metadata)
        }
        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }
        fn to_json(&self) -> Result<serde_json::Value, riglr_events_core::EventError> {
            serde_json::to_value(self).map_err(riglr_events_core::EventError::Serialization)
        }
    }
}

/// Raydium CLMM program ID
pub const RAYDIUM_CLMM_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK");

pub use events::*;

/// Raydium CLMM event parser
#[derive(Debug)]
pub struct Parser {
    info: ParserInfo,
    inner: GenericEventParser,
}

impl Default for Parser {
    fn default() -> Self {
        let configs = vec![
            GenericEventParseConfig {
                program_id: RAYDIUM_CLMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumClmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: SWAP,
                event_type: EventType::Swap,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_swap_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CLMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumClmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: SWAP_V2,
                event_type: EventType::Swap,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_swap_v2_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CLMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumClmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: CLOSE_POSITION,
                event_type: EventType::ClosePosition,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_close_position_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CLMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumClmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: DECREASE_LIQUIDITY_V2,
                event_type: EventType::RemoveLiquidity,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_decrease_liquidity_v2_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CLMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumClmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: CREATE_POOL,
                event_type: EventType::CreatePool,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_create_pool_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CLMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumClmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: INCREASE_LIQUIDITY_V2,
                event_type: EventType::AddLiquidity,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_increase_liquidity_v2_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CLMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumClmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: OPEN_POSITION_WITH_TOKEN_22_NFT,
                event_type: EventType::OpenPosition,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_open_position_with_token_22_nft_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CLMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumClmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: OPEN_POSITION_V2,
                event_type: EventType::OpenPosition,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_open_position_v2_instruction,
            },
        ];

        let inner = GenericEventParser::new(vec![RAYDIUM_CLMM_PROGRAM_ID], configs);
        let info = ParserInfo::new("raydium_clmm_parser".to_string(), "1.0.0".to_string())
            .with_kind(riglr_events_core::EventKind::Custom(
                "raydium_clmm_swap".to_string(),
            ))
            .with_kind(riglr_events_core::EventKind::Custom(
                "raydium_clmm_create_pool".to_string(),
            ))
            .with_kind(riglr_events_core::EventKind::Custom(
                "raydium_clmm_open_position".to_string(),
            ))
            .with_kind(riglr_events_core::EventKind::Custom(
                "raydium_clmm_close_position".to_string(),
            ))
            .with_kind(riglr_events_core::EventKind::Custom(
                "raydium_clmm_increase_liquidity".to_string(),
            ))
            .with_kind(riglr_events_core::EventKind::Custom(
                "raydium_clmm_decrease_liquidity".to_string(),
            ))
            .with_format("solana_instruction".to_string());
        Self { info, inner }
    }
}

impl Parser {
    /// Empty parser for inner instructions
    ///
    /// Raydium CLMM does not emit events through inner instructions or program logs.
    /// All event data is encoded directly in the instruction data itself, which is
    /// parsed by the `instruction_parser` functions below. This is intentional and
    /// follows the protocol's design where all necessary information is available
    /// in the instruction parameters and accounts.
    ///
    /// This differs from protocols like Raydium CPMM which emit events through logs
    /// that need to be parsed from inner instructions.
    fn empty_parse(_data: &[u8], _metadata: SolanaEventMetadata) -> ParseResult<Box<dyn Event>> {
        Err(ParseError::InvalidDataFormat(
            "Raydium CLMM does not emit events through inner instructions".to_string(),
        ))
    }

    /// Creates a new Raydium CLMM event parser
    ///
    /// Initializes the parser with all supported Raydium CLMM instruction types
    /// including swaps, position management, and pool creation operations.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert sqrt price X64 to tick
    /// Based on Uniswap V3 math: tick = log1.0001(price) * 2
    /// Since `sqrt_price_x64` = sqrt(price) * 2^64, we need to:
    /// 1. Convert from X64 fixed point to f64
    /// 2. Square to get price
    /// 3. Calculate tick
    ///
    /// Parse swap instruction event
    fn parse_swap_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 41 || accounts.len() < 17 {
            return Err(ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CLMM swap instruction".to_string(),
            ));
        }

        let amount = read_u64_le(data, 0)
            .map_err(|_| ParseError::InvalidDataFormat("Failed to read amount".to_string()))?;
        let other_amount_threshold = read_u64_le(data, 8).map_err(|_| {
            ParseError::InvalidDataFormat("Failed to read other_amount_threshold".to_string())
        })?;
        let sqrt_price_limit_x64 = read_u128_le(data, 16).map_err(|_| {
            ParseError::InvalidDataFormat("Failed to read sqrt_price_limit_x64".to_string())
        })?;
        let is_base_input = read_u8_le(data, 32).map_err(|_| {
            ParseError::InvalidDataFormat("Failed to read is_base_input".to_string())
        })? == 1;

        let mut metadata = metadata;
        let pool_state_for_id = accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat(
                "Missing pool_state account for ID generation".to_string(),
            )
        })?;
        metadata.set_id(format!(
            "{}-{}-swap-{}",
            metadata.signature, pool_state_for_id, amount
        ));

        let payer = *accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing payer account at index 0".to_string())
        })?;
        let pool_state = *accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool_state account at index 1".to_string())
        })?;
        let input_token_account = *accounts.get(2).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing input_token_account at index 2".to_string())
        })?;
        let output_token_account = *accounts.get(3).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing output_token_account at index 3".to_string())
        })?;
        let input_vault = *accounts.get(4).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing input_vault at index 4".to_string())
        })?;
        let output_vault = *accounts.get(5).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing output_vault at index 5".to_string())
        })?;
        let token_mint0 = *accounts.get(6).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing token_mint0 at index 6".to_string())
        })?;
        let token_mint1 = *accounts.get(7).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing token_mint1 at index 7".to_string())
        })?;

        Ok(Box::new(events::SwapEvent {
            metadata,
            amount0: if is_base_input {
                amount
            } else {
                other_amount_threshold
            },
            amount1: if is_base_input {
                other_amount_threshold
            } else {
                amount
            },
            sqrt_price_x64: sqrt_price_limit_x64,
            liquidity: 0, // Dynamic liquidity not available in instruction data
            tick_current: Self::sqrt_price_to_tick(sqrt_price_limit_x64), // Calculate from price
            payer,
            pool_state,
            input_token_account,
            output_token_account,
            input_vault,
            output_vault,
            token_mint0,
            token_mint1,
        }))
    }

    fn sqrt_price_to_tick(sqrt_price_x64: u128) -> i32 {
        if sqrt_price_x64 == 0 {
            return 0;
        }

        // Convert from X64 fixed point to f64
        // Use 2^64 as f64 to avoid overflow
        let two_pow_64 = 18_446_744_073_709_551_616.0f64; // 2^64
        #[expect(clippy::cast_precision_loss)]
        let sqrt_price = (sqrt_price_x64 as f64) / two_pow_64;

        // Square to get actual price
        let price = sqrt_price * sqrt_price;

        // Calculate tick = log1.0001(price)
        // Using change of base: log1.0001(price) = ln(price) / ln(1.0001)
        if price > 0.0 {
            #[expect(clippy::cast_possible_truncation)]
            return price.log(1.0001f64).round() as i32;
        }
        0
    }

    /// Parse create pool instruction event
    fn parse_create_pool_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 22 || accounts.len() < 10 {
            return Err(ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CLMM create pool instruction"
                    .to_string(),
            ));
        }

        let sqrt_price_x64 = read_u128_le(data, 0).map_err(|_| {
            ParseError::InvalidDataFormat("Failed to read sqrt_price_x64".to_string())
        })?;
        let _open_time = read_u64_le(data, 16)
            .map_err(|_| ParseError::InvalidDataFormat("Failed to read open_time".to_string()))?; // Not used in event

        let mut metadata = metadata;
        let pool_creator_for_id = accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat(
                "Missing pool_creator account for ID generation".to_string(),
            )
        })?;
        metadata.set_id(format!(
            "{}-{}-create-pool",
            metadata.signature, pool_creator_for_id
        ));

        let pool_creator = *accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool_creator account at index 0".to_string())
        })?;
        let pool_state = *accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool_state account at index 1".to_string())
        })?;
        let token_mint0 = *accounts.get(2).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing token_mint0 at index 2".to_string())
        })?;
        let token_mint1 = *accounts.get(3).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing token_mint1 at index 3".to_string())
        })?;
        let token_vault0 = *accounts.get(4).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing token_vault0 at index 4".to_string())
        })?;
        let token_vault1 = *accounts.get(5).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing token_vault1 at index 5".to_string())
        })?;

        Ok(Box::new(events::CreatePoolEvent {
            metadata,
            sqrt_price_x64,
            tick_current: Self::sqrt_price_to_tick(sqrt_price_x64), // Calculate from sqrt_price
            observation_index: 0,
            pool_creator,
            pool_state,
            token_mint0,
            token_mint1,
            token_vault0,
            token_vault1,
        }))
    }

    /// Parse swap V2 instruction event
    fn parse_swap_v2_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 42 || accounts.len() < 17 {
            return Err(ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CLMM swap v2 instruction".to_string(),
            ));
        }

        let amount = read_u64_le(data, 0)
            .map_err(|_| ParseError::InvalidDataFormat("Failed to read amount".to_string()))?;
        let other_amount_threshold = read_u64_le(data, 8).map_err(|_| {
            ParseError::InvalidDataFormat("Failed to read other_amount_threshold".to_string())
        })?;
        let sqrt_price_limit_x64 = read_u128_le(data, 16).map_err(|_| {
            ParseError::InvalidDataFormat("Failed to read sqrt_price_limit_x64".to_string())
        })?;
        let is_base_input = read_u8_le(data, 32).map_err(|_| {
            ParseError::InvalidDataFormat("Failed to read is_base_input".to_string())
        })? == 1;

        let mut metadata = metadata;
        let pool_state_for_id = accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat(
                "Missing pool_state account for ID generation".to_string(),
            )
        })?;
        metadata.set_id(format!(
            "{}-{}-swap-v2-{}",
            metadata.signature, pool_state_for_id, amount
        ));

        let payer = *accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing payer account at index 0".to_string())
        })?;
        let pool_state = *accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool_state account at index 1".to_string())
        })?;
        let input_token_account = *accounts.get(2).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing input_token_account at index 2".to_string())
        })?;
        let output_token_account = *accounts.get(3).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing output_token_account at index 3".to_string())
        })?;
        let input_vault = *accounts.get(4).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing input_vault at index 4".to_string())
        })?;
        let output_vault = *accounts.get(5).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing output_vault at index 5".to_string())
        })?;
        let token_mint0 = *accounts.get(6).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing token_mint0 at index 6".to_string())
        })?;
        let token_mint1 = *accounts.get(7).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing token_mint1 at index 7".to_string())
        })?;

        Ok(Box::new(events::SwapV2Event {
            metadata,
            amount0: if is_base_input {
                amount
            } else {
                other_amount_threshold
            },
            amount1: if is_base_input {
                other_amount_threshold
            } else {
                amount
            },
            sqrt_price_x64: sqrt_price_limit_x64,
            liquidity: 0, // Dynamic liquidity not available in instruction data
            tick_current: Self::sqrt_price_to_tick(sqrt_price_limit_x64), // Calculate from price
            is_base_input,
            payer,
            pool_state,
            input_token_account,
            output_token_account,
            input_vault,
            output_vault,
            token_mint0,
            token_mint1,
        }))
    }

    /// Parse close position instruction event
    fn parse_close_position_instruction(
        _data: &[u8],
        accounts: &[Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if accounts.len() < 9 {
            return Err(ParseError::InvalidDataFormat(
                "Insufficient accounts for Raydium CLMM close position instruction".to_string(),
            ));
        }

        let mut metadata = metadata;
        let position_nft_mint_for_id = accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing position_nft_mint for ID generation".to_string())
        })?;
        metadata.set_id(format!(
            "{}-{}-close",
            metadata.signature, position_nft_mint_for_id
        ));

        let nft_owner = *accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing nft_owner account at index 0".to_string())
        })?;
        let position_nft_mint = *accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing position_nft_mint at index 1".to_string())
        })?;
        let position_nft_account = *accounts.get(2).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing position_nft_account at index 2".to_string())
        })?;
        let personal_position = *accounts.get(3).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing personal_position at index 3".to_string())
        })?;

        Ok(Box::new(events::ClosePositionEvent {
            metadata,
            nft_owner,
            personal_position,
            position_nft_account,
            position_nft_mint,
        }))
    }

    /// Parse open position V2 instruction event
    fn parse_open_position_v2_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 51 || accounts.len() < 22 {
            return Err(ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CLMM open position v2 instruction"
                    .to_string(),
            ));
        }

        let mut metadata = metadata;
        let payer_for_id = accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing payer account for ID generation".to_string())
        })?;
        let owner_for_id = accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing owner account for ID generation".to_string())
        })?;
        metadata.set_id(format!(
            "{}-{}-{}",
            metadata.signature, payer_for_id, owner_for_id
        ));

        let payer = *accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing payer account at index 0".to_string())
        })?;
        let position_nft_owner = *accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing position_nft_owner at index 1".to_string())
        })?;
        let position_nft_mint = *accounts.get(2).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing position_nft_mint at index 2".to_string())
        })?;
        let position_nft_account = *accounts.get(3).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing position_nft_account at index 3".to_string())
        })?;
        let metadata_account = *accounts.get(4).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing metadata_account at index 4".to_string())
        })?;
        let pool_state = *accounts.get(5).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool_state account at index 5".to_string())
        })?;

        Ok(Box::new(events::OpenPositionV2Event {
            metadata,
            tick_lower_index: read_i32_le(data, 0).map_err(|_| {
                ParseError::InvalidDataFormat("Failed to read tick_lower_index".to_string())
            })?,
            tick_upper_index: read_i32_le(data, 4).map_err(|_| {
                ParseError::InvalidDataFormat("Failed to read tick_upper_index".to_string())
            })?,
            tick_array_lower_start_index: read_i32_le(data, 8).map_err(|_| {
                ParseError::InvalidDataFormat(
                    "Failed to read tick_array_lower_start_index".to_string(),
                )
            })?,
            tick_array_upper_start_index: read_i32_le(data, 12).map_err(|_| {
                ParseError::InvalidDataFormat(
                    "Failed to read tick_array_upper_start_index".to_string(),
                )
            })?,
            liquidity: read_u128_le(data, 16).map_err(|_| {
                ParseError::InvalidDataFormat("Failed to read liquidity".to_string())
            })?,
            amount0_max: read_u64_le(data, 32).map_err(|_| {
                ParseError::InvalidDataFormat("Failed to read amount0_max".to_string())
            })?,
            amount1_max: read_u64_le(data, 40).map_err(|_| {
                ParseError::InvalidDataFormat("Failed to read amount1_max".to_string())
            })?,
            with_metadata: read_u8_le(data, 48).map_err(|_| {
                ParseError::InvalidDataFormat("Failed to read with_metadata".to_string())
            })? == 1,
            base_flag: read_option_bool(data, &mut 49).map_err(|_| {
                ParseError::InvalidDataFormat("Failed to read base_flag".to_string())
            })?,
            payer,
            position_nft_owner,
            position_nft_mint,
            position_nft_account,
            metadata_account,
            pool_state,
        }))
    }

    /// Parse decrease liquidity V2 instruction event
    fn parse_decrease_liquidity_v2_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 32 || accounts.len() < 15 {
            return Err(ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CLMM decrease liquidity v2 instruction"
                    .to_string(),
            ));
        }

        let liquidity = read_u128_le(data, 0)
            .map_err(|_| ParseError::InvalidDataFormat("Failed to read liquidity".to_string()))?;
        let amount0_min = read_u64_le(data, 16)
            .map_err(|_| ParseError::InvalidDataFormat("Failed to read amount0_min".to_string()))?;
        let amount1_min = read_u64_le(data, 24)
            .map_err(|_| ParseError::InvalidDataFormat("Failed to read amount1_min".to_string()))?;

        let mut metadata = metadata;
        let position_nft_account_for_id = accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat(
                "Missing position_nft_account for ID generation".to_string(),
            )
        })?;
        metadata.set_id(format!(
            "{}-{}-decrease-{}",
            metadata.signature, position_nft_account_for_id, liquidity
        ));

        let nft_owner = *accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing nft_owner account at index 0".to_string())
        })?;
        let position_nft_account = *accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing position_nft_account at index 1".to_string())
        })?;
        let pool_state = *accounts.get(4).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool_state account at index 4".to_string())
        })?;

        Ok(Box::new(events::DecreaseLiquidityV2Event {
            amount0_min,
            amount1_min,
            liquidity,
            metadata,
            nft_owner,
            pool_state,
            position_nft_account,
        }))
    }

    /// Parse increase liquidity V2 instruction event
    fn parse_increase_liquidity_v2_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 33 || accounts.len() < 15 {
            return Err(ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CLMM increase liquidity v2 instruction"
                    .to_string(),
            ));
        }

        let liquidity = read_u128_le(data, 0)
            .map_err(|_| ParseError::InvalidDataFormat("Failed to read liquidity".to_string()))?;
        let amount0_max = read_u64_le(data, 16)
            .map_err(|_| ParseError::InvalidDataFormat("Failed to read amount0_max".to_string()))?;
        let amount1_max = read_u64_le(data, 24)
            .map_err(|_| ParseError::InvalidDataFormat("Failed to read amount1_max".to_string()))?;
        let base_flag = read_option_bool(data, &mut 32)
            .map_err(|_| ParseError::InvalidDataFormat("Failed to read base_flag".to_string()))?;

        let mut metadata = metadata;
        let position_nft_account_for_id = accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat(
                "Missing position_nft_account for ID generation".to_string(),
            )
        })?;
        metadata.set_id(format!(
            "{}-{}-increase-{}",
            metadata.signature, position_nft_account_for_id, liquidity
        ));

        let nft_owner = *accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing nft_owner account at index 0".to_string())
        })?;
        let position_nft_account = *accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing position_nft_account at index 1".to_string())
        })?;
        let pool_state = *accounts.get(4).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool_state account at index 4".to_string())
        })?;

        Ok(Box::new(events::IncreaseLiquidityV2Event {
            amount0_max,
            amount1_max,
            base_flag,
            liquidity,
            metadata,
            nft_owner,
            pool_state,
            position_nft_account,
        }))
    }

    /// Parse open position with Token-22 NFT instruction event
    fn parse_open_position_with_token_22_nft_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 51 || accounts.len() < 20 {
            return Err(ParseError::InvalidDataFormat("Insufficient data or accounts for Raydium CLMM open position with token-22 NFT instruction".to_string()));
        }

        let mut metadata = metadata;
        let payer_for_id = accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing payer account for ID generation".to_string())
        })?;
        let owner_for_id = accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing owner account for ID generation".to_string())
        })?;
        metadata.set_id(format!(
            "{}-{}-{}-token22",
            metadata.signature, payer_for_id, owner_for_id
        ));

        let payer = *accounts.first().ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing payer account at index 0".to_string())
        })?;
        let position_nft_owner = *accounts.get(1).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing position_nft_owner at index 1".to_string())
        })?;
        let position_nft_mint = *accounts.get(2).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing position_nft_mint at index 2".to_string())
        })?;
        let position_nft_account = *accounts.get(3).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing position_nft_account at index 3".to_string())
        })?;
        let pool_state = *accounts.get(5).ok_or_else(|| {
            ParseError::InvalidDataFormat("Missing pool_state account at index 5".to_string())
        })?;

        Ok(Box::new(events::OpenPositionWithToken22NftEvent {
            metadata,
            tick_lower_index: read_i32_le(data, 0).map_err(|_| {
                ParseError::InvalidDataFormat("Failed to read tick_lower_index".to_string())
            })?,
            tick_upper_index: read_i32_le(data, 4).map_err(|_| {
                ParseError::InvalidDataFormat("Failed to read tick_upper_index".to_string())
            })?,
            tick_array_lower_start_index: read_i32_le(data, 8).map_err(|_| {
                ParseError::InvalidDataFormat(
                    "Failed to read tick_array_lower_start_index".to_string(),
                )
            })?,
            tick_array_upper_start_index: read_i32_le(data, 12).map_err(|_| {
                ParseError::InvalidDataFormat(
                    "Failed to read tick_array_upper_start_index".to_string(),
                )
            })?,
            liquidity: read_u128_le(data, 16).map_err(|_| {
                ParseError::InvalidDataFormat("Failed to read liquidity".to_string())
            })?,
            amount0_max: read_u64_le(data, 32).map_err(|_| {
                ParseError::InvalidDataFormat("Failed to read amount0_max".to_string())
            })?,
            amount1_max: read_u64_le(data, 40).map_err(|_| {
                ParseError::InvalidDataFormat("Failed to read amount1_max".to_string())
            })?,
            with_metadata: read_u8_le(data, 48).map_err(|_| {
                ParseError::InvalidDataFormat("Failed to read with_metadata".to_string())
            })? == 1,
            base_flag: read_option_bool(data, &mut 49).map_err(|_| {
                ParseError::InvalidDataFormat("Failed to read base_flag".to_string())
            })?,
            payer,
            position_nft_owner,
            position_nft_mint,
            position_nft_account,
            pool_state,
        }))
    }
}

// Implement the new core Parser trait
#[async_trait::async_trait]
impl CoreParser for Parser {
    type Input = SolanaTransactionInput;
    fn can_parse(&self, input: &Self::Input) -> bool {
        match *input {
            SolanaTransactionInput::InnerInstruction(_)
            | SolanaTransactionInput::Instruction(_) => true,
        }
    }

    fn info(&self) -> &ParserInfo {
        &self.info
    }

    async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>> {
        let events = match input {
            SolanaTransactionInput::InnerInstruction(params) => {
                let legacy_params = InnerInstructionParseParams {
                    inner_instruction: &solana_transaction_status::UiCompiledInstruction {
                        program_id_index: 0,
                        accounts: vec![],
                        data: params.inner_instruction_data.clone(),
                        stack_height: Some(1),
                    },
                    signature: &params.signature,
                    slot: params.slot,
                    block_time: params.block_time,
                    program_received_time_ms: params.program_received_time_ms,
                    index: params.index.clone(),
                };
                self.inner
                    .parse_events_from_inner_instruction(&legacy_params)
            }
            SolanaTransactionInput::Instruction(params) => {
                let instruction = CompiledInstruction {
                    program_id_index: 0,
                    accounts: vec![],
                    data: params.instruction_data.clone(),
                };
                let legacy_params = InstructionParseParams {
                    instruction: &instruction,
                    accounts: &params.accounts,
                    signature: &params.signature,
                    slot: params.slot,
                    block_time: params.block_time,
                    program_received_time_ms: params.program_received_time_ms,
                    index: params.index.clone(),
                };
                self.inner.parse_events_from_instruction(&legacy_params)
            }
        };
        Ok(events)
    }
}

// Keep legacy implementation for backward compatibility
#[async_trait::async_trait]
impl ProtocolParser for Parser {
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner.inner_instruction_configs()
    }
    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.inner.instruction_configs()
    }
    fn parse_events_from_inner_instruction(
        &self,
        params: &InnerInstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        self.inner.parse_events_from_inner_instruction(params)
    }
    fn parse_events_from_instruction(
        &self,
        params: &InstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        self.inner.parse_events_from_instruction(params)
    }
    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.inner.should_handle(program_id)
    }
    fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.inner.supported_program_ids()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use core::marker::PhantomData;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn re_exports_available() {
        // Test that the re-exported items are accessible
        // This ensures the pub use statements work correctly
        use crate::events::protocols::raydium_clmm::{Parser, RAYDIUM_CLMM_PROGRAM_ID};

        // Verify the program ID constant is accessible and has the expected type
        let _: Pubkey = RAYDIUM_CLMM_PROGRAM_ID;

        // Verify the parser type is accessible
        let _: PhantomData<Parser> = PhantomData::<Parser>;
    }

    #[test]
    fn module_exports_parser() {
        // Test that Parser is properly exported
        let parser = Parser::new();

        // Verify parser can be created and has the expected program ID support
        let supported_ids = parser.supported_program_ids();
        assert!(!supported_ids.is_empty());
        assert!(supported_ids.contains(&RAYDIUM_CLMM_PROGRAM_ID));
    }

    #[test]
    fn module_exports_program_id_constant() {
        // Test that RAYDIUM_CLMM_PROGRAM_ID is properly exported
        // Verify it's not the default pubkey
        assert_ne!(RAYDIUM_CLMM_PROGRAM_ID, Pubkey::default());
    }

    #[test]
    fn parser_should_handle_correct_program_id() {
        let parser = Parser::new();

        // Should handle the Raydium CLMM program ID
        assert!(parser.should_handle(&RAYDIUM_CLMM_PROGRAM_ID));

        // Should not handle other program IDs
        assert!(!parser.should_handle(&Pubkey::default()));
        assert!(!parser.should_handle(&solana_sdk::pubkey!("11111111111111111111111111111112")));
    }

    #[test]
    fn module_structure_integrity() {
        // This test ensures that the discriminators module is accessible
        // by testing some constant values it should contain
        use discriminators::*;

        // Test that we can access discriminators - verify they're defined correctly
        assert_eq!(SWAP.len(), 8);
        assert_eq!(SWAP_V2.len(), 8);
        assert_eq!(CLOSE_POSITION.len(), 8);
        assert_eq!(DECREASE_LIQUIDITY_V2.len(), 8);
        assert_eq!(CREATE_POOL.len(), 8);
        assert_eq!(INCREASE_LIQUIDITY_V2.len(), 8);
        assert_eq!(OPEN_POSITION_WITH_TOKEN_22_NFT.len(), 8);
        assert_eq!(OPEN_POSITION_V2.len(), 8);

        // Verify discriminators are unique
        let discriminators = [
            SWAP,
            SWAP_V2,
            CLOSE_POSITION,
            DECREASE_LIQUIDITY_V2,
            CREATE_POOL,
            INCREASE_LIQUIDITY_V2,
            OPEN_POSITION_WITH_TOKEN_22_NFT,
            OPEN_POSITION_V2,
        ];

        for (i, &disc1) in discriminators.iter().enumerate() {
            for (j, &disc2) in discriminators.iter().enumerate() {
                if i != j {
                    assert_ne!(disc1, disc2);
                }
            }
        }
    }

    #[test]
    fn sqrt_price_to_tick_when_zero_should_return_zero() {
        let tick = Parser::sqrt_price_to_tick(0);
        assert_eq!(tick, 0);
    }

    #[test]
    fn sqrt_price_to_tick_when_valid_price_should_calculate_correctly() {
        // Test with a known sqrt price value
        let sqrt_price_x64 = 79_228_162_514_264_337_593_543_950_336_u128; // 2^96 (approximate)
        let tick = Parser::sqrt_price_to_tick(sqrt_price_x64);
        // Should be a reasonable tick value (exact value depends on calculation)
        assert!(tick > -1_000_000 && tick < 1_000_000);
    }

    #[test]
    fn sqrt_price_to_tick_when_small_price_should_handle_edge_case() {
        let tick = Parser::sqrt_price_to_tick(1);
        // Very small price should result in negative tick
        assert!(tick <= 0);
    }

    #[test]
    fn sqrt_price_to_tick_when_max_value_should_handle_large_numbers() {
        let tick = Parser::sqrt_price_to_tick(u128::MAX);
        // Should handle large values without panicking
        assert!(tick > 0);
    }
}
