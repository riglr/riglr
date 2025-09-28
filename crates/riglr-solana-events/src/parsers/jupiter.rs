//! High-performance Jupiter aggregator parser for complex swap routing analysis
//!
//! This parser handles Jupiter protocol operations with optimized parsing for
//! multi-hop swaps and complex routing instructions.

extern crate alloc;

use crate::metadata_helpers::set_protocol_type;
use crate::solana_metadata::SolanaEventMetadata;
use crate::types::{EventType, ProtocolType};
use crate::zero_copy::{ByteSliceEventParser, CustomDeserializer, ParseError, ZeroCopyEvent};
use alloc::sync::Arc;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;

/// Jupiter aggregator program ID
pub const JUPITER_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";

/// Type alias for event metadata used in Jupiter parser
type EventMetadata = SolanaEventMetadata;
// UnifiedEvent trait has been removed

/// Jupiter instruction discriminators (using Anchor discriminators)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Discriminator {
    /// `ExactOutRoute` discriminator: [123, 87, 17, 219, 42, 126, 197, 78]
    ExactOutRoute,
    /// Route discriminator: [229, 23, 203, 151, 122, 227, 173, 42]
    Route,
    /// `RouteWithTokenLedger` discriminator: [206, 198, 71, 54, 47, 82, 194, 13]
    RouteWithTokenLedger,
    /// `SharedAccountsExactOutRoute` discriminator: [77, 119, 2, 23, 198, 126, 79, 175]
    SharedAccountsExactOutRoute,
    /// `SharedAccountsRoute` discriminator: [95, 180, 10, 172, 84, 174, 232, 239]
    SharedAccountsRoute,
    /// `SharedAccountsRouteWithTokenLedger` discriminator: [18, 99, 149, 21, 45, 126, 144, 122]
    SharedAccountsRouteWithTokenLedger,
}

impl Discriminator {
    /// Get discriminator bytes
    #[must_use]
    #[inline]
    pub const fn bytes(self) -> &'static [u8; 8] {
        match self {
            Self::Route => &[229, 23, 203, 151, 122, 227, 173, 42],
            Self::RouteWithTokenLedger => &[206, 198, 71, 54, 47, 82, 194, 13],
            Self::ExactOutRoute => &[123, 87, 17, 219, 42, 126, 197, 78],
            Self::SharedAccountsRoute => &[95, 180, 10, 172, 84, 174, 232, 239],
            Self::SharedAccountsRouteWithTokenLedger => &[18, 99, 149, 21, 45, 126, 144, 122],
            Self::SharedAccountsExactOutRoute => &[77, 119, 2, 23, 198, 126, 79, 175],
        }
    }

    /// Get corresponding event type
    #[must_use]
    #[inline]
    pub const fn event_type(self) -> EventType {
        EventType::Swap // Jupiter operations are always swaps
    }

    /// Parse discriminator from first 8 bytes
    #[must_use]
    #[inline]
    pub const fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 8 {
            return None;
        }

        match *bytes {
            [229, 23, 203, 151, 122, 227, 173, 42] => Some(Self::Route),
            [206, 198, 71, 54, 47, 82, 194, 13] => Some(Self::RouteWithTokenLedger),
            [123, 87, 17, 219, 42, 126, 197, 78] => Some(Self::ExactOutRoute),
            [95, 180, 10, 172, 84, 174, 232, 239] => Some(Self::SharedAccountsRoute),
            [18, 99, 149, 21, 45, 126, 144, 122] => Some(Self::SharedAccountsRouteWithTokenLedger),
            [77, 119, 2, 23, 198, 126, 79, 175] => Some(Self::SharedAccountsExactOutRoute),
            _ => None,
        }
    }
}

/// Jupiter Route instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[non_exhaustive]
pub struct RouteInstruction {
    /// Amount to swap in
    pub amount_in: u64,
    /// Minimum amount out expected
    pub minimum_amount_out: u64,
    /// Platform fee basis points
    pub platform_fee_bps: u16,
}

/// Jupiter `ExactOutRoute` instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[non_exhaustive]
pub struct ExactOutRouteInstruction {
    /// Exact amount out desired
    pub amount_out: u64,
    /// Maximum amount in
    pub max_amount_in: u64,
    /// Platform fee basis points
    pub platform_fee_bps: u16,
}

/// Simplified route hop representation
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RouteHop {
    /// Amount in for this hop
    pub amount_in: Option<u64>,
    /// Amount out for this hop
    pub amount_out: Option<u64>,
    /// Input mint
    pub input_mint: Option<Pubkey>,
    /// Output mint
    pub output_mint: Option<Pubkey>,
    /// DEX program ID
    pub program_id: Pubkey,
}

/// Combined Jupiter Route Data (instruction + analysis)
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RouteData {
    /// The route analysis result
    pub analysis: RouteAnalysis,
    /// The parsed instruction data
    pub instruction: RouteInstruction,
}

