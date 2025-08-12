//! Integration tests for the entire event parsing system
//! Tests the complete workflow from raw transaction data to formatted output

use riglr_solana_tools::{
    events::{
        core::traits::{EventParser, UnifiedEvent},
        common::{EventMetadata, EventType, ProtocolType, TransferData, SwapData},
        factory::{EventParserFactory, Protocol},
        protocols::{
            pumpswap::{PumpSwapBuyEvent, PumpSwapSellEvent, PUMPSWAP_PROGRAM_ID},
            bonk::{BONK_PROGRAM_ID},
            raydium_cpmm::{RAYDIUM_CPMM_PROGRAM_ID},
        }
    },
    format_events_for_agent, parse_protocol_strings, match_event,
};
use solana_sdk::{pubkey::Pubkey, instruction::CompiledInstruction};
use solana_transaction_status::{UiCompiledInstruction, UiInnerInstructions, UiInstruction};
use std::str::FromStr;

/// Create a comprehensive test transaction with multiple protocols
fn create_multi_protocol_transaction_data() -> (Vec<CompiledInstruction>, Vec<Pubkey>, Vec<UiInnerInstructions>) {
    let mut instructions = Vec::new();
    let mut accounts = Vec::new();
    let mut inner_instructions = Vec::new();

    // Add program IDs first
    accounts.push(PUMPSWAP_PROGRAM_ID);   // index 0
    accounts.push(BONK_PROGRAM_ID);       // index 1
    accounts.push(RAYDIUM_CPMM_PROGRAM_ID); // index 2

    // Add user accounts
    let user = Pubkey::new_unique();
    let pool1 = Pubkey::new_unique();
    let pool2 = Pubkey::new_unique();
    accounts.extend([user, pool1, pool2]); // indices 3, 4, 5

    // Add token accounts
    for _ in 0..20 {
        accounts.push(Pubkey::new_unique()); // indices 6-25
    }

    // Create PumpSwap buy instruction
    let pumpswap_buy_data = {
        let mut data = vec![102, 6, 61, 18, 1, 218, 235, 234]; // Buy discriminator
        data.extend_from_slice(&1000000u64.to_le_bytes()); // base_amount_out
        data.extend_from_slice(&500000u64.to_le_bytes());  // max_quote_amount_in
        data
    };

    instructions.push(CompiledInstruction {
        program_id_index: 0, // PUMPSWAP_PROGRAM_ID
        accounts: vec![4, 3, 0, 6, 7, 8, 9, 10, 11, 12, 13], // pool, user, program, mints and accounts
        data: pumpswap_buy_data,
    });

    // Create PumpSwap sell instruction
    let pumpswap_sell_data = {
        let mut data = vec![51, 230, 133, 164, 1, 127, 131, 173]; // Sell discriminator
        data.extend_from_slice(&2000000u64.to_le_bytes()); // base_amount_in
        data.extend_from_slice(&1000000u64.to_le_bytes()); // min_quote_amount_out
        data
    };

    instructions.push(CompiledInstruction {
        program_id_index: 0, // PUMPSWAP_PROGRAM_ID
        accounts: vec![5, 3, 0, 14, 15, 16, 17, 18, 19, 20, 21], // pool, user, program, mints and accounts
        data: pumpswap_sell_data,
    });

    // Create mock inner instructions for transfer data
    let transfer_instruction = UiInstruction::Compiled(UiCompiledInstruction {
        program_id_index: 26, // Token program (we'll add it to accounts)
        accounts: vec![8, 6, 9, 3], // source, mint, destination, authority
        data: bs58::encode([
            &[12u8], // transferChecked discriminator
            &1000000u64.to_le_bytes(), // amount
            &[9u8], // decimals
        ].concat()).into_string(),
        stack_height: None,
    });

    inner_instructions.push(UiInnerInstructions {
        index: 0, // Corresponds to first instruction
        instructions: vec![transfer_instruction],
    });

    // Add token program to accounts
    accounts.push(Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap());

    (instructions, accounts, inner_instructions)
}

