/// Common constants for parsing blockchain data
pub mod parsing {
    /// Size constants for basic data types
    pub const U8_SIZE: usize = 1;
    /// Size of a u16 in bytes
    pub const U16_SIZE: usize = 2;
    /// Size of a u32 in bytes
    pub const U32_SIZE: usize = 4;
    /// Size of a u64 in bytes
    pub const U64_SIZE: usize = 8;
    /// Size of a u128 in bytes
    pub const U128_SIZE: usize = 16;
    /// Size of a Solana public key in bytes
    pub const PUBKEY_SIZE: usize = 32;

    /// Common byte ranges for parsing
    pub const U8_RANGE: std::ops::Range<usize> = 0..U8_SIZE;
    /// Byte range for parsing a single u16 value
    pub const U16_RANGE: std::ops::Range<usize> = 0..U16_SIZE;
    /// Byte range for parsing a single u32 value
    pub const U32_RANGE: std::ops::Range<usize> = 0..U32_SIZE;
    /// Byte range for parsing a single u64 value
    pub const U64_RANGE: std::ops::Range<usize> = 0..U64_SIZE;
    /// Byte range for parsing a single u128 value
    pub const U128_RANGE: std::ops::Range<usize> = 0..U128_SIZE;
    /// Byte range for parsing a single public key value
    pub const PUBKEY_RANGE: std::ops::Range<usize> = 0..PUBKEY_SIZE;

    /// Second u64 range (for parsing two consecutive u64 values)
    pub const SECOND_U64_RANGE: std::ops::Range<usize> = U64_SIZE..(U64_SIZE * 2);

    /// Third u64 range (for parsing three consecutive u64 values)
    pub const THIRD_U64_RANGE: std::ops::Range<usize> = (U64_SIZE * 2)..(U64_SIZE * 3);

    /// Anchor discriminator size (8 bytes)
    pub const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;

    /// Instruction data offset (after discriminator)
    pub const INSTRUCTION_DATA_OFFSET: usize = ANCHOR_DISCRIMINATOR_SIZE;
}

/// Common minimum data lengths for instruction parsing
pub mod instruction_sizes {
    use super::parsing::*;

    /// Minimum size for swap instructions (discriminator + 2 u64s)
    pub const MIN_SWAP_INSTRUCTION_SIZE: usize = ANCHOR_DISCRIMINATOR_SIZE + (U64_SIZE * 2);

    /// Minimum size for liquidity instructions (discriminator + 3 u64s)
    pub const MIN_LIQUIDITY_INSTRUCTION_SIZE: usize = ANCHOR_DISCRIMINATOR_SIZE + (U64_SIZE * 3);

    /// Minimum size for position instructions (discriminator + various fields)
    pub const MIN_POSITION_INSTRUCTION_SIZE: usize = ANCHOR_DISCRIMINATOR_SIZE + U32_SIZE * 2;

    /// Jupiter minimum instruction size (discriminator + pubkeys + amounts)
    pub const JUPITER_MIN_INSTRUCTION_SIZE: usize =
        ANCHOR_DISCRIMINATOR_SIZE + PUBKEY_SIZE * 2 + U64_SIZE * 2;
}

/// Common minimum account counts for various operations
pub mod account_counts {
    /// Minimum accounts for basic swap operations
    pub const MIN_SWAP_ACCOUNTS: usize = 8;

    /// Minimum accounts for liquidity operations
    pub const MIN_LIQUIDITY_ACCOUNTS: usize = 10;

    /// Minimum accounts for position operations
    pub const MIN_POSITION_ACCOUNTS: usize = 15;

    /// Minimum accounts for complex operations (AMM v4, etc.)
    pub const MIN_COMPLEX_ACCOUNTS: usize = 17;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_basic_size_constants_should_have_correct_values() {
        assert_eq!(parsing::U8_SIZE, 1);
        assert_eq!(parsing::U16_SIZE, 2);
        assert_eq!(parsing::U32_SIZE, 4);
        assert_eq!(parsing::U64_SIZE, 8);
        assert_eq!(parsing::U128_SIZE, 16);
        assert_eq!(parsing::PUBKEY_SIZE, 32);
    }

