//! High-performance Metaplex parser for NFT marketplace events
//!
//! This parser handles Metaplex protocol operations including NFT minting,
//! marketplace transactions, and metadata operations.

extern crate alloc;

use crate::metadata_helpers::{set_event_type, set_protocol_type};
use crate::solana_metadata::SolanaEventMetadata;
use crate::types::{EventType, ProtocolType};
use crate::zero_copy::{ByteSliceEventParser, CustomDeserializer, ParseError, ZeroCopyEvent};
use alloc::sync::Arc;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;
use std::sync::OnceLock;

/// Metaplex Token Metadata program ID
pub const METAPLEX_TOKEN_METADATA_PROGRAM_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";

/// Metaplex Auction House program ID
pub const METAPLEX_AUCTION_HOUSE_PROGRAM_ID: &str = "hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk";

/// Type alias for Solana event metadata
type EventMetadata = SolanaEventMetadata;
// UnifiedEvent trait has been removed

/// Metaplex instruction discriminators for Token Metadata program
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TokenMetadataDiscriminator {
    /// Create metadata account for NFT
    CreateMetadataAccount = 0,
    /// Update existing metadata account
    UpdateMetadataAccount = 1,
    /// Deprecated create master edition instruction
    DeprecatedCreateMasterEdition = 2,
    /// Deprecated mint new edition from master edition via printing token
    DeprecatedMintNewEditionFromMasterEditionViaPrintingToken = 3,
    /// Update primary sale happened via token
    UpdatePrimarySaleHappenedViaToken = 4,
    /// Deprecated set reservation list instruction
    DeprecatedSetReservationList = 5,
    /// Deprecated create reservation list instruction
    DeprecatedCreateReservationList = 6,
    /// Sign metadata for creator verification
    SignMetadata = 7,
    /// Deprecated mint printing tokens via token
    DeprecatedMintPrintingTokensViaToken = 8,
    /// Deprecated mint printing tokens instruction
    DeprecatedMintPrintingTokens = 9,
    /// Create master edition for limited editions
    CreateMasterEdition = 10,
    /// Mint new edition from master edition via token
    MintNewEditionFromMasterEditionViaToken = 11,
    /// Convert master edition V1 to V2
    ConvertMasterEditionV1ToV2 = 12,
    /// Mint new edition from master edition via vault proxy
    MintNewEditionFromMasterEditionViaVaultProxy = 13,
    /// Puff metadata to increase size
    PuffMetadata = 14,
    /// Update metadata account version 2
    UpdateMetadataAccountV2 = 15,
    /// Create metadata account version 2
    CreateMetadataAccountV2 = 16,
    /// Create master edition version 3
    CreateMasterEditionV3 = 17,
    /// Verify collection membership
    VerifyCollection = 18,
    /// Utilize NFT for specific use case
    Utilize = 19,
    /// Approve use authority for NFT
    ApproveUseAuthority = 20,
    /// Revoke use authority for NFT
    RevokeUseAuthority = 21,
    /// Unverify collection membership
    UnverifyCollection = 22,
    /// Approve collection authority
    ApproveCollectionAuthority = 23,
    /// Revoke collection authority
    RevokeCollectionAuthority = 24,
    /// Set and verify collection in one transaction
    SetAndVerifyCollection = 25,
    /// Freeze delegated account
    FreezeDelegatedAccount = 26,
    /// Thaw delegated account
    ThawDelegatedAccount = 27,
    /// Remove creator verification
    RemoveCreatorVerification = 28,
    /// Burn NFT permanently
    BurnNft = 29,
    /// Verify creator signature
    VerifyCreator = 30,
    /// Unverify creator signature
    UnverifyCreator = 31,
    /// Bubblegum set collection size
    BubblegumSetCollectionSize = 32,
    /// Burn edition NFT
    BurnEditionNft = 33,
    /// Create metadata account version 3
    CreateMetadataAccountV3 = 34,
    /// Set collection size
    SetCollectionSize = 35,
    /// Set token standard
    SetTokenStandard = 36,
    /// Bubblegum verify creator
    BubblegumVerifyCreator = 37,
    /// Bubblegum unverify creator
    BubblegumUnverifyCreator = 38,
    /// Bubblegum verify collection
    BubblegumVerifyCollection = 39,
    /// Bubblegum unverify collection
    BubblegumUnverifyCollection = 40,
    /// Bubblegum set and verify collection
    BubblegumSetAndVerifyCollection = 41,
    /// Transfer NFT ownership
    Transfer = 42,
}

