//! Example: RAG Agent with Graph Memory
//!
//! This example demonstrates how to build a Retrieval-Augmented Generation (RAG) agent
//! that uses riglr-graph-memory to answer questions about wallet transaction history
//! and blockchain activities stored in a Neo4j knowledge graph.

use riglr_graph_memory::{
    GraphMemory, GraphMemoryConfig, RawTextDocument, DocumentMetadata, DocumentSource,
};
use std::env;

/// Simulated transaction data for demonstration
struct TransactionData {
    wallet: String,
    action: String,
    token: String,
    amount: String,
    protocol: String,
    timestamp: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🤖 RAG Agent with Graph Memory Example\n");
    println!("Building a knowledge graph of wallet transaction history\n");

    // Initialize graph memory
    let config = GraphMemoryConfig {
        neo4j_url: env::var("NEO4J_URL").unwrap_or_else(|_| "neo4j://localhost:7687".to_string()),
        username: Some(env::var("NEO4J_USERNAME").unwrap_or_else(|_| "neo4j".to_string())),
        password: Some(env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string())),
        ..Default::default()
    };

    let mut memory = match GraphMemory::new(config).await {
        Ok(m) => m,
        Err(e) => {
            println!("⚠️ Could not connect to Neo4j: {}", e);
            println!("Please ensure Neo4j is running with:");
            println!("docker run -d --name neo4j -p 7474:7474 -p 7687:7687 \\");
            println!("  -e NEO4J_AUTH=neo4j/password neo4j:5.13.0");
            return Ok(());
        }
    };

    println!("✅ Connected to Neo4j graph database\n");

    // Example 1: Build knowledge graph from transaction history
    println!("📊 Building Knowledge Graph...\n");
    build_transaction_knowledge_graph(&mut memory).await?;

    // Example 2: Answer questions about wallet activity
    println!("\n🔍 Answering Questions About Wallet Activity...\n");
    answer_wallet_questions(&memory).await?;

    // Example 3: Analyze transaction patterns
    println!("\n📈 Analyzing Transaction Patterns...\n");
    analyze_transaction_patterns(&memory).await?;

    // Example 4: Simulate RAG agent queries
    println!("\n💬 Simulating RAG Agent Queries...\n");
    simulate_rag_agent(&memory).await?;

    println!("\n✅ RAG agent example complete!");

    Ok(())
}

/// Build a knowledge graph from transaction history
async fn build_transaction_knowledge_graph(memory: &mut GraphMemory) -> anyhow::Result<()> {
    // Sample transaction data (in production, this would come from blockchain indexers)
    let transactions = vec![
        TransactionData {
            wallet: "0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B".to_string(),
            action: "swapped".to_string(),
            token: "1000 USDC for 0.5 ETH".to_string(),
            amount: "1000".to_string(),
            protocol: "Uniswap V3".to_string(),
            timestamp: "2024-01-15 10:30:00".to_string(),
        },
        TransactionData {
            wallet: "0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B".to_string(),
            action: "provided liquidity".to_string(),
            token: "500 USDC and 0.25 ETH".to_string(),
            amount: "500".to_string(),
            protocol: "Uniswap V3".to_string(),
            timestamp: "2024-01-15 11:00:00".to_string(),
        },
        TransactionData {
            wallet: "0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B".to_string(),
            action: "staked".to_string(),
            token: "10 ETH".to_string(),
            amount: "10".to_string(),
            protocol: "Lido".to_string(),
            timestamp: "2024-01-16 09:00:00".to_string(),
        },
        TransactionData {
            wallet: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            action: "borrowed".to_string(),
            token: "5000 USDC".to_string(),
            amount: "5000".to_string(),
            protocol: "Aave V3".to_string(),
            timestamp: "2024-01-16 14:30:00".to_string(),
        },
        TransactionData {
            wallet: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            action: "deposited".to_string(),
            token: "2 ETH".to_string(),
            amount: "2".to_string(),
            protocol: "Compound".to_string(),
            timestamp: "2024-01-17 10:15:00".to_string(),
        },
    ];

    // Add each transaction to the knowledge graph
    for (idx, tx) in transactions.iter().enumerate() {
        let content = format!(
            "On {}, wallet {} {} {} on {}",
            tx.timestamp, tx.wallet, tx.action, tx.token, tx.protocol
        );

        // Create metadata for better organization
        let mut metadata = DocumentMetadata::default();
        metadata.source = Some(DocumentSource::Blockchain {
            chain: "ethereum".to_string(),
            block_number: Some(18500000 + idx as u64),
            transaction_hash: Some(format!("0x{:064x}", idx + 1)),
        });
        metadata.tags = vec![
            "defi".to_string(),
            tx.protocol.to_lowercase().replace(" ", "_"),
            tx.action.replace(" ", "_"),
        ];
        metadata.timestamp = Some(chrono::Utc::now());

        let doc = RawTextDocument::with_metadata(content, metadata);
        
        match memory.add_documents(vec![doc]).await {
            Ok(doc_ids) => {
                println!("  ✅ Added transaction {}: {:?}", idx + 1, doc_ids);
            }
            Err(e) => {
                println!("  ⚠️ Failed to add transaction {}: {}", idx + 1, e);
            }
        }
    }

    println!("\n📊 Knowledge graph built with {} transactions", transactions.len());

    // Get statistics
    match memory.get_stats().await {
        Ok(stats) => {
            println!("  Documents: {}", stats.document_count);
            println!("  Entities: {}", stats.entity_count);
            println!("  Relationships: {}", stats.relationship_count);
        }
        Err(e) => {
            println!("  ⚠️ Could not fetch statistics: {}", e);
        }
    }

    Ok(())
}

