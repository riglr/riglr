//! High-performance Jupiter aggregator parser for complex swap routing analysis
//!
//! This parser handles Jupiter protocol operations with optimized parsing for
//! multi-hop swaps and complex routing instructions.

use crate::types::{EventMetadata, EventType, ProtocolType};
use crate::zero_copy::{ByteSliceEventParser, CustomDeserializer, ParseError, ZeroCopyEvent};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
// UnifiedEvent trait has been removed

/// Jupiter aggregator program ID
pub const JUPITER_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";

/// Jupiter instruction discriminators (using Anchor discriminators)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JupiterDiscriminator {
    /// Route discriminator: [229, 23, 203, 151, 122, 227, 173, 42]
    Route,
    /// RouteWithTokenLedger discriminator: [206, 198, 71, 54, 47, 82, 194, 13]
    RouteWithTokenLedger,
    /// ExactOutRoute discriminator: [123, 87, 17, 219, 42, 126, 197, 78]
    ExactOutRoute,
    /// SharedAccountsRoute discriminator: [95, 180, 10, 172, 84, 174, 232, 239]
    SharedAccountsRoute,
    /// SharedAccountsRouteWithTokenLedger discriminator: [18, 99, 149, 21, 45, 126, 144, 122]
    SharedAccountsRouteWithTokenLedger,
    /// SharedAccountsExactOutRoute discriminator: [77, 119, 2, 23, 198, 126, 79, 175]
    SharedAccountsExactOutRoute,
}

impl JupiterDiscriminator {
    /// Parse discriminator from first 8 bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 8 {
            return None;
        }

        match bytes {
            [229, 23, 203, 151, 122, 227, 173, 42] => Some(Self::Route),
            [206, 198, 71, 54, 47, 82, 194, 13] => Some(Self::RouteWithTokenLedger),
            [123, 87, 17, 219, 42, 126, 197, 78] => Some(Self::ExactOutRoute),
            [95, 180, 10, 172, 84, 174, 232, 239] => Some(Self::SharedAccountsRoute),
            [18, 99, 149, 21, 45, 126, 144, 122] => Some(Self::SharedAccountsRouteWithTokenLedger),
            [77, 119, 2, 23, 198, 126, 79, 175] => Some(Self::SharedAccountsExactOutRoute),
            _ => None,
        }
    }

    /// Get corresponding event type
    pub fn event_type(&self) -> EventType {
        EventType::Swap // Jupiter operations are always swaps
    }

    /// Get discriminator bytes
    pub fn bytes(&self) -> &'static [u8; 8] {
        match self {
            Self::Route => &[229, 23, 203, 151, 122, 227, 173, 42],
            Self::RouteWithTokenLedger => &[206, 198, 71, 54, 47, 82, 194, 13],
            Self::ExactOutRoute => &[123, 87, 17, 219, 42, 126, 197, 78],
            Self::SharedAccountsRoute => &[95, 180, 10, 172, 84, 174, 232, 239],
            Self::SharedAccountsRouteWithTokenLedger => &[18, 99, 149, 21, 45, 126, 144, 122],
            Self::SharedAccountsExactOutRoute => &[77, 119, 2, 23, 198, 126, 79, 175],
        }
    }
}

/// Jupiter Route instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct JupiterRouteInstruction {
    /// Amount to swap in
    pub amount_in: u64,
    /// Minimum amount out expected
    pub minimum_amount_out: u64,
    /// Platform fee basis points
    pub platform_fee_bps: u16,
}

/// Jupiter ExactOutRoute instruction data
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct JupiterExactOutRouteInstruction {
    /// Maximum amount in
    pub max_amount_in: u64,
    /// Exact amount out desired
    pub amount_out: u64,
    /// Platform fee basis points
    pub platform_fee_bps: u16,
}

/// Simplified route hop representation
#[derive(Debug, Clone)]
pub struct RouteHop {
    /// DEX program ID
    pub program_id: Pubkey,
    /// Input mint
    pub input_mint: Option<Pubkey>,
    /// Output mint
    pub output_mint: Option<Pubkey>,
    /// Amount in for this hop
    pub amount_in: Option<u64>,
    /// Amount out for this hop
    pub amount_out: Option<u64>,
}

/// Combined Jupiter Route Data (instruction + analysis)
#[derive(Debug, Clone)]
pub struct JupiterRouteData {
    pub instruction: JupiterRouteInstruction,
    pub analysis: JupiterRouteAnalysis,
}

/// Jupiter route analysis result
#[derive(Debug, Clone)]
pub struct JupiterRouteAnalysis {
    /// Instruction type
    pub instruction_type: String,
    /// Total amount in
    pub total_amount_in: u64,
    /// Total amount out (minimum or exact)
    pub total_amount_out: u64,
    /// Platform fee in basis points
    pub platform_fee_bps: u16,
    /// Number of hops in the route
    pub hop_count: usize,
    /// Route hops (simplified analysis)
    pub hops: Vec<RouteHop>,
}

/// High-performance Jupiter parser
pub struct JupiterParser {
    /// Program ID for validation
    #[allow(dead_code)]
    program_id: Pubkey,
    /// Enable zero-copy parsing
    zero_copy: bool,
    /// Enable detailed route analysis
    detailed_analysis: bool,
}

impl Default for JupiterParser {
    fn default() -> Self {
        Self {
            program_id: JUPITER_PROGRAM_ID
                .parse()
                .expect("Valid Jupiter program ID"),
            zero_copy: true,
            detailed_analysis: true,
        }
    }
}

impl JupiterParser {
    /// Create new Jupiter parser with full analysis
    pub fn new() -> Self {
        Self {
            program_id: JUPITER_PROGRAM_ID
                .parse()
                .expect("Valid Jupiter program ID"),
            zero_copy: true,
            detailed_analysis: true,
        }
    }

    /// Create parser with simplified analysis (faster)
    pub fn new_fast() -> Self {
        Self {
            program_id: JUPITER_PROGRAM_ID
                .parse()
                .expect("Valid Jupiter program ID"),
            zero_copy: true,
            detailed_analysis: false,
        }
    }

    /// Create parser with zero-copy disabled
    pub fn new_standard() -> Self {
        Self {
            program_id: JUPITER_PROGRAM_ID
                .parse()
                .expect("Valid Jupiter program ID"),
            zero_copy: false,
            detailed_analysis: true,
        }
    }

    /// Parse Route instruction
    fn parse_route<'a>(
        &self,
        data: &'a [u8],
        discriminator: JupiterDiscriminator,
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

        let instruction_data = JupiterRouteInstruction {
            amount_in,
            minimum_amount_out,
            platform_fee_bps,
        };

        // Perform route analysis if enabled
        let analysis = if self.detailed_analysis {
            self.analyze_route(
                data,
                discriminator,
                amount_in,
                minimum_amount_out,
                platform_fee_bps,
            )?
        } else {
            JupiterRouteAnalysis {
                instruction_type: format!("{:?}", discriminator),
                total_amount_in: amount_in,
                total_amount_out: minimum_amount_out,
                platform_fee_bps,
                hop_count: 1, // Default assumption
                hops: Vec::new(),
            }
        };

        // Store combined data
        let combined_data = JupiterRouteData {
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

    /// Parse ExactOutRoute instruction
    fn parse_exact_out_route<'a>(
        &self,
        data: &'a [u8],
        discriminator: JupiterDiscriminator,
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

        let instruction_data = JupiterExactOutRouteInstruction {
            max_amount_in,
            amount_out,
            platform_fee_bps,
        };
        event.set_parsed_data(instruction_data);

        let analysis = if self.detailed_analysis {
            self.analyze_route(
                data,
                discriminator,
                max_amount_in,
                amount_out,
                platform_fee_bps,
            )?
        } else {
            JupiterRouteAnalysis {
                instruction_type: format!("{:?}", discriminator),
                total_amount_in: max_amount_in,
                total_amount_out: amount_out,
                platform_fee_bps,
                hop_count: 1,
                hops: Vec::new(),
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

    /// Analyze route structure (simplified implementation)
    fn analyze_route(
        &self,
        _data: &[u8],
        discriminator: JupiterDiscriminator,
        amount_in: u64,
        amount_out: u64,
        platform_fee_bps: u16,
    ) -> Result<JupiterRouteAnalysis, ParseError> {
        // In a full implementation, this would parse the route structure
        // including all the accounts and intermediate hops.
        // For now, we provide a simplified analysis.

        let instruction_type = match discriminator {
            JupiterDiscriminator::Route => "route".to_string(),
            JupiterDiscriminator::RouteWithTokenLedger => "route_with_token_ledger".to_string(),
            JupiterDiscriminator::ExactOutRoute => "exact_out_route".to_string(),
            JupiterDiscriminator::SharedAccountsRoute => "shared_accounts_route".to_string(),
            JupiterDiscriminator::SharedAccountsRouteWithTokenLedger => {
                "shared_accounts_route_with_token_ledger".to_string()
            }
            JupiterDiscriminator::SharedAccountsExactOutRoute => {
                "shared_accounts_exact_out_route".to_string()
            }
        };

        // Estimate hop count based on instruction type
        let hop_count = match discriminator {
            JupiterDiscriminator::SharedAccountsRoute
            | JupiterDiscriminator::SharedAccountsRouteWithTokenLedger
            | JupiterDiscriminator::SharedAccountsExactOutRoute => 3, // Multi-hop routes
            _ => 1, // Single hop routes
        };

        Ok(JupiterRouteAnalysis {
            instruction_type,
            total_amount_in: amount_in,
            total_amount_out: amount_out,
            platform_fee_bps,
            hop_count,
            hops: Vec::new(), // Would be populated in full implementation
        })
    }
}

impl ByteSliceEventParser for JupiterParser {
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
        metadata.protocol_type = ProtocolType::Jupiter;

        // Parse discriminator from first 8 bytes
        let discriminator = JupiterDiscriminator::from_bytes(&data[0..8]).ok_or_else(|| {
            ParseError::UnknownDiscriminator {
                discriminator: data[0..8].to_vec(),
            }
        })?;

        // Update event type
        metadata.event_type = discriminator.event_type();

        let event = match discriminator {
            JupiterDiscriminator::Route
            | JupiterDiscriminator::RouteWithTokenLedger
            | JupiterDiscriminator::SharedAccountsRoute
            | JupiterDiscriminator::SharedAccountsRouteWithTokenLedger => {
                self.parse_route(data, discriminator, metadata)?
            }
            JupiterDiscriminator::ExactOutRoute
            | JupiterDiscriminator::SharedAccountsExactOutRoute => {
                self.parse_exact_out_route(data, discriminator, metadata)?
            }
        };

        Ok(vec![event])
    }

    fn can_parse(&self, data: &[u8]) -> bool {
        if data.len() < 8 {
            return false;
        }

        JupiterDiscriminator::from_bytes(&data[0..8]).is_some()
    }

    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::Jupiter
    }
}

/// Factory for creating Jupiter parsers
pub struct JupiterParserFactory;

impl JupiterParserFactory {
    /// Create a new high-performance zero-copy parser with full analysis
    pub fn create_zero_copy() -> Arc<dyn ByteSliceEventParser> {
        Arc::new(JupiterParser::new())
    }

    /// Create a fast parser with simplified analysis
    pub fn create_fast() -> Arc<dyn ByteSliceEventParser> {
        Arc::new(JupiterParser::new_fast())
    }

    /// Create a standard parser
    pub fn create_standard() -> Arc<dyn ByteSliceEventParser> {
        Arc::new(JupiterParser::new_standard())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EventMetadata;

    #[test]
    fn test_discriminator_parsing() {
        let route_discriminator = [229, 23, 203, 151, 122, 227, 173, 42];
        assert_eq!(
            JupiterDiscriminator::from_bytes(&route_discriminator),
            Some(JupiterDiscriminator::Route)
        );

        let invalid_discriminator = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        assert_eq!(
            JupiterDiscriminator::from_bytes(&invalid_discriminator),
            None
        );
    }

    #[test]
    fn test_route_instruction_parsing() {
        let parser = JupiterParser::new();

        let mut data = Vec::new();
        data.extend_from_slice(&[229, 23, 203, 151, 122, 227, 173, 42]); // Route discriminator
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount_in
        data.extend_from_slice(&950u64.to_le_bytes()); // minimum_amount_out
        data.extend_from_slice(&50u32.to_le_bytes()); // platform_fee_bps (50 bps = 0.5%)

        let metadata = EventMetadata::default();
        let events = parser.parse_from_slice(&data, metadata).unwrap();

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.protocol_type(), ProtocolType::Jupiter);

        // Check combined parsed data
        let combined = event.get_parsed_data::<JupiterRouteData>().unwrap();
        assert_eq!(combined.instruction.amount_in, 1000);
        assert_eq!(combined.instruction.minimum_amount_out, 950);
        assert_eq!(combined.instruction.platform_fee_bps, 50);
        assert_eq!(combined.analysis.instruction_type, "route");
        assert_eq!(combined.analysis.hop_count, 1);
    }

    #[test]
    fn test_can_parse() {
        let parser = JupiterParser::new();

        let valid_data = vec![229, 23, 203, 151, 122, 227, 173, 42, 0, 0]; // Route discriminator
        assert!(parser.can_parse(&valid_data));

        let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        assert!(!parser.can_parse(&invalid_data));

        let short_data = vec![229, 23]; // Too short
        assert!(!parser.can_parse(&short_data));
    }

    #[test]
    fn test_factory() {
        let zero_copy_parser = JupiterParserFactory::create_zero_copy();
        let fast_parser = JupiterParserFactory::create_fast();
        let standard_parser = JupiterParserFactory::create_standard();

        assert_eq!(zero_copy_parser.protocol_type(), ProtocolType::Jupiter);
        assert_eq!(fast_parser.protocol_type(), ProtocolType::Jupiter);
        assert_eq!(standard_parser.protocol_type(), ProtocolType::Jupiter);
    }
}
