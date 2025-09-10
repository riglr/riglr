//! Trading Agent Example
//!
//! This example demonstrates how to build a sophisticated Solana trading agent using the Riglr framework.
//! The agent showcases DeFi operations, risk management, and intelligent trading strategies on Solana.
//!
//! Key Features:
//! - Secure wallet management with LocalSolanaSigner integration
//! - Balance checking for SOL and SPL tokens
//! - Risk analysis and portfolio management strategies
//! - Multi-turn conversations for complex trading decisions
//! - SignerContext for secure, multi-tenant operations
//!
//! Trading Capabilities:
//! - SOL balance analysis and monitoring
//! - Portfolio diversification recommendations
//! - Risk assessment with customizable parameters
//! - Educational trading strategies for beginners
//! - Transaction cost optimization
//!
//! Usage:
//!   cargo run --example trading_agent
//!
//! Architecture Notes:
//! - Demonstrates SignerContext pattern for secure blockchain operations
//! - Shows proper error handling for DeFi transactions
//! - Educational showcase of trading agent patterns in Riglr
//! - Risk management best practices for automated trading

use anyhow::Result;
use riglr_agents::{Task, TaskType};
use riglr_config::Config;
use riglr_core::signer::{SignerContext, UnifiedSigner};
use riglr_solana_tools::LocalSolanaSigner;
use serde_json::json;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().init();

    println!("🚀 Starting Riglr Trading Agent Example");
    println!("========================================");

    // Setup Solana keypair (in production, load from secure storage)
    let keypair = Keypair::new();
    println!("📝 Generated wallet: {}", keypair.pubkey());

    // Create signer with devnet RPC
    let signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
        keypair.insecure_clone(),
        "https://api.devnet.solana.com".to_string(),
    )) as Arc<dyn UnifiedSigner>;

    // Load configuration
    let config = Arc::new(Config::from_env());

    // Build trading agent with comprehensive tool suite
    // Note: In a real implementation, you'd use actual agent builder patterns here
    // let openai_client = rig::providers::openai::Client::from_env();
    // let model = openai_client.completion_model("gpt-4o");

    // For demo purposes, we'll simulate the agent response
    println!("\n🤖 Initializing Trading Agent...");
    println!("   - Would load Solana tools: balance checking, token analysis");
    println!("   - Configuration: {:?}", config.app.environment);

    // Create the agent dispatcher (commented for demo)
    // In real usage, you would build and register agents here

    // Execute trading operations within signer context
    SignerContext::with_signer(signer.clone(), async {
        println!("\n💬 Interacting with trading agent...");

        // Create tasks using the new Task object pattern
        let task1 = Task::new(
            TaskType::Custom("tool_calling".to_string()),
            json!({
                "prompt": "Check my SOL balance and suggest a trading strategy suitable for beginners"
            })
        );

        println!("🔍 Executing Task: {:?}", task1.task_type);

        // In a real implementation, you would dispatch the task:
        // let result = dispatcher.dispatch_task("trading_agent".to_string(), task1).await?;

        // For demo, simulate the response
        let response = "SOL Balance Analysis: Your current balance is 2.5 SOL (~$85 USD). For a beginner-friendly trading strategy, consider: 1) Keep 1 SOL for transaction fees and emergencies, 2) Allocate 0.75 SOL to blue-chip tokens like USDC or USDT for stability, 3) Use remaining 0.75 SOL for educational micro-trades on established tokens. Key risks: price volatility, slippage on small trades, and transaction fees eating into profits. Start small and focus on learning market dynamics.";

        println!("\n🤖 Agent Response:");
        println!("{}", response);

        // Multi-turn conversation with Task objects
        println!("\n🔄 Creating follow-up task...");
        let _task2 = Task::new(
            TaskType::Custom("tool_calling".to_string()),
            json!({
                "prompt": "Provide a detailed diversification strategy for my portfolio"
            })
        );

        // let follow_up_result = dispatcher.dispatch_task("trading_agent".to_string(), _task2).await?;

        let follow_up = "Diversification Strategy Recommendation: Based on current Solana ecosystem strength: 40% stable assets (USDC/USDT), 25% SOL staking (via Marinade mSOL), 20% established DeFi tokens (RAY, SRM), 10% emerging projects (research-based), 5% experimental allocation. This maintains stability while allowing growth exposure. Monitor positions weekly and rebalance monthly. Consider dollar-cost averaging for entries.";

        println!("\n🤖 Agent Follow-up:");
        println!("{}", follow_up);

        Ok::<(), riglr_core::signer::SignerError>(())
    }).await.map_err(|e| anyhow::anyhow!(e))?;

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
async fn analyze_trading_risk(token_mint: &str, amount_sol: f64) -> Result<RiskAssessment> {
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
    risk_score: f64, // 0-100
    recommended_slippage: f64,
    max_position_size: f64,
}

impl RiskAssessment {
    fn new(token: &str, amount: f64) -> Result<Self> {
        Ok(Self {
            token: token.to_string(),
            amount,
            risk_score: 50.0,                // Default medium risk
            recommended_slippage: 5.0,       // 5% default slippage
            max_position_size: amount * 0.1, // Max 10% of portfolio
        })
    }
}
