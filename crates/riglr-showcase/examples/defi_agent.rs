//! `DeFi` Agent Example
//!
//! This example demonstrates how to build an advanced `DeFi` (Decentralized Finance) agent
//! using the Riglr framework. The agent showcases sophisticated yield optimization,
//! risk management, and multi-protocol strategies across various `DeFi` protocols.
//!
//! Key Features:
//! - Portfolio analysis and yield optimization strategies
//! - Risk assessment and mitigation techniques  
//! - Multi-protocol `DeFi` patterns (yield farming, lending, staking)
//! - Advanced strategies like delta-neutral farming and auto-compounding
//! - MEV protection and gas optimization
//! - Cross-protocol arbitrage and strategy rotation
//!
//! Supported `DeFi` Protocols:
//! - Solana: Jupiter, Solend, Raydium, Marinade, Tulip, Saber
//! - Ethereum: Uniswap V3, Aave, Compound, Curve, Convex, Yearn
//!
//! The agent demonstrates best practices for:
//! - Capital preservation and diversification
//! - Risk-adjusted return optimization
//! - Automated rebalancing and compounding
//! - Smart contract risk assessment

use riglr_config::SolanaNetworkConfig;
use riglr_core::signer::{SignerContext, SignerError, UnifiedSigner};
use riglr_solana_tools::signer::Local as LocalSolanaSigner;
// TODO: Re-enable when rig tools are updated
// use riglr_solana_tools::{get_sol_balance, get_spl_token_balance};
use anyhow::Result;
use core::cmp::Ordering;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().init();

    println!("🏦 Starting Riglr DeFi Agent Example");
    println!("====================================");

    // Setup DeFi-focused signer
    let keypair = Keypair::new();
    let network_config =
        SolanaNetworkConfig::new("mainnet", "https://api.mainnet-beta.solana.com".to_string());
    let signer = Arc::new(LocalSolanaSigner::from_keypair(
        keypair.insecure_clone(),
        network_config,
    )) as Arc<dyn UnifiedSigner>;

    println!("💼 DeFi Portfolio Manager initialized");
    println!("🔐 Wallet: {}", keypair.pubkey());

    // Build sophisticated DeFi agent
    // TODO: Re-enable when rig provider API is updated
    // let agent = AgentBuilder::new("gpt-4")
    //     .preamble(
    //         "You are an advanced DeFi yield optimization agent with deep expertise in: \
    //          \n• Yield farming strategies and protocol analysis \
    //          \n• Liquidity provision and impermanent loss management \
    //          \n• Lending and borrowing optimization \
    //          \n• DeFi protocol security assessment \
    //          \n• MEV protection and transaction optimization \
    //          \n• Risk management and position sizing \
    //          \n\nKey DeFi Protocols (Solana): \
    //          \n- Jupiter (DEX aggregation) \
    //          \n- Solend (lending/borrowing) \
    //          \n- Raydium (AMM + liquidity) \
    //          \n- Marinade (liquid staking) \
    //          \n- Tulip Protocol (yield farming) \
    //          \n- Saber (stable swaps) \
    //          \n\nKey DeFi Protocols (Ethereum): \
    //          \n- Uniswap V3 (concentrated liquidity) \
    //          \n- Aave (lending/borrowing) \
    //          \n- Compound (money markets) \
    //          \n- Curve (stable/exotic pairs) \
    //          \n- Convex (Curve boost) \
    //          \n- Yearn (vault strategies) \
    //          \n\nStrategy Framework: \
    //          \n1. Assess current market conditions and yields \
    //          \n2. Identify optimal allocation strategies \
    //          \n3. Calculate risk-adjusted returns \
    //          \n4. Execute transactions with MEV protection \
    //          \n5. Monitor and rebalance positions \
    //          \n\nAlways prioritize: \
    //          \n- Capital preservation over maximum yield \
    //          \n- Diversification across protocols and strategies \
    //          \n- Gas/fee optimization \
    //          \n- Smart contract risk assessment \
    //          \n- Liquidity and exit strategy planning"
    //     )
    //     // Core DeFi tools
    //     .tool(get_sol_balance)
    //     .tool(get_spl_token_balance)
    //     .max_tokens(3000)
    //     .build();

    // Execute comprehensive DeFi strategy workflow
    SignerContext::with_signer(signer.clone(), async {

        // Step 1: Portfolio Assessment & Analysis
        println!("\n📊 Step 1: Portfolio Analysis & Current Positions");
        // TODO: Re-enable when agent is restored
        let portfolio_analysis = "DeFi Portfolio Analysis: Current SOL balance 2.8 SOL (~$95 USD). Detected SPL tokens: 200 USDC, 50 RAY. Market conditions favor liquid staking (6.2% APY) and stable farming (8-12% APY). Recommended strategy: 40% Marinade liquid staking, 30% USDC-SOL LP on Raydium (14% APY), 20% Solend lending (5.8% APY), 10% emergency reserves.";

        println!("Portfolio Analysis: {}", truncate_response(portfolio_analysis, 400));

        // Step 2: Yield Optimization Strategy
        println!("\n🌾 Step 2: Yield Optimization Strategy");
        // TODO: Re-enable when agent is restored
        let yield_strategy = "Comprehensive Yield Strategy: 1) Marinade liquid staking: 1.12 SOL (40%, 6.2% APY, low risk), 2) USDC-SOL LP on Raydium: 150 USDC + 0.84 SOL (30%, 14% APY, medium risk), 3) Solend USDC lending: 50 USDC (20%, 5.8% APY, low risk), 4) Emergency reserve: 0.84 SOL (10%). Expected blended APY: 8.9%. Rebalance monthly. Monitor impermanent loss on LP positions.";

        println!("Yield Strategy: {}", truncate_response(yield_strategy, 400));

        // Step 3: Risk Assessment & Management
        println!("\n⚠️  Step 3: Risk Assessment & Mitigation");
        // TODO: Re-enable when agent is restored
        let risk_assessment = "Risk Assessment: Marinade (Risk: 3/10, audited, insurance fund), USDC-SOL LP (Risk: 6/10, impermanent loss up to 8% in volatile markets), Solend (Risk: 4/10, overcollateralized, liquidation buffer). Mitigation: Diversify across 3+ protocols, monitor positions daily, set 5% stop-loss on LP positions, maintain 15% cash buffer for opportunities. Overall portfolio risk: 4.2/10 (moderate).";

        println!("Risk Assessment: {}", truncate_response(risk_assessment, 400));

        // Step 4: Advanced DeFi Strategies
        println!("\n🧠 Step 4: Advanced Strategy Implementation");
        // TODO: Re-enable when agent is restored
        let advanced_strategies = "Advanced DeFi Strategies: 1) Delta-neutral: Long SOL spot + Short SOL-PERP on Mango (market-neutral yield), 2) Cross-protocol arb: Monitor USDC rates across Solend/Tulip (0.2-0.5% spreads), 3) Leveraged farming: Borrow USDC on Solend, farm USDC-SOL LP (leverage 1.5x max), 4) Auto-compound: Raydium rewards → swap 50% to USDC → re-LP weekly, 5) MEV protection: Use private mempool via Jito for large swaps.";

        println!("Advanced Strategies: {}", truncate_response(advanced_strategies, 400));

        Ok::<(), Box<dyn SignerError>>(())
    }).await.map_err(|e| anyhow::anyhow!(e))?;

    // Demonstrate DeFi patterns and best practices
    demonstrate_defi_patterns();
    demonstrate_yield_optimization();
    demonstrate_risk_management();

    // Demonstrate yield optimization functionality
    println!("\n🔧 Demonstrating Yield Optimization:");
    let strategies = vec![
        YieldStrategy {
            name: "Solend Lending".to_string(),
            protocols: vec!["solend".to_string()],
            expected_apy: 8.5,
            risk_score: 3.0,
            min_amount_usd: 100.0,
            lock_period_days: None,
            auto_compound: true,
        },
        YieldStrategy {
            name: "Raydium LP".to_string(),
            protocols: vec!["raydium".to_string()],
            expected_apy: 15.2,
            risk_score: 7.0,
            min_amount_usd: 500.0,
            lock_period_days: Some(30),
            auto_compound: false,
        },
    ];

    let optimizer = YieldOptimizer {
        strategies,
        rebalance_threshold: 5.0,
        max_gas_ratio: 2.0,
    };

    if let Some(optimal_strategy) = optimizer.find_optimal_strategy(1000.0, 5.0) {
        println!("  💰 Optimal strategy found: {}", optimal_strategy.name);
        println!("  📊 Expected APY: {:.1}%", optimal_strategy.expected_apy);
        println!("  ⚖️ Risk Score: {:.1}/10", optimal_strategy.risk_score);
    } else {
        println!("  ❌ No suitable strategy found for given criteria");
    }

    println!("\n✅ DeFi agent demo completed successfully!");
    println!("\n📚 Key Learning Points:");
    println!("  • DeFi agents can orchestrate complex multi-protocol strategies");
    println!("  • Risk assessment is crucial for automated DeFi operations");
    println!("  • Yield optimization requires continuous monitoring and rebalancing");
    println!("  • MEV protection and gas optimization are essential for profitability");
    println!("  • Diversification across protocols reduces smart contract risk");

    Ok(())
}

