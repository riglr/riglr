//! Market Analysis with Reasoning Examples
//!
//! This example demonstrates sophisticated market analysis workflows using rig's
//! native reasoning capabilities. Shows how agents can perform complex multi-step
//! analysis, synthesize information from multiple sources, and generate actionable
//! insights without custom analysis loops.
//!
//! Key Features:
//! 1. Multi-source data analysis and synthesis
//! 2. Cross-chain opportunity identification and evaluation
//! 3. Risk-adjusted decision making frameworks
//! 4. Adaptive analysis based on market conditions
//! 5. Complex workflow orchestration through natural reasoning

use anyhow::Result;

const OPENAI_API_KEY: &str = "OPENAI_API_KEY";
use rig::agent::AgentBuilder;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::openai;
use serde_json::json;
use std::env;
use tracing::warn;

/// Demo: Comprehensive Token Analysis Through Systematic Reasoning
///
/// Shows how agents can perform multi-methodology analysis combining technical,
/// fundamental, and sentiment analysis without custom analysis frameworks.
async fn demo_comprehensive_token_analysis() -> Result<()> {
    println!("🔍 Demo: Comprehensive Token Analysis");
    println!("=====================================");

    // Create OpenAI client and model
    let openai_client = openai::Client::new(&env::var(OPENAI_API_KEY)?);
    let model = openai_client.completion_model("gpt-4");

    let market_analyst = AgentBuilder::new(model)
        .preamble(
            r#"
You are a comprehensive cryptocurrency market analyst who performs systematic
multi-methodology analysis combining multiple approaches.

Analysis Framework:
1. Technical Analysis: Price action, volume, momentum, support/resistance
2. Fundamental Analysis: Protocol utility, team, tokenomics, adoption
3. Sentiment Analysis: Social media, community, influencer opinions
4. On-Chain Analysis: Network activity, whale behavior, exchange flows
5. Risk Assessment: Liquidity, volatility, correlation, regulatory factors

Your approach is systematic and thorough:
- Break down complex analysis into logical components
- Synthesize information from multiple perspectives
- Provide confidence-weighted conclusions
- Generate specific, actionable recommendations
- Adapt analysis based on market conditions

You reason through analysis naturally without needing custom frameworks.
        "#,
        )
        .build();

    // Comprehensive market data simulation (would come from riglr tools)
    let market_data = json!({
        "token": "SOL",
        "technical_data": {
            "price": "$163.50",
            "24h_change": "-1.8%",
            "volume_24h": "$2.1B",
            "market_cap": "$76.8B",
            "rsi": 58,
            "macd": "bullish crossover",
            "support_levels": ["$155", "$145", "$135"],
            "resistance_levels": ["$175", "$190", "$210"]
        },
        "fundamental_data": {
            "tps_capability": "65,000+",
            "active_developers": "2,500+",
            "defi_tvl": "$5.8B",
            "nft_volume": "$85M monthly",
            "validator_count": "3,200+",
            "staking_ratio": "68%",
            "upcoming_catalysts": ["Firedancer upgrade", "Mobile ecosystem", "RWA adoption"]
        },
        "sentiment_data": {
            "social_sentiment": "74% positive",
            "twitter_mentions": "+15% vs 30d avg",
            "reddit_engagement": "high activity",
            "fear_greed_index": 67,
            "analyst_ratings": "8.2/10 average"
        },
        "onchain_data": {
            "daily_active_addresses": "1.35M",
            "transaction_volume": "$3.2B daily",
            "whale_accumulation": "+2.1M SOL last 7d",
            "exchange_flows": "-890K SOL (net outflow)",
            "new_wallet_creation": "52K daily"
        }
    });

    let _analysis_prompt = format!(
        r#"
Perform comprehensive analysis of SOL using systematic multi-methodology approach:

Market Data:
{}

Comprehensive Analysis Required:
1. Technical Analysis: Chart patterns, momentum, key levels
2. Fundamental Strength: Protocol development, ecosystem growth
3. Sentiment Assessment: Community and market psychology
4. On-Chain Health: Network usage and whale behavior
5. Risk Evaluation: Volatility, liquidity, external factors
6. Investment Thesis: Bull/bear cases with probabilities
7. Actionable Recommendations: Entry levels, targets, stops

Walk through each methodology systematically and synthesize into clear conclusions.
    "#,
        serde_json::to_string_pretty(&market_data)?
    );

    println!("📊 Requesting comprehensive multi-methodology analysis...");
    let comprehensive_analysis = market_analyst.prompt(&_analysis_prompt).await?;
    println!("🎯 Comprehensive Analysis:");
    println!("{}\n", comprehensive_analysis);

    // Test comparative analysis capabilities
    let _comparative_prompt = r#"
Now perform comparative analysis against key competitors:

Compare SOL vs:
1. ETH - Established smart contract leader
2. AVAX - Similar high-performance architecture
3. MATIC - Ethereum scaling solution
4. ADA - Academic approach to blockchain

For each comparison:
- Technology advantages/disadvantages
- Market positioning and adoption
- Valuation metrics and growth potential
- Competitive moats and threats
- Risk/reward profiles

Based on this analysis:
- Should I increase, maintain, or reduce SOL exposure?
- What's the optimal allocation among these alternatives?
- Which has best risk-adjusted opportunity?

Provide systematic comparative evaluation.
    "#;

    println!("⚖️ Performing competitive analysis...");
    let competitive_analysis = market_analyst.prompt(_comparative_prompt).await?;
    println!("🏆 Competitive Analysis:");
    println!("{}\n", competitive_analysis);

    // Test adaptation to new information
    let _breaking_news_prompt = r#"
Breaking Market Update: Federal Reserve announces crypto-supportive framework:
- Clear DeFi regulatory guidelines
- Staking rewards classified as utility (not securities)
- Institutional custody solutions approved
- Major compliance barriers removed

Immediate Market Reaction:
- SOL up 11% to $182 in 45 minutes
- All smart contract platforms rallying
- Institutional buying interest spiking
- Options volatility increasing

How does this fundamental news change your analysis?

Update your:
1. Fundamental analysis given regulatory clarity
2. Technical analysis with new price action
3. Risk assessment with reduced regulatory overhang
4. Investment thesis and recommendation changes
5. Position sizing and entry strategy adjustments

Show systematic adaptation to major fundamental changes.
    "#;

    println!("📰 Testing adaptation to breaking news...");
    let news_adaptation = market_analyst.prompt(_breaking_news_prompt).await?;
    println!("⚡ News Impact Analysis:");
    println!("{}\n", news_adaptation);

    println!("✅ Comprehensive analysis demo complete!");
    Ok(())
}

