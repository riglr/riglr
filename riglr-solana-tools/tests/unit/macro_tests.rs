//! Unit tests specifically for macro implementations
//! Tests the impl_unified_event! and match_event! macros

use riglr_solana_tools::{
    events::{
        core::traits::UnifiedEvent,
        common::{EventMetadata, EventType, ProtocolType, TransferData, SwapData},
        protocols::pumpswap::{PumpSwapBuyEvent, PumpSwapSellEvent, PumpSwapCreatePoolEvent, PUMPSWAP_PROGRAM_ID},
    },
    match_event,
};
use solana_sdk::pubkey::Pubkey;

fn create_test_metadata(event_type: EventType, index: &str) -> EventMetadata {
    EventMetadata::new(
        "test_sig".to_string(),
        "test_sig".to_string(),
        12345,
        1640995200,
        1640995200000,
        ProtocolType::PumpSwap,
        event_type,
        PUMPSWAP_PROGRAM_ID,
        index.to_string(),
        1640995200000,
    )
}

#[test]
fn test_impl_unified_event_basic_functionality() {
    let mut buy_event = PumpSwapBuyEvent {
        metadata: create_test_metadata(EventType::PumpSwapBuy, "0"),
        base_amount_out: 1000000,
        max_quote_amount_in: 500000,
        pool: Pubkey::new_unique(),
        user: Pubkey::new_unique(),
        ..Default::default()
    };

    // Test basic trait methods
    assert_eq!(buy_event.event_type(), EventType::PumpSwapBuy);
    assert_eq!(buy_event.signature(), "test_sig");
    assert_eq!(buy_event.slot(), 12345);
    assert_eq!(buy_event.protocol_type(), ProtocolType::PumpSwap);
    assert_eq!(buy_event.program_received_time_ms(), 1640995200000);
    assert_eq!(buy_event.program_handle_time_consuming_ms(), 0);
    assert_eq!(buy_event.index(), "0");

    // Test mutable methods
    buy_event.set_program_handle_time_consuming_ms(100);
    assert_eq!(buy_event.program_handle_time_consuming_ms(), 100);
}

#[test]
fn test_impl_unified_event_transfer_data_setting() {
    let mut buy_event = PumpSwapBuyEvent {
        metadata: create_test_metadata(EventType::PumpSwapBuy, "0"),
        ..Default::default()
    };

    let transfer_data = vec![
        TransferData {
            token_program: Pubkey::new_unique(),
            source: Pubkey::new_unique(),
            destination: Pubkey::new_unique(),
            authority: Some(Pubkey::new_unique()),
            amount: 1000000,
            decimals: Some(9),
            mint: Some(Pubkey::new_unique()),
        },
    ];

    let swap_data = Some(SwapData {
        from_mint: Pubkey::new_unique(),
        to_mint: Pubkey::new_unique(),
        from_amount: 1000000,
        to_amount: 500000,
        description: Some("Test swap".to_string()),
    });

    buy_event.set_transfer_datas(transfer_data.clone(), swap_data.clone());

    assert_eq!(buy_event.metadata.transfer_datas.len(), 1);
    assert!(buy_event.metadata.swap_data.is_some());
    assert_eq!(buy_event.metadata.transfer_datas[0].amount, 1000000);
    assert_eq!(buy_event.metadata.swap_data.as_ref().unwrap().from_amount, 1000000);
}

#[test]
fn test_impl_unified_event_cloning() {
    let original = PumpSwapBuyEvent {
        metadata: create_test_metadata(EventType::PumpSwapBuy, "0"),
        base_amount_out: 2000000,
        max_quote_amount_in: 1000000,
        pool: Pubkey::new_unique(),
        user: Pubkey::new_unique(),
        ..Default::default()
    };

    let cloned = original.clone_boxed();

    assert_eq!(cloned.event_type(), original.event_type());
    assert_eq!(cloned.signature(), original.signature());
    assert_eq!(cloned.slot(), original.slot());
    assert_eq!(cloned.protocol_type(), original.protocol_type());
    
    // Verify it can be downcast back
    let downcast_result = cloned.as_any().downcast_ref::<PumpSwapBuyEvent>();
    assert!(downcast_result.is_some());
    let downcast_event = downcast_result.unwrap();
    assert_eq!(downcast_event.base_amount_out, 2000000);
    assert_eq!(downcast_event.max_quote_amount_in, 1000000);
}

