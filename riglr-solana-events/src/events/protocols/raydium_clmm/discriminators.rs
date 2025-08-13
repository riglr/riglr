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
