use solana_sdk::pubkey::Pubkey;

/// Check if discriminator matches
pub fn discriminator_matches(data: &str, discriminator: &str) -> bool {
    data.starts_with(discriminator)
}

/// Validate account indices
pub fn validate_account_indices(indices: &[u8], accounts_len: usize) -> bool {
    indices.iter().all(|&idx| (idx as usize) < accounts_len)
}

/// Solana program IDs
pub mod program_ids {
    use super::*;
    use std::str::FromStr;
    
    lazy_static::lazy_static! {
        pub static ref PUMP_SWAP: Pubkey = Pubkey::from_str("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA").unwrap();
        pub static ref BONK: Pubkey = Pubkey::from_str("bonksoHKfNJJ8Wo8ZJjpw7dHGePNxS2z2WE5GxUPdSo").unwrap();
        pub static ref RAYDIUM_CPMM: Pubkey = Pubkey::from_str("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C").unwrap();
        pub static ref RAYDIUM_CLMM: Pubkey = Pubkey::from_str("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK").unwrap();
        pub static ref RAYDIUM_AMM_V4: Pubkey = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap();
        pub static ref TOKEN_PROGRAM: Pubkey = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
        pub static ref TOKEN_2022_PROGRAM: Pubkey = Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb").unwrap();
        pub static ref SYSTEM_PROGRAM: Pubkey = Pubkey::from_str("11111111111111111111111111111111").unwrap();
        pub static ref SOL_MINT: Pubkey = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    }
}

/// Instruction discriminators for various protocols
pub mod discriminators {
    // PumpSwap discriminators
    pub const PUMPSWAP_BUY: &[u8] = &[102, 6, 61, 18, 1, 218, 235, 234];
    pub const PUMPSWAP_SELL: &[u8] = &[51, 230, 133, 164, 1, 127, 131, 173];
    pub const PUMPSWAP_CREATE_POOL: &[u8] = &[233, 146, 209, 142, 207, 104, 64, 188];
    pub const PUMPSWAP_DEPOSIT: &[u8] = &[242, 35, 198, 137, 82, 225, 242, 182];
    pub const PUMPSWAP_WITHDRAW: &[u8] = &[183, 18, 70, 156, 148, 109, 161, 34];

    // Inner instruction discriminators
    pub const PUMPSWAP_BUY_INNER: &str = "0x66063d1201daebea";
    pub const PUMPSWAP_SELL_INNER: &str = "0x33e685a4017f83ad";
}

/// Borsh deserialization helpers
pub fn read_u8_le(data: &[u8], offset: usize) -> Option<u8> {
    if data.len() < offset + 1 {
        return None;
    }
    Some(data[offset])
}

pub fn read_u16_le(data: &[u8], offset: usize) -> Option<u16> {
    if data.len() < offset + 2 {
        return None;
    }
    let bytes: [u8; 2] = data[offset..offset + 2].try_into().ok()?;
    Some(u16::from_le_bytes(bytes))
}

pub fn read_i32_le(data: &[u8], offset: usize) -> Option<i32> {
    if data.len() < offset + 4 {
        return None;
    }
    let bytes: [u8; 4] = data[offset..offset + 4].try_into().ok()?;
    Some(i32::from_le_bytes(bytes))
}

pub fn read_u64_le(data: &[u8], offset: usize) -> Option<u64> {
    if data.len() < offset + 8 {
        return None;
    }
    let bytes: [u8; 8] = data[offset..offset + 8].try_into().ok()?;
    Some(u64::from_le_bytes(bytes))
}

pub fn read_u128_le(data: &[u8], offset: usize) -> Option<u128> {
    if data.len() < offset + 16 {
        return None;
    }
    let bytes: [u8; 16] = data[offset..offset + 16].try_into().ok()?;
    Some(u128::from_le_bytes(bytes))
}

pub fn read_option_bool(data: &[u8], offset: &mut usize) -> Option<Option<bool>> {
    if data.len() <= *offset {
        return None;
    }
    let has_value = data[*offset];
    *offset += 1;
    
    if has_value == 0 {
        Some(None)
    } else {
        if data.len() <= *offset {
            return None;
        }
        let value = data[*offset] != 0;
        *offset += 1;
        Some(Some(value))
    }
}

pub fn read_pubkey(data: &[u8], offset: usize) -> Result<Pubkey, std::io::Error> {
    if data.len() < offset + 32 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Insufficient data for Pubkey",
        ));
    }
    let bytes: [u8; 32] = data[offset..offset + 32].try_into().unwrap();
    Ok(Pubkey::from(bytes))
}