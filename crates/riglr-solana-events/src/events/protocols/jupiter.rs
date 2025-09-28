/// Jupiter protocol event definitions and constants.
use borsh::{BorshDeserialize, BorshSerialize};
use core::any::Any;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, sync::OnceLock, time::SystemTime};

// Re-export core types
use crate::events::core::EventParameters;
use crate::solana_metadata::SolanaEventMetadata;
use crate::types::{metadata_helpers, EventType, ParserInfo, ProtocolType, TransferData};
use riglr_events_core::EventMetadata as CoreEventMetadata;
use riglr_events_core::{
    error::EventResult,
    traits::{EventFilter, EventParser as CoreEventParser},
};
use riglr_events_core::{Event, EventKind};

// For parser functionality
use crate::{
    error::ParseResult,
    events::{
        factory::{InnerInstructionParseParams, InstructionParseParams, SolanaTransactionInput},
        parser_types::{GenericEventParseConfig, ProtocolParser},
    },
};

// ==================== TYPES SECTION ====================

/// Jupiter V6 program ID
pub const JUPITER_V6_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";

/// Jupiter swap discriminators (calculated from Anchor's "global:<`instruction_name`>")
/// sharedAccountsRoute is the most common Jupiter V6 swap instruction
pub const ROUTE_DISCRIMINATOR: [u8; 8] = [0x57, 0x03, 0xfe, 0xb8, 0xe7, 0x57, 0x39, 0x09]; // sharedAccountsRoute
/// Discriminator for shared accounts exact out route instruction
pub const EXACT_OUT_ROUTE_DISCRIMINATOR: [u8; 8] = [0x41, 0xd8, 0xfa, 0x8d, 0xac, 0x72, 0x6b, 0x69]; // sharedAccountsExactOutRoute

// Alternative discriminators for other Jupiter instruction types
/// Discriminator for legacy route instruction
pub const LEGACY_ROUTE_DISCRIMINATOR: [u8; 8] = [0xe5, 0x17, 0xcb, 0x97, 0x7a, 0xe3, 0xad, 0x2a]; // route
/// Discriminator for legacy exact out route instruction
pub const LEGACY_EXACT_OUT_DISCRIMINATOR: [u8; 8] =
    [0x7e, 0x2c, 0x8e, 0xa1, 0xd9, 0xa6, 0x5b, 0xc6]; // exactOutRoute
/// Discriminator for swap instruction
pub const SWAP_DISCRIMINATOR: [u8; 8] = [0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8]; // swap
/// Discriminator for route with token ledger instruction
pub const ROUTE_WITH_TOKEN_LEDGER_DISCRIMINATOR: [u8; 8] =
    [0x34, 0x65, 0x0f, 0x14, 0x74, 0x5e, 0x8d, 0xe8]; // routeWithTokenLedger

/// Jupiter shared accounts route instruction data (after discriminator)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[non_exhaustive]
pub struct SharedAccountsRouteData {
    /// Input amount in base units
    pub in_amount: u64,
    /// Platform fee in basis points
    pub platform_fee_bps: u8,
    /// Quoted output amount in base units
    pub quoted_out_amount: u64,
    /// Route plan steps for the swap
    pub route_plan: Vec<RoutePlanStep>,
    /// Slippage tolerance in basis points
    pub slippage_bps: u16,
}

/// Jupiter exact out route instruction data (after discriminator)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[non_exhaustive]
pub struct SharedAccountsExactOutRouteData {
    /// Output amount in base units
    pub out_amount: u64,
    /// Platform fee in basis points
    pub platform_fee_bps: u8,
    /// Quoted input amount in base units
    pub quoted_in_amount: u64,
    /// Route plan steps for the swap
    pub route_plan: Vec<RoutePlanStep>,
    /// Slippage tolerance in basis points
    pub slippage_bps: u16,
}

