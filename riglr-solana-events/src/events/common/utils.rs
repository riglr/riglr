use solana_sdk::pubkey::Pubkey;
use crate::error::{ParseError, ParseResult};

/// Common utility functions for event parsing

/// Read u64 from little-endian bytes at offset
pub fn read_u64_le(data: &[u8], offset: usize) -> ParseResult<u64> {
    if data.len() < offset + 8 {
        return Err(ParseError::not_enough_bytes(8, data.len() - offset, offset));
    }
    
    let mut u64_bytes = [0u8; 8];
    u64_bytes.copy_from_slice(&data[offset..offset + 8]);
    Ok(u64::from_le_bytes(u64_bytes))
}

/// Read u128 from little-endian bytes at offset
pub fn read_u128_le(data: &[u8], offset: usize) -> ParseResult<u128> {
    if data.len() < offset + 16 {
        return Err(ParseError::not_enough_bytes(16, data.len() - offset, offset));
    }
    
    let mut u128_bytes = [0u8; 16];
    u128_bytes.copy_from_slice(&data[offset..offset + 16]);
    Ok(u128::from_le_bytes(u128_bytes))
}

/// Read u32 from little-endian bytes at offset
pub fn read_u32_le(data: &[u8], offset: usize) -> ParseResult<u32> {
    if data.len() < offset + 4 {
        return Err(ParseError::not_enough_bytes(4, data.len() - offset, offset));
    }
    
    let mut u32_bytes = [0u8; 4];
    u32_bytes.copy_from_slice(&data[offset..offset + 4]);
    Ok(u32::from_le_bytes(u32_bytes))
}

/// Read i32 from little-endian bytes at offset
pub fn read_i32_le(data: &[u8], offset: usize) -> ParseResult<i32> {
    if data.len() < offset + 4 {
        return Err(ParseError::not_enough_bytes(4, data.len() - offset, offset));
    }
    
    let mut i32_bytes = [0u8; 4];
    i32_bytes.copy_from_slice(&data[offset..offset + 4]);
    Ok(i32::from_le_bytes(i32_bytes))
}

/// Read u8 from bytes at offset
pub fn read_u8_le(data: &[u8], offset: usize) -> ParseResult<u8> {
    if data.len() <= offset {
        return Err(ParseError::not_enough_bytes(1, 0, offset));
    }
    
    Ok(data[offset])
}

/// Read optional bool from bytes at offset
pub fn read_option_bool(data: &[u8], offset: &mut usize) -> ParseResult<Option<bool>> {
    if data.len() <= *offset {
        return Ok(None);
    }
    
    let tag = data[*offset];
    *offset += 1;
    
    match tag {
        0 => Ok(None),
        1 => {
            if data.len() <= *offset {
                return Err(ParseError::not_enough_bytes(1, 0, *offset));
            }
            let value = data[*offset] != 0;
            *offset += 1;
            Ok(Some(value))
        }
        _ => Err(ParseError::invalid_enum_variant(tag, "Option<bool>"))
    }
}

/// Parse a pubkey from bytes
pub fn parse_pubkey_from_bytes(bytes: &[u8]) -> ParseResult<Pubkey> {
    if bytes.len() != 32 {
        return Err(ParseError::InvalidPubkey(format!("expected 32 bytes, got {}", bytes.len())));
    }
    
    let mut pubkey_bytes = [0u8; 32];
    pubkey_bytes.copy_from_slice(bytes);
    Ok(Pubkey::from(pubkey_bytes))
}

/// Parse a u64 from little-endian bytes
pub fn parse_u64_le(bytes: &[u8]) -> ParseResult<u64> {
    if bytes.len() < 8 {
        return Err(ParseError::not_enough_bytes(8, bytes.len(), 0));
    }
    
    let mut u64_bytes = [0u8; 8];
    u64_bytes.copy_from_slice(&bytes[..8]);
    Ok(u64::from_le_bytes(u64_bytes))
}

/// Parse a u128 from little-endian bytes
pub fn parse_u128_le(bytes: &[u8]) -> ParseResult<u128> {
    if bytes.len() < 16 {
        return Err(ParseError::not_enough_bytes(16, bytes.len(), 0));
    }
    
    let mut u128_bytes = [0u8; 16];
    u128_bytes.copy_from_slice(&bytes[..16]);
    Ok(u128::from_le_bytes(u128_bytes))
}

