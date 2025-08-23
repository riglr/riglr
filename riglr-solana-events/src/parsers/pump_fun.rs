//! High-performance PumpFun parser with specialized event extraction
//!
//! This parser handles PumpFun protocol operations with optimized parsing
//! for trading events and pool operations.

use crate::metadata_helpers::{set_event_type, set_protocol_type};
use crate::types::{EventMetadata, EventType, ProtocolType};
use crate::zero_copy::{ByteSliceEventParser, CustomDeserializer, ParseError, ZeroCopyEvent};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
// UnifiedEvent trait has been removed

/// PumpFun program ID
pub const PUMP_FUN_PROGRAM_ID: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";

/// PumpFun instruction discriminators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PumpFunDiscriminator {
    /// Buy tokens instruction
    Buy,
    /// Sell tokens instruction
    Sell,
    /// Create new pool instruction
    CreatePool,
    /// Deposit liquidity instruction
    Deposit,
    /// Withdraw liquidity instruction
    Withdraw,
    /// Set pool parameters instruction
    SetParams,
}

impl PumpFunDiscriminator {
    /// Parse discriminator from first 8 bytes (u64 little-endian)
    pub fn from_u64(value: u64) -> Option<Self> {
        match value {
            0x66063d1201daebea => Some(Self::Buy),
            0x33e685a4017f83ad => Some(Self::Sell),
            0x181ec828051c0777 => Some(Self::CreatePool),
            0xf223c68952e1f2b6 => Some(Self::Deposit),
            0x2e2d8d4ec43b1d79 => Some(Self::Withdraw),
            0xa6f0308f80e7b5c9 => Some(Self::SetParams),
            _ => None,
        }
    }

    /// Get corresponding event type
    pub fn event_type(&self) -> EventType {
        match self {
            Self::Buy => EventType::PumpSwapBuy,
            Self::Sell => EventType::PumpSwapSell,
            Self::CreatePool => EventType::PumpSwapCreatePool,
            Self::Deposit => EventType::PumpSwapDeposit,
            Self::Withdraw => EventType::PumpSwapWithdraw,
            Self::SetParams => EventType::PumpSwapSetParams,
        }
    }
}

/// PumpFun Buy instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct PumpBuyInstruction {
    /// 8-byte discriminator
    pub discriminator: u64,
    /// Amount to buy (in lamports or token base units)
    pub amount: u64,
    /// Maximum price per token (slippage protection)
    pub max_price_per_token: u64,
}

/// PumpFun Sell instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct PumpSellInstruction {
    /// 8-byte discriminator
    pub discriminator: u64,
    /// Amount to sell (in token units)
    pub amount: u64,
    /// Minimum price per token (slippage protection)
    pub min_price_per_token: u64,
}

/// PumpFun CreatePool instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct PumpCreatePoolInstruction {
    /// 8-byte discriminator
    pub discriminator: u64,
    /// Token name (32 bytes max)
    pub name: String,
    /// Token symbol (10 bytes max)
    pub symbol: String,
    /// Token URI for metadata
    pub uri: String,
}

/// PumpFun Deposit instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct PumpDepositInstruction {
    /// 8-byte discriminator
    pub discriminator: u64,
    /// Amount to deposit
    pub amount: u64,
    /// Minimum LP tokens expected
    pub min_lp_tokens: u64,
}

/// PumpFun Withdraw instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct PumpWithdrawInstruction {
    /// 8-byte discriminator
    pub discriminator: u64,
    /// LP tokens to burn
    pub lp_tokens: u64,
    /// Minimum amount out
    pub min_amount_out: u64,
}

/// High-performance PumpFun parser
#[derive(Debug)]
pub struct PumpFunParser {
    /// Program ID for validation
    #[allow(dead_code)]
    program_id: Pubkey,
    /// Enable zero-copy parsing
    zero_copy: bool,
}

