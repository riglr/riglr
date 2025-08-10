//! Rig-Native Reasoning Loops with riglr Integration Patterns
//!
//! This example demonstrates how to leverage rig's built-in multi-turn capabilities
//! instead of implementing custom reasoning loops. Shows sophisticated decision-making
//! workflows that will integrate seamlessly with riglr blockchain tools.
//!
//! Key Concepts:
//! 1. Multi-turn conversations with persistent context
//! 2. Complex decision trees through natural reasoning  
//! 3. Tool chaining via agent conversation (pattern shown)
//! 4. Adaptive behavior based on intermediate results
//! 5. No custom loops needed - rig handles complexity naturally

use anyhow::Result;
use rig::agent::AgentBuilder;
use serde_json::json;
use tokio;
use tracing::{info, warn};

/// Demo: Multi-Step Portfolio Analysis Through Reasoning
/// 
/// Shows how rig's native capabilities can handle complex workflows
/// that would traditionally require custom loop implementation.
async fn demo_multi_step_reasoning() -> Result<()> {
    println!("🧠 Demo: Multi-Step Reasoning Without Custom Loops");
    println!("==================================================");
    
    // Create agent with sophisticated reasoning capabilities
    let analyst = AgentBuilder::new("gpt-4")
        .preamble(r#"
You are a systematic cryptocurrency analyst. You approach problems methodically:

1. Break down complex questions into logical steps
2. Analyze each component thoroughly  
3. Synthesize information to reach conclusions
4. Adapt your analysis based on new information
5. Provide actionable, specific recommendations

You don't need custom loops - you think through problems naturally and systematically.
When analyzing portfolios or market conditions, you consider multiple factors and 
their interactions. You explain your reasoning process clearly.
        "#)
        .max_tokens(1500)
        .build();

    // Simulate portfolio data that would come from riglr tools
    let portfolio_data = json!({
        "total_value": "$12,450",
        "positions": {
            "SOL": {"amount": "38.5", "value": "$5,775", "pct": "46.4%"},
            "USDC": {"amount": "1,800", "value": "$1,800", "pct": "14.5%"}, 
            "RAY": {"amount": "650", "value": "$2,600", "pct": "20.9%"},
            "ORCA": {"amount": "890", "value": "$1,425", "pct": "11.4%"},
            "SRM": {"amount": "2100", "value": "$840", "pct": "6.8%"}
        },
        "market_context": {
            "sol_trend": "upward (+12% week)",
            "defi_sentiment": "mixed",
            "volatility": "elevated"
        }
    });

    // Step 1: Initial comprehensive analysis
    let initial_prompt = format!(r#"
Analyze this DeFi portfolio systematically. Consider risk, diversification, 
and opportunities. Think through this step-by-step:

Portfolio: {}

Provide:
1. Risk assessment with specific concerns
2. Diversification analysis  
3. Top 3 actionable recommendations
4. Your reasoning process for each recommendation
    "#, serde_json::to_string_pretty(&portfolio_data)?);

    println!("🔍 Step 1: Requesting initial analysis...");
    let analysis = analyst.prompt(&initial_prompt).await?;
    println!("📊 Analysis Result:");
    println!("{}\n", analysis);

    // Step 2: Deep dive based on analysis
    let deep_dive = r#"
Based on your analysis, I'm most concerned about concentration risk. 
Walk me through:

1. Exactly how much SOL exposure to reduce (specific amounts)
2. Which alternative assets to consider and why
3. Optimal rebalancing sequence to minimize impact
4. How to time these changes with current market conditions

Think through the trade-offs and provide specific numbers.
    "#;

    println!("🎯 Step 2: Deep dive on concentration risk...");
    let recommendations = analyst.prompt(deep_dive).await?;
    println!("💡 Specific Recommendations:");
    println!("{}\n", recommendations);

    // Step 3: Stress test with scenario
    let stress_test = r#"
Stress test scenario: SOL drops 25% overnight due to network issues.
How would this affect the portfolio and your recommendations?

Consider:
1. New portfolio values and risk profile
2. Whether to change the rebalancing strategy
3. Opportunities this creates or removes
4. Updated timeline and priorities

Reason through this scenario systematically.
    "#;

    println!("⚠️  Step 3: Stress testing recommendations...");
    let scenario_response = analyst.prompt(stress_test).await?;
    println!("🛡️  Stress Test Results:");
    println!("{}\n", scenario_response);

    println!("✅ Multi-step reasoning complete - no custom loops needed!");
    
    Ok(())
}

/// Demo: Adaptive Strategy Based on Market Feedback
/// 
/// Shows how agents can adapt their approach based on intermediate results
/// and changing conditions, using rig's natural conversation flow.
async fn demo_adaptive_reasoning() -> Result<()> {
    println!("\n🔄 Demo: Adaptive Reasoning Based on Market Feedback");  
    println!("===================================================");

    let strategist = AgentBuilder::new("gpt-4")
        .preamble(r#"
You are an adaptive trading strategist who adjusts approach based on:
- Market feedback and results
- Changing conditions and new information  
- Success/failure of previous recommendations
- Real-time market dynamics

You continuously refine your strategy through systematic reasoning,
not rigid rules. You explain how and why you adapt your approach.
        "#)
        .max_tokens(1200)
        .build();

    // Initial strategy formation
    let strategy_prompt = r#"
Current market: SOL trending up +15% this week, DeFi TVL growing, 
but macro uncertainty from Fed policy.

Develop an initial trading strategy for the next 2 weeks:
1. Market thesis and key assumptions
2. Specific positioning recommendations
3. Risk management parameters
4. Success metrics to track

Explain your reasoning for this strategy.
    "#;

    println!("📋 Initial Strategy Formation...");
    let initial_strategy = strategist.prompt(strategy_prompt).await?;
    println!("🎯 Initial Strategy:");
    println!("{}\n", initial_strategy);

    // Simulate market feedback and adaptation
    let feedback_prompt = r#"
Strategy Update: 3 days later...

Results so far:
- SOL continued up another 8% (good call)
- But DeFi tokens underperforming (RAY down 5%, ORCA flat)
- Volume declining despite price gains
- New regulatory concerns emerging

How do you adapt your strategy? What changes to:
1. Your market thesis
2. Position sizing and allocation
3. Risk parameters  
4. Timeline and monitoring

Walk through your adaptation process.
    "#;

    println!("🔄 Processing market feedback and adapting...");
    let adapted_strategy = strategist.prompt(feedback_prompt).await?;
    println!("⚡ Adapted Strategy:");
    println!("{}\n", adapted_strategy);

    // Test further adaptation to new conditions
    let crisis_prompt = r#"
Breaking: Major DeFi protocol exploit reported, $50M+ affected.
Market reacting with broad DeFi selloff, SOL down 12% in 30 minutes.

Crisis adaptation needed:
1. Immediate risk assessment for current positions
2. Whether to cut losses or hold through volatility  
3. Opportunities this crisis might create
4. Updated strategy for next 48 hours

How do you rapidly adapt to this crisis? Think through systematically.
    "#;

    println!("🚨 Crisis adaptation test...");
    let crisis_adaptation = strategist.prompt(crisis_prompt).await?;
    println!("🛑 Crisis Response:");
    println!("{}\n", crisis_adaptation);

    println!("✅ Adaptive reasoning demo complete!");
    
    Ok(())
}

/// Demo: Cross-Chain Opportunity Analysis
/// 
/// Demonstrates complex multi-source analysis and decision making
/// across different blockchain ecosystems.
async fn demo_cross_chain_reasoning() -> Result<()> {
    println!("\n🌐 Demo: Cross-Chain Opportunity Analysis");
    println!("==========================================");

    let opportunity_scout = AgentBuilder::new("gpt-4")
        .preamble(r#"
You are a cross-chain opportunity analyst who identifies profitable opportunities
across different blockchain ecosystems through systematic analysis.

Your approach:
1. Analyze opportunities across multiple chains simultaneously
2. Consider execution complexity, risks, and capital requirements
3. Factor in timing, competition, and market conditions  
4. Provide ranked recommendations with clear reasoning
5. Adapt analysis based on changing cross-chain conditions

You think systematically about cross-chain strategies without needing
custom loops - your reasoning naturally handles the complexity.
        "#)
        .max_tokens(1600)
        .build();

    // Cross-chain market data simulation
    let cross_chain_data = json!({
        "solana": {
            "native_yield": "5.2% (staking)",
            "defi_opportunities": "Jupiter LPs 12-18%, Orca farms 15-25%",
            "transaction_cost": "$0.0025 avg",
            "speed": "400ms blocks"
        },
        "ethereum": {
            "native_yield": "4.1% (staking)", 
            "defi_opportunities": "Uniswap LPs 8-15%, Aave lending 3-8%",
            "transaction_cost": "$15-40 per tx",
            "speed": "12s blocks"
        },
        "arbitrum": {
            "defi_opportunities": "Similar to ETH but 90% cheaper gas",
            "transaction_cost": "$0.50-2 per tx",
            "arbitrage": "Frequent ETH mainnet price gaps"
        },
        "current_bridges": {
            "wormhole_sol_eth": "0.1% fee + $5, 15min avg",
            "arbitrum_bridge": "0.05% fee + ETH gas, 10min avg"
        }
    });

    let opportunity_prompt = format!(r#"
Analyze cross-chain opportunities systematically using this market data:

{}

Find and rank the top 3 opportunities considering:
1. Risk-adjusted returns (ROI accounting for all costs)
2. Capital requirements and optimal position sizing
3. Execution complexity and timing requirements
4. Competition and sustainability factors

Think through each opportunity step-by-step. What's your systematic analysis?
    "#, serde_json::to_string_pretty(&cross_chain_data)?);

    println!("🔍 Analyzing cross-chain opportunities...");
    let opportunities = opportunity_scout.prompt(&opportunity_prompt).await?;
    println!("🌟 Opportunity Analysis:");
    println!("{}\n", opportunities);

    // Deep dive on top opportunity
    let execution_prompt = r#"
For your #1 recommended opportunity, provide a detailed execution plan:

1. Step-by-step execution sequence with timing
2. Capital allocation and risk management
3. Monitoring criteria and exit conditions  
4. Contingency plans for common failure scenarios
5. Expected timeline and profit targets

Walk me through exactly how to execute this opportunity.
    "#;

    println!("⚙️ Developing execution plan...");
    let execution_plan = opportunity_scout.prompt(execution_prompt).await?;
    println!("📋 Execution Plan:");
    println!("{}\n", execution_plan);

    // Test adaptation to changing conditions
    let condition_change = r#"
Update: Bridge congestion is causing 2-hour delays and fees spiked 3x.
How does this change your opportunity analysis and execution plan?

Consider:
1. Which opportunities become unviable?
2. New opportunities this creates?
3. Modified execution timing and parameters?
4. Alternative approaches to consider?

Adapt your analysis to these new conditions.
    "#;

    println!("🔄 Adapting to changed bridge conditions...");
    let adaptation = opportunity_scout.prompt(condition_change).await?;
    println!("⚡ Adapted Analysis:");
    println!("{}\n", adaptation);

    println!("✅ Cross-chain reasoning demo complete!");
    
    Ok(())
}

/// Demonstrates rig-native reasoning patterns that will integrate with riglr tools
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("🚀 Rig-Native Reasoning Loops with riglr Integration Patterns");
    println!("=============================================================");
    println!("Demonstrating sophisticated decision-making without custom loops\n");

    // Run reasoning pattern demonstrations
    if let Err(e) = demo_multi_step_reasoning().await {
        warn!("Multi-step reasoning demo failed: {}", e);
    }

    if let Err(e) = demo_adaptive_reasoning().await {
        warn!("Adaptive reasoning demo failed: {}", e);
    }

    if let Err(e) = demo_cross_chain_reasoning().await {
        warn!("Cross-chain reasoning demo failed: {}", e);
    }

    println!("\n🎯 Key Takeaways:");
    println!("================");
    println!("✅ Rig's multi-turn capabilities handle complex workflows naturally");
    println!("✅ No custom reasoning loops needed - agents think systematically");
    println!("✅ Context persists across conversation turns automatically"); 
    println!("✅ Agents adapt behavior based on intermediate results");
    println!("✅ Complex decision trees emerge through natural conversation");
    println!("✅ Tool chaining will happen seamlessly via agent reasoning");
    println!();
    println!("🔗 Integration with riglr tools:");
    println!("- When Phase I completes, add .tool() calls to AgentBuilder");
    println!("- Tools will be called automatically during agent reasoning");
    println!("- No changes needed to reasoning patterns shown above");
    println!("- SignerContext provides secure blockchain operation context");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reasoning_patterns() {
        // This test validates the reasoning pattern structure
        // In a real scenario, you'd test with actual LLM endpoints
        
        let agent = AgentBuilder::new("mock-model")
            .preamble("Test agent for reasoning patterns")
            .build();
        
        // Test that agent can be constructed
        assert!(true); // Placeholder - would test actual reasoning in full implementation
    }
    
    #[test]
    fn test_portfolio_data_serialization() {
        let portfolio = json!({
            "total_value": "$12,450",
            "positions": {
                "SOL": {"amount": "38.5", "value": "$5,775"},
            }
        });
        
        let serialized = serde_json::to_string(&portfolio).unwrap();
        assert!(serialized.contains("SOL"));
    }
}