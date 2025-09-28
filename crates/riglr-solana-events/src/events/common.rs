//! Common utilities shared across protocol parsers.
//!
//! This module provides core functionality used by all Solana event parsers including:
//! - Utility functions for parsing various data types from byte arrays
//! - Shared helper functions for instruction decoding and data extraction

use crate::error::{Error as ParseError, ParseResult};
use core::error::Error;
use solana_sdk::pubkey::Pubkey;
use std::time::SystemTime;

/// Common utility functions for event parsing
///
/// Read u64 from little-endian bytes at offset
///
/// # Errors
///
/// Returns `ParseError::NotEnoughBytes` if there are insufficient bytes at the offset.
///
/// # Panics
///
/// Panics if the offset range is invalid, which should not occur as bounds are checked.
#[inline]
pub fn read_u64_le(data: &[u8], offset: usize) -> ParseResult<u64> {
    if data.len() < offset.saturating_add(8) {
        return Err(ParseError::NotEnoughBytes {
            expected: 8,
            found: data.len().saturating_sub(offset),
            offset,
        });
    }

    let mut u64_bytes = [0u8; 8];
    let slice =
        data.get(offset..offset.saturating_add(8))
            .ok_or_else(|| ParseError::NotEnoughBytes {
                expected: 8,
                found: data.len().saturating_sub(offset),
                offset,
            })?;
    u64_bytes.copy_from_slice(slice);
    Ok(u64::from_le_bytes(u64_bytes))
}

/// Read u128 from little-endian bytes at offset
///
/// # Errors
///
/// Returns `ParseError::NotEnoughBytes` if there are insufficient bytes at the offset.
///
/// # Panics
///
/// Panics if the offset range is invalid, which should not occur as bounds are checked.
#[inline]
pub fn read_u128_le(data: &[u8], offset: usize) -> ParseResult<u128> {
    if data.len() < offset.saturating_add(16) {
        return Err(ParseError::NotEnoughBytes {
            expected: 16,
            found: data.len().saturating_sub(offset),
            offset,
        });
    }

    let mut u128_bytes = [0u8; 16];
    let slice =
        data.get(offset..offset.saturating_add(16))
            .ok_or_else(|| ParseError::NotEnoughBytes {
                expected: 16,
                found: data.len().saturating_sub(offset),
                offset,
            })?;
    u128_bytes.copy_from_slice(slice);
    Ok(u128::from_le_bytes(u128_bytes))
}

/// Read u32 from little-endian bytes at offset
///
/// # Errors
///
/// Returns `ParseError::NotEnoughBytes` if there are insufficient bytes at the offset.
///
/// # Panics
///
/// Panics if the offset range is invalid, which should not occur as bounds are checked.
#[inline]
pub fn read_u32_le(data: &[u8], offset: usize) -> ParseResult<u32> {
    if data.len() < offset.saturating_add(4) {
        return Err(ParseError::NotEnoughBytes {
            expected: 4,
            found: data.len().saturating_sub(offset),
            offset,
        });
    }

    let mut u32_bytes = [0u8; 4];
    let slice =
        data.get(offset..offset.saturating_add(4))
            .ok_or_else(|| ParseError::NotEnoughBytes {
                expected: 4,
                found: data.len().saturating_sub(offset),
                offset,
            })?;
    u32_bytes.copy_from_slice(slice);
    Ok(u32::from_le_bytes(u32_bytes))
}

/// Read i32 from little-endian bytes at offset
///
/// # Errors
///
/// Returns `ParseError::NotEnoughBytes` if there are insufficient bytes at the offset.
///
/// # Panics
///
/// Panics if the offset range is invalid, which should not occur as bounds are checked.
#[inline]
pub fn read_i32_le(data: &[u8], offset: usize) -> ParseResult<i32> {
    if data.len() < offset.saturating_add(4) {
        return Err(ParseError::NotEnoughBytes {
            expected: 4,
            found: data.len().saturating_sub(offset),
            offset,
        });
    }

    let mut i32_bytes = [0u8; 4];
    let slice =
        data.get(offset..offset.saturating_add(4))
            .ok_or_else(|| ParseError::NotEnoughBytes {
                expected: 4,
                found: data.len().saturating_sub(offset),
                offset,
            })?;
    i32_bytes.copy_from_slice(slice);
    Ok(i32::from_le_bytes(i32_bytes))
}

