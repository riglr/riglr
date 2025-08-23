//! Raydium AMM V4 instruction discriminators

/// Instruction discriminator for swapping with base token as input
pub const SWAP_BASE_IN: &[u8] = &[0x8f, 0x9a, 0x14, 0xdf, 0x90, 0x38, 0x15, 0xe5];

/// Instruction discriminator for swapping with base token as output
pub const SWAP_BASE_OUT: &[u8] = &[0xab, 0x69, 0x6b, 0xc3, 0xb2, 0x02, 0xee, 0x55];

/// Instruction discriminator for depositing liquidity into the pool
pub const DEPOSIT: &[u8] = &[0x3e, 0xc2, 0xf7, 0x7f, 0x68, 0x0e, 0xc1, 0x0d];

/// Instruction discriminator for initializing the AMM pool (version 2)
pub const INITIALIZE2: &[u8] = &[0xa3, 0xa5, 0xba, 0xcd, 0xeb, 0xc8, 0xd4, 0xe2];

/// Instruction discriminator for withdrawing liquidity from the pool
pub const WITHDRAW: &[u8] = &[0xb7, 0x12, 0x46, 0x9c, 0x94, 0x37, 0xa0, 0xf4];

/// Instruction discriminator for withdrawing profit and loss from the pool
pub const WITHDRAW_PNL: &[u8] = &[0xd6, 0x8f, 0x37, 0x9a, 0x1f, 0xe1, 0x28, 0x52];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_base_in_discriminator_has_correct_bytes() {
        assert_eq!(
            SWAP_BASE_IN,
            &[0x8f, 0x9a, 0x14, 0xdf, 0x90, 0x38, 0x15, 0xe5]
        );
    }

    #[test]
    fn test_swap_base_in_discriminator_has_correct_length() {
        assert_eq!(SWAP_BASE_IN.len(), 8);
    }

    #[test]
    fn test_swap_base_out_discriminator_has_correct_bytes() {
        assert_eq!(
            SWAP_BASE_OUT,
            &[0xab, 0x69, 0x6b, 0xc3, 0xb2, 0x02, 0xee, 0x55]
        );
    }

    #[test]
    fn test_swap_base_out_discriminator_has_correct_length() {
        assert_eq!(SWAP_BASE_OUT.len(), 8);
    }

    #[test]
    fn test_deposit_discriminator_has_correct_bytes() {
        assert_eq!(DEPOSIT, &[0x3e, 0xc2, 0xf7, 0x7f, 0x68, 0x0e, 0xc1, 0x0d]);
    }

    #[test]
    fn test_deposit_discriminator_has_correct_length() {
        assert_eq!(DEPOSIT.len(), 8);
    }

    #[test]
    fn test_initialize2_discriminator_has_correct_bytes() {
        assert_eq!(
            INITIALIZE2,
            &[0xa3, 0xa5, 0xba, 0xcd, 0xeb, 0xc8, 0xd4, 0xe2]
        );
    }

    #[test]
    fn test_initialize2_discriminator_has_correct_length() {
        assert_eq!(INITIALIZE2.len(), 8);
    }

    #[test]
    fn test_withdraw_discriminator_has_correct_bytes() {
        assert_eq!(WITHDRAW, &[0xb7, 0x12, 0x46, 0x9c, 0x94, 0x37, 0xa0, 0xf4]);
    }

    #[test]
    fn test_withdraw_discriminator_has_correct_length() {
        assert_eq!(WITHDRAW.len(), 8);
    }

    #[test]
    fn test_withdraw_pnl_discriminator_has_correct_bytes() {
        assert_eq!(
            WITHDRAW_PNL,
            &[0xd6, 0x8f, 0x37, 0x9a, 0x1f, 0xe1, 0x28, 0x52]
        );
    }

    #[test]
    fn test_withdraw_pnl_discriminator_has_correct_length() {
        assert_eq!(WITHDRAW_PNL.len(), 8);
    }

    #[test]
    fn test_all_discriminators_are_unique() {
        let discriminators = [
            SWAP_BASE_IN,
            SWAP_BASE_OUT,
            DEPOSIT,
            INITIALIZE2,
            WITHDRAW,
            WITHDRAW_PNL,
        ];

        // Test that each discriminator is unique compared to all others
        for (i, disc1) in discriminators.iter().enumerate() {
            for (j, disc2) in discriminators.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        disc1, disc2,
                        "Discriminators at indices {} and {} should be unique",
                        i, j
                    );
                }
            }
        }
    }

    #[test]
    fn test_discriminators_are_not_empty() {
        assert!(!SWAP_BASE_IN.is_empty());
        assert!(!SWAP_BASE_OUT.is_empty());
        assert!(!DEPOSIT.is_empty());
        assert!(!INITIALIZE2.is_empty());
        assert!(!WITHDRAW.is_empty());
        assert!(!WITHDRAW_PNL.is_empty());
    }

    #[test]
    fn test_discriminators_contain_expected_first_bytes() {
        assert_eq!(SWAP_BASE_IN[0], 0x8f);
        assert_eq!(SWAP_BASE_OUT[0], 0xab);
        assert_eq!(DEPOSIT[0], 0x3e);
        assert_eq!(INITIALIZE2[0], 0xa3);
        assert_eq!(WITHDRAW[0], 0xb7);
        assert_eq!(WITHDRAW_PNL[0], 0xd6);
    }

    #[test]
    fn test_discriminators_contain_expected_last_bytes() {
        assert_eq!(SWAP_BASE_IN[7], 0xe5);
        assert_eq!(SWAP_BASE_OUT[7], 0x55);
        assert_eq!(DEPOSIT[7], 0x0d);
        assert_eq!(INITIALIZE2[7], 0xe2);
        assert_eq!(WITHDRAW[7], 0xf4);
        assert_eq!(WITHDRAW_PNL[7], 0x52);
    }

    #[test]
    fn test_discriminators_as_slices_equality() {
        // Test slice comparison works as expected
        let swap_base_in_slice: &[u8] = SWAP_BASE_IN;
        assert_eq!(
            swap_base_in_slice,
            &[0x8f, 0x9a, 0x14, 0xdf, 0x90, 0x38, 0x15, 0xe5]
        );

        let swap_base_out_slice: &[u8] = SWAP_BASE_OUT;
        assert_eq!(
            swap_base_out_slice,
            &[0xab, 0x69, 0x6b, 0xc3, 0xb2, 0x02, 0xee, 0x55]
        );
    }

    #[test]
    fn test_discriminators_can_be_used_in_match_patterns() {
        fn identify_instruction(discriminator: &[u8]) -> &'static str {
            match discriminator {
                SWAP_BASE_IN => "swap_base_in",
                SWAP_BASE_OUT => "swap_base_out",
                DEPOSIT => "deposit",
                INITIALIZE2 => "initialize2",
                WITHDRAW => "withdraw",
                WITHDRAW_PNL => "withdraw_pnl",
                _ => "unknown",
            }
        }

        assert_eq!(identify_instruction(SWAP_BASE_IN), "swap_base_in");
        assert_eq!(identify_instruction(SWAP_BASE_OUT), "swap_base_out");
        assert_eq!(identify_instruction(DEPOSIT), "deposit");
        assert_eq!(identify_instruction(INITIALIZE2), "initialize2");
        assert_eq!(identify_instruction(WITHDRAW), "withdraw");
        assert_eq!(identify_instruction(WITHDRAW_PNL), "withdraw_pnl");
        assert_eq!(
            identify_instruction(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            "unknown"
        );
    }
}
