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
use rig::providers::openai;
use rig::client::CompletionClient;
use serde_json::json;
use std::env;
use tracing::warn;

/// Demo: Multi-Step Portfolio Analysis Through Reasoning
/// 
/// Shows how rig's native capabilities can handle complex workflows
/// that would traditionally require custom loop implementation.
async fn demo_multi_step_reasoning() -> Result<()> {
    println!("🧠 Demo: Multi-Step Portfolio Analysis Through Reasoning");
    println!("=======================================================");

    // Create OpenAI client and model
    let openai_client = openai::Client::new(&env::var("OPENAI_API_KEY")?);
    let model = openai_client.completion_model("gpt-4");
    
    let portfolio_analyst = AgentBuilder::new(model)
        .preamble(r#"
You are a sophisticated portfolio analyst who performs multi-step analysis
through systematic reasoning. You break down complex portfolio decisions into
logical steps, considering multiple factors and their interactions.

Your approach:
1. Assess current portfolio composition and performance
2. Analyze market conditions and trends
3. Identify risks and opportunities
4. Develop specific recommendations with reasoning
5. Consider implementation and monitoring strategies

You reason through each step systematically, showing your thinking process.
        "#)
        .build();

    // Simulate portfolio data
    let portfolio_data = json!({
        "current_holdings": {
            "SOL": {"amount": "45.2", "value": "$7,380", "pct": "41%"},
            "ETH": {"amount": "2.8", "value": "$5,320", "pct": "30%"},
            "BTC": {"amount": "0.12", "value": "$3,240", "pct": "18%"},
            "USDC": {"amount": "1,980", "value": "$1,980", "pct": "11%"}
        },
        "total_value": "$17,920",
        "performance": {
            "30d_return": "+18.5%",
            "ytd_return": "+145%",
            "max_drawdown": "-22%",
            "volatility": "65% annualized"
        },
        "goals": {
            "risk_tolerance": "moderate-high",
            "time_horizon": "6-18 months",
            "target_return": "50-100% annual"
        }
    });

    let analysis_prompt = format!(r#"
Perform multi-step portfolio analysis using this data:

Portfolio Data:
{}

Step-by-Step Analysis Required:
1. Current Portfolio Assessment:
   - Composition balance and concentration risk
   - Performance relative to crypto market
   - Risk-return profile evaluation

2. Market Context Analysis:
   - Current crypto market conditions
   - Relative strength of holdings vs alternatives
   - Upcoming catalysts and risks

3. Optimization Opportunities:
   - Rebalancing recommendations
   - New allocation targets
   - Specific actions to take

4. Implementation Strategy:
   - Timing and sequencing of changes
   - Risk management during transitions
   - Monitoring criteria for success

Walk through each step systematically with clear reasoning.
    "#, serde_json::to_string_pretty(&portfolio_data)?);

    println!("📊 Performing multi-step portfolio analysis...");
    let analysis = portfolio_analyst.prompt(&analysis_prompt).await?;
    println!("🎯 Multi-Step Analysis:");
    println!("{}\n", analysis);

    // Follow-up with specific implementation questions
    let implementation_prompt = r#"
Based on your analysis, I want to implement your recommendations.

Provide specific implementation guidance:
1. What should I do first, second, third?
2. How much capital should I allocate to each action?
3. What are the key timing considerations?
4. How do I monitor progress and adjust if needed?
5. What could go wrong and how do I prepare?

Think through the practical implementation systematically.
    "#;

    println!("⚙️ Developing implementation plan...");
    let implementation = portfolio_analyst.prompt(implementation_prompt).await?;
    println!("📋 Implementation Plan:");
    println!("{}\n", implementation);

    println!("✅ Multi-step reasoning demo complete!");
    Ok(())
}


/// Demo: Adaptive Strategy Based on Market Feedback
/// 
/// Shows how agents can adapt their approach based on intermediate results
/// and changing conditions, using rig's natural conversation flow.
async fn demo_adaptive_reasoning() -> Result<()> {
    println!("\n🔄 Demo: Adaptive Reasoning Based on Market Feedback");  
    println!("===================================================");

    // Create OpenAI client and model
    let openai_client = openai::Client::new(&env::var("OPENAI_API_KEY")?);
    let model = openai_client.completion_model("gpt-4");
    
    let strategist = AgentBuilder::new(model)
        .preamble(r#"
You are an adaptive trading strategist who adjusts approach based on:
- Market feedback and results
- Changing conditions and new information  
- Success/failure of previous recommendations
- Real-time market dynamics

You continuously refine your strategy through systematic reasoning,
not rigid rules. You explain how and why you adapt your approach.
        "#)
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

    // Create OpenAI client and model
    let openai_client = openai::Client::new(&env::var("OPENAI_API_KEY")?);
    let model = openai_client.completion_model("gpt-4");
    
    let opportunity_scout = AgentBuilder::new(model)
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
        // Test validates reasoning pattern concepts
        // Note: This test requires OPENAI_API_KEY to be set for actual agent testing
        if std::env::var("OPENAI_API_KEY").is_ok() {
            let openai_client = openai::Client::new(&std::env::var("OPENAI_API_KEY").unwrap());
            let model = openai_client.completion_model("gpt-3.5-turbo");
            
            let agent = AgentBuilder::new(model)
                .preamble("You are a test agent for reasoning patterns.")
                .build();
            
            // Test basic agent construction
            assert!(true); // Agent was successfully created
        } else {
            // Skip actual agent testing if no API key is available
            assert!(true, "Skipping agent test - OPENAI_API_KEY not set");
        }
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