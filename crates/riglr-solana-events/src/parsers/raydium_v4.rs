//! High-performance Raydium AMM V4 parser with optimized instruction decoding
//!
//! This parser provides zero-copy parsing for Raydium AMM V4 swap and liquidity operations
//! using discriminator-based instruction identification and custom deserialization.

extern crate alloc;

use alloc::sync::Arc;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;
use std::sync::OnceLock;

use crate::metadata_helpers::{set_event_type, set_protocol_type};
use crate::solana_metadata::SolanaEventMetadata;
use crate::types::{EventType, ProtocolType};
use crate::zero_copy::{ByteSliceEventParser, CustomDeserializer, ParseError, ZeroCopyEvent};

/// Raydium AMM V4 program ID
pub const RAYDIUM_AMM_V4_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

/// Type alias for event metadata used in Raydium V4 parsing
type EventMetadata = SolanaEventMetadata;
// UnifiedEvent trait has been removed

/// Raydium AMM V4 instruction discriminators
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Discriminator {
    /// Swap with base input amount (discriminator 0x09)
    SwapBaseIn = 0x09,
    /// Swap with base output amount (discriminator 0x0a)
    SwapBaseOut = 0x0a,
    /// Deposit liquidity to pool (discriminator 0x03)
    Deposit = 0x03,
    /// Withdraw liquidity from pool (discriminator 0x04)
    Withdraw = 0x04,
    /// Initialize pool version 2 (discriminator 0x00)
    Initialize2 = 0x00,
    /// Withdraw profit and loss (discriminator 0x05)
    WithdrawPnl = 0x05,
}

impl Discriminator {
    /// Get corresponding event type
    #[must_use]
    #[inline]
    pub const fn event_type(&self) -> EventType {
        match *self {
            Self::SwapBaseIn | Self::SwapBaseOut => EventType::RaydiumAmmV4SwapBaseIn,
            Self::Deposit => EventType::RaydiumAmmV4Deposit,
            Self::Withdraw => EventType::RaydiumAmmV4Withdraw,
            Self::Initialize2 => EventType::RaydiumAmmV4Initialize2,
            Self::WithdrawPnl => EventType::RaydiumAmmV4WithdrawPnl,
        }
    }

    /// Parse discriminator from byte
    #[must_use]
    #[inline]
    pub const fn from_byte(byte: u8) -> Option<Self> {
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
}

/// Raydium V4 `SwapBaseIn` instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[non_exhaustive]
pub struct SwapBaseInInstruction {
    /// Amount to swap in
    pub amount_in: u64,
    /// Discriminator (should be 0x09)
    pub discriminator: u8,
    /// Minimum amount out expected
    pub minimum_amount_out: u64,
}

/// Raydium V4 `SwapBaseOut` instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[non_exhaustive]
pub struct SwapBaseOutInstruction {
    /// Amount out desired
    pub amount_out: u64,
    /// Discriminator (should be 0x0a)
    pub discriminator: u8,
    /// Maximum amount in
    pub max_amount_in: u64,
}

/// Raydium V4 Deposit instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[non_exhaustive]
pub struct DepositInstruction {
    /// Base side (true for coin, false for pc)
    pub base_side: u64,
    /// Discriminator (should be 0x03)
    pub discriminator: u8,
    /// Maximum amount of token A to deposit
    pub max_coin_amount: u64,
    /// Maximum amount of token B to deposit
    pub max_pc_amount: u64,
}

/// Raydium V4 Withdraw instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[non_exhaustive]
pub struct WithdrawInstruction {
    /// Amount of LP tokens to burn
    pub amount: u64,
    /// Discriminator (should be 0x04)
    pub discriminator: u8,
}

/// High-performance Raydium V4 parser
#[derive(Debug)]
pub struct Parser {
    /// Program ID for validation
    #[allow(dead_code)]
    program_id: Pubkey,
    /// Enable zero-copy parsing
    zero_copy: bool,
}

/// Get the Raydium AMM V4 program ID as a Pubkey
static RAYDIUM_V4_PUBKEY: OnceLock<Pubkey> = OnceLock::new();
fn get_raydium_v4_program_id() -> Pubkey {
    *RAYDIUM_V4_PUBKEY.get_or_init(|| {
        RAYDIUM_AMM_V4_PROGRAM_ID
            .parse()
            .unwrap_or_else(|_| Pubkey::default())
    })
}

impl Default for Parser {
    #[inline]
    fn default() -> Self {
        Self {
            program_id: get_raydium_v4_program_id(),
            zero_copy: true,
        }
    }
}

impl Parser {
    /// Create new Raydium V4 parser with zero-copy enabled
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded `RAYDIUM_AMM_V4_PROGRAM_ID` cannot be parsed as a valid Pubkey.
    /// This should never happen as the ID is a known constant.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            program_id: get_raydium_v4_program_id(),
            zero_copy: true,
        }
    }

    /// Create parser with zero-copy disabled
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded `RAYDIUM_AMM_V4_PROGRAM_ID` cannot be parsed as a valid Pubkey.
    /// This should never happen as the ID is a known constant.
    #[must_use]
    #[inline]
    pub fn new_standard() -> Self {
        Self {
            program_id: get_raydium_v4_program_id(),
            zero_copy: false,
        }
    }

    /// Parse Deposit instruction
    fn parse_deposit<'instruction_lifetime>(
        &self,
        data: &'instruction_lifetime [u8],
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'instruction_lifetime>, ParseError> {
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

    /// Parse `SwapBaseIn` instruction with zero-copy
    fn parse_swap_base_in<'instruction_lifetime>(
        &self,
        data: &'instruction_lifetime [u8],
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'instruction_lifetime>, ParseError> {
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

    /// Parse `SwapBaseOut` instruction with zero-copy
    fn parse_swap_base_out<'instruction_lifetime>(
        &self,
        data: &'instruction_lifetime [u8],
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'instruction_lifetime>, ParseError> {
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

    /// Parse Withdraw instruction
    fn parse_withdraw<'instruction_lifetime>(
        &self,
        data: &'instruction_lifetime [u8],
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'instruction_lifetime>, ParseError> {
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

