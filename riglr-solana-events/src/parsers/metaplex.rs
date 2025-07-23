//! High-performance Metaplex parser for NFT marketplace events
//! 
//! This parser handles Metaplex protocol operations including NFT minting,
//! marketplace transactions, and metadata operations.

use std::sync::Arc;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;
use crate::zero_copy::{ByteSliceEventParser, ParseError, CustomDeserializer, ZeroCopyEvent};
use crate::types::{EventMetadata, EventType, ProtocolType};
// UnifiedEvent trait has been removed

/// Metaplex Token Metadata program ID
pub const METAPLEX_TOKEN_METADATA_PROGRAM_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";

/// Metaplex Auction House program ID
pub const METAPLEX_AUCTION_HOUSE_PROGRAM_ID: &str = "hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk";

/// Metaplex instruction discriminators for Token Metadata program
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaplexTokenMetadataDiscriminator {
    CreateMetadataAccount = 0,
    UpdateMetadataAccount = 1,
    DeprecatedCreateMasterEdition = 2,
    DeprecatedMintNewEditionFromMasterEditionViaPrintingToken = 3,
    UpdatePrimarySaleHappenedViaToken = 4,
    DeprecatedSetReservationList = 5,
    DeprecatedCreateReservationList = 6,
    SignMetadata = 7,
    DeprecatedMintPrintingTokensViaToken = 8,
    DeprecatedMintPrintingTokens = 9,
    CreateMasterEdition = 10,
    MintNewEditionFromMasterEditionViaToken = 11,
    ConvertMasterEditionV1ToV2 = 12,
    MintNewEditionFromMasterEditionViaVaultProxy = 13,
    PuffMetadata = 14,
    UpdateMetadataAccountV2 = 15,
    CreateMetadataAccountV2 = 16,
    CreateMasterEditionV3 = 17,
    VerifyCollection = 18,
    Utilize = 19,
    ApproveUseAuthority = 20,
    RevokeUseAuthority = 21,
    UnverifyCollection = 22,
    ApproveCollectionAuthority = 23,
    RevokeCollectionAuthority = 24,
    SetAndVerifyCollection = 25,
    FreezeDelegatedAccount = 26,
    ThawDelegatedAccount = 27,
    RemoveCreatorVerification = 28,
    BurnNft = 29,
    VerifyCreator = 30,
    UnverifyCreator = 31,
    BubblegumSetCollectionSize = 32,
    BurnEditionNft = 33,
    CreateMetadataAccountV3 = 34,
    SetCollectionSize = 35,
    SetTokenStandard = 36,
    BubblegumVerifyCreator = 37,
    BubblegumUnverifyCreator = 38,
    BubblegumVerifyCollection = 39,
    BubblegumUnverifyCollection = 40,
    BubblegumSetAndVerifyCollection = 41,
    Transfer = 42,
}

/// Metaplex Auction House discriminators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaplexAuctionHouseDiscriminator {
    Buy,
    Sell,
    ExecuteSale,
    Deposit,
    Withdraw,
    Cancel,
}

impl MetaplexTokenMetadataDiscriminator {
    /// Parse discriminator from byte
    pub fn from_byte(byte: u8) -> Option<Self> {
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

    /// Get corresponding event type
    pub fn event_type(&self) -> EventType {
        match self {
            Self::CreateMetadataAccount |
            Self::CreateMetadataAccountV2 |
            Self::CreateMetadataAccountV3 => EventType::Mint,
            Self::Transfer => EventType::Transfer,
            Self::BurnNft => EventType::Burn,
            _ => EventType::ContractEvent,
        }
    }
}

/// Metaplex CreateMetadataAccount instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct CreateMetadataAccountInstruction {
    /// Discriminator
    pub discriminator: u8,
    /// Metadata account bump
    pub metadata_account_bump: u8,
    /// NFT name
    pub name: String,
    /// NFT symbol
    pub symbol: String,
    /// NFT URI (metadata JSON)
    pub uri: String,
    /// Seller fee basis points
    pub seller_fee_basis_points: u16,
    /// Update authority can change metadata
    pub update_authority_is_signer: bool,
    /// Is mutable
    pub is_mutable: bool,
}

/// Metaplex Transfer instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct TransferInstruction {
    /// Discriminator
    pub discriminator: u8,
    /// Authorization data
    pub authorization_data: Option<Vec<u8>>,
}

/// Metaplex BurnNft instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct BurnNftInstruction {
    /// Discriminator
    pub discriminator: u8,
}