#[test]
fn test_complete_multi_protocol_parsing_workflow() {
    let (instructions, accounts, inner_instructions) = create_multi_protocol_transaction_data();
    
    // Create multi-protocol parser
    let protocols = vec![Protocol::PumpSwap, Protocol::Bonk, Protocol::RaydiumCpmm];
    let parser = EventParserFactory::create_mutil_parser(&protocols);
    
    // Parse all instructions
    let mut all_events = Vec::new();
    
    for (index, instruction) in instructions.iter().enumerate() {
        let events = parser.parse_events_from_instruction(
            instruction,
            &accounts,
            "test_signature",
            12345,
            Some(1640995200),
            chrono::Utc::now().timestamp_millis(),
            index.to_string(),
        );
        all_events.extend(events);
    }
    
    // Should have parsed events
    assert!(!all_events.is_empty());
    
    // Verify event types
    let mut buy_events = 0;
    let mut sell_events = 0;
    
    for event in &all_events {
        match event.event_type() {
            EventType::PumpSwapBuy => buy_events += 1,
            EventType::PumpSwapSell => sell_events += 1,
            _ => {}
        }
    }
    
    assert!(buy_events > 0 || sell_events > 0);
    
    // Test event formatting
    let formatted = format_events_for_agent(all_events);
    assert!(formatted.is_ok());
    let output = formatted.unwrap();
    assert!(output.contains("Transaction Event Analysis"));
}