impl ByteSliceEventParser for Parser {
    #[inline]
    fn can_parse(&self, data: &[u8]) -> bool {
        if data.is_empty() {
            return false;
        }

        data.first()
            .and_then(|&byte| Discriminator::from_byte(byte))
            .is_some()
    }

    #[inline]
    fn parse_from_slice<'instruction_lifetime>(
        &self,
        data: &'instruction_lifetime [u8],
        mut metadata: EventMetadata,
    ) -> Result<Vec<ZeroCopyEvent<'instruction_lifetime>>, ParseError> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        // Update metadata with protocol info
        set_protocol_type(&mut metadata.core, &ProtocolType::RaydiumAmmV4);

        // Parse discriminator
        let first_byte = data.first().ok_or_else(|| {
            ParseError::InvalidInstructionData("Missing discriminator byte".to_owned())
        })?;

        let discriminator = Discriminator::from_byte(*first_byte).ok_or_else(|| {
            ParseError::UnknownDiscriminator {
                discriminator: vec![*first_byte],
            }
        })?;

        // Update event type based on discriminator
        set_event_type(&mut metadata.core, &discriminator.event_type());

        let event = match discriminator {
            Discriminator::SwapBaseIn => self.parse_swap_base_in(data, metadata)?,
            Discriminator::SwapBaseOut => self.parse_swap_base_out(data, metadata)?,
            Discriminator::Deposit => self.parse_deposit(data, metadata)?,
            Discriminator::Withdraw => self.parse_withdraw(data, metadata)?,
            Discriminator::Initialize2 => {
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
            }
            Discriminator::WithdrawPnl => {
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
            }
        };

        Ok(vec![event])
    }

    #[inline]
    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::RaydiumAmmV4
    }
}

/// Factory for creating Raydium V4 parsers
#[derive(Debug)]
#[non_exhaustive]
pub struct ParserFactory;

impl ParserFactory {
    /// Create a standard parser
    #[must_use]
    #[inline]
    pub fn create_standard() -> Arc<dyn ByteSliceEventParser> {
        Arc::new(Parser::new_standard())
    }