#[test]
fn test_impl_unified_event_merging() {
    let mut target = PumpSwapBuyEvent {
        metadata: create_test_metadata(EventType::PumpSwapBuy, "0"),
        base_amount_out: 1000000,
        max_quote_amount_in: 500000,
        quote_amount_in: 0,
        lp_fee: 0,
        pool: Pubkey::new_unique(),
        user: Pubkey::new_unique(),
        ..Default::default()
    };

    let source = PumpSwapBuyEvent {
        metadata: create_test_metadata(EventType::PumpSwapBuy, "0.1"),
        base_amount_out: 2000000,
        max_quote_amount_in: 1000000,
        quote_amount_in: 750000,
        lp_fee: 5000,
        pool: Pubkey::new_unique(),
        user: Pubkey::new_unique(),
        ..Default::default()
    };

    let source_boxed = Box::new(source) as Box<dyn UnifiedEvent>;
    target.merge(source_boxed);

    // Verify fields were merged
    assert_eq!(target.base_amount_out, 2000000);
    assert_eq!(target.max_quote_amount_in, 1000000);
    assert_eq!(target.quote_amount_in, 750000);
    assert_eq!(target.lp_fee, 5000);
}

#[test]
fn test_impl_unified_event_merge_wrong_type() {
    let mut buy_event = PumpSwapBuyEvent {
        metadata: create_test_metadata(EventType::PumpSwapBuy, "0"),
        base_amount_out: 1000000,
        ..Default::default()
    };

    let sell_event = PumpSwapSellEvent {
        metadata: create_test_metadata(EventType::PumpSwapSell, "1"),
        base_amount_in: 2000000,
        ..Default::default()
    };

    let original_amount = buy_event.base_amount_out;
    
    // Merge with different type should not change the target
    let sell_boxed = Box::new(sell_event) as Box<dyn UnifiedEvent>;
    buy_event.merge(sell_boxed);

    assert_eq!(buy_event.base_amount_out, original_amount); // Should remain unchanged
}

#[test]
fn test_impl_unified_event_as_any() {
    let buy_event = PumpSwapBuyEvent {
        metadata: create_test_metadata(EventType::PumpSwapBuy, "0"),
        base_amount_out: 3000000,
        ..Default::default()
    };

    // Test immutable as_any
    let any_ref = buy_event.as_any();
    let downcast_ref = any_ref.downcast_ref::<PumpSwapBuyEvent>();
    assert!(downcast_ref.is_some());
    assert_eq!(downcast_ref.unwrap().base_amount_out, 3000000);

    // Test failed downcast
    let failed_downcast = any_ref.downcast_ref::<PumpSwapSellEvent>();
    assert!(failed_downcast.is_none());
}

#[test]
fn test_impl_unified_event_as_any_mut() {
    let mut buy_event = PumpSwapBuyEvent {
        metadata: create_test_metadata(EventType::PumpSwapBuy, "0"),
        base_amount_out: 3000000,
        ..Default::default()
    };

    // Test mutable as_any
    let any_mut_ref = buy_event.as_any_mut();
    let downcast_mut_ref = any_mut_ref.downcast_mut::<PumpSwapBuyEvent>();
    assert!(downcast_mut_ref.is_some());
    
    let downcast_mut = downcast_mut_ref.unwrap();
    downcast_mut.base_amount_out = 4000000;
    
    assert_eq!(buy_event.base_amount_out, 4000000);
}

