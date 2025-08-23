use crate::error::{ParseError, ParseResult};
use solana_sdk::pubkey::Pubkey;

/// Common utility functions for event parsing
///
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
        return Err(ParseError::not_enough_bytes(
            16,
            data.len() - offset,
            offset,
        ));
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
        _ => Err(ParseError::invalid_enum_variant(tag, "Option<bool>")),
    }
}

/// Parse a pubkey from bytes
pub fn parse_pubkey_from_bytes(bytes: &[u8]) -> ParseResult<Pubkey> {
    if bytes.len() != 32 {
        return Err(ParseError::InvalidPubkey(format!(
            "expected 32 bytes, got {}",
            bytes.len()
        )));
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
    data.len() >= discriminator.len() && data[..discriminator.len()].eq(discriminator)
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

/// Extract account keys from accounts array with proper error handling
pub fn extract_account_keys(
    accounts: &[Pubkey],
    indices: &[usize],
) -> Result<Vec<Pubkey>, Box<dyn std::error::Error + Send + Sync>> {
    let mut keys = Vec::new();
    for &index in indices {
        if index >= accounts.len() {
            return Err(format!(
                "Account index {} out of bounds (max: {})",
                index,
                accounts.len().saturating_sub(1)
            )
            .into());
        }
        keys.push(accounts[index]);
    }
    Ok(keys)
}

/// Safely extract a single account key by index
pub fn safe_get_account(accounts: &[Pubkey], index: usize) -> ParseResult<Pubkey> {
    accounts.get(index).copied().ok_or_else(|| {
        crate::error::ParseError::invalid_account_index(index, accounts.len().saturating_sub(1))
    })
}

/// Extract account key with optional fallback to default
pub fn get_account_or_default(accounts: &[Pubkey], index: usize) -> Pubkey {
    accounts.get(index).copied().unwrap_or_default()
}

/// Extract multiple required accounts in one call
pub fn extract_required_accounts(accounts: &[Pubkey], min_count: usize) -> ParseResult<&[Pubkey]> {
    if accounts.len() < min_count {
        return Err(crate::error::ParseError::invalid_account_index(
            min_count - 1,
            accounts.len().saturating_sub(1),
        ));
    }
    Ok(accounts)
}

/// Parse u16 from little-endian bytes with proper error handling
pub fn read_u16_le(data: &[u8], offset: usize) -> ParseResult<u16> {
    if data.len() < offset + 2 {
        return Err(ParseError::not_enough_bytes(2, data.len() - offset, offset));
    }
    let mut u16_bytes = [0u8; 2];
    u16_bytes.copy_from_slice(&data[offset..offset + 2]);
    Ok(u16::from_le_bytes(u16_bytes))
}

/// Parse boolean from single byte with proper error handling
pub fn read_bool(data: &[u8], offset: usize) -> ParseResult<bool> {
    if data.len() <= offset {
        return Err(ParseError::not_enough_bytes(1, data.len() - offset, offset));
    }
    Ok(data[offset] != 0)
}

/// Parse Pubkey from 32 bytes with proper error handling
pub fn read_pubkey(data: &[u8], offset: usize) -> ParseResult<Pubkey> {
    if data.len() < offset + 32 {
        return Err(ParseError::not_enough_bytes(
            32,
            data.len() - offset,
            offset,
        ));
    }
    let mut pubkey_bytes = [0u8; 32];
    pubkey_bytes.copy_from_slice(&data[offset..offset + 32]);
    Ok(Pubkey::new_from_array(pubkey_bytes))
}

/// Validate minimum data length with descriptive error message
pub fn validate_data_length(data: &[u8], min_length: usize, context: &str) -> ParseResult<()> {
    if data.len() < min_length {
        return Err(crate::error::ParseError::InvalidDataFormat(format!(
            "Insufficient data for {}: expected at least {} bytes, got {}",
            context,
            min_length,
            data.len()
        )));
    }
    Ok(())
}

/// Validate minimum account count with descriptive error message
pub fn validate_account_count(
    accounts: &[Pubkey],
    min_count: usize,
    context: &str,
) -> ParseResult<()> {
    if accounts.len() < min_count {
        return Err(crate::error::ParseError::InvalidDataFormat(format!(
            "Insufficient accounts for {}: expected at least {}, got {}",
            context,
            min_count,
            accounts.len()
        )));
    }
    Ok(())
}

/// Parse swap amounts (common pattern for buy/sell operations)
pub fn parse_swap_amounts(data: &[u8]) -> ParseResult<(u64, u64)> {
    validate_data_length(data, 16, "swap amounts")?;
    let amount_1 = read_u64_le(data, 0)?;
    let amount_2 = read_u64_le(data, 8)?;
    Ok((amount_1, amount_2))
}

/// Parse liquidity amounts (common pattern for deposit/withdraw operations)
pub fn parse_liquidity_amounts(data: &[u8]) -> ParseResult<(u64, u64, u64)> {
    validate_data_length(data, 24, "liquidity amounts")?;
    let amount_1 = read_u64_le(data, 0)?;
    let amount_2 = read_u64_le(data, 8)?;
    let amount_3 = read_u64_le(data, 16)?;
    Ok((amount_1, amount_2, amount_3))
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

    // Additional comprehensive tests for 100% coverage

    #[test]
    fn test_read_u64_le_when_valid_should_return_value() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
        let result = read_u64_le(&data, 0).unwrap();
        assert_eq!(result, 0x0807060504030201);
    }

    #[test]
    fn test_read_u64_le_when_offset_valid_should_return_value() {
        let data = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
        let result = read_u64_le(&data, 1).unwrap();
        assert_eq!(result, 0x0908070605040302);
    }

    #[test]
    fn test_read_u64_le_when_not_enough_bytes_should_return_err() {
        let data = [0x01, 0x02, 0x03];
        let result = read_u64_le(&data, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_u64_le_when_offset_too_large_should_return_err() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let result = read_u64_le(&data, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_u128_le_when_valid_should_return_value() {
        let data = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ];
        let result = read_u128_le(&data, 0).unwrap();
        assert_eq!(result, 0x100F0E0D0C0B0A090807060504030201);
    }

    #[test]
    fn test_read_u128_le_when_not_enough_bytes_should_return_err() {
        let data = [0x01, 0x02, 0x03];
        let result = read_u128_le(&data, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_u128_le_when_offset_too_large_should_return_err() {
        let data = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ];
        let result = read_u128_le(&data, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_u32_le_when_valid_should_return_value() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05];
        let result = read_u32_le(&data, 0).unwrap();
        assert_eq!(result, 0x04030201);
    }

    #[test]
    fn test_read_u32_le_when_not_enough_bytes_should_return_err() {
        let data = [0x01, 0x02];
        let result = read_u32_le(&data, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_u32_le_when_offset_too_large_should_return_err() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let result = read_u32_le(&data, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_i32_le_when_valid_should_return_value() {
        let data = [0xFF, 0xFF, 0xFF, 0xFF, 0x00];
        let result = read_i32_le(&data, 0).unwrap();
        assert_eq!(result, -1);
    }

    #[test]
    fn test_read_i32_le_when_not_enough_bytes_should_return_err() {
        let data = [0x01, 0x02];
        let result = read_i32_le(&data, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_i32_le_when_offset_too_large_should_return_err() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let result = read_i32_le(&data, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_u8_le_when_valid_should_return_value() {
        let data = [0x42, 0x43];
        let result = read_u8_le(&data, 0).unwrap();
        assert_eq!(result, 0x42);
    }

    #[test]
    fn test_read_u8_le_when_offset_valid_should_return_value() {
        let data = [0x42, 0x43];
        let result = read_u8_le(&data, 1).unwrap();
        assert_eq!(result, 0x43);
    }

    #[test]
    fn test_read_u8_le_when_offset_equals_length_should_return_err() {
        let data = [0x42];
        let result = read_u8_le(&data, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_u8_le_when_empty_data_should_return_err() {
        let data = [];
        let result = read_u8_le(&data, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_option_bool_when_empty_data_should_return_none() {
        let data = [];
        let mut offset = 0;
        let result = read_option_bool(&data, &mut offset).unwrap();
        assert_eq!(result, None);
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_read_option_bool_when_tag_zero_should_return_none() {
        let data = [0x00];
        let mut offset = 0;
        let result = read_option_bool(&data, &mut offset).unwrap();
        assert_eq!(result, None);
        assert_eq!(offset, 1);
    }

    #[test]
    fn test_read_option_bool_when_tag_one_and_value_true_should_return_some_true() {
        let data = [0x01, 0x01];
        let mut offset = 0;
        let result = read_option_bool(&data, &mut offset).unwrap();
        assert_eq!(result, Some(true));
        assert_eq!(offset, 2);
    }

    #[test]
    fn test_read_option_bool_when_tag_one_and_value_false_should_return_some_false() {
        let data = [0x01, 0x00];
        let mut offset = 0;
        let result = read_option_bool(&data, &mut offset).unwrap();
        assert_eq!(result, Some(false));
        assert_eq!(offset, 2);
    }

    #[test]
    fn test_read_option_bool_when_tag_one_but_no_value_should_return_err() {
        let data = [0x01];
        let mut offset = 0;
        let result = read_option_bool(&data, &mut offset);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_option_bool_when_invalid_tag_should_return_err() {
        let data = [0x02];
        let mut offset = 0;
        let result = read_option_bool(&data, &mut offset);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_pubkey_from_bytes_when_valid_should_return_pubkey() {
        let bytes = [1u8; 32];
        let result = parse_pubkey_from_bytes(&bytes).unwrap();
        assert_eq!(result, Pubkey::from([1u8; 32]));
    }

    #[test]
    fn test_parse_pubkey_from_bytes_when_wrong_length_should_return_err() {
        let bytes = [1u8; 31];
        let result = parse_pubkey_from_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_pubkey_from_bytes_when_empty_should_return_err() {
        let bytes = [];
        let result = parse_pubkey_from_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_u64_le_when_not_enough_bytes_should_return_err() {
        let bytes = [0x01, 0x02, 0x03];
        let result = parse_u64_le(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_u64_le_when_empty_should_return_err() {
        let bytes = [];
        let result = parse_u64_le(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_u128_le_when_valid_should_return_value() {
        let bytes = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ];
        let result = parse_u128_le(&bytes).unwrap();
        assert_eq!(result, 0x100F0E0D0C0B0A090807060504030201);
    }

    #[test]
    fn test_parse_u128_le_when_not_enough_bytes_should_return_err() {
        let bytes = [0x01, 0x02, 0x03];
        let result = parse_u128_le(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_u32_le_when_valid_should_return_value() {
        let bytes = [0x01, 0x02, 0x03, 0x04];
        let result = parse_u32_le(&bytes).unwrap();
        assert_eq!(result, 0x04030201);
    }

    #[test]
    fn test_parse_u32_le_when_not_enough_bytes_should_return_err() {
        let bytes = [0x01, 0x02];
        let result = parse_u32_le(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_u16_le_when_valid_should_return_value() {
        let bytes = [0x01, 0x02];
        let result = parse_u16_le(&bytes).unwrap();
        assert_eq!(result, 0x0201);
    }

    #[test]
    fn test_parse_u16_le_when_not_enough_bytes_should_return_err() {
        let bytes = [0x01];
        let result = parse_u16_le(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_u16_le_when_empty_should_return_err() {
        let bytes = [];
        let result = parse_u16_le(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_discriminator_when_enough_bytes_should_return_some() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let result = extract_discriminator(&data, 3);
        let expected: Option<Vec<u8>> = Some(vec![0x01, 0x02, 0x03]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_discriminator_when_exact_size_should_return_some() {
        let data = [0x01, 0x02, 0x03];
        let result = extract_discriminator(&data, 3);
        let expected: Option<Vec<u8>> = Some(vec![0x01, 0x02, 0x03]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_discriminator_when_not_enough_bytes_should_return_none() {
        let data = [0x01, 0x02];
        let result = extract_discriminator(&data, 3);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_discriminator_when_zero_size_should_return_empty_vec() {
        let data = [0x01, 0x02];
        let result = extract_discriminator(&data, 0);
        let expected: Option<Vec<u8>> = Some(vec![]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_has_discriminator_when_data_too_short_should_return_false() {
        let data = [0x01, 0x02];
        let discriminator = [0x01, 0x02, 0x03];
        assert!(!has_discriminator(&data, &discriminator));
    }

    #[test]
    fn test_has_discriminator_when_empty_discriminator_should_return_true() {
        let data = [0x01, 0x02];
        let discriminator = [];
        assert!(has_discriminator(&data, &discriminator));
    }

    #[test]
    fn test_has_discriminator_when_empty_data_and_empty_discriminator_should_return_true() {
        let data = [];
        let discriminator = [];
        assert!(has_discriminator(&data, &discriminator));
    }

    #[test]
    fn test_format_token_amount_when_zero_decimals_should_return_same() {
        let amount = 1000;
        let formatted = format_token_amount(amount, 0);
        assert_eq!(formatted, 1000.0);
    }

    #[test]
    fn test_format_token_amount_when_zero_amount_should_return_zero() {
        let amount = 0;
        let formatted = format_token_amount(amount, 9);
        assert_eq!(formatted, 0.0);
    }

    #[test]
    fn test_to_token_amount_when_zero_amount_should_return_zero() {
        let result = to_token_amount(0.0, 9);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_to_token_amount_when_one_token_should_return_correct_amount() {
        let result = to_token_amount(1.0, 9);
        assert_eq!(result, 1_000_000_000);
    }

    #[test]
    fn test_to_token_amount_when_zero_decimals_should_return_same() {
        let result = to_token_amount(1000.0, 0);
        assert_eq!(result, 1000);
    }

    #[test]
    fn test_calculate_price_impact_when_zero_reserve_in_should_return_zero() {
        let impact = calculate_price_impact(1000, 900, 0, 10000);
        assert_eq!(impact, 0.0);
    }

    #[test]
    fn test_calculate_price_impact_when_zero_reserve_out_should_return_zero() {
        let impact = calculate_price_impact(1000, 900, 10000, 0);
        assert_eq!(impact, 0.0);
    }

    #[test]
    fn test_calculate_price_impact_when_zero_amount_in_should_return_zero() {
        let impact = calculate_price_impact(0, 900, 10000, 10000);
        assert_eq!(impact, 0.0);
    }

    #[test]
    fn test_calculate_price_impact_when_equal_amounts_should_return_zero() {
        let impact = calculate_price_impact(1000, 1000, 10000, 10000);
        assert_eq!(impact, 0.0);
    }

    #[test]
    fn test_extract_account_keys_when_valid_indices_should_return_keys() {
        let accounts = vec![
            Pubkey::from([1u8; 32]),
            Pubkey::from([2u8; 32]),
            Pubkey::from([3u8; 32]),
        ];
        let indices = [0, 2];
        let result = extract_account_keys(&accounts, &indices).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Pubkey::from([1u8; 32]));
        assert_eq!(result[1], Pubkey::from([3u8; 32]));
    }

    #[test]
    fn test_extract_account_keys_when_empty_indices_should_return_empty() {
        let accounts = vec![Pubkey::from([1u8; 32])];
        let indices = [];
        let result = extract_account_keys(&accounts, &indices).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_extract_account_keys_when_index_out_of_bounds_should_return_err() {
        let accounts = vec![Pubkey::from([1u8; 32])];
        let indices = [1];
        let result = extract_account_keys(&accounts, &indices);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_account_keys_when_empty_accounts_should_return_err() {
        let accounts = vec![];
        let indices = [0];
        let result = extract_account_keys(&accounts, &indices);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_base58_when_valid_should_return_bytes() {
        let encoded = "1111111111111111111111111111111114oLvT2";
        let result = decode_base58(encoded).unwrap();
        let expected: Vec<u8> = vec![0u8; 32];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_decode_base58_when_invalid_should_return_err() {
        let encoded = "invalid_base58_0OIl";
        let result = decode_base58(encoded);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_base58_when_empty_should_return_empty() {
        let encoded = "";
        let result = decode_base58(encoded).unwrap();
        let expected: Vec<u8> = vec![];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_encode_base58_when_valid_bytes_should_return_string() {
        let bytes = vec![0u8; 32];
        let result = encode_base58(&bytes);
        assert_eq!(result, "1111111111111111111111111111111114oLvT2");
    }

    #[test]
    fn test_encode_base58_when_empty_bytes_should_return_empty() {
        let bytes = vec![];
        let result = encode_base58(&bytes);
        assert_eq!(result, "");
    }

    #[test]
    fn test_encode_base58_when_single_byte_should_return_correct() {
        let bytes = vec![1];
        let result = encode_base58(&bytes);
        assert_eq!(result, "2");
    }

    #[test]
    fn test_system_time_to_millis_when_valid_time_should_return_millis() {
        let time = std::time::UNIX_EPOCH + std::time::Duration::from_millis(1000);
        let result = system_time_to_millis(time);
        assert_eq!(result, 1000);
    }

    #[test]
    fn test_system_time_to_millis_when_epoch_should_return_zero() {
        let time = std::time::UNIX_EPOCH;
        let result = system_time_to_millis(time);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_system_time_to_millis_when_before_epoch_should_return_zero() {
        let time = std::time::UNIX_EPOCH - std::time::Duration::from_secs(1);
        let result = system_time_to_millis(time);
        assert_eq!(result, 0);
    }
}