/// Jupiter route analysis result
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RouteAnalysis {
    /// Number of hops in the route
    pub hop_count: usize,
    /// Route hops (simplified analysis)
    pub hops: Vec<RouteHop>,
    /// Instruction type
    pub instruction_type: String,
    /// Platform fee in basis points
    pub platform_fee_bps: u16,
    /// Total amount in
    pub total_amount_in: u64,
    /// Total amount out (minimum or exact)
    pub total_amount_out: u64,
}

/// High-performance Jupiter parser
#[derive(Debug)]
pub struct Parser {
    /// Enable detailed route analysis with hop information
    detailed_analysis: bool,
    /// Enable zero-copy parsing for better performance
    zero_copy: bool,
}

impl Default for Parser {
    #[inline]
    fn default() -> Self {
        Self {
            zero_copy: true,
            detailed_analysis: true,
        }
    }
}

impl Parser {
    /// Analyze route structure (simplified implementation)
    fn analyze_route(
        _data: &[u8],
        discriminator: Discriminator,
        amount_in: u64,
        amount_out: u64,
        platform_fee_bps: u16,
    ) -> RouteAnalysis {
        // In a full implementation, this would parse the route structure
        // including all the accounts and intermediate hops.
        // For now, we provide a simplified analysis.

        let instruction_type = match discriminator {
            Discriminator::Route => "route".to_owned(),
            Discriminator::RouteWithTokenLedger => "route_with_token_ledger".to_owned(),
            Discriminator::ExactOutRoute => "exact_out_route".to_owned(),
            Discriminator::SharedAccountsRoute => "shared_accounts_route".to_owned(),
            Discriminator::SharedAccountsRouteWithTokenLedger => {
                "shared_accounts_route_with_token_ledger".to_owned()
            }
            Discriminator::SharedAccountsExactOutRoute => {
                "shared_accounts_exact_out_route".to_owned()
            }
        };

        // Estimate hop count based on instruction type
        let hop_count = match discriminator {
            Discriminator::SharedAccountsRoute
            | Discriminator::SharedAccountsRouteWithTokenLedger
            | Discriminator::SharedAccountsExactOutRoute => 3, // Multi-hop routes
            Discriminator::Route
            | Discriminator::RouteWithTokenLedger
            | Discriminator::ExactOutRoute => 1, // Single hop routes
        };

        RouteAnalysis {
            instruction_type,
            total_amount_in: amount_in,
            total_amount_out: amount_out,
            platform_fee_bps,
            hop_count,
            hops: vec![], // Would be populated in full implementation
        }
    }

    /// Create new Jupiter parser with full analysis
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create parser with simplified analysis (faster)
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded Jupiter program ID cannot be parsed
    #[must_use]
    #[inline]
    pub const fn new_fast() -> Self {
        Self {
            zero_copy: true,
            detailed_analysis: false,
        }
    }

    /// Create parser with zero-copy disabled
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded Jupiter program ID cannot be parsed
    #[must_use]
    #[inline]
    pub const fn new_standard() -> Self {
        Self {
            zero_copy: false,
            detailed_analysis: true,
        }
    }

    /// Parse `ExactOutRoute` instruction
    fn parse_exact_out_route<'a>(
        &self,
        data: &'a [u8],
        discriminator: Discriminator,
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'a>, ParseError> {
        let mut deserializer = CustomDeserializer::new(data);

        // Skip discriminator (8 bytes)
        deserializer.skip(8)?;

        let max_amount_in = deserializer.read_u64_le()?;
        let amount_out = deserializer.read_u64_le()?;