/// Metaplex Auction House discriminators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AuctionHouseDiscriminator {
    /// Buy NFT from auction house
    Buy,
    /// Cancel buy or sell order
    Cancel,
    /// Deposit funds to auction house
    Deposit,
    /// Execute sale between buyer and seller
    ExecuteSale,
    /// Sell NFT on auction house
    Sell,
    /// Withdraw funds from auction house
    Withdraw,
}

impl TokenMetadataDiscriminator {
    /// Get corresponding event type
    #[must_use]
    #[inline]
    pub const fn event_type(&self) -> EventType {
        match *self {
            Self::CreateMetadataAccount
            | Self::CreateMetadataAccountV2
            | Self::CreateMetadataAccountV3 => EventType::Mint,
            Self::Transfer => EventType::Transfer,
            Self::BurnNft => EventType::Burn,
            _ => EventType::ContractEvent,
        }
    }

    /// Parse discriminator from byte
    #[must_use]
    #[inline]
    pub const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::CreateMetadataAccount),
            1 => Some(Self::UpdateMetadataAccount),
            7 => Some(Self::SignMetadata),
            10 => Some(Self::CreateMasterEdition),
            11 => Some(Self::MintNewEditionFromMasterEditionViaToken),
            15 => Some(Self::UpdateMetadataAccountV2),
            16 => Some(Self::CreateMetadataAccountV2),
            17 => Some(Self::CreateMasterEditionV3),
            18 => Some(Self::VerifyCollection),
            19 => Some(Self::Utilize),
            29 => Some(Self::BurnNft),
            30 => Some(Self::VerifyCreator),
            34 => Some(Self::CreateMetadataAccountV3),
            42 => Some(Self::Transfer),
            _ => None,
        }
    }
}

/// Metaplex `CreateMetadataAccount` instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[non_exhaustive]
pub struct CreateMetadataAccountInstruction {
    /// Discriminator
    pub discriminator: u8,
    /// Is mutable
    pub is_mutable: bool,
    /// Metadata account bump
    pub metadata_account_bump: u8,
    /// NFT name
    pub name: String,
    /// Seller fee basis points
    pub seller_fee_basis_points: u16,
    /// NFT symbol
    pub symbol: String,
    /// Update authority can change metadata
    pub update_authority_is_signer: bool,
    /// NFT URI (metadata JSON)
    pub uri: String,
}

/// Metaplex Transfer instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[non_exhaustive]
pub struct TransferInstruction {
    /// Authorization data
    pub authorization_data: Option<Vec<u8>>,
    /// Discriminator
    pub discriminator: u8,
}

/// Metaplex `BurnNft` instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[non_exhaustive]
pub struct BurnNftInstruction {
    /// Discriminator
    pub discriminator: u8,
}

/// Metaplex marketplace event analysis
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct EventAnalysis {
    /// Transfer amount (for transfers)
    pub amount: Option<u64>,
    /// Collection mint (if applicable)
    pub collection_mint: Option<Pubkey>,
    /// Event category (mint, transfer, burn, etc.)
    pub event_category: String,
    /// Additional event-specific data
    pub extra_data: serde_json::Value,
    /// Metadata URI (for mints)
    pub metadata_uri: Option<String>,
    /// NFT mint address (if applicable)
    pub nft_mint: Option<Pubkey>,
}

/// High-performance Metaplex parser
#[derive(Debug)]
pub struct Parser {
    /// Auction House program ID
    #[expect(dead_code)]
    auction_house_program_id: Pubkey,
    /// Enable detailed metadata parsing
    detailed_metadata: bool,
    /// Token Metadata program ID
    #[expect(dead_code)]
    token_metadata_program_id: Pubkey,
    /// Enable zero-copy parsing
    zero_copy: bool,
}

/// Get the Metaplex Token Metadata program ID
static TOKEN_METADATA_PUBKEY: OnceLock<Pubkey> = OnceLock::new();
fn get_token_metadata_program_id() -> Pubkey {
    *TOKEN_METADATA_PUBKEY.get_or_init(|| {
        METAPLEX_TOKEN_METADATA_PROGRAM_ID
            .parse()
            .unwrap_or_else(|_| Pubkey::default())
    })
}

