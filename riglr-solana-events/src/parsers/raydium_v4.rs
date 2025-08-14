//! High-performance Raydium AMM V4 parser with optimized instruction decoding
//! 
//! This parser provides zero-copy parsing for Raydium AMM V4 swap and liquidity operations
//! using discriminator-based instruction identification and custom deserialization.

use std::sync::Arc;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;
use crate::zero_copy::{ByteSliceEventParser, ParseError, CustomDeserializer, ZeroCopyEvent};
use crate::types::{EventMetadata, EventType, ProtocolType};
use crate::events::core::traits::UnifiedEvent;

/// Raydium AMM V4 program ID
pub const RAYDIUM_AMM_V4_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

/// Raydium AMM V4 instruction discriminators
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaydiumV4Discriminator {
    SwapBaseIn = 0x09,
    SwapBaseOut = 0x0a,
    Deposit = 0x03,
    Withdraw = 0x04,
    Initialize2 = 0x00,
    WithdrawPnl = 0x05,
}

impl RaydiumV4Discriminator {
    /// Parse discriminator from byte
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x09 => Some(Self::SwapBaseIn),
            0x0a => Some(Self::SwapBaseOut),
            0x03 => Some(Self::Deposit),
            0x04 => Some(Self::Withdraw),
            0x00 => Some(Self::Initialize2),
            0x05 => Some(Self::WithdrawPnl),
            _ => None,
        }
    }

    /// Get corresponding event type
    pub fn event_type(&self) -> EventType {
        match self {
            Self::SwapBaseIn | Self::SwapBaseOut => EventType::RaydiumAmmV4SwapBaseIn,
            Self::Deposit => EventType::RaydiumAmmV4Deposit,
            Self::Withdraw => EventType::RaydiumAmmV4Withdraw,
            Self::Initialize2 => EventType::RaydiumAmmV4Initialize2,
            Self::WithdrawPnl => EventType::RaydiumAmmV4WithdrawPnl,
        }
    }
}

/// Raydium V4 SwapBaseIn instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct SwapBaseInInstruction {
    /// Discriminator (should be 0x09)
    pub discriminator: u8,
    /// Amount to swap in
    pub amount_in: u64,
    /// Minimum amount out expected
    pub minimum_amount_out: u64,
}

/// Raydium V4 SwapBaseOut instruction data  
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct SwapBaseOutInstruction {
    /// Discriminator (should be 0x0a)
    pub discriminator: u8,
    /// Maximum amount in
    pub max_amount_in: u64,
    /// Amount out desired
    pub amount_out: u64,
}

/// Raydium V4 Deposit instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct DepositInstruction {
    /// Discriminator (should be 0x03)
    pub discriminator: u8,
    /// Maximum amount of token A to deposit
    pub max_coin_amount: u64,
    /// Maximum amount of token B to deposit  
    pub max_pc_amount: u64,
    /// Base side (true for coin, false for pc)
    pub base_side: u64,
}

/// Raydium V4 Withdraw instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct WithdrawInstruction {
    /// Discriminator (should be 0x04)
    pub discriminator: u8,
    /// Amount of LP tokens to burn
    pub amount: u64,
}

/// High-performance Raydium V4 parser
pub struct RaydiumV4Parser {
    /// Program ID for validation
    program_id: Pubkey,
    /// Enable zero-copy parsing
    zero_copy: bool,
}

impl RaydiumV4Parser {
    /// Create new Raydium V4 parser
    pub fn new() -> Self {
        Self {
            program_id: RAYDIUM_AMM_V4_PROGRAM_ID.parse().expect("Valid Raydium program ID"),
            zero_copy: true,
        }
    }

    /// Create parser with zero-copy disabled
    pub fn new_standard() -> Self {
        Self {
            program_id: RAYDIUM_AMM_V4_PROGRAM_ID.parse().expect("Valid Raydium program ID"),
            zero_copy: false,
        }
    }

