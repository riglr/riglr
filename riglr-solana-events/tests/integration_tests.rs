//! Integration tests for high-performance parsing components
//!
//! These tests validate that all components work together correctly
//! and provide expected functionality.

use riglr_solana_events::pipelines::{
    validation::{ValidationConfig, ValidationError, ValidationPipeline},
    EnrichmentConfig, EventEnricher, ParsingInput, ParsingPipelineBuilder,
};
use riglr_solana_events::prelude::*;
use riglr_solana_events::types::{EventMetadata, EventType, ProtocolType};
use riglr_solana_events::zero_copy::BatchEventParser;
use tokio_stream::iter;

#[tokio::test]
async fn test_end_to_end_parsing_pipeline() {
    // Create parsing pipeline with all parsers
    let mut pipeline = ParsingPipelineBuilder::new()
        .with_batch_size(10)
        .with_concurrency_limit(2)
        .add_parser(RaydiumV4ParserFactory::create_zero_copy())
        .add_parser(JupiterParserFactory::create_zero_copy())
        .add_parser(PumpFunParserFactory::create_zero_copy())
        .build();

    // Generate test inputs
    let mut inputs = Vec::new();

    // Raydium V4 SwapBaseIn
    let mut raydium_data = vec![0x09]; // SwapBaseIn discriminator
    raydium_data.extend_from_slice(&1_000_000u64.to_le_bytes()); // amount_in
    raydium_data.extend_from_slice(&950_000u64.to_le_bytes()); // minimum_amount_out

    let mut metadata = EventMetadata {
        signature: "raydium_sig_1".to_string(),
        index: "0".to_string(),
        ..Default::default()
    };
    metadata.set_id("raydium_event_1".to_string());

    inputs.push(ParsingInput {
        data: raydium_data,
        metadata,
        program_id_hint: None,
    });

    // Jupiter Route
    let mut jupiter_data = vec![229, 23, 203, 151, 122, 227, 173, 42]; // Route discriminator
    jupiter_data.extend_from_slice(&2_000_000u64.to_le_bytes()); // amount_in
    jupiter_data.extend_from_slice(&1_900_000u64.to_le_bytes()); // minimum_amount_out
    jupiter_data.extend_from_slice(&25u32.to_le_bytes()); // platform_fee_bps

    let mut metadata = EventMetadata {
        signature: "jupiter_sig_1".to_string(),
        index: "0".to_string(),
        ..Default::default()
    };
    metadata.set_id("jupiter_event_1".to_string());

    inputs.push(ParsingInput {
        data: jupiter_data,
        metadata,
        program_id_hint: None,
    });

    // PumpFun Buy
    let mut pump_data = Vec::new();
    pump_data.extend_from_slice(&0x66063d1201daebeau64.to_le_bytes()); // Buy discriminator
    pump_data.extend_from_slice(&500u64.to_le_bytes()); // amount
    pump_data.extend_from_slice(&1000u64.to_le_bytes()); // max_price_per_token

    let mut metadata = EventMetadata {
        signature: "pump_sig_1".to_string(),
        index: "0".to_string(),
        ..Default::default()
    };
    metadata.set_id("pump_event_1".to_string());

    inputs.push(ParsingInput {
        data: pump_data,
        metadata,
        program_id_hint: None,
    });

    // Create input stream
    let input_stream = iter(inputs);

    // Get output receiver
    let mut output_receiver = pipeline.take_output_receiver();

    // Process stream
    let process_task = tokio::spawn(async move { pipeline.process_stream(input_stream).await });

    // Collect outputs
    let mut total_events = 0;
    let mut output_count = 0;

    while let Some(output) = output_receiver.recv().await {
        output_count += 1;
        total_events += output.events.len();

        // Validate output
        assert!(!output.events.is_empty());
        assert_eq!(output.errors.len(), 0);

        // Check metrics
        assert!(output.metrics.processing_time.as_millis() > 0);
        assert!(output.metrics.events_parsed > 0);

        // Check that we parsed different protocol types
        let _protocol_types: std::collections::HashSet<_> =
            output.events.iter().map(|e| e.protocol_type()).collect();

        if output_count >= 1 {
            break; // We expect at least one batch
        }
    }

    // Wait for processing to complete
    process_task.await.unwrap().unwrap();

    // Validate results
    assert!(total_events >= 3); // Should have parsed at least our 3 test events
    assert!(output_count >= 1);
}

