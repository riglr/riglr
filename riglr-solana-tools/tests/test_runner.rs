//! Test runner to verify all event parsing tests compile and run
//! This test ensures basic compilation of the event parsing system

use riglr_solana_tools::events::{
    core::traits::UnifiedEvent,
    common::{EventMetadata, EventType, ProtocolType},
    factory::{EventParserFactory, Protocol},
    protocols::pumpswap::{PumpSwapBuyEvent, PUMPSWAP_PROGRAM_ID},
};
//use solana_sdk::pubkey::Pubkey;

#[test]
fn test_basic_event_system_compilation() {
    // Test that the core components can be instantiated
    let parser = EventParserFactory::create_parser(Protocol::PumpSwap);
    assert!(!parser.supported_program_ids().is_empty());
    
    // Test multi-parser creation
    let multi_parser = EventParserFactory::create_mutil_parser(&[Protocol::PumpSwap, Protocol::Bonk]);
    assert!(multi_parser.should_handle(&PUMPSWAP_PROGRAM_ID));
    
    // Test event creation
    let metadata = EventMetadata::new(
        "test".to_string(),
        "test".to_string(),
        12345,
        1640995200,
        1640995200000,
        ProtocolType::PumpSwap,
        EventType::PumpSwapBuy,
        PUMPSWAP_PROGRAM_ID,
        "0".to_string(),
        chrono::Utc::now().timestamp_millis(),
    );
    
    let event = PumpSwapBuyEvent {
        metadata,
        ..Default::default()
    };
    
    // Test trait methods
    assert_eq!(event.event_type(), EventType::PumpSwapBuy);
    assert_eq!(event.protocol_type(), ProtocolType::PumpSwap);
    
    println!("✅ Basic event system compilation test passed");
}

#[test]
fn test_tool_functions_exist() {
    // Test that tool functions are available
    use riglr_solana_tools::{
        format_events_for_agent, parse_protocol_strings
    };
    
    // Test protocol parsing
    let protocols = vec!["PumpSwap".to_string()];
    let result = parse_protocol_strings(protocols);
    assert!(result.is_ok());
    
    // Test event formatting
    let empty_events: Vec<Box<dyn UnifiedEvent>> = vec![];
    let result = format_events_for_agent(empty_events);
    assert!(result.is_ok());
    
    println!("✅ Tool functions availability test passed");
}

#[test]
fn test_macro_imports() {
    // Test that match_event macro is available
    use riglr_solana_tools::match_event;
    
    let event = Box::new(PumpSwapBuyEvent {
        metadata: EventMetadata::new(
            "test".to_string(),
            "test".to_string(),
            12345,
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
    }) as Box<dyn UnifiedEvent>;
    
    #[allow(unused_assignments)]
    let mut matched = false;
    match_event!(event, {
        PumpSwapBuyEvent => |_buy: &PumpSwapBuyEvent| {
            matched = true;
        },
    });
    
    assert!(matched);
    println!("✅ Macro imports test passed");
}

#[test]
fn test_all_protocols_instantiate() {
    // Test that all protocols can be instantiated
    for protocol in Protocol::all() {
        let parser = EventParserFactory::create_parser(protocol);
        assert!(!parser.supported_program_ids().is_empty(), 
               "Protocol {:?} has no supported program IDs", protocol);
    }
    
    println!("✅ All protocols instantiation test passed");
}