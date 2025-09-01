/// Jupiter protocol event definitions and constants.
pub mod events;
/// Jupiter protocol transaction and log parsing functionality.
pub mod parser;
/// Jupiter protocol data types and structures.
pub mod types;

pub use events::*;
pub use parser::*;
pub use types::*;

// Re-export core types
pub use crate::events::core::EventParameters;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::parser_types::ProtocolParser;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_module_exports_events() {
        // Test that event types are accessible through re-exports
        let _event = EventParameters::default();
        let _swap_event = JupiterSwapEvent::default();
        let _liquidity_event = JupiterLiquidityEvent::default();
        let _borsh_event = JupiterSwapBorshEvent::default();
    }

    #[test]
    fn test_module_exports_parser() {
        // Test that parser types are accessible through re-exports
        let parser = JupiterEventParser::default();
        assert!(parser.supported_program_ids().len() > 0);
    }

    #[test]
    fn test_module_exports_types() {
        // Test that type constants and functions are accessible through re-exports
        let program_id = jupiter_v6_program_id();
        assert!(is_jupiter_v6_program(&program_id));

        // Test discriminator constants are accessible
        let _route_disc = ROUTE_DISCRIMINATOR;
        let _exact_out_disc = EXACT_OUT_ROUTE_DISCRIMINATOR;
        let _legacy_route_disc = LEGACY_ROUTE_DISCRIMINATOR;
        let _legacy_exact_out_disc = LEGACY_EXACT_OUT_DISCRIMINATOR;
        let _swap_disc = SWAP_DISCRIMINATOR;
        let _token_ledger_disc = ROUTE_WITH_TOKEN_LEDGER_DISCRIMINATOR;

        // Test type structs are accessible
        let _swap_data = JupiterSwapData::default();
        let _route_plan = RoutePlan {
            input_mint: Pubkey::default(),
            output_mint: Pubkey::default(),
            amount_in: 0,
            amount_out: 0,
            dex_label: String::default(),
        };
        let _account_layout = JupiterAccountLayout {
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
    fn test_jupiter_program_id_constant() {
        // Test that the program ID constant is accessible and valid
        assert_eq!(
            JUPITER_V6_PROGRAM_ID,
            "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
        );
    }

    #[test]
    fn test_jupiter_v6_program_id_function() {
        // Test the jupiter_v6_program_id function
        let program_id = jupiter_v6_program_id();
        assert_eq!(program_id.to_string(), JUPITER_V6_PROGRAM_ID);
    }

    #[test]
    fn test_is_jupiter_v6_program_when_correct_program_id_should_return_true() {
        // Test with correct Jupiter V6 program ID
        let program_id = jupiter_v6_program_id();
        assert!(is_jupiter_v6_program(&program_id));
    }

    #[test]
    fn test_is_jupiter_v6_program_when_incorrect_program_id_should_return_false() {
        // Test with a different program ID
        let other_program_id = Pubkey::default();
        assert!(!is_jupiter_v6_program(&other_program_id));
    }

    #[test]
    fn test_discriminator_constants_have_correct_length() {
        // Test that all discriminator constants have the correct 8-byte length
        assert_eq!(ROUTE_DISCRIMINATOR.len(), 8);
        assert_eq!(EXACT_OUT_ROUTE_DISCRIMINATOR.len(), 8);
        assert_eq!(LEGACY_ROUTE_DISCRIMINATOR.len(), 8);
        assert_eq!(LEGACY_EXACT_OUT_DISCRIMINATOR.len(), 8);
        assert_eq!(SWAP_DISCRIMINATOR.len(), 8);
        assert_eq!(ROUTE_WITH_TOKEN_LEDGER_DISCRIMINATOR.len(), 8);
    }

    #[test]
    fn test_discriminator_constants_are_unique() {
        // Test that all discriminator constants are unique
        let discriminators = vec![
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
                        "Discriminators at indices {} and {} are identical",
                        i, j
                    );
                }
            }
        }
    }

    #[test]
    fn test_discriminator_constants_have_expected_values() {
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