    /// Create a new high-performance zero-copy parser
    #[must_use]
    #[inline]
    pub fn create_zero_copy() -> Arc<dyn ByteSliceEventParser> {
        Arc::new(Parser::default())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_discriminator_parsing() {
        assert_eq!(
            Discriminator::from_byte(0x09),
            Some(Discriminator::SwapBaseIn)
        );
        assert_eq!(
            Discriminator::from_byte(0x0a),
            Some(Discriminator::SwapBaseOut)
        );
        assert_eq!(Discriminator::from_byte(0xFF), None);
    }

    #[test]
    fn test_swap_base_in_parsing() {
        let parser = Parser::default();

        // Create test data: discriminator (1 byte) + amount_in (8 bytes) + min_amount_out (8 bytes)
        let mut data = vec![0x09]; // SwapBaseIn discriminator
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount_in
        data.extend_from_slice(&900u64.to_le_bytes()); // minimum_amount_out

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse SwapBaseIn instruction data");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.protocol_type(), ProtocolType::RaydiumAmmV4);

        // Check parsed data
        let parsed = event
            .get_parsed_data::<SwapBaseInInstruction>()
            .expect("Failed to get parsed SwapBaseInInstruction data");
        assert_eq!(parsed.amount_in, 1000);
        assert_eq!(parsed.minimum_amount_out, 900);
    }

    #[test]
    fn test_can_parse() {
        let parser = Parser::default();

        assert!(parser.can_parse(&[0x09])); // SwapBaseIn
        assert!(parser.can_parse(&[0x0a])); // SwapBaseOut
        assert!(!parser.can_parse(&[0xFF])); // Unknown
        assert!(!parser.can_parse(&[])); // Empty
    }

    #[test]
    fn test_factory() {
        let zero_copy_parser = ParserFactory::create_zero_copy();
        let standard_parser = ParserFactory::create_standard();

        assert_eq!(zero_copy_parser.protocol_type(), ProtocolType::RaydiumAmmV4);
        assert_eq!(standard_parser.protocol_type(), ProtocolType::RaydiumAmmV4);
    }

    // Additional comprehensive tests for 100% coverage

    #[test]
    fn test_discriminator_from_byte_all_variants() {
        // Test all valid discriminators
        assert_eq!(
            Discriminator::from_byte(0x09),
            Some(Discriminator::SwapBaseIn)
        );
        assert_eq!(
            Discriminator::from_byte(0x0a),
            Some(Discriminator::SwapBaseOut)
        );
        assert_eq!(Discriminator::from_byte(0x03), Some(Discriminator::Deposit));
        assert_eq!(
            Discriminator::from_byte(0x04),
            Some(Discriminator::Withdraw)
        );
        assert_eq!(
            Discriminator::from_byte(0x00),
            Some(Discriminator::Initialize2)
        );
        assert_eq!(
            Discriminator::from_byte(0x05),
            Some(Discriminator::WithdrawPnl)
        );

        // Test invalid discriminators
        assert_eq!(Discriminator::from_byte(0x01), None);
        assert_eq!(Discriminator::from_byte(0x02), None);
        assert_eq!(Discriminator::from_byte(0x06), None);
        assert_eq!(Discriminator::from_byte(0xFF), None);
    }

    #[test]
    fn test_discriminator_event_type() {
        assert_eq!(
            Discriminator::SwapBaseIn.event_type(),
            EventType::RaydiumAmmV4SwapBaseIn
        );
        assert_eq!(
            Discriminator::SwapBaseOut.event_type(),
            EventType::RaydiumAmmV4SwapBaseIn
        );
        assert_eq!(
            Discriminator::Deposit.event_type(),
            EventType::RaydiumAmmV4Deposit
        );
        assert_eq!(
            Discriminator::Withdraw.event_type(),
            EventType::RaydiumAmmV4Withdraw
        );
        assert_eq!(
            Discriminator::Initialize2.event_type(),
            EventType::RaydiumAmmV4Initialize2
        );
        assert_eq!(
            Discriminator::WithdrawPnl.event_type(),
            EventType::RaydiumAmmV4WithdrawPnl
        );
    }

    #[test]
    fn test_raydium_v4_parser_default() {
        let parser = Parser::default();
        assert_eq!(parser.program_id.to_string(), RAYDIUM_AMM_V4_PROGRAM_ID);
        assert!(parser.zero_copy);
    }

    #[test]
    fn test_raydium_v4_parser_new_standard() {
        let parser = Parser::new_standard();
        assert_eq!(parser.program_id.to_string(), RAYDIUM_AMM_V4_PROGRAM_ID);
        assert!(!parser.zero_copy);
    }

    #[test]
    fn test_swap_base_in_parsing_zero_copy_disabled() {
        let parser = Parser::new_standard();

        let mut data = vec![0x09];
        data.extend_from_slice(&u64::MAX.to_le_bytes());
        data.extend_from_slice(&0u64.to_le_bytes());

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse SwapBaseIn instruction with zero copy disabled");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");

        let parsed = event
            .get_parsed_data::<SwapBaseInInstruction>()
            .expect("Failed to get parsed SwapBaseInInstruction data");
        assert_eq!(parsed.discriminator, 0x09);
        assert_eq!(parsed.amount_in, u64::MAX);
        assert_eq!(parsed.minimum_amount_out, 0);
    }