/// Route plan step for Jupiter swaps
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[non_exhaustive]
pub struct RoutePlanStep {
    /// Percentage of the input amount for this step
    pub percent: u8,
    /// Swap information for this step
    pub swap: SwapInfo,
}

/// Swap information within a route step
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[non_exhaustive]
pub struct SwapInfo {
    /// Destination token mint address
    pub destination_token: Pubkey,
    /// Destination token account address
    pub destination_token_account: Pubkey,
    /// Source token mint address
    pub source_token: Pubkey,
    /// Source token account address
    pub source_token_account: Pubkey,
    /// Account metadata required for the swap
    pub swap_accounts: Vec<AccountMeta>,
    /// Instruction data for the swap
    pub swap_data: Vec<u8>,
    /// Program ID for the swap
    pub swap_program_id: Pubkey,
}

/// Account metadata for swap
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[non_exhaustive]
pub struct AccountMeta {
    /// Whether the account is a signer
    pub is_signer: bool,
    /// Whether the account is writable
    pub is_writable: bool,
    /// Public key of the account
    pub pubkey: Pubkey,
}

/// Jupiter swap event data (for `UnifiedEvent`)
#[expect(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct JupiterSwapData {
    /// Input amount in base units
    pub input_amount: u64,
    /// Input token mint address
    pub input_mint: Pubkey,
    /// Output amount in base units
    pub output_amount: u64,
    /// Output token mint address
    pub output_mint: Pubkey,
    /// Platform fee in basis points
    pub platform_fee_bps: Option<u32>,
    /// Price impact percentage as string
    pub price_impact_pct: Option<String>,
    /// Route plan for the swap
    pub route_plan: Vec<RoutePlan>,
    /// User who initiated the swap
    pub user: Pubkey,
}

/// Route plan information (simplified for event data)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RoutePlan {
    /// Amount going into this step
    pub amount_in: u64,
    /// Amount coming out of this step
    pub amount_out: u64,
    /// Label identifying the DEX used
    pub dex_label: String,
    /// Input token mint address
    pub input_mint: Pubkey,
    /// Output token mint address
    pub output_mint: Pubkey,
}

/// Jupiter program account layout
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccountLayout {
    /// Destination token mint address
    pub destination_mint: Pubkey,
    /// Destination token account for the swap
    pub destination_token_account: Pubkey,
    /// Optional platform fee account
    pub platform_fee_account: Option<Pubkey>,
    /// Source token mint address
    pub source_mint: Pubkey,
    /// User's destination token account
    pub user_destination_token_account: Pubkey,
    /// User's source token account
    pub user_source_token_account: Pubkey,
    /// User's transfer authority
    pub user_transfer_authority: Pubkey,
}

// ==================== EVENTS SECTION ====================

/// Jupiter swap event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct SwapEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Jupiter-specific swap data
    pub swap_data: JupiterSwapData,
    /// Associated token transfer data
    pub transfer_data: Vec<TransferData>,
}

/// Jupiter liquidity provision event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct LiquidityEvent {
    /// Amount of token A
    pub amount_a: u64,
    /// Amount of token B
    pub amount_b: u64,
    /// Whether this is a liquidity removal operation
    pub is_remove: bool,
    /// Amount of liquidity tokens
    pub liquidity_amount: u64,
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// First token mint address
    pub mint_a: Pubkey,
    /// Second token mint address
    pub mint_b: Pubkey,
    /// Associated token transfer data
    pub transfer_data: Vec<TransferData>,
    /// User account providing/removing liquidity
    pub user: Pubkey,
}

/// Jupiter swap event with borsh (for simple events)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
#[non_exhaustive]
pub struct SwapBorshEvent {
    /// Input token amount
    pub input_amount: u64,
    /// Input token mint address
    pub input_mint: Pubkey,
    /// Event metadata (skipped during serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: SolanaEventMetadata,
    /// Output token amount received
    pub output_amount: u64,
    /// Output token mint address
    pub output_mint: Pubkey,
    /// Platform fee in basis points
    pub platform_fee_bps: u8,
    /// Slippage tolerance in basis points
    pub slippage_bps: u16,
    /// User account performing the swap
    pub user: Pubkey,
}