/// Answer questions about wallet activity
async fn answer_wallet_questions(memory: &GraphMemory) -> anyhow::Result<()> {
    let questions = vec![
        ("What protocols has wallet 0x742d35Cc used?", "0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B"),
        ("What DeFi activities happened on Uniswap?", "Uniswap"),
        ("Which wallets have borrowed tokens?", "borrowed"),
    ];

    for (question, query) in questions {
        println!("Q: {}", question);
        
        match memory.find_related_documents(query, Some(5)).await {
            Ok(docs) => {
                if docs.is_empty() {
                    println!("A: No relevant information found.\n");
                } else {
                    println!("A: Found {} relevant activities:", docs.len());
                    for (idx, doc) in docs.iter().take(3).enumerate() {
                        println!("   {}. {}", idx + 1, doc.content);
                    }
                    println!();
                }
            }
            Err(e) => {
                println!("A: Error retrieving information: {}\n", e);
            }
        }
    }

    Ok(())
}

/// Analyze transaction patterns using graph queries
async fn analyze_transaction_patterns(memory: &GraphMemory) -> anyhow::Result<()> {
    // Analyze wallet activity patterns
    let wallet_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B";
    
    println!("Analyzing wallet: {}", wallet_address);
    
    match memory.get_wallet_history(wallet_address).await {
        Ok(history) => {
            println!("  Transaction count: {}", history.documents.len());
            
            // Extract unique protocols
            let protocols: Vec<String> = history.documents
                .iter()
                .filter_map(|doc| {
                    if doc.content.contains("Uniswap") {
                        Some("Uniswap".to_string())
                    } else if doc.content.contains("Aave") {
                        Some("Aave".to_string())
                    } else if doc.content.contains("Lido") {
                        Some("Lido".to_string())
                    } else if doc.content.contains("Compound") {
                        Some("Compound".to_string())
                    } else {
                        None
                    }
                })
                .collect();
            
            println!("  Protocols used: {:?}", protocols);
            
            // Analyze activity types
            let mut swaps = 0;
            let mut liquidity = 0;
            let mut staking = 0;
            
            for doc in &history.documents {
                if doc.content.contains("swapped") {
                    swaps += 1;
                }
                if doc.content.contains("liquidity") {
                    liquidity += 1;
                }
                if doc.content.contains("staked") {
                    staking += 1;
                }
            }
            
            println!("  Activity breakdown:");
            println!("    - Swaps: {}", swaps);
            println!("    - Liquidity provisions: {}", liquidity);
            println!("    - Staking operations: {}", staking);
        }
        Err(e) => {
            println!("  Error analyzing wallet: {}", e);
        }
    }

    Ok(())
}

