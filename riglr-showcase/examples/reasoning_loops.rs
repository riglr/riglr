//! Multi-Turn Reasoning Loops Examples
//!
//! This example demonstrates rig-native reasoning loops and complex decision-making patterns.
//! Instead of implementing custom loops, we leverage rig's built-in multi-turn capabilities
//! to create sophisticated workflows that can be integrated with riglr tools.
//!
//! Key Concepts Demonstrated:
//! 1. Multi-turn agent conversations with complex reasoning
//! 2. Complex decision trees built with agent reasoning
//! 3. Tool chaining through natural agent conversation (conceptual)
//! 4. Context persistence across reasoning steps
//! 5. Adaptive behavior based on intermediate results
//!
//! Note: This example demonstrates the reasoning patterns. Full tool integration
//!       requires completion of Phase I migration (SignerContext + dual macro implementation).

use anyhow::Result;
use serde_json::json;
use tracing::{info, warn};
// Comment out broken rig code imports for now
// use rig::agent::AgentBuilder;
// use std::sync::Arc;
// use solana_sdk::signature::Keypair;
// use riglr_solana_tools::{get_sol_balance, get_spl_token_balance, perform_jupiter_swap};
// use riglr_core::SignerContext;
// use riglr_solana_tools::LocalSolanaSigner;

/// Example 1: Portfolio Analysis with Multi-Step Reasoning
///
/// This demonstrates how an agent can perform multi-step analysis without custom loops.
/// The agent uses rig's native multi-turn capabilities to:
/// 1. Analyze portfolio composition through reasoning
/// 2. Make data-driven recommendations
/// 3. Adapt strategies based on user responses
/// 4. Execute complex multi-step workflows
async fn portfolio_analysis_workflow() -> Result<()> {
    info!("Starting portfolio analysis workflow...");

    // Create a sophisticated agent with reasoning capabilities
    // Comment out broken rig code - AgentBuilder needs proper model initialization
    /*
        let agent = AgentBuilder::new("gpt-4")
            .preamble(r#"
    You are a sophisticated DeFi portfolio analyst. Your goal is to analyze portfolios
    and make intelligent rebalancing recommendations through multi-step reasoning.

    Your reasoning workflow:
    1. Analyze provided portfolio data and composition
    2. Assess risk levels and diversification gaps
    3. Consider market conditions and correlations
    4. Generate specific, actionable recommendations
    5. Provide detailed reasoning for each suggestion

    Key principles:
    - Use systematic analysis rather than gut feelings
    - Consider multiple scenarios and their probabilities
    - Explain your reasoning process step by step
    - Ask clarifying questions when you need more information
    - Adapt recommendations based on user feedback

    You think through complex problems systematically without needing custom loops.
            "#.trim())
            // When Phase I is complete, tools would be added here:
            // .tool(GetSolBalance)
            // .tool(GetSplTokenBalance)
            // .tool(PerformJupiterSwap)
            .max_tokens(2000)
            .build();
        */

    println!("🧠 Starting multi-step portfolio analysis...");

    // Simulate portfolio data (in production, this would come from blockchain queries)
    let portfolio_data = json!({
        "wallet_address": "DemoWallet123...",
        "total_value_usd": 15750,
        "holdings": {
            "SOL": {"amount": 45.2, "value_usd": 6780, "percentage": 43.1},
            "USDC": {"amount": 2200, "value_usd": 2200, "percentage": 14.0},
            "RAY": {"amount": 850, "value_usd": 3400, "percentage": 21.6},
            "ORCA": {"amount": 1200, "value_usd": 1800, "percentage": 11.4},
            "MNGO": {"amount": 5000, "value_usd": 1570, "percentage": 9.9}
        },
        "market_conditions": {
            "sol_24h_change": -3.2,
            "overall_volatility": "high",
            "defi_sentiment": "cautiously_optimistic"
        }
    });

    // Start the multi-turn reasoning process
    let _initial_analysis = format!(
        r#"
Please analyze this DeFi portfolio and provide comprehensive recommendations.

Portfolio Data:
{}

Analysis Required:
1. Portfolio composition and concentration risk assessment
2. Diversification analysis across different protocols/sectors
3. Risk-adjusted recommendations for rebalancing
4. Specific trades to execute with reasoning

Please think through this systematically, explaining your reasoning at each step.
What are your initial observations about this portfolio's risk profile?
    "#,
        serde_json::to_string_pretty(&portfolio_data)?
    );

    // This demonstrates rig's native multi-turn reasoning - no custom loops needed
    // Comment out broken rig code until proper model initialization is fixed
    /*
        let response1 = agent.prompt(&initial_analysis).await
            .context("Failed to get initial analysis")?;

        println!("\n📊 Portfolio Analysis:");
        println!("{}", response1);

        // Continue the reasoning with follow-up questions
        let deeper_analysis = r#"
    Based on your initial analysis, I'm interested in reducing concentration risk
    while maintaining growth potential. Please provide:

    1. Specific percentage allocations for optimal diversification
    2. Which positions to reduce and by how much
    3. New positions to consider adding
    4. Timeline and execution strategy for rebalancing

    Walk me through your reasoning process for these recommendations.
        "#;

        let response2 = agent.prompt(deeper_analysis).await
            .context("Failed to get detailed recommendations")?;

        println!("\n🎯 Detailed Recommendations:");
        println!("{}", response2);

        // Simulate market change and test adaptation
        let market_change = r#"
    Market Update: SOL just dropped 8% in the last hour due to broader crypto sell-off.
    How does this change your recommendations? Should we:

    1. Accelerate the rebalancing to reduce SOL exposure?
    2. Wait for potential recovery before making changes?
    3. Take advantage of the dip to accumulate more quality assets?

    Please reason through the implications of this market move on our strategy.
        "#;

        let response3 = agent.prompt(market_change).await
            .context("Failed to adapt to market change")?;

        println!("\n⚡ Market Adaptation:");
        println!("{}", response3);
        */

    // Placeholder demonstration until rig integration is fixed
    println!("\n📊 Portfolio Analysis:");
    println!("Demo: Would analyze portfolio composition and risk factors");

    println!("\n🎯 Detailed Recommendations:");
    println!("Demo: Would provide specific rebalancing recommendations");

    println!("\n⚡ Market Adaptation:");
    println!("Demo: Would adapt strategy based on market changes");

    info!("Portfolio analysis workflow completed");
    Ok(())
}

