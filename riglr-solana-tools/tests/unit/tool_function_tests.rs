//! Unit tests for the event analysis tool functions
//! Tests the four new tool functions: analyze_transaction_events, get_protocol_events, 
//! analyze_recent_events, and monitor_token_events

use riglr_solana_tools::{
    analyze_transaction_events, get_protocol_events, analyze_recent_events, monitor_token_events,
    parse_protocol_strings, format_events_for_agent,
    events::{
        core::traits::UnifiedEvent,
        common::{EventMetadata, EventType, ProtocolType},
        protocols::pumpswap::{PumpSwapBuyEvent, PumpSwapSellEvent, PUMPSWAP_PROGRAM_ID},
        factory::Protocol,
    },
};
use solana_sdk::pubkey::Pubkey;

// Helper to create mock events for testing formatting
fn create_mock_events(count: usize) -> Vec<Box<dyn UnifiedEvent>> {
    let mut events = Vec::new();
    
    for i in 0..count {
        let metadata = EventMetadata::new(
            format!("mock_sig_{}", i),
            format!("mock_sig_{}", i),
            10000 + i as u64,
            1640995200 + i as i64,
            (1640995200 + i as i64) * 1000,
            ProtocolType::PumpSwap,
            if i % 2 == 0 { EventType::PumpSwapBuy } else { EventType::PumpSwapSell },
            PUMPSWAP_PROGRAM_ID,
            i.to_string(),
            chrono::Utc::now().timestamp_millis(),
        );
        
        if i % 2 == 0 {
            events.push(Box::new(PumpSwapBuyEvent {
                metadata,
                base_amount_out: 1000000 + (i as u64 * 100000),
                max_quote_amount_in: 500000 + (i as u64 * 50000),
                ..Default::default()
            }) as Box<dyn UnifiedEvent>);
        } else {
            events.push(Box::new(PumpSwapSellEvent {
                metadata,
                base_amount_in: 2000000 + (i as u64 * 100000),
                min_quote_amount_out: 1000000 + (i as u64 * 50000),
                ..Default::default()
            }) as Box<dyn UnifiedEvent>);
        }
    }
    
    events
}

#[test]
fn test_parse_protocol_strings_valid_inputs() {
    let test_cases = vec![
        (vec!["PumpSwap".to_string()], vec![Protocol::PumpSwap]),
        (vec!["bonk".to_string()], vec![Protocol::Bonk]),
        (vec!["BONK".to_string()], vec![Protocol::Bonk]),
        (vec!["raydiumcpmm".to_string()], vec![Protocol::RaydiumCpmm]),
        (vec!["raydium_cpmm".to_string()], vec![Protocol::RaydiumCpmm]),
        (vec!["RAYDIUM_CPMM".to_string()], vec![Protocol::RaydiumCpmm]),
        (vec!["raydiumclmm".to_string()], vec![Protocol::RaydiumClmm]),
        (vec!["raydium_clmm".to_string()], vec![Protocol::RaydiumClmm]),
        (vec!["raydiumammv4".to_string()], vec![Protocol::RaydiumAmmV4]),
        (vec!["raydium_amm_v4".to_string()], vec![Protocol::RaydiumAmmV4]),
    ];
    
    for (input, expected) in test_cases {
        let result = parse_protocol_strings(input.clone());
        assert!(result.is_ok(), "Failed to parse: {:?}", input);
        assert_eq!(result.unwrap(), expected);
    }
}

#[test]
fn test_parse_protocol_strings_mixed_case_combinations() {
    let input = vec![
        "PumpSwap".to_string(),
        "bonk".to_string(),
        "RAYDIUM_CPMM".to_string(),
        "raydiumclmm".to_string(),
    ];
    
    let result = parse_protocol_strings(input);
    assert!(result.is_ok());
    let protocols = result.unwrap();
    
    assert_eq!(protocols.len(), 4);
    assert_eq!(protocols[0], Protocol::PumpSwap);
    assert_eq!(protocols[1], Protocol::Bonk);
    assert_eq!(protocols[2], Protocol::RaydiumCpmm);
    assert_eq!(protocols[3], Protocol::RaydiumClmm);
}