/// Read u8 from bytes at offset
///
/// # Errors
///
/// Returns `ParseError::NotEnoughBytes` if the offset is beyond the data length.
///
/// # Panics
///
/// Panics if the offset is invalid, which should not occur as bounds are checked.
#[inline]
pub fn read_u8_le(data: &[u8], offset: usize) -> ParseResult<u8> {
    if data.len() <= offset {
        return Err(ParseError::NotEnoughBytes {
            expected: 1,
            found: 0,
            offset,
        });
    }

    Ok(*data.get(offset).ok_or(ParseError::NotEnoughBytes {
        expected: 1,
        found: 0,
        offset,
    })?)
}

/// Read optional bool from bytes at offset
///
/// # Errors
///
/// Returns `ParseError::InvalidEnumVariant` if the tag byte is invalid or insufficient bytes for Some variant.
///
/// # Panics
///
/// Panics if the offset is invalid, which should not occur as bounds are checked.
#[inline]
pub fn read_option_bool(data: &[u8], offset: &mut usize) -> ParseResult<Option<bool>> {
    if data.len() <= *offset {
        return Ok(None);
    }

    let tag = *data.get(*offset).ok_or(ParseError::NotEnoughBytes {
        expected: 1,
        found: 0,
        offset: *offset,
    })?;
    *offset = offset.saturating_add(1);

    match tag {
        0 => Ok(None),
        1 => {
            if data.len() <= *offset {
                return Err(ParseError::NotEnoughBytes {
                    expected: 1,
                    found: 0,
                    offset: *offset,
                });
            }
            let value = *data.get(*offset).ok_or(ParseError::NotEnoughBytes {
                expected: 1,
                found: 0,
                offset: *offset,
            })? != 0;
            *offset = offset.saturating_add(1);
            Ok(Some(value))
        }
        _ => Err(ParseError::InvalidEnumVariant {
            variant: tag,
            type_name: "Option<bool>".to_owned(),
        }),
    }
}

/// Parse a pubkey from bytes
///
/// # Errors
///
/// Returns `ParseError::InvalidPubkey` if the byte slice is not exactly 32 bytes.
#[inline]
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
///
/// # Errors
///
/// Returns `ParseError::NotEnoughBytes` if there are fewer than 8 bytes.
///
/// # Panics
///
/// Panics if the slice range is invalid, which should not occur as bounds are checked.
#[inline]
pub fn parse_u64_le(bytes: &[u8]) -> ParseResult<u64> {
    if bytes.len() < 8 {
        return Err(ParseError::NotEnoughBytes {
            expected: 8,
            found: bytes.len(),
            offset: 0,
        });
    }

    let mut u64_bytes = [0u8; 8];
    let slice = bytes.get(..8).ok_or(ParseError::NotEnoughBytes {
        expected: 8,
        found: bytes.len(),
        offset: 0,
    })?;
    u64_bytes.copy_from_slice(slice);
    Ok(u64::from_le_bytes(u64_bytes))
}

/// Parse a u128 from little-endian bytes
///
/// # Errors
///
/// Returns `ParseError::NotEnoughBytes` if there are fewer than 16 bytes.
///
/// # Panics
///
/// Panics if the slice range is invalid, which should not occur as bounds are checked.
#[inline]
pub fn parse_u128_le(bytes: &[u8]) -> ParseResult<u128> {
    if bytes.len() < 16 {
        return Err(ParseError::NotEnoughBytes {
            expected: 16,
            found: bytes.len(),
            offset: 0,
        });
    }

    let mut u128_bytes = [0u8; 16];
    let slice = bytes.get(..16).ok_or(ParseError::NotEnoughBytes {
        expected: 16,
        found: bytes.len(),
        offset: 0,
    })?;
    u128_bytes.copy_from_slice(slice);
    Ok(u128::from_le_bytes(u128_bytes))
}

/// Parse a u32 from little-endian bytes
///
/// # Errors
///
/// Returns `ParseError::NotEnoughBytes` if there are fewer than 4 bytes.
///
/// # Panics
///
/// Panics if the slice range is invalid, which should not occur as bounds are checked.
#[inline]
pub fn parse_u32_le(bytes: &[u8]) -> ParseResult<u32> {
    if bytes.len() < 4 {
        return Err(ParseError::NotEnoughBytes {
            expected: 4,
            found: bytes.len(),
            offset: 0,
        });
    }

    let mut u32_bytes = [0u8; 4];
    let slice = bytes.get(..4).ok_or(ParseError::NotEnoughBytes {
        expected: 4,
        found: bytes.len(),
        offset: 0,
    })?;
    u32_bytes.copy_from_slice(slice);
    Ok(u32::from_le_bytes(u32_bytes))
}