/// Demo: Cross-Chain Opportunity Discovery and Analysis
///
/// Shows sophisticated analysis across multiple blockchain ecosystems to identify
/// and evaluate arbitrage, yield, and strategic opportunities.
async fn demo_cross_chain_opportunity_analysis() -> Result<()> {
    println!("\n🌐 Demo: Cross-Chain Opportunity Discovery");
    println!("==========================================");

    // Create OpenAI client and model
    let openai_client = openai::Client::new(&env::var(OPENAI_API_KEY)?);
    let model = openai_client.completion_model("gpt-4");

    let opportunity_analyst = AgentBuilder::new(model)
        .preamble(
            r#"
You are a cross-chain opportunity analyst specializing in identifying and
evaluating profit opportunities across different blockchain ecosystems.

Opportunity Analysis Framework:
1. Arbitrage Opportunities: Price gaps, DEX differences, bridge premiums
2. Yield Opportunities: Staking, farming, lending rate differences
3. Strategic Opportunities: New launches, airdrops, first-mover advantages
4. Risk Assessment: Execution complexity, smart contract risks, timing
5. Capital Efficiency: ROI calculations, optimal position sizing

Cross-Chain Considerations:
- Bridge costs, speeds, and security
- Gas fees and transaction economics
- Liquidity depth and slippage impact
- Timing coordination across chains
- Regulatory differences between ecosystems

You systematically evaluate opportunities without needing custom discovery loops.
        "#,
        )
        .build();

    // Cross-chain market conditions
    let cross_chain_data = json!({
        "solana_ecosystem": {
            "native_yield": "5.8% (staking SOL)",
            "defi_yields": {
                "jupiter_lp": "14-22%",
                "orca_farms": "18-28%",
                "marinade_liquid_staking": "6.2%"
            },
            "transaction_cost": "$0.0025 avg",
            "block_time": "400ms",
            "current_opportunities": "New yield farms launching weekly"
        },
        "ethereum_ecosystem": {
            "native_yield": "4.3% (ETH staking)",
            "defi_yields": {
                "uniswap_v3": "8-18% (depending on range)",
                "aave_lending": "3-9%",
                "compound": "4-7%",
                "lido_staking": "4.8%"
            },
            "transaction_cost": "$12-35 per tx",
            "block_time": "12s",
            "current_opportunities": "L2 migration incentives active"
        },
        "arbitrum_l2": {
            "defi_yields": "Similar to ETH mainnet",
            "transaction_cost": "$0.30-1.50 per tx",
            "arbitrage_potential": "Frequent mainnet price gaps",
            "current_opportunities": "ARB token incentives for liquidity"
        },
        "polygon_ecosystem": {
            "defi_yields": {
                "quickswap": "15-35%",
                "aave_polygon": "5-12%",
                "curve_polygon": "8-20%"
            },
            "transaction_cost": "$0.01-0.05 per tx",
            "bridge_activity": "High USDC/USDT flows",
            "current_opportunities": "MATIC staking rewards increased"
        },
        "bridge_conditions": {
            "wormhole": "0.1% fee + gas, 15min avg time",
            "multichain": "0.05-0.1% fee, 10-20min",
            "layerzero": "Variable fees, 5-15min",
            "current_status": "Normal congestion, fees stable"
        }
    });

    let _opportunity_prompt = format!(
        r#"
Systematically analyze cross-chain opportunities using current market data:

Cross-Chain Data:
{}

Opportunity Discovery Required:
1. Identify top 5 opportunities ranked by risk-adjusted returns
2. For each opportunity calculate:
   - Expected annual returns (accounting for all costs)
   - Capital requirements and optimal position size
   - Execution complexity and time commitment
   - Key risks and mitigation strategies
   - Competition and sustainability factors

3. Provide detailed execution plan for #1 opportunity
4. Consider portfolio allocation across multiple opportunities

Think through each ecosystem systematically and identify the best opportunities.
    "#,
        serde_json::to_string_pretty(&cross_chain_data)?
    );

    println!("🔍 Discovering cross-chain opportunities...");
    let opportunities = opportunity_analyst.prompt(&_opportunity_prompt).await?;
    println!("💎 Opportunity Analysis:");
    println!("{}\n", opportunities);

    // Deep dive on execution
    let _execution_prompt = r#"
For your top recommended opportunity, provide detailed execution framework:

Execution Plan Required:
1. Step-by-step implementation sequence with timing
2. Capital allocation strategy and risk management
3. Monitoring criteria and performance tracking
4. Exit conditions and profit-taking strategy
5. Contingency plans for common failure scenarios
6. Expected timeline and milestone targets

Risk Management:
- Position sizing relative to total portfolio
- Stop-loss levels and drawdown limits
- Correlation risks with other positions
- Smart contract and bridge security considerations

Walk through exactly how to implement this opportunity safely and profitably.
    "#;

    println!("⚙️ Developing detailed execution plan...");
    let execution_plan = opportunity_analyst.prompt(_execution_prompt).await?;
    println!("📋 Execution Framework:");
    println!("{}\n", execution_plan);

    // Test adaptation to changing conditions
    let _condition_change = r#"
Market Condition Update: Bridge congestion crisis!

New Developments:
- Major bridge exploit causes network-wide congestion
- Bridge fees spiked 5x normal levels
- Transaction times extended to 2-4 hours
- Alternative bridges temporarily paused
- Cross-chain arbitrage spreads widening significantly

Impact Assessment:
1. Which opportunities become unviable due to increased costs?
2. What new opportunities does this congestion create?
3. How should execution timing and parameters change?
4. What alternative strategies should be considered?
5. Should we pause cross-chain activities entirely?

Systematically adapt your opportunity analysis to these changed conditions.
    "#;

    println!("🚨 Adapting to bridge congestion crisis...");
    let crisis_adaptation = opportunity_analyst.prompt(_condition_change).await?;
    println!("⚡ Crisis Adaptation:");
    println!("{}\n", crisis_adaptation);

    println!("✅ Cross-chain opportunity analysis complete!");
    Ok(())
}