/// Parse a u32 from little-endian bytes
pub fn parse_u32_le(bytes: &[u8]) -> ParseResult<u32> {
    if bytes.len() < 4 {
        return Err(ParseError::not_enough_bytes(4, bytes.len(), 0));
    }
    
    let mut u32_bytes = [0u8; 4];
    u32_bytes.copy_from_slice(&bytes[..4]);
    Ok(u32::from_le_bytes(u32_bytes))
}

/// Parse a u16 from little-endian bytes
pub fn parse_u16_le(bytes: &[u8]) -> ParseResult<u16> {
    if bytes.len() < 2 {
        return Err(ParseError::not_enough_bytes(2, bytes.len(), 0));
    }
    
    let mut u16_bytes = [0u8; 2];
    u16_bytes.copy_from_slice(&bytes[..2]);
    Ok(u16::from_le_bytes(u16_bytes))
}

/// Extract discriminator from instruction data
pub fn extract_discriminator(data: &[u8], size: usize) -> Option<Vec<u8>> {
    if data.len() >= size {
        Some(data[..size].to_vec())
    } else {
        None
    }
}

/// Check if instruction data starts with a specific discriminator
pub fn has_discriminator(data: &[u8], discriminator: &[u8]) -> bool {
    data.len() >= discriminator.len() && &data[..discriminator.len()] == discriminator
}

/// Convert amount with decimals to human-readable format
pub fn format_token_amount(amount: u64, decimals: u8) -> f64 {
    amount as f64 / 10_f64.powi(decimals as i32)
}

/// Convert human-readable amount to token amount with decimals
pub fn to_token_amount(amount: f64, decimals: u8) -> u64 {
    (amount * 10_f64.powi(decimals as i32)) as u64
}

/// Calculate price impact for a swap
pub fn calculate_price_impact(
    amount_in: u64,
    amount_out: u64,
    reserve_in: u64,
    reserve_out: u64,
) -> f64 {
    if reserve_in == 0 || reserve_out == 0 {
        return 0.0;
    }
    
    let expected_amount_out = (amount_in as f64 * reserve_out as f64) / reserve_in as f64;
    let actual_amount_out = amount_out as f64;
    
    if expected_amount_out == 0.0 {
        return 0.0;
    }
    
    ((expected_amount_out - actual_amount_out) / expected_amount_out * 100.0).abs()
}

/// Extract account keys from accounts array
pub fn extract_account_keys(
    accounts: &[Pubkey],
    indices: &[usize],
) -> Result<Vec<Pubkey>, Box<dyn std::error::Error + Send + Sync>> {
    let mut keys = Vec::new();
    for &index in indices {
        if index >= accounts.len() {
            return Err(format!("Account index {} out of bounds (max: {})", index, accounts.len() - 1).into());
        }
        keys.push(accounts[index]);
    }
    Ok(keys)
}

/// Decode base58 string to bytes
pub fn decode_base58(data: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    bs58::decode(data)
        .into_vec()
        .map_err(|e| format!("Failed to decode base58: {}", e).into())
}

/// Encode bytes to base58 string
pub fn encode_base58(data: &[u8]) -> String {
    bs58::encode(data).into_string()
}

/// Convert SystemTime to milliseconds since epoch
pub fn system_time_to_millis(time: std::time::SystemTime) -> u64 {
    time.duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_u64_le() {
        let bytes = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let result = parse_u64_le(&bytes).unwrap();
        assert_eq!(result, 0x0807060504030201);
    }

    #[test]
    fn test_has_discriminator() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05];
        let discriminator = [0x01, 0x02, 0x03];
        assert!(has_discriminator(&data, &discriminator));
        
        let wrong_discriminator = [0x01, 0x02, 0x04];
        assert!(!has_discriminator(&data, &wrong_discriminator));
    }

    #[test]
    fn test_format_token_amount() {
        let amount = 1_000_000_000; // 1 token with 9 decimals
        let formatted = format_token_amount(amount, 9);
        assert_eq!(formatted, 1.0);
    }

    #[test]
    fn test_calculate_price_impact() {
        let impact = calculate_price_impact(1000, 900, 10000, 10000);
        assert!(impact > 0.0);
        assert!(impact < 100.0);
    }
}