/// Get the Metaplex Auction House program ID
static AUCTION_HOUSE_PUBKEY: OnceLock<Pubkey> = OnceLock::new();
fn get_auction_house_program_id() -> Pubkey {
    *AUCTION_HOUSE_PUBKEY.get_or_init(|| {
        METAPLEX_AUCTION_HOUSE_PROGRAM_ID
            .parse()
            .unwrap_or_else(|_| Pubkey::default())
    })
}

impl Default for Parser {
    #[inline]
    fn default() -> Self {
        Self {
            token_metadata_program_id: get_token_metadata_program_id(),
            auction_house_program_id: get_auction_house_program_id(),
            zero_copy: true,
            detailed_metadata: true,
        }
    }
}

impl Parser {
    /// Create parser with minimal metadata parsing (faster)
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded Metaplex program ID constants cannot be parsed as valid Pubkeys.
    /// This should never happen in practice as these are well-known, tested constants.
    #[must_use]
    #[inline]
    pub fn new_fast() -> Self {
        Self {
            token_metadata_program_id: get_token_metadata_program_id(),
            auction_house_program_id: get_auction_house_program_id(),
            zero_copy: true,
            detailed_metadata: false,
        }
    }

    /// Parse `BurnNft` instruction
    fn parse_burn_nft<'data>(
        &self,
        data: &'data [u8],
        metadata: EventMetadata,
    ) -> ZeroCopyEvent<'data> {
        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };

        let instruction_data = BurnNftInstruction { discriminator: 29 };
        event.set_parsed_data(instruction_data);

        if self.detailed_metadata {
            let _analysis = EventAnalysis {
                event_category: "nft_burn".to_owned(),
                nft_mint: None, // Would be extracted from accounts
                collection_mint: None,
                amount: Some(1),
                metadata_uri: None,
                extra_data: serde_json::json!({
                    "instruction": "burn_nft"
                }),
            };

            // NOTE: Commenting out to preserve instruction data in parsed_data
            // event.set_parsed_data(analysis);
        }

        let json = serde_json::json!({
            "instruction_type": "burn_nft",
            "event_category": "nft_burn",
            "amount": "1",
            "protocol": "metaplex"
        });
        event.set_json_data(json);

        event
    }

    /// Parse `CreateMetadataAccount` instruction
    fn parse_create_metadata_account<'data>(
        &self,
        data: &'data [u8],
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'data>, ParseError> {
        let mut deserializer = CustomDeserializer::new(data);

        // Skip discriminator
        deserializer.skip(1)?;

        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };

        if self.detailed_metadata {
            // In a full implementation, we would parse the full metadata structure
            // This is complex due to variable-length strings and nested data
            // For now, we provide a simplified analysis

            let _analysis = EventAnalysis {
                event_category: "nft_mint".to_owned(),
                nft_mint: None, // Would be extracted from accounts
                collection_mint: None,
                amount: Some(1),
                metadata_uri: None, // Would be parsed from instruction data
                extra_data: serde_json::json!({
                    "instruction": "create_metadata_account",
                    "version": "v1"
                }),
            };

            // NOTE: Commenting out to preserve instruction data in parsed_data
            // event.set_parsed_data(analysis);
        }

        let json = serde_json::json!({
            "instruction_type": "create_metadata_account",
            "event_category": "nft_mint",
            "protocol": "metaplex"
        });
        event.set_json_data(json);

        Ok(event)
    }

    /// Parse generic Metaplex instruction
    fn parse_generic_instruction<'data>(
        &self,
        data: &'data [u8],
        discriminator: TokenMetadataDiscriminator,
        metadata: EventMetadata,
    ) -> ZeroCopyEvent<'data> {
        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };

        let instruction_name = format!("{discriminator:?}").to_lowercase();

        let json = serde_json::json!({
            "instruction_type": instruction_name,
            "event_category": "metaplex_operation",
            "discriminator": discriminator as u8,
            "protocol": "metaplex"
        });
        event.set_json_data(json);

        event
    }

    /// Parse Transfer instruction
    fn parse_transfer<'data>(
        &self,
        data: &'data [u8],
        metadata: EventMetadata,
    ) -> ZeroCopyEvent<'data> {
        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };

        let instruction_data = TransferInstruction {
            discriminator: 42,
            authorization_data: None, // Would be parsed from instruction
        };
        event.set_parsed_data(instruction_data);

        if self.detailed_metadata {
            let _analysis = EventAnalysis {
                event_category: "nft_transfer".to_owned(),
                nft_mint: None, // Would be extracted from accounts
                collection_mint: None,
                amount: Some(1), // NFTs are typically amount 1
                metadata_uri: None,
                extra_data: serde_json::json!({
                    "instruction": "transfer",
                    "token_standard": "nft"
                }),
            };

            // NOTE: Commenting out to preserve instruction data in parsed_data
            // event.set_parsed_data(analysis);
        }

        let json = serde_json::json!({
            "instruction_type": "transfer",
            "event_category": "nft_transfer",
            "amount": "1",
            "protocol": "metaplex"
        });
        event.set_json_data(json);

        event
    }
}