impl SwapEvent {
    /// Creates a new `SwapEvent` with the provided parameters and swap data
    #[must_use]
    #[inline]
    pub fn new(params: EventParameters, swap_data: JupiterSwapData) -> Self {
        let metadata = metadata_helpers::create_solana_metadata(
            params.id,
            params.signature,
            params.slot,
            params.block_time,
            ProtocolType::Jupiter,
            EventType::Swap,
            v6_program_id(),
            params.index,
            params.program_received_time_ms,
        );

        Self {
            metadata,
            swap_data,
            transfer_data: Vec::default(),
        }
    }

    /// Adds transfer data to the swap event
    #[must_use]
    #[inline]
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

impl LiquidityEvent {
    /// Creates a new `LiquidityEvent` with the provided parameters
    #[must_use]
    #[inline]
    pub fn new(params: EventParameters) -> Self {
        let metadata = metadata_helpers::create_solana_metadata(
            params.id,
            params.signature,
            params.slot,
            params.block_time,
            ProtocolType::Jupiter,
            EventType::AddLiquidity,
            v6_program_id(),
            params.index,
            params.program_received_time_ms,
        );

        Self {
            metadata,
            user: Pubkey::default(),
            mint_a: Pubkey::default(),
            mint_b: Pubkey::default(),
            amount_a: 0,
            amount_b: 0,
            liquidity_amount: 0,
            is_remove: false,
            transfer_data: Vec::default(),
        }
    }

    /// Sets amounts and returns self for method chaining
    #[must_use]
    #[inline]
    pub const fn with_amounts(
        mut self,
        amount_a: u64,
        amount_b: u64,
        liquidity_amount: u64,
    ) -> Self {
        self.amount_a = amount_a;
        self.amount_b = amount_b;
        self.liquidity_amount = liquidity_amount;
        self
    }

    /// Sets token mints and returns self for method chaining
    #[must_use]
    #[inline]
    pub const fn with_mints(mut self, mint_a: Pubkey, mint_b: Pubkey) -> Self {
        self.mint_a = mint_a;
        self.mint_b = mint_b;
        self
    }

    /// Sets removal flag and returns self for method chaining
    #[must_use]
    #[inline]
    pub const fn with_removal(mut self, is_remove: bool) -> Self {
        self.is_remove = is_remove;
        self
    }

    /// Adds transfer data to the event
    #[must_use]
    #[inline]
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }

    /// Sets user account and returns self for method chaining
    #[must_use]
    #[inline]
    pub const fn with_user(mut self, user: Pubkey) -> Self {
        self.user = user;
        self
    }
}

/// Extract Jupiter program ID as Pubkey
///
/// # Panics
/// Never panics - this is a compile-time constant that is verified by tests
// Event trait implementation for SwapEvent
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
    fn source(&self) -> &'static str {
        "jupiter"
    }
    fn timestamp(&self) -> SystemTime {
        self.metadata.core.timestamp.into()
    }

    #[inline]
    fn to_json(&self) -> EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

/// Returns the Jupiter V6 program ID as a Pubkey.
///
/// This uses a static lazy-evaluated Pubkey to avoid repeated parsing.
static JUPITER_V6_PUBKEY: OnceLock<Pubkey> = OnceLock::new();

/// Returns the Jupiter V6 program ID.
///
/// Returns the Jupiter V6 program ID.
///
/// This function will return a default pubkey if the constant is somehow invalid,
/// though this should never happen as it's a hardcoded valid constant.
pub fn v6_program_id() -> Pubkey {
    *JUPITER_V6_PUBKEY.get_or_init(|| {
        Pubkey::try_from(JUPITER_V6_PROGRAM_ID).unwrap_or_else(|_| Pubkey::default())
    })
}

