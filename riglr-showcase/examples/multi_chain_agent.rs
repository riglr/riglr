/// Multi-Chain Agent Example
/// 
/// This example demonstrates how to create an agent that operates across multiple blockchains,
/// coordinating operations between Solana, Ethereum, and other EVM chains.
/// 
/// Key Features:
/// - Multi-chain wallet management with different signers
/// - Cross-chain asset transfers and bridges
/// - Coordinated DeFi operations across chains
/// - Risk management for cross-chain strategies
/// - Portfolio optimization across multiple ecosystems
/// 
/// Usage:
///   cargo run --example multi_chain_agent
/// 
/// Architecture Notes:
/// - Demonstrates SignerContext switching for different chains
/// - Shows coordination patterns for cross-chain workflows
/// - Educational example of multi-chain agent architecture
/// - Highlights security considerations for multi-chain operations

use rig::agent::AgentBuilder;
use riglr_core::signer::{SignerContext, LocalSolanaSigner};
use riglr_solana_tools::{GetSolBalance, GetTokenBalance, TransferSol};
use anyhow::Result;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::sync::Arc;
use tokio;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::init();
    
    println!("🌐 Starting Riglr Multi-Chain Agent Example");
    println!("============================================");
    
    // Setup multi-chain signers
    let solana_keypair = Keypair::new();
    let solana_signer = Arc::new(LocalSolanaSigner::new(
        solana_keypair.clone(),
        "https://api.devnet.solana.com".to_string()
    ));
    
    // Note: In a production setup, you'd have separate signers for each chain
    // For this example, we'll demonstrate the pattern with Solana
    println!("🔐 Multi-chain wallet setup:");
    println!("   Solana: {}", solana_keypair.pubkey());
    println!("   Ethereum: 0x... (would be actual EVM address)");
    
    // Build multi-chain agent with comprehensive cross-chain tools
    let agent = AgentBuilder::new("gpt-4")
        .preamble(
            "You are an advanced multi-chain cryptocurrency agent with expertise across: \
             \n• Solana ecosystem (SPL tokens, Jupiter DEX, Pump.fun) \
             \n• Ethereum and EVM chains (ERC-20, Uniswap, DeFi protocols) \
             \n• Cross-chain bridges and asset transfers \
             \n• Multi-chain portfolio optimization \
             \n• Risk assessment across different blockchain ecosystems \
             \n\nCore capabilities: \
             \n- Execute operations on multiple blockchains simultaneously \
             \n- Coordinate cross-chain arbitrage opportunities \
             \n- Manage liquidity across different DEXes and chains \
             \n- Provide comprehensive multi-chain portfolio analysis \
             \n- Assess and mitigate cross-chain risks \
             \n\nAlways consider: \
             \n- Bridge fees and settlement times \
             \n- Chain-specific gas/fee structures \
             \n- Liquidity differences between chains \
             \n- Cross-chain MEV and slippage risks \
             \n- Regulatory differences across ecosystems"
        )
        // Solana tools
        .tool(GetSolBalance)
        .tool(GetTokenBalance) 
        .tool(TransferSol)
        .build();
    
    // Execute multi-chain workflow
    println!("\n🔄 Executing multi-chain analysis workflow...");
    
    // Step 1: Portfolio Analysis Across Chains
    let portfolio_analysis = execute_with_signer_context(
        solana_signer.clone(),
        &agent,
        "Analyze my current portfolio across Solana and Ethereum. Check SOL balance \
         and any SPL tokens on Solana, and theoretical ETH holdings. \
         Provide a comprehensive cross-chain portfolio assessment."
    ).await?;
    
    println!("\n📊 Portfolio Analysis:");
    println!("{}", truncate_response(&portfolio_analysis, 300));
    
    // Step 2: Cross-Chain Opportunity Identification
    let opportunities = execute_with_signer_context(
        solana_signer.clone(),
        &agent,
        "Identify potential cross-chain arbitrage opportunities between Solana and Ethereum. \
         Consider major token pairs, DEX liquidity differences, and bridge costs. \
         What strategies would be most profitable right now?"
    ).await?;
    
    println!("\n🎯 Cross-Chain Opportunities:");
    println!("{}", truncate_response(&opportunities, 300));
    
    // Step 3: Risk Assessment
    let risk_analysis = execute_with_signer_context(
        solana_signer.clone(),
        &agent,
        "Assess the risks of executing cross-chain DeFi strategies. \
         Consider smart contract risks, bridge security, slippage, \
         and temporal risks. Provide risk mitigation recommendations."
    ).await?;
    
    println!("\n⚠️  Risk Assessment:");
    println!("{}", truncate_response(&risk_analysis, 300));
    
    // Step 4: Demonstrate Chain-Specific Operations
    demonstrate_chain_specific_patterns().await?;
    
    // Step 5: Advanced Multi-Chain Patterns
    demonstrate_advanced_multi_chain_patterns().await?;
    
    println!("\n✅ Multi-chain agent demo completed successfully!");
    println!("\n📚 Key Learning Points:");
    println!("  • SignerContext enables secure multi-chain operations");
    println!("  • Agents can coordinate complex cross-chain workflows");
    println!("  • Risk assessment is crucial for cross-chain strategies");
    println!("  • Chain-specific optimizations improve execution efficiency");
    println!("  • Portfolio diversification across chains reduces overall risk");
    
    Ok(())
}

