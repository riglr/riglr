//! Raydium CLMM instruction discriminators

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_discriminator_when_accessed_should_return_correct_bytes() {
        assert_eq!(SWAP, &[0xa9, 0x0d, 0xd0, 0xfe, 0x89, 0xbc, 0xab, 0x27]);
        assert_eq!(SWAP.len(), 8);
    }

    #[test]
    fn test_swap_v2_discriminator_when_accessed_should_return_correct_bytes() {
        assert_eq!(SWAP_V2, &[0x2a, 0x2d, 0x80, 0xb5, 0xce, 0x24, 0x7b, 0x87]);
        assert_eq!(SWAP_V2.len(), 8);
    }

    #[test]
    fn test_close_position_discriminator_when_accessed_should_return_correct_bytes() {
        assert_eq!(
            CLOSE_POSITION,
            &[0x7b, 0x86, 0x51, 0x10, 0x31, 0xc0, 0xa1, 0x7a]
        );
        assert_eq!(CLOSE_POSITION.len(), 8);
    }

    #[test]
    fn test_decrease_liquidity_v2_discriminator_when_accessed_should_return_correct_bytes() {
        assert_eq!(
            DECREASE_LIQUIDITY_V2,
            &[0x58, 0x12, 0x7a, 0x1a, 0x95, 0x04, 0xac, 0xa0]
        );
        assert_eq!(DECREASE_LIQUIDITY_V2.len(), 8);
    }

    #[test]
    fn test_create_pool_discriminator_when_accessed_should_return_correct_bytes() {
        assert_eq!(
            CREATE_POOL,
            &[0xe2, 0x58, 0x01, 0x5f, 0xc2, 0xc2, 0x49, 0xe9]
        );
        assert_eq!(CREATE_POOL.len(), 8);
    }

    #[test]
    fn test_increase_liquidity_v2_discriminator_when_accessed_should_return_correct_bytes() {
        assert_eq!(
            INCREASE_LIQUIDITY_V2,
            &[0x85, 0x15, 0x1a, 0xa4, 0xd1, 0x8b, 0x74, 0x2e]
        );
        assert_eq!(INCREASE_LIQUIDITY_V2.len(), 8);
    }

    #[test]
    fn test_open_position_with_token_22_nft_discriminator_when_accessed_should_return_correct_bytes(
    ) {
        assert_eq!(
            OPEN_POSITION_WITH_TOKEN_22_NFT,
            &[0x3e, 0xf4, 0xcc, 0x1f, 0x66, 0x42, 0xee, 0xd1]
        );
        assert_eq!(OPEN_POSITION_WITH_TOKEN_22_NFT.len(), 8);
    }

    #[test]
    fn test_open_position_v2_discriminator_when_accessed_should_return_correct_bytes() {
        assert_eq!(
            OPEN_POSITION_V2,
            &[0x4e, 0x14, 0xbb, 0x8b, 0xdd, 0xa8, 0xfc, 0x07]
        );
        assert_eq!(OPEN_POSITION_V2.len(), 8);
    }

    #[test]
    fn test_all_discriminators_when_compared_should_be_unique() {
        let discriminators = vec![
            SWAP,
            SWAP_V2,
            CLOSE_POSITION,
            DECREASE_LIQUIDITY_V2,
            CREATE_POOL,
            INCREASE_LIQUIDITY_V2,
            OPEN_POSITION_WITH_TOKEN_22_NFT,
            OPEN_POSITION_V2,
        ];

        // Test that all discriminators are unique
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
    fn test_discriminators_when_sliced_should_maintain_values() {
        // Test that discriminators can be sliced and maintain their values
        assert_eq!(&SWAP[0..4], &[0xa9, 0x0d, 0xd0, 0xfe]);
        assert_eq!(&SWAP[4..8], &[0x89, 0xbc, 0xab, 0x27]);

        assert_eq!(&SWAP_V2[0..4], &[0x2a, 0x2d, 0x80, 0xb5]);
        assert_eq!(&SWAP_V2[4..8], &[0xce, 0x24, 0x7b, 0x87]);
    }

    #[test]
    fn test_discriminators_when_converted_to_vec_should_match_original() {
        // Test conversion to Vec maintains the same values
        assert_eq!(
            SWAP.to_vec(),
            vec![0xa9, 0x0d, 0xd0, 0xfe, 0x89, 0xbc, 0xab, 0x27]
        );
        assert_eq!(
            SWAP_V2.to_vec(),
            vec![0x2a, 0x2d, 0x80, 0xb5, 0xce, 0x24, 0x7b, 0x87]
        );
        assert_eq!(
            CLOSE_POSITION.to_vec(),
            vec![0x7b, 0x86, 0x51, 0x10, 0x31, 0xc0, 0xa1, 0x7a]
        );
        assert_eq!(
            DECREASE_LIQUIDITY_V2.to_vec(),
            vec![0x58, 0x12, 0x7a, 0x1a, 0x95, 0x04, 0xac, 0xa0]
        );
        assert_eq!(
            CREATE_POOL.to_vec(),
            vec![0xe2, 0x58, 0x01, 0x5f, 0xc2, 0xc2, 0x49, 0xe9]
        );
        assert_eq!(
            INCREASE_LIQUIDITY_V2.to_vec(),
            vec![0x85, 0x15, 0x1a, 0xa4, 0xd1, 0x8b, 0x74, 0x2e]
        );
        assert_eq!(
            OPEN_POSITION_WITH_TOKEN_22_NFT.to_vec(),
            vec![0x3e, 0xf4, 0xcc, 0x1f, 0x66, 0x42, 0xee, 0xd1]
        );
        assert_eq!(
            OPEN_POSITION_V2.to_vec(),
            vec![0x4e, 0x14, 0xbb, 0x8b, 0xdd, 0xa8, 0xfc, 0x07]
        );
    }

    #[test]
    fn test_discriminators_when_indexed_should_return_correct_bytes() {
        // Test individual byte access
        assert_eq!(SWAP[0], 0xa9);
        assert_eq!(SWAP[7], 0x27);

        assert_eq!(SWAP_V2[0], 0x2a);
        assert_eq!(SWAP_V2[7], 0x87);

        assert_eq!(CLOSE_POSITION[0], 0x7b);
        assert_eq!(CLOSE_POSITION[7], 0x7a);

        assert_eq!(DECREASE_LIQUIDITY_V2[0], 0x58);
        assert_eq!(DECREASE_LIQUIDITY_V2[7], 0xa0);

        assert_eq!(CREATE_POOL[0], 0xe2);
        assert_eq!(CREATE_POOL[7], 0xe9);

        assert_eq!(INCREASE_LIQUIDITY_V2[0], 0x85);
        assert_eq!(INCREASE_LIQUIDITY_V2[7], 0x2e);

        assert_eq!(OPEN_POSITION_WITH_TOKEN_22_NFT[0], 0x3e);
        assert_eq!(OPEN_POSITION_WITH_TOKEN_22_NFT[7], 0xd1);

        assert_eq!(OPEN_POSITION_V2[0], 0x4e);
        assert_eq!(OPEN_POSITION_V2[7], 0x07);
    }
}