impl Default for PumpFunParser {
    fn default() -> Self {
        Self {
            program_id: PUMP_FUN_PROGRAM_ID
                .parse()
                .expect("Valid PumpFun program ID"),
            zero_copy: true,
        }
    }
}

impl PumpFunParser {
    /// Create new PumpFun parser
    pub fn new() -> Self {
        Self {
            program_id: PUMP_FUN_PROGRAM_ID
                .parse()
                .expect("Valid PumpFun program ID"),
            zero_copy: true,
        }
    }

    /// Create parser with zero-copy disabled
    pub fn new_standard() -> Self {
        Self {
            program_id: PUMP_FUN_PROGRAM_ID
                .parse()
                .expect("Valid PumpFun program ID"),
            zero_copy: false,
        }
    }

    /// Parse Buy instruction with zero-copy
    fn parse_buy<'a>(
        &self,
        data: &'a [u8],
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'a>, ParseError> {
        let mut deserializer = CustomDeserializer::new(data);

        let discriminator = deserializer.read_u64_le()?;
        let amount = deserializer.read_u64_le()?;
        let max_price_per_token = deserializer.read_u64_le()?;

        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };

        let instruction_data = PumpBuyInstruction {
            discriminator,
            amount,
            max_price_per_token,
        };
        event.set_parsed_data(instruction_data);

        let json = serde_json::json!({
            "instruction_type": "buy",
            "amount": amount.to_string(),
            "max_price_per_token": max_price_per_token.to_string(),
            "protocol": "pump_fun"
        });
        event.set_json_data(json);

        Ok(event)
    }

    /// Parse Sell instruction with zero-copy
    fn parse_sell<'a>(
        &self,
        data: &'a [u8],
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'a>, ParseError> {
        let mut deserializer = CustomDeserializer::new(data);

        let discriminator = deserializer.read_u64_le()?;
        let amount = deserializer.read_u64_le()?;
        let min_price_per_token = deserializer.read_u64_le()?;

        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };

        let instruction_data = PumpSellInstruction {
            discriminator,
            amount,
            min_price_per_token,
        };
        event.set_parsed_data(instruction_data);

        let json = serde_json::json!({
            "instruction_type": "sell",
            "amount": amount.to_string(),
            "min_price_per_token": min_price_per_token.to_string(),
            "protocol": "pump_fun"
        });
        event.set_json_data(json);

        Ok(event)
    }

    /// Parse CreatePool instruction
    fn parse_create_pool<'a>(
        &self,
        data: &'a [u8],
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'a>, ParseError> {
        // For CreatePool, we need to parse string data which is more complex
        // This is a simplified implementation
        let mut deserializer = CustomDeserializer::new(data);

        let discriminator = deserializer.read_u64_le()?;

        // Skip complex string parsing for now - would need proper Borsh deserialization
        // In a real implementation, we'd parse the name, symbol, and URI fields

        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };

        let json = serde_json::json!({
            "instruction_type": "create_pool",
            "discriminator": discriminator,
            "protocol": "pump_fun"
        });
        event.set_json_data(json);

        Ok(event)
    }

    /// Parse Deposit instruction
    fn parse_deposit<'a>(
        &self,
        data: &'a [u8],
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'a>, ParseError> {
        let mut deserializer = CustomDeserializer::new(data);

        let discriminator = deserializer.read_u64_le()?;
        let amount = deserializer.read_u64_le()?;
        let min_lp_tokens = deserializer.read_u64_le()?;

        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };

        let instruction_data = PumpDepositInstruction {
            discriminator,
            amount,
            min_lp_tokens,
        };
        event.set_parsed_data(instruction_data);

        let json = serde_json::json!({
            "instruction_type": "deposit",
            "amount": amount.to_string(),
            "min_lp_tokens": min_lp_tokens.to_string(),
            "protocol": "pump_fun"
        });
        event.set_json_data(json);

        Ok(event)
    }

    /// Parse Withdraw instruction
    fn parse_withdraw<'a>(
        &self,
        data: &'a [u8],
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'a>, ParseError> {
        let mut deserializer = CustomDeserializer::new(data);

        let discriminator = deserializer.read_u64_le()?;
        let lp_tokens = deserializer.read_u64_le()?;
        let min_amount_out = deserializer.read_u64_le()?;

        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };

        let instruction_data = PumpWithdrawInstruction {
            discriminator,
            lp_tokens,
            min_amount_out,
        };
        event.set_parsed_data(instruction_data);

        let json = serde_json::json!({
            "instruction_type": "withdraw",
            "lp_tokens": lp_tokens.to_string(),
            "min_amount_out": min_amount_out.to_string(),
            "protocol": "pump_fun"
        });
        event.set_json_data(json);

        Ok(event)
    }
}