// Event trait implementation for LiquidityEvent
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
    fn source(&self) -> &'static str {
        "jupiter"
    }
    fn timestamp(&self) -> SystemTime {
        self.metadata.core.timestamp.into()
    }

    #[inline]
    fn to_json(&self) -> EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

/// Check if the given pubkey is Jupiter V6 program
#[must_use]
#[inline]
pub fn is_jupiter_v6_program(program_id: &Pubkey) -> bool {
    *program_id == v6_program_id()
}

// Event trait implementation for SwapBorshEvent
impl Event for SwapBorshEvent {
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
    fn source(&self) -> &'static str {
        "jupiter"
    }
    fn timestamp(&self) -> SystemTime {
        self.metadata.core.timestamp.into()
    }

    #[inline]
    fn to_json(&self) -> EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

// ==================== PARSER SECTION ====================

/// Jupiter event parser
#[derive(Debug)]
pub struct EventParser {
    /// Parser information
    info: ParserInfo,
    /// Inner instruction configuration mappings
    inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>,
    /// Instruction configuration mappings
    instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
    /// Supported program IDs
    program_ids: Vec<Pubkey>,
}

impl EventParser {
    /// Creates a new Jupiter event parser with default configurations for routing and exact-out routing
    #[inline]
    pub fn new() -> Self {
        let program_ids = vec![v6_program_id()];

        let configs = vec![
            GenericEventParseConfig {
                program_id: v6_program_id(),
                protocol_type: ProtocolType::Other("Jupiter".to_owned()),
                inner_instruction_discriminator: "swap",
                instruction_discriminator: &ROUTE_DISCRIMINATOR,
                event_type: EventType::Swap,
                inner_instruction_parser: parse_jupiter_swap_inner_instruction,
                instruction_parser: parse_jupiter_swap_instruction,
            },
            GenericEventParseConfig {
                program_id: v6_program_id(),
                protocol_type: ProtocolType::Other("Jupiter".to_owned()),
                inner_instruction_discriminator: "exactOutRoute",
                instruction_discriminator: &EXACT_OUT_ROUTE_DISCRIMINATOR,
                event_type: EventType::Swap,
                inner_instruction_parser: parse_jupiter_exact_out_inner_instruction,
                instruction_parser: parse_jupiter_exact_out_instruction,
            },
        ];

        let mut inner_instruction_configs = HashMap::default();
        let mut instruction_configs = HashMap::default();

        for config in configs {
            inner_instruction_configs
                .entry(config.inner_instruction_discriminator)
                .or_insert_with(Vec::default)
                .push(config.clone());
            instruction_configs
                .entry(config.instruction_discriminator.to_vec())
                .or_insert_with(Vec::default)
                .push(config);
        }

        let info = ParserInfo::new("jupiter".to_owned(), "1.0.0".to_owned())
            .with_format("solana".to_owned())
            .with_kind(EventKind::Swap)
            .with_kind(EventKind::Liquidity);

        Self {
            info,
            inner_instruction_configs,
            instruction_configs,
            program_ids,
        }
    }
}

impl Default for EventParser {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the new core EventParser trait
#[async_trait::async_trait]
impl CoreEventParser for EventParser {
    type Input = SolanaTransactionInput;

    #[inline]
    fn can_parse(&self, _input: &Self::Input) -> bool {
        // For now, assume we can parse all inputs
        true
    }

    #[inline]
    fn info(&self) -> &ParserInfo {
        &self.info
    }

    #[inline]
    async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>> {
        let events = match input {
            SolanaTransactionInput::InnerInstruction(params) => {
                let legacy_params = crate::events::factory::InnerInstructionParseParams {
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

                self.parse_events_from_inner_instruction(&legacy_params)
            }
            SolanaTransactionInput::Instruction(_params) => {
                // For now, only inner instructions are supported
                vec![]
            }
        };

        Ok(events)
    }
}

// Legacy ProtocolParser trait implementation for backward compatibility
impl ProtocolParser for EventParser {
    #[inline]
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner_instruction_configs.clone()
    }

