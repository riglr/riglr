use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// MarginFi program ID
pub const MARGINFI_PROGRAM_ID: &str = "MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA";

/// MarginFi Bank program ID (for lending pools)
pub const MARGINFI_BANK_PROGRAM_ID: &str = "4Be9aW2D8f3G2b3ZP8uo5kd9z8zwJ3FYD1tNKmQ9c9xA";

/// MarginFi instruction discriminators
/// Discriminator for lending deposit instruction
pub const MARGINFI_DEPOSIT_DISCRIMINATOR: [u8; 8] =
    [0x13, 0x65, 0x32, 0x1f, 0x7a, 0x43, 0x2a, 0x9f];
/// Discriminator for lending withdraw instruction
pub const MARGINFI_WITHDRAW_DISCRIMINATOR: [u8; 8] =
    [0x4c, 0x1c, 0x9b, 0x2d, 0xe3, 0x7a, 0x8b, 0x12];
/// Discriminator for lending borrow instruction
pub const MARGINFI_BORROW_DISCRIMINATOR: [u8; 8] = [0xa2, 0xfd, 0x67, 0xe3, 0x45, 0x1b, 0x8c, 0x9a];
/// Discriminator for lending repay instruction
pub const MARGINFI_REPAY_DISCRIMINATOR: [u8; 8] = [0x85, 0x72, 0x1a, 0x5f, 0x9d, 0x4e, 0x23, 0x7c];
/// Discriminator for liquidation instruction
pub const MARGINFI_LIQUIDATE_DISCRIMINATOR: [u8; 8] =
    [0x6a, 0x8b, 0x47, 0x2e, 0x1c, 0x93, 0x5f, 0x4d];

/// MarginFi account types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarginFiAccountType {
    /// MarginFi group account containing global settings
    MarginfiGroup,
    /// Individual user account for lending positions
    MarginfiAccount,
    /// Bank account representing a lending pool
    Bank,
    /// Unknown or unrecognized account type
    Unknown,
}

/// MarginFi bank configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiBankConfig {
    /// Bank account public key
    pub bank: Pubkey,
    /// Token mint for this bank
    pub mint: Pubkey,
    /// Liquidity vault holding deposited tokens
    pub vault: Pubkey,
    /// Price oracle for this token
    pub oracle: Pubkey,
    /// Authority that can modify bank settings
    pub bank_authority: Pubkey,
    /// Outstanding insurance fees collected
    pub collected_insurance_fees_outstanding: u64,
    /// Fee rate charged on operations
    pub fee_rate: u64,
    /// Insurance fee rate for risk coverage
    pub insurance_fee_rate: u64,
    /// Vault holding insurance funds
    pub insurance_vault: Pubkey,
    /// Maximum amount that can be deposited
    pub deposit_limit: u64,
    /// Maximum amount that can be borrowed
    pub borrow_limit: u64,
    /// Current operational state of the bank
    pub operational_state: u8,
    /// Oracle configuration setup
    pub oracle_setup: u8,
    /// Array of oracle public keys
    pub oracle_keys: [Pubkey; 5],
}

/// MarginFi bank state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiBankState {
    /// Total shares representing assets in the bank
    pub total_asset_shares: u128,
    /// Total shares representing liabilities in the bank
    pub total_liability_shares: u128,
    /// Timestamp of last state update
    pub last_update: u64,
    /// Current lending interest rate
    pub lending_rate: u64,
    /// Current borrowing interest rate
    pub borrowing_rate: u64,
    /// Value per asset share
    pub asset_share_value: u128,
    /// Value per liability share
    pub liability_share_value: u128,
    /// Authority for the liquidity vault
    pub liquidity_vault_authority: Pubkey,
    /// Bump seed for liquidity vault authority
    pub liquidity_vault_authority_bump: u8,
    /// Authority for the insurance vault
    pub insurance_vault_authority: Pubkey,
    /// Bump seed for insurance vault authority
    pub insurance_vault_authority_bump: u8,
    /// Outstanding group fees collected
    pub collected_group_fees_outstanding: u64,
    /// Authority for the fee vault
    pub fee_vault_authority: Pubkey,
    /// Bump seed for fee vault authority
    pub fee_vault_authority_bump: u8,
    /// Fee collection vault
    pub fee_vault: Pubkey,
}