/// Demonstrate core `DeFi` patterns
fn demonstrate_defi_patterns() {
    println!("\n💡 Core DeFi Patterns:");
    println!("======================");

    // Yield Farming Pattern
    println!("🌾 Pattern 1: Yield Farming");
    println!("   Strategy: Provide liquidity to earn farming rewards");
    println!("   Implementation: LP tokens → farming contract → rewards");
    println!("   Risks: Impermanent loss, farming token price volatility");
    println!("   Example: Raydium USDC-SOL LP farming");

    // Lending/Borrowing Pattern
    println!("\n🏦 Pattern 2: Lending & Borrowing");
    println!("   Strategy: Lend assets for interest, borrow for leverage");
    println!("   Implementation: Deposit collateral → borrow → reinvest");
    println!("   Risks: Liquidation, interest rate volatility");
    println!("   Example: Solend SOL collateral → borrow USDC → buy more SOL");

    // Liquid Staking Pattern
    println!("\n💧 Pattern 3: Liquid Staking");
    println!("   Strategy: Stake native tokens while maintaining liquidity");
    println!("   Implementation: SOL → mSOL/stSOL → use in DeFi");
    println!("   Risks: Slashing, liquid staking token depeg");
    println!("   Example: Marinade mSOL in Solend as collateral");

    // Delta-Neutral Pattern
    println!("\n⚖️  Pattern 4: Delta-Neutral Farming");
    println!("   Strategy: Farm yield without directional price exposure");
    println!("   Implementation: LP position + short hedge");
    println!("   Risks: Basis risk, funding costs");
    println!("   Example: SOL-USDC LP + SOL perp short on Mango");
}

/// Demonstrate yield optimization strategies
fn demonstrate_yield_optimization() {
    println!("\n📈 Yield Optimization Strategies:");
    println!("=================================");

    // Strategy 1: Auto-Compounding
    println!("🔄 Strategy 1: Auto-Compounding");
    println!("   Mechanism: Automatically reinvest rewards to maximize returns");
    println!("   Frequency: Optimal based on gas costs vs reward value");
    println!("   Example: Harvest TULIP rewards → swap to underlying → re-deposit");

    // Strategy 2: Yield Laddering
    println!("\n🪜 Strategy 2: Yield Laddering");
    println!("   Mechanism: Stagger positions across different lockup periods");
    println!("   Benefit: Balance between yield and liquidity");
    println!("   Example: Split SOL across 1-week, 1-month, 3-month stakes");

    // Strategy 3: Protocol Rotation
    println!("\n🔄 Strategy 3: Protocol Rotation");
    println!("   Mechanism: Move capital to highest-yielding protocols");
    println!("   Frequency: Weekly rebalancing based on yield changes");
    println!("   Example: Migrate between Solend, Tulip, Francium based on APY");

    // Strategy 4: Risk-Adjusted Optimization
    println!("\n📊 Strategy 4: Risk-Adjusted Yield");
    println!("   Mechanism: Optimize for Sharpe ratio, not just APY");
    println!("   Formula: (Expected Return - Risk-Free Rate) / Volatility");
    println!("   Example: 15% APY with 5% volatility > 25% APY with 40% volatility");
}

/// Demonstrate risk management techniques
fn demonstrate_risk_management() {
    println!("\n🛡️  Risk Management Techniques:");
    println!("===============================");

    // Risk 1: Smart Contract Risk
    println!("🔒 Smart Contract Risk Mitigation:");
    println!("   • Diversify across multiple audited protocols");
    println!("   • Monitor TVL and protocol maturity");
    println!("   • Set maximum exposure limits per protocol");
    println!("   • Use time-locks and multi-sig where possible");

    // Risk 2: Impermanent Loss
    println!("\n📉 Impermanent Loss Protection:");
    println!("   • Choose correlated token pairs (e.g., ETH-stETH)");
    println!("   • Monitor price divergence actively");
    println!("   • Use impermanent loss insurance when available");
    println!("   • Calculate break-even farming duration");

    // Risk 3: Liquidation Risk
    println!("\n💥 Liquidation Risk Management:");
    println!("   • Maintain healthy collateralization ratios (>150%)");
    println!("   • Set up automated collateral top-ups");
    println!("   • Monitor liquidation prices actively");
    println!("   • Use stop-loss mechanisms for leveraged positions");

    // Risk 4: Market Risk
    println!("\n📊 Market Risk Hedging:");
    println!("   • Use delta-neutral strategies");
    println!("   • Implement position sizing based on volatility");
    println!("   • Maintain emergency exit strategies");
    println!("   • Hedge with derivatives when appropriate");
}

/// Utility function for response truncation
fn truncate_response(response: &str, max_length: usize) -> String {
    if response.len() > max_length {
        format!(
            "{}... [View full response in production]",
            &response[..max_length]
        )
    } else {
        response.to_string()
    }
}

/// `DeFi` Portfolio Manager Structure
#[derive(Debug, Serialize, Deserialize)]
struct DeFiPortfolio {
    total_value_usd: f64,
    positions: Vec<DeFiPosition>,
    risk_metrics: RiskMetrics,
    yield_summary: YieldSummary,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeFiPosition {
    protocol: String,
    position_type: PositionType,
    token_a: String,
    token_b: Option<String>, // For LP positions
    amount_usd: f64,
    apy: f64,
    risk_score: f64, // 1-10
    days_until_unlock: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
enum PositionType {
    Lending,
    Borrowing,
    LiquidityProvision,
    YieldFarming,
    Staking,
    LeveragedFarming,
}

#[derive(Debug, Serialize, Deserialize)]
struct RiskMetrics {
    portfolio_risk_score: f64, // 1-10
    max_drawdown_30d: f64,
    sharpe_ratio: f64,
    protocol_concentration: HashMap<String, f64>, // protocol -> % allocation
    impermanent_loss_exposure: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct YieldSummary {
    weighted_avg_apy: f64,
    total_rewards_24h: f64,
    total_rewards_7d: f64,
    total_rewards_30d: f64,
    next_compound_date: chrono::DateTime<chrono::Utc>,
}

/// Yield Strategy Optimizer
struct YieldOptimizer {
    strategies: Vec<YieldStrategy>,
    #[expect(dead_code)] // Used in real implementation for rebalancing logic
    rebalance_threshold: f64, // % difference to trigger rebalance
    #[expect(dead_code)] // Used in real implementation for gas optimization
    max_gas_ratio: f64, // Max gas as % of position value
}

#[derive(Debug)]
struct YieldStrategy {
    name: String,
    #[expect(dead_code)] // Used in real implementation for protocol-specific logic
    protocols: Vec<String>,
    expected_apy: f64,
    risk_score: f64,
    min_amount_usd: f64,
    #[expect(dead_code)] // Used in real implementation for lock period calculations
    lock_period_days: Option<u32>,
    #[expect(dead_code)] // Used in real implementation for auto-compounding logic
    auto_compound: bool,
}

impl YieldOptimizer {
    fn find_optimal_strategy(
        &self,
        amount_usd: f64,
        risk_tolerance: f64,
    ) -> Option<&YieldStrategy> {
        self.strategies
            .iter()
            .filter(|s| s.risk_score <= risk_tolerance && s.min_amount_usd <= amount_usd)
            .max_by(|a, b| {
                let a_score = a.expected_apy / a.risk_score;
                let b_score = b.expected_apy / b.risk_score;
                a_score.partial_cmp(&b_score).unwrap_or(Ordering::Equal)
            })
    }
}