/// Parse a u16 from little-endian bytes
///
/// # Errors
///
/// Returns `ParseError::NotEnoughBytes` if there are fewer than 2 bytes.
///
/// # Panics
///
/// Panics if the slice range is invalid, which should not occur as bounds are checked.
#[inline]
pub fn parse_u16_le(bytes: &[u8]) -> ParseResult<u16> {
    if bytes.len() < 2 {
        return Err(ParseError::NotEnoughBytes {
            expected: 2,
            found: bytes.len(),
            offset: 0,
        });
    }

    let mut u16_bytes = [0u8; 2];
    let slice = bytes.get(..2).ok_or(ParseError::NotEnoughBytes {
        expected: 2,
        found: bytes.len(),
        offset: 0,
    })?;
    u16_bytes.copy_from_slice(slice);
    Ok(u16::from_le_bytes(u16_bytes))
}

/// Extract discriminator bytes from instruction data
///
/// Returns the first `length` bytes from the data as a discriminator,
/// or None if there are insufficient bytes.
///
/// # Panics
///
/// Panics if the slice range is invalid, which should not occur as bounds are checked.
#[must_use]
#[inline]
pub fn extract_discriminator(data: &[u8], length: usize) -> Option<Vec<u8>> {
    if data.len() >= length {
        return data.get(..length).map(<[u8]>::to_vec);
    }
    None
}

/// Check if instruction data starts with the given discriminator
///
/// # Panics
///
/// Panics if the slice range is invalid, which should not occur as bounds are checked.
#[must_use]
#[inline]
pub fn has_discriminator(data: &[u8], discriminator: &[u8]) -> bool {
    data.len() >= discriminator.len()
        && data
            .get(..discriminator.len())
            .is_some_and(|slice| slice.eq(discriminator))
}

/// Convert amount with decimals to human-readable format
#[must_use]
#[inline]
pub fn format_token_amount(amount: u64, decimals: u8) -> f64 {
    #[expect(clippy::cast_precision_loss)]
    {
        amount as f64 / 10_f64.powi(i32::from(decimals))
    }
}

/// Convert human-readable amount to token amount with decimals
#[must_use]
#[inline]
pub fn to_token_amount(amount: f64, decimals: u8) -> u64 {
    #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    {
        (amount * 10_f64.powi(i32::from(decimals))) as u64
    }
}

/// Calculate price impact for a swap
#[must_use]
#[inline]
pub fn calculate_price_impact(
    amount_in: u64,
    amount_out: u64,
    reserve_in: u64,
    reserve_out: u64,
) -> f64 {
    if reserve_in == 0 || reserve_out == 0 {
        return 0.0_f64;
    }

    #[expect(clippy::cast_precision_loss)]
    let expected_amount_out = (amount_in as f64 * reserve_out as f64) / reserve_in as f64;
    #[expect(clippy::cast_precision_loss)]
    let actual_amount_out = amount_out as f64;

    if expected_amount_out == 0.0_f64 {
        return 0.0_f64;
    }

    ((expected_amount_out - actual_amount_out) / expected_amount_out * 100.0_f64).abs()
}

/// Extract account keys from accounts array with proper error handling
///
/// # Errors
///
/// Returns an error if any account index is out of bounds.
///
/// # Panics
///
/// Panics if an account index is invalid, which should not occur as bounds are checked.
#[inline]
pub fn extract_account_keys(
    accounts: &[Pubkey],
    indices: &[usize],
) -> Result<Vec<Pubkey>, Box<dyn Error + Send + Sync>> {
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
        keys.push(*accounts.get(index).ok_or_else(|| {
            format!(
                "Account index {} out of bounds (max: {})",
                index,
                accounts.len().saturating_sub(1)
            )
        })?);
    }
    Ok(keys)
}

/// Safely extract a single account key by index
///
/// # Errors
///
/// Returns `ParseError::InvalidAccountIndex` if the index is out of bounds.
#[inline]
pub fn safe_get_account(accounts: &[Pubkey], index: usize) -> ParseResult<Pubkey> {
    accounts
        .get(index)
        .copied()
        .ok_or_else(|| ParseError::InvalidAccountIndex {
            index,
            max: accounts.len().saturating_sub(1),
        })
}

