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

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    #[test]
    fn test_jupiter_v6_program_id_when_called_should_return_valid_pubkey() {
        let program_id = jupiter_v6_program_id();
        let expected = Pubkey::from_str(JUPITER_V6_PROGRAM_ID).unwrap();
        assert_eq!(program_id, expected);
    }

    #[test]
    fn test_is_jupiter_v6_program_when_matching_program_should_return_true() {
        let jupiter_id = jupiter_v6_program_id();
        assert!(is_jupiter_v6_program(&jupiter_id));
    }

    #[test]
    fn test_is_jupiter_v6_program_when_different_program_should_return_false() {
        let different_id = Pubkey::new_unique();
        assert!(!is_jupiter_v6_program(&different_id));
    }

    #[test]
    fn test_shared_accounts_route_data_when_borsh_serialize_deserialize_should_succeed() {
        let test_pubkey = Pubkey::new_unique();
        let swap_info = SwapInfo {
            source_token: test_pubkey,
            destination_token: test_pubkey,
            source_token_account: test_pubkey,
            destination_token_account: test_pubkey,
            swap_program_id: test_pubkey,
            swap_accounts: vec![AccountMeta {
                pubkey: test_pubkey,
                is_signer: true,
                is_writable: false,
            }],
            swap_data: vec![1, 2, 3],
        };

        let route_step = RoutePlanStep {
            swap: swap_info,
            percent: 50,
        };

        let original = SharedAccountsRouteData {
            route_plan: vec![route_step],
            in_amount: 1000000,
            quoted_out_amount: 2000000,
            slippage_bps: 100,
            platform_fee_bps: 10,
        };

        let serialized = borsh::to_vec(&original).unwrap();
        let deserialized: SharedAccountsRouteData = borsh::from_slice(&serialized).unwrap();

        assert_eq!(original.in_amount, deserialized.in_amount);
        assert_eq!(original.quoted_out_amount, deserialized.quoted_out_amount);
        assert_eq!(original.slippage_bps, deserialized.slippage_bps);
        assert_eq!(original.platform_fee_bps, deserialized.platform_fee_bps);
        assert_eq!(original.route_plan.len(), deserialized.route_plan.len());
    }

    #[test]
    fn test_shared_accounts_exact_out_route_data_when_borsh_serialize_deserialize_should_succeed() {
        let test_pubkey = Pubkey::new_unique();
        let swap_info = SwapInfo {
            source_token: test_pubkey,
            destination_token: test_pubkey,
            source_token_account: test_pubkey,
            destination_token_account: test_pubkey,
            swap_program_id: test_pubkey,
            swap_accounts: vec![],
            swap_data: vec![],
        };

        let route_step = RoutePlanStep {
            swap: swap_info,
            percent: 100,
        };

        let original = SharedAccountsExactOutRouteData {
            route_plan: vec![route_step],
            out_amount: 500000,
            quoted_in_amount: 1000000,
            slippage_bps: 200,
            platform_fee_bps: 5,
        };

        let serialized = borsh::to_vec(&original).unwrap();
        let deserialized: SharedAccountsExactOutRouteData = borsh::from_slice(&serialized).unwrap();

        assert_eq!(original.out_amount, deserialized.out_amount);
        assert_eq!(original.quoted_in_amount, deserialized.quoted_in_amount);
        assert_eq!(original.slippage_bps, deserialized.slippage_bps);
        assert_eq!(original.platform_fee_bps, deserialized.platform_fee_bps);
    }

    #[test]
    fn test_route_plan_step_when_borsh_serialize_deserialize_should_succeed() {
        let test_pubkey = Pubkey::new_unique();
        let swap_info = SwapInfo {
            source_token: test_pubkey,
            destination_token: test_pubkey,
            source_token_account: test_pubkey,
            destination_token_account: test_pubkey,
            swap_program_id: test_pubkey,
            swap_accounts: vec![
                AccountMeta {
                    pubkey: test_pubkey,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: test_pubkey,
                    is_signer: true,
                    is_writable: true,
                },
            ],
            swap_data: vec![4, 5, 6, 7],
        };

        let original = RoutePlanStep {
            swap: swap_info,
            percent: 75,
        };

        let serialized = borsh::to_vec(&original).unwrap();
        let deserialized: RoutePlanStep = borsh::from_slice(&serialized).unwrap();

        assert_eq!(original.percent, deserialized.percent);
        assert_eq!(original.swap.source_token, deserialized.swap.source_token);
        assert_eq!(
            original.swap.swap_accounts.len(),
            deserialized.swap.swap_accounts.len()
        );
    }

    #[test]
    fn test_swap_info_when_borsh_serialize_deserialize_should_succeed() {
        let test_pubkey = Pubkey::new_unique();
        let original = SwapInfo {
            source_token: test_pubkey,
            destination_token: test_pubkey,
            source_token_account: test_pubkey,
            destination_token_account: test_pubkey,
            swap_program_id: test_pubkey,
            swap_accounts: vec![],
            swap_data: vec![],
        };

        let serialized = borsh::to_vec(&original).unwrap();
        let deserialized: SwapInfo = borsh::from_slice(&serialized).unwrap();

        assert_eq!(original.source_token, deserialized.source_token);
        assert_eq!(original.destination_token, deserialized.destination_token);
        assert_eq!(
            original.source_token_account,
            deserialized.source_token_account
        );
        assert_eq!(
            original.destination_token_account,
            deserialized.destination_token_account
        );
        assert_eq!(original.swap_program_id, deserialized.swap_program_id);
    }

    #[test]
    fn test_account_meta_when_borsh_serialize_deserialize_should_succeed() {
        let test_pubkey = Pubkey::new_unique();
        let original = AccountMeta {
            pubkey: test_pubkey,
            is_signer: true,
            is_writable: false,
        };

        let serialized = borsh::to_vec(&original).unwrap();
        let deserialized: AccountMeta = borsh::from_slice(&serialized).unwrap();

        assert_eq!(original.pubkey, deserialized.pubkey);
        assert_eq!(original.is_signer, deserialized.is_signer);
        assert_eq!(original.is_writable, deserialized.is_writable);
    }

    #[test]
    fn test_account_meta_when_not_signer_and_writable_should_serialize_correctly() {
        let test_pubkey = Pubkey::new_unique();
        let original = AccountMeta {
            pubkey: test_pubkey,
            is_signer: false,
            is_writable: true,
        };

        let serialized = borsh::to_vec(&original).unwrap();
        let deserialized: AccountMeta = borsh::from_slice(&serialized).unwrap();

        assert_eq!(original.pubkey, deserialized.pubkey);
        assert!(!deserialized.is_signer);
        assert!(deserialized.is_writable);
    }

    #[test]
    fn test_jupiter_swap_data_when_default_should_have_expected_values() {
        let default_data = JupiterSwapData::default();

        assert_eq!(default_data.user, Pubkey::default());
        assert_eq!(default_data.input_mint, Pubkey::default());
        assert_eq!(default_data.output_mint, Pubkey::default());
        assert_eq!(default_data.input_amount, 0);
        assert_eq!(default_data.output_amount, 0);
        assert_eq!(default_data.price_impact_pct, None);
        assert_eq!(default_data.platform_fee_bps, None);
        assert_eq!(default_data.route_plan.len(), 0);
    }

    #[test]
    fn test_jupiter_swap_data_when_serde_serialize_deserialize_should_succeed() {
        let test_pubkey = Pubkey::new_unique();
        let route_plan = RoutePlan {
            input_mint: test_pubkey,
            output_mint: test_pubkey,
            amount_in: 1000,
            amount_out: 2000,
            dex_label: "Orca".to_string(),
        };

        let original = JupiterSwapData {
            user: test_pubkey,
            input_mint: test_pubkey,
            output_mint: test_pubkey,
            input_amount: 1000000,
            output_amount: 2000000,
            price_impact_pct: Some("0.5".to_string()),
            platform_fee_bps: Some(25),
            route_plan: vec![route_plan],
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: JupiterSwapData = serde_json::from_str(&json).unwrap();

        assert_eq!(original.user, deserialized.user);
        assert_eq!(original.input_amount, deserialized.input_amount);
        assert_eq!(original.output_amount, deserialized.output_amount);
        assert_eq!(original.price_impact_pct, deserialized.price_impact_pct);
        assert_eq!(original.platform_fee_bps, deserialized.platform_fee_bps);
        assert_eq!(original.route_plan.len(), deserialized.route_plan.len());
    }

    #[test]
    fn test_jupiter_swap_data_when_none_optional_fields_should_serialize_correctly() {
        let test_pubkey = Pubkey::new_unique();
        let original = JupiterSwapData {
            user: test_pubkey,
            input_mint: test_pubkey,
            output_mint: test_pubkey,
            input_amount: 500000,
            output_amount: 1000000,
            price_impact_pct: None,
            platform_fee_bps: None,
            route_plan: vec![],
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: JupiterSwapData = serde_json::from_str(&json).unwrap();

        assert_eq!(original.user, deserialized.user);
        assert_eq!(original.price_impact_pct, None);
        assert_eq!(original.platform_fee_bps, None);
        assert_eq!(original.route_plan.len(), 0);
    }

    #[test]
    fn test_route_plan_when_serde_serialize_deserialize_should_succeed() {
        let test_pubkey = Pubkey::new_unique();
        let original = RoutePlan {
            input_mint: test_pubkey,
            output_mint: test_pubkey,
            amount_in: 1500000,
            amount_out: 3000000,
            dex_label: "Raydium".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: RoutePlan = serde_json::from_str(&json).unwrap();

        assert_eq!(original.input_mint, deserialized.input_mint);
        assert_eq!(original.output_mint, deserialized.output_mint);
        assert_eq!(original.amount_in, deserialized.amount_in);
        assert_eq!(original.amount_out, deserialized.amount_out);
        assert_eq!(original.dex_label, deserialized.dex_label);
    }

    #[test]
    fn test_route_plan_when_empty_dex_label_should_serialize_correctly() {
        let test_pubkey = Pubkey::new_unique();
        let original = RoutePlan {
            input_mint: test_pubkey,
            output_mint: test_pubkey,
            amount_in: 0,
            amount_out: 0,
            dex_label: "".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: RoutePlan = serde_json::from_str(&json).unwrap();

        assert_eq!(original.dex_label, "");
        assert_eq!(deserialized.dex_label, "");
    }

    #[test]
    fn test_jupiter_account_layout_when_serde_serialize_deserialize_should_succeed() {
        let test_pubkey = Pubkey::new_unique();
        let original = JupiterAccountLayout {
            user_transfer_authority: test_pubkey,
            user_source_token_account: test_pubkey,
            user_destination_token_account: test_pubkey,
            destination_token_account: test_pubkey,
            source_mint: test_pubkey,
            destination_mint: test_pubkey,
            platform_fee_account: Some(test_pubkey),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: JupiterAccountLayout = serde_json::from_str(&json).unwrap();

        assert_eq!(
            original.user_transfer_authority,
            deserialized.user_transfer_authority
        );
        assert_eq!(
            original.user_source_token_account,
            deserialized.user_source_token_account
        );
        assert_eq!(
            original.user_destination_token_account,
            deserialized.user_destination_token_account
        );
        assert_eq!(
            original.destination_token_account,
            deserialized.destination_token_account
        );
        assert_eq!(original.source_mint, deserialized.source_mint);
        assert_eq!(original.destination_mint, deserialized.destination_mint);
        assert_eq!(
            original.platform_fee_account,
            deserialized.platform_fee_account
        );
    }

    #[test]
    fn test_jupiter_account_layout_when_none_platform_fee_account_should_serialize_correctly() {
        let test_pubkey = Pubkey::new_unique();
        let original = JupiterAccountLayout {
            user_transfer_authority: test_pubkey,
            user_source_token_account: test_pubkey,
            user_destination_token_account: test_pubkey,
            destination_token_account: test_pubkey,
            source_mint: test_pubkey,
            destination_mint: test_pubkey,
            platform_fee_account: None,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: JupiterAccountLayout = serde_json::from_str(&json).unwrap();

        assert_eq!(original.platform_fee_account, None);
        assert_eq!(deserialized.platform_fee_account, None);
    }

    #[test]
    fn test_discriminator_constants_should_have_correct_values() {
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

    #[test]
    fn test_jupiter_v6_program_id_constant_should_be_valid() {
        assert_eq!(
            JUPITER_V6_PROGRAM_ID,
            "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
        );
        // Verify it can be parsed as a valid Pubkey
        assert!(Pubkey::from_str(JUPITER_V6_PROGRAM_ID).is_ok());
    }

    #[test]
    fn test_swap_info_when_empty_swap_accounts_and_data_should_work() {
        let test_pubkey = Pubkey::new_unique();
        let swap_info = SwapInfo {
            source_token: test_pubkey,
            destination_token: test_pubkey,
            source_token_account: test_pubkey,
            destination_token_account: test_pubkey,
            swap_program_id: test_pubkey,
            swap_accounts: vec![],
            swap_data: vec![],
        };

        let serialized = borsh::to_vec(&swap_info).unwrap();
        let deserialized: SwapInfo = borsh::from_slice(&serialized).unwrap();

        assert_eq!(deserialized.swap_accounts.len(), 0);
        assert_eq!(deserialized.swap_data.len(), 0);
    }

    #[test]
    fn test_swap_info_when_large_swap_data_should_serialize_correctly() {
        let test_pubkey = Pubkey::new_unique();
        let large_data = vec![255u8; 1000]; // Large data array

        let swap_info = SwapInfo {
            source_token: test_pubkey,
            destination_token: test_pubkey,
            source_token_account: test_pubkey,
            destination_token_account: test_pubkey,
            swap_program_id: test_pubkey,
            swap_accounts: vec![],
            swap_data: large_data.clone(),
        };

        let serialized = borsh::to_vec(&swap_info).unwrap();
        let deserialized: SwapInfo = borsh::from_slice(&serialized).unwrap();

        assert_eq!(deserialized.swap_data, large_data);
        assert_eq!(deserialized.swap_data.len(), 1000);
    }

    #[test]
    fn test_shared_accounts_route_data_when_empty_route_plan_should_work() {
        let original = SharedAccountsRouteData {
            route_plan: vec![],
            in_amount: 0,
            quoted_out_amount: 0,
            slippage_bps: 0,
            platform_fee_bps: 0,
        };

        let serialized = borsh::to_vec(&original).unwrap();
        let deserialized: SharedAccountsRouteData = borsh::from_slice(&serialized).unwrap();

        assert_eq!(deserialized.route_plan.len(), 0);
        assert_eq!(deserialized.in_amount, 0);
        assert_eq!(deserialized.quoted_out_amount, 0);
        assert_eq!(deserialized.slippage_bps, 0);
        assert_eq!(deserialized.platform_fee_bps, 0);
    }

    #[test]
    fn test_shared_accounts_route_data_when_max_values_should_serialize_correctly() {
        let test_pubkey = Pubkey::new_unique();
        let swap_info = SwapInfo {
            source_token: test_pubkey,
            destination_token: test_pubkey,
            source_token_account: test_pubkey,
            destination_token_account: test_pubkey,
            swap_program_id: test_pubkey,
            swap_accounts: vec![],
            swap_data: vec![],
        };

        let route_step = RoutePlanStep {
            swap: swap_info,
            percent: 255, // max u8
        };

        let original = SharedAccountsRouteData {
            route_plan: vec![route_step],
            in_amount: u64::MAX,
            quoted_out_amount: u64::MAX,
            slippage_bps: u16::MAX,
            platform_fee_bps: u8::MAX,
        };

        let serialized = borsh::to_vec(&original).unwrap();
        let deserialized: SharedAccountsRouteData = borsh::from_slice(&serialized).unwrap();

        assert_eq!(deserialized.in_amount, u64::MAX);
        assert_eq!(deserialized.quoted_out_amount, u64::MAX);
        assert_eq!(deserialized.slippage_bps, u16::MAX);
        assert_eq!(deserialized.platform_fee_bps, u8::MAX);
        assert_eq!(deserialized.route_plan[0].percent, 255);
    }
}