/// Demo: Multi-Source Intelligence Synthesis
///
/// Shows how agents can synthesize information from multiple disparate sources
/// to create comprehensive market intelligence and resolve conflicting signals.
async fn demo_intelligence_synthesis() -> Result<()> {
    println!("\n🧠 Demo: Multi-Source Intelligence Synthesis");
    println!("===============================================");

    // Create OpenAI client and model
    let openai_client = openai::Client::new(&env::var(OPENAI_API_KEY)?);
    let model = openai_client.completion_model("gpt-4");

    let intelligence_synthesizer = AgentBuilder::new(model)
        .preamble(
            r#"
You are a market intelligence synthesizer who combines information from multiple
sources to generate high-conviction investment insights.

Intelligence Sources:
1. On-Chain Data: Transaction flows, whale behavior, network metrics
2. Market Data: Price action, volume, derivatives, cross-exchange analysis
3. Social Intelligence: Sentiment, influencer opinions, community activity
4. Fundamental Data: Development progress, partnerships, competitive position
5. Macro Environment: Regulations, institutions, traditional market correlations

Synthesis Methodology:
- Weight sources by reliability and relevance to current context
- Identify convergent vs divergent signals across sources
- Resolve conflicts through deeper analysis and priority weighting
- Generate confidence-weighted conclusions with uncertainty bounds
- Provide specific, actionable recommendations with risk assessment

You synthesize complex, contradictory information systematically without rigid rules.
        "#,
        )
        .build();

    // Multi-source intelligence data
    let intelligence_data = json!({
        "onchain_signals": {
            "whale_activity": "Mixed - some accumulation, some distribution",
            "exchange_flows": "Net outflow of 1.2M SOL (bullish)",
            "active_addresses": "Growing +12% monthly (bullish)",
            "transaction_volume": "Declining -8% vs last month (bearish)",
            "staking_participation": "Increasing to 69.2% (neutral/bullish)",
            "confidence": "70% - data is clear but mixed signals"
        },
        "market_signals": {
            "price_action": "Consolidation after +45% rally",
            "volume_profile": "Above average, institutional size trades",
            "options_flow": "60% call bias, increasing volatility",
            "futures_funding": "+0.12% (moderate bullish sentiment)",
            "cross_exchange_premium": "Coinbase +0.4% vs Binance (institutional buying)",
            "confidence": "85% - market structure data is reliable"
        },
        "social_intelligence": {
            "twitter_sentiment": "78% positive, trending upward",
            "reddit_activity": "High engagement, optimistic discussions",
            "influencer_opinions": "Mixed - some very bullish, others cautious",
            "google_trends": "Search interest +22% vs last month",
            "news_sentiment": "80% positive coverage, regulatory optimism",
            "confidence": "60% - social data can be noisy and manipulated"
        },
        "fundamental_signals": {
            "development_activity": "High - Firedancer progress, Mobile ecosystem",
            "partnership_pipeline": "Strong - enterprise adoption growing",
            "competitive_position": "Gaining share vs Ethereum L2s",
            "ecosystem_growth": "DeFi TVL +15% monthly, NFTs stable",
            "regulatory_status": "Improving - not mentioned in SEC enforcement",
            "confidence": "90% - fundamental data is most reliable"
        },
        "macro_environment": {
            "crypto_regulation": "Improving clarity, supportive framework emerging",
            "institutional_adoption": "Accelerating - PayPal, Visa integration",
            "traditional_markets": "Risk-on environment, correlations decreasing",
            "central_bank_policy": "Dovish pivot supporting risk assets",
            "geopolitical_factors": "Stable, no major crypto-negative developments",
            "confidence": "75% - macro trends are clear but can change quickly"
        },
        "conflicting_signals": [
            "On-chain: Whale behavior mixed vs exchange outflows positive",
            "Market: Strong structure vs recent volume decline",
            "Social: High sentiment vs some influencer caution",
            "Timing: Strong fundamentals vs potential short-term consolidation"
        ]
    });

    let _synthesis_prompt = format!(
        r#"
Synthesize this multi-source intelligence into actionable investment thesis:

Intelligence Sources:
{}

Synthesis Required:
1. Weight each source by reliability and current relevance
2. Identify where sources converge vs diverge
3. Resolve the key conflicting signals systematically
4. Generate investment thesis with confidence levels:
   - Bull case scenario with probability and targets
   - Bear case risks with probability and downside
   - Base case most likely outcome
5. Specific recommendations:
   - Optimal entry strategy and position sizing
   - Key monitoring criteria and decision triggers
   - Risk management and stop-loss levels
   - Timeline expectations and milestone targets

Show your systematic approach to synthesizing contradictory information.
    "#,
        serde_json::to_string_pretty(&intelligence_data)?
    );

    println!("🔍 Synthesizing multi-source intelligence...");
    let synthesis = intelligence_synthesizer.prompt(&_synthesis_prompt).await?;
    println!("🎯 Intelligence Synthesis:");
    println!("{}\n", synthesis);

    // Test handling of new contradictory information
    let _contradictory_update = r#"
Intelligence Update: Major contradictory signals emerging!

New Conflicting Information:
- On-Chain: Large whale address just sold 2.8M SOL (contradicts accumulation narrative)
- Market: Hidden sell orders detected at $170 level (contradicts bullish structure)
- Social: Major crypto influencer turned bearish citing "distribution phase"
- Fundamental: Competitor announced major technical breakthrough
- Macro: Fed officials hint at potential rate hike despite dovish pivot

Information Reliability Questions:
- Whale sale: Is this forced liquidation, tax selling, or actual bearish signal?
- Hidden orders: Market manipulation or genuine supply?
- Influencer opinion: Contrarian indicator or legitimate concern?
- Competitor news: Marketing hype or real technical advantage?
- Fed communication: Hawkish minority or policy shift?

How do you handle these contradictory intelligence updates?

Required:
1. Methodology for verifying conflicting information
2. Updated source weighting given new data
3. Revised investment thesis with new probability distributions
4. Modified risk management for increased uncertainty
5. Additional data needed to resolve key conflicts

Show systematic approach to handling contradictory intelligence.
    "#;

    println!("❓ Processing contradictory intelligence updates...");
    let contradiction_handling = intelligence_synthesizer
        .prompt(_contradictory_update)
        .await?;
    println!("🔄 Contradiction Resolution:");
    println!("{}\n", contradiction_handling);

    println!("✅ Intelligence synthesis demo complete!");
    Ok(())
}