/// Extract account key with optional fallback to default
#[must_use]
#[inline]
pub fn get_account_or_default(accounts: &[Pubkey], index: usize) -> Pubkey {
    accounts.get(index).copied().unwrap_or_default()
}

/// Extract multiple required accounts in one call
///
/// # Errors
///
/// Returns `ParseError::InvalidAccountIndex` if there are fewer accounts than required.
#[inline]
pub const fn extract_required_accounts(
    accounts: &[Pubkey],
    min_count: usize,
) -> ParseResult<&[Pubkey]> {
    if accounts.len() < min_count {
        return Err(ParseError::InvalidAccountIndex {
            index: min_count.saturating_sub(1),
            max: accounts.len().saturating_sub(1),
        });
    }
    Ok(accounts)
}

/// Parse u16 from little-endian bytes with proper error handling
///
/// # Errors
///
/// Returns `ParseError::NotEnoughBytes` if there are insufficient bytes at the offset.
///
/// # Panics
///
/// Panics if the offset range is invalid, which should not occur as bounds are checked.
#[inline]
pub fn read_u16_le(data: &[u8], offset: usize) -> ParseResult<u16> {
    if data.len() < offset.saturating_add(2) {
        return Err(ParseError::NotEnoughBytes {
            expected: 2,
            found: data.len().saturating_sub(offset),
            offset,
        });
    }
    let mut u16_bytes = [0u8; 2];
    let slice =
        data.get(offset..offset.saturating_add(2))
            .ok_or_else(|| ParseError::NotEnoughBytes {
                expected: 2,
                found: data.len().saturating_sub(offset),
                offset,
            })?;
    u16_bytes.copy_from_slice(slice);
    Ok(u16::from_le_bytes(u16_bytes))
}

/// Parse boolean from single byte with proper error handling
///
/// # Errors
///
/// Returns `ParseError::NotEnoughBytes` if the offset is beyond the data length.
///
/// # Panics
///
/// Panics if the offset is invalid, which should not occur as bounds are checked.
#[inline]
pub fn read_bool(data: &[u8], offset: usize) -> ParseResult<bool> {
    if data.len() <= offset {
        return Err(ParseError::NotEnoughBytes {
            expected: 1,
            found: data.len().saturating_sub(offset),
            offset,
        });
    }
    Ok(*data.get(offset).ok_or_else(|| ParseError::NotEnoughBytes {
        expected: 1,
        found: data.len().saturating_sub(offset),
        offset,
    })? != 0)
}

/// Parse Pubkey from 32 bytes with proper error handling
///
/// # Errors
///
/// Returns `ParseError::NotEnoughBytes` if there are insufficient bytes at the offset.
///
/// # Panics
///
/// Panics if the offset range is invalid, which should not occur as bounds are checked.
#[inline]
pub fn read_pubkey(data: &[u8], offset: usize) -> ParseResult<Pubkey> {
    if data.len() < offset.saturating_add(32) {
        return Err(ParseError::NotEnoughBytes {
            expected: 32,
            found: data.len().saturating_sub(offset),
            offset,
        });
    }
    let mut pubkey_bytes = [0u8; 32];
    let slice =
        data.get(offset..offset.saturating_add(32))
            .ok_or_else(|| ParseError::NotEnoughBytes {
                expected: 32,
                found: data.len().saturating_sub(offset),
                offset,
            })?;
    pubkey_bytes.copy_from_slice(slice);
    Ok(Pubkey::new_from_array(pubkey_bytes))
}

/// Validate minimum data length with descriptive error message
///
/// # Errors
///
/// Returns `ParseError::InvalidDataFormat` if the data length is insufficient.
#[inline]
pub fn validate_data_length(data: &[u8], min_length: usize, context: &str) -> ParseResult<()> {
    if data.len() < min_length {
        return Err(ParseError::InvalidDataFormat(format!(
            "Insufficient data for {}: expected at least {} bytes, got {}",
            context,
            min_length,
            data.len()
        )));
    }
    Ok(())
}