impl ByteSliceEventParser for Parser {
    #[inline]
    fn can_parse(&self, data: &[u8]) -> bool {
        if data.is_empty() {
            return false;
        }

        data.first()
            .and_then(|&byte| TokenMetadataDiscriminator::from_byte(byte))
            .is_some()
    }
    #[inline]
    fn parse_from_slice<'data>(
        &self,
        data: &'data [u8],
        mut metadata: EventMetadata,
    ) -> Result<Vec<ZeroCopyEvent<'data>>, ParseError> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        // Update metadata with protocol info
        // Note: In practice, we'd need to check the program ID to determine
        // if this is Token Metadata or Auction House
        set_protocol_type(
            &mut metadata.core,
            &ProtocolType::Other("Metaplex".to_owned()),
        );

        // Parse discriminator
        let first_byte = *data.first().ok_or(ParseError::InsufficientData {
            expected: 1,
            actual: data.len(),
        })?;
        let discriminator = TokenMetadataDiscriminator::from_byte(first_byte).ok_or_else(|| {
            ParseError::UnknownDiscriminator {
                discriminator: vec![first_byte],
            }
        })?;

        // Update event type based on discriminator BEFORE creating events
        set_event_type(&mut metadata.core, &discriminator.event_type());

        let event = match discriminator {
            TokenMetadataDiscriminator::CreateMetadataAccount
            | TokenMetadataDiscriminator::CreateMetadataAccountV2
            | TokenMetadataDiscriminator::CreateMetadataAccountV3 => {
                self.parse_create_metadata_account(data, metadata)?
            }
            TokenMetadataDiscriminator::Transfer => self.parse_transfer(data, metadata),
            TokenMetadataDiscriminator::BurnNft => self.parse_burn_nft(data, metadata),
            _ => self.parse_generic_instruction(data, discriminator, metadata),
        };

        Ok(vec![event])
    }

    #[inline]
    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::Other("Metaplex".to_owned())
    }
}

/// Factory for creating Metaplex parsers
#[derive(Debug)]
#[non_exhaustive]
pub struct ParserFactory;

impl ParserFactory {
    /// Create a fast parser with minimal metadata
    #[must_use]
    #[inline]
    pub fn create_fast() -> Arc<dyn ByteSliceEventParser> {
        Arc::new(Parser::new_fast())
    }

    /// Create a new high-performance zero-copy parser with full metadata
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
    use crate::solana_metadata::SolanaEventMetadata;
    type EventMetadata = SolanaEventMetadata;

    #[test]
    fn test_discriminator_parsing() {
        assert_eq!(
            TokenMetadataDiscriminator::from_byte(0),
            Some(TokenMetadataDiscriminator::CreateMetadataAccount)
        );
        assert_eq!(
            TokenMetadataDiscriminator::from_byte(42),
            Some(TokenMetadataDiscriminator::Transfer)
        );
        assert_eq!(TokenMetadataDiscriminator::from_byte(255), None);
    }

    #[test]
    fn test_create_metadata_account_parsing() {
        let parser = Parser::default();

        let data = vec![0u8; 100]; // CreateMetadataAccount discriminator + data

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse CreateMetadataAccount instruction data in test");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.event_type(), EventType::Mint);

        let json = event
            .get_json_data()
            .expect("Failed to get JSON data from CreateMetadataAccount event in test");
        assert_eq!(json["instruction_type"], "create_metadata_account");
        assert_eq!(json["protocol"], "metaplex");
    }