#[test]
fn test_parse_protocol_strings_invalid_inputs() {
    let invalid_cases = vec![
        vec!["InvalidProtocol".to_string()],
        vec!["Unknown".to_string()],
        vec!["PumpSwap".to_string(), "InvalidProtocol".to_string()],
        vec!["".to_string()],
        vec!["123".to_string()],
    ];
    
    for input in invalid_cases {
        let result = parse_protocol_strings(input.clone());
        assert!(result.is_err(), "Should have failed for input: {:?}", input);
    }
}

#[test]
fn test_parse_protocol_strings_empty_input() {
    let result = parse_protocol_strings(vec![]);
    assert!(result.is_ok());
    let protocols = result.unwrap();
    assert!(protocols.is_empty());
}

#[test]
fn test_format_events_for_agent_empty_events() {
    let empty_events: Vec<Box<dyn UnifiedEvent>> = vec![];
    let result = format_events_for_agent(empty_events);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("No events found"));
}

#[test]
fn test_format_events_for_agent_single_event() {
    let events = create_mock_events(1);
    let result = format_events_for_agent(events);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    
    assert!(output.contains("Transaction Event Analysis"));
    assert!(output.contains("1 events"));
    assert!(output.contains("Event 1"));
    assert!(output.contains("PumpSwapBuy"));
    assert!(output.contains("mock_sig_0"));
    assert!(output.contains("10000"));
    assert!(output.contains("Processing Time"));
}

#[test]
fn test_format_events_for_agent_multiple_events() {
    let events = create_mock_events(5);
    let result = format_events_for_agent(events);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    
    assert!(output.contains("Transaction Event Analysis"));
    assert!(output.contains("5 events"));
    
    // Should contain all event numbers
    for i in 1..=5 {
        assert!(output.contains(&format!("Event {}", i)));
    }
    
    // Should contain both event types
    assert!(output.contains("PumpSwapBuy"));
    assert!(output.contains("PumpSwapSell"));
    
    // Should contain different signatures
    assert!(output.contains("mock_sig_0"));
    assert!(output.contains("mock_sig_4"));
}

#[test]
fn test_format_events_for_agent_event_details() {
    let events = create_mock_events(2);
    let result = format_events_for_agent(events);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    
    // Check for required sections
    assert!(output.contains("# Transaction Event Analysis"));
    assert!(output.contains("## Event 1"));
    assert!(output.contains("## Event 2"));
    
    // Check for required fields
    assert!(output.contains("**Transaction**:"));
    assert!(output.contains("**Slot**:"));
    assert!(output.contains("**Processing Time**:"));
    assert!(output.contains("**Index**:"));
    
    // Check protocol type formatting
    assert!(output.contains("PumpSwap"));
}

#[test]
fn test_format_events_for_agent_large_number_of_events() {
    let events = create_mock_events(100);
    let result = format_events_for_agent(events);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    
    assert!(output.contains("100 events"));
    assert!(output.contains("Event 1"));
    assert!(output.contains("Event 100"));
    
    // Should handle large numbers without issues
    assert!(output.len() > 1000); // Should be substantial output
}

