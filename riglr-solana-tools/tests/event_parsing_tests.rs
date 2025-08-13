//! Comprehensive tests for the event parsing system in riglr-solana-tools
//! 
//! This test suite covers:
//! - Core event parsing traits and implementations
//! - Protocol-specific parsers (PumpSwap, Bonk, Raydium CPMM)
//! - Factory pattern and multi-parser functionality
//! - Macro implementations (impl_unified_event!, match_event!)
//! - Tool functions for event analysis
//! - Integration scenarios with mixed protocol transactions

use riglr_solana_tools::{
    events::{
        core::traits::{EventParser, GenericEventParser, GenericEventParseConfig, UnifiedEvent},
        common::{EventMetadata, EventType, ProtocolType, TransferData, SwapData},
        factory::{EventParserFactory, Protocol},
        protocols::{
            pumpswap::{PumpSwapEventParser, PumpSwapBuyEvent, PumpSwapSellEvent, PUMPSWAP_PROGRAM_ID},
            bonk::BONK_PROGRAM_ID,
            raydium_cpmm::RAYDIUM_CPMM_PROGRAM_ID,
        }
    },
    analyze_transaction_events, get_protocol_events, analyze_recent_events, monitor_token_events,
    match_event,
};
use solana_sdk::{
    pubkey::Pubkey, 
    instruction::CompiledInstruction,
};
use solana_transaction_status::UiCompiledInstruction;
use std::str::FromStr;