/// MarginFi account balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiBalance {
    /// Whether this balance is active
    pub active: bool,
    /// Bank public key for this balance
    pub bank_pk: Pubkey,
    /// Shares representing deposited assets
    pub asset_shares: u128,
    /// Shares representing borrowed liabilities
    pub liability_shares: u128,
    /// Outstanding emission rewards
    pub emissions_outstanding: u64,
    /// Timestamp of last balance update
    pub last_update: u64,
    /// Padding for future use
    pub padding: [u64; 1],
}

/// MarginFi user account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiAccount {
    /// MarginFi group this account belongs to
    pub group: Pubkey,
    /// Authority that controls this account
    pub authority: Pubkey,
    /// Lending account with balance information
    pub lending_account: MarginFiLendingAccount,
    /// Account configuration flags
    pub account_flags: u64,
    /// Padding for future use
    pub padding: [u128; 8],
}

/// MarginFi lending account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginFiLendingAccount {
    /// Array of balances for different tokens
    pub balances: [MarginFiBalance; 16],
    /// Padding for future use
    pub padding: [u64; 8],
}

/// MarginFi deposit data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginFiDepositData {
    /// MarginFi group for the deposit
    pub marginfi_group: Pubkey,
    /// User's MarginFi account
    pub marginfi_account: Pubkey,
    /// Transaction signer
    pub signer: Pubkey,
    /// Bank receiving the deposit
    pub bank: Pubkey,
    /// User's token account being debited
    pub token_account: Pubkey,
    /// Bank's liquidity vault being credited
    pub bank_liquidity_vault: Pubkey,
    /// Token program ID
    pub token_program: Pubkey,
    /// Amount being deposited
    pub amount: u64,
}

/// MarginFi withdraw data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginFiWithdrawData {
    /// MarginFi group for the withdrawal
    pub marginfi_group: Pubkey,
    /// User's MarginFi account
    pub marginfi_account: Pubkey,
    /// Transaction signer
    pub signer: Pubkey,
    /// Bank from which tokens are withdrawn
    pub bank: Pubkey,
    /// User's token account being credited
    pub token_account: Pubkey,
    /// Bank's liquidity vault being debited
    pub bank_liquidity_vault: Pubkey,
    /// Authority for the bank's liquidity vault
    pub bank_liquidity_vault_authority: Pubkey,
    /// Token program ID
    pub token_program: Pubkey,
    /// Amount being withdrawn
    pub amount: u64,
    /// Whether to withdraw all available tokens
    pub withdraw_all: bool,
}

/// MarginFi borrow data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginFiBorrowData {
    /// MarginFi group for the borrow
    pub marginfi_group: Pubkey,
    /// User's MarginFi account
    pub marginfi_account: Pubkey,
    /// Transaction signer
    pub signer: Pubkey,
    /// Bank from which tokens are borrowed
    pub bank: Pubkey,
    /// User's token account being credited
    pub token_account: Pubkey,
    /// Bank's liquidity vault being debited
    pub bank_liquidity_vault: Pubkey,
    /// Authority for the bank's liquidity vault
    pub bank_liquidity_vault_authority: Pubkey,
    /// Token program ID
    pub token_program: Pubkey,
    /// Amount being borrowed
    pub amount: u64,
}

/// MarginFi repay data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginFiRepayData {
    /// MarginFi group for the repayment
    pub marginfi_group: Pubkey,
    /// User's MarginFi account
    pub marginfi_account: Pubkey,
    /// Transaction signer
    pub signer: Pubkey,
    /// Bank to which tokens are repaid
    pub bank: Pubkey,
    /// User's token account being debited
    pub token_account: Pubkey,
    /// Bank's liquidity vault being credited
    pub bank_liquidity_vault: Pubkey,
    /// Token program ID
    pub token_program: Pubkey,
    /// Amount being repaid
    pub amount: u64,
    /// Whether to repay all outstanding debt
    pub repay_all: bool,
}