        let platform_fee_bps = if deserializer.remaining() >= 2 {
            (deserializer.read_u32_le()? & 0xFFFF) as u16
        } else {
            0
        };

        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };

        let instruction_data = ExactOutRouteInstruction {
            amount_out,
            max_amount_in,
            platform_fee_bps,
        };
        event.set_parsed_data(instruction_data);

        let analysis = if self.detailed_analysis {
            Self::analyze_route(
                data,
                discriminator,
                max_amount_in,
                amount_out,
                platform_fee_bps,
            )
        } else {
            RouteAnalysis {
                instruction_type: format!("{discriminator:?}"),
                total_amount_in: max_amount_in,
                total_amount_out: amount_out,
                platform_fee_bps,
                hop_count: 1,
                hops: vec![],
            }
        };

        let json = serde_json::json!({
            "instruction_type": analysis.instruction_type,
            "max_amount_in": analysis.total_amount_in.to_string(),
            "amount_out": analysis.total_amount_out.to_string(),
            "platform_fee_bps": analysis.platform_fee_bps,
            "hop_count": analysis.hop_count,
            "protocol": "jupiter"
        });
        event.set_json_data(json);

        Ok(event)
    }

    /// Parse `Route` instruction
    fn parse_route<'a>(
        &self,
        data: &'a [u8],
        discriminator: Discriminator,
        metadata: EventMetadata,
    ) -> Result<ZeroCopyEvent<'a>, ParseError> {
        let mut deserializer = CustomDeserializer::new(data);

        // Skip discriminator (8 bytes)
        deserializer.skip(8)?;

        // Parse basic route parameters
        let amount_in = deserializer.read_u64_le()?;
        let minimum_amount_out = deserializer.read_u64_le()?;

        // Try to read platform fee (may not be present in all versions)
        let platform_fee_bps = if deserializer.remaining() >= 2 {
            (deserializer.read_u32_le()? & 0xFFFF) as u16
        } else {
            0
        };

        let mut event = if self.zero_copy {
            ZeroCopyEvent::new_borrowed(metadata, data)
        } else {
            ZeroCopyEvent::new_owned(metadata, data.to_vec())
        };

        let instruction_data = RouteInstruction {
            amount_in,
            minimum_amount_out,
            platform_fee_bps,
        };

        // Perform route analysis if enabled
        let analysis = if self.detailed_analysis {
            Self::analyze_route(
                data,
                discriminator,
                amount_in,
                minimum_amount_out,
                platform_fee_bps,
            )
        } else {
            RouteAnalysis {
                instruction_type: format!("{discriminator:?}"),
                total_amount_in: amount_in,
                total_amount_out: minimum_amount_out,
                platform_fee_bps,
                hop_count: 1, // Default assumption
                hops: vec![],
            }
        };

        // Store combined data
        let combined_data = RouteData {
            instruction: instruction_data,
            analysis: analysis.clone(),
        };
        event.set_parsed_data(combined_data);

        let json = serde_json::json!({
            "instruction_type": analysis.instruction_type,
            "total_amount_in": analysis.total_amount_in.to_string(),
            "total_amount_out": analysis.total_amount_out.to_string(),
            "platform_fee_bps": analysis.platform_fee_bps,
            "hop_count": analysis.hop_count,
            "protocol": "jupiter"
        });
        event.set_json_data(json);

        Ok(event)
    }
}

impl ByteSliceEventParser for Parser {
    #[inline]
    fn can_parse(&self, data: &[u8]) -> bool {
        if data.len() < 8 {
            return false;
        }

        data.get(0..8).and_then(Discriminator::from_bytes).is_some()
    }

    #[inline]
    fn parse_from_slice<'data>(
        &self,
        data: &'data [u8],
        mut metadata: EventMetadata,
    ) -> Result<Vec<ZeroCopyEvent<'data>>, ParseError> {
        if data.len() < 8 {
            return Err(ParseError::InsufficientData {
                expected: 8,
                actual: data.len(),
            });
        }

        // Update metadata with protocol info
        metadata.protocol_type = ProtocolType::Jupiter;
        set_protocol_type(&mut metadata.core, &ProtocolType::Jupiter);

        // Parse discriminator from first 8 bytes
        let discriminator_bytes = data.get(0..8).ok_or(ParseError::InsufficientData {
            expected: 8,
            actual: data.len(),
        })?;
        let discriminator = Discriminator::from_bytes(discriminator_bytes).ok_or_else(|| {
            ParseError::UnknownDiscriminator {
                discriminator: discriminator_bytes.to_vec(),
            }
        })?;

        // Update event type
        metadata.event_type = discriminator.event_type();

        let event = match discriminator {
            Discriminator::Route
            | Discriminator::RouteWithTokenLedger
            | Discriminator::SharedAccountsRoute
            | Discriminator::SharedAccountsRouteWithTokenLedger => {
                self.parse_route(data, discriminator, metadata)?
            }
            Discriminator::ExactOutRoute | Discriminator::SharedAccountsExactOutRoute => {
                self.parse_exact_out_route(data, discriminator, metadata)?
            }
        };

        Ok(vec![event])
    }

    #[inline]
    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::Jupiter
    }
}

/// Factory for creating Jupiter parsers
#[derive(Debug)]
#[non_exhaustive]
pub struct ParserFactory;

impl ParserFactory {
    /// Create a fast parser with simplified analysis
    #[must_use]
    #[inline]
    pub fn create_fast() -> Arc<dyn ByteSliceEventParser> {
        Arc::new(Parser::new_fast())
    }

    /// Create a standard parser
    #[must_use]
    #[inline]
    pub fn create_standard() -> Arc<dyn ByteSliceEventParser> {
        Arc::new(Parser::new_standard())
    }

    /// Create a new high-performance zero-copy parser with full analysis
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
    use crate::metadata_helpers::create_core_metadata;
    use crate::solana_metadata::SolanaEventMetadata;
    use riglr_events_core::{prelude::ChainData, EventKind};
    type EventMetadata = SolanaEventMetadata;

