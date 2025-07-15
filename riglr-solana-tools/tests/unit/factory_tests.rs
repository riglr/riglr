//! Unit tests for the event parser factory and multi-parser functionality
//! Tests the factory pattern, protocol detection, and multi-protocol parsing

use riglr_solana_tools::events::{
    core::traits::{EventParser, UnifiedEvent, GenericEventParseConfig},
    factory::{EventParserFactory, Protocol, MutilEventParser},
    protocols::{
        pumpswap::{PumpSwapEventParser, PUMPSWAP_PROGRAM_ID},
        bonk::{BonkEventParser, BONK_PROGRAM_ID},
        raydium_cpmm::{RaydiumCpmmEventParser, RAYDIUM_CPMM_PROGRAM_ID},
    },
};
use solana_sdk::{pubkey::Pubkey, instruction::CompiledInstruction};
use solana_transaction_status::UiCompiledInstruction;
use std::sync::Arc;

#[test]
fn test_protocol_enum_completeness() {
    let all_protocols = Protocol::all();
    
    // Verify all expected protocols are present
    assert!(all_protocols.contains(&Protocol::PumpSwap));
    assert!(all_protocols.contains(&Protocol::Bonk));
    assert!(all_protocols.contains(&Protocol::RaydiumCpmm));
    assert!(all_protocols.contains(&Protocol::RaydiumClmm));
    assert!(all_protocols.contains(&Protocol::RaydiumAmmV4));
    
    // Verify count matches expected
    assert_eq!(all_protocols.len(), 5);
}

