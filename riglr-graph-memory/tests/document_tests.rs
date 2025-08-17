//! Comprehensive tests for document module

use chrono::Utc;
use riglr_graph_memory::document::*;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_raw_text_document_new() {
    let doc = RawTextDocument::new("test content");

    assert!(!doc.id.is_empty());
    assert_eq!(doc.content, "test content");
    assert!(doc.metadata.is_none());
    assert!(doc.embedding.is_none());
    assert!(matches!(doc.source, DocumentSource::UserInput));
}

#[test]
fn test_raw_text_document_with_metadata() {
    let mut metadata = DocumentMetadata::default();
    metadata.title = Some("Test Document".to_string());
    metadata.add_tag("test");

    let doc = RawTextDocument::with_metadata("content", metadata.clone());

    assert!(!doc.id.is_empty());
    assert_eq!(doc.content, "content");
    assert!(doc.metadata.is_some());

    let doc_metadata = doc.metadata.unwrap();
    assert_eq!(doc_metadata.title, Some("Test Document".to_string()));
    assert_eq!(doc_metadata.tags, vec!["test"]);
}

#[test]
fn test_raw_text_document_with_source() {
    let source = DocumentSource::OnChain {
        chain: "ethereum".to_string(),
        transaction_hash: "0x123".to_string(),
    };

    let doc = RawTextDocument::with_source("transaction data", source.clone());

    assert_eq!(doc.content, "transaction data");
    assert!(matches!(doc.source, DocumentSource::OnChain { .. }));

    if let DocumentSource::OnChain {
        chain,
        transaction_hash,
    } = doc.source
    {
        assert_eq!(chain, "ethereum");
        assert_eq!(transaction_hash, "0x123");
    }
}

#[test]
fn test_raw_text_document_from_transaction() {
    let doc = RawTextDocument::from_transaction("tx content", "solana", "abc123def456");

    assert_eq!(doc.content, "tx content");

    // Check source
    assert!(matches!(doc.source, DocumentSource::OnChain { .. }));
    if let DocumentSource::OnChain {
        chain,
        transaction_hash,
    } = doc.source
    {
        assert_eq!(chain, "solana");
        assert_eq!(transaction_hash, "abc123def456");
    }

    // Check metadata
    assert!(doc.metadata.is_some());
    let metadata = doc.metadata.unwrap();
    assert_eq!(metadata.chain, Some("solana".to_string()));
    assert_eq!(metadata.transaction_hash, Some("abc123def456".to_string()));
}

#[test]
fn test_raw_text_document_is_processed() {
    let mut doc = RawTextDocument::new("test");
    assert!(!doc.is_processed());

    doc.embedding = Some(vec![0.1, 0.2, 0.3]);
    assert!(doc.is_processed());
}

#[test]
fn test_raw_text_document_word_count() {
    let doc = RawTextDocument::new("This is a test document with several words");
    assert_eq!(doc.word_count(), 8);

    let doc2 = RawTextDocument::new("");
    assert_eq!(doc2.word_count(), 0);

    let doc3 = RawTextDocument::new("   multiple   spaces   between   words   ");
    assert_eq!(doc3.word_count(), 4);
}

#[test]
fn test_raw_text_document_char_count() {
    let doc = RawTextDocument::new("Hello");
    assert_eq!(doc.char_count(), 5);

    let doc2 = RawTextDocument::new("");
    assert_eq!(doc2.char_count(), 0);

    let doc3 = RawTextDocument::new("Hello 世界"); // With Unicode
    assert_eq!(doc3.char_count(), "Hello 世界".len());
}

#[test]
fn test_raw_text_document_serialization() {
    let mut doc = RawTextDocument::new("test content");
    doc.embedding = Some(vec![0.1, 0.2]);

    let json = serde_json::to_string(&doc).unwrap();
    assert!(json.contains("\"content\":\"test content\""));
    assert!(json.contains("\"embedding\":[0.1,0.2]"));

    let deserialized: RawTextDocument = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.content, doc.content);
    assert_eq!(deserialized.embedding, doc.embedding);
}