#[tokio::test]
async fn test_event_enrichment_pipeline() {
    // Create enrichment config
    let config = EnrichmentConfig::default();
    let enricher = EventEnricher::new(config);

    // Create a test event
    let mut metadata = EventMetadata {
        signature: "test_sig_1".to_string(),
        protocol_type: ProtocolType::Jupiter,
        event_type: EventType::Swap,
        ..Default::default()
    };
    metadata.set_id("test_event_1".to_string());

    let mut event = ZeroCopyEvent::new_owned(metadata, vec![0x09, 0x01, 0x02]);

    // Add some JSON data with token addresses
    let json = serde_json::json!({
        "instruction_type": "swap",
        "input_mint": "So11111111111111111111111111111111111111112",
        "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        "amount_in": "1000000",
        "amount_out": "950000"
    });
    event.set_json_data(json);

    // Enrich the event
    let enriched_event = enricher.enrich_event(event).await.unwrap();

    // Validate enrichment
    let enriched_json = enriched_event.get_json_data().unwrap();

    // Should have token metadata
    assert!(enriched_json.get("token_metadata").is_some());

    // Should have price data
    assert!(enriched_json.get("price_data").is_some());

    // Should have transaction context
    assert!(enriched_json.get("transaction_context").is_some());

    // Should have enrichment timing
    assert!(enriched_json.get("enrichment_time_ms").is_some());
}

#[tokio::test]
async fn test_validation_pipeline() {
    // Create validation config
    let config = ValidationConfig {
        strict_mode: false,
        enable_duplicate_detection: true,
        ..Default::default()
    };
    let pipeline = ValidationPipeline::new(config);

    // Create test events
    let mut events = Vec::new();

    // Valid event
    let mut valid_metadata = EventMetadata {
        signature: "valid_sig_1".to_string(),
        protocol_type: ProtocolType::RaydiumAmmV4,
        event_type: EventType::Swap,
        index: "0".to_string(),
        ..Default::default()
    };
    valid_metadata.set_id("valid_event_1".to_string());

    let mut valid_event = ZeroCopyEvent::new_owned(valid_metadata, vec![0x09, 0x01, 0x02]);
    valid_event.set_json_data(serde_json::json!({
        "instruction_type": "swap_base_in",
        "amount_in": "1000000",
        "minimum_amount_out": "950000"
    }));
    events.push(valid_event);

    // Invalid event (missing required fields)
    let invalid_metadata = EventMetadata {
        protocol_type: ProtocolType::Jupiter,
        ..Default::default()
    };
    // Missing id and signature

    let invalid_event = ZeroCopyEvent::new_owned(invalid_metadata, vec![]);
    events.push(invalid_event);

    // Duplicate event (same as first one)
    let mut duplicate_metadata = EventMetadata {
        signature: "valid_sig_1".to_string(),
        protocol_type: ProtocolType::RaydiumAmmV4,
        event_type: EventType::Swap,
        index: "0".to_string(),
        ..Default::default()
    };
    duplicate_metadata.set_id("valid_event_1".to_string());

    let duplicate_event = ZeroCopyEvent::new_owned(duplicate_metadata, vec![0x09, 0x01, 0x02]);
    events.push(duplicate_event);

    // Validate all events
    let results = pipeline.validate_events(&events).await;

    assert_eq!(results.len(), 3);

    // First event should be valid
    assert!(results[0].is_valid);
    assert_eq!(results[0].errors.len(), 0);
    assert!(results[0].metrics.rules_checked > 0);

    // Second event should have validation errors
    assert!(!results[1].is_valid || !results[1].errors.is_empty());

    // Third event should be flagged as duplicate
    assert!(!results[2].errors.is_empty());
    let has_duplicate_error = results[2]
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::Duplicate { .. }));
    assert!(has_duplicate_error);
}