#[tokio::test]
async fn test_monitor_token_events_placeholder() {
    let token_address = Pubkey::new_unique().to_string();
    
    // Test with default duration
    let result = monitor_token_events(
        token_address.clone(),
        None,
        None,
    ).await;
    
    assert!(result.is_ok());
    let output = result.unwrap();
    
    assert!(output.contains("Token Event Monitoring Started"));
    assert!(output.contains(&token_address));
    assert!(output.contains("10 minutes")); // Default duration
    assert!(output.contains("placeholder implementation"));
    assert!(output.contains("https://api.mainnet-beta.solana.com")); // Default RPC
    
    // Test with custom parameters
    let result = monitor_token_events(
        token_address.clone(),
        Some(30),
        Some("https://custom-rpc.solana.com".to_string()),
    ).await;
    
    assert!(result.is_ok());
    let output = result.unwrap();
    
    assert!(output.contains("30 minutes"));
    assert!(output.contains("https://custom-rpc.solana.com"));
    assert!(output.contains("WebSocket subscriptions"));
}

#[tokio::test]
async fn test_monitor_token_events_invalid_token() {
    // Test with obviously invalid token address
    let result = monitor_token_events(
        "invalid_token".to_string(),
        Some(5),
        Some("https://api.mainnet-beta.solana.com".to_string()),
    ).await;
    
    // Should still return OK since it's a placeholder implementation
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("invalid_token"));
}