    #[test]
    fn test_swap_base_out_parsing() {
        let parser = Parser::default();

        let mut data = vec![0x0a];
        data.extend_from_slice(&2000u64.to_le_bytes());
        data.extend_from_slice(&1800u64.to_le_bytes());

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse SwapBaseOut instruction data");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.protocol_type(), ProtocolType::RaydiumAmmV4);

        let parsed = event
            .get_parsed_data::<SwapBaseOutInstruction>()
            .expect("Failed to get parsed SwapBaseOutInstruction data");
        assert_eq!(parsed.discriminator, 0x0a);
        assert_eq!(parsed.max_amount_in, 2000);
        assert_eq!(parsed.amount_out, 1800);
    }

    #[test]
    fn test_swap_base_out_parsing_zero_copy_disabled() {
        let parser = Parser::new_standard();

        let mut data = vec![0x0a];
        data.extend_from_slice(&0u64.to_le_bytes());
        data.extend_from_slice(&u64::MAX.to_le_bytes());

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse SwapBaseOut instruction with zero copy disabled");

        assert_eq!(events.len(), 1);
        let parsed = events
            .first()
            .expect("Expected at least one event")
            .get_parsed_data::<SwapBaseOutInstruction>()
            .expect("Failed to get parsed SwapBaseOutInstruction data");
        assert_eq!(parsed.max_amount_in, 0);
        assert_eq!(parsed.amount_out, u64::MAX);
    }

    #[test]
    fn test_deposit_parsing() {
        let parser = Parser::default();

        let mut data = vec![0x03];
        data.extend_from_slice(&5000u64.to_le_bytes());
        data.extend_from_slice(&3000u64.to_le_bytes());
        data.extend_from_slice(&1u64.to_le_bytes());

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse Deposit instruction data");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.protocol_type(), ProtocolType::RaydiumAmmV4);

        let parsed = event
            .get_parsed_data::<DepositInstruction>()
            .expect("Failed to get parsed DepositInstruction data");
        assert_eq!(parsed.discriminator, 0x03);
        assert_eq!(parsed.max_coin_amount, 5000);
        assert_eq!(parsed.max_pc_amount, 3000);
        assert_eq!(parsed.base_side, 1);
    }

    #[test]
    fn test_deposit_parsing_zero_copy_disabled() {
        let parser = Parser::new_standard();

        let mut data = vec![0x03];
        data.extend_from_slice(&u64::MAX.to_le_bytes());
        data.extend_from_slice(&0u64.to_le_bytes());
        data.extend_from_slice(&0u64.to_le_bytes());

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse Deposit instruction with zero copy disabled");

        assert_eq!(events.len(), 1);
        let parsed = events
            .first()
            .expect("Expected at least one event")
            .get_parsed_data::<DepositInstruction>()
            .expect("Failed to get parsed DepositInstruction data");
        assert_eq!(parsed.max_coin_amount, u64::MAX);
        assert_eq!(parsed.max_pc_amount, 0);
        assert_eq!(parsed.base_side, 0);
    }

    #[test]
    fn test_withdraw_parsing() {
        let parser = Parser::default();

        let mut data = vec![0x04];
        data.extend_from_slice(&1500u64.to_le_bytes());

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse Withdraw instruction data");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.protocol_type(), ProtocolType::RaydiumAmmV4);

        let parsed = event
            .get_parsed_data::<WithdrawInstruction>()
            .expect("Failed to get parsed WithdrawInstruction data");
        assert_eq!(parsed.discriminator, 0x04);
        assert_eq!(parsed.amount, 1500);
    }