    #[test]
    fn test_discriminator_parsing() {
        let route_discriminator = [229, 23, 203, 151, 122, 227, 173, 42];
        assert_eq!(
            Discriminator::from_bytes(&route_discriminator),
            Some(Discriminator::Route)
        );

        let invalid_discriminator = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        assert_eq!(Discriminator::from_bytes(&invalid_discriminator), None);
    }

    #[test]
    fn test_route_instruction_parsing() {
        let parser = Parser::default();

        let mut data = Vec::new();
        data.extend_from_slice(&[229, 23, 203, 151, 122, 227, 173, 42]); // Route discriminator
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount_in
        data.extend_from_slice(&950u64.to_le_bytes()); // minimum_amount_out
        data.extend_from_slice(&50u32.to_le_bytes()); // platform_fee_bps (50 bps = 0.5%)

        let core_metadata = create_core_metadata(
            "test_id".to_string(),
            EventKind::Transaction,
            "test_source".to_string(),
            None,
        )
        .with_chain_data(ChainData::Solana {
            slot: 0,
            signature: None,
            program_id: None,
            instruction_index: None,
            block_time: None,
            protocol_data: Some(serde_json::json!({
                "protocol_type": ProtocolType::Other("temp".to_string()),
                "event_type": EventType::Unknown,
            })),
        });
        let metadata = EventMetadata {
            signature: "test_signature".to_string(),
            slot: 0,
            event_type: EventType::Unknown,
            protocol_type: ProtocolType::Other("temp".to_string()),
            index: "0".to_string(),
            program_received_time_ms: 0,
            core: core_metadata,
        };
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse route instruction data");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("First event should exist");
        assert_eq!(event.protocol_type(), ProtocolType::Jupiter);

        // Check combined parsed data
        let combined = event
            .get_parsed_data::<RouteData>()
            .expect("Failed to get parsed data as RouteData");
        assert_eq!(combined.instruction.amount_in, 1000);
        assert_eq!(combined.instruction.minimum_amount_out, 950);
        assert_eq!(combined.instruction.platform_fee_bps, 50);
        assert_eq!(combined.analysis.instruction_type, "route");
        assert_eq!(combined.analysis.hop_count, 1);
    }

    #[test]
    fn test_can_parse() {
        let parser = Parser::default();

        let valid_data = vec![229, 23, 203, 151, 122, 227, 173, 42, 0, 0]; // Route discriminator
        assert!(parser.can_parse(&valid_data));

        let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        assert!(!parser.can_parse(&invalid_data));

        let short_data = vec![229, 23]; // Too short
        assert!(!parser.can_parse(&short_data));
    }

    #[test]
    fn test_factory() {
        let zero_copy_parser = ParserFactory::create_zero_copy();
        let fast_parser = ParserFactory::create_fast();
        let standard_parser = ParserFactory::create_standard();

        assert_eq!(zero_copy_parser.protocol_type(), ProtocolType::Jupiter);
        assert_eq!(fast_parser.protocol_type(), ProtocolType::Jupiter);
        assert_eq!(standard_parser.protocol_type(), ProtocolType::Jupiter);
    }

    #[test]
    fn test_jupiter_discriminator_all_variants() {
        // Test all discriminator variants
        let route_with_token_ledger = [206, 198, 71, 54, 47, 82, 194, 13];
        assert_eq!(
            Discriminator::from_bytes(&route_with_token_ledger),
            Some(Discriminator::RouteWithTokenLedger)
        );

        let exact_out_route = [123, 87, 17, 219, 42, 126, 197, 78];
        assert_eq!(
            Discriminator::from_bytes(&exact_out_route),
            Some(Discriminator::ExactOutRoute)
        );

        let shared_accounts_route = [95, 180, 10, 172, 84, 174, 232, 239];
        assert_eq!(
            Discriminator::from_bytes(&shared_accounts_route),
            Some(Discriminator::SharedAccountsRoute)
        );

        let shared_accounts_route_with_token_ledger = [18, 99, 149, 21, 45, 126, 144, 122];
        assert_eq!(
            Discriminator::from_bytes(&shared_accounts_route_with_token_ledger),
            Some(Discriminator::SharedAccountsRouteWithTokenLedger)
        );

        let shared_accounts_exact_out_route = [77, 119, 2, 23, 198, 126, 79, 175];
        assert_eq!(
            Discriminator::from_bytes(&shared_accounts_exact_out_route),
            Some(Discriminator::SharedAccountsExactOutRoute)
        );
    }