/// Simulate a RAG agent answering complex queries
async fn simulate_rag_agent(memory: &GraphMemory) -> anyhow::Result<()> {
    println!("🤖 RAG Agent Simulation\n");
    
    let queries = vec![
        "What is the total DeFi activity for wallet 0x742d35Cc?",
        "Which protocols are most commonly used for borrowing?",
        "What are the recent liquidity provision events?",
        "Show me all staking activities in the knowledge graph",
    ];

    for query in queries {
        println!("User: {}", query);
        
        // Extract key terms from query (simplified - in production use NLP)
        let search_term = if query.contains("0x742d35Cc") {
            "0x742d35Cc"
        } else if query.contains("borrowing") || query.contains("borrowed") {
            "borrowed"
        } else if query.contains("liquidity") {
            "liquidity"
        } else if query.contains("staking") {
            "staked"
        } else {
            "DeFi"
        };
        
        // Retrieve relevant context from graph memory
        match memory.search(search_term, Some(10)).await {
            Ok(search_results) => {
                // Simulate RAG response generation
                let response = generate_rag_response(query, &search_results.documents);
                println!("Agent: {}\n", response);
            }
            Err(e) => {
                println!("Agent: I encountered an error retrieving information: {}\n", e);
            }
        }
    }

    Ok(())
}

/// Generate a simulated RAG response based on retrieved context
fn generate_rag_response(query: &str, context_docs: &[RawTextDocument]) -> String {
    if context_docs.is_empty() {
        return "I don't have any relevant information to answer that question.".to_string();
    }

    // Analyze the query type
    if query.contains("total") || query.contains("activity") {
        let count = context_docs.len();
        let protocols: Vec<&str> = context_docs
            .iter()
            .filter_map(|doc| {
                if doc.content.contains("Uniswap") {
                    Some("Uniswap")
                } else if doc.content.contains("Aave") {
                    Some("Aave")
                } else if doc.content.contains("Lido") {
                    Some("Lido")
                } else if doc.content.contains("Compound") {
                    Some("Compound")
                } else {
                    None
                }
            })
            .collect();
        
        format!(
            "Based on the knowledge graph, I found {} DeFi activities. The wallet has interacted with protocols including: {}. Recent activities include: {}",
            count,
            protocols.join(", "),
            context_docs.first().map(|d| &d.content[..]).unwrap_or("no recent activity")
        )
    } else if query.contains("commonly used") {
        let mut protocol_counts = std::collections::HashMap::new();
        for doc in context_docs {
            for protocol in ["Uniswap", "Aave", "Lido", "Compound"] {
                if doc.content.contains(protocol) {
                    *protocol_counts.entry(protocol).or_insert(0) += 1;
                }
            }
        }
        
        let most_common = protocol_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(protocol, _)| *protocol)
            .unwrap_or("Unknown");
        
        format!(
            "Based on the transaction history, {} appears to be the most commonly used protocol for this type of activity. I found {} relevant transactions in the knowledge graph.",
            most_common,
            context_docs.len()
        )
    } else if query.contains("recent") {
        let recent_activities: Vec<String> = context_docs
            .iter()
            .take(3)
            .map(|doc| {
                // Extract key information from content
                let parts: Vec<&str> = doc.content.split(" on ").collect();
                if parts.len() >= 2 {
                    format!("• {}", parts[0])
                } else {
                    format!("• {}", doc.content)
                }
            })
            .collect();
        
        format!(
            "Here are the recent activities from the knowledge graph:\n{}",
            recent_activities.join("\n")
        )
    } else {
        // Generic response with context summary
        let summary: Vec<String> = context_docs
            .iter()
            .take(3)
            .enumerate()
            .map(|(i, doc)| format!("{}. {}", i + 1, doc.content))
            .collect();
        
        format!(
            "Based on the knowledge graph, here's what I found:\n{}",
            summary.join("\n")
        )
    }
}

/// Helper function to display graph statistics
async fn display_graph_stats(memory: &GraphMemory) {
    println!("\n📊 Graph Statistics:");
    
    match memory.get_stats().await {
        Ok(stats) => {
            println!("  Total documents: {}", stats.document_count);
            println!("  Total entities: {}", stats.entity_count);
            println!("  Total relationships: {}", stats.relationship_count);
            
            println!("  Node breakdown:");
            println!("    - Wallets: {}", stats.wallet_count);
            println!("    - Tokens: {}", stats.token_count);
            println!("    - Protocols: {}", stats.protocol_count);
            println!("    - Transactions: {}", stats.transaction_count);
        }
        Err(e) => {
            println!("  ⚠️ Could not fetch statistics: {}", e);
        }
    }
}