    #[test]
    fn test_withdraw_parsing_zero_copy_disabled() {
        let parser = Parser::new_standard();

        let mut data = vec![0x04];
        data.extend_from_slice(&u64::MAX.to_le_bytes());

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse Withdraw instruction with zero copy disabled");

        assert_eq!(events.len(), 1);
        let parsed = events
            .first()
            .expect("Expected at least one event")
            .get_parsed_data::<WithdrawInstruction>()
            .expect("Failed to get parsed WithdrawInstruction data");
        assert_eq!(parsed.amount, u64::MAX);
    }

    #[test]
    fn test_initialize2_parsing() {
        let parser = Parser::default();

        let data = vec![0x00];

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse Initialize2 instruction data");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.protocol_type(), ProtocolType::RaydiumAmmV4);
    }

    #[test]
    fn test_initialize2_parsing_zero_copy_disabled() {
        let parser = Parser::new_standard();

        let data = vec![0x00];

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse Initialize2 instruction with zero copy disabled");

        assert_eq!(events.len(), 1);
        assert_eq!(
            events
                .first()
                .expect("Expected at least one event")
                .protocol_type(),
            ProtocolType::RaydiumAmmV4
        );
    }

    #[test]
    fn test_withdraw_pnl_parsing() {
        let parser = Parser::default();

        let data = vec![0x05];

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse WithdrawPnl instruction data");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.protocol_type(), ProtocolType::RaydiumAmmV4);
    }

    #[test]
    fn test_withdraw_pnl_parsing_zero_copy_disabled() {
        let parser = Parser::new_standard();

        let data = vec![0x05];

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse WithdrawPnl instruction with zero copy disabled");

        assert_eq!(events.len(), 1);
        assert_eq!(
            events
                .first()
                .expect("Expected at least one event")
                .protocol_type(),
            ProtocolType::RaydiumAmmV4
        );
    }