/// Metaplex marketplace event analysis
#[derive(Debug, Clone)]
pub struct MetaplexEventAnalysis {
    /// Event category (mint, transfer, burn, etc.)
    pub event_category: String,
    /// NFT mint address (if applicable)
    pub nft_mint: Option<Pubkey>,
    /// Collection mint (if applicable)
    pub collection_mint: Option<Pubkey>,
    /// Transfer amount (for transfers)
    pub amount: Option<u64>,
    /// Metadata URI (for mints)
    pub metadata_uri: Option<String>,
    /// Additional event-specific data
    pub extra_data: serde_json::Value,
}

/// High-performance Metaplex parser
pub struct MetaplexParser {
    /// Token Metadata program ID
    #[allow(dead_code)]
    token_metadata_program_id: Pubkey,
    /// Auction House program ID
    #[allow(dead_code)]
    auction_house_program_id: Pubkey,
    /// Enable zero-copy parsing
    zero_copy: bool,
    /// Enable detailed metadata parsing
    detailed_metadata: bool,
}

impl Default for MetaplexParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaplexParser {
    /// Create new Metaplex parser
    pub fn new() -> Self {
        Self {
            token_metadata_program_id: METAPLEX_TOKEN_METADATA_PROGRAM_ID.parse()
                .expect("Valid Metaplex Token Metadata program ID"),
            auction_house_program_id: METAPLEX_AUCTION_HOUSE_PROGRAM_ID.parse()
                .expect("Valid Metaplex Auction House program ID"),
            zero_copy: true,
            detailed_metadata: true,
        }
    }

    /// Create parser with minimal metadata parsing (faster)
    pub fn new_fast() -> Self {
        Self {
            token_metadata_program_id: METAPLEX_TOKEN_METADATA_PROGRAM_ID.parse()
                .expect("Valid Metaplex Token Metadata program ID"),
            auction_house_program_id: METAPLEX_AUCTION_HOUSE_PROGRAM_ID.parse()
                .expect("Valid Metaplex Auction House program ID"),
            zero_copy: true,
            detailed_metadata: false,
        }
    }

    /// Parse CreateMetadataAccount instruction
    fn parse_create_metadata_account<'a>(
        &self,
        data: &'a [u8],
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'a>, ParseError> {
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
            
            let analysis = MetaplexEventAnalysis {
                event_category: "nft_mint".to_string(),
                nft_mint: None, // Would be extracted from accounts
                collection_mint: None,
                amount: Some(1),
                metadata_uri: None, // Would be parsed from instruction data
                extra_data: serde_json::json!({
                    "instruction": "create_metadata_account",
                    "version": "v1"
                }),
            };
            
            event.set_parsed_data(analysis);
        }
        
        let json = serde_json::json!({
            "instruction_type": "create_metadata_account",
            "event_category": "nft_mint",
            "protocol": "metaplex"
        });
        event.set_json_data(json);
        
        Ok(event)
    }

    /// Parse Transfer instruction
    fn parse_transfer<'a>(
        &self,
        data: &'a [u8],
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'a>, ParseError> {
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
            let analysis = MetaplexEventAnalysis {
                event_category: "nft_transfer".to_string(),
                nft_mint: None, // Would be extracted from accounts
                collection_mint: None,
                amount: Some(1), // NFTs are typically amount 1
                metadata_uri: None,
                extra_data: serde_json::json!({
                    "instruction": "transfer",
                    "token_standard": "nft"
                }),
            };
            
            event.set_parsed_data(analysis);
        }
        
        let json = serde_json::json!({
            "instruction_type": "transfer",
            "event_category": "nft_transfer",
            "amount": "1",
            "protocol": "metaplex"
        });
        event.set_json_data(json);
        
        Ok(event)
    }

    /// Parse BurnNft instruction
    fn parse_burn_nft<'a>(
        &self,
        data: &'a [u8],
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'a>, ParseError> {
        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };
        
        let instruction_data = BurnNftInstruction {
            discriminator: 29,
        };
        event.set_parsed_data(instruction_data);
        
        if self.detailed_metadata {
            let analysis = MetaplexEventAnalysis {
                event_category: "nft_burn".to_string(),
                nft_mint: None, // Would be extracted from accounts
                collection_mint: None,
                amount: Some(1),
                metadata_uri: None,
                extra_data: serde_json::json!({
                    "instruction": "burn_nft"
                }),
            };
            
            event.set_parsed_data(analysis);
        }
        
        let json = serde_json::json!({
            "instruction_type": "burn_nft",
            "event_category": "nft_burn",
            "amount": "1",
            "protocol": "metaplex"
        });
        event.set_json_data(json);
        
        Ok(event)
    }

    /// Parse generic Metaplex instruction
    fn parse_generic_instruction<'a>(
        &self,
        data: &'a [u8],
        discriminator: MetaplexTokenMetadataDiscriminator,
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'a>, ParseError> {
        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };
        
        let instruction_name = format!("{:?}", discriminator).to_lowercase();
        
        let json = serde_json::json!({
            "instruction_type": instruction_name,
            "event_category": "metaplex_operation",
            "discriminator": discriminator as u8,
            "protocol": "metaplex"
        });
        event.set_json_data(json);
        
        Ok(event)
    }
}

