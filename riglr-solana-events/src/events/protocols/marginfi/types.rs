use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// MarginFi program ID
pub const MARGINFI_PROGRAM_ID: &str = "MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA";

/// MarginFi Bank program ID (for lending pools)
pub const MARGINFI_BANK_PROGRAM_ID: &str = "4Be9aW2D8f3G2b3ZP8uo5kd9z8zwJ3FYD1tNKmQ9c9x";

/// MarginFi instruction discriminators
pub const MARGINFI_DEPOSIT_DISCRIMINATOR: [u8; 8] =
    [0x13, 0x65, 0x32, 0x1f, 0x7a, 0x43, 0x2a, 0x9f];
pub const MARGINFI_WITHDRAW_DISCRIMINATOR: [u8; 8] =
    [0x4c, 0x1c, 0x9b, 0x2d, 0xe3, 0x7a, 0x8b, 0x12];
pub const MARGINFI_BORROW_DISCRIMINATOR: [u8; 8] = [0xa2, 0xfd, 0x67, 0xe3, 0x45, 0x1b, 0x8c, 0x9a];
pub const MARGINFI_REPAY_DISCRIMINATOR: [u8; 8] = [0x85, 0x72, 0x1a, 0x5f, 0x9d, 0x4e, 0x23, 0x7c];
pub const MARGINFI_LIQUIDATE_DISCRIMINATOR: [u8; 8] =
    [0x6a, 0x8b, 0x47, 0x2e, 0x1c, 0x93, 0x5f, 0x4d];

/// MarginFi account types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarginFiAccountType {
    MarginfiGroup,
    MarginfiAccount,
    Bank,
    Unknown,
}

/// MarginFi bank configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiBankConfig {
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub oracle: Pubkey,
    pub bank_authority: Pubkey,
    pub collected_insurance_fees_outstanding: u64,
    pub fee_rate: u64,
    pub insurance_fee_rate: u64,
    pub insurance_vault: Pubkey,
    pub deposit_limit: u64,
    pub borrow_limit: u64,
    pub operational_state: u8,
    pub oracle_setup: u8,
    pub oracle_keys: [Pubkey; 5],
}

/// MarginFi bank state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiBankState {
    pub total_asset_shares: u128,
    pub total_liability_shares: u128,
    pub last_update: u64,
    pub lending_rate: u64,
    pub borrowing_rate: u64,
    pub asset_share_value: u128,
    pub liability_share_value: u128,
    pub liquidity_vault_authority: Pubkey,
    pub liquidity_vault_authority_bump: u8,
    pub insurance_vault_authority: Pubkey,
    pub insurance_vault_authority_bump: u8,
    pub collected_group_fees_outstanding: u64,
    pub fee_vault_authority: Pubkey,
    pub fee_vault_authority_bump: u8,
    pub fee_vault: Pubkey,
}

/// MarginFi account balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiBalance {
    pub active: bool,
    pub bank_pk: Pubkey,
    pub asset_shares: u128,
    pub liability_shares: u128,
    pub emissions_outstanding: u64,
    pub last_update: u64,
    pub padding: [u64; 1],
}

/// MarginFi user account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiAccount {
    pub group: Pubkey,
    pub authority: Pubkey,
    pub lending_account: MarginFiLendingAccount,
    pub account_flags: u64,
    pub padding: [u128; 8],
}

/// MarginFi lending account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiLendingAccount {
    pub balances: [MarginFiBalance; 16],
    pub padding: [u64; 8],
}

/// MarginFi deposit data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiDepositData {
    pub marginfi_group: Pubkey,
    pub marginfi_account: Pubkey,
    pub signer: Pubkey,
    pub bank: Pubkey,
    pub token_account: Pubkey,
    pub bank_liquidity_vault: Pubkey,
    pub token_program: Pubkey,
    pub amount: u64,
}

/// MarginFi withdraw data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiWithdrawData {
    pub marginfi_group: Pubkey,
    pub marginfi_account: Pubkey,
    pub signer: Pubkey,
    pub bank: Pubkey,
    pub token_account: Pubkey,
    pub bank_liquidity_vault: Pubkey,
    pub bank_liquidity_vault_authority: Pubkey,
    pub token_program: Pubkey,
    pub amount: u64,
    pub withdraw_all: bool,
}