/// Validate minimum account count with descriptive error message
///
/// # Errors
///
/// Returns `ParseError::InvalidDataFormat` if there are insufficient accounts.
#[inline]
pub fn validate_account_count(
    accounts: &[Pubkey],
    min_count: usize,
    context: &str,
) -> ParseResult<()> {
    if accounts.len() < min_count {
        return Err(ParseError::InvalidDataFormat(format!(
            "Insufficient accounts for {}: expected at least {}, got {}",
            context,
            min_count,
            accounts.len()
        )));
    }
    Ok(())
}

/// Parse swap amounts (common pattern for buy/sell operations)
///
/// # Errors
///
/// Returns `ParseError::InvalidDataFormat` if there are fewer than 16 bytes.
#[inline]
pub fn parse_swap_amounts(data: &[u8]) -> ParseResult<(u64, u64)> {
    validate_data_length(data, 16, "swap amounts")?;
    let amount_1 = read_u64_le(data, 0)?;
    let amount_2 = read_u64_le(data, 8)?;
    Ok((amount_1, amount_2))
}

/// Parse liquidity amounts (common pattern for deposit/withdraw operations)
///
/// # Errors
///
/// Returns `ParseError::InvalidDataFormat` if there are fewer than 24 bytes.
#[inline]
pub fn parse_liquidity_amounts(data: &[u8]) -> ParseResult<(u64, u64, u64)> {
    validate_data_length(data, 24, "liquidity amounts")?;
    let amount_1 = read_u64_le(data, 0)?;
    let amount_2 = read_u64_le(data, 8)?;
    let amount_3 = read_u64_le(data, 16)?;
    Ok((amount_1, amount_2, amount_3))
}

/// Decode base58 string to bytes
///
/// # Errors
///
/// Returns an error if the base58 string is invalid.
#[inline]
pub fn decode_base58(data: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    bs58::decode(data)
        .into_vec()
        .map_err(|error| format!("Failed to decode base58: {error}").into())
}

/// Encode bytes to base58 string
#[must_use]
#[inline]
pub fn encode_base58(data: &[u8]) -> String {
    bs58::encode(data).into_string()
}