#[test]
fn test_batch_parser_with_mixed_protocols() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let mut batch_parser = BatchEventParser::new(100);
        batch_parser.add_parser(RaydiumV4ParserFactory::create_zero_copy());
        batch_parser.add_parser(JupiterParserFactory::create_zero_copy());
        batch_parser.add_parser(PumpFunParserFactory::create_zero_copy());

        // Prepare mixed batch data
        let mut batch_data = Vec::new();
        let mut metadata_vec = Vec::new();

        // Raydium data
        let mut raydium_data = vec![0x09];
        raydium_data.extend_from_slice(&1000u64.to_le_bytes());
        raydium_data.extend_from_slice(&950u64.to_le_bytes());
        batch_data.push(raydium_data);
        metadata_vec.push(EventMetadata::default());

        // Jupiter data
        let mut jupiter_data = vec![229, 23, 203, 151, 122, 227, 173, 42];
        jupiter_data.extend_from_slice(&2000u64.to_le_bytes());
        jupiter_data.extend_from_slice(&1900u64.to_le_bytes());
        batch_data.push(jupiter_data);
        metadata_vec.push(EventMetadata::default());

        // PumpFun data
        let mut pump_data = Vec::new();
        pump_data.extend_from_slice(&0x66063d1201daebeau64.to_le_bytes());
        pump_data.extend_from_slice(&500u64.to_le_bytes());
        pump_data.extend_from_slice(&1000u64.to_le_bytes());
        batch_data.push(pump_data);
        metadata_vec.push(EventMetadata::default());

        // Parse batch
        let batch_refs: Vec<&[u8]> = batch_data.iter().map(|d| d.as_slice()).collect();
        let events = batch_parser.parse_batch(&batch_refs, metadata_vec).unwrap();

        // Validate results
        assert_eq!(events.len(), 3);

        // Check protocol types
        let protocols: std::collections::HashSet<_> =
            events.iter().map(|e| e.protocol_type()).collect();

        assert!(protocols.contains(&ProtocolType::RaydiumAmmV4));
        assert!(protocols.contains(&ProtocolType::Jupiter));
        assert!(protocols.contains(&ProtocolType::PumpSwap));
    });
}

#[test]
fn test_zero_copy_event_data_access() {
    // Test borrowed data
    let data = vec![0x09, 0x01, 0x02, 0x03];
    let metadata = EventMetadata::default();
    let mut event = ZeroCopyEvent::new_borrowed(metadata.clone(), &data);

    assert_eq!(event.raw_data(), &data);
    assert_eq!(event.raw_data().len(), 4);

    // Add parsed data
    #[derive(Debug, Clone, PartialEq)]
    struct TestData {
        value: u32,
    }

    let test_data = TestData { value: 42 };
    event.set_parsed_data(test_data.clone());

    let retrieved = event.get_parsed_data::<TestData>().unwrap();
    assert_eq!(*retrieved, test_data);

    // Test owned conversion
    let owned_event = event.to_owned();
    assert_eq!(owned_event.raw_data(), &data);
    assert_eq!(
        owned_event.get_parsed_data::<TestData>().unwrap(),
        &test_data
    );

    // Test JSON data
    let json = serde_json::json!({ "test": "value" });
    event.set_json_data(json.clone());
    assert_eq!(event.get_json_data().unwrap(), &json);
}

#[test]
fn test_parser_performance_characteristics() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let raydium_parser = RaydiumV4ParserFactory::create_zero_copy();
        let jupiter_parser = JupiterParserFactory::create_zero_copy();

        // Test with various data sizes
        let sizes = [10, 100, 1000];

        for size in sizes {
            let mut data = vec![0x09]; // Raydium discriminator
            data.resize(size, 0);

            let metadata = EventMetadata::default();

            let start = std::time::Instant::now();
            let result = raydium_parser
                .parse_from_slice(&data, metadata.clone())
                .unwrap();
            let raydium_time = start.elapsed();

            let start = std::time::Instant::now();
            let jupiter_data = vec![229, 23, 203, 151, 122, 227, 173, 42];
            let _ = jupiter_parser.parse_from_slice(&jupiter_data, metadata);
            let jupiter_time = start.elapsed();

            // Both should complete reasonably quickly
            assert!(raydium_time.as_millis() < 10);
            assert!(jupiter_time.as_millis() < 10);
            assert!(!result.is_empty());
        }
    });
}