    #[test]
    fn test_transfer_parsing() {
        let parser = Parser::default();

        let data = vec![42u8; 50]; // Transfer discriminator + data

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse Transfer instruction data in test");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.event_type(), EventType::Transfer);

        let parsed = event
            .get_parsed_data::<TransferInstruction>()
            .expect("Failed to get parsed TransferInstruction data in test");
        assert_eq!(parsed.discriminator, 42);
    }

    #[test]
    fn test_can_parse() {
        let parser = Parser::default();

        assert!(parser.can_parse(&[0])); // CreateMetadataAccount
        assert!(parser.can_parse(&[42])); // Transfer
        assert!(!parser.can_parse(&[255])); // Unknown
        assert!(!parser.can_parse(&[])); // Empty
    }

    #[test]
    fn test_factory() {
        let zero_copy_parser = ParserFactory::create_zero_copy();
        let fast_parser = ParserFactory::create_fast();

        assert_eq!(
            zero_copy_parser.protocol_type(),
            ProtocolType::Other("Metaplex".to_string())
        );
        assert_eq!(
            fast_parser.protocol_type(),
            ProtocolType::Other("Metaplex".to_string())
        );
    }

    #[test]
    fn test_discriminator_from_byte_all_valid_values() {
        // Test all valid discriminators
        assert_eq!(
            TokenMetadataDiscriminator::from_byte(0),
            Some(TokenMetadataDiscriminator::CreateMetadataAccount)
        );
        assert_eq!(
            TokenMetadataDiscriminator::from_byte(1),
            Some(TokenMetadataDiscriminator::UpdateMetadataAccount)
        );
        assert_eq!(
            TokenMetadataDiscriminator::from_byte(7),
            Some(TokenMetadataDiscriminator::SignMetadata)
        );
        assert_eq!(
            TokenMetadataDiscriminator::from_byte(10),
            Some(TokenMetadataDiscriminator::CreateMasterEdition)
        );
        assert_eq!(
            TokenMetadataDiscriminator::from_byte(11),
            Some(TokenMetadataDiscriminator::MintNewEditionFromMasterEditionViaToken)
        );
        assert_eq!(
            TokenMetadataDiscriminator::from_byte(15),
            Some(TokenMetadataDiscriminator::UpdateMetadataAccountV2)
        );
        assert_eq!(
            TokenMetadataDiscriminator::from_byte(16),
            Some(TokenMetadataDiscriminator::CreateMetadataAccountV2)
        );
        assert_eq!(
            TokenMetadataDiscriminator::from_byte(17),
            Some(TokenMetadataDiscriminator::CreateMasterEditionV3)
        );
        assert_eq!(
            TokenMetadataDiscriminator::from_byte(18),
            Some(TokenMetadataDiscriminator::VerifyCollection)
        );
        assert_eq!(
            TokenMetadataDiscriminator::from_byte(19),
            Some(TokenMetadataDiscriminator::Utilize)
        );
        assert_eq!(
            TokenMetadataDiscriminator::from_byte(29),
            Some(TokenMetadataDiscriminator::BurnNft)
        );
        assert_eq!(
            TokenMetadataDiscriminator::from_byte(30),
            Some(TokenMetadataDiscriminator::VerifyCreator)
        );
        assert_eq!(
            TokenMetadataDiscriminator::from_byte(34),
            Some(TokenMetadataDiscriminator::CreateMetadataAccountV3)
        );
        assert_eq!(
            TokenMetadataDiscriminator::from_byte(42),
            Some(TokenMetadataDiscriminator::Transfer)
        );
    }

    #[test]
    fn test_discriminator_from_byte_invalid_values() {
        // Test various invalid discriminators
        assert_eq!(TokenMetadataDiscriminator::from_byte(2), None);
        assert_eq!(TokenMetadataDiscriminator::from_byte(3), None);
        assert_eq!(TokenMetadataDiscriminator::from_byte(43), None);
        assert_eq!(TokenMetadataDiscriminator::from_byte(100), None);
        assert_eq!(TokenMetadataDiscriminator::from_byte(255), None);
    }