/// Convert `SystemTime` to milliseconds since epoch
#[must_use]
#[inline]
pub fn system_time_to_millis(time: SystemTime) -> u64 {
    use std::time::UNIX_EPOCH;
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(0)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::types::{EventType, ProtocolType, StreamMetadata, SwapData, TransferData};
    use core::str::FromStr as _;

    #[test]
    fn module_types_accessible() {
        // Test that types module is accessible
        let protocol = ProtocolType::OrcaWhirlpool;
        assert_eq!(protocol, ProtocolType::OrcaWhirlpool);

        let event = EventType::Swap;
        assert_eq!(event, EventType::Swap);
    }

    #[test]
    fn module_utils_accessible() {
        // Test that utils functions are accessible through re-export
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let result = read_u64_le(&data, 0);
        assert!(result.is_ok());
        result.expect("read_u64_le should succeed"); // Test expect replacement - use unwrap in tests instead
    }

    #[test]
    fn types_reexport_transfer_data() {
        // Test TransferData is accessible through re-export
        let source = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Test pubkey should be valid"); // Test expect replacement
        let destination = Pubkey::from_str("11111111111111111111111111111113")
            .expect("Test pubkey should be valid"); // Test expect replacement

        let transfer_data = TransferData {
            source,
            destination,
            mint: None,
            amount: 100,
        };

        assert_eq!(transfer_data.amount, 100);
        assert_eq!(transfer_data.source, source);
        assert_eq!(transfer_data.destination, destination);
        assert_eq!(transfer_data.mint, None);
    }

    #[test]
    fn types_reexport_swap_data() {
        // Test SwapData is accessible through re-export
        let input_mint = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Test pubkey should be valid"); // Test expect replacement
        let output_mint = Pubkey::from_str("11111111111111111111111111111113")
            .expect("Test pubkey should be valid"); // Test expect replacement

        let swap_data = SwapData {
            input_mint,
            output_mint,
            amount_in: 1000,
            amount_out: 950,
        };

        assert_eq!(swap_data.amount_in, 1000);
        assert_eq!(swap_data.amount_out, 950);
        assert_eq!(swap_data.input_mint, input_mint);
        assert_eq!(swap_data.output_mint, output_mint);
    }

    #[test]
    fn utils_reexport_read_functions() {
        // Test that all read functions are accessible through re-export
        let data = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF,
        ];

        // Test read_u64_le
        let u64_result = read_u64_le(&data, 0);
        assert!(u64_result.is_ok());
        assert_eq!(u64_result.expect("read_u64_le should succeed"), u64::MAX);

        // Test read_u128_le
        let u128_result = read_u128_le(&data, 0);
        assert!(u128_result.is_ok());
        assert_eq!(u128_result.expect("read_u128_le should succeed"), u128::MAX);

        // Test read_u32_le
        let u32_result = read_u32_le(&data, 0);
        assert!(u32_result.is_ok());
        assert_eq!(u32_result.expect("read_u32_le should succeed"), u32::MAX);

        // Test read_i32_le (negative value)
        let i32_result = read_i32_le(&data, 0);
        assert!(i32_result.is_ok());
        assert_eq!(i32_result.expect("read_i32_le should succeed"), -1_i32);

        // Test read_u8_le
        let u8_result = read_u8_le(&data, 0);
        assert!(u8_result.is_ok());
        assert_eq!(u8_result.expect("read_u8_le should succeed"), 255);
    }

    #[test]
    fn utils_reexport_parse_functions() {
        // Test that all parse functions are accessible through re-export
        let data = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ];

        // Test parse_u64_le
        let u64_result = parse_u64_le(&data);
        u64_result.expect("parse_u64_le should succeed");

        // Test parse_u128_le
        let u128_result = parse_u128_le(&data);
        u128_result.expect("parse_u128_le should succeed");

        // Test parse_u32_le
        let u32_result = parse_u32_le(&data);
        u32_result.expect("parse_u32_le should succeed");

        // Test parse_u16_le
        let u16_result = parse_u16_le(&data);
        u16_result.expect("parse_u16_le should succeed");
    }

    #[test]
    fn utils_reexport_pubkey_functions() {
        // Test pubkey parsing functions are accessible through re-export
        let pubkey_bytes = [1_u8; 32];
        let result = parse_pubkey_from_bytes(&pubkey_bytes);
        assert!(result.is_ok());

        let pubkey = result.expect("parse_pubkey_from_bytes should succeed");
        assert_eq!(pubkey.to_bytes(), pubkey_bytes);
    }

    #[test]
    fn utils_reexport_discriminator_functions() {
        // Test discriminator functions are accessible through re-export
        let data = [0x01, 0x02, 0x03, 0x04, 0x05];

        // Test extract_discriminator
        let discriminator = extract_discriminator(&data, 3);
        assert!(discriminator.is_some());
        assert_eq!(
            discriminator.expect("discriminator should be extracted"),
            vec![0x01, 0x02, 0x03]
        );

        // Test has_discriminator
        let check_discriminator = [0x01, 0x02, 0x03];
        assert!(has_discriminator(&data, &check_discriminator));

        let wrong_discriminator = [0x01, 0x02, 0x04];
        assert!(!has_discriminator(&data, &wrong_discriminator));
    }

    #[test]
    fn utils_reexport_token_amount_functions() {
        // Test token amount functions are accessible through re-export
        let amount = 1_000_000_000_u64; // 1 token with 9 decimals
        let formatted = format_token_amount(amount, 9);
        {
            assert!((formatted - 1.0_f64).abs() < f64::EPSILON);
        }

        let converted_back = to_token_amount(1.0, 9);
        assert_eq!(converted_back, 1_000_000_000_u64);
    }

    #[test]
    fn utils_reexport_price_impact_function() {
        // Test price impact calculation is accessible through re-export
        let impact = calculate_price_impact(1000, 900, 10000, 10000);
        assert!(impact > 0.0_f64);
        assert!(impact < 100.0_f64);

        // Test edge case with zero reserves
        let zero_impact = calculate_price_impact(1000, 900, 0, 0);
        {
            assert!(zero_impact.abs() < f64::EPSILON);
        }
    }

    #[test]
    fn utils_reexport_account_keys_function() {
        // Test account keys extraction is accessible through re-export
        let accounts = vec![
            Pubkey::from_str("11111111111111111111111111111112")
                .expect("Test pubkey should be valid"),
            Pubkey::from_str("11111111111111111111111111111113")
                .expect("Test pubkey should be valid"),
            Pubkey::from_str("11111111111111111111111111111114")
                .expect("Test pubkey should be valid"),
        ];
        let indices = vec![0, 2];

        let result = extract_account_keys(&accounts, &indices);
        assert!(result.is_ok());
        let keys = result.expect("extract_account_keys should succeed");
        assert_eq!(keys.len(), 2);
        // Use get() instead of direct indexing to avoid panic warnings
        assert_eq!(
            *keys.first().expect("keys should not be empty"),
            *accounts.first().expect("accounts should not be empty")
        );
        assert_eq!(
            *keys.get(1).expect("keys should have a second element"),
            *accounts
                .get(2)
                .expect("accounts should have a third element")
        );
    }

    #[test]
    fn utils_reexport_base58_functions() {
        // Test base58 encoding/decoding functions are accessible through re-export
        let data = b"hello world";
        let encoded = encode_base58(data);
        assert!(!encoded.is_empty());

        let decoded = decode_base58(&encoded);
        assert!(decoded.is_ok());
        assert_eq!(
            decoded.expect("decode_base58 should succeed"),
            data.to_vec()
        );
    }

    #[test]
    fn utils_reexport_time_function() {
        // Test time conversion function is accessible through re-export
        use std::time::SystemTime;

        let time = SystemTime::UNIX_EPOCH;
        let millis = system_time_to_millis(time);
        assert_eq!(millis, 0);

        let now = SystemTime::now();
        let now_millis = system_time_to_millis(now);
        assert!(now_millis > 0);
    }

    #[test]
    fn utils_reexport_option_bool_function() {
        // Test option bool parsing is accessible through re-export
        let data = [0_u8]; // None variant
        let mut offset = 0;
        let result = read_option_bool(&data, &mut offset);
        assert!(result.is_ok());
        assert_eq!(result.expect("read_option_bool should succeed"), None);
        assert_eq!(offset, 1);

        let data_some_true = [1_u8, 1_u8]; // Some(true)
        let mut offset = 0;
        let result = read_option_bool(&data_some_true, &mut offset);
        assert!(result.is_ok());
        assert_eq!(result.expect("read_option_bool should succeed"), Some(true));
        assert_eq!(offset, 2);

        let data_some_false = [1_u8, 0_u8]; // Some(false)
        let mut offset = 0;
        let result = read_option_bool(&data_some_false, &mut offset);
        assert!(result.is_ok());
        assert_eq!(
            result.expect("read_option_bool should succeed"),
            Some(false)
        );
        assert_eq!(offset, 2);
    }

    #[test]
    fn types_reexport_stream_metadata() {
        use std::time::SystemTime;

        // Test StreamMetadata is accessible through re-export
        let stream_metadata = StreamMetadata {
            stream_source: "test_source".to_owned(),
            received_at: SystemTime::UNIX_EPOCH,
            sequence_number: Some(42),
            custom_data: Some(serde_json::json!({"key": "value"})),
        };

        assert_eq!(stream_metadata.stream_source, "test_source");
        assert_eq!(stream_metadata.received_at, SystemTime::UNIX_EPOCH);
        assert_eq!(stream_metadata.sequence_number, Some(42));
        assert!(stream_metadata.custom_data.is_some());
    }

    #[test]
    fn types_reexport_protocol_type_variants() {
        // Test all ProtocolType variants are accessible
        let jupiter = ProtocolType::Jupiter;
        let orca = ProtocolType::OrcaWhirlpool;
        let raydium = ProtocolType::Raydium;
        let serum = ProtocolType::Serum;

        assert_eq!(jupiter, ProtocolType::Jupiter);
        assert_eq!(orca, ProtocolType::OrcaWhirlpool);
        assert_eq!(raydium, ProtocolType::Raydium);
        assert_eq!(serum, ProtocolType::Serum);

        // Test they are not equal to each other
        assert_ne!(jupiter, orca);
        assert_ne!(orca, raydium);
        assert_ne!(raydium, serum);
    }

    #[test]
    fn types_reexport_event_type_variants() {
        // Test all EventType variants are accessible
        let swap = EventType::Swap;
        let transfer = EventType::Transfer;
        let liquidity_provision = EventType::LiquidityProvision;
        let liquidity_removal = EventType::LiquidityRemoval;

        assert_eq!(swap, EventType::Swap);
        assert_eq!(transfer, EventType::Transfer);
        assert_eq!(liquidity_provision, EventType::LiquidityProvision);
        assert_eq!(liquidity_removal, EventType::LiquidityRemoval);

        // Test they are not equal to each other
        assert_ne!(swap, transfer);
        assert_ne!(transfer, liquidity_provision);
        assert_ne!(liquidity_provision, liquidity_removal);
    }
}