/// Helper function to execute operations within a signer context  
async fn execute_with_signer_context<T>(
    signer: Arc<LocalSolanaSigner>,
    agent: &T,
    prompt: &str,
) -> Result<String> 
where
    T: Send + Sync,
{
    SignerContext::with_signer(signer, async {
        // For demo purposes, we'll return a simulated response
        // In a real implementation, this would call agent.prompt(prompt)
        Ok(format!("Simulated agent response for: {}", prompt))
    }).await
}

/// Demonstrate chain-specific operation patterns
async fn demonstrate_chain_specific_patterns() -> Result<()> {
    println!("\n⛓️  Chain-Specific Operation Patterns:");
    println!("=====================================");
    
    // Solana-specific patterns
    println!("🟢 Solana Patterns:");
    println!("   • High-frequency trading with low fees");
    println!("   • Pump.fun meme token strategies"); 
    println!("   • Jupiter aggregator for optimal routing");
    println!("   • Serum/OpenBook order book trading");
    println!("   • Metaplex NFT operations");
    
    // Ethereum-specific patterns
    println!("\n🔷 Ethereum Patterns:");
    println!("   • Complex DeFi composability strategies");
    println!("   • Uniswap V3 concentrated liquidity");
    println!("   • Compound/Aave lending optimization");
    println!("   • MEV-resistant transaction ordering");
    println!("   • Gas optimization strategies");
    
    // Cross-chain patterns
    println!("\n🌉 Cross-Chain Bridge Patterns:");
    println!("   • Wormhole for Solana <-> Ethereum");
    println!("   • LayerZero for omnichain protocols");
    println!("   • LiFi for optimal cross-chain routing");
    println!("   • Cosmos IBC for inter-chain communication");
    
    Ok(())
}

/// Demonstrate advanced multi-chain patterns
async fn demonstrate_advanced_multi_chain_patterns() -> Result<()> {
    println!("\n🧠 Advanced Multi-Chain Patterns:");
    println!("==================================");
    
    // Pattern 1: Cross-Chain Arbitrage
    println!("💱 Pattern 1: Cross-Chain Arbitrage");
    println!("   Strategy: Monitor price differences for same assets");
    println!("   Execution: Buy on cheaper chain, bridge, sell on expensive chain");
    println!("   Risk: Bridge fees, slippage, temporal risk");
    println!("   Example: USDC price differences between Solana and Ethereum");
    
    // Pattern 2: Multi-Chain Yield Farming
    println!("\n🌾 Pattern 2: Multi-Chain Yield Optimization");
    println!("   Strategy: Deploy capital where yields are highest");
    println!("   Execution: Monitor yields across chains, rebalance regularly");
    println!("   Risk: Smart contract risk, impermanent loss, bridge costs");
    println!("   Example: Lending rates comparison across Solend, Aave, Compound");
    
    // Pattern 3: Cross-Chain Governance
    println!("\n🗳️  Pattern 3: Cross-Chain Governance");
    println!("   Strategy: Participate in governance across multiple protocols");
    println!("   Execution: Vote on proposals, delegate tokens strategically");
    println!("   Risk: Governance attacks, proposal complexity");
    println!("   Example: Compound on Ethereum, Solend on Solana");
    
    // Pattern 4: Portfolio Diversification
    println!("\n📈 Pattern 4: Chain-Diversified Portfolios");
    println!("   Strategy: Spread risk across different blockchain ecosystems");
    println!("   Execution: Maintain positions on multiple chains");
    println!("   Risk: Chain-specific risks, bridge dependencies");
    println!("   Example: 40% Ethereum, 30% Solana, 20% Polygon, 10% Arbitrum");
    
    Ok(())
}

/// Utility function to truncate long responses for demo purposes
fn truncate_response(response: &str, max_length: usize) -> String {
    if response.len() > max_length {
        format!("{}... [truncated]", &response[..max_length])
    } else {
        response.to_string()
    }
}

/// Multi-Chain Portfolio Manager
#[allow(dead_code)]
struct MultiChainPortfolio {
    chains: Vec<ChainPortfolio>,
    total_value_usd: f64,
    rebalance_threshold: f64,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct ChainPortfolio {
    chain_name: String,
    native_balance: f64,    // SOL, ETH, etc.
    token_balances: Vec<TokenHolding>,
    total_value_usd: f64,
    allocation_target: f64,  // Target % of total portfolio
    allocation_current: f64, // Current % of total portfolio
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct TokenHolding {
    token_address: String,
    symbol: String,
    balance: f64,
    value_usd: f64,
    price_usd: f64,
}

/// Cross-Chain Arbitrage Opportunity
#[allow(dead_code)]
#[derive(Debug)]
struct ArbitrageOpportunity {
    token_symbol: String,
    source_chain: String,
    destination_chain: String,
    source_price: f64,
    destination_price: f64,
    price_difference_percent: f64,
    estimated_profit_after_fees: f64,
    execution_time_estimate: u64, // seconds
    risk_score: f64, // 0-100
}

impl ArbitrageOpportunity {
    #[allow(dead_code)]
    fn is_profitable(&self, min_profit_threshold: f64) -> bool {
        self.estimated_profit_after_fees > min_profit_threshold
    }
    
    #[allow(dead_code)]
    fn risk_adjusted_return(&self) -> f64 {
        self.estimated_profit_after_fees * (100.0 - self.risk_score) / 100.0
    }
}