/// MarginFi borrow data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiBorrowData {
    pub marginfi_group: Pubkey,
    pub marginfi_account: Pubkey,
    pub signer: Pubkey,
    pub bank: Pubkey,
    pub token_account: Pubkey,
    pub bank_liquidity_vault: Pubkey,
    pub bank_liquidity_vault_authority: Pubkey,
    pub token_program: Pubkey,
    pub amount: u64,
}

/// MarginFi repay data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiRepayData {
    pub marginfi_group: Pubkey,
    pub marginfi_account: Pubkey,
    pub signer: Pubkey,
    pub bank: Pubkey,
    pub token_account: Pubkey,
    pub bank_liquidity_vault: Pubkey,
    pub token_program: Pubkey,
    pub amount: u64,
    pub repay_all: bool,
}

/// MarginFi liquidation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiLiquidationData {
    pub marginfi_group: Pubkey,
    pub asset_bank: Pubkey,
    pub liab_bank: Pubkey,
    pub liquidatee_marginfi_account: Pubkey,
    pub liquidator_marginfi_account: Pubkey,
    pub liquidator: Pubkey,
    pub asset_bank_liquidity_vault: Pubkey,
    pub liab_bank_liquidity_vault: Pubkey,
    pub liquidator_token_account: Pubkey,
    pub token_program: Pubkey,
    pub asset_amount: u64,
    pub liab_amount: u64,
}

/// Extract MarginFi program ID as Pubkey
pub fn marginfi_program_id() -> Pubkey {
    Pubkey::try_from(MARGINFI_PROGRAM_ID).expect("Invalid MarginFi program ID")
}

/// Extract MarginFi bank program ID as Pubkey
pub fn marginfi_bank_program_id() -> Pubkey {
    Pubkey::try_from(MARGINFI_BANK_PROGRAM_ID).expect("Invalid MarginFi bank program ID")
}

/// Check if the given pubkey is MarginFi program
pub fn is_marginfi_program(program_id: &Pubkey) -> bool {
    *program_id == marginfi_program_id() || *program_id == marginfi_bank_program_id()
}

/// Calculate health ratio for a MarginFi account
pub fn calculate_health_ratio(
    total_asset_value: u128,
    total_liability_value: u128,
    maintenance_margin: u64,
) -> f64 {
    if total_liability_value == 0 {
        return f64::INFINITY;
    }

    let asset_value = total_asset_value as f64;
    let liability_value = total_liability_value as f64;
    let margin_factor = maintenance_margin as f64 / 10000.0; // Basis points to decimal

    (asset_value * margin_factor) / liability_value
}

/// Calculate liquidation threshold
pub fn calculate_liquidation_threshold(total_asset_value: u128, liquidation_ltv: u64) -> u128 {
    let ltv_factor = liquidation_ltv as u128;
    (total_asset_value * ltv_factor) / 10000 // Convert from basis points
}

/// Convert shares to amount using share value
pub fn shares_to_amount(shares: u128, share_value: u128) -> u64 {
    ((shares * share_value) / (1u128 << 64)) as u64
}

/// Convert amount to shares using share value
pub fn amount_to_shares(amount: u64, share_value: u128) -> u128 {
    if share_value == 0 {
        return amount as u128;
    }
    ((amount as u128) * (1u128 << 64)) / share_value
}

/// Calculate interest rate based on utilization
pub fn calculate_interest_rate(
    utilization_rate: u64,
    base_rate: u64,
    slope1: u64,
    slope2: u64,
    optimal_utilization: u64,
) -> u64 {
    if utilization_rate <= optimal_utilization {
        base_rate + (utilization_rate * slope1) / optimal_utilization
    } else {
        let excess_utilization = utilization_rate - optimal_utilization;
        let max_excess = 10000 - optimal_utilization; // 100% - optimal in basis points
        base_rate + slope1 + (excess_utilization * slope2) / max_excess
    }
}