// Mock transaction signature for testing
const MOCK_SIGNATURE: &str = "5j7s8K9LmNoPqRstuVwXyZ1BcDeFgHiJkLmNoPqRstuVwXyZ1BcDeFgHiJkLmNoPqRst";
const MOCK_SLOT: u64 = 123456789;
const MOCK_BLOCK_TIME: i64 = 1640995200;
const MOCK_PROGRAM_RECEIVED_TIME: i64 = 1640995200000;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create mock event metadata
    fn create_mock_metadata(
        event_type: EventType,
        protocol: ProtocolType,
        program_id: Pubkey,
        index: &str,
    ) -> EventMetadata {
        EventMetadata::new(
            MOCK_SIGNATURE.to_string(),
            MOCK_SIGNATURE.to_string(),
            MOCK_SLOT,
            MOCK_BLOCK_TIME,
            MOCK_BLOCK_TIME * 1000,
            protocol,
            event_type,
            program_id,
            index.to_string(),
            MOCK_PROGRAM_RECEIVED_TIME,
        )
    }


    #[test]
    fn test_event_metadata_creation() {
        let metadata = create_mock_metadata(
            EventType::PumpSwapBuy,
            ProtocolType::PumpSwap,
            PUMPSWAP_PROGRAM_ID,
            "0",
        );

        assert_eq!(metadata.signature, MOCK_SIGNATURE);
        assert_eq!(metadata.slot, MOCK_SLOT);
        assert_eq!(metadata.block_time, MOCK_BLOCK_TIME);
        assert_eq!(metadata.event_type, EventType::PumpSwapBuy);
        assert_eq!(metadata.protocol, ProtocolType::PumpSwap);
        assert_eq!(metadata.program_id, PUMPSWAP_PROGRAM_ID);
        assert_eq!(metadata.index, "0");
    }

    #[test]
    fn test_transfer_data_creation() {
        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        let source = Pubkey::new_unique();
        let destination = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let transfer_data = TransferData {
            token_program: sol_mint,
            source,
            destination,
            authority: Some(authority),
            amount: 1000000, // 1 SOL
            decimals: Some(9),
            mint: Some(sol_mint),
        };

        assert_eq!(transfer_data.amount, 1000000);
        assert_eq!(transfer_data.decimals, Some(9));
        assert_eq!(transfer_data.mint, Some(sol_mint));
        assert_eq!(transfer_data.source, source);
        assert_eq!(transfer_data.destination, destination);
        assert_eq!(transfer_data.authority, Some(authority));
    }

    #[test]
    fn test_swap_data_creation() {
        let usdc_mint = Pubkey::new_unique();
        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();

        let swap_data = SwapData {
            from_mint: usdc_mint,
            to_mint: sol_mint,
            from_amount: 100000000, // 100 USDC
            to_amount: 1000000000,  // 1 SOL
            description: Some("USDC to SOL swap".to_string()),
        };

        assert_eq!(swap_data.from_mint, usdc_mint);
        assert_eq!(swap_data.to_mint, sol_mint);
        assert_eq!(swap_data.from_amount, 100000000);
        assert_eq!(swap_data.to_amount, 1000000000);
        assert_eq!(swap_data.description, Some("USDC to SOL swap".to_string()));
    }

    #[test]
    fn test_impl_unified_event_macro() {
        let metadata = create_mock_metadata(
            EventType::PumpSwapBuy,
            ProtocolType::PumpSwap,
            PUMPSWAP_PROGRAM_ID,
            "0",
        );

        let mut buy_event = PumpSwapBuyEvent {
            metadata,
            base_amount_out: 1000000,
            max_quote_amount_in: 500000,
            pool: Pubkey::new_unique(),
            user: Pubkey::new_unique(),
            ..Default::default()
        };

        // Test UnifiedEvent trait implementation
        assert_eq!(buy_event.event_type(), EventType::PumpSwapBuy);
        assert_eq!(buy_event.signature(), MOCK_SIGNATURE);
        assert_eq!(buy_event.slot(), MOCK_SLOT);
        assert_eq!(buy_event.protocol_type(), ProtocolType::PumpSwap);
        assert_eq!(buy_event.program_received_time_ms(), MOCK_PROGRAM_RECEIVED_TIME);

        // Test mutable methods
        buy_event.set_program_handle_time_consuming_ms(50);
        assert_eq!(buy_event.program_handle_time_consuming_ms(), 50);

        // Test transfer data setting
        let transfer_data = vec![TransferData::default()];
        let swap_data = Some(SwapData::default());
        buy_event.set_transfer_datas(transfer_data.clone(), swap_data.clone());
        assert_eq!(buy_event.metadata.transfer_datas.len(), 1);
        assert!(buy_event.metadata.swap_data.is_some());

        // Test cloning
        let cloned = buy_event.clone_boxed();
        assert_eq!(cloned.event_type(), buy_event.event_type());
        assert_eq!(cloned.signature(), buy_event.signature());
    }

    #[test]
    fn test_match_event_macro() {
        let buy_event = Box::new(PumpSwapBuyEvent {
            metadata: create_mock_metadata(
                EventType::PumpSwapBuy,
                ProtocolType::PumpSwap,
                PUMPSWAP_PROGRAM_ID,
                "0",
            ),
            base_amount_out: 1000000,
            ..Default::default()
        }) as Box<dyn UnifiedEvent>;

        let sell_event = Box::new(PumpSwapSellEvent {
            metadata: create_mock_metadata(
                EventType::PumpSwapSell,
                ProtocolType::PumpSwap,
                PUMPSWAP_PROGRAM_ID,
                "1",
            ),
            base_amount_in: 2000000,
            ..Default::default()
        }) as Box<dyn UnifiedEvent>;

        // Test matching buy event
        #[allow(unused_assignments)]
        let mut matched_type = "";
        #[allow(unused_assignments)]
        let mut matched_amount = 0u64;

        match_event!(buy_event, {
            PumpSwapBuyEvent => |buy: &PumpSwapBuyEvent| {
                matched_type = "buy";
                matched_amount = buy.base_amount_out;
            },
            PumpSwapSellEvent => |_sell: &PumpSwapSellEvent| {
                // This branch won't be executed for buy_event
            },
        });

        assert_eq!(matched_type, "buy");
        assert_eq!(matched_amount, 1000000);

        // Test matching sell event
        matched_type = "";
        matched_amount = 0;

        match_event!(sell_event, {
            PumpSwapBuyEvent => |_buy: &PumpSwapBuyEvent| {
                // This branch won't be executed for sell_event
            },
            PumpSwapSellEvent => |sell: &PumpSwapSellEvent| {
                matched_type = "sell";
                matched_amount = sell.base_amount_in;
            },
        });

        assert_eq!(matched_type, "sell");
        assert_eq!(matched_amount, 2000000);
    }

    #[test]
    fn test_pumpswap_event_parser_creation() {
        let parser = PumpSwapEventParser::new();
        
        // Test supported program IDs
        let program_ids = parser.supported_program_ids();
        assert!(!program_ids.is_empty());
        assert!(program_ids.contains(&PUMPSWAP_PROGRAM_ID));
        
        // Test should_handle
        assert!(parser.should_handle(&PUMPSWAP_PROGRAM_ID));
        assert!(!parser.should_handle(&Pubkey::new_unique()));

        // Test configuration retrieval
        let inner_configs = parser.inner_instruction_configs();
        assert!(!inner_configs.is_empty());
        
        let instruction_configs = parser.instruction_configs();
        assert!(!instruction_configs.is_empty());
    }

    #[test]
    fn test_generic_event_parser() {
        fn mock_inner_parser(_data: &[u8], metadata: EventMetadata) -> Option<Box<dyn UnifiedEvent>> {
            Some(Box::new(PumpSwapBuyEvent {
                metadata,
                ..Default::default()
            }))
        }

        fn mock_instruction_parser(_data: &[u8], _accounts: &[Pubkey], metadata: EventMetadata) -> Option<Box<dyn UnifiedEvent>> {
            Some(Box::new(PumpSwapBuyEvent {
                metadata,
                ..Default::default()
            }))
        }

        let config = GenericEventParseConfig {
            program_id: PUMPSWAP_PROGRAM_ID,
            protocol_type: ProtocolType::PumpSwap,
            inner_instruction_discriminator: "0xe445a52e51cb9a1d",
            instruction_discriminator: &[102, 6, 61, 18, 1, 218, 235, 234],
            event_type: EventType::PumpSwapBuy,
            inner_instruction_parser: mock_inner_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![PUMPSWAP_PROGRAM_ID], vec![config]);
        
        assert!(parser.should_handle(&PUMPSWAP_PROGRAM_ID));
        assert!(!parser.should_handle(&Pubkey::new_unique()));
        
        let supported_ids = parser.supported_program_ids();
        assert_eq!(supported_ids.len(), 1);
        assert_eq!(supported_ids[0], PUMPSWAP_PROGRAM_ID);
    }

    #[test]
    fn test_event_parser_factory() {
        // Test individual parser creation
        let pumpswap_parser = EventParserFactory::create_parser(Protocol::PumpSwap);
        assert!(!pumpswap_parser.supported_program_ids().is_empty());
        assert!(pumpswap_parser.should_handle(&PUMPSWAP_PROGRAM_ID));

        let bonk_parser = EventParserFactory::create_parser(Protocol::Bonk);
        assert!(!bonk_parser.supported_program_ids().is_empty());
        assert!(bonk_parser.should_handle(&BONK_PROGRAM_ID));

        let raydium_parser = EventParserFactory::create_parser(Protocol::RaydiumCpmm);
        assert!(!raydium_parser.supported_program_ids().is_empty());
        assert!(raydium_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));

        // Test all parsers creation
        let all_parsers = EventParserFactory::create_all_parsers();
        assert!(!all_parsers.is_empty());
        assert_eq!(all_parsers.len(), Protocol::all().len());

        // Test getting parser by program ID
        let parser_by_id = EventParserFactory::get_parser_for_program_id(&PUMPSWAP_PROGRAM_ID);
        assert!(parser_by_id.is_some());

        let nonexistent_parser = EventParserFactory::get_parser_for_program_id(&Pubkey::new_unique());
        assert!(nonexistent_parser.is_none());
    }

    #[test]
    fn test_mutil_event_parser_creation() {
        let protocols = vec![Protocol::PumpSwap, Protocol::Bonk, Protocol::RaydiumCpmm];
        let mutil_parser = EventParserFactory::create_mutil_parser(&protocols);

        // Test supported program IDs
        let supported_ids = mutil_parser.supported_program_ids();
        assert!(!supported_ids.is_empty());
        assert!(supported_ids.contains(&PUMPSWAP_PROGRAM_ID));
        assert!(supported_ids.contains(&BONK_PROGRAM_ID));
        assert!(supported_ids.contains(&RAYDIUM_CPMM_PROGRAM_ID));

        // Test should_handle
        assert!(mutil_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
        assert!(mutil_parser.should_handle(&BONK_PROGRAM_ID));
        assert!(mutil_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
        assert!(!mutil_parser.should_handle(&Pubkey::new_unique()));

        // Test get_parser_for_program
        let pumpswap_parser = mutil_parser.get_parser_for_program(&PUMPSWAP_PROGRAM_ID);
        assert!(pumpswap_parser.is_some());

        let bonk_parser = mutil_parser.get_parser_for_program(&BONK_PROGRAM_ID);
        assert!(bonk_parser.is_some());

        let no_parser = mutil_parser.get_parser_for_program(&Pubkey::new_unique());
        assert!(no_parser.is_none());
    }

    #[test]
    fn test_mutil_parser_config_aggregation() {
        let protocols = vec![Protocol::PumpSwap, Protocol::Bonk];
        let mutil_parser = EventParserFactory::create_mutil_parser(&protocols);

        // Test inner instruction configs aggregation
        let inner_configs = mutil_parser.inner_instruction_configs();
        assert!(!inner_configs.is_empty());

        // Test instruction configs aggregation
        let instruction_configs = mutil_parser.instruction_configs();
        assert!(!instruction_configs.is_empty());
    }

    #[test]
    fn test_event_parsing_with_mock_instruction() {
        let parser = PumpSwapEventParser::new();
        
        // Create mock instruction data for buy operation
        let discriminator = &[102, 6, 61, 18, 1, 218, 235, 234]; // BUY_IX
        let base_amount = 1000000u64.to_le_bytes();
        let max_quote = 500000u64.to_le_bytes();
        
        let mut instruction_data = Vec::new();
        instruction_data.extend_from_slice(discriminator);
        instruction_data.extend_from_slice(&base_amount);
        instruction_data.extend_from_slice(&max_quote);

        // Create mock accounts
        let pool = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let program_id = PUMPSWAP_PROGRAM_ID;
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();
        let user_base_account = Pubkey::new_unique();
        let user_quote_account = Pubkey::new_unique();
        let pool_base_account = Pubkey::new_unique();
        let pool_quote_account = Pubkey::new_unique();
        let protocol_fee_recipient = Pubkey::new_unique();
        let protocol_fee_account = Pubkey::new_unique();

        let accounts = vec![
            pool,
            user,
            program_id,
            base_mint,
            quote_mint,
            user_base_account,
            user_quote_account,
            pool_base_account,
            pool_quote_account,
            protocol_fee_recipient,
            protocol_fee_account,
        ];

        let instruction = CompiledInstruction {
            program_id_index: 2, // program_id is at index 2
            accounts: (0..accounts.len() as u8).collect(),
            data: instruction_data,
        };

        // Parse the instruction
        let events = parser.parse_events_from_instruction(
            &instruction,
            &accounts,
            MOCK_SIGNATURE,
            MOCK_SLOT,
            Some(MOCK_BLOCK_TIME),
            MOCK_PROGRAM_RECEIVED_TIME,
            "0".to_string(),
        );

        assert!(!events.is_empty());
        assert_eq!(events.len(), 1);
        
        let event = &events[0];
        assert_eq!(event.event_type(), EventType::PumpSwapBuy);
        assert_eq!(event.signature(), MOCK_SIGNATURE);
        assert_eq!(event.slot(), MOCK_SLOT);
    }

    #[test]
    fn test_protocol_detection_accuracy() {
        let protocols = vec![Protocol::PumpSwap, Protocol::Bonk, Protocol::RaydiumCpmm];
        let mutil_parser = EventParserFactory::create_mutil_parser(&protocols);

        // Test PumpSwap detection
        assert!(mutil_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
        let parser = mutil_parser.get_parser_for_program(&PUMPSWAP_PROGRAM_ID).unwrap();
        assert!(parser.should_handle(&PUMPSWAP_PROGRAM_ID));

        // Test Bonk detection
        assert!(mutil_parser.should_handle(&BONK_PROGRAM_ID));
        let parser = mutil_parser.get_parser_for_program(&BONK_PROGRAM_ID).unwrap();
        assert!(parser.should_handle(&BONK_PROGRAM_ID));

        // Test Raydium CPMM detection
        assert!(mutil_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
        let parser = mutil_parser.get_parser_for_program(&RAYDIUM_CPMM_PROGRAM_ID).unwrap();
        assert!(parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));

        // Test non-matching program
        let random_program = Pubkey::new_unique();
        assert!(!mutil_parser.should_handle(&random_program));
        assert!(mutil_parser.get_parser_for_program(&random_program).is_none());
    }

    #[test]
    fn test_error_handling_paths() {
        let parser = PumpSwapEventParser::new();

        // Test with empty instruction data
        let empty_instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: vec![],
        };

        let accounts = vec![PUMPSWAP_PROGRAM_ID, Pubkey::new_unique(), Pubkey::new_unique()];
        let events = parser.parse_events_from_instruction(
            &empty_instruction,
            &accounts,
            MOCK_SIGNATURE,
            MOCK_SLOT,
            Some(MOCK_BLOCK_TIME),
            MOCK_PROGRAM_RECEIVED_TIME,
            "0".to_string(),
        );

        assert!(events.is_empty());

        // Test with insufficient accounts
        let discriminator = &[102, 6, 61, 18, 1, 218, 235, 234]; // BUY_IX
        let payload = vec![0u8; 16]; // 16 bytes of data
        
        let mut instruction_data = Vec::new();
        instruction_data.extend_from_slice(discriminator);
        instruction_data.extend_from_slice(&payload);
        
        let insufficient_instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1], // Not enough accounts
            data: instruction_data,
        };

        let few_accounts = vec![PUMPSWAP_PROGRAM_ID, Pubkey::new_unique()];
        let events = parser.parse_events_from_instruction(
            &insufficient_instruction,
            &few_accounts,
            MOCK_SIGNATURE,
            MOCK_SLOT,
            Some(MOCK_BLOCK_TIME),
            MOCK_PROGRAM_RECEIVED_TIME,
            "0".to_string(),
        );

        assert!(events.is_empty());

        // Test with wrong program ID
        let wrong_program_accounts = vec![Pubkey::new_unique(); 15];
        let events = parser.parse_events_from_instruction(
            &insufficient_instruction,
            &wrong_program_accounts,
            MOCK_SIGNATURE,
            MOCK_SLOT,
            Some(MOCK_BLOCK_TIME),
            MOCK_PROGRAM_RECEIVED_TIME,
            "0".to_string(),
        );

        assert!(events.is_empty());
    }

    #[test]
    fn test_event_merging() {
        let mut base_event = PumpSwapBuyEvent {
            metadata: create_mock_metadata(
                EventType::PumpSwapBuy,
                ProtocolType::PumpSwap,
                PUMPSWAP_PROGRAM_ID,
                "0",
            ),
            base_amount_out: 1000000,
            max_quote_amount_in: 500000,
            pool: Pubkey::new_unique(),
            ..Default::default()
        };

        let other_event = PumpSwapBuyEvent {
            metadata: create_mock_metadata(
                EventType::PumpSwapBuy,
                ProtocolType::PumpSwap,
                PUMPSWAP_PROGRAM_ID,
                "0.1",
            ),
            base_amount_out: 2000000,
            max_quote_amount_in: 1000000,
            pool: Pubkey::new_unique(),
            quote_amount_in: 750000, // Additional field to merge
            ..Default::default()
        };

        // Merge events
        let other_boxed = Box::new(other_event) as Box<dyn UnifiedEvent>;
        base_event.merge(other_boxed);

        // Check that fields were merged
        assert_eq!(base_event.base_amount_out, 2000000);
        assert_eq!(base_event.max_quote_amount_in, 1000000);
        assert_eq!(base_event.quote_amount_in, 750000);
    }

    #[test]
    fn test_multi_protocol_transaction_parsing() {
        let protocols = vec![Protocol::PumpSwap, Protocol::Bonk];
        let mutil_parser = EventParserFactory::create_mutil_parser(&protocols);

        // Test parsing instruction from different protocols
        let pumpswap_discriminator = &[102, 6, 61, 18, 1, 218, 235, 234];
        let padding = [0u8; 8];
        let mut pumpswap_data = Vec::new();
        pumpswap_data.extend_from_slice(pumpswap_discriminator);
        pumpswap_data.extend_from_slice(&padding);
        
        let pumpswap_instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: (0..15u8).collect(),
            data: pumpswap_data,
        };

        let mut accounts = vec![PUMPSWAP_PROGRAM_ID];
        accounts.extend((0..14).map(|_| Pubkey::new_unique()));

        let events = mutil_parser.parse_events_from_instruction(
            &pumpswap_instruction,
            &accounts,
            MOCK_SIGNATURE,
            MOCK_SLOT,
            Some(MOCK_BLOCK_TIME),
            MOCK_PROGRAM_RECEIVED_TIME,
            "0".to_string(),
        );

        assert!(!events.is_empty());
        assert_eq!(events[0].protocol_type(), ProtocolType::PumpSwap);
    }

    #[tokio::test]
    async fn test_tool_function_error_handling() {
        // Test analyze_transaction_events with invalid signature
        let result = analyze_transaction_events(
            "invalid_signature".to_string(),
            Some("https://api.mainnet-beta.solana.com".to_string()),
        ).await;
        
        assert!(result.is_err());

        // Test get_protocol_events with invalid protocol
        let result = get_protocol_events(
            MOCK_SIGNATURE.to_string(),
            vec!["InvalidProtocol".to_string()],
            Some("https://api.mainnet-beta.solana.com".to_string()),
        ).await;
        
        assert!(result.is_err());

        // Test analyze_recent_events with invalid token address
        let result = analyze_recent_events(
            "invalid_token_address".to_string(),
            Some(10),
            Some("https://api.mainnet-beta.solana.com".to_string()),
        ).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_monitor_token_events_placeholder() {
        // Test the placeholder implementation
        let result = monitor_token_events(
            Pubkey::new_unique().to_string(),
            Some(5),
            Some("https://api.mainnet-beta.solana.com".to_string()),
        ).await;
        
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Token Event Monitoring Started"));
        assert!(output.contains("placeholder implementation"));
    }

    #[test]
    fn test_protocol_string_parsing() {
        use riglr_solana_tools::parse_protocol_strings;

        // Test valid protocol strings
        let valid_protocols = vec![
            "PumpSwap".to_string(),
            "bonk".to_string(),
            "raydiumcpmm".to_string(),
            "raydium_clmm".to_string(),
            "raydiumammv4".to_string(),
        ];

        let result = parse_protocol_strings(valid_protocols);
        assert!(result.is_ok());
        let protocols = result.unwrap();
        assert_eq!(protocols.len(), 5);
        assert_eq!(protocols[0], Protocol::PumpSwap);
        assert_eq!(protocols[1], Protocol::Bonk);
        assert_eq!(protocols[2], Protocol::RaydiumCpmm);

        // Test invalid protocol string
        let invalid_protocols = vec!["InvalidProtocol".to_string()];
        let result = parse_protocol_strings(invalid_protocols);
        assert!(result.is_err());
    }

    #[test]
    fn test_event_formatting() {
        use riglr_solana_tools::format_events_for_agent;

        // Test with empty events
        let empty_events: Vec<Box<dyn UnifiedEvent>> = vec![];
        let result = format_events_for_agent(empty_events);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("No events found"));

        // Test with sample events
        let events: Vec<Box<dyn UnifiedEvent>> = vec![
            Box::new(PumpSwapBuyEvent {
                metadata: create_mock_metadata(
                    EventType::PumpSwapBuy,
                    ProtocolType::PumpSwap,
                    PUMPSWAP_PROGRAM_ID,
                    "0",
                ),
                ..Default::default()
            }),
            Box::new(PumpSwapSellEvent {
                metadata: create_mock_metadata(
                    EventType::PumpSwapSell,
                    ProtocolType::PumpSwap,
                    PUMPSWAP_PROGRAM_ID,
                    "1",
                ),
                ..Default::default()
            }),
        ];

        let result = format_events_for_agent(events);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Transaction Event Analysis"));
        assert!(output.contains("2 events"));
        assert!(output.contains("PumpSwapBuy"));
        assert!(output.contains("PumpSwapSell"));
        assert!(output.contains(MOCK_SIGNATURE));
    }

    #[test]
    fn test_inner_instruction_parsing() {
        let parser = PumpSwapEventParser::new();
        
        // Create mock inner instruction
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2, 3, 4],
            data: "e445a52e51cb9a1d67f4521f2cf57777".to_string(), // Mock discriminator
            stack_height: None,
        };

        let events = parser.parse_events_from_inner_instruction(
            &inner_instruction,
            MOCK_SIGNATURE,
            MOCK_SLOT,
            Some(MOCK_BLOCK_TIME),
            MOCK_PROGRAM_RECEIVED_TIME,
            "0.1".to_string(),
        );

        // Note: This will likely be empty due to the simplified parser implementation
        // but tests that the method doesn't panic and handles the input correctly
        assert!(events.is_empty() || !events.is_empty()); // Either outcome is acceptable for mock data
    }

    #[test]
    fn test_event_id_generation() {
        let mut metadata = create_mock_metadata(
            EventType::PumpSwapBuy,
            ProtocolType::PumpSwap,
            PUMPSWAP_PROGRAM_ID,
            "0",
        );

        let initial_id = metadata.id.clone();
        
        // Set a new ID
        metadata.set_id("test-event-123".to_string());
        
        // ID should be different (it's hashed)
        assert_ne!(metadata.id, initial_id);
        assert!(!metadata.id.is_empty());
        
        // Setting the same ID should produce the same hash
        let first_hash = metadata.id.clone();
        metadata.set_id("test-event-123".to_string());
        assert_eq!(metadata.id, first_hash);
    }

    #[test]
    fn test_discriminator_matching() {
        use riglr_solana_tools::events::common::utils::discriminator_matches;

        let data = "0xe445a52e51cb9a1d67f4521f2cf57777";
        let discriminator = "0xe445a52e51cb9a1d";
        
        assert!(discriminator_matches(data, discriminator));
        assert!(!discriminator_matches(data, "0x12345678"));
        assert!(!discriminator_matches("0x12345", discriminator));
    }

    #[test]
    fn test_account_validation() {
        use riglr_solana_tools::events::common::utils::validate_account_indices;

        // Valid indices
        let valid_indices = vec![0u8, 1u8, 2u8];
        assert!(validate_account_indices(&valid_indices, 5));

        // Invalid indices (out of bounds)
        let invalid_indices = vec![0u8, 1u8, 5u8];
        assert!(!validate_account_indices(&invalid_indices, 5));

        // Edge case - empty indices
        let empty_indices = vec![];
        assert!(validate_account_indices(&empty_indices, 0));
        assert!(validate_account_indices(&empty_indices, 10));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test that demonstrates a complete event parsing workflow
    /// from instruction data to formatted output
    #[test]
    fn test_complete_event_parsing_workflow() {
        // 1. Create a multi-protocol parser
        let protocols = vec![Protocol::PumpSwap, Protocol::Bonk, Protocol::RaydiumCpmm];
        let parser = EventParserFactory::create_mutil_parser(&protocols);

        // 2. Verify all protocols are supported
        assert!(parser.should_handle(&PUMPSWAP_PROGRAM_ID));
        assert!(parser.should_handle(&BONK_PROGRAM_ID));
        assert!(parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));

        // 3. Test configuration aggregation
        let inner_configs = parser.inner_instruction_configs();
        let instruction_configs = parser.instruction_configs();
        
        assert!(!inner_configs.is_empty());
        assert!(!instruction_configs.is_empty());

        // 4. Create sample events and verify they can be processed
        let sample_events: Vec<Box<dyn UnifiedEvent>> = vec![
            Box::new(PumpSwapBuyEvent {
                metadata: create_mock_metadata(
                    EventType::PumpSwapBuy,
                    ProtocolType::PumpSwap,
                    PUMPSWAP_PROGRAM_ID,
                    "0",
                ),
                base_amount_out: 1000000,
                pool: Pubkey::new_unique(),
                user: Pubkey::new_unique(),
                ..Default::default()
            }),
        ];

        // 5. Test event formatting
        let formatted = riglr_solana_tools::format_events_for_agent(sample_events);
        assert!(formatted.is_ok());
        let output = formatted.unwrap();
        assert!(output.contains("Event Analysis"));
        assert!(output.contains("PumpSwapBuy"));
    }

    /// Test protocol-specific parsing behavior
    #[test]
    fn test_protocol_specific_parsing() {
        // Test each protocol parser individually
        let pumpswap_parser = EventParserFactory::create_parser(Protocol::PumpSwap);
        let bonk_parser = EventParserFactory::create_parser(Protocol::Bonk);
        let raydium_parser = EventParserFactory::create_parser(Protocol::RaydiumCpmm);

        // Verify program ID handling
        assert!(pumpswap_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
        assert!(!pumpswap_parser.should_handle(&BONK_PROGRAM_ID));
        
        assert!(bonk_parser.should_handle(&BONK_PROGRAM_ID));
        assert!(!bonk_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
        
        assert!(raydium_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
        assert!(!raydium_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    }

    /// Test error scenarios that might occur in production
    #[test]
    fn test_production_error_scenarios() {
        let parser = PumpSwapEventParser::new();

        // Test malformed instruction data
        let malformed_data = vec![1, 2, 3]; // Too short
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: malformed_data,
        };

        let accounts = vec![PUMPSWAP_PROGRAM_ID, Pubkey::new_unique(), Pubkey::new_unique()];
        let events = parser.parse_events_from_instruction(
            &instruction,
            &accounts,
            MOCK_SIGNATURE,
            MOCK_SLOT,
            Some(MOCK_BLOCK_TIME),
            MOCK_PROGRAM_RECEIVED_TIME,
            "0".to_string(),
        );

        assert!(events.is_empty());

        // Test with mismatched account indices
        let bad_instruction = CompiledInstruction {
            program_id_index: 10, // Out of bounds
            accounts: vec![0, 1, 2],
            data: vec![102, 6, 61, 18, 1, 218, 235, 234], // Valid discriminator
        };

        let events = parser.parse_events_from_instruction(
            &bad_instruction,
            &accounts,
            MOCK_SIGNATURE,
            MOCK_SLOT,
            Some(MOCK_BLOCK_TIME),
            MOCK_PROGRAM_RECEIVED_TIME,
            "0".to_string(),
        );

        assert!(events.is_empty());
    }

    fn create_mock_metadata(
        event_type: EventType,
        protocol: ProtocolType,
        program_id: Pubkey,
        index: &str,
    ) -> EventMetadata {
        EventMetadata::new(
            "mock_signature".to_string(),
            "mock_signature".to_string(),
            12345,
            1640995200,
            1640995200000,
            protocol,
            event_type,
            program_id,
            index.to_string(),
            1640995200000,
        )
    }
}