/// MarginFi liquidation data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginFiLiquidationData {
    /// MarginFi group for the liquidation
    pub marginfi_group: Pubkey,
    /// Bank holding the asset being seized
    pub asset_bank: Pubkey,
    /// Bank holding the liability being repaid
    pub liab_bank: Pubkey,
    /// Account being liquidated
    pub liquidatee_marginfi_account: Pubkey,
    /// Liquidator's MarginFi account
    pub liquidator_marginfi_account: Pubkey,
    /// Liquidator's wallet address
    pub liquidator: Pubkey,
    /// Asset bank's liquidity vault
    pub asset_bank_liquidity_vault: Pubkey,
    /// Liability bank's liquidity vault
    pub liab_bank_liquidity_vault: Pubkey,
    /// Liquidator's token account
    pub liquidator_token_account: Pubkey,
    /// Token program ID
    pub token_program: Pubkey,
    /// Amount of asset being seized
    pub asset_amount: u64,
    /// Amount of liability being repaid
    pub liab_amount: u64,
}

/// Extract MarginFi program ID as Pubkey
pub fn marginfi_program_id() -> Pubkey {
    MARGINFI_PROGRAM_ID
        .parse()
        .expect("Invalid MarginFi program ID")
}

/// Extract MarginFi bank program ID as Pubkey
pub fn marginfi_bank_program_id() -> Pubkey {
    MARGINFI_BANK_PROGRAM_ID
        .parse()
        .expect("Invalid MarginFi bank program ID")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marginfi_program_id_when_valid_should_return_pubkey() {
        let program_id = marginfi_program_id();
        assert_eq!(program_id.to_string(), MARGINFI_PROGRAM_ID);
    }

    #[test]
    fn test_marginfi_bank_program_id_when_valid_should_return_pubkey() {
        let program_id = marginfi_bank_program_id();
        assert_eq!(program_id.to_string(), MARGINFI_BANK_PROGRAM_ID);
    }

    #[test]
    fn test_is_marginfi_program_when_main_program_id_should_return_true() {
        let program_id = marginfi_program_id();
        assert!(is_marginfi_program(&program_id));
    }

    #[test]
    fn test_is_marginfi_program_when_bank_program_id_should_return_true() {
        let program_id = marginfi_bank_program_id();
        assert!(is_marginfi_program(&program_id));
    }

    #[test]
    fn test_is_marginfi_program_when_other_program_id_should_return_false() {
        let other_program_id = Pubkey::new_unique();
        assert!(!is_marginfi_program(&other_program_id));
    }

    #[test]
    fn test_calculate_health_ratio_when_no_liabilities_should_return_infinity() {
        let ratio = calculate_health_ratio(100, 0, 8000);
        assert_eq!(ratio, f64::INFINITY);
    }

    #[test]
    fn test_calculate_health_ratio_when_has_liabilities_should_calculate_correctly() {
        let ratio = calculate_health_ratio(100, 50, 8000);
        let expected = (100.0 * 0.8) / 50.0; // 8000 basis points = 0.8
        assert_eq!(ratio, expected);
    }

    #[test]
    fn test_calculate_health_ratio_when_zero_assets_should_return_zero() {
        let ratio = calculate_health_ratio(0, 100, 8000);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_calculate_liquidation_threshold_when_normal_values_should_calculate_correctly() {
        let threshold = calculate_liquidation_threshold(10000, 8000);
        assert_eq!(threshold, 8000); // 10000 * 8000 / 10000
    }

    #[test]
    fn test_calculate_liquidation_threshold_when_zero_asset_value_should_return_zero() {
        let threshold = calculate_liquidation_threshold(0, 8000);
        assert_eq!(threshold, 0);
    }

    #[test]
    fn test_calculate_liquidation_threshold_when_zero_ltv_should_return_zero() {
        let threshold = calculate_liquidation_threshold(10000, 0);
        assert_eq!(threshold, 0);
    }

    #[test]
    fn test_shares_to_amount_when_normal_values_should_calculate_correctly() {
        let shares = 1u128 << 64; // 1 * 2^64
        let share_value = 1u128 << 64; // 1 * 2^64
        let amount = shares_to_amount(shares, share_value);
        assert_eq!(amount, 1);
    }

    #[test]
    fn test_shares_to_amount_when_zero_shares_should_return_zero() {
        let amount = shares_to_amount(0, 1u128 << 64);
        assert_eq!(amount, 0);
    }

    #[test]
    fn test_shares_to_amount_when_zero_share_value_should_return_zero() {
        let amount = shares_to_amount(100, 0);
        assert_eq!(amount, 0);
    }

    #[test]
    fn test_amount_to_shares_when_normal_values_should_calculate_correctly() {
        let amount = 100u64;
        let share_value = 1u128 << 64;
        let shares = amount_to_shares(amount, share_value);
        assert_eq!(shares, 100);
    }

    #[test]
    fn test_amount_to_shares_when_zero_amount_should_return_zero() {
        let shares = amount_to_shares(0, 1u128 << 64);
        assert_eq!(shares, 0);
    }

    #[test]
    fn test_amount_to_shares_when_zero_share_value_should_return_amount() {
        let shares = amount_to_shares(100, 0);
        assert_eq!(shares, 100);
    }

    #[test]
    fn test_calculate_interest_rate_when_utilization_below_optimal_should_use_slope1() {
        let rate = calculate_interest_rate(5000, 200, 400, 2000, 8000);
        let expected = 200 + (5000 * 400) / 8000;
        assert_eq!(rate, expected);
    }

    #[test]
    fn test_calculate_interest_rate_when_utilization_equals_optimal_should_use_slope1() {
        let rate = calculate_interest_rate(8000, 200, 400, 2000, 8000);
        let expected = 200 + (8000 * 400) / 8000;
        assert_eq!(rate, expected);
    }

    #[test]
    fn test_calculate_interest_rate_when_utilization_above_optimal_should_use_slope2() {
        let rate = calculate_interest_rate(9000, 200, 400, 2000, 8000);
        let excess_utilization = 9000 - 8000;
        let max_excess = 10000 - 8000;
        let expected = 200 + 400 + (excess_utilization * 2000) / max_excess;
        assert_eq!(rate, expected);
    }

    #[test]
    fn test_calculate_interest_rate_when_zero_utilization_should_return_base_rate() {
        let rate = calculate_interest_rate(0, 200, 400, 2000, 8000);
        assert_eq!(rate, 200);
    }

    #[test]
    fn test_calculate_interest_rate_when_max_utilization_should_handle_correctly() {
        let rate = calculate_interest_rate(10000, 200, 400, 2000, 8000);
        let excess_utilization = 10000 - 8000;
        let max_excess = 10000 - 8000;
        let expected = 200 + 400 + (excess_utilization * 2000) / max_excess;
        assert_eq!(rate, expected);
    }

    #[test]
    fn test_marginfi_account_type_enum_variants_should_serialize_deserialize() {
        use serde_json;

        let group = MarginFiAccountType::MarginfiGroup;
        let serialized = serde_json::to_string(&group).unwrap();
        let deserialized: MarginFiAccountType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(group, deserialized);

        let account = MarginFiAccountType::MarginfiAccount;
        let serialized = serde_json::to_string(&account).unwrap();
        let deserialized: MarginFiAccountType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(account, deserialized);

        let bank = MarginFiAccountType::Bank;
        let serialized = serde_json::to_string(&bank).unwrap();
        let deserialized: MarginFiAccountType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(bank, deserialized);

        let unknown = MarginFiAccountType::Unknown;
        let serialized = serde_json::to_string(&unknown).unwrap();
        let deserialized: MarginFiAccountType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(unknown, deserialized);
    }

    #[test]
    fn test_marginfi_bank_config_should_serialize_deserialize() {
        use serde_json;

        let config = MarginFiBankConfig {
            bank: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            vault: Pubkey::new_unique(),
            oracle: Pubkey::new_unique(),
            bank_authority: Pubkey::new_unique(),
            collected_insurance_fees_outstanding: 1000,
            fee_rate: 500,
            insurance_fee_rate: 100,
            insurance_vault: Pubkey::new_unique(),
            deposit_limit: 1000000,
            borrow_limit: 800000,
            operational_state: 1,
            oracle_setup: 2,
            oracle_keys: [Pubkey::new_unique(); 5],
        };

        let serialized = serde_json::to_string(&config).unwrap();
        let _deserialized: MarginFiBankConfig = serde_json::from_str(&serialized).unwrap();
    }

    #[test]
    fn test_marginfi_deposit_data_default_should_have_zero_values() {
        let deposit_data = MarginFiDepositData::default();
        assert_eq!(deposit_data.amount, 0);
        assert_eq!(deposit_data.marginfi_group, Pubkey::default());
    }

    #[test]
    fn test_marginfi_withdraw_data_default_should_have_zero_values() {
        let withdraw_data = MarginFiWithdrawData::default();
        assert_eq!(withdraw_data.amount, 0);
        assert!(!withdraw_data.withdraw_all);
        assert_eq!(withdraw_data.marginfi_group, Pubkey::default());
    }

    #[test]
    fn test_marginfi_borrow_data_default_should_have_zero_values() {
        let borrow_data = MarginFiBorrowData::default();
        assert_eq!(borrow_data.amount, 0);
        assert_eq!(borrow_data.marginfi_group, Pubkey::default());
    }

    #[test]
    fn test_marginfi_repay_data_default_should_have_zero_values() {
        let repay_data = MarginFiRepayData::default();
        assert_eq!(repay_data.amount, 0);
        assert!(!repay_data.repay_all);
        assert_eq!(repay_data.marginfi_group, Pubkey::default());
    }

    #[test]
    fn test_marginfi_liquidation_data_default_should_have_zero_values() {
        let liquidation_data = MarginFiLiquidationData::default();
        assert_eq!(liquidation_data.asset_amount, 0);
        assert_eq!(liquidation_data.liab_amount, 0);
        assert_eq!(liquidation_data.marginfi_group, Pubkey::default());
    }

    #[test]
    fn test_marginfi_balance_should_serialize_deserialize() {
        use serde_json;

        let balance = MarginFiBalance {
            active: true,
            bank_pk: Pubkey::new_unique(),
            asset_shares: 1000,
            liability_shares: 500,
            emissions_outstanding: 100,
            last_update: 1234567890,
            padding: [0],
        };

        let serialized = serde_json::to_string(&balance).unwrap();
        let _deserialized: MarginFiBalance = serde_json::from_str(&serialized).unwrap();
    }

    #[test]
    fn test_constants_should_have_correct_values() {
        assert_eq!(
            MARGINFI_PROGRAM_ID,
            "MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA"
        );
        assert_eq!(
            MARGINFI_BANK_PROGRAM_ID,
            "4Be9aW2D8f3G2b3ZP8uo5kd9z8zwJ3FYD1tNKmQ9c9xA"
        );

        assert_eq!(
            MARGINFI_DEPOSIT_DISCRIMINATOR,
            [0x13, 0x65, 0x32, 0x1f, 0x7a, 0x43, 0x2a, 0x9f]
        );
        assert_eq!(
            MARGINFI_WITHDRAW_DISCRIMINATOR,
            [0x4c, 0x1c, 0x9b, 0x2d, 0xe3, 0x7a, 0x8b, 0x12]
        );
        assert_eq!(
            MARGINFI_BORROW_DISCRIMINATOR,
            [0xa2, 0xfd, 0x67, 0xe3, 0x45, 0x1b, 0x8c, 0x9a]
        );
        assert_eq!(
            MARGINFI_REPAY_DISCRIMINATOR,
            [0x85, 0x72, 0x1a, 0x5f, 0x9d, 0x4e, 0x23, 0x7c]
        );
        assert_eq!(
            MARGINFI_LIQUIDATE_DISCRIMINATOR,
            [0x6a, 0x8b, 0x47, 0x2e, 0x1c, 0x93, 0x5f, 0x4d]
        );
    }
}