#[test]
fn test_match_event_single_match() {
    let buy_event = Box::new(PumpSwapBuyEvent {
        metadata: create_test_metadata(EventType::PumpSwapBuy, "0"),
        base_amount_out: 1500000,
        ..Default::default()
    }) as Box<dyn UnifiedEvent>;

    let mut matched = false;
    let mut extracted_amount = 0u64;

    match_event!(buy_event, {
        PumpSwapBuyEvent => |event: &PumpSwapBuyEvent| {
            matched = true;
            extracted_amount = event.base_amount_out;
        },
    });

    assert!(matched);
    assert_eq!(extracted_amount, 1500000);
}

#[test]
fn test_match_event_multiple_patterns() {
    let sell_event = Box::new(PumpSwapSellEvent {
        metadata: create_test_metadata(EventType::PumpSwapSell, "1"),
        base_amount_in: 2500000,
        min_quote_amount_out: 1200000,
        ..Default::default()
    }) as Box<dyn UnifiedEvent>;

    let mut matched_type = "";
    let mut amount = 0u64;

    match_event!(sell_event, {
        PumpSwapBuyEvent => |buy: &PumpSwapBuyEvent| {
            matched_type = "buy";
            amount = buy.base_amount_out;
        },
        PumpSwapSellEvent => |sell: &PumpSwapSellEvent| {
            matched_type = "sell";
            amount = sell.base_amount_in;
        },
        PumpSwapCreatePoolEvent => |create: &PumpSwapCreatePoolEvent| {
            matched_type = "create";
            amount = create.base_amount_in;
        },
    });

    assert_eq!(matched_type, "sell");
    assert_eq!(amount, 2500000);
}

#[test]
fn test_match_event_no_match() {
    let buy_event = Box::new(PumpSwapBuyEvent {
        metadata: create_test_metadata(EventType::PumpSwapBuy, "0"),
        base_amount_out: 1500000,
        ..Default::default()
    }) as Box<dyn UnifiedEvent>;

    let mut matched = false;

    match_event!(buy_event, {
        PumpSwapSellEvent => |_sell: &PumpSwapSellEvent| {
            matched = true;
        },
        PumpSwapCreatePoolEvent => |_create: &PumpSwapCreatePoolEvent| {
            matched = true;
        },
    });

    // Should not match since we only have patterns for sell and create events
    assert!(!matched);
}

#[test]
fn test_match_event_complex_matching() {
    let create_event = Box::new(PumpSwapCreatePoolEvent {
        metadata: create_test_metadata(EventType::PumpSwapCreatePool, "2"),
        index: 1,
        creator: Pubkey::new_unique(),
        base_mint: Pubkey::new_unique(),
        quote_mint: Pubkey::new_unique(),
        base_amount_in: 5000000,
        quote_amount_in: 2000000,
        ..Default::default()
    }) as Box<dyn UnifiedEvent>;

    let mut result = String::new();

    match_event!(create_event, {
        PumpSwapBuyEvent => |buy: &PumpSwapBuyEvent| {
            result = format!("Buy: {} out", buy.base_amount_out);
        },
        PumpSwapSellEvent => |sell: &PumpSwapSellEvent| {
            result = format!("Sell: {} in", sell.base_amount_in);
        },
        PumpSwapCreatePoolEvent => |create: &PumpSwapCreatePoolEvent| {
            result = format!("Create: {} base, {} quote", create.base_amount_in, create.quote_amount_in);
        },
    });

    assert_eq!(result, "Create: 5000000 base, 2000000 quote");
}

#[test]
fn test_match_event_with_mutations() {
    let mut sell_event = PumpSwapSellEvent {
        metadata: create_test_metadata(EventType::PumpSwapSell, "1"),
        base_amount_in: 3000000,
        min_quote_amount_out: 1500000,
        ..Default::default()
    };

    // First, box it
    let mut boxed_event = Box::new(sell_event) as Box<dyn UnifiedEvent>;
    
    // Use match_event to examine (not mutate since we can't get mutable reference)
    let mut verified_amount = 0u64;
    match_event!(boxed_event, {
        PumpSwapSellEvent => |sell: &PumpSwapSellEvent| {
            verified_amount = sell.base_amount_in;
        },
    });

    assert_eq!(verified_amount, 3000000);
}