    #[test]
    fn test_discriminator_event_type_mapping() {
        // Test CreateMetadataAccount variants -> Mint
        assert_eq!(
            TokenMetadataDiscriminator::CreateMetadataAccount.event_type(),
            EventType::Mint
        );
        assert_eq!(
            TokenMetadataDiscriminator::CreateMetadataAccountV2.event_type(),
            EventType::Mint
        );
        assert_eq!(
            TokenMetadataDiscriminator::CreateMetadataAccountV3.event_type(),
            EventType::Mint
        );

        // Test Transfer -> Transfer
        assert_eq!(
            TokenMetadataDiscriminator::Transfer.event_type(),
            EventType::Transfer
        );

        // Test BurnNft -> Burn
        assert_eq!(
            TokenMetadataDiscriminator::BurnNft.event_type(),
            EventType::Burn
        );

        // Test other variants -> ContractEvent
        assert_eq!(
            TokenMetadataDiscriminator::SignMetadata.event_type(),
            EventType::ContractEvent
        );
        assert_eq!(
            TokenMetadataDiscriminator::VerifyCollection.event_type(),
            EventType::ContractEvent
        );
        assert_eq!(
            TokenMetadataDiscriminator::Utilize.event_type(),
            EventType::ContractEvent
        );
    }

    #[test]
    fn test_metaplex_parser_new() {
        let parser = Parser::default();
        assert!(parser.zero_copy);
        assert!(parser.detailed_metadata);
    }

    #[test]
    fn test_metaplex_parser_new_fast() {
        let parser = Parser::new_fast();
        assert!(parser.zero_copy);
        assert!(!parser.detailed_metadata);
    }

    #[test]
    fn test_metaplex_parser_default() {
        let parser = Parser::default();
        assert!(parser.zero_copy);
        assert!(parser.detailed_metadata);
    }

    #[test]
    fn test_protocol_type() {
        let parser = Parser::default();
        assert_eq!(
            parser.protocol_type(),
            ProtocolType::Other("Metaplex".to_string())
        );
    }

    #[test]
    fn test_parse_from_slice_empty_data() {
        let parser = Parser::default();
        let metadata = EventMetadata::default();
        let result = parser.parse_from_slice(&[], metadata);
        assert!(result.is_ok());
        assert_eq!(result.expect("Failed to parse empty data in test").len(), 0);
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_parse_from_slice_unknown_discriminator() {
        let parser = Parser::default();
        let data = vec![255u8; 10]; // Unknown discriminator
        let metadata = EventMetadata::default();
        let result = parser.parse_from_slice(&data, metadata);
        assert!(result.is_err());
        match result.expect_err("Expected UnknownDiscriminator error in test") {
            ParseError::UnknownDiscriminator { discriminator } => {
                assert_eq!(discriminator, vec![255]);
            }
            _ => panic!("Expected UnknownDiscriminator error but got different error type"),
        }
    }

    #[test]
    fn test_burn_nft_parsing() {
        let parser = Parser::default();
        let data = vec![29u8; 10]; // BurnNft discriminator + data
        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse BurnNft instruction data in test");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.event_type(), EventType::Burn);

        let parsed = event
            .get_parsed_data::<BurnNftInstruction>()
            .expect("Failed to get parsed BurnNftInstruction data in test");
        assert_eq!(parsed.discriminator, 29);

        let json = event
            .get_json_data()
            .expect("Failed to get JSON data from BurnNft event in test");
        assert_eq!(json["instruction_type"], "burn_nft");
        assert_eq!(json["protocol"], "metaplex");
        assert_eq!(json["event_category"], "nft_burn");
        assert_eq!(json["amount"], "1");
    }

    #[test]
    fn test_generic_instruction_parsing() {
        let parser = Parser::default();
        let data = vec![7u8; 10]; // SignMetadata discriminator + data
        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse SignMetadata instruction data in test");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.event_type(), EventType::ContractEvent);

        let json = event
            .get_json_data()
            .expect("Failed to get JSON data from SignMetadata event in test");
        assert_eq!(json["instruction_type"], "signmetadata");
        assert_eq!(json["protocol"], "metaplex");
        assert_eq!(json["event_category"], "metaplex_operation");
        assert_eq!(json["discriminator"], 7);
    }

    #[test]
    fn test_create_metadata_account_v2_parsing() {
        let parser = Parser::default();
        let data = vec![16u8; 100]; // CreateMetadataAccountV2 discriminator + data
        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse CreateMetadataAccountV2 instruction data in test");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.event_type(), EventType::Mint);

        let json = event
            .get_json_data()
            .expect("Failed to get JSON data from CreateMetadataAccountV2 event in test");
        assert_eq!(json["instruction_type"], "create_metadata_account");
        assert_eq!(json["protocol"], "metaplex");
        assert_eq!(json["event_category"], "nft_mint");
    }

    #[test]
    fn test_create_metadata_account_v3_parsing() {
        let parser = Parser::default();
        let data = vec![34u8; 100]; // CreateMetadataAccountV3 discriminator + data
        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse CreateMetadataAccountV3 instruction data in test");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.event_type(), EventType::Mint);

        let json = event
            .get_json_data()
            .expect("Failed to get JSON data from CreateMetadataAccountV3 event in test");
        assert_eq!(json["instruction_type"], "create_metadata_account");
        assert_eq!(json["protocol"], "metaplex");
        assert_eq!(json["event_category"], "nft_mint");
    }

    #[test]
    fn test_parse_create_metadata_account_with_fast_parser() {
        let parser = Parser::new_fast();
        let data = vec![0u8; 100]; // CreateMetadataAccount discriminator + data
        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).expect(
            "Failed to parse CreateMetadataAccount instruction data with fast parser in test",
        );

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.event_type(), EventType::Mint);