    #[inline]
    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.instruction_configs.clone()
    }

    #[inline]
    fn parse_events_from_inner_instruction(
        &self,
        _params: &InnerInstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        // Implementation would go here - this is a simplified version
        // In the actual implementation, this would parse the inner instruction data
        // and create SwapEvent instances based on the discriminators
        vec![]
    }

    #[inline]
    fn parse_events_from_instruction(
        &self,
        _params: &InstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        // Implementation would go here - this is a simplified version
        vec![]
    }

    #[inline]
    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.program_ids.contains(program_id)
    }

    #[inline]
    fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.program_ids.clone()
    }
}

// Parser helper functions (these would be implemented with actual parsing logic)

/// Parse Jupiter swap inner instruction data into event
#[expect(clippy::unnecessary_wraps)]
fn parse_jupiter_swap_inner_instruction(
    _data: &[u8],
    _metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    // Implementation would parse Jupiter swap inner instructions
    Ok(Box::new(SwapEvent::default()))
}

/// Parse Jupiter swap instruction data into event
#[expect(clippy::unnecessary_wraps)]
fn parse_jupiter_swap_instruction(
    _data: &[u8],
    _accounts: &[Pubkey],
    _metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    // Implementation would parse Jupiter swap instructions
    Ok(Box::new(SwapEvent::default()))
}

/// Parse Jupiter exact-out inner instruction data into event
#[expect(clippy::unnecessary_wraps)]
fn parse_jupiter_exact_out_inner_instruction(
    _data: &[u8],
    _metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    // Implementation would parse Jupiter exact-out inner instructions
    Ok(Box::new(SwapEvent::default()))
}