    #[test]
    fn test_parsing_basic_range_constants_should_have_correct_ranges() {
        assert_eq!(parsing::U8_RANGE, 0..1);
        assert_eq!(parsing::U16_RANGE, 0..2);
        assert_eq!(parsing::U32_RANGE, 0..4);
        assert_eq!(parsing::U64_RANGE, 0..8);
        assert_eq!(parsing::U128_RANGE, 0..16);
        assert_eq!(parsing::PUBKEY_RANGE, 0..32);
    }

    #[test]
    fn test_parsing_multi_u64_ranges_should_have_correct_values() {
        assert_eq!(parsing::SECOND_U64_RANGE, 8..16);
        assert_eq!(parsing::THIRD_U64_RANGE, 16..24);
    }

    #[test]
    fn test_parsing_multi_u64_ranges_should_be_consecutive() {
        // Verify that ranges are consecutive and non-overlapping
        assert_eq!(parsing::U64_RANGE.end, parsing::SECOND_U64_RANGE.start);
        assert_eq!(
            parsing::SECOND_U64_RANGE.end,
            parsing::THIRD_U64_RANGE.start
        );
    }

    #[test]
    fn test_parsing_anchor_constants_should_have_correct_values() {
        assert_eq!(parsing::ANCHOR_DISCRIMINATOR_SIZE, 8);
        assert_eq!(parsing::INSTRUCTION_DATA_OFFSET, 8);
        assert_eq!(
            parsing::INSTRUCTION_DATA_OFFSET,
            parsing::ANCHOR_DISCRIMINATOR_SIZE
        );
    }

    #[test]
    fn test_instruction_sizes_min_swap_should_be_correctly_calculated() {
        let expected = parsing::ANCHOR_DISCRIMINATOR_SIZE + (parsing::U64_SIZE * 2);
        assert_eq!(instruction_sizes::MIN_SWAP_INSTRUCTION_SIZE, expected);
        assert_eq!(instruction_sizes::MIN_SWAP_INSTRUCTION_SIZE, 24); // 8 + 16
    }

    #[test]
    fn test_instruction_sizes_min_liquidity_should_be_correctly_calculated() {
        let expected = parsing::ANCHOR_DISCRIMINATOR_SIZE + (parsing::U64_SIZE * 3);
        assert_eq!(instruction_sizes::MIN_LIQUIDITY_INSTRUCTION_SIZE, expected);
        assert_eq!(instruction_sizes::MIN_LIQUIDITY_INSTRUCTION_SIZE, 32); // 8 + 24
    }

    #[test]
    fn test_instruction_sizes_min_position_should_be_correctly_calculated() {
        let expected = parsing::ANCHOR_DISCRIMINATOR_SIZE + parsing::U32_SIZE * 2;
        assert_eq!(instruction_sizes::MIN_POSITION_INSTRUCTION_SIZE, expected);
        assert_eq!(instruction_sizes::MIN_POSITION_INSTRUCTION_SIZE, 16); // 8 + 8
    }

    #[test]
    fn test_instruction_sizes_jupiter_min_should_be_correctly_calculated() {
        let expected =
            parsing::ANCHOR_DISCRIMINATOR_SIZE + parsing::PUBKEY_SIZE * 2 + parsing::U64_SIZE * 2;
        assert_eq!(instruction_sizes::JUPITER_MIN_INSTRUCTION_SIZE, expected);
        assert_eq!(instruction_sizes::JUPITER_MIN_INSTRUCTION_SIZE, 88); // 8 + 64 + 16
    }

    #[test]
    fn test_instruction_sizes_should_have_logical_ordering() {
        // Position instructions should be smaller than swap instructions
        assert!(
            instruction_sizes::MIN_POSITION_INSTRUCTION_SIZE
                < instruction_sizes::MIN_SWAP_INSTRUCTION_SIZE
        );

        // Swap instructions should be smaller than liquidity instructions
        assert!(
            instruction_sizes::MIN_SWAP_INSTRUCTION_SIZE
                < instruction_sizes::MIN_LIQUIDITY_INSTRUCTION_SIZE
        );

        // Jupiter instructions should be larger than basic swap instructions
        assert!(
            instruction_sizes::JUPITER_MIN_INSTRUCTION_SIZE
                > instruction_sizes::MIN_SWAP_INSTRUCTION_SIZE
        );
    }