impl ByteSliceEventParser for PumpFunParser {
    fn parse_from_slice<'a>(
        &self,
        data: &'a [u8],
        mut metadata: EventMetadata,
    ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError> {
        if data.len() < 8 {
            return Err(ParseError::InsufficientData {
                expected: 8,
                actual: data.len(),
            });
        }

        // Update metadata with protocol info
        set_protocol_type(&mut metadata.core, ProtocolType::PumpSwap);

        // Parse 8-byte discriminator
        let discriminator_bytes = &data[0..8];
        let discriminator = u64::from_le_bytes([
            discriminator_bytes[0],
            discriminator_bytes[1],
            discriminator_bytes[2],
            discriminator_bytes[3],
            discriminator_bytes[4],
            discriminator_bytes[5],
            discriminator_bytes[6],
            discriminator_bytes[7],
        ]);

        let discriminator_enum =
            PumpFunDiscriminator::from_u64(discriminator).ok_or_else(|| {
                ParseError::UnknownDiscriminator {
                    discriminator: discriminator_bytes.to_vec(),
                }
            })?;

        // Update event type based on discriminator
        set_event_type(&mut metadata.core, discriminator_enum.event_type());

        let event = match discriminator_enum {
            PumpFunDiscriminator::Buy => self.parse_buy(data, metadata)?,
            PumpFunDiscriminator::Sell => self.parse_sell(data, metadata)?,
            PumpFunDiscriminator::CreatePool => self.parse_create_pool(data, metadata)?,
            PumpFunDiscriminator::Deposit => self.parse_deposit(data, metadata)?,
            PumpFunDiscriminator::Withdraw => self.parse_withdraw(data, metadata)?,
            PumpFunDiscriminator::SetParams => {
                // Handle set params instruction
                let mut event = if self.zero_copy {
                    ZeroCopyEvent::new_borrowed(metadata, data)
                } else {
                    ZeroCopyEvent::new_owned(metadata, data.to_vec())
                };

                let json = serde_json::json!({
                    "instruction_type": "set_params",
                    "discriminator": discriminator,
                    "protocol": "pump_fun"
                });
                event.set_json_data(json);
                event
            }
        };

        Ok(vec![event])
    }

    fn can_parse(&self, data: &[u8]) -> bool {
        if data.len() < 8 {
            return false;
        }

        let discriminator_bytes = &data[0..8];
        let discriminator = u64::from_le_bytes([
            discriminator_bytes[0],
            discriminator_bytes[1],
            discriminator_bytes[2],
            discriminator_bytes[3],
            discriminator_bytes[4],
            discriminator_bytes[5],
            discriminator_bytes[6],
            discriminator_bytes[7],
        ]);

        PumpFunDiscriminator::from_u64(discriminator).is_some()
    }

    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::PumpSwap
    }
}

/// Factory for creating PumpFun parsers
pub struct PumpFunParserFactory;

impl PumpFunParserFactory {
    /// Create a new high-performance zero-copy parser
    pub fn create_zero_copy() -> Arc<dyn ByteSliceEventParser> {
        Arc::new(PumpFunParser::default())
    }