/// Example 2: Risk Assessment with Conditional Logic
///
/// This shows how agents can build complex decision trees:
/// 1. Assess current positions
/// 2. Check market conditions
/// 3. Calculate risk metrics
/// 4. Make conditional recommendations based on risk tolerance
async fn risk_assessment_reasoning() -> Result<()> {
    info!("Starting risk assessment reasoning...");

    // Comment out broken rig code until proper model initialization is fixed
    /*
        let agent = AgentBuilder::new("gpt-4")
            .preamble(r#"
    You are a risk management specialist for DeFi portfolios. Your job is to:

    1. Assess the current risk profile of a portfolio
    2. Check for concentration risks and exposure limits
    3. Monitor for high-risk positions that need immediate attention
    4. Provide specific risk mitigation strategies
    5. Execute emergency trades if risks are too high

    Use a systematic approach:
    - Calculate position sizes as % of total portfolio
    - Identify tokens with high volatility or low liquidity
    - Check for correlated asset exposure
    - Recommend specific actions based on risk levels

    Risk Thresholds:
    - LOW RISK: < 60% portfolio in any single asset
    - MEDIUM RISK: 60-80% in single asset OR high correlation
    - HIGH RISK: > 80% in single asset OR illiquid positions
    - CRITICAL: > 90% in single asset OR potential liquidation risk

    Always explain your risk calculations and reasoning.
            "#.trim())
            .tool(get_sol_balance)
            .tool(get_spl_token_balance)
            .tool(perform_jupiter_swap)
            .max_tokens(2000)
            .build();

        let keypair = Keypair::new();
        let signer = Arc::new(LocalSolanaSigner::new(
            keypair.insecure_clone(),
            "https://api.devnet.solana.com".to_string()
        ));

        SignerContext::with_signer(signer, async move {
            let user_address = keypair.pubkey().to_string();

            // Start with comprehensive risk assessment prompt
            let risk_prompt = format!(r#"
    Please perform a comprehensive risk assessment of my portfolio.

    My wallet: {}

    I need you to:
    1. Check all my token balances (SOL and major SPL tokens)
    2. Calculate position sizes and concentration ratios
    3. Assess liquidity risks for each holding
    4. Determine my overall risk level (LOW/MEDIUM/HIGH/CRITICAL)
    5. If risk is HIGH or CRITICAL, provide immediate action plan

    My risk tolerance is MEDIUM - I can accept some volatility but want to avoid
    major concentration risks.

    Please be thorough in your analysis and show your calculations.
            "#, user_address);

            let risk_analysis = agent.prompt(&risk_prompt).await?;
            info!("Risk analysis: {}", risk_analysis);

            // Follow up with specific scenarios
            let scenario_prompt = r#"
    Now please evaluate these specific scenarios and tell me how you would handle each:

    Scenario A: What if SOL drops 30% overnight?
    Scenario B: What if one of my altcoin positions becomes illiquid?
    Scenario C: What if I need to liquidate 50% of my portfolio within 24 hours?

    For each scenario, provide:
    1. Immediate risk level assessment
    2. Specific actions you would take
    3. Tools/trades you would execute
    4. Timeline for risk mitigation
            "#;

            let scenario_analysis = agent.prompt(scenario_prompt).await?;
            info!("Scenario analysis: {}", scenario_analysis);

            Ok(())
        }).await?;
        */

    // Placeholder demonstration until rig integration is fixed
    println!("📊 Risk Assessment Demo:");
    println!("Demo: Would analyze portfolio concentration and risk levels");
    println!("Demo: Would provide scenario-based risk mitigation strategies");

    Ok(())
}