    #[test]
    fn test_account_counts_should_have_correct_values() {
        assert_eq!(account_counts::MIN_SWAP_ACCOUNTS, 8);
        assert_eq!(account_counts::MIN_LIQUIDITY_ACCOUNTS, 10);
        assert_eq!(account_counts::MIN_POSITION_ACCOUNTS, 15);
        assert_eq!(account_counts::MIN_COMPLEX_ACCOUNTS, 17);
    }

    #[test]
    fn test_account_counts_should_have_logical_ordering() {
        // Swap operations should require fewer accounts than liquidity operations
        assert!(account_counts::MIN_SWAP_ACCOUNTS < account_counts::MIN_LIQUIDITY_ACCOUNTS);

        // Liquidity operations should require fewer accounts than position operations
        assert!(account_counts::MIN_LIQUIDITY_ACCOUNTS < account_counts::MIN_POSITION_ACCOUNTS);

        // Position operations should require fewer accounts than complex operations
        assert!(account_counts::MIN_POSITION_ACCOUNTS < account_counts::MIN_COMPLEX_ACCOUNTS);
    }

    #[test]
    fn test_range_properties_should_be_valid() {
        // All ranges should start at 0 for basic types
        assert_eq!(parsing::U8_RANGE.start, 0);
        assert_eq!(parsing::U16_RANGE.start, 0);
        assert_eq!(parsing::U32_RANGE.start, 0);
        assert_eq!(parsing::U64_RANGE.start, 0);
        assert_eq!(parsing::U128_RANGE.start, 0);
        assert_eq!(parsing::PUBKEY_RANGE.start, 0);

        // All ranges should have positive lengths
        assert!(parsing::U8_RANGE.len() > 0);
        assert!(parsing::U16_RANGE.len() > 0);
        assert!(parsing::U32_RANGE.len() > 0);
        assert!(parsing::U64_RANGE.len() > 0);
        assert!(parsing::U128_RANGE.len() > 0);
        assert!(parsing::PUBKEY_RANGE.len() > 0);
        assert!(parsing::SECOND_U64_RANGE.len() > 0);
        assert!(parsing::THIRD_U64_RANGE.len() > 0);
    }

    #[test]
    fn test_size_type_relationships_should_be_correct() {
        // Each larger type should be exactly double the size of the previous power-of-2 type
        assert_eq!(parsing::U16_SIZE, parsing::U8_SIZE * 2);
        assert_eq!(parsing::U32_SIZE, parsing::U16_SIZE * 2);
        assert_eq!(parsing::U64_SIZE, parsing::U32_SIZE * 2);
        assert_eq!(parsing::U128_SIZE, parsing::U64_SIZE * 2);

        // Pubkey size should be exactly 4 times a u64
        assert_eq!(parsing::PUBKEY_SIZE, parsing::U64_SIZE * 4);
    }

    #[test]
    fn test_multi_u64_range_lengths_should_be_consistent() {
        // All u64 ranges should have the same length
        assert_eq!(parsing::U64_RANGE.len(), parsing::SECOND_U64_RANGE.len());
        assert_eq!(parsing::U64_RANGE.len(), parsing::THIRD_U64_RANGE.len());
        assert_eq!(
            parsing::SECOND_U64_RANGE.len(),
            parsing::THIRD_U64_RANGE.len()
        );
    }

    #[test]
    fn test_instruction_sizes_all_include_discriminator() {
        // All instruction sizes should be at least as large as the discriminator
        assert!(instruction_sizes::MIN_SWAP_INSTRUCTION_SIZE >= parsing::ANCHOR_DISCRIMINATOR_SIZE);
        assert!(
            instruction_sizes::MIN_LIQUIDITY_INSTRUCTION_SIZE >= parsing::ANCHOR_DISCRIMINATOR_SIZE
        );
        assert!(
            instruction_sizes::MIN_POSITION_INSTRUCTION_SIZE >= parsing::ANCHOR_DISCRIMINATOR_SIZE
        );
        assert!(
            instruction_sizes::JUPITER_MIN_INSTRUCTION_SIZE >= parsing::ANCHOR_DISCRIMINATOR_SIZE
        );
    }
}