    /// Create a standard parser
    pub fn create_standard() -> Arc<dyn ByteSliceEventParser> {
        Arc::new(PumpFunParser::new_standard())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EventMetadata;

    #[test]
    fn test_discriminator_parsing() {
        assert_eq!(
            PumpFunDiscriminator::from_u64(0x66063d1201daebea),
            Some(PumpFunDiscriminator::Buy)
        );
        assert_eq!(
            PumpFunDiscriminator::from_u64(0x33e685a4017f83ad),
            Some(PumpFunDiscriminator::Sell)
        );
        assert_eq!(PumpFunDiscriminator::from_u64(0xFFFFFFFFFFFFFFFF), None);
    }

    #[test]
    fn test_buy_instruction_parsing() {
        let parser = PumpFunParser::default();

        let mut data = Vec::new();
        data.extend_from_slice(&0x66063d1201daebeau64.to_le_bytes()); // Buy discriminator
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount
        data.extend_from_slice(&2000u64.to_le_bytes()); // max_price_per_token

        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).unwrap();

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.protocol_type(), ProtocolType::PumpSwap);

        // Check parsed data
        let parsed = event.get_parsed_data::<PumpBuyInstruction>().unwrap();
        assert_eq!(parsed.amount, 1000);
        assert_eq!(parsed.max_price_per_token, 2000);
    }

    #[test]
    fn test_can_parse() {
        let parser = PumpFunParser::default();

        let mut buy_data = Vec::new();
        buy_data.extend_from_slice(&0x66063d1201daebeau64.to_le_bytes());
        assert!(parser.can_parse(&buy_data));

        let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        assert!(!parser.can_parse(&invalid_data));

        let short_data = vec![0x66, 0x06]; // Too short
        assert!(!parser.can_parse(&short_data));
    }

    #[test]
    fn test_factory() {
        let zero_copy_parser = PumpFunParserFactory::create_zero_copy();
        let standard_parser = PumpFunParserFactory::create_standard();

        assert_eq!(zero_copy_parser.protocol_type(), ProtocolType::PumpSwap);
        assert_eq!(standard_parser.protocol_type(), ProtocolType::PumpSwap);
    }

    #[test]
    fn test_discriminator_event_type_mapping() {
        assert_eq!(
            PumpFunDiscriminator::Buy.event_type(),
            EventType::PumpSwapBuy
        );
        assert_eq!(
            PumpFunDiscriminator::Sell.event_type(),
            EventType::PumpSwapSell
        );
        assert_eq!(
            PumpFunDiscriminator::CreatePool.event_type(),
            EventType::PumpSwapCreatePool
        );
        assert_eq!(
            PumpFunDiscriminator::Deposit.event_type(),
            EventType::PumpSwapDeposit
        );
        assert_eq!(
            PumpFunDiscriminator::Withdraw.event_type(),
            EventType::PumpSwapWithdraw
        );
        assert_eq!(
            PumpFunDiscriminator::SetParams.event_type(),
            EventType::PumpSwapSetParams
        );
    }

    #[test]
    fn test_all_discriminator_values() {
        assert_eq!(
            PumpFunDiscriminator::from_u64(0x181ec828051c0777),
            Some(PumpFunDiscriminator::CreatePool)
        );
        assert_eq!(
            PumpFunDiscriminator::from_u64(0xf223c68952e1f2b6),
            Some(PumpFunDiscriminator::Deposit)
        );
        assert_eq!(
            PumpFunDiscriminator::from_u64(0x2e2d8d4ec43b1d79),
            Some(PumpFunDiscriminator::Withdraw)
        );
        assert_eq!(
            PumpFunDiscriminator::from_u64(0xa6f0308f80e7b5c9),
            Some(PumpFunDiscriminator::SetParams)
        );
    }

    #[test]
    fn test_pump_fun_parser_default() {
        let parser = PumpFunParser::default();
        assert!(parser.zero_copy);
        assert_eq!(parser.protocol_type(), ProtocolType::PumpSwap);
    }

    #[test]
    fn test_pump_fun_parser_new_standard() {
        let parser = PumpFunParser::new_standard();
        assert!(!parser.zero_copy);
        assert_eq!(parser.protocol_type(), ProtocolType::PumpSwap);
    }

    #[test]
    fn test_sell_instruction_parsing() {
        let parser = PumpFunParser::default();

        let mut data = Vec::new();
        data.extend_from_slice(&0x33e685a4017f83adu64.to_le_bytes()); // Sell discriminator
        data.extend_from_slice(&5000u64.to_le_bytes()); // amount
        data.extend_from_slice(&1500u64.to_le_bytes()); // min_price_per_token

        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).unwrap();

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.protocol_type(), ProtocolType::PumpSwap);

        let parsed = event.get_parsed_data::<PumpSellInstruction>().unwrap();
        assert_eq!(parsed.amount, 5000);
        assert_eq!(parsed.min_price_per_token, 1500);
    }

    #[test]
    fn test_sell_instruction_parsing_standard() {
        let parser = PumpFunParser::new_standard();

        let mut data = Vec::new();
        data.extend_from_slice(&0x33e685a4017f83adu64.to_le_bytes()); // Sell discriminator
        data.extend_from_slice(&5000u64.to_le_bytes()); // amount
        data.extend_from_slice(&1500u64.to_le_bytes()); // min_price_per_token

        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).unwrap();

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.protocol_type(), ProtocolType::PumpSwap);

        let parsed = event.get_parsed_data::<PumpSellInstruction>().unwrap();
        assert_eq!(parsed.amount, 5000);
        assert_eq!(parsed.min_price_per_token, 1500);
    }

    #[test]
    fn test_create_pool_instruction_parsing() {
        let parser = PumpFunParser::default();

        let mut data = Vec::new();
        data.extend_from_slice(&0x181ec828051c0777u64.to_le_bytes()); // CreatePool discriminator
                                                                      // Add some dummy data for the strings (simplified)
        data.extend_from_slice(&[0u8; 32]); // name placeholder
        data.extend_from_slice(&[0u8; 16]); // symbol and uri placeholder

        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).unwrap();

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.protocol_type(), ProtocolType::PumpSwap);
    }

    #[test]
    fn test_deposit_instruction_parsing() {
        let parser = PumpFunParser::default();

        let mut data = Vec::new();
        data.extend_from_slice(&0xf223c68952e1f2b6u64.to_le_bytes()); // Deposit discriminator
        data.extend_from_slice(&3000u64.to_le_bytes()); // amount
        data.extend_from_slice(&2500u64.to_le_bytes()); // min_lp_tokens

        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).unwrap();

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.protocol_type(), ProtocolType::PumpSwap);

        let parsed = event.get_parsed_data::<PumpDepositInstruction>().unwrap();
        assert_eq!(parsed.amount, 3000);
        assert_eq!(parsed.min_lp_tokens, 2500);
    }

    #[test]
    fn test_withdraw_instruction_parsing() {
        let parser = PumpFunParser::default();

        let mut data = Vec::new();
        data.extend_from_slice(&0x2e2d8d4ec43b1d79u64.to_le_bytes()); // Withdraw discriminator
        data.extend_from_slice(&4000u64.to_le_bytes()); // lp_tokens
        data.extend_from_slice(&3500u64.to_le_bytes()); // min_amount_out

        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).unwrap();

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.protocol_type(), ProtocolType::PumpSwap);

        let parsed = event.get_parsed_data::<PumpWithdrawInstruction>().unwrap();
        assert_eq!(parsed.lp_tokens, 4000);
        assert_eq!(parsed.min_amount_out, 3500);
    }

    #[test]
    fn test_set_params_instruction_parsing() {
        let parser = PumpFunParser::default();

        let mut data = Vec::new();
        data.extend_from_slice(&0xa6f0308f80e7b5c9u64.to_le_bytes()); // SetParams discriminator
        data.extend_from_slice(&[0u8; 16]); // Some dummy parameter data

        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).unwrap();

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.protocol_type(), ProtocolType::PumpSwap);
    }

    #[test]
    fn test_parse_from_slice_insufficient_data() {
        let parser = PumpFunParser::default();
        let data = vec![0x66, 0x06, 0x3d]; // Only 3 bytes, need at least 8
        let metadata = EventMetadata::default();

        let result = parser.parse_from_slice(&data, metadata);
        assert!(result.is_err());

        match result.unwrap_err() {
            ParseError::InsufficientData { expected, actual } => {
                assert_eq!(expected, 8);
                assert_eq!(actual, 3);
            }
            _ => panic!("Expected InsufficientData error"),
        }
    }

    #[test]
    fn test_parse_from_slice_unknown_discriminator() {
        let parser = PumpFunParser::default();

        let mut data = Vec::new();
        data.extend_from_slice(&0xDEADBEEFCAFEBABEu64.to_le_bytes()); // Unknown discriminator
        data.extend_from_slice(&[0u8; 16]); // Some dummy data

        let metadata = EventMetadata::default();
        let result = parser.parse_from_slice(&data, metadata);

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::UnknownDiscriminator { discriminator } => {
                assert_eq!(
                    discriminator,
                    vec![0xBE, 0xBA, 0xFE, 0xCA, 0xEF, 0xBE, 0xAD, 0xDE]
                );
            }
            _ => panic!("Expected UnknownDiscriminator error"),
        }
    }

    #[test]
    fn test_buy_instruction_parsing_standard_parser() {
        let parser = PumpFunParser::new_standard();

        let mut data = Vec::new();
        data.extend_from_slice(&0x66063d1201daebeau64.to_le_bytes()); // Buy discriminator
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount
        data.extend_from_slice(&2000u64.to_le_bytes()); // max_price_per_token

        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).unwrap();

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.protocol_type(), ProtocolType::PumpSwap);

        let parsed = event.get_parsed_data::<PumpBuyInstruction>().unwrap();
        assert_eq!(parsed.amount, 1000);
        assert_eq!(parsed.max_price_per_token, 2000);
    }

    #[test]
    fn test_create_pool_standard_parser() {
        let parser = PumpFunParser::new_standard();

        let mut data = Vec::new();
        data.extend_from_slice(&0x181ec828051c0777u64.to_le_bytes()); // CreatePool discriminator
        data.extend_from_slice(&[0u8; 32]); // name placeholder

        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).unwrap();

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.protocol_type(), ProtocolType::PumpSwap);
    }

    #[test]
    fn test_deposit_standard_parser() {
        let parser = PumpFunParser::new_standard();

        let mut data = Vec::new();
        data.extend_from_slice(&0xf223c68952e1f2b6u64.to_le_bytes()); // Deposit discriminator
        data.extend_from_slice(&3000u64.to_le_bytes()); // amount
        data.extend_from_slice(&2500u64.to_le_bytes()); // min_lp_tokens

        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).unwrap();

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.protocol_type(), ProtocolType::PumpSwap);

        let parsed = event.get_parsed_data::<PumpDepositInstruction>().unwrap();
        assert_eq!(parsed.amount, 3000);
        assert_eq!(parsed.min_lp_tokens, 2500);
    }

    #[test]
    fn test_withdraw_standard_parser() {
        let parser = PumpFunParser::new_standard();

        let mut data = Vec::new();
        data.extend_from_slice(&0x2e2d8d4ec43b1d79u64.to_le_bytes()); // Withdraw discriminator
        data.extend_from_slice(&4000u64.to_le_bytes()); // lp_tokens
        data.extend_from_slice(&3500u64.to_le_bytes()); // min_amount_out

        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).unwrap();

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.protocol_type(), ProtocolType::PumpSwap);

        let parsed = event.get_parsed_data::<PumpWithdrawInstruction>().unwrap();
        assert_eq!(parsed.lp_tokens, 4000);
        assert_eq!(parsed.min_amount_out, 3500);
    }

    #[test]
    fn test_set_params_standard_parser() {
        let parser = PumpFunParser::new_standard();

        let mut data = Vec::new();
        data.extend_from_slice(&0xa6f0308f80e7b5c9u64.to_le_bytes()); // SetParams discriminator
        data.extend_from_slice(&[0u8; 16]); // Some dummy parameter data

        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).unwrap();

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.protocol_type(), ProtocolType::PumpSwap);
    }

    #[test]
    fn test_can_parse_all_valid_discriminators() {
        let parser = PumpFunParser::default();

        let discriminators = [
            0x66063d1201daebeau64, // Buy
            0x33e685a4017f83adu64, // Sell
            0x181ec828051c0777u64, // CreatePool
            0xf223c68952e1f2b6u64, // Deposit
            0x2e2d8d4ec43b1d79u64, // Withdraw
            0xa6f0308f80e7b5c9u64, // SetParams
        ];

        for disc in discriminators.iter() {
            let mut data = Vec::new();
            data.extend_from_slice(&disc.to_le_bytes());
            assert!(
                parser.can_parse(&data),
                "Should be able to parse discriminator: {:x}",
                disc
            );
        }
    }

    #[test]
    fn test_can_parse_empty_data() {
        let parser = PumpFunParser::default();
        let data = vec![];
        assert!(!parser.can_parse(&data));
    }

    #[test]
    fn test_instruction_data_structures() {
        // Test PumpBuyInstruction
        let buy_instruction = PumpBuyInstruction {
            discriminator: 0x66063d1201daebeau64,
            amount: 1000,
            max_price_per_token: 2000,
        };
        assert_eq!(buy_instruction.discriminator, 0x66063d1201daebeau64);
        assert_eq!(buy_instruction.amount, 1000);
        assert_eq!(buy_instruction.max_price_per_token, 2000);

        // Test PumpSellInstruction
        let sell_instruction = PumpSellInstruction {
            discriminator: 0x33e685a4017f83adu64,
            amount: 5000,
            min_price_per_token: 1500,
        };
        assert_eq!(sell_instruction.discriminator, 0x33e685a4017f83adu64);
        assert_eq!(sell_instruction.amount, 5000);
        assert_eq!(sell_instruction.min_price_per_token, 1500);

        // Test PumpCreatePoolInstruction
        let create_pool_instruction = PumpCreatePoolInstruction {
            discriminator: 0x181ec828051c0777u64,
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            uri: "https://example.com".to_string(),
        };
        assert_eq!(create_pool_instruction.discriminator, 0x181ec828051c0777u64);
        assert_eq!(create_pool_instruction.name, "Test Token");
        assert_eq!(create_pool_instruction.symbol, "TEST");
        assert_eq!(create_pool_instruction.uri, "https://example.com");

        // Test PumpDepositInstruction
        let deposit_instruction = PumpDepositInstruction {
            discriminator: 0xf223c68952e1f2b6u64,
            amount: 3000,
            min_lp_tokens: 2500,
        };
        assert_eq!(deposit_instruction.discriminator, 0xf223c68952e1f2b6u64);
        assert_eq!(deposit_instruction.amount, 3000);
        assert_eq!(deposit_instruction.min_lp_tokens, 2500);

        // Test PumpWithdrawInstruction
        let withdraw_instruction = PumpWithdrawInstruction {
            discriminator: 0x2e2d8d4ec43b1d79u64,
            lp_tokens: 4000,
            min_amount_out: 3500,
        };
        assert_eq!(withdraw_instruction.discriminator, 0x2e2d8d4ec43b1d79u64);
        assert_eq!(withdraw_instruction.lp_tokens, 4000);
        assert_eq!(withdraw_instruction.min_amount_out, 3500);
    }

    #[test]
    fn test_discriminator_clone_and_debug() {
        let disc = PumpFunDiscriminator::Buy;
        let cloned = disc.clone();
        assert_eq!(disc, cloned);

        // Test debug format
        let debug_str = format!("{:?}", disc);
        assert!(debug_str.contains("Buy"));
    }

    #[test]
    fn test_program_id_constant() {
        assert_eq!(
            PUMP_FUN_PROGRAM_ID,
            "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"
        );
    }
}