#[test]
fn test_macro_with_different_event_types() {
    // Test that the macro works with events from different structs
    let events: Vec<Box<dyn UnifiedEvent>> = vec![
        Box::new(PumpSwapBuyEvent {
            metadata: create_test_metadata(EventType::PumpSwapBuy, "0"),
            base_amount_out: 1000000,
            ..Default::default()
        }),
        Box::new(PumpSwapSellEvent {
            metadata: create_test_metadata(EventType::PumpSwapSell, "1"),
            base_amount_in: 2000000,
            ..Default::default()
        }),
        Box::new(PumpSwapCreatePoolEvent {
            metadata: create_test_metadata(EventType::PumpSwapCreatePool, "2"),
            base_amount_in: 3000000,
            ..Default::default()
        }),
    ];

    let mut results = Vec::new();

    for event in events {
        match_event!(event, {
            PumpSwapBuyEvent => |buy: &PumpSwapBuyEvent| {
                results.push(format!("buy:{}", buy.base_amount_out));
            },
            PumpSwapSellEvent => |sell: &PumpSwapSellEvent| {
                results.push(format!("sell:{}", sell.base_amount_in));
            },
            PumpSwapCreatePoolEvent => |create: &PumpSwapCreatePoolEvent| {
                results.push(format!("create:{}", create.base_amount_in));
            },
        });
    }

    assert_eq!(results.len(), 3);
    assert_eq!(results[0], "buy:1000000");
    assert_eq!(results[1], "sell:2000000");
    assert_eq!(results[2], "create:3000000");
}

#[test]
fn test_macro_field_specification() {
    // Test that the macro correctly handles the field specification for merging
    let mut target = PumpSwapBuyEvent {
        metadata: create_test_metadata(EventType::PumpSwapBuy, "0"),
        base_amount_out: 1000000,
        max_quote_amount_in: 500000,
        user_base_token_reserves: 100000,
        lp_fee: 1000,
        protocol_fee: 2000,
        ..Default::default()
    };

    let source = PumpSwapBuyEvent {
        metadata: create_test_metadata(EventType::PumpSwapBuy, "0.1"),
        base_amount_out: 2000000,
        max_quote_amount_in: 1000000,
        user_base_token_reserves: 200000,
        lp_fee: 2000,
        protocol_fee: 4000,
        ..Default::default()
    };

    let source_boxed = Box::new(source) as Box<dyn UnifiedEvent>;
    target.merge(source_boxed);

    // All the fields specified in the macro should be merged
    assert_eq!(target.base_amount_out, 2000000);
    assert_eq!(target.max_quote_amount_in, 1000000);
    assert_eq!(target.user_base_token_reserves, 200000);
    assert_eq!(target.lp_fee, 2000);
    assert_eq!(target.protocol_fee, 4000);
}

#[test]
fn test_macro_generated_methods_consistency() {
    // Test that all macro-generated methods work consistently across different event types
    let buy_event = PumpSwapBuyEvent {
        metadata: create_test_metadata(EventType::PumpSwapBuy, "buy"),
        ..Default::default()
    };

    let sell_event = PumpSwapSellEvent {
        metadata: create_test_metadata(EventType::PumpSwapSell, "sell"),
        ..Default::default()
    };

    let create_event = PumpSwapCreatePoolEvent {
        metadata: create_test_metadata(EventType::PumpSwapCreatePool, "create"),
        ..Default::default()
    };

    // Test that all have consistent behavior
    let events: Vec<Box<dyn UnifiedEvent>> = vec![
        Box::new(buy_event),
        Box::new(sell_event),
        Box::new(create_event),
    ];

    for event in &events {
        assert_eq!(event.signature(), "test_sig");
        assert_eq!(event.slot(), 12345);
        assert_eq!(event.protocol_type(), ProtocolType::PumpSwap);
        assert!(event.program_handle_time_consuming_ms() >= 0);
        
        // Test cloning
        let cloned = event.clone_boxed();
        assert_eq!(cloned.signature(), event.signature());
        assert_eq!(cloned.event_type(), event.event_type());
    }
}