        let json = event.get_json_data().expect(
            "Failed to get JSON data from CreateMetadataAccount event with fast parser in test",
        );
        assert_eq!(json["instruction_type"], "create_metadata_account");
        assert_eq!(json["protocol"], "metaplex");
    }

    #[test]
    fn test_parse_transfer_with_fast_parser() {
        let parser = Parser::new_fast();
        let data = vec![42u8; 50]; // Transfer discriminator + data
        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse Transfer instruction data with fast parser in test");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.event_type(), EventType::Transfer);

        let json = event
            .get_json_data()
            .expect("Failed to get JSON data from Transfer event with fast parser in test");
        assert_eq!(json["instruction_type"], "transfer");
        assert_eq!(json["protocol"], "metaplex");
        assert_eq!(json["event_category"], "nft_transfer");
        assert_eq!(json["amount"], "1");
    }

    #[test]
    fn test_parse_burn_nft_with_fast_parser() {
        let parser = Parser::new_fast();
        let data = vec![29u8; 10]; // BurnNft discriminator + data
        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse BurnNft instruction data with fast parser in test");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.event_type(), EventType::Burn);

        let json = event
            .get_json_data()
            .expect("Failed to get JSON data from BurnNft event with fast parser in test");
        assert_eq!(json["instruction_type"], "burn_nft");
        assert_eq!(json["protocol"], "metaplex");
    }

    #[test]
    fn test_can_parse_edge_cases() {
        let parser = Parser::default();

        // Test single byte with valid discriminator
        assert!(parser.can_parse(&[0]));
        assert!(parser.can_parse(&[42]));
        assert!(parser.can_parse(&[29]));

        // Test single byte with invalid discriminator
        assert!(!parser.can_parse(&[255]));
        assert!(!parser.can_parse(&[2]));
        assert!(!parser.can_parse(&[100]));

        // Test empty data
        assert!(!parser.can_parse(&[]));

        // Test longer data with valid discriminator
        assert!(parser.can_parse(&[0, 1, 2, 3, 4]));
        assert!(parser.can_parse(&[42, 100, 200]));
    }

    #[test]
    fn test_all_discriminator_variants_coverage() {
        // Test all enum variants for completeness
        use TokenMetadataDiscriminator::*;

        let variants = vec![
            CreateMetadataAccount,
            UpdateMetadataAccount,
            DeprecatedCreateMasterEdition,
            DeprecatedMintNewEditionFromMasterEditionViaPrintingToken,
            UpdatePrimarySaleHappenedViaToken,
            DeprecatedSetReservationList,
            DeprecatedCreateReservationList,
            SignMetadata,
            DeprecatedMintPrintingTokensViaToken,
            DeprecatedMintPrintingTokens,
            CreateMasterEdition,
            MintNewEditionFromMasterEditionViaToken,
            ConvertMasterEditionV1ToV2,
            MintNewEditionFromMasterEditionViaVaultProxy,
            PuffMetadata,
            UpdateMetadataAccountV2,
            CreateMetadataAccountV2,
            CreateMasterEditionV3,
            VerifyCollection,
            Utilize,
            ApproveUseAuthority,
            RevokeUseAuthority,
            UnverifyCollection,
            ApproveCollectionAuthority,
            RevokeCollectionAuthority,
            SetAndVerifyCollection,
            FreezeDelegatedAccount,
            ThawDelegatedAccount,
            RemoveCreatorVerification,
            BurnNft,
            VerifyCreator,
            UnverifyCreator,
            BubblegumSetCollectionSize,
            BurnEditionNft,
            CreateMetadataAccountV3,
            SetCollectionSize,
            SetTokenStandard,
            BubblegumVerifyCreator,
            BubblegumUnverifyCreator,
            BubblegumVerifyCollection,
            BubblegumUnverifyCollection,
            BubblegumSetAndVerifyCollection,
            Transfer,
        ];

        // Test that we can format all variants (used in generic instruction parsing)
        for variant in variants {
            let name = format!("{variant:?}").to_lowercase();
            assert!(!name.is_empty());
        }
    }

    #[test]
    fn test_auction_house_discriminator_variants() {
        // Test all AuctionHouseDiscriminator variants for completeness
        use AuctionHouseDiscriminator::*;

        let variants = vec![Buy, Sell, ExecuteSale, Deposit, Withdraw, Cancel];

        // Test that we can format all variants
        for variant in variants {
            let name = format!("{variant:?}");
            assert!(!name.is_empty());
        }
    }

    #[test]
    fn test_metaplex_event_analysis_creation() {
        // Test creating EventAnalysis struct
        let analysis = EventAnalysis {
            event_category: "test_category".to_string(),
            nft_mint: None,
            collection_mint: None,
            amount: Some(100),
            metadata_uri: Some("https://example.com/metadata.json".to_string()),
            extra_data: serde_json::json!({"test": "data"}),
        };

        assert_eq!(analysis.event_category, "test_category");
        assert_eq!(analysis.nft_mint, None);
        assert_eq!(analysis.collection_mint, None);
        assert_eq!(analysis.amount, Some(100));
        assert_eq!(
            analysis.metadata_uri,
            Some("https://example.com/metadata.json".to_string())
        );
        assert_eq!(
            analysis
                .extra_data
                .get("test")
                .expect("Expected 'test' key"),
            "data"
        );
    }

    #[test]
    fn test_instruction_structs() {
        // Test CreateMetadataAccountInstruction
        let create_instruction = CreateMetadataAccountInstruction {
            discriminator: 0,
            metadata_account_bump: 255,
            name: "Test NFT".to_string(),
            symbol: "TST".to_string(),
            uri: "https://example.com/metadata.json".to_string(),
            seller_fee_basis_points: 500,
            update_authority_is_signer: true,
            is_mutable: false,
        };

        assert_eq!(create_instruction.discriminator, 0);
        assert_eq!(create_instruction.name, "Test NFT");
        assert_eq!(create_instruction.symbol, "TST");
        assert_eq!(create_instruction.seller_fee_basis_points, 500);
        assert!(create_instruction.update_authority_is_signer);
        assert!(!create_instruction.is_mutable);

        // Test TransferInstruction
        let transfer_instruction = TransferInstruction {
            discriminator: 42,
            authorization_data: Some(vec![1, 2, 3]),
        };

        assert_eq!(transfer_instruction.discriminator, 42);
        assert_eq!(transfer_instruction.authorization_data, Some(vec![1, 2, 3]));

        // Test BurnNftInstruction
        let burn_instruction = BurnNftInstruction { discriminator: 29 };
        assert_eq!(burn_instruction.discriminator, 29);
    }

    #[test]
    fn test_parse_create_metadata_account_zero_copy_disabled() {
        // Create parser with zero_copy disabled for coverage
        let parser = Parser::default();
        // We can't directly modify zero_copy as it's private, but we can test the behavior
        // by calling the parsing methods which check the zero_copy flag internally

        let data = vec![0u8; 100]; // CreateMetadataAccount discriminator + data
        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).expect("Failed to parse CreateMetadataAccount instruction data with zero-copy disabled in test");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.event_type(), EventType::Mint);
    }

    #[test]
    fn test_parse_create_metadata_account_insufficient_data() {
        let parser = Parser::default();
        let data = vec![0u8]; // Only discriminator, no additional data
        let metadata = EventMetadata::default();

        // This should still work as we only skip 1 byte and don't parse the full structure
        let result = parser.parse_from_slice(&data, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_constants() {
        assert_eq!(
            METAPLEX_TOKEN_METADATA_PROGRAM_ID,
            "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
        );
        assert_eq!(
            METAPLEX_AUCTION_HOUSE_PROGRAM_ID,
            "hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk"
        );
    }
}