#[test]
fn test_document_metadata_new() {
    let metadata = DocumentMetadata::default();

    assert!(metadata.title.is_none());
    assert!(metadata.tags.is_empty());
    assert!(metadata.chain.is_none());
    assert!(metadata.block_number.is_none());
    assert!(metadata.transaction_hash.is_none());
    assert!(metadata.wallet_addresses.is_empty());
    assert!(metadata.token_addresses.is_empty());
    assert!(metadata.protocols.is_empty());
    assert!(metadata.extraction_confidence.is_none());
    assert!(metadata.custom_fields.is_empty());
}

#[test]
fn test_document_metadata_add_tag() {
    let mut metadata = DocumentMetadata::default();

    metadata.add_tag("defi");
    metadata.add_tag("ethereum");
    metadata.add_tag("swap");

    assert_eq!(metadata.tags, vec!["defi", "ethereum", "swap"]);
}

#[test]
fn test_document_metadata_add_wallet() {
    let mut metadata = DocumentMetadata::default();

    metadata.add_wallet("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb");
    metadata.add_wallet("0x123456789abcdef");

    assert_eq!(metadata.wallet_addresses.len(), 2);
    assert!(metadata
        .wallet_addresses
        .contains(&"0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string()));
}

#[test]
fn test_document_metadata_add_token() {
    let mut metadata = DocumentMetadata::default();

    metadata.add_token("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    metadata.add_token("0xdAC17F958D2ee523a2206206994597C13D831ec7");

    assert_eq!(metadata.token_addresses.len(), 2);
}

#[test]
fn test_document_metadata_add_protocol() {
    let mut metadata = DocumentMetadata::default();

    metadata.add_protocol("Uniswap");
    metadata.add_protocol("Aave");
    metadata.add_protocol("Compound");

    assert_eq!(metadata.protocols, vec!["Uniswap", "Aave", "Compound"]);
}

#[test]
fn test_document_metadata_complex() {
    let mut metadata = DocumentMetadata::default();

    metadata.title = Some("DeFi Transaction Analysis".to_string());
    metadata.chain = Some("ethereum".to_string());
    metadata.block_number = Some(18500000);
    metadata.transaction_hash = Some("0xabc123".to_string());
    metadata.extraction_confidence = Some(0.95);

    metadata.add_tag("defi");
    metadata.add_wallet("0xwallet1");
    metadata.add_token("0xtoken1");
    metadata.add_protocol("Protocol1");

    metadata
        .custom_fields
        .insert("gas_price".to_string(), json!(20000000000u64));
    metadata
        .custom_fields
        .insert("is_suspicious".to_string(), json!(false));

    assert_eq!(
        metadata.title,
        Some("DeFi Transaction Analysis".to_string())
    );
    assert_eq!(metadata.chain, Some("ethereum".to_string()));
    assert_eq!(metadata.block_number, Some(18500000));
    assert_eq!(metadata.extraction_confidence, Some(0.95));
    assert!(metadata.custom_fields.contains_key("gas_price"));
}

#[test]
fn test_document_metadata_serialization() {
    let mut metadata = DocumentMetadata::default();
    metadata.title = Some("Test".to_string());
    metadata.chain = Some("solana".to_string());
    metadata.add_tag("test");
    metadata
        .custom_fields
        .insert("key".to_string(), json!("value"));

    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("\"title\":\"Test\""));
    assert!(json.contains("\"chain\":\"solana\""));

    let deserialized: DocumentMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.title, metadata.title);
    assert_eq!(deserialized.chain, metadata.chain);
    assert_eq!(deserialized.tags, metadata.tags);
}

#[test]
fn test_document_source_variants() {
    let user_input = DocumentSource::UserInput;
    assert!(matches!(user_input, DocumentSource::UserInput));

    let onchain = DocumentSource::OnChain {
        chain: "ethereum".to_string(),
        transaction_hash: "0x123".to_string(),
    };
    assert!(matches!(onchain, DocumentSource::OnChain { .. }));

    let social = DocumentSource::Social {
        platform: "Twitter".to_string(),
        post_id: "123456789".to_string(),
        author: Some("@user".to_string()),
    };
    assert!(matches!(social, DocumentSource::Social { .. }));

    let news = DocumentSource::News {
        url: "https://example.com/article".to_string(),
        publication: Some("Example News".to_string()),
    };
    assert!(matches!(news, DocumentSource::News { .. }));

    let api = DocumentSource::ApiResponse {
        endpoint: "/api/v1/data".to_string(),
        timestamp: Utc::now(),
    };
    assert!(matches!(api, DocumentSource::ApiResponse { .. }));

    let other = DocumentSource::Other("Custom source".to_string());
    assert!(matches!(other, DocumentSource::Other(_)));
}

#[test]
fn test_document_source_serialization() {
    let source = DocumentSource::OnChain {
        chain: "solana".to_string(),
        transaction_hash: "sig123".to_string(),
    };

    let json = serde_json::to_string(&source).unwrap();
    assert!(json.contains("OnChain"));
    assert!(json.contains("solana"));
    assert!(json.contains("sig123"));

    let deserialized: DocumentSource = serde_json::from_str(&json).unwrap();
    assert!(matches!(deserialized, DocumentSource::OnChain { .. }));
}

#[test]
fn test_extracted_entities_creation() {
    let entities = ExtractedEntities {
        wallets: vec![],
        tokens: vec![],
        protocols: vec![],
        chains: vec![],
        amounts: vec![],
        relationships: vec![],
    };

    assert!(entities.wallets.is_empty());
    assert!(entities.relationships.is_empty());
}

#[test]
fn test_entity_mention_creation() {
    let mention = EntityMention {
        text: "0x742d35Cc...".to_string(),
        canonical: "0x742d35cc6634c0532925a3b844bc9e7595f0beb".to_string(),
        entity_type: EntityType::Wallet,
        confidence: 0.95,
        span: (10, 52),
        properties: HashMap::new(),
    };

    assert_eq!(mention.text, "0x742d35Cc...");
    assert_eq!(mention.confidence, 0.95);
    assert_eq!(mention.span, (10, 52));
    assert!(matches!(mention.entity_type, EntityType::Wallet));
}

#[test]
fn test_entity_mention_with_properties() {
    let mut properties = HashMap::new();
    properties.insert("label".to_string(), "Vitalik's Wallet".to_string());
    properties.insert("balance".to_string(), "1000 ETH".to_string());

    let mention = EntityMention {
        text: "vitalik.eth".to_string(),
        canonical: "0xd8da6bf26964af9d7eed9e03e53415d37aa96045".to_string(),
        entity_type: EntityType::Wallet,
        confidence: 1.0,
        span: (0, 11),
        properties,
    };

    assert_eq!(
        mention.properties.get("label"),
        Some(&"Vitalik's Wallet".to_string())
    );
    assert_eq!(
        mention.properties.get("balance"),
        Some(&"1000 ETH".to_string())
    );
}

#[test]
fn test_entity_type_variants() {
    let wallet = EntityType::Wallet;
    let token = EntityType::Token;
    let protocol = EntityType::Protocol;
    let chain = EntityType::Chain;
    let other = EntityType::Other("NFT".to_string());

    assert!(matches!(wallet, EntityType::Wallet));
    assert!(matches!(token, EntityType::Token));
    assert!(matches!(protocol, EntityType::Protocol));
    assert!(matches!(chain, EntityType::Chain));
    assert!(matches!(other, EntityType::Other(_)));
}

#[test]
fn test_amount_mention_creation() {
    let amount = AmountMention {
        text: "1.5 ETH".to_string(),
        value: 1.5,
        unit: Some("ETH".to_string()),
        amount_type: AmountType::Balance,
        span: (100, 107),
    };

    assert_eq!(amount.text, "1.5 ETH");
    assert_eq!(amount.value, 1.5);
    assert_eq!(amount.unit, Some("ETH".to_string()));
    assert!(matches!(amount.amount_type, AmountType::Balance));
}

#[test]
fn test_amount_mention_without_unit() {
    let amount = AmountMention {
        text: "1000000".to_string(),
        value: 1000000.0,
        unit: None,
        amount_type: AmountType::Volume,
        span: (50, 57),
    };

    assert_eq!(amount.value, 1000000.0);
    assert!(amount.unit.is_none());
}

#[test]
fn test_amount_type_variants() {
    let balance = AmountType::Balance;
    let price = AmountType::Price;
    let fee = AmountType::Fee;
    let volume = AmountType::Volume;
    let market_cap = AmountType::MarketCap;
    let other = AmountType::Other("TVL".to_string());

    assert!(matches!(balance, AmountType::Balance));
    assert!(matches!(price, AmountType::Price));
    assert!(matches!(fee, AmountType::Fee));
    assert!(matches!(volume, AmountType::Volume));
    assert!(matches!(market_cap, AmountType::MarketCap));
    assert!(matches!(other, AmountType::Other(_)));
}

#[test]
fn test_relationship_mention_creation() {
    let relationship = RelationshipMention {
        from_entity: "0xwallet1".to_string(),
        to_entity: "0xwallet2".to_string(),
        relationship_type: RelationshipType::Transferred,
        confidence: 0.9,
        context: "0xwallet1 sent 100 USDC to 0xwallet2".to_string(),
    };

    assert_eq!(relationship.from_entity, "0xwallet1");
    assert_eq!(relationship.to_entity, "0xwallet2");
    assert_eq!(relationship.confidence, 0.9);
    assert!(matches!(
        relationship.relationship_type,
        RelationshipType::Transferred
    ));
}

#[test]
fn test_relationship_type_variants() {
    let transferred = RelationshipType::Transferred;
    let interacted = RelationshipType::Interacted;
    let holds = RelationshipType::Holds;
    let part_of = RelationshipType::PartOf;
    let deployed_on = RelationshipType::DeployedOn;
    let related = RelationshipType::Related;

    assert!(matches!(transferred, RelationshipType::Transferred));
    assert!(matches!(interacted, RelationshipType::Interacted));
    assert!(matches!(holds, RelationshipType::Holds));
    assert!(matches!(part_of, RelationshipType::PartOf));
    assert!(matches!(deployed_on, RelationshipType::DeployedOn));
    assert!(matches!(related, RelationshipType::Related));
}

#[test]
fn test_complex_extracted_entities() {
    let mut wallet_props = HashMap::new();
    wallet_props.insert("ens".to_string(), "vitalik.eth".to_string());

    let entities = ExtractedEntities {
        wallets: vec![EntityMention {
            text: "0x742d...".to_string(),
            canonical: "0x742d35cc6634c0532925a3b844bc9e7595f0beb".to_string(),
            entity_type: EntityType::Wallet,
            confidence: 0.95,
            span: (0, 9),
            properties: wallet_props,
        }],
        tokens: vec![EntityMention {
            text: "USDC".to_string(),
            canonical: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(),
            entity_type: EntityType::Token,
            confidence: 1.0,
            span: (20, 24),
            properties: HashMap::new(),
        }],
        protocols: vec![EntityMention {
            text: "Uniswap".to_string(),
            canonical: "uniswap".to_string(),
            entity_type: EntityType::Protocol,
            confidence: 0.98,
            span: (30, 37),
            properties: HashMap::new(),
        }],
        chains: vec![EntityMention {
            text: "Ethereum".to_string(),
            canonical: "ethereum".to_string(),
            entity_type: EntityType::Chain,
            confidence: 1.0,
            span: (40, 48),
            properties: HashMap::new(),
        }],
        amounts: vec![AmountMention {
            text: "100 USDC".to_string(),
            value: 100.0,
            unit: Some("USDC".to_string()),
            amount_type: AmountType::Balance,
            span: (50, 58),
        }],
        relationships: vec![RelationshipMention {
            from_entity: "0x742d35cc6634c0532925a3b844bc9e7595f0beb".to_string(),
            to_entity: "uniswap".to_string(),
            relationship_type: RelationshipType::Interacted,
            confidence: 0.85,
            context: "wallet swapped on Uniswap".to_string(),
        }],
    };

    assert_eq!(entities.wallets.len(), 1);
    assert_eq!(entities.tokens.len(), 1);
    assert_eq!(entities.protocols.len(), 1);
    assert_eq!(entities.chains.len(), 1);
    assert_eq!(entities.amounts.len(), 1);
    assert_eq!(entities.relationships.len(), 1);
}

#[test]
fn test_all_serialization_roundtrip() {
    // Test complete serialization/deserialization
    let mut metadata = DocumentMetadata::default();
    metadata.title = Some("Test".to_string());
    metadata.add_tag("blockchain");
    metadata
        .custom_fields
        .insert("test".to_string(), json!(true));

    let doc = RawTextDocument::with_metadata("content", metadata);

    let entities = ExtractedEntities {
        wallets: vec![EntityMention {
            text: "wallet".to_string(),
            canonical: "0xabc".to_string(),
            entity_type: EntityType::Wallet,
            confidence: 0.9,
            span: (0, 6),
            properties: HashMap::new(),
        }],
        tokens: vec![],
        protocols: vec![],
        chains: vec![],
        amounts: vec![AmountMention {
            text: "10 ETH".to_string(),
            value: 10.0,
            unit: Some("ETH".to_string()),
            amount_type: AmountType::Balance,
            span: (10, 16),
        }],
        relationships: vec![],
    };

    // Serialize everything
    let doc_json = serde_json::to_string(&doc).unwrap();
    let entities_json = serde_json::to_string(&entities).unwrap();

    // Deserialize and verify
    let doc_deser: RawTextDocument = serde_json::from_str(&doc_json).unwrap();
    let entities_deser: ExtractedEntities = serde_json::from_str(&entities_json).unwrap();

    assert_eq!(doc_deser.content, doc.content);
    assert_eq!(entities_deser.wallets.len(), entities.wallets.len());
    assert_eq!(entities_deser.amounts.len(), entities.amounts.len());
}

#[test]
fn test_edge_cases() {
    // Empty document
    let empty_doc = RawTextDocument::new("");
    assert_eq!(empty_doc.word_count(), 0);
    assert_eq!(empty_doc.char_count(), 0);

    // Very long content
    let long_content = "a".repeat(10000);
    let long_doc = RawTextDocument::new(&long_content);
    assert_eq!(long_doc.char_count(), 10000);

    // Special characters in content
    let special_doc = RawTextDocument::new("Content with 特殊字符 and émojis 🚀");
    assert!(special_doc.word_count() > 0);

    // Large confidence values
    let mention = EntityMention {
        text: "test".to_string(),
        canonical: "test".to_string(),
        entity_type: EntityType::Other("test".to_string()),
        confidence: 1.0,
        span: (0, 4),
        properties: HashMap::new(),
    };
    assert_eq!(mention.confidence, 1.0);

    // Zero confidence
    let zero_mention = EntityMention {
        text: "test".to_string(),
        canonical: "test".to_string(),
        entity_type: EntityType::Other("test".to_string()),
        confidence: 0.0,
        span: (0, 4),
        properties: HashMap::new(),
    };
    assert_eq!(zero_mention.confidence, 0.0);
}

#[test]
fn test_document_clone() {
    let mut doc = RawTextDocument::new("test");
    doc.embedding = Some(vec![0.1, 0.2]);

    let cloned = doc.clone();
    assert_eq!(cloned.content, doc.content);
    assert_eq!(cloned.embedding, doc.embedding);
    assert_eq!(cloned.id, doc.id);
}

#[test]
fn test_metadata_clone() {
    let mut metadata = DocumentMetadata::default();
    metadata.title = Some("Test".to_string());
    metadata.add_tag("tag1");

    let cloned = metadata.clone();
    assert_eq!(cloned.title, metadata.title);
    assert_eq!(cloned.tags, metadata.tags);
}

#[test]
fn test_document_debug() {
    let doc = RawTextDocument::new("test");
    let debug_str = format!("{:?}", doc);

    assert!(debug_str.contains("RawTextDocument"));
    assert!(debug_str.contains("content"));
}

#[test]
fn test_metadata_default() {
    let metadata = DocumentMetadata::default();
    assert!(metadata.title.is_none());
    assert!(metadata.tags.is_empty());
    assert!(metadata.chain.is_none());
}