    #[test]
    fn test_jupiter_discriminator_bytes() {
        // Test bytes() method for all variants
        assert_eq!(
            Discriminator::Route.bytes(),
            &[229, 23, 203, 151, 122, 227, 173, 42]
        );
        assert_eq!(
            Discriminator::RouteWithTokenLedger.bytes(),
            &[206, 198, 71, 54, 47, 82, 194, 13]
        );
        assert_eq!(
            Discriminator::ExactOutRoute.bytes(),
            &[123, 87, 17, 219, 42, 126, 197, 78]
        );
        assert_eq!(
            Discriminator::SharedAccountsRoute.bytes(),
            &[95, 180, 10, 172, 84, 174, 232, 239]
        );
        assert_eq!(
            Discriminator::SharedAccountsRouteWithTokenLedger.bytes(),
            &[18, 99, 149, 21, 45, 126, 144, 122]
        );
        assert_eq!(
            Discriminator::SharedAccountsExactOutRoute.bytes(),
            &[77, 119, 2, 23, 198, 126, 79, 175]
        );
    }

    #[test]
    fn test_jupiter_discriminator_event_type() {
        // All discriminators should return EventType::Swap
        assert_eq!(Discriminator::Route.event_type(), EventType::Swap);
        assert_eq!(
            Discriminator::RouteWithTokenLedger.event_type(),
            EventType::Swap
        );
        assert_eq!(Discriminator::ExactOutRoute.event_type(), EventType::Swap);
        assert_eq!(
            Discriminator::SharedAccountsRoute.event_type(),
            EventType::Swap
        );
        assert_eq!(
            Discriminator::SharedAccountsRouteWithTokenLedger.event_type(),
            EventType::Swap
        );
        assert_eq!(
            Discriminator::SharedAccountsExactOutRoute.event_type(),
            EventType::Swap
        );
    }

    #[test]
    fn test_jupiter_discriminator_from_bytes_edge_cases() {
        // Test with empty slice
        assert_eq!(Discriminator::from_bytes(&[]), None);

        // Test with slice less than 8 bytes
        assert_eq!(Discriminator::from_bytes(&[229, 23, 203]), None);

        // Test with exactly 8 bytes but invalid discriminator
        assert_eq!(Discriminator::from_bytes(&[1, 2, 3, 4, 5, 6, 7, 8]), None);
    }

    #[test]
    fn test_jupiter_parser_new_methods() {
        let default_parser = Parser::default();
        assert!(default_parser.zero_copy);
        assert!(default_parser.detailed_analysis);

        let new_parser = Parser::default();
        assert!(new_parser.zero_copy);
        assert!(new_parser.detailed_analysis);

        let fast_parser = Parser::new_fast();
        assert!(fast_parser.zero_copy);
        assert!(!fast_parser.detailed_analysis);

        let standard_parser = Parser::new_standard();
        assert!(!standard_parser.zero_copy);
        assert!(standard_parser.detailed_analysis);
    }

    #[test]
    fn test_route_with_token_ledger_parsing() {
        let parser = Parser::default();

        let mut data = Vec::new();
        data.extend_from_slice(&[206, 198, 71, 54, 47, 82, 194, 13]); // RouteWithTokenLedger
        data.extend_from_slice(&1500u64.to_le_bytes()); // amount_in
        data.extend_from_slice(&1450u64.to_le_bytes()); // minimum_amount_out
        data.extend_from_slice(&30u32.to_le_bytes()); // platform_fee_bps

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse route with token ledger data");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("First event should exist");
        let combined = event
            .get_parsed_data::<RouteData>()
            .expect("Failed to get parsed data as RouteData");
        assert_eq!(combined.instruction.amount_in, 1500);
        assert_eq!(combined.instruction.minimum_amount_out, 1450);
        assert_eq!(combined.instruction.platform_fee_bps, 30);
        assert_eq!(
            combined.analysis.instruction_type,
            "route_with_token_ledger"
        );
    }

    #[test]
    fn test_exact_out_route_parsing() {
        let parser = Parser::default();

        let mut data = Vec::new();
        data.extend_from_slice(&[123, 87, 17, 219, 42, 126, 197, 78]); // ExactOutRoute
        data.extend_from_slice(&2000u64.to_le_bytes()); // max_amount_in
        data.extend_from_slice(&1900u64.to_le_bytes()); // amount_out
        data.extend_from_slice(&25u32.to_le_bytes()); // platform_fee_bps

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse exact out route data");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("First event should exist");
        let instruction = event
            .get_parsed_data::<ExactOutRouteInstruction>()
            .expect("Failed to get parsed data as ExactOutRouteInstruction");
        assert_eq!(instruction.max_amount_in, 2000);
        assert_eq!(instruction.amount_out, 1900);
        assert_eq!(instruction.platform_fee_bps, 25);
    }

