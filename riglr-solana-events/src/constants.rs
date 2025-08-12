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
