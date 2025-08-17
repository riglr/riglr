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