    #[test]
    fn test_shared_accounts_route_parsing() {
        let parser = Parser::default();

        let mut data = Vec::new();
        data.extend_from_slice(&[95, 180, 10, 172, 84, 174, 232, 239]); // SharedAccountsRoute
        data.extend_from_slice(&3000u64.to_le_bytes()); // amount_in
        data.extend_from_slice(&2900u64.to_le_bytes()); // minimum_amount_out
        data.extend_from_slice(&75u32.to_le_bytes()); // platform_fee_bps

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse shared accounts route data");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("First event should exist");
        let combined = event
            .get_parsed_data::<RouteData>()
            .expect("Failed to get parsed data as RouteData");
        assert_eq!(combined.instruction.amount_in, 3000);
        assert_eq!(combined.instruction.minimum_amount_out, 2900);
        assert_eq!(combined.instruction.platform_fee_bps, 75);
        assert_eq!(combined.analysis.instruction_type, "shared_accounts_route");
        assert_eq!(combined.analysis.hop_count, 3); // Multi-hop route
    }

    #[test]
    fn test_shared_accounts_route_with_token_ledger_parsing() {
        let parser = Parser::default();

        let mut data = Vec::new();
        data.extend_from_slice(&[18, 99, 149, 21, 45, 126, 144, 122]); // SharedAccountsRouteWithTokenLedger
        data.extend_from_slice(&4000u64.to_le_bytes()); // amount_in
        data.extend_from_slice(&3800u64.to_le_bytes()); // minimum_amount_out
        data.extend_from_slice(&100u32.to_le_bytes()); // platform_fee_bps

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse shared accounts route with token ledger data");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("First event should exist");
        let combined = event
            .get_parsed_data::<RouteData>()
            .expect("Failed to get parsed data as RouteData");
        assert_eq!(
            combined.analysis.instruction_type,
            "shared_accounts_route_with_token_ledger"
        );
        assert_eq!(combined.analysis.hop_count, 3); // Multi-hop route
    }

    #[test]
    fn test_shared_accounts_exact_out_route_parsing() {
        let parser = Parser::default();

        let mut data = Vec::new();
        data.extend_from_slice(&[77, 119, 2, 23, 198, 126, 79, 175]); // SharedAccountsExactOutRoute
        data.extend_from_slice(&5000u64.to_le_bytes()); // max_amount_in
        data.extend_from_slice(&4800u64.to_le_bytes()); // amount_out
        data.extend_from_slice(&120u32.to_le_bytes()); // platform_fee_bps

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse shared accounts exact out route data");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("First event should exist");
        let instruction = event
            .get_parsed_data::<ExactOutRouteInstruction>()
            .expect("Failed to get parsed data as ExactOutRouteInstruction");
        assert_eq!(instruction.max_amount_in, 5000);
        assert_eq!(instruction.amount_out, 4800);
        assert_eq!(instruction.platform_fee_bps, 120);
    }

    #[test]
    fn test_route_without_platform_fee() {
        let parser = Parser::default();

        let mut data = Vec::new();
        data.extend_from_slice(&[229, 23, 203, 151, 122, 227, 173, 42]); // Route discriminator
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount_in
        data.extend_from_slice(&950u64.to_le_bytes()); // minimum_amount_out
                                                       // No platform fee bytes

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse route without platform fee");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("First event should exist");
        let combined = event
            .get_parsed_data::<RouteData>()
            .expect("Failed to get parsed data as RouteData");
        assert_eq!(combined.instruction.platform_fee_bps, 0); // Default to 0
    }

    #[test]
    fn test_exact_out_route_without_platform_fee() {
        let parser = Parser::default();

        let mut data = Vec::new();
        data.extend_from_slice(&[123, 87, 17, 219, 42, 126, 197, 78]); // ExactOutRoute
        data.extend_from_slice(&2000u64.to_le_bytes()); // max_amount_in
        data.extend_from_slice(&1900u64.to_le_bytes()); // amount_out
                                                        // No platform fee bytes

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse exact out route without platform fee");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("First event should exist");
        let instruction = event
            .get_parsed_data::<ExactOutRouteInstruction>()
            .expect("Failed to get parsed data as ExactOutRouteInstruction");
        assert_eq!(instruction.platform_fee_bps, 0); // Default to 0
    }

    #[test]
    fn test_parser_with_fast_mode() {
        let parser = Parser::new_fast();

        let mut data = Vec::new();
        data.extend_from_slice(&[229, 23, 203, 151, 122, 227, 173, 42]); // Route discriminator
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount_in
        data.extend_from_slice(&950u64.to_le_bytes()); // minimum_amount_out
        data.extend_from_slice(&50u32.to_le_bytes()); // platform_fee_bps

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse route in fast mode");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("First event should exist");
        let combined = event
            .get_parsed_data::<RouteData>()
            .expect("Failed to get parsed data as RouteData");
        assert_eq!(combined.analysis.hop_count, 1); // Default assumption in fast mode
        assert!(combined.analysis.hops.is_empty()); // No detailed analysis
    }

    #[test]
    fn test_parser_with_standard_mode() {
        let parser = Parser::new_standard();

        let mut data = Vec::new();
        data.extend_from_slice(&[229, 23, 203, 151, 122, 227, 173, 42]); // Route discriminator
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount_in
        data.extend_from_slice(&950u64.to_le_bytes()); // minimum_amount_out
        data.extend_from_slice(&50u32.to_le_bytes()); // platform_fee_bps

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse route in standard mode");

        assert_eq!(events.len(), 1);
        // Standard mode should work just like default mode for parsing
    }