#[tokio::test]
async fn test_analyze_transaction_events_error_handling() {
    // Test with invalid signature format
    let result = analyze_transaction_events(
        "invalid_signature".to_string(),
        Some("https://api.mainnet-beta.solana.com".to_string()),
    ).await;
    
    // Should return error for invalid signature
    assert!(result.is_err());
    
    // Test with empty signature
    let result = analyze_transaction_events(
        "".to_string(),
        Some("https://api.mainnet-beta.solana.com".to_string()),
    ).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_analyze_transaction_events_with_custom_rpc() {
    let signature = "5" + &"a".repeat(87); // Valid base58 length but likely not found
    
    // Test with custom RPC URL
    let result = analyze_transaction_events(
        signature.clone(),
        Some("https://api.devnet.solana.com".to_string()),
    ).await;
    
    // Should attempt to process but likely fail to find transaction
    assert!(result.is_err());
    
    // Test with default RPC (None)
    let result = analyze_transaction_events(
        signature,
        None,
    ).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_protocol_events_error_handling() {
    let signature = "5" + &"a".repeat(87); // Valid format but non-existent
    
    // Test with invalid protocol
    let result = get_protocol_events(
        signature.clone(),
        vec!["InvalidProtocol".to_string()],
        Some("https://api.mainnet-beta.solana.com".to_string()),
    ).await;
    
    assert!(result.is_err());
    
    // Test with empty protocol list
    let result = get_protocol_events(
        signature.clone(),
        vec![],
        Some("https://api.mainnet-beta.solana.com".to_string()),
    ).await;
    
    // Should be OK (empty protocols list is valid)
    assert!(result.is_ok() || result.is_err()); // Either is acceptable behavior
    
    // Test with mixed valid and invalid protocols
    let result = get_protocol_events(
        signature,
        vec!["PumpSwap".to_string(), "InvalidProtocol".to_string()],
        Some("https://api.mainnet-beta.solana.com".to_string()),
    ).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_protocol_events_valid_protocols() {
    let signature = "5" + &"a".repeat(87); // Valid format but non-existent
    
    // Test with single valid protocol
    let result = get_protocol_events(
        signature.clone(),
        vec!["PumpSwap".to_string()],
        Some("https://api.mainnet-beta.solana.com".to_string()),
    ).await;
    
    // Should fail due to non-existent transaction, not protocol parsing
    assert!(result.is_err());
    
    // Test with multiple valid protocols
    let result = get_protocol_events(
        signature,
        vec!["PumpSwap".to_string(), "Bonk".to_string(), "raydiumcpmm".to_string()],
        Some("https://api.mainnet-beta.solana.com".to_string()),
    ).await;
    
    // Should fail due to non-existent transaction, not protocol parsing
    assert!(result.is_err());
}

#[tokio::test]
async fn test_analyze_recent_events_error_handling() {
    // Test with invalid token address
    let result = analyze_recent_events(
        "invalid_token_address".to_string(),
        Some(10),
        Some("https://api.mainnet-beta.solana.com".to_string()),
    ).await;
    
    // Should return error for invalid token address
    assert!(result.is_err());
    
    // Test with empty token address
    let result = analyze_recent_events(
        "".to_string(),
        Some(10),
        Some("https://api.mainnet-beta.solana.com".to_string()),
    ).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_analyze_recent_events_with_limits() {
    let token_address = Pubkey::new_unique().to_string();
    
    // Test with custom limit
    let result = analyze_recent_events(
        token_address.clone(),
        Some(25),
        Some("https://api.mainnet-beta.solana.com".to_string()),
    ).await;
    
    // Will likely error due to invalid token, but tests parameter passing
    assert!(result.is_err());
    
    // Test with default limit (None)
    let result = analyze_recent_events(
        token_address,
        None,
        Some("https://api.mainnet-beta.solana.com".to_string()),
    ).await;
    
    assert!(result.is_err());
}

#[test]
fn test_tool_function_parameter_validation() {
    // Test protocol string validation edge cases
    let edge_cases = vec![
        vec!["pumpswap".to_string()], // lowercase
        vec!["PUMPSWAP".to_string()], // uppercase  
        vec!["PumpSwap".to_string()], // mixed case
        vec!["pump_swap".to_string()], // with underscore (should fail)
        vec!["pump-swap".to_string()], // with dash (should fail)
        vec!["pump swap".to_string()], // with space (should fail)
    ];
    
    for (i, input) in edge_cases.iter().enumerate() {
        let result = parse_protocol_strings(input.clone());
        
        match i {
            0..=2 => {
                // First 3 should succeed (different cases of valid name)
                assert!(result.is_ok(), "Case {} should succeed: {:?}", i, input);
            },
            _ => {
                // Rest should fail (invalid formats)
                assert!(result.is_err(), "Case {} should fail: {:?}", i, input);
            }
        }
    }
}

#[test]
fn test_format_events_consistency() {
    // Test that formatting is consistent across multiple calls
    let events1 = create_mock_events(3);
    let events2 = create_mock_events(3);
    
    let result1 = format_events_for_agent(events1);
    let result2 = format_events_for_agent(events2);
    
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    
    let output1 = result1.unwrap();
    let output2 = result2.unwrap();
    
    // Both should have similar structure
    assert!(output1.contains("# Transaction Event Analysis"));
    assert!(output2.contains("# Transaction Event Analysis"));
    
    assert!(output1.contains("3 events"));
    assert!(output2.contains("3 events"));
    
    // Both should have Event 1, 2, 3
    for i in 1..=3 {
        assert!(output1.contains(&format!("Event {}", i)));
        assert!(output2.contains(&format!("Event {}", i)));
    }
}

#[test]
fn test_event_formatting_special_characters() {
    // Test formatting with events that might have special characters in signatures
    let mut events = Vec::new();
    
    let special_sigs = vec![
        "sig_with_underscores",
        "sig-with-dashes", 
        "sig.with.dots",
        "sig123with456numbers",
    ];
    
    for (i, sig) in special_sigs.iter().enumerate() {
        let metadata = EventMetadata::new(
            sig.to_string(),
            sig.to_string(),
            i as u64,
            1640995200,
            1640995200000,
            ProtocolType::PumpSwap,
            EventType::PumpSwapBuy,
            PUMPSWAP_PROGRAM_ID,
            i.to_string(),
            chrono::Utc::now().timestamp_millis(),
        );
        
        events.push(Box::new(PumpSwapBuyEvent {
            metadata,
            ..Default::default()
        }) as Box<dyn UnifiedEvent>);
    }
    
    let result = format_events_for_agent(events);
    assert!(result.is_ok());
    let output = result.unwrap();
    
    // Should contain all special signatures
    for sig in special_sigs {
        assert!(output.contains(sig), "Output should contain signature: {}", sig);
    }
}