/// Demo: Risk-Adjusted Portfolio Construction Through Systematic Analysis
///
/// Shows sophisticated portfolio analysis including correlation analysis, scenario
/// testing, and quantitative risk management through agent reasoning.
async fn demo_portfolio_risk_analysis() -> Result<()> {
    println!("\n📊 Demo: Risk-Adjusted Portfolio Analysis");
    println!("==========================================");

    // Create OpenAI client and model
    let openai_client = openai::Client::new(&env::var(OPENAI_API_KEY)?);
    let model = openai_client.completion_model("gpt-4");

    let portfolio_analyst = AgentBuilder::new(model)
        .preamble(
            r#"
You are a quantitative portfolio analyst specializing in risk-adjusted optimization
for cryptocurrency portfolios using systematic analysis.

Risk Analysis Framework:
1. Quantitative Metrics: VaR, Sharpe ratios, correlation analysis, drawdown
2. Scenario Analysis: Bull/bear/crisis scenarios with probability weighting
3. Portfolio Construction: Modern portfolio theory, risk parity, factor exposure
4. Dynamic Risk Management: Volatility adjustment, correlation monitoring
5. Stress Testing: Extreme scenarios, liquidity crises, correlation breakdown

Portfolio Optimization Approach:
- Maximize risk-adjusted returns, not absolute returns
- Consider correlation dynamics and regime changes
- Account for tail risks and extreme scenarios
- Balance growth potential with downside protection
- Adapt to changing market conditions systematically

You perform sophisticated quantitative analysis through systematic reasoning.
        "#,
        )
        .build();

    // Portfolio and market data
    let portfolio_data = json!({
        "current_portfolio": {
            "total_value": "$125,000",
            "positions": {
                "BTC": {"value": "$37,500", "pct": "30%", "6m_return": "+18%"},
                "ETH": {"value": "$31,250", "pct": "25%", "6m_return": "+28%"},
                "SOL": {"value": "$25,000", "pct": "20%", "6m_return": "+85%"},
                "AVAX": {"value": "$12,500", "pct": "10%", "6m_return": "+42%"},
                "MATIC": {"value": "$10,000", "pct": "8%", "6m_return": "+15%"},
                "LINK": {"value": "$6,250", "pct": "5%", "6m_return": "+22%"},
                "USDC": {"value": "$2,500", "pct": "2%", "6m_return": "0%"}
            },
            "performance_metrics": {
                "6m_total_return": "+35%",
                "max_drawdown": "-28% (during May crash)",
                "volatility": "68% annualized",
                "sharpe_ratio": 1.1,
                "correlation_with_btc": "0.85 average"
            }
        },
        "correlation_matrix": {
            "btc_eth": 0.82,
            "btc_sol": 0.78,
            "btc_avax": 0.85,
            "eth_sol": 0.88,
            "sol_avax": 0.91,
            "altcoins_avg": 0.87,
            "usdc_correlations": "near zero"
        },
        "market_conditions": {
            "current_regime": "Bull market with high volatility",
            "crypto_fear_greed": 72,
            "btc_dominance": "52%",
            "institutional_flow": "Net positive $2.1B monthly",
            "regulatory_environment": "Improving clarity"
        }
    });

    let _risk_analysis_prompt = format!(
        r#"
Perform comprehensive risk-adjusted portfolio analysis:

Portfolio Data:
{}

Risk Analysis Required:
1. Current Risk Assessment:
   - Portfolio concentration and correlation risks
   - Value at Risk (95% confidence) estimation
   - Maximum potential drawdown analysis
   - Sharpe ratio optimization opportunities

2. Scenario Analysis:
   - Bull case: +100% crypto market scenario
   - Bear case: -70% crypto crash scenario
   - Crisis case: Correlation breakdown, liquidity crisis
   - Probability-weighted expected outcomes

3. Portfolio Optimization:
   - Optimal allocation for current market regime
   - Rebalancing recommendations with specific targets
   - New positions to improve risk-adjusted returns
   - Position sizing based on Kelly criterion concepts

4. Dynamic Risk Management:
   - Stop-loss levels and position sizing rules
   - Volatility-based allocation adjustments
   - Correlation monitoring and hedging strategies
   - Liquidity management for different market phases

Provide systematic quantitative analysis with specific recommendations.
    "#,
        serde_json::to_string_pretty(&portfolio_data)?
    );

    println!("📈 Performing quantitative risk analysis...");
    let risk_analysis = portfolio_analyst.prompt(&_risk_analysis_prompt).await?;
    println!("📊 Risk Analysis Results:");
    println!("{}\n", risk_analysis);

    // Stress test with extreme scenario
    let _stress_test_prompt = r#"
Stress Test: "Crypto Winter 2.0" Crisis Scenario

Crisis Parameters:
- Major regulatory crackdown: SEC declares most DeFi tokens securities
- Institutional selling accelerates: $50B+ outflows monthly
- Credit crisis hits crypto lending: Major platform bankruptcies
- Stablecoin de-pegging: USDT/USDC confidence crisis
- Exchange issues: Binance faces regulatory shutdown

Expected Impact on Holdings:
- BTC: -50% (becomes "safe haven" within crypto)
- ETH: -70% (DeFi ecosystem under attack)
- SOL: -85% (high DeFi exposure, exchange risk)
- AVAX: -80% (similar risk profile to SOL)
- MATIC: -75% (Ethereum scaling concerns)
- LINK: -60% (oracle utility provides some protection)
- USDC: Potential 10% de-peg risk

Crisis Duration: 18-36 months historically

Stress Test Analysis:
1. Calculate portfolio value under this scenario
2. Assess liquidity availability during crisis
3. Determine which positions become toxic first
4. Evaluate survival probability and recovery timeline
5. Design crisis management strategy:
   - Pre-emptive risk reduction before crisis
   - Position cutting sequence during crisis
   - Opportunity accumulation strategy for recovery
   - Portfolio reconstruction for post-crisis environment

Show systematic crisis management framework.
    "#;

    println!("🚨 Running extreme stress test...");
    let stress_test = portfolio_analyst.prompt(_stress_test_prompt).await?;
    println!("⚠️ Stress Test Results:");
    println!("{}\n", stress_test);

    println!("✅ Portfolio risk analysis demo complete!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("🎯 Market Analysis with Reasoning - Rig-Native Intelligence");
    println!("============================================================");
    println!("Demonstrating sophisticated market analysis workflows without custom loops\n");

    // Run market analysis demonstrations
    if let Err(e) = demo_comprehensive_token_analysis().await {
        warn!("Token analysis demo failed: {}", e);
    }

    if let Err(e) = demo_cross_chain_opportunity_analysis().await {
        warn!("Cross-chain analysis demo failed: {}", e);
    }

    if let Err(e) = demo_intelligence_synthesis().await {
        warn!("Intelligence synthesis demo failed: {}", e);
    }

    if let Err(e) = demo_portfolio_risk_analysis().await {
        warn!("Portfolio risk analysis demo failed: {}", e);
    }

    println!("\n🎯 Key Market Analysis Capabilities:");
    println!("====================================");
    println!("✅ Multi-methodology token evaluation (technical + fundamental + sentiment)");
    println!("✅ Cross-chain opportunity discovery and execution planning");
    println!("✅ Multi-source intelligence synthesis with conflict resolution");
    println!("✅ Quantitative portfolio risk analysis and optimization");
    println!("✅ Scenario analysis and stress testing frameworks");
    println!("✅ Adaptive analysis based on changing market conditions");
    println!();
    println!("🔗 Integration with riglr blockchain tools:");
    println!("- Real-time data feeds from blockchain APIs enhance analysis");
    println!("- Cross-chain tools enable opportunity execution");
    println!("- Portfolio tools provide current position data");
    println!("- Risk management tools enable systematic position sizing");
    println!("- All analysis patterns work seamlessly with tool integration");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_market_analysis_patterns() {
        // Test validates market analysis pattern concepts
        // Note: This test requires OPENAI_API_KEY to be set for actual agent testing
        if std::env::var(OPENAI_API_KEY).is_ok() {
            let openai_client = openai::Client::new(&std::env::var(OPENAI_API_KEY).unwrap());
            let model = openai_client.completion_model("gpt-3.5-turbo");

            let analyst = AgentBuilder::new(model)
                .preamble("You are a market analysis testing assistant.")
                .build();

            // Test basic agent construction - agent was successfully created
        } else {
            // Skip actual agent testing if no API key is available - OPENAI_API_KEY not set
        }
    }

    #[test]
    fn test_market_data_structure() {
        let data = json!({
            "technical_data": {"price": "$163.50", "rsi": 58},
            "fundamental_data": {"tps_capability": "65,000+"}
        });

        assert!(data.get("technical_data").is_some());
        assert!(data.get("fundamental_data").is_some());
    }

    #[test]
    fn test_cross_chain_data_structure() {
        let cross_chain = json!({
            "solana_ecosystem": {"native_yield": "5.8%"},
            "ethereum_ecosystem": {"native_yield": "4.3%"}
        });

        assert!(cross_chain.get("solana_ecosystem").is_some());
        assert!(cross_chain.get("ethereum_ecosystem").is_some());
    }

    #[test]
    fn test_portfolio_data_structure() {
        let portfolio = json!({
            "total_value": "$125,000",
            "positions": {
                "BTC": {"value": "$37,500", "pct": "30%"}
            }
        });

        assert_eq!(portfolio["total_value"], "$125,000");
        assert!(portfolio["positions"].get("BTC").is_some());
    }
}