    #[test]
    fn test_parse_from_slice_empty_data() {
        let parser = Parser::default();
        let metadata = EventMetadata::default();

        let result = parser
            .parse_from_slice(&[], metadata)
            .expect("Failed to parse empty data slice");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_from_slice_unknown_discriminator() {
        let parser = Parser::default();
        let data = vec![0xFF]; // Unknown discriminator
        let metadata = EventMetadata::default();

        let result = parser.parse_from_slice(&data, metadata);
        assert!(result.is_err());

        match result.expect_err("Expected parse to fail with unknown discriminator") {
            ParseError::UnknownDiscriminator { discriminator } => {
                assert_eq!(discriminator, vec![0xFF]);
            }
            _ => panic!("Expected UnknownDiscriminator error"),
        }
    }

    #[test]
    fn test_can_parse_all_valid_discriminators() {
        let parser = Parser::default();

        assert!(parser.can_parse(&[0x09])); // SwapBaseIn
        assert!(parser.can_parse(&[0x0a])); // SwapBaseOut
        assert!(parser.can_parse(&[0x03])); // Deposit
        assert!(parser.can_parse(&[0x04])); // Withdraw
        assert!(parser.can_parse(&[0x00])); // Initialize2
        assert!(parser.can_parse(&[0x05])); // WithdrawPnl
    }

    #[test]
    fn test_can_parse_invalid_cases() {
        let parser = Parser::default();

        assert!(!parser.can_parse(&[])); // Empty
        assert!(!parser.can_parse(&[0x01])); // Invalid
        assert!(!parser.can_parse(&[0x02])); // Invalid
        assert!(!parser.can_parse(&[0x06])); // Invalid
        assert!(!parser.can_parse(&[0xFF])); // Invalid
    }

    #[test]
    fn test_protocol_type() {
        let parser = Parser::default();
        assert_eq!(parser.protocol_type(), ProtocolType::RaydiumAmmV4);
    }

    #[test]
    fn test_instruction_serialization_deserialization() {
        // Test SwapBaseInInstruction
        let swap_in = SwapBaseInInstruction {
            discriminator: 0x09,
            amount_in: 1000,
            minimum_amount_out: 900,
        };
        let serialized =
            borsh::to_vec(&swap_in).expect("Failed to serialize SwapBaseInInstruction");
        let deserialized: SwapBaseInInstruction =
            borsh::from_slice(&serialized).expect("Failed to deserialize SwapBaseInInstruction");
        assert_eq!(deserialized.discriminator, swap_in.discriminator);
        assert_eq!(deserialized.amount_in, swap_in.amount_in);
        assert_eq!(deserialized.minimum_amount_out, swap_in.minimum_amount_out);

        // Test SwapBaseOutInstruction
        let swap_out = SwapBaseOutInstruction {
            discriminator: 0x0a,
            max_amount_in: 2000,
            amount_out: 1800,
        };
        let serialized =
            borsh::to_vec(&swap_out).expect("Failed to serialize SwapBaseOutInstruction");
        let deserialized: SwapBaseOutInstruction =
            borsh::from_slice(&serialized).expect("Failed to deserialize SwapBaseOutInstruction");
        assert_eq!(deserialized.discriminator, swap_out.discriminator);
        assert_eq!(deserialized.max_amount_in, swap_out.max_amount_in);
        assert_eq!(deserialized.amount_out, swap_out.amount_out);

        // Test DepositInstruction
        let deposit = DepositInstruction {
            discriminator: 0x03,
            max_coin_amount: 5000,
            max_pc_amount: 3000,
            base_side: 1,
        };
        let serialized = borsh::to_vec(&deposit).expect("Failed to serialize DepositInstruction");
        let deserialized: DepositInstruction =
            borsh::from_slice(&serialized).expect("Failed to deserialize DepositInstruction");
        assert_eq!(deserialized.discriminator, deposit.discriminator);
        assert_eq!(deserialized.max_coin_amount, deposit.max_coin_amount);
        assert_eq!(deserialized.max_pc_amount, deposit.max_pc_amount);
        assert_eq!(deserialized.base_side, deposit.base_side);

        // Test WithdrawInstruction
        let withdraw = WithdrawInstruction {
            discriminator: 0x04,
            amount: 1500,
        };
        let serialized = borsh::to_vec(&withdraw).expect("Failed to serialize WithdrawInstruction");
        let deserialized: WithdrawInstruction =
            borsh::from_slice(&serialized).expect("Failed to deserialize WithdrawInstruction");
        assert_eq!(deserialized.discriminator, withdraw.discriminator);
        assert_eq!(deserialized.amount, withdraw.amount);
    }

    #[test]
    fn test_discriminator_clone_debug_partialeq() {
        let disc1 = Discriminator::SwapBaseIn;
        let disc2 = disc1;

        assert_eq!(disc1, disc2);
        assert_eq!(format!("{disc1:?}"), "SwapBaseIn");
    }

    #[test]
    fn test_instruction_clone_debug() {
        let swap_in = SwapBaseInInstruction {
            discriminator: 0x09,
            amount_in: 1000,
            minimum_amount_out: 900,
        };
        let cloned = swap_in.clone();
        assert_eq!(cloned.amount_in, swap_in.amount_in);

        // Test debug formatting exists
        let debug_str = format!("{swap_in:?}");
        assert!(debug_str.contains("SwapBaseInInstruction"));
    }

    #[test]
    fn test_parse_insufficient_data_error() {
        let parser = Parser::default();

        // Test with insufficient data for SwapBaseIn (needs 17 bytes: 1 + 8 + 8)
        let data = vec![0x09, 0x01]; // Only 2 bytes
        let metadata = EventMetadata::default();

        let result = parser.parse_from_slice(&data, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_edge_case_values() {
        let parser = Parser::default();

        // Test with zero values
        let mut data = vec![0x09];
        data.extend_from_slice(&0u64.to_le_bytes());
        data.extend_from_slice(&0u64.to_le_bytes());

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse edge case values");

        let parsed = events
            .first()
            .expect("Expected at least one event")
            .get_parsed_data::<SwapBaseInInstruction>()
            .expect("Failed to get parsed SwapBaseInInstruction data from edge case test");
        assert_eq!(parsed.amount_in, 0);
        assert_eq!(parsed.minimum_amount_out, 0);
    }
}
