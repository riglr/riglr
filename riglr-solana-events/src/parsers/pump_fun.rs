//! High-performance PumpFun parser with specialized event extraction
//! 
//! This parser handles PumpFun protocol operations with optimized parsing
//! for trading events and pool operations.

use std::sync::Arc;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;
use crate::zero_copy::{ByteSliceEventParser, ParseError, CustomDeserializer, ZeroCopyEvent};
use crate::types::{EventMetadata, EventType, ProtocolType};
use crate::events::core::traits::UnifiedEvent;

/// PumpFun program ID
pub const PUMP_FUN_PROGRAM_ID: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";

/// PumpFun instruction discriminators  
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PumpFunDiscriminator {
    Buy,
    Sell,
    CreatePool,
    Deposit,
    Withdraw,
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
pub struct PumpFunParser {
    /// Program ID for validation
    program_id: Pubkey,
    /// Enable zero-copy parsing
    zero_copy: bool,
}

impl PumpFunParser {
    /// Create new PumpFun parser
    pub fn new() -> Self {
        Self {
            program_id: PUMP_FUN_PROGRAM_ID.parse().expect("Valid PumpFun program ID"),
            zero_copy: true,
        }
    }

    /// Create parser with zero-copy disabled
    pub fn new_standard() -> Self {
        Self {
            program_id: PUMP_FUN_PROGRAM_ID.parse().expect("Valid PumpFun program ID"),
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
        metadata.protocol_type = ProtocolType::PumpSwap;
        
        // Parse 8-byte discriminator
        let discriminator_bytes = &data[0..8];
        let discriminator = u64::from_le_bytes([
            discriminator_bytes[0], discriminator_bytes[1], discriminator_bytes[2], discriminator_bytes[3],
            discriminator_bytes[4], discriminator_bytes[5], discriminator_bytes[6], discriminator_bytes[7],
        ]);
        
        let discriminator_enum = PumpFunDiscriminator::from_u64(discriminator)
            .ok_or_else(|| ParseError::UnknownDiscriminator { 
                discriminator: discriminator_bytes.to_vec() 
            })?;

        // Update event type based on discriminator
        metadata.event_type = discriminator_enum.event_type();

        let event = match discriminator_enum {
            PumpFunDiscriminator::Buy => {
                self.parse_buy(data, metadata)?
            },
            PumpFunDiscriminator::Sell => {
                self.parse_sell(data, metadata)?
            },
            PumpFunDiscriminator::CreatePool => {
                self.parse_create_pool(data, metadata)?
            },
            PumpFunDiscriminator::Deposit => {
                self.parse_deposit(data, metadata)?
            },
            PumpFunDiscriminator::Withdraw => {
                self.parse_withdraw(data, metadata)?
            },
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
            },
        };

        Ok(vec![event])
    }

    fn can_parse(&self, data: &[u8]) -> bool {
        if data.len() < 8 {
            return false;
        }
        
        let discriminator_bytes = &data[0..8];
        let discriminator = u64::from_le_bytes([
            discriminator_bytes[0], discriminator_bytes[1], discriminator_bytes[2], discriminator_bytes[3],
            discriminator_bytes[4], discriminator_bytes[5], discriminator_bytes[6], discriminator_bytes[7],
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
        Arc::new(PumpFunParser::new())
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
        let parser = PumpFunParser::new();
        
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
        let parser = PumpFunParser::new();
        
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
}