    #[test]
    fn test_parse_from_slice_insufficient_data() {
        let parser = Parser::default();
        let short_data = vec![229, 23, 203, 151]; // Only 4 bytes
        let metadata = EventMetadata::default();

        let result = parser.parse_from_slice(&short_data, metadata);
        assert!(result.is_err());

        if let Err(ParseError::InsufficientData { expected, actual }) = result {
            assert_eq!(expected, 8);
            assert_eq!(actual, 4);
        } else {
            panic!("Expected InsufficientData error");
        }
    }

    #[test]
    fn test_parse_from_slice_unknown_discriminator() {
        let parser = Parser::default();
        let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0];
        let metadata = EventMetadata::default();

        let result = parser.parse_from_slice(&invalid_data, metadata);
        assert!(result.is_err());

        if let Err(ParseError::UnknownDiscriminator { discriminator }) = result {
            assert_eq!(
                discriminator,
                vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
            );
        } else {
            panic!("Expected UnknownDiscriminator error");
        }
    }

    #[test]
    fn test_can_parse_edge_cases() {
        let parser = Parser::default();

        // Empty data
        assert!(!parser.can_parse(&[]));

        // Exactly 8 bytes with valid discriminator
        let valid_8_bytes = vec![229, 23, 203, 151, 122, 227, 173, 42];
        assert!(parser.can_parse(&valid_8_bytes));

        // More than 8 bytes with valid discriminator
        let valid_longer = vec![229, 23, 203, 151, 122, 227, 173, 42, 0, 0, 0, 0];
        assert!(parser.can_parse(&valid_longer));

        // 7 bytes (one short)
        let seven_bytes = vec![229, 23, 203, 151, 122, 227, 173];
        assert!(!parser.can_parse(&seven_bytes));
    }

    #[test]
    fn test_protocol_type() {
        let parser = Parser::default();
        assert_eq!(parser.protocol_type(), ProtocolType::Jupiter);
    }

    #[test]
    fn test_analyze_route_all_instruction_types() {
        // Test all discriminator types for analyze_route
        let test_cases = vec![
            (Discriminator::Route, "route", 1),
            (
                Discriminator::RouteWithTokenLedger,
                "route_with_token_ledger",
                1,
            ),
            (Discriminator::ExactOutRoute, "exact_out_route", 1),
            (
                Discriminator::SharedAccountsRoute,
                "shared_accounts_route",
                3,
            ),
            (
                Discriminator::SharedAccountsRouteWithTokenLedger,
                "shared_accounts_route_with_token_ledger",
                3,
            ),
            (
                Discriminator::SharedAccountsExactOutRoute,
                "shared_accounts_exact_out_route",
                3,
            ),
        ];

        for (discriminator, expected_type, expected_hop_count) in test_cases {
            let result = Parser::analyze_route(
                &[], // data not used in current implementation
                discriminator,
                1000,
                950,
                25,
            );

            assert_eq!(result.instruction_type, expected_type);
            assert_eq!(result.total_amount_in, 1000);
            assert_eq!(result.total_amount_out, 950);
            assert_eq!(result.platform_fee_bps, 25);
            assert_eq!(result.hop_count, expected_hop_count);
            assert!(result.hops.is_empty()); // Always empty in current implementation
        }
    }

    #[test]
    fn test_route_hop_debug_clone() {
        // Test that RouteHop implements Debug and Clone
        let hop = RouteHop {
            program_id: Pubkey::default(),
            input_mint: Some(Pubkey::default()),
            output_mint: None,
            amount_in: Some(1000),
            amount_out: None,
        };

        let cloned_hop = hop.clone();
        assert_eq!(hop.program_id, cloned_hop.program_id);
        assert_eq!(hop.input_mint, cloned_hop.input_mint);
        assert_eq!(hop.output_mint, cloned_hop.output_mint);
        assert_eq!(hop.amount_in, cloned_hop.amount_in);
        assert_eq!(hop.amount_out, cloned_hop.amount_out);

        // Test Debug format
        let debug_string = format!("{hop:?}");
        assert!(debug_string.contains("RouteHop"));
    }

    #[test]
    fn test_jupiter_route_data_debug_clone() {
        // Test that RouteData implements Debug and Clone
        let instruction = RouteInstruction {
            amount_in: 1000,
            minimum_amount_out: 950,
            platform_fee_bps: 25,
        };

        let analysis = RouteAnalysis {
            instruction_type: "test".to_owned(),
            total_amount_in: 1000,
            total_amount_out: 950,
            platform_fee_bps: 25,
            hop_count: 1,
            hops: Vec::new(),
        };

        let route_data = RouteData {
            analysis,
            instruction,
        };

        let cloned_data = route_data.clone();
        assert_eq!(
            route_data.instruction.amount_in,
            cloned_data.instruction.amount_in
        );
        assert_eq!(
            route_data.analysis.instruction_type,
            cloned_data.analysis.instruction_type
        );

        // Test Debug format
        let debug_string = format!("{route_data:?}");
        assert!(debug_string.contains("RouteData"));
    }

    #[test]
    fn test_jupiter_exact_out_route_instruction_debug_clone() {
        // Test that ExactOutRouteInstruction implements Debug and Clone
        let instruction = ExactOutRouteInstruction {
            max_amount_in: 2000,
            amount_out: 1900,
            platform_fee_bps: 50,
        };

        let cloned = instruction.clone();
        assert_eq!(instruction.max_amount_in, cloned.max_amount_in);
        assert_eq!(instruction.amount_out, cloned.amount_out);
        assert_eq!(instruction.platform_fee_bps, cloned.platform_fee_bps);

        // Test Debug format
        let debug_string = format!("{instruction:?}");
        assert!(debug_string.contains("ExactOutRouteInstruction"));
    }

    #[test]
    fn test_jupiter_route_analysis_debug_clone() {
        // Test that RouteAnalysis implements Debug and Clone
        let analysis = RouteAnalysis {
            instruction_type: "test_type".to_owned(),
            total_amount_in: 5000,
            total_amount_out: 4800,
            platform_fee_bps: 100,
            hop_count: 2,
            hops: vec![RouteHop {
                program_id: Pubkey::default(),
                input_mint: None,
                output_mint: None,
                amount_in: None,
                amount_out: None,
            }],
        };

        let cloned = analysis.clone();
        assert_eq!(analysis.instruction_type, cloned.instruction_type);
        assert_eq!(analysis.total_amount_in, cloned.total_amount_in);
        assert_eq!(analysis.total_amount_out, cloned.total_amount_out);
        assert_eq!(analysis.platform_fee_bps, cloned.platform_fee_bps);
        assert_eq!(analysis.hop_count, cloned.hop_count);
        assert_eq!(analysis.hops.len(), cloned.hops.len());

        // Test Debug format
        let debug_string = format!("{analysis:?}");
        assert!(debug_string.contains("RouteAnalysis"));
    }

    #[test]
    fn test_jupiter_discriminator_partial_eq() {
        // Test PartialEq implementation
        assert_eq!(Discriminator::Route, Discriminator::Route);
        assert_ne!(Discriminator::Route, Discriminator::ExactOutRoute);

        // Test with all variants
        let discriminators = [
            Discriminator::Route,
            Discriminator::RouteWithTokenLedger,
            Discriminator::ExactOutRoute,
            Discriminator::SharedAccountsRoute,
            Discriminator::SharedAccountsRouteWithTokenLedger,
            Discriminator::SharedAccountsExactOutRoute,
        ];

        for (i, disc1) in discriminators.iter().enumerate() {
            for (j, disc2) in discriminators.iter().enumerate() {
                if i == j {
                    assert_eq!(disc1, disc2);
                } else {
                    assert_ne!(disc1, disc2);
                }
            }
        }
    }

    #[test]
    fn test_platform_fee_bps_masking() {
        // Test that platform_fee_bps properly masks to 16 bits
        let parser = Parser::default();

        let mut data = Vec::new();
        data.extend_from_slice(&[229, 23, 203, 151, 122, 227, 173, 42]); // Route discriminator
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount_in
        data.extend_from_slice(&950u64.to_le_bytes()); // minimum_amount_out
        data.extend_from_slice(&0xFFFF_0032_u32.to_le_bytes()); // platform_fee_bps with high bits set

        let metadata = EventMetadata::default();
        let events = parser
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse route for platform fee bps masking test");

        let event = events.first().expect("First event should exist");
        let combined = event
            .get_parsed_data::<RouteData>()
            .expect("Failed to get parsed data as RouteData");
        assert_eq!(combined.instruction.platform_fee_bps, 0x0032); // Should be masked to 16 bits
    }

    #[test]
    fn test_zero_copy_vs_owned_event_creation() {
        // Test zero_copy = true (borrowed data)
        let parser_zero_copy = Parser::default();
        let mut data = Vec::new();
        data.extend_from_slice(&[229, 23, 203, 151, 122, 227, 173, 42]);
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&950u64.to_le_bytes());

        let metadata = EventMetadata::default();
        let events = parser_zero_copy
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse route with zero copy parser");
        assert_eq!(events.len(), 1);

        // Test zero_copy = false (owned data)
        let parser_standard = Parser::new_standard();
        let metadata = EventMetadata::default();
        let events = parser_standard
            .parse_from_slice(&data, metadata)
            .expect("Failed to parse route with standard parser");
        assert_eq!(events.len(), 1);
    }
}
