use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Jupiter V6 program ID
pub const JUPITER_V6_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";

/// Jupiter swap discriminators (calculated from Anchor's "global:<instruction_name>")
/// sharedAccountsRoute is the most common Jupiter V6 swap instruction
pub const ROUTE_DISCRIMINATOR: [u8; 8] = [0x57, 0x03, 0xfe, 0xb8, 0xe7, 0x57, 0x39, 0x09]; // sharedAccountsRoute
pub const EXACT_OUT_ROUTE_DISCRIMINATOR: [u8; 8] = [0x41, 0xd8, 0xfa, 0x8d, 0xac, 0x72, 0x6b, 0x69]; // sharedAccountsExactOutRoute

// Alternative discriminators for other Jupiter instruction types
pub const LEGACY_ROUTE_DISCRIMINATOR: [u8; 8] = [0xe5, 0x17, 0xcb, 0x97, 0x7a, 0xe3, 0xad, 0x2a]; // route
pub const LEGACY_EXACT_OUT_DISCRIMINATOR: [u8; 8] =
    [0x7e, 0x2c, 0x8e, 0xa1, 0xd9, 0xa6, 0x5b, 0xc6]; // exactOutRoute
pub const SWAP_DISCRIMINATOR: [u8; 8] = [0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8]; // swap
pub const ROUTE_WITH_TOKEN_LEDGER_DISCRIMINATOR: [u8; 8] =
    [0x34, 0x65, 0x0f, 0x14, 0x74, 0x5e, 0x8d, 0xe8]; // routeWithTokenLedger

/// Jupiter shared accounts route instruction data (after discriminator)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct SharedAccountsRouteData {
    pub route_plan: Vec<RoutePlanStep>,
    pub in_amount: u64,
    pub quoted_out_amount: u64,
    pub slippage_bps: u16,
    pub platform_fee_bps: u8,
}

/// Jupiter exact out route instruction data (after discriminator)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct SharedAccountsExactOutRouteData {
    pub route_plan: Vec<RoutePlanStep>,
    pub out_amount: u64,
    pub quoted_in_amount: u64,
    pub slippage_bps: u16,
    pub platform_fee_bps: u8,
}

/// Route plan step for Jupiter swaps
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RoutePlanStep {
    pub swap: SwapInfo,
    pub percent: u8,
}

/// Swap information within a route step
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct SwapInfo {
    pub source_token: Pubkey,
    pub destination_token: Pubkey,
    pub source_token_account: Pubkey,
    pub destination_token_account: Pubkey,
    pub swap_program_id: Pubkey,
    pub swap_accounts: Vec<AccountMeta>,
    pub swap_data: Vec<u8>,
}

/// Account metadata for swap
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct AccountMeta {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

/// Jupiter swap event data (for UnifiedEvent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterSwapData {
    pub user: Pubkey,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact_pct: Option<String>,
    pub platform_fee_bps: Option<u32>,
    pub route_plan: Vec<RoutePlan>,
}

/// Route plan information (simplified for event data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePlan {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub dex_label: String,
}

/// Jupiter program account layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterAccountLayout {
    pub user_transfer_authority: Pubkey,
    pub user_source_token_account: Pubkey,
    pub user_destination_token_account: Pubkey,
    pub destination_token_account: Pubkey,
    pub source_mint: Pubkey,
    pub destination_mint: Pubkey,
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