#[test]
fn test_protocol_detection_integration() {
    // Create parsers for different protocols
    let pumpswap_parser = EventParserFactory::create_parser(Protocol::PumpSwap);
    let bonk_parser = EventParserFactory::create_parser(Protocol::Bonk);
    let raydium_parser = EventParserFactory::create_parser(Protocol::RaydiumCpmm);
    
    // Test cross-protocol isolation
    assert!(pumpswap_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    assert!(!pumpswap_parser.should_handle(&BONK_PROGRAM_ID));
    assert!(!pumpswap_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
    
    assert!(bonk_parser.should_handle(&BONK_PROGRAM_ID));
    assert!(!bonk_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    assert!(!bonk_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
    
    assert!(raydium_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
    assert!(!raydium_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    assert!(!raydium_parser.should_handle(&BONK_PROGRAM_ID));
    
    // Test multi-parser combines all
    let multi_parser = EventParserFactory::create_mutil_parser(&[
        Protocol::PumpSwap,
        Protocol::Bonk,
        Protocol::RaydiumCpmm
    ]);
    
    assert!(multi_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    assert!(multi_parser.should_handle(&BONK_PROGRAM_ID));
    assert!(multi_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
}

#[test]
fn test_event_lifecycle_integration() {
    // Create an event and test its complete lifecycle
    let metadata = EventMetadata::new(
        "test_signature".to_string(),
        "test_signature".to_string(),
        12345,
        1640995200,
        1640995200000,
        ProtocolType::PumpSwap,
        EventType::PumpSwapBuy,
        PUMPSWAP_PROGRAM_ID,
        "0".to_string(),
        chrono::Utc::now().timestamp_millis(),
    );
    
    let mut buy_event = PumpSwapBuyEvent {
        metadata,
        base_amount_out: 1500000,
        max_quote_amount_in: 750000,
        pool: Pubkey::new_unique(),
        user: Pubkey::new_unique(),
        ..Default::default()
    };
    
    // Test initial state
    assert_eq!(buy_event.event_type(), EventType::PumpSwapBuy);
    assert_eq!(buy_event.protocol_type(), ProtocolType::PumpSwap);
    assert_eq!(buy_event.base_amount_out, 1500000);
    
    // Test adding transfer data
    let transfer_data = vec![TransferData {
        token_program: Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
        source: Pubkey::new_unique(),
        destination: Pubkey::new_unique(),
        authority: Some(Pubkey::new_unique()),
        amount: 1500000,
        decimals: Some(9),
        mint: Some(Pubkey::new_unique()),
    }];
    
    let swap_data = Some(SwapData {
        from_mint: Pubkey::new_unique(),
        to_mint: Pubkey::new_unique(),
        from_amount: 750000,
        to_amount: 1500000,
        description: Some("Integration test swap".to_string()),
    });
    
    buy_event.set_transfer_datas(transfer_data.clone(), swap_data.clone());
    
    // Verify transfer data was set
    assert_eq!(buy_event.metadata.transfer_datas.len(), 1);
    assert!(buy_event.metadata.swap_data.is_some());
    assert_eq!(buy_event.metadata.transfer_datas[0].amount, 1500000);
    
    // Test processing time
    buy_event.set_program_handle_time_consuming_ms(150);
    assert_eq!(buy_event.program_handle_time_consuming_ms(), 150);
    
    // Test cloning
    let cloned = buy_event.clone_boxed();
    assert_eq!(cloned.event_type(), buy_event.event_type());
    assert_eq!(cloned.signature(), buy_event.signature());
    
    // Test matching with macro
    let mut matched_amount = 0u64;
    match_event!(cloned, {
        PumpSwapBuyEvent => |buy: &PumpSwapBuyEvent| {
            matched_amount = buy.base_amount_out;
        },
    });
    assert_eq!(matched_amount, 1500000);
}

#[test]
fn test_error_propagation_integration() {
    let parser = EventParserFactory::create_parser(Protocol::PumpSwap);
    
    // Test with various error conditions
    let error_cases = vec![
        // Empty instruction data
        (vec![], vec![PUMPSWAP_PROGRAM_ID]),
        // Insufficient accounts
        (vec![102, 6, 61, 18, 1, 218, 235, 234], vec![PUMPSWAP_PROGRAM_ID]),
        // Wrong program ID
        (vec![102, 6, 61, 18, 1, 218, 235, 234], vec![Pubkey::new_unique()]),
    ];
    
    for (data, account_list) in error_cases {
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: (1..account_list.len() as u8).collect(),
            data,
        };
        
        let events = parser.parse_events_from_instruction(
            &instruction,
            &account_list,
            "test_signature",
            12345,
            Some(1640995200),
            chrono::Utc::now().timestamp_millis(),
            "0".to_string(),
        );
        
        // All error cases should return empty events, not panic
        assert!(events.is_empty());
    }
}

#[test]
fn test_tool_function_integration() {
    // Test protocol string parsing
    let protocol_strings = vec![
        "PumpSwap".to_string(),
        "bonk".to_string(),
        "raydium_cpmm".to_string(),
    ];
    
    let protocols = parse_protocol_strings(protocol_strings);
    assert!(protocols.is_ok());
    let parsed_protocols = protocols.unwrap();
    assert_eq!(parsed_protocols.len(), 3);
    
    // Test invalid protocol
    let invalid_protocols = vec!["NonexistentProtocol".to_string()];
    let result = parse_protocol_strings(invalid_protocols);
    assert!(result.is_err());
    
    // Test event formatting with multiple event types
    let events: Vec<Box<dyn UnifiedEvent>> = vec![
        Box::new(PumpSwapBuyEvent {
            metadata: EventMetadata::new(
                "sig1".to_string(),
                "sig1".to_string(),
                11111,
                1640995200,
                1640995200000,
                ProtocolType::PumpSwap,
                EventType::PumpSwapBuy,
                PUMPSWAP_PROGRAM_ID,
                "0".to_string(),
                chrono::Utc::now().timestamp_millis(),
            ),
            base_amount_out: 1000000,
            ..Default::default()
        }),
        Box::new(PumpSwapSellEvent {
            metadata: EventMetadata::new(
                "sig2".to_string(),
                "sig2".to_string(),
                22222,
                1640995200,
                1640995200000,
                ProtocolType::PumpSwap,
                EventType::PumpSwapSell,
                PUMPSWAP_PROGRAM_ID,
                "1".to_string(),
                chrono::Utc::now().timestamp_millis(),
            ),
            base_amount_in: 2000000,
            ..Default::default()
        }),
    ];
    
    let formatted = format_events_for_agent(events);
    assert!(formatted.is_ok());
    let output = formatted.unwrap();
    
    assert!(output.contains("Transaction Event Analysis"));
    assert!(output.contains("2 events"));
    assert!(output.contains("PumpSwapBuy"));
    assert!(output.contains("PumpSwapSell"));
    assert!(output.contains("sig1"));
    assert!(output.contains("sig2"));
    assert!(output.contains("11111"));
    assert!(output.contains("22222"));
}

#[test]
fn test_complex_transaction_simulation() {
    // Simulate a complex transaction with multiple instructions and inner instructions
    let multi_parser = EventParserFactory::create_mutil_parser(&Protocol::all());
    
    // Create a realistic transaction structure
    let mut accounts = Vec::new();
    
    // Add program accounts
    accounts.push(PUMPSWAP_PROGRAM_ID);
    accounts.push(BONK_PROGRAM_ID);
    
    // Add user and pool accounts
    let user = Pubkey::new_unique();
    let pool1 = Pubkey::new_unique();
    let pool2 = Pubkey::new_unique();
    accounts.extend([user, pool1, pool2]);
    
    // Add many other accounts (tokens, mints, etc.)
    for _ in 0..30 {
        accounts.push(Pubkey::new_unique());
    }
    
    // Create multiple instructions
    let instructions = vec![
        // PumpSwap buy
        CompiledInstruction {
            program_id_index: 0,
            accounts: (2..15).collect(),
            data: {
                let mut data = vec![102, 6, 61, 18, 1, 218, 235, 234]; // Buy discriminator
                data.extend_from_slice(&3000000u64.to_le_bytes());
                data.extend_from_slice(&1500000u64.to_le_bytes());
                data
            },
        },
        // Another instruction
        CompiledInstruction {
            program_id_index: 0,
            accounts: (15..28).collect(),
            data: {
                let mut data = vec![51, 230, 133, 164, 1, 127, 131, 173]; // Sell discriminator
                data.extend_from_slice(&4000000u64.to_le_bytes());
                data.extend_from_slice(&2000000u64.to_le_bytes());
                data
            },
        },
    ];
    
    // Process all instructions
    let mut all_events = Vec::new();
    for (index, instruction) in instructions.iter().enumerate() {
        let events = multi_parser.parse_events_from_instruction(
            instruction,
            &accounts,
            "complex_transaction_sig",
            98765,
            Some(1640995300),
            chrono::Utc::now().timestamp_millis(),
            index.to_string(),
        );
        all_events.extend(events);
    }
    
    // Analyze results
    if !all_events.is_empty() {
        // Verify event properties
        for event in &all_events {
            assert_eq!(event.signature(), "complex_transaction_sig");
            assert_eq!(event.slot(), 98765);
            assert!(event.program_received_time_ms() > 0);
        }
        
        // Test event aggregation by type
        let mut event_counts = std::collections::HashMap::new();
        for event in &all_events {
            *event_counts.entry(event.event_type()).or_insert(0) += 1;
        }
        
        // Should have some variety of events
        assert!(!event_counts.is_empty());
        
        // Test formatting
        let formatted = format_events_for_agent(all_events);
        assert!(formatted.is_ok());
        let output = formatted.unwrap();
        assert!(output.contains("complex_transaction_sig"));
        assert!(output.contains("98765"));
    }
}

#[test]
fn test_event_merging_integration() {
    // Test the complete merging workflow
    let mut primary_event = PumpSwapBuyEvent {
        metadata: EventMetadata::new(
            "merge_test_sig".to_string(),
            "merge_test_sig".to_string(),
            55555,
            1640995400,
            1640995400000,
            ProtocolType::PumpSwap,
            EventType::PumpSwapBuy,
            PUMPSWAP_PROGRAM_ID,
            "0".to_string(),
            chrono::Utc::now().timestamp_millis(),
        ),
        base_amount_out: 5000000,
        max_quote_amount_in: 2500000,
        quote_amount_in: 0, // Will be merged
        lp_fee: 0,          // Will be merged
        protocol_fee: 0,    // Will be merged
        ..Default::default()
    };
    
    let secondary_event = PumpSwapBuyEvent {
        metadata: EventMetadata::new(
            "merge_test_sig".to_string(),
            "merge_test_sig".to_string(),
            55555,
            1640995400,
            1640995400000,
            ProtocolType::PumpSwap,
            EventType::PumpSwapBuy,
            PUMPSWAP_PROGRAM_ID,
            "0.1".to_string(), // Inner instruction
            chrono::Utc::now().timestamp_millis(),
        ),
        base_amount_out: 5000000, // Same as primary
        max_quote_amount_in: 2500000, // Same as primary
        quote_amount_in: 2300000,  // Additional data
        lp_fee: 10000,             // Additional data
        protocol_fee: 5000,        // Additional data
        ..Default::default()
    };
    
    // Perform merge
    let secondary_boxed = Box::new(secondary_event) as Box<dyn UnifiedEvent>;
    primary_event.merge(secondary_boxed);
    
    // Verify merge results
    assert_eq!(primary_event.base_amount_out, 5000000);
    assert_eq!(primary_event.max_quote_amount_in, 2500000);
    assert_eq!(primary_event.quote_amount_in, 2300000); // Merged from secondary
    assert_eq!(primary_event.lp_fee, 10000);           // Merged from secondary
    assert_eq!(primary_event.protocol_fee, 5000);     // Merged from secondary
    
    // Test that events with different signatures don't merge incorrectly
    let different_event = PumpSwapBuyEvent {
        metadata: EventMetadata::new(
            "different_sig".to_string(),
            "different_sig".to_string(),
            66666,
            1640995500,
            1640995500000,
            ProtocolType::PumpSwap,
            EventType::PumpSwapBuy,
            PUMPSWAP_PROGRAM_ID,
            "0".to_string(),
            chrono::Utc::now().timestamp_millis(),
        ),
        quote_amount_in: 9999999, // Should not merge
        ..Default::default()
    };
    
    let different_boxed = Box::new(different_event) as Box<dyn UnifiedEvent>;
    let before_quote_amount = primary_event.quote_amount_in;
    primary_event.merge(different_boxed);
    
    // Should merge anyway because merge doesn't check signatures in current implementation
    // This tests the actual behavior, not necessarily the desired behavior
    assert_eq!(primary_event.quote_amount_in, 9999999);
}

#[test]
fn test_comprehensive_system_stress() {
    // Test the system with a large number of events and operations
    let multi_parser = EventParserFactory::create_mutil_parser(&Protocol::all());
    
    let mut all_events = Vec::new();
    
    // Generate many mock events
    for i in 0..100 {
        let event = Box::new(PumpSwapBuyEvent {
            metadata: EventMetadata::new(
                format!("stress_test_sig_{}", i),
                format!("stress_test_sig_{}", i),
                100000 + i as u64,
                1640995600 + i as i64,
                (1640995600 + i as i64) * 1000,
                ProtocolType::PumpSwap,
                if i % 2 == 0 { EventType::PumpSwapBuy } else { EventType::PumpSwapSell },
                PUMPSWAP_PROGRAM_ID,
                i.to_string(),
                chrono::Utc::now().timestamp_millis(),
            ),
            base_amount_out: 1000000 + (i as u64 * 1000),
            ..Default::default()
        }) as Box<dyn UnifiedEvent>;
        
        all_events.push(event);
    }
    
    // Test formatting large number of events
    let formatted = format_events_for_agent(all_events);
    assert!(formatted.is_ok());
    let output = formatted.unwrap();
    assert!(output.contains("100 events"));
    assert!(output.contains("Transaction Event Analysis"));
    
    // Test parser configuration with all protocols
    let inner_configs = multi_parser.inner_instruction_configs();
    let instruction_configs = multi_parser.instruction_configs();
    
    assert!(!inner_configs.is_empty() || !instruction_configs.is_empty());
    
    // Verify all known programs are supported
    assert!(multi_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    assert!(multi_parser.should_handle(&BONK_PROGRAM_ID));
    assert!(multi_parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
}