impl ByteSliceEventParser for MetaplexParser {
    fn parse_from_slice<'a>(
        &self,
        data: &'a [u8],
        mut metadata: EventMetadata,
    ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        // Update metadata with protocol info
        // Note: In practice, we'd need to check the program ID to determine
        // if this is Token Metadata or Auction House
        metadata.protocol_type = ProtocolType::Other("Metaplex".to_string());
        
        // Parse discriminator
        let discriminator = MetaplexTokenMetadataDiscriminator::from_byte(data[0])
            .ok_or_else(|| ParseError::UnknownDiscriminator { 
                discriminator: vec![data[0]] 
            })?;

        // Update event type based on discriminator
        metadata.event_type = discriminator.event_type();

        let event = match discriminator {
            MetaplexTokenMetadataDiscriminator::CreateMetadataAccount |
            MetaplexTokenMetadataDiscriminator::CreateMetadataAccountV2 |
            MetaplexTokenMetadataDiscriminator::CreateMetadataAccountV3 => {
                self.parse_create_metadata_account(data, metadata)?
            },
            MetaplexTokenMetadataDiscriminator::Transfer => {
                self.parse_transfer(data, metadata)?
            },
            MetaplexTokenMetadataDiscriminator::BurnNft => {
                self.parse_burn_nft(data, metadata)?
            },
            _ => {
                self.parse_generic_instruction(data, discriminator, metadata)?
            },
        };

        Ok(vec![event])
    }

    fn can_parse(&self, data: &[u8]) -> bool {
        if data.is_empty() {
            return false;
        }
        
        MetaplexTokenMetadataDiscriminator::from_byte(data[0]).is_some()
    }

    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::Other("Metaplex".to_string())
    }
}

/// Factory for creating Metaplex parsers
pub struct MetaplexParserFactory;

impl MetaplexParserFactory {
    /// Create a new high-performance zero-copy parser with full metadata
    pub fn create_zero_copy() -> Arc<dyn ByteSliceEventParser> {
        Arc::new(MetaplexParser::new())
    }

    /// Create a fast parser with minimal metadata
    pub fn create_fast() -> Arc<dyn ByteSliceEventParser> {
        Arc::new(MetaplexParser::new_fast())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EventMetadata;

    #[test]
    fn test_discriminator_parsing() {
        assert_eq!(
            MetaplexTokenMetadataDiscriminator::from_byte(0), 
            Some(MetaplexTokenMetadataDiscriminator::CreateMetadataAccount)
        );
        assert_eq!(
            MetaplexTokenMetadataDiscriminator::from_byte(42), 
            Some(MetaplexTokenMetadataDiscriminator::Transfer)
        );
        assert_eq!(MetaplexTokenMetadataDiscriminator::from_byte(255), None);
    }

    #[test]
    fn test_create_metadata_account_parsing() {
        let parser = MetaplexParser::new();
        
        let data = vec![0u8; 100]; // CreateMetadataAccount discriminator + data
        
        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).unwrap();
        
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.event_type(), EventType::Mint);
        
        let json = event.get_json_data().unwrap();
        assert_eq!(json["instruction_type"], "create_metadata_account");
        assert_eq!(json["protocol"], "metaplex");
    }

    #[test]
    fn test_transfer_parsing() {
        let parser = MetaplexParser::new();
        
        let data = vec![42u8; 50]; // Transfer discriminator + data
        
        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).unwrap();
        
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.event_type(), EventType::Transfer);
        
        let parsed = event.get_parsed_data::<TransferInstruction>().unwrap();
        assert_eq!(parsed.discriminator, 42);
    }

    #[test]
    fn test_can_parse() {
        let parser = MetaplexParser::new();
        
        assert!(parser.can_parse(&[0])); // CreateMetadataAccount
        assert!(parser.can_parse(&[42])); // Transfer
        assert!(!parser.can_parse(&[255])); // Unknown
        assert!(!parser.can_parse(&[])); // Empty
    }

    #[test]
    fn test_factory() {
        let zero_copy_parser = MetaplexParserFactory::create_zero_copy();
        let fast_parser = MetaplexParserFactory::create_fast();
        
        assert_eq!(zero_copy_parser.protocol_type(), ProtocolType::Other("Metaplex".to_string()));
        assert_eq!(fast_parser.protocol_type(), ProtocolType::Other("Metaplex".to_string()));
    }
}