/// Parse Jupiter exact-out instruction data into event
#[expect(clippy::unnecessary_wraps)]
fn parse_jupiter_exact_out_instruction(
    _data: &[u8],
    _accounts: &[Pubkey],
    _metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    // Implementation would parse Jupiter exact-out instructions
    Ok(Box::new(SwapEvent::default()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn module_exports_events() {
        // Test that event types are accessible through re-exports
        let _event = EventParameters::default();
        let _swap_event = SwapEvent::default();
        let _liquidity_event = LiquidityEvent::default();
        let _borsh_event = SwapBorshEvent::default();
    }

    #[test]
    fn module_exports_parser() {
        // Test that parser types are accessible through re-exports
        let parser = EventParser::default();
        assert!(!parser.supported_program_ids().is_empty());
    }

    #[test]
    fn module_exports_types() {
        // Test that type constants and functions are accessible through re-exports
        let program_id = v6_program_id();
        assert!(is_jupiter_v6_program(&program_id));

        // Test discriminator constants are accessible
        let _: [u8; 8] = ROUTE_DISCRIMINATOR;
        let _: [u8; 8] = EXACT_OUT_ROUTE_DISCRIMINATOR;
        let _: [u8; 8] = LEGACY_ROUTE_DISCRIMINATOR;
        let _: [u8; 8] = LEGACY_EXACT_OUT_DISCRIMINATOR;
        let _: [u8; 8] = SWAP_DISCRIMINATOR;
        let _: [u8; 8] = ROUTE_WITH_TOKEN_LEDGER_DISCRIMINATOR;

        // Test type structs are accessible
        let _swap_data = JupiterSwapData::default();
        let _route_plan = RoutePlan {
            input_mint: Pubkey::default(),
            output_mint: Pubkey::default(),
            amount_in: 0,
            amount_out: 0,
            dex_label: String::default(),
        };
        let _account_layout = AccountLayout {
            user_transfer_authority: Pubkey::default(),
            user_source_token_account: Pubkey::default(),
            user_destination_token_account: Pubkey::default(),
            destination_token_account: Pubkey::default(),
            source_mint: Pubkey::default(),
            destination_mint: Pubkey::default(),
            platform_fee_account: None,
        };
    }

    #[test]
    fn jupiter_program_id_constant() {
        // Test that the program ID constant is accessible and valid
        assert_eq!(
            JUPITER_V6_PROGRAM_ID,
            "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
        );
    }

    #[test]
    fn v6_program_id_function() {
        // Test the v6_program_id function
        let program_id = v6_program_id();
        assert_eq!(program_id.to_string(), JUPITER_V6_PROGRAM_ID);
    }

    #[test]
    fn is_jupiter_v6_program_when_correct_program_id_should_return_true() {
        // Test with correct Jupiter V6 program ID
        let program_id = v6_program_id();
        assert!(is_jupiter_v6_program(&program_id));
    }

    #[test]
    fn is_jupiter_v6_program_when_incorrect_program_id_should_return_false() {
        // Test with a different program ID
        let other_program_id = Pubkey::default();
        assert!(!is_jupiter_v6_program(&other_program_id));
    }

    #[test]
    fn discriminator_constants_have_correct_length() {
        // Test that all discriminator constants have the correct 8-byte length
        assert_eq!(ROUTE_DISCRIMINATOR.len(), 8);
        assert_eq!(EXACT_OUT_ROUTE_DISCRIMINATOR.len(), 8);
        assert_eq!(LEGACY_ROUTE_DISCRIMINATOR.len(), 8);
        assert_eq!(LEGACY_EXACT_OUT_DISCRIMINATOR.len(), 8);
        assert_eq!(SWAP_DISCRIMINATOR.len(), 8);
        assert_eq!(ROUTE_WITH_TOKEN_LEDGER_DISCRIMINATOR.len(), 8);
    }

    #[test]
    fn discriminator_constants_are_unique() {
        // Test that all discriminator constants are unique
        let discriminators = [
            ROUTE_DISCRIMINATOR,
            EXACT_OUT_ROUTE_DISCRIMINATOR,
            LEGACY_ROUTE_DISCRIMINATOR,
            LEGACY_EXACT_OUT_DISCRIMINATOR,
            SWAP_DISCRIMINATOR,
            ROUTE_WITH_TOKEN_LEDGER_DISCRIMINATOR,
        ];

        for (i, disc1) in discriminators.iter().enumerate() {
            for (j, disc2) in discriminators.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        disc1, disc2,
                        "Discriminators at indices {i} and {j} are identical"
                    );
                }
            }
        }
    }

    #[test]
    fn discriminator_constants_have_expected_values() {
        // Test that discriminators have the expected byte values (prevents accidental changes)
        assert_eq!(
            ROUTE_DISCRIMINATOR,
            [0x57, 0x03, 0xfe, 0xb8, 0xe7, 0x57, 0x39, 0x09]
        );
        assert_eq!(
            EXACT_OUT_ROUTE_DISCRIMINATOR,
            [0x41, 0xd8, 0xfa, 0x8d, 0xac, 0x72, 0x6b, 0x69]
        );
        assert_eq!(
            LEGACY_ROUTE_DISCRIMINATOR,
            [0xe5, 0x17, 0xcb, 0x97, 0x7a, 0xe3, 0xad, 0x2a]
        );
        assert_eq!(
            LEGACY_EXACT_OUT_DISCRIMINATOR,
            [0x7e, 0x2c, 0x8e, 0xa1, 0xd9, 0xa6, 0x5b, 0xc6]
        );
        assert_eq!(
            SWAP_DISCRIMINATOR,
            [0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8]
        );
        assert_eq!(
            ROUTE_WITH_TOKEN_LEDGER_DISCRIMINATOR,
            [0x34, 0x65, 0x0f, 0x14, 0x74, 0x5e, 0x8d, 0xe8]
        );
    }
}
