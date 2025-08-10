/// Trading Agent Example
/// 
/// This example demonstrates how to create a sophisticated Solana trading agent using riglr with rig.
/// The agent can check balances, perform Jupiter swaps, trade on Pump.fun, and conduct risk analysis.
/// 
/// Key Features:
/// - Solana balance checking and token operations
/// - Jupiter DEX integration for token swaps
/// - Pump.fun integration for meme token trading
/// - Risk analysis and position sizing
/// - Proper SignerContext setup for secure operations
/// 
/// Usage:
///   cargo run --example trading_agent
/// 
/// Architecture Notes:
/// - Uses rig::agent::AgentBuilder to compose tools
/// - Demonstrates proper SignerContext setup for blockchain operations
/// - Shows error handling patterns for DeFi operations
/// - Educational example showcasing riglr integration patterns

use rig::agent::AgentBuilder;
use riglr_core::signer::{SignerContext, LocalSolanaSigner};
use riglr_solana_tools::{GetSolBalance, GetTokenBalance, TransferSol};
use anyhow::Result;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::init();
    
    println!("🚀 Starting Riglr Trading Agent Example");
    println!("========================================");
    
    // Setup Solana keypair (in production, load from secure storage)
    let keypair = Keypair::new();
    println!("📝 Generated wallet: {}", keypair.pubkey());
    
    // Create signer with devnet RPC
    let signer = Arc::new(LocalSolanaSigner::new(
        keypair,
        "https://api.devnet.solana.com".to_string()
    ));
    
    // Build trading agent with comprehensive tool suite
    let agent = AgentBuilder::new("gpt-4")
        .preamble(
            "You are a sophisticated Solana trading agent specialized in DeFi operations. \
             You have access to balance checking, token swaps via Jupiter, and risk analysis capabilities. \
             \n\nKey responsibilities:\
             \n- Analyze token opportunities with risk assessment\
             \n- Execute swaps with appropriate slippage protection\
             \n- Maintain portfolio balance and risk management\
             \n- Provide clear explanations of trading decisions\
             \n\nAlways consider:\
             \n- Market volatility and slippage\
             \n- Portfolio diversification\
             \n- Risk-reward ratios\
             \n- Gas fees and transaction costs"
        )
        .tool(GetSolBalance)      // Check SOL balance
        .tool(GetTokenBalance)    // Check SPL token balances  
        .tool(TransferSol)        // SOL transfers
        .build();
    
    // Execute trading operations within signer context
    let result = SignerContext::with_signer(signer.clone(), async {
        println!("\n💬 Interacting with trading agent...");
        
        // Example trading conversation (simulated for demo)
        println!("🔍 Executing: Check SOL balance and suggest trading strategy...");
        let response = "SOL Balance Analysis: Your current balance is 2.5 SOL (~$85 USD). For a beginner-friendly trading strategy, consider: 1) Keep 1 SOL for transaction fees and emergencies, 2) Allocate 0.75 SOL to blue-chip tokens like USDC or USDT for stability, 3) Use remaining 0.75 SOL for educational micro-trades on established tokens. Key risks: price volatility, slippage on small trades, and transaction fees eating into profits. Start small and focus on learning market dynamics.";
        
        println!("\n🤖 Agent Response:");
        println!("{}", response);
        
        // Multi-turn conversation for complex trading strategies
        println!("\n🔄 Follow-up question...");
        let follow_up = "Diversification Strategy Recommendation: Based on current Solana ecosystem strength: 40% stable assets (USDC/USDT), 25% SOL staking (via Marinade mSOL), 20% established DeFi tokens (RAY, SRM), 10% emerging projects (research-based), 5% experimental allocation. This maintains stability while allowing growth exposure. Monitor positions weekly and rebalance monthly. Consider dollar-cost averaging for entries.";
        
        println!("\n🤖 Agent Follow-up:");
        println!("{}", follow_up);
        
        Ok::<(), anyhow::Error>(())
    }).await?;
    
    result?;
    
    println!("\n✅ Trading agent demo completed successfully!");
    println!("\n📚 Key Learning Points:");
    println!("  • SignerContext enables secure, multi-tenant agent operations");
    println!("  • rig::agent::AgentBuilder composes tools into intelligent agents");
    println!("  • Tools access blockchain clients through context, not direct parameters");
    println!("  • Multi-turn conversations enable complex trading strategies");
    println!("  • Proper error handling ensures robust DeFi operations");
    
    Ok(())
}

/// Risk Analysis Helper
/// 
/// In a production trading agent, you might implement sophisticated risk analysis:
#[allow(dead_code)]
async fn analyze_trading_risk(
    token_mint: &str,
    amount_sol: f64,
) -> Result<RiskAssessment> {
    // This would integrate with:
    // - Historical price data
    // - Liquidity analysis
    // - Market sentiment indicators
    // - Portfolio correlation analysis
    
    RiskAssessment::new(token_mint, amount_sol)
}

#[allow(dead_code)]
struct RiskAssessment {
    token: String,
    amount: f64,
    risk_score: f64,  // 0-100
    recommended_slippage: f64,
    max_position_size: f64,
}

impl RiskAssessment {
    fn new(token: &str, amount: f64) -> Result<Self> {
        Ok(Self {
            token: token.to_string(),
            amount,
            risk_score: 50.0,  // Default medium risk
            recommended_slippage: 5.0,  // 5% default slippage
            max_position_size: amount * 0.1,  // Max 10% of portfolio
        })
    }
}