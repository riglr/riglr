use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Jupiter V6 program ID
pub const JUPITER_V6_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";

/// Jupiter swap discriminators (calculated from Anchor's "global:<instruction_name>")
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
pub struct SharedAccountsRouteData {
    /// Route plan steps for the swap
    pub route_plan: Vec<RoutePlanStep>,
    /// Input amount in base units
    pub in_amount: u64,
    /// Quoted output amount in base units
    pub quoted_out_amount: u64,
    /// Slippage tolerance in basis points
    pub slippage_bps: u16,
    /// Platform fee in basis points
    pub platform_fee_bps: u8,
}

/// Jupiter exact out route instruction data (after discriminator)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct SharedAccountsExactOutRouteData {
    /// Route plan steps for the swap
    pub route_plan: Vec<RoutePlanStep>,
    /// Output amount in base units
    pub out_amount: u64,
    /// Quoted input amount in base units
    pub quoted_in_amount: u64,
    /// Slippage tolerance in basis points
    pub slippage_bps: u16,
    /// Platform fee in basis points
    pub platform_fee_bps: u8,
}

/// Route plan step for Jupiter swaps
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RoutePlanStep {
    /// Swap information for this step
    pub swap: SwapInfo,
    /// Percentage of the input amount for this step
    pub percent: u8,
}

/// Swap information within a route step
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct SwapInfo {
    /// Source token mint address
    pub source_token: Pubkey,
    /// Destination token mint address
    pub destination_token: Pubkey,
    /// Source token account address
    pub source_token_account: Pubkey,
    /// Destination token account address
    pub destination_token_account: Pubkey,
    /// Program ID for the swap
    pub swap_program_id: Pubkey,
    /// Account metadata required for the swap
    pub swap_accounts: Vec<AccountMeta>,
    /// Instruction data for the swap
    pub swap_data: Vec<u8>,
}

/// Account metadata for swap
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct AccountMeta {
    /// Public key of the account
    pub pubkey: Pubkey,
    /// Whether the account is a signer
    pub is_signer: bool,
    /// Whether the account is writable
    pub is_writable: bool,
}

/// Jupiter swap event data (for UnifiedEvent)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JupiterSwapData {
    /// User who initiated the swap
    pub user: Pubkey,
    /// Input token mint address
    pub input_mint: Pubkey,
    /// Output token mint address
    pub output_mint: Pubkey,
    /// Input amount in base units
    pub input_amount: u64,
    /// Output amount in base units
    pub output_amount: u64,
    /// Price impact percentage as string
    pub price_impact_pct: Option<String>,
    /// Platform fee in basis points
    pub platform_fee_bps: Option<u32>,
    /// Route plan for the swap
    pub route_plan: Vec<RoutePlan>,
}

/// Route plan information (simplified for event data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePlan {
    /// Input token mint address
    pub input_mint: Pubkey,
    /// Output token mint address
    pub output_mint: Pubkey,
    /// Amount going into this step
    pub amount_in: u64,
    /// Amount coming out of this step
    pub amount_out: u64,
    /// Label identifying the DEX used
    pub dex_label: String,
}

/// Jupiter program account layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterAccountLayout {
    /// User's transfer authority
    pub user_transfer_authority: Pubkey,
    /// User's source token account
    pub user_source_token_account: Pubkey,
    /// User's destination token account
    pub user_destination_token_account: Pubkey,
    /// Destination token account for the swap
    pub destination_token_account: Pubkey,
    /// Source token mint address
    pub source_mint: Pubkey,
    /// Destination token mint address
    pub destination_mint: Pubkey,
    /// Optional platform fee account
    pub platform_fee_account: Option<Pubkey>,
}

/// Extract Jupiter program ID as Pubkey
pub fn jupiter_v6_program_id() -> Pubkey {
    Pubkey::try_from(JUPITER_V6_PROGRAM_ID).expect("Invalid Jupiter V6 program ID")
}

/// Check if the given pubkey is Jupiter V6 program
pub fn is_jupiter_v6_program(program_id: &Pubkey) -> bool {
    *program_id == jupiter_v6_program_id()
}