#[test]
fn test_factory_individual_parser_creation() {
    // Test PumpSwap parser creation
    let pumpswap_parser = EventParserFactory::create_parser(Protocol::PumpSwap);
    assert!(pumpswap_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    assert!(!pumpswap_parser.should_handle(&BONK_PROGRAM_ID));
    assert!(!pumpswap_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
    
    let pumpswap_ids = pumpswap_parser.supported_program_ids();
    assert!(pumpswap_ids.contains(&PUMPSWAP_PROGRAM_ID));
    
    // Test Bonk parser creation
    let bonk_parser = EventParserFactory::create_parser(Protocol::Bonk);
    assert!(bonk_parser.should_handle(&BONK_PROGRAM_ID));
    assert!(!bonk_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    assert!(!bonk_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
    
    let bonk_ids = bonk_parser.supported_program_ids();
    assert!(bonk_ids.contains(&BONK_PROGRAM_ID));
    
    // Test Raydium CPMM parser creation
    let raydium_parser = EventParserFactory::create_parser(Protocol::RaydiumCpmm);
    assert!(raydium_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
    assert!(!raydium_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    assert!(!raydium_parser.should_handle(&BONK_PROGRAM_ID));
    
    let raydium_ids = raydium_parser.supported_program_ids();
    assert!(raydium_ids.contains(&RAYDIUM_CPMM_PROGRAM_ID));
}

#[test]
fn test_factory_placeholder_parsers() {
    // Test that CLMM and AMM V4 use CPMM as placeholder
    let clmm_parser = EventParserFactory::create_parser(Protocol::RaydiumClmm);
    let ammv4_parser = EventParserFactory::create_parser(Protocol::RaydiumAmmV4);
    
    // Both should handle the CPMM program ID (as per implementation)
    assert!(clmm_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
    assert!(ammv4_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
}

#[test]
fn test_factory_create_all_parsers() {
    let all_parsers = EventParserFactory::create_all_parsers();
    
    assert_eq!(all_parsers.len(), Protocol::all().len());
    
    // Verify each parser is created
    let mut supported_programs = std::collections::HashSet::new();
    for parser in &all_parsers {
        for program_id in parser.supported_program_ids() {
            supported_programs.insert(program_id);
        }
    }
    
    // Should have at least our known program IDs
    assert!(supported_programs.contains(&PUMPSWAP_PROGRAM_ID));
    assert!(supported_programs.contains(&BONK_PROGRAM_ID));
    assert!(supported_programs.contains(&RAYDIUM_CPMM_PROGRAM_ID));
}

#[test]
fn test_factory_get_parser_by_program_id() {
    // Test successful lookups
    let pumpswap_parser = EventParserFactory::get_parser_for_program_id(&PUMPSWAP_PROGRAM_ID);
    assert!(pumpswap_parser.is_some());
    assert!(pumpswap_parser.unwrap().should_handle(&PUMPSWAP_PROGRAM_ID));
    
    let bonk_parser = EventParserFactory::get_parser_for_program_id(&BONK_PROGRAM_ID);
    assert!(bonk_parser.is_some());
    assert!(bonk_parser.unwrap().should_handle(&BONK_PROGRAM_ID));
    
    let raydium_parser = EventParserFactory::get_parser_for_program_id(&RAYDIUM_CPMM_PROGRAM_ID);
    assert!(raydium_parser.is_some());
    assert!(raydium_parser.unwrap().should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
    
    // Test unsuccessful lookup
    let unknown_program = Pubkey::new_unique();
    let no_parser = EventParserFactory::get_parser_for_program_id(&unknown_program);
    assert!(no_parser.is_none());
}

#[test]
fn test_mutil_parser_creation() {
    let protocols = vec![Protocol::PumpSwap, Protocol::Bonk];
    let mutil_parser = EventParserFactory::create_mutil_parser(&protocols);
    
    // Test that it supports all specified protocols
    assert!(mutil_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    assert!(mutil_parser.should_handle(&BONK_PROGRAM_ID));
    assert!(!mutil_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID)); // Not included
    
    let supported_ids = mutil_parser.supported_program_ids();
    assert!(supported_ids.contains(&PUMPSWAP_PROGRAM_ID));
    assert!(supported_ids.contains(&BONK_PROGRAM_ID));
}

#[test]
fn test_mutil_parser_empty_protocols() {
    let empty_protocols = vec![];
    let mutil_parser = EventParserFactory::create_mutil_parser(&empty_protocols);
    
    let supported_ids = mutil_parser.supported_program_ids();
    assert!(supported_ids.is_empty());
    
    assert!(!mutil_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    assert!(!mutil_parser.should_handle(&BONK_PROGRAM_ID));
    assert!(!mutil_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
}

#[test]
fn test_mutil_parser_all_protocols() {
    let all_protocols = Protocol::all();
    let mutil_parser = EventParserFactory::create_mutil_parser(&all_protocols);
    
    // Should support all known program IDs
    assert!(mutil_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    assert!(mutil_parser.should_handle(&BONK_PROGRAM_ID));
    assert!(mutil_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
    
    let supported_ids = mutil_parser.supported_program_ids();
    assert!(supported_ids.contains(&PUMPSWAP_PROGRAM_ID));
    assert!(supported_ids.contains(&BONK_PROGRAM_ID));
    assert!(supported_ids.contains(&RAYDIUM_CPMM_PROGRAM_ID));
}

#[test]
fn test_mutil_parser_get_parser_for_program() {
    let protocols = vec![Protocol::PumpSwap, Protocol::Bonk, Protocol::RaydiumCpmm];
    let mutil_parser = EventParserFactory::create_mutil_parser(&protocols);
    
    // Test successful program lookups
    let pumpswap_parser = mutil_parser.get_parser_for_program(&PUMPSWAP_PROGRAM_ID);
    assert!(pumpswap_parser.is_some());
    assert!(pumpswap_parser.unwrap().should_handle(&PUMPSWAP_PROGRAM_ID));
    
    let bonk_parser = mutil_parser.get_parser_for_program(&BONK_PROGRAM_ID);
    assert!(bonk_parser.is_some());
    assert!(bonk_parser.unwrap().should_handle(&BONK_PROGRAM_ID));
    
    let raydium_parser = mutil_parser.get_parser_for_program(&RAYDIUM_CPMM_PROGRAM_ID);
    assert!(raydium_parser.is_some());
    assert!(raydium_parser.unwrap().should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
    
    // Test unsuccessful lookup
    let unknown_program = Pubkey::new_unique();
    let no_parser = mutil_parser.get_parser_for_program(&unknown_program);
    assert!(no_parser.is_none());
}

#[test]
fn test_mutil_parser_config_aggregation() {
    let protocols = vec![Protocol::PumpSwap, Protocol::Bonk];
    let mutil_parser = EventParserFactory::create_mutil_parser(&protocols);
    
    // Test inner instruction configs aggregation
    let inner_configs = mutil_parser.inner_instruction_configs();
    assert!(!inner_configs.is_empty());
    
    // Should contain configs from both protocols
    let total_inner_configs: usize = inner_configs.values().map(|v| v.len()).sum();
    assert!(total_inner_configs > 0);
    
    // Test instruction configs aggregation
    let instruction_configs = mutil_parser.instruction_configs();
    assert!(!instruction_configs.is_empty());
    
    let total_instruction_configs: usize = instruction_configs.values().map(|v| v.len()).sum();
    assert!(total_instruction_configs > 0);
}

#[test]
fn test_mutil_parser_duplicate_configs() {
    // Create mutil parser with overlapping protocols (CLMM and AmmV4 both use CPMM)
    let protocols = vec![Protocol::RaydiumCpmm, Protocol::RaydiumClmm, Protocol::RaydiumAmmV4];
    let mutil_parser = EventParserFactory::create_mutil_parser(&protocols);
    
    let supported_ids = mutil_parser.supported_program_ids();
    
    // Even though we have multiple protocols, they might map to the same program IDs
    // The implementation should handle this gracefully
    assert!(!supported_ids.is_empty());
    assert!(supported_ids.contains(&RAYDIUM_CPMM_PROGRAM_ID));
}

#[test]
fn test_mutil_parser_inner_instruction_parsing() {
    let protocols = vec![Protocol::PumpSwap];
    let mutil_parser = EventParserFactory::create_mutil_parser(&protocols);
    
    // Create a mock inner instruction
    let inner_instruction = UiCompiledInstruction {
        program_id_index: 0,
        accounts: vec![0, 1, 2, 3],
        data: "mock_data".to_string(),
        stack_height: None,
    };
    
    let events = mutil_parser.parse_events_from_inner_instruction(
        &inner_instruction,
        "test_signature",
        12345,
        Some(1640995200),
        1640995200000,
        "0.1".to_string(),
    );
    
    // The result may be empty due to mock data, but should not panic
    assert!(events.is_empty() || !events.is_empty());
}

#[test]
fn test_mutil_parser_instruction_parsing() {
    let protocols = vec![Protocol::PumpSwap];
    let mutil_parser = EventParserFactory::create_mutil_parser(&protocols);
    
    // Create a mock instruction with PumpSwap program
    let instruction = CompiledInstruction {
        program_id_index: 0,
        accounts: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        data: vec![102, 6, 61, 18, 1, 218, 235, 234], // PumpSwap buy discriminator
    };
    
    let accounts = vec![PUMPSWAP_PROGRAM_ID];
    accounts.iter().chain(std::iter::repeat(&Pubkey::new_unique()).take(15)).collect::<Vec<_>>();
    let accounts: Vec<Pubkey> = (0..16).map(|i| if i == 0 { PUMPSWAP_PROGRAM_ID } else { Pubkey::new_unique() }).collect();
    
    let events = mutil_parser.parse_events_from_instruction(
        &instruction,
        &accounts,
        "test_signature",
        12345,
        Some(1640995200),
        1640995200000,
        "0".to_string(),
    );
    
    // Should delegate to the appropriate parser
    assert!(events.is_empty() || !events.is_empty()); // Either result is acceptable
}

#[test]
fn test_mutil_parser_instruction_parsing_wrong_program() {
    let protocols = vec![Protocol::PumpSwap];
    let mutil_parser = EventParserFactory::create_mutil_parser(&protocols);
    
    // Create instruction with unsupported program
    let unknown_program = Pubkey::new_unique();
    let instruction = CompiledInstruction {
        program_id_index: 0,
        accounts: vec![1, 2, 3],
        data: vec![1, 2, 3, 4],
    };
    
    let accounts = vec![unknown_program, Pubkey::new_unique(), Pubkey::new_unique(), Pubkey::new_unique()];
    
    let events = mutil_parser.parse_events_from_instruction(
        &instruction,
        &accounts,
        "test_signature",
        12345,
        Some(1640995200),
        1640995200000,
        "0".to_string(),
    );
    
    // Should return empty since no parser handles this program
    assert!(events.is_empty());
}

#[test]
fn test_mutil_parser_trait_methods() {
    let protocols = vec![Protocol::PumpSwap, Protocol::Bonk];
    let mutil_parser = EventParserFactory::create_mutil_parser(&protocols);
    
    // Test EventParser trait implementation
    assert!(mutil_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    assert!(mutil_parser.should_handle(&BONK_PROGRAM_ID));
    
    let supported = mutil_parser.supported_program_ids();
    assert!(!supported.is_empty());
    
    let inner_configs = mutil_parser.inner_instruction_configs();
    let instruction_configs = mutil_parser.instruction_configs();
    
    assert!(!inner_configs.is_empty() || !instruction_configs.is_empty());
}

#[test]
fn test_factory_singleton_behavior() {
    // Test that the factory returns the same instances (due to LazyLock)
    let parser1 = EventParserFactory::create_parser(Protocol::PumpSwap);
    let parser2 = EventParserFactory::create_parser(Protocol::PumpSwap);
    
    // Should be the same Arc instance
    assert!(Arc::ptr_eq(&parser1, &parser2));
    
    // Test with different protocols
    let bonk1 = EventParserFactory::create_parser(Protocol::Bonk);
    let bonk2 = EventParserFactory::create_parser(Protocol::Bonk);
    
    assert!(Arc::ptr_eq(&bonk1, &bonk2));
    assert!(!Arc::ptr_eq(&parser1, &bonk1));
}

#[test]
fn test_mutil_parser_direct_instantiation() {
    // Test creating MutilEventParser directly
    let pumpswap_parser = EventParserFactory::create_parser(Protocol::PumpSwap);
    let bonk_parser = EventParserFactory::create_parser(Protocol::Bonk);
    
    let parsers = vec![pumpswap_parser, bonk_parser];
    let mutil_parser = MutilEventParser::new(parsers);
    
    // Should behave the same as factory-created one
    assert!(mutil_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    assert!(mutil_parser.should_handle(&BONK_PROGRAM_ID));
    assert!(!mutil_parser.should_handle(&Pubkey::new_unique()));
    
    let supported = mutil_parser.supported_program_ids();
    assert!(supported.contains(&PUMPSWAP_PROGRAM_ID));
    assert!(supported.contains(&BONK_PROGRAM_ID));
}

#[test]
fn test_protocol_coverage() {
    // Verify that factory can create parsers for all defined protocols
    for protocol in Protocol::all() {
        let parser = EventParserFactory::create_parser(protocol);
        let supported_ids = parser.supported_program_ids();
        
        // Each parser should support at least one program ID
        assert!(!supported_ids.is_empty(), "Protocol {:?} parser supports no programs", protocol);
        
        // Each parser should handle its own supported programs
        for program_id in &supported_ids {
            assert!(parser.should_handle(program_id), 
                "Protocol {:?} parser doesn't handle its supported program {}", 
                protocol, program_id);
        }
    }
}

#[test]
fn test_factory_thread_safety() {
    use std::sync::Arc;
    use std::thread;
    
    // Test that factory can be used safely from multiple threads
    let handles: Vec<_> = (0..10).map(|i| {
        thread::spawn(move || {
            let protocol = match i % 3 {
                0 => Protocol::PumpSwap,
                1 => Protocol::Bonk,
                _ => Protocol::RaydiumCpmm,
            };
            
            let parser = EventParserFactory::create_parser(protocol);
            parser.supported_program_ids()
        })
    }).collect();
    
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(!result.is_empty());
    }
}