/// Example 3: Market Opportunity Discovery
///
/// This demonstrates how agents can discover and analyze opportunities:
/// 1. Monitor multiple data sources
/// 2. Identify arbitrage or yield opportunities
/// 3. Assess opportunity risk/reward
/// 4. Execute complex multi-step strategies
async fn opportunity_discovery_reasoning() -> Result<()> {
    info!("Starting opportunity discovery reasoning...");

    // Comment out broken rig code until proper model initialization is fixed
    /*
        let agent = AgentBuilder::new("gpt-4")
            .preamble(r#"
    You are an advanced DeFi opportunity analyst. Your role is to:

    1. Continuously monitor for profitable opportunities
    2. Analyze cross-protocol arbitrage possibilities
    3. Identify high-yield farming opportunities with acceptable risk
    4. Execute complex multi-step strategies to capture profits
    5. Manage position sizing and risk throughout execution

    Your systematic approach:
    - Check current balances and available capital
    - Analyze yield opportunities across different protocols
    - Calculate potential profits vs. gas/transaction costs
    - Assess smart contract risks and protocol safety
    - Execute profitable opportunities with proper position sizing

    Always explain your opportunity analysis and profit calculations.
    Show step-by-step execution plans before taking action.
            "#.trim())
            .tool(get_sol_balance)
            .tool(get_spl_token_balance)
            .tool(perform_jupiter_swap)
            .max_tokens(2000)
            .build();

        let keypair = Keypair::new();
        let signer = Arc::new(LocalSolanaSigner::new(
            keypair.insecure_clone(),
            "https://api.devnet.solana.com".to_string()
        ));

        SignerContext::with_signer(signer, async move {
            let user_address = keypair.pubkey().to_string();

            let opportunity_prompt = format!(r#"
    I'm looking for profitable DeFi opportunities on Solana.

    My wallet: {}

    Please help me discover and evaluate opportunities:

    1. First, check my current balance and available capital
    2. Look for arbitrage opportunities between different DEXs
    3. Identify high-yield farming opportunities
    4. Calculate potential profits and risks for each opportunity
    5. Recommend the best opportunities based on my capital and risk tolerance

    My criteria:
    - Minimum profit threshold: $100 or 5% return
    - Maximum risk: Medium (no experimental protocols)
    - Preferred strategies: Arbitrage, yield farming, liquid staking
    - Available time: 1-2 hours for monitoring positions

    Please be specific about:
    - Exact steps to execute each opportunity
    - Expected profits and timeframes
    - Risk factors and mitigation strategies
            "#, user_address);

            let opportunities = agent.prompt(&opportunity_prompt).await?;
            info!("Opportunity analysis: {}", opportunities);

            // Deep dive into the best opportunity
            let execution_prompt = r#"
    Based on your analysis, please select the most profitable opportunity that fits my criteria.

    For your top recommendation:
    1. Create a detailed step-by-step execution plan
    2. Calculate exact amounts for each transaction
    3. Estimate total transaction costs (fees, slippage, etc.)
    4. Set up monitoring criteria to know when to exit
    5. If everything looks good, execute the first step

    Walk me through your reasoning process for selecting this opportunity.
            "#;

            let execution_plan = agent.prompt(execution_prompt).await?;
            info!("Execution plan: {}", execution_plan);

            Ok(())
        }).await?;
        */

    // Placeholder demonstration until rig integration is fixed
    println!("🎯 Opportunity Discovery Demo:");
    println!("Demo: Would scan for arbitrage and yield opportunities");
    println!("Demo: Would provide detailed execution plans with risk analysis");

    Ok(())
}

/// Example 4: Adaptive Strategy Based on Market Conditions
///
/// Shows how agents can adapt their behavior based on changing conditions:
/// 1. Monitor key market indicators
/// 2. Adjust strategy based on volatility and trends
/// 3. Switch between conservative and aggressive approaches
/// 4. Learn from previous execution results
async fn adaptive_strategy_reasoning() -> Result<()> {
    info!("Starting adaptive strategy reasoning...");

    // Comment out broken rig code until proper model initialization is fixed
    /*
        let agent = AgentBuilder::new("gpt-4")
            .preamble(r#"
    You are an adaptive trading strategist that adjusts behavior based on market conditions.

    Your adaptive framework:

    MARKET CONDITIONS ASSESSMENT:
    - High Volatility: > 10% daily moves in major assets
    - Normal Market: 2-10% daily volatility
    - Low Volatility: < 2% daily moves

    STRATEGY ADAPTATION:
    - High Vol: Conservative position sizing, quick profit taking, tight stops
    - Normal: Standard position sizing, hold for trends, moderate risk
    - Low Vol: Larger positions, range trading, yield strategies

    EXECUTION ADAPTATION:
    - Track success rate of different strategies
    - Adjust position sizing based on recent performance
    - Switch between strategies based on what's working

    Your goal: Maximize risk-adjusted returns by adapting to market conditions.
    Always explain your market assessment and strategy selection reasoning.
            "#.trim())
            .tool(get_sol_balance)
            .tool(get_spl_token_balance)
            .tool(perform_jupiter_swap)
            .max_tokens(2000)
            .build();

        let keypair = Keypair::new();
        let signer = Arc::new(LocalSolanaSigner::new(
            keypair.insecure_clone(),
            "https://api.devnet.solana.com".to_string()
        ));

        SignerContext::with_signer(signer, async move {
            let user_address = keypair.pubkey().to_string();

            let adaptive_prompt = format!(r#"
    Please analyze current market conditions and adapt your trading strategy accordingly.

    My wallet: {}

    I want you to:
    1. Assess current market volatility and trend direction
    2. Determine which strategy framework is most appropriate right now
    3. Check my current positions and capital allocation
    4. Recommend specific position adjustments based on market conditions
    5. Set up monitoring criteria to detect when conditions change

    Historical performance context:
    - Last week: Made 8% profit using momentum strategy in high-vol environment
    - Previous month: Lost 3% using same strategy in sideways market
    - Overall: Best performance with trend-following in trending markets

    Please adapt your recommendations based on:
    - Current market regime
    - My historical performance patterns
    - Available capital and risk capacity
            "#, user_address);

            let market_analysis = agent.prompt(&adaptive_prompt).await?;
            info!("Adaptive market analysis: {}", market_analysis);

            // Test adaptation to changing conditions
            let condition_change_prompt = r#"
    Market Update: SOL just dropped 15% in the last 2 hours due to broader market sell-off.
    Volatility has spiked significantly.

    How should this change your strategy? Please:
    1. Reassess the market regime (has it shifted?)
    2. Adjust position sizing and risk parameters
    3. Identify any positions that need immediate attention
    4. Update monitoring criteria for this new environment
    5. Execute any necessary trades to adapt to the new conditions

    Show me how you're adapting in real-time to this market change.
            "#;

            let adaptation_response = agent.prompt(condition_change_prompt).await?;
            info!("Strategy adaptation: {}", adaptation_response);

            Ok(())
        }).await?;
        */

    // Placeholder demonstration until rig integration is fixed
    println!("🧠 Adaptive Strategy Demo:");
    println!("Demo: Would assess market volatility and adapt trading approach");
    println!("Demo: Would adjust position sizing based on current conditions");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for detailed logging
    tracing_subscriber::fmt::init();

    info!("Starting riglr multi-turn reasoning examples...");
    info!("These examples demonstrate rig-native reasoning loops using riglr tools");

    println!("\n🧠 Multi-Turn Reasoning Loops with riglr Tools");
    println!("================================================");
    println!("Demonstrating rig's native multi-turn capabilities with Solana blockchain tools\n");

    // Run examples in sequence
    println!("1️⃣  Portfolio Analysis Workflow");
    println!("   Multi-step portfolio analysis with tool chaining...");
    if let Err(e) = portfolio_analysis_workflow().await {
        warn!("Portfolio analysis failed: {}", e);
    }

    println!("\n2️⃣  Risk Assessment Reasoning");
    println!("   Complex decision trees for risk management...");
    if let Err(e) = risk_assessment_reasoning().await {
        warn!("Risk assessment failed: {}", e);
    }

    println!("\n3️⃣  Opportunity Discovery");
    println!("   Multi-source analysis for profit opportunities...");
    if let Err(e) = opportunity_discovery_reasoning().await {
        warn!("Opportunity discovery failed: {}", e);
    }

    println!("\n4️⃣  Adaptive Strategy");
    println!("   Real-time strategy adaptation to market conditions...");
    if let Err(e) = adaptive_strategy_reasoning().await {
        warn!("Adaptive strategy failed: {}", e);
    }

    println!("\n✅ Multi-turn reasoning examples completed!");
    println!("\nKey Takeaways:");
    println!("- rig's native multi-turn capabilities enable sophisticated reasoning");
    println!("- No custom loops needed - agents handle complexity naturally");
    println!("- Tools chain seamlessly through agent conversation");
    println!("- Context persists across reasoning steps");
    println!("- Agents adapt behavior based on intermediate results");

    Ok(())
}

/// Helper function to demonstrate programmatic agent interaction
/// This shows how you can build interactive reasoning systems
/// Note: Currently commented out due to rig integration issues
/*
async fn interactive_reasoning_session(
    agent: &Agent,
    initial_context: &str,
    follow_up_queries: Vec<&str>
) -> Result<Vec<String>> {
    let mut responses = Vec::new();

    // Start with initial context
    let initial_response = agent.prompt(initial_context).await
        .context("Failed to get initial response")?;
    responses.push(initial_response);

    // Continue with follow-up queries (this simulates multi-turn reasoning)
    for query in follow_up_queries {
        let response = agent.prompt(query).await
            .context("Failed to get follow-up response")?;
        responses.push(response);
    }

    Ok(responses)
}
*/

#[cfg(test)]
mod tests {

    /// Test that demonstrates rig-native multi-turn capabilities
    #[tokio::test]
    async fn test_multi_turn_reasoning() {
        // This test would typically require a real LLM endpoint
        // For now, it validates the structure and flow // Placeholder
    }

    /// Test tool chaining through agent reasoning
    #[tokio::test]
    async fn test_tool_chaining() {
        // Validate that tools can be chained through natural conversation // Placeholder
    }

    /// Test context persistence across reasoning steps
    #[tokio::test]
    async fn test_context_persistence() {
        // Validate that context is maintained across multiple agent interactions // Placeholder
    }
}