    /// Parse SwapBaseIn instruction with zero-copy
    fn parse_swap_base_in<'a>(
        &self,
        data: &'a [u8],
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'a>, ParseError> {
        let mut deserializer = CustomDeserializer::new(data);
        
        // Skip discriminator (already validated)
        deserializer.skip(1)?;
        
        let amount_in = deserializer.read_u64_le()?;
        let minimum_amount_out = deserializer.read_u64_le()?;
        
        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };
        
        // Store parsed instruction data
        let instruction_data = SwapBaseInInstruction {
            discriminator: 0x09,
            amount_in,
            minimum_amount_out,
        };
        event.set_parsed_data(instruction_data);
        
        // Create JSON representation
        let json = serde_json::json!({
            "instruction_type": "swap_base_in",
            "amount_in": amount_in.to_string(),
            "minimum_amount_out": minimum_amount_out.to_string(),
            "protocol": "raydium_amm_v4"
        });
        event.set_json_data(json);
        
        Ok(event)
    }

    /// Parse SwapBaseOut instruction with zero-copy
    fn parse_swap_base_out<'a>(
        &self,
        data: &'a [u8],
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'a>, ParseError> {
        let mut deserializer = CustomDeserializer::new(data);
        
        // Skip discriminator
        deserializer.skip(1)?;
        
        let max_amount_in = deserializer.read_u64_le()?;
        let amount_out = deserializer.read_u64_le()?;
        
        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };
        
        let instruction_data = SwapBaseOutInstruction {
            discriminator: 0x0a,
            max_amount_in,
            amount_out,
        };
        event.set_parsed_data(instruction_data);
        
        let json = serde_json::json!({
            "instruction_type": "swap_base_out",
            "max_amount_in": max_amount_in.to_string(),
            "amount_out": amount_out.to_string(),
            "protocol": "raydium_amm_v4"
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
        
        deserializer.skip(1)?; // Skip discriminator
        
        let max_coin_amount = deserializer.read_u64_le()?;
        let max_pc_amount = deserializer.read_u64_le()?;
        let base_side = deserializer.read_u64_le()?;
        
        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };
        
        let instruction_data = DepositInstruction {
            discriminator: 0x03,
            max_coin_amount,
            max_pc_amount,
            base_side,
        };
        event.set_parsed_data(instruction_data);
        
        let json = serde_json::json!({
            "instruction_type": "deposit",
            "max_coin_amount": max_coin_amount.to_string(),
            "max_pc_amount": max_pc_amount.to_string(),
            "base_side": base_side,
            "protocol": "raydium_amm_v4"
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
        
        deserializer.skip(1)?; // Skip discriminator
        
        let amount = deserializer.read_u64_le()?;
        
        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };
        
        let instruction_data = WithdrawInstruction {
            discriminator: 0x04,
            amount,
        };
        event.set_parsed_data(instruction_data);
        
        let json = serde_json::json!({
            "instruction_type": "withdraw",
            "amount": amount.to_string(),
            "protocol": "raydium_amm_v4"
        });
        event.set_json_data(json);
        
        Ok(event)
    }
}

impl ByteSliceEventParser for RaydiumV4Parser {
    fn parse_from_slice<'a>(
        &self,
        data: &'a [u8],
        mut metadata: EventMetadata,
    ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        // Update metadata with protocol info
        metadata.protocol_type = ProtocolType::RaydiumAmmV4;
        
        // Parse discriminator
        let discriminator = RaydiumV4Discriminator::from_byte(data[0])
            .ok_or_else(|| ParseError::UnknownDiscriminator { 
                discriminator: vec![data[0]] 
            })?;

        // Update event type based on discriminator
        metadata.event_type = discriminator.event_type();

        let event = match discriminator {
            RaydiumV4Discriminator::SwapBaseIn => {
                self.parse_swap_base_in(data, metadata)?
            },
            RaydiumV4Discriminator::SwapBaseOut => {
                self.parse_swap_base_out(data, metadata)?
            },
            RaydiumV4Discriminator::Deposit => {
                self.parse_deposit(data, metadata)?
            },
            RaydiumV4Discriminator::Withdraw => {
                self.parse_withdraw(data, metadata)?
            },
            RaydiumV4Discriminator::Initialize2 => {
                // Handle initialize instruction
                let mut event = if self.zero_copy {
                    ZeroCopyEvent::new_borrowed(metadata, data)
                } else {
                    ZeroCopyEvent::new_owned(metadata, data.to_vec())
                };
                
                let json = serde_json::json!({
                    "instruction_type": "initialize2",
                    "protocol": "raydium_amm_v4"
                });
                event.set_json_data(json);
                event
            },
            RaydiumV4Discriminator::WithdrawPnl => {
                // Handle withdraw PNL instruction
                let mut event = if self.zero_copy {
                    ZeroCopyEvent::new_borrowed(metadata, data)
                } else {
                    ZeroCopyEvent::new_owned(metadata, data.to_vec())
                };
                
                let json = serde_json::json!({
                    "instruction_type": "withdraw_pnl",
                    "protocol": "raydium_amm_v4"
                });
                event.set_json_data(json);
                event
            },
        };

        Ok(vec![event])
    }

    fn can_parse(&self, data: &[u8]) -> bool {
        if data.is_empty() {
            return false;
        }
        
        RaydiumV4Discriminator::from_byte(data[0]).is_some()
    }

    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::RaydiumAmmV4
    }
}

/// Factory for creating Raydium V4 parsers
pub struct RaydiumV4ParserFactory;

impl RaydiumV4ParserFactory {
    /// Create a new high-performance zero-copy parser
    pub fn create_zero_copy() -> Arc<dyn ByteSliceEventParser> {
        Arc::new(RaydiumV4Parser::new())
    }

    /// Create a standard parser
    pub fn create_standard() -> Arc<dyn ByteSliceEventParser> {
        Arc::new(RaydiumV4Parser::new_standard())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EventMetadata;

    #[test]
    fn test_discriminator_parsing() {
        assert_eq!(RaydiumV4Discriminator::from_byte(0x09), Some(RaydiumV4Discriminator::SwapBaseIn));
        assert_eq!(RaydiumV4Discriminator::from_byte(0x0a), Some(RaydiumV4Discriminator::SwapBaseOut));
        assert_eq!(RaydiumV4Discriminator::from_byte(0xFF), None);
    }

    #[test]
    fn test_swap_base_in_parsing() {
        let parser = RaydiumV4Parser::new();
        
        // Create test data: discriminator (1 byte) + amount_in (8 bytes) + min_amount_out (8 bytes)
        let mut data = vec![0x09]; // SwapBaseIn discriminator
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount_in
        data.extend_from_slice(&900u64.to_le_bytes());  // minimum_amount_out
        
        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).unwrap();
        
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.protocol_type(), ProtocolType::RaydiumAmmV4);
        
        // Check parsed data
        let parsed = event.get_parsed_data::<SwapBaseInInstruction>().unwrap();
        assert_eq!(parsed.amount_in, 1000);
        assert_eq!(parsed.minimum_amount_out, 900);
    }

    #[test]
    fn test_can_parse() {
        let parser = RaydiumV4Parser::new();
        
        assert!(parser.can_parse(&[0x09])); // SwapBaseIn
        assert!(parser.can_parse(&[0x0a])); // SwapBaseOut
        assert!(!parser.can_parse(&[0xFF])); // Unknown
        assert!(!parser.can_parse(&[])); // Empty
    }

    #[test]
    fn test_factory() {
        let zero_copy_parser = RaydiumV4ParserFactory::create_zero_copy();
        let standard_parser = RaydiumV4ParserFactory::create_standard();
        
        assert_eq!(zero_copy_parser.protocol_type(), ProtocolType::RaydiumAmmV4);
        assert_eq!(standard_parser.protocol_type(), ProtocolType::RaydiumAmmV4);
    }
}