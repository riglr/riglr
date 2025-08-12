use std::sync::{LazyLock, Arc};
use crate::events::{
    core::traits::EventParser,
    protocols::{
        PumpSwapEventParser, BonkEventParser, RaydiumCpmmEventParser,
        pumpswap::PUMPSWAP_PROGRAM_ID,
        bonk::BONK_PROGRAM_ID, 
        raydium_cpmm::RAYDIUM_CPMM_PROGRAM_ID,
    },
};

/// Supported protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    PumpSwap,
    Bonk,
    RaydiumCpmm,
    RaydiumClmm,
    RaydiumAmmV4,
}

impl Protocol {
    pub fn all() -> Vec<Protocol> {
        vec![
            Protocol::PumpSwap,
            Protocol::Bonk,
            Protocol::RaydiumCpmm,
            Protocol::RaydiumClmm,
            Protocol::RaydiumAmmV4,
        ]
    }
}

/// Global parser instances using LazyLock
static PUMPSWAP_PARSER: LazyLock<Arc<PumpSwapEventParser>> = LazyLock::new(|| {
    Arc::new(PumpSwapEventParser::new())
});

static BONK_PARSER: LazyLock<Arc<BonkEventParser>> = LazyLock::new(|| {
    Arc::new(BonkEventParser::new())
});

static RAYDIUM_CPMM_PARSER: LazyLock<Arc<RaydiumCpmmEventParser>> = LazyLock::new(|| {
    Arc::new(RaydiumCpmmEventParser::new())
});

/// Event parser factory
pub struct EventParserFactory;

impl EventParserFactory {
    /// Create a specific parser
    pub fn create_parser(protocol: Protocol) -> Arc<dyn EventParser> {
        match protocol {
            Protocol::PumpSwap => PUMPSWAP_PARSER.clone(),
            Protocol::Bonk => BONK_PARSER.clone(),
            Protocol::RaydiumCpmm => RAYDIUM_CPMM_PARSER.clone(),
            Protocol::RaydiumClmm => RAYDIUM_CPMM_PARSER.clone(), // Using CPMM as placeholder
            Protocol::RaydiumAmmV4 => RAYDIUM_CPMM_PARSER.clone(), // Using CPMM as placeholder
        }
    }

    /// Create all parsers
    pub fn create_all_parsers() -> Vec<Arc<dyn EventParser>> {
        Protocol::all().into_iter()
            .map(Self::create_parser)
            .collect()
    }

    /// Create multi-protocol parser
    pub fn create_mutil_parser(protocols: &[Protocol]) -> MutilEventParser {
        let parsers = protocols.iter()
            .map(|&protocol| Self::create_parser(protocol))
            .collect();
        
        MutilEventParser::new(parsers)
    }

    /// Get parser for a specific program ID
    pub fn get_parser_for_program_id(program_id: &solana_sdk::pubkey::Pubkey) -> Option<Arc<dyn EventParser>> {
        match *program_id {
            PUMPSWAP_PROGRAM_ID => Some(PUMPSWAP_PARSER.clone()),
            BONK_PROGRAM_ID => Some(BONK_PARSER.clone()),
            RAYDIUM_CPMM_PROGRAM_ID => Some(RAYDIUM_CPMM_PARSER.clone()),
            _ => None,
        }
    }
}

/// Multi-protocol event parser that combines multiple parsers
pub struct MutilEventParser {
    parsers: Vec<Arc<dyn EventParser>>,
}

impl MutilEventParser {
    pub fn new(parsers: Vec<Arc<dyn EventParser>>) -> Self {
        Self { parsers }
    }

    /// Get all supported program IDs from all parsers
    pub fn supported_program_ids(&self) -> Vec<solana_sdk::pubkey::Pubkey> {
        self.parsers.iter()
            .flat_map(|parser| parser.supported_program_ids())
            .collect()
    }

    /// Check if any parser can handle this program ID
    pub fn should_handle(&self, program_id: &solana_sdk::pubkey::Pubkey) -> bool {
        self.parsers.iter().any(|parser| parser.should_handle(program_id))
    }

    /// Get parser that can handle this program ID
    pub fn get_parser_for_program(&self, program_id: &solana_sdk::pubkey::Pubkey) -> Option<&Arc<dyn EventParser>> {
        self.parsers.iter().find(|parser| parser.should_handle(program_id))
    }
}

#[async_trait::async_trait]
impl EventParser for MutilEventParser {
    fn inner_instruction_configs(&self) -> std::collections::HashMap<&'static str, Vec<crate::events::core::traits::GenericEventParseConfig>> {
        let mut combined = std::collections::HashMap::new();
        for parser in &self.parsers {
            let configs = parser.inner_instruction_configs();
            for (disc, mut parser_configs) in configs {
                combined.entry(disc).or_insert_with(Vec::new).append(&mut parser_configs);
            }
        }
        combined
    }

    fn instruction_configs(&self) -> std::collections::HashMap<Vec<u8>, Vec<crate::events::core::traits::GenericEventParseConfig>> {
        let mut combined = std::collections::HashMap::new();
        for parser in &self.parsers {
            let configs = parser.instruction_configs();
            for (disc, mut parser_configs) in configs {
                combined.entry(disc).or_insert_with(Vec::new).append(&mut parser_configs);
            }
        }
        combined
    }

    fn parse_events_from_inner_instruction(
        &self,
        inner_instruction: &solana_transaction_status::UiCompiledInstruction,
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn crate::events::core::traits::UnifiedEvent>> {
        let mut all_events = Vec::new();
        for parser in &self.parsers {
            let mut events = parser.parse_events_from_inner_instruction(
                inner_instruction,
                signature,
                slot,
                block_time,
                program_received_time_ms,
                index.clone(),
            );
            all_events.append(&mut events);
        }
        all_events
    }

    fn parse_events_from_instruction(
        &self,
        instruction: &solana_sdk::instruction::CompiledInstruction,
        accounts: &[solana_sdk::pubkey::Pubkey],
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn crate::events::core::traits::UnifiedEvent>> {
        if let Some(program_id) = accounts.get(instruction.program_id_index as usize) {
            if let Some(parser) = self.get_parser_for_program(program_id) {
                return parser.parse_events_from_instruction(
                    instruction,
                    accounts,
                    signature,
                    slot,
                    block_time,
                    program_received_time_ms,
                    index,
                );
            }
        }
        Vec::new()
    }

    fn should_handle(&self, program_id: &solana_sdk::pubkey::Pubkey) -> bool {
        self.should_handle(program_id)
    }

    fn supported_program_ids(&self) -> Vec<solana_sdk::pubkey::Pubkey> {
        self.supported_program_ids()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_creates_parsers() {
        let pumpswap_parser = EventParserFactory::create_parser(Protocol::PumpSwap);
        assert!(!pumpswap_parser.supported_program_ids().is_empty());

        let bonk_parser = EventParserFactory::create_parser(Protocol::Bonk);
        assert!(!bonk_parser.supported_program_ids().is_empty());
    }

    #[test]
    fn test_mutil_parser_creation() {
        let protocols = vec![Protocol::PumpSwap, Protocol::Bonk];
        let mutil_parser = EventParserFactory::create_mutil_parser(&protocols);
        
        let supported_ids = mutil_parser.supported_program_ids();
        assert!(!supported_ids.is_empty());
        assert!(supported_ids.contains(&PUMPSWAP_PROGRAM_ID));
        assert!(supported_ids.contains(&BONK_PROGRAM_ID));
    }

    #[test]
    fn test_factory_get_parser_by_program_id() {
        let parser = EventParserFactory::get_parser_for_program_id(&PUMPSWAP_PROGRAM_ID);
        assert!(parser.is_some());

        let nonexistent = solana_sdk::pubkey::Pubkey::new_unique();
        let parser = EventParserFactory::get_parser_for_program_id(&nonexistent);
        assert!(parser.is_none());
    }
}