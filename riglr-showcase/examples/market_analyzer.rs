//! Market Analysis with Reasoning Examples
//!
//! This example demonstrates sophisticated market analysis workflows using rig's
//! native reasoning capabilities combined with riglr tools. Shows how agents can
//! perform complex multi-step analysis, cross-chain opportunity discovery,
//! and adaptive decision making.
//!
//! Key Features:
//! 1. Multi-source data analysis and synthesis
//! 2. Cross-chain opportunity identification
//! 3. Risk-adjusted decision making
//! 4. Adaptive analysis based on market conditions
//! 5. Complex workflow orchestration through natural conversation

use anyhow::Result;
use riglr_core::signer::SignerContext;
use riglr_solana_tools::LocalSolanaSigner;
// use riglr_solana_tools::{get_sol_balance, get_spl_token_balance, perform_jupiter_swap};
// use rig::agent::AgentBuilder;
use serde::{Deserialize, Serialize};
use solana_sdk::signer::{keypair::Keypair, Signer};
use std::sync::Arc;
use tracing::{info, warn};

/// Market analysis data structures for comprehensive evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
struct TokenAnalysis {
    symbol: String,
    address: String,
    current_price: f64,
    market_cap: Option<u64>,
    volume_24h: Option<u64>,
    price_change_24h: f64,
    liquidity_score: f64,
    volatility_score: f64,
    sentiment_score: f64,
    technical_score: f64,
    fundamental_score: f64,
    overall_rating: AnalysisRating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
enum AnalysisRating {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
struct CrossChainOpportunity {
    opportunity_type: OpportunityType,
    source_chain: String,
    target_chain: String,
    estimated_profit: f64,
    risk_level: RiskLevel,
    execution_complexity: ComplexityLevel,
    time_sensitivity: TimeSensitivity,
    required_capital: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
enum OpportunityType {
    Arbitrage,
    YieldFarming,
    LiquidityMining,
    StakingRewards,
    Bridge,
    Governance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
enum RiskLevel {
    Low,
    Medium,
    High,
    Extreme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
enum ComplexityLevel {
    Simple,   // Single transaction
    Medium,   // 2-3 transactions
    Complex,  // Multiple steps, timing required
    Advanced, // Cross-chain, multiple protocols
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
enum TimeSensitivity {
    None,   // Opportunity exists for days/weeks
    Low,    // Several hours available
    Medium, // 1-2 hours window
    High,   // Minutes to act
}

/// Example 1: Comprehensive Token Analysis Workflow
///
/// Demonstrates multi-step token analysis combining technical, fundamental,
/// and sentiment analysis with risk assessment and actionable recommendations.
async fn comprehensive_token_analysis() -> Result<()> {
    info!("Starting comprehensive token analysis workflow...");

    // TODO: Commented out due to rig API changes
    /*
        let agent = AgentBuilder::new("gpt-4")
            .preamble(r#"
    You are an advanced crypto market analyst specializing in comprehensive token evaluation.
    Your analysis framework combines multiple methodologies for holistic assessment.

    ANALYSIS FRAMEWORK:

    TECHNICAL ANALYSIS:
    - Price action and chart patterns
    - Support and resistance levels
    - Volume analysis and trend confirmation
    - Momentum indicators (RSI, MACD, etc.)
    - Moving averages and trend strength

    FUNDAMENTAL ANALYSIS:
    - Protocol utility and value proposition
    - Team and development activity
    - Tokenomics and supply dynamics
    - Partnership and adoption metrics
    - Competitive positioning

    SENTIMENT ANALYSIS:
    - Social media sentiment and mentions
    - Community engagement and growth
    - Influencer opinions and analysis
    - Market psychology indicators
    - Fear & greed levels

    ON-CHAIN ANALYSIS:
    - Transaction volume and active addresses
    - Whale movement patterns
    - Exchange flow analysis
    - Staking and governance participation
    - Network health metrics

    RISK ASSESSMENT:
    - Liquidity analysis and market depth
    - Smart contract risk evaluation
    - Regulatory and compliance factors
    - Market correlation and concentration
    - Volatility and drawdown analysis

    Your goal: Provide actionable investment recommendations with clear reasoning,
    risk assessment, and specific entry/exit strategies.
            "#.trim())
            .tool(get_sol_balance)
            .tool(get_spl_token_balance)
            .tool(perform_jupiter_swap)
            .max_tokens(3000)
            .build();
        */

    let keypair = Keypair::new();
    let signer = Arc::new(LocalSolanaSigner::new(
        keypair.insecure_clone(),
        "https://api.devnet.solana.com".to_string(),
    ));

    SignerContext::with_signer(signer, async move {
        let user_address = keypair.pubkey().to_string();

        let analysis_context = r#"
TOKEN ANALYSIS REQUEST: SOL (Solana)

CURRENT MARKET DATA:
- Price: $148.50 (24h change: -2.3%)
- Market Cap: $71B (Rank #4)
- 24h Volume: $3.2B
- Circulating Supply: 478M SOL
- Max Supply: No hard cap

TECHNICAL INDICATORS:
- 20-day MA: $142.30 (price above)
- 50-day MA: $135.80 (price above)
- RSI: 58 (neutral)
- MACD: Bullish crossover 3 days ago
- Volume: 120% of 30-day average

FUNDAMENTAL FACTORS:
- Network TPS: 3,000+ transactions per second
- Active developers: Growing ecosystem
- DeFi TVL: $6.8B across Solana protocols
- NFT marketplace volume: Leading platform
- Institutional adoption: Increasing

SENTIMENT INDICATORS:
- Social sentiment: 72% positive mentions
- Reddit discussions: High engagement
- Twitter mentions: Trending upward
- Google search trends: Stable interest
- Analyst ratings: 8/10 average

ON-CHAIN METRICS:
- Daily active addresses: 1.2M (growing)
- Transaction fees: Low and stable
- Staking participation: 68% of supply
- Validator count: 3,400+ (decentralized)
- Network uptime: 99.95% (improved)
        "#;

        let _comprehensive_prompt = format!(r#"
Please perform a comprehensive analysis of SOL using your multi-methodology framework.

My portfolio context:
- Wallet: {}
- Current SOL exposure: Will check with tools
- Risk tolerance: Medium to high
- Investment horizon: 3-6 months
- Available capital: $50,000

{}

Analysis Request:
1. Check my current SOL balance and calculate position sizing
2. Perform technical analysis with specific entry/exit levels
3. Evaluate fundamental strength and growth prospects
4. Assess sentiment and market psychology factors
5. Analyze on-chain metrics and network health
6. Provide comprehensive risk assessment
7. Generate specific trading recommendations with:
   - Entry strategy and optimal timing
   - Position sizing based on risk assessment
   - Target prices and profit-taking levels
   - Stop-loss levels and risk management
   - Timeline and monitoring criteria

Please be thorough and show your reasoning process across all analysis dimensions.
        "#, user_address, analysis_context);

        // let analysis_response = agent.prompt(&comprehensive_prompt).await?;
        let analysis_response = "COMPREHENSIVE SOL ANALYSIS: Technical analysis shows bullish momentum with price above key moving averages. Fundamental strength from growing developer ecosystem and institutional adoption. On-chain metrics indicate healthy network growth. Recommendation: BUY with target $165-180, stop-loss $135.";
        info!("Comprehensive analysis: {}", analysis_response);

        // Follow up with comparative analysis
        let _comparative_prompt = r#"
Now please compare SOL against its main competitors and alternative opportunities:

COMPARISON ANALYSIS REQUEST:
Compare SOL vs:
1. ETH (Ethereum) - established smart contract leader
2. AVAX (Avalanche) - similar high-performance chain
3. MATIC (Polygon) - Ethereum scaling solution
4. ADA (Cardano) - academic approach to blockchain

For each comparison:
- Relative valuation metrics (P/E ratios, network value)
- Technology advantages and disadvantages
- Ecosystem development and adoption
- Market positioning and competitive moats
- Risk/reward profiles for current market conditions

Based on this comparative analysis:
- Should I increase, decrease, or maintain SOL exposure?
- Are there better risk-adjusted opportunities in alternatives?
- How should I allocate between SOL and competitors?
- What market conditions would change this assessment?

Provide specific allocation recommendations with reasoning.
        "#;

        // let comparative_response = agent.prompt(comparative_prompt).await?;
        let comparative_response = "COMPARATIVE ANALYSIS: SOL offers better risk-adjusted returns vs competitors. Superior TPS vs ETH, lower fees vs AVAX, stronger ecosystem vs ADA. Recommend maintaining 60% SOL allocation vs 25% ETH, 15% alternatives for optimal portfolio diversification.";
        info!("Comparative analysis: {}", comparative_response);

        // Test analysis adaptation to changing conditions
        let _market_change_prompt = r#"
MARKET UPDATE: Major news just broke affecting your analysis:

NEWS: Federal Reserve announces crypto-friendly regulation framework
- Clear guidelines for DeFi protocols
- Staking rewards classified as utility, not securities
- Institutional custody solutions approved
- Major compliance barriers removed

IMMEDIATE MARKET REACTION:
- SOL up 8% in 30 minutes to $160.50
- All altcoins rallying on the news
- Institutional buying interest spiking
- Options volatility increasing

ANALYSIS UPDATE NEEDED:
How does this fundamental news change your analysis and recommendations?

Please:
1. Reassess the fundamental analysis given regulatory clarity
2. Update technical analysis with new price action
3. Evaluate how this changes risk/reward profile
4. Adjust position sizing and entry strategy recommendations
5. Consider if timing of entry should be accelerated
6. Update comparison with competitors (how do they benefit differently?)

Show how you adapt comprehensive analysis to major news events.
        "#;

        // let adaptation_response = agent.prompt(market_change_prompt).await?;
        let adaptation_response = "REGULATORY UPDATE ANALYSIS: Crypto-friendly regulation is a major positive catalyst. Upgrading SOL target to $185-200 and increasing recommended allocation to 70%. Regulatory clarity reduces risk premium and attracts institutional capital.";
        info!("Analysis adaptation: {}", adaptation_response);

        Ok(())
    }).await.map_err(|_e| anyhow::anyhow!("Token analysis failed"))?;

    Ok(())
}

/// Example 2: Cross-Chain Opportunity Discovery
///
/// Shows sophisticated analysis across multiple blockchains to identify
/// arbitrage, yield, and strategic opportunities requiring cross-chain execution.
async fn cross_chain_opportunity_analysis() -> Result<()> {
    info!("Starting cross-chain opportunity analysis...");

    // TODO: Commented out due to rig API changes
    /*
        let agent = AgentBuilder::new("gpt-4")
            .preamble(r#"
    You are a cross-chain opportunity analyst specializing in identifying profit opportunities
    across different blockchain ecosystems.

    CROSS-CHAIN OPPORTUNITY FRAMEWORK:

    ARBITRAGE OPPORTUNITIES:
    - Price differences for same tokens across chains
    - DEX price discrepancies (Uniswap vs Raydium vs PancakeSwap)
    - CEX vs DEX pricing gaps
    - Bridge token premium/discounts
    - Timing-based arbitrage during high volatility

    YIELD OPPORTUNITIES:
    - Cross-chain yield farming strategies
    - Staking rewards comparison across chains
    - Liquidity mining programs with token emissions
    - Cross-chain lending/borrowing rate differences
    - Governance token farming opportunities

    STRATEGIC OPPORTUNITIES:
    - New protocol launches with airdrop potential
    - Cross-chain bridge liquidity incentives
    - Multi-chain governance participation
    - Ecosystem migration opportunities
    - First-mover advantages in new chains

    RISK ANALYSIS:
    - Bridge security and failure risks
    - Chain congestion and timing risks
    - Impermanent loss calculations
    - Smart contract risks across protocols
    - Regulatory differences between chains

    EXECUTION COMPLEXITY:
    - Transaction sequencing requirements
    - Gas optimization across chains
    - Timing coordination challenges
    - Capital efficiency optimization
    - Monitoring and management overhead

    Your goal: Find high-probability, risk-adjusted opportunities across chains
    and provide detailed execution plans.
            "#.trim())
            .tool(get_sol_balance)
            .tool(get_spl_token_balance)
            .tool(perform_jupiter_swap)
            .max_tokens(3000)
            .build();
        */

    let keypair = Keypair::new();
    let signer = Arc::new(LocalSolanaSigner::new(
        keypair.insecure_clone(),
        "https://api.devnet.solana.com".to_string(),
    ));

    SignerContext::with_signer(signer, async move {
        let user_address = keypair.pubkey().to_string();

        let opportunity_context = r#"
CROSS-CHAIN MARKET CONDITIONS:

SOLANA ECOSYSTEM:
- Native SOL DeFi: Jupiter, Orca, Raydium active
- USDC liquidity: Deep on all major pools
- Transaction costs: $0.0025 average
- Speed: 400ms block times
- Current opportunity: New yield farms launching

ETHEREUM ECOSYSTEM:
- Gas costs: $12-25 per transaction
- DeFi maturity: Largest TVL ($45B+)
- Arbitrage opportunities: Limited by gas costs
- Staking: ETH 2.0 yields 4.2%
- Current opportunity: Layer 2 migration incentives

POLYGON ECOSYSTEM:
- Low gas costs: $0.01-0.10 per transaction
- Bridge activity: High USDC/USDT flows
- DeFi growth: QuickSwap, Aave, Curve active
- Speed: 2-second block times
- Current opportunity: MATIC staking rewards

BSC ECOSYSTEM:
- PancakeSwap dominant DEX
- High yield farming rates (caution on sustainability)
- Bridge volumes: Strong BNB/BUSD flows
- Regulatory uncertainty in some regions
- Current opportunity: New launchpad projects

ARBITRUM/OPTIMISM:
- Lower Ethereum gas costs
- Growing TVL and user adoption
- Arbitrage with mainnet Ethereum
- Current opportunity: Airdrop farming potential

CROSS-CHAIN BRIDGES:
- Wormhole: Solana ↔ Ethereum, other chains
- Multichain: High volume, multiple routes
- LayerZero: Omnichain applications
- Bridge token premiums currently 0.1-0.5%
        "#;

        let _discovery_prompt = format!(r#"
Please analyze cross-chain opportunities available right now.

My multi-chain setup:
- Solana wallet: {}
- Available on Ethereum: Will need to bridge funds
- Available on Polygon: Can bridge from Ethereum
- Available on BSC: Can use Binance Bridge
- Total available capital: $75,000
- Risk tolerance: Medium (will take smart contract risk for good returns)
- Time commitment: Can actively manage for 2-3 weeks

{}

Discovery Analysis Request:
1. Check my current Solana balances and liquidity
2. Identify top 5 cross-chain opportunities by risk-adjusted return
3. For each opportunity, provide:
   - Expected annual percentage return (APR/APY)
   - Risk level and specific risk factors
   - Capital requirements and optimal position size
   - Step-by-step execution plan
   - Time commitment and monitoring requirements
   - Exit strategy and profit-taking approach

4. Rank opportunities by Sharpe ratio (return per unit of risk)
5. Recommend optimal portfolio allocation across opportunities
6. Identify any combination strategies that work together

Please be specific about execution details and timing requirements.
        "#, user_address, opportunity_context);

        // let discovery_response = agent.prompt(&discovery_prompt).await?;
        let discovery_response = "CROSS-CHAIN OPPORTUNITIES IDENTIFIED: 1) USDC arbitrage Solana->Polygon (0.23% profit), 2) ETH staking rewards differential, 3) New yield farm on Arbitrum (45% APY), 4) Bridge liquidity mining (25% APY), 5) Airdrop farming on LayerZero protocols.";
        info!("Cross-chain opportunity discovery: {}", discovery_response);

        // Analyze specific arbitrage opportunity in detail
        let _arbitrage_deep_dive = r#"
ARBITRAGE OPPORTUNITY DEEP DIVE:

USDC Price Discrepancy Detected:
- Solana USDC: $0.9992 (Orca DEX)
- Ethereum USDC: $1.0000 (Uniswap)
- Polygon USDC: $1.0015 (QuickSwap)
- Price spread: 0.23% (Polygon highest, Solana lowest)

BRIDGE ANALYSIS:
- Wormhole bridge fee: 0.1% + $5 fixed
- Bridge time: 15-30 minutes average
- Current bridge volume: Normal (not congested)
- Historical success rate: 99.8%

EXECUTION WINDOW:
- Spread has been stable for 2 hours
- Volume suggests sustainability
- Gas costs favorable on all chains
- Estimated window: 4-6 hours before convergence

Please analyze this specific arbitrage opportunity:

1. Calculate exact profit potential for different capital sizes ($10K, $25K, $50K)
2. Factor in all transaction costs (DEX fees, bridge fees, gas)
3. Analyze execution risks (bridge failure, price movement, slippage)
4. Determine optimal trade size for risk-adjusted returns
5. Create step-by-step execution timeline
6. Set up monitoring criteria to know when to exit
7. Identify potential complications and contingency plans

Should I execute this arbitrage? If yes, provide exact execution instructions.
        "#;

        // let arbitrage_response = agent.prompt(arbitrage_deep_dive).await?;
        let arbitrage_response = "ARBITRAGE EXECUTION: For $25K capital, net profit ~$50 after all fees (0.2% return). Risk: Bridge failure, slippage. RECOMMENDATION: Execute with $15K to minimize risk while capturing opportunity. Monitor bridge congestion closely.";
        info!("Arbitrage deep dive: {}", arbitrage_response);

        // Test opportunity evaluation under time pressure
        let _urgent_opportunity_prompt = r#"
🚨 URGENT OPPORTUNITY ALERT 🚨 [Time Sensitive - 30 minutes remaining]

NEW YIELD FARM LAUNCH:
- Protocol: New Solana-based yield aggregator
- Rewards: 180% APY for first week (decreasing after)
- Pool: SOL-USDC liquidity pair
- Additional rewards: Governance tokens with potential airdrop value
- Current TVL: $2.3M (room for growth)
- Audit: Completed by reputable firm
- Team: Anonymous but experienced developers

OPPORTUNITY DETAILS:
- LP rewards: 45% APY base yield
- Token emissions: 135% APY additional (first week only)
- Governance tokens: Unknown value but similar protocols saw 10-50x gains
- Pool safety: Impermanent loss risk due to SOL volatility
- Competition: Only 847 participants so far

TIME CONSTRAINTS:
- Farm launches in 28 minutes
- First 1000 participants get bonus multiplier
- Early participant NFT with potential future utility
- High gas activity expected at launch

QUICK ANALYSIS NEEDED:
1. Check my SOL and USDC balances for immediate participation
2. Rapid risk assessment: Is 180% APY sustainable or dump risk?
3. Calculate optimal position size for risk/reward
4. Identify red flags or deal-breakers
5. Fast execution plan if opportunity is valid
6. Set up monitoring for quick exit if needed

TIME IS CRITICAL - Should I participate? If yes, what's the execution plan?
        "#;

        // let urgent_response = agent.prompt(urgent_opportunity_prompt).await?;
        let urgent_response = "URGENT OPPORTUNITY ASSESSMENT: 180% APY unsustainable - likely dump after week 1. However, first-week participants get bonus. RECOMMENDATION: Allocate $10K max, plan exit after 3-5 days, monitor for governance token value.";
        info!("Urgent opportunity analysis: {}", urgent_response);

        Ok(())
    }).await.map_err(|_e| anyhow::anyhow!("Cross-chain analysis failed"))?;

    Ok(())
}

/// Example 3: Risk-Adjusted Portfolio Analysis
///
/// Demonstrates sophisticated portfolio risk analysis including correlation analysis,
/// scenario testing, and adaptive risk management based on market conditions.
async fn risk_adjusted_portfolio_analysis() -> Result<()> {
    info!("Starting risk-adjusted portfolio analysis...");

    // TODO: Commented out due to rig API changes
    /*
        let agent = AgentBuilder::new("gpt-4")
            .preamble(r#"
    You are a quantitative portfolio analyst specializing in risk-adjusted return optimization
    for crypto portfolios.

    RISK ANALYSIS FRAMEWORK:

    QUANTITATIVE RISK METRICS:
    - Value at Risk (VaR) calculations
    - Maximum drawdown analysis
    - Sharpe and Sortino ratios
    - Beta analysis vs market (BTC/ETH)
    - Correlation matrix between holdings
    - Volatility clustering analysis

    SCENARIO ANALYSIS:
    - Bull market scenarios (+50% crypto market)
    - Bear market scenarios (-70% crypto market)
    - Black swan events (major protocol hacks, regulatory bans)
    - Correlation breakdown scenarios
    - Liquidity crisis scenarios

    PORTFOLIO CONSTRUCTION:
    - Modern Portfolio Theory application
    - Risk parity allocation strategies
    - Factor-based diversification
    - Rebalancing frequency optimization
    - Position sizing based on Kelly criterion

    DYNAMIC RISK MANAGEMENT:
    - Adaptive position sizing based on volatility
    - Correlation-based hedging strategies
    - Momentum and mean-reversion regime detection
    - Drawdown-based risk reduction protocols
    - Liquidity management for different market conditions

    ALTERNATIVE RISK MEASURES:
    - Tail risk and extreme event preparation
    - Liquidity risk assessment
    - Counterparty risk evaluation
    - Smart contract risk quantification
    - Regulatory risk scoring

    Your goal: Optimize portfolio construction for maximum risk-adjusted returns
    while maintaining downside protection.
            "#.trim())
            .tool(get_sol_balance)
            .tool(get_spl_token_balance)
            .tool(perform_jupiter_swap)
            .max_tokens(3000)
            .build();
        */

    let keypair = Keypair::new();
    let signer = Arc::new(LocalSolanaSigner::new(
        keypair.insecure_clone(),
        "https://api.devnet.solana.com".to_string(),
    ));

    SignerContext::with_signer(signer, async move {
        let user_address = keypair.pubkey().to_string();

        let portfolio_context = r#"
CURRENT PORTFOLIO ANALYSIS REQUEST:

PORTFOLIO HOLDINGS (Total: $150,000):
- BTC: 30% ($45,000) - Digital gold, store of value
- ETH: 25% ($37,500) - Smart contract platform leader
- SOL: 20% ($30,000) - High-performance blockchain
- AVAX: 10% ($15,000) - Ethereum competitor
- MATIC: 8% ($12,000) - Scaling solution
- LINK: 5% ($7,500) - Oracle network
- Cash/USDC: 2% ($3,000) - Dry powder

HISTORICAL PERFORMANCE (6 months):
- Total return: +28% (vs BTC +15%, ETH +22%)
- Max drawdown: -35% (during May crash)
- Volatility: 65% annualized
- Sharpe ratio: 0.85
- Best month: +45% (November rally)
- Worst month: -28% (May crash)

CORRELATION MATRIX (last 90 days):
- BTC-ETH: 0.82 (high correlation)
- SOL-AVAX: 0.91 (very high correlation)
- All altcoins vs BTC: 0.70-0.85 range
- USDC correlations: Near zero (good)
- Major concern: Portfolio highly correlated during crashes

CURRENT MARKET CONDITIONS:
- Market regime: Bull market with high volatility
- VIX crypto equivalent: 85 (elevated fear)
- Institutional flow: Mixed (some buying, some selling)
- Regulatory environment: Uncertain but improving
- Technical setup: Above key support, below resistance

RISK TOLERANCE PROFILE:
- Can handle 50% drawdown maximum
- Target return: 25%+ annual (crypto bull market)
- Liquidity needs: May need 20% in 6 months
- Risk capacity: High (young, high income, long term)
        "#;

        let _risk_analysis_prompt = format!(r#"
Please perform comprehensive risk-adjusted portfolio analysis.

My wallet for verification: {}

{}

Risk Analysis Request:
1. Check current Solana balances to verify portfolio data
2. Calculate current portfolio risk metrics:
   - Value at Risk (95% confidence)
   - Expected maximum drawdown
   - Sharpe and Sortino ratios
   - Portfolio beta vs crypto market

3. Analyze portfolio construction issues:
   - Correlation problems and concentration risks
   - Diversification effectiveness
   - Position sizing optimization opportunities
   - Rebalancing needs

4. Perform scenario analysis:
   - Bull case: +100% crypto market scenario
   - Bear case: -70% crypto market crash
   - Correlation breakdown: When altcoins decouple
   - Black swan: Major protocol hack affecting holdings

5. Generate optimization recommendations:
   - Optimal portfolio allocation for current conditions
   - Position sizing adjustments
   - New positions to add for better diversification
   - Positions to reduce for risk management
   - Rebalancing schedule and triggers

6. Set up dynamic risk management:
   - Stop-loss levels and position sizing rules
   - Volatility-based position adjustments
   - Correlation monitoring and hedging strategies
   - Liquidity management for different market phases

Please provide specific, actionable recommendations with quantitative targets.
        "#, user_address, portfolio_context);

        // let risk_analysis_response = agent.prompt(&risk_analysis_prompt).await?;
        let risk_analysis_response = "PORTFOLIO RISK ANALYSIS: Current portfolio shows high correlation risk (0.85 average). VaR at 95%: -$52K. Recommendation: Reduce altcoin allocation from 45% to 30%, increase cash to 10%, add non-correlated assets. Rebalance monthly or on 15% drift.";
        info!("Risk-adjusted portfolio analysis: {}", risk_analysis_response);

        // Test portfolio stress under extreme conditions
        let _stress_test_prompt = r#"
PORTFOLIO STRESS TEST: Extreme market scenario simulation

CRISIS SCENARIO: "DeFi Winter" - Regulatory crackdown and liquidity crisis
- SEC classifies most DeFi tokens as securities
- Major exchanges delist altcoins in US
- Institutional selling accelerates
- Credit crisis hits crypto lending protocols
- Stablecoin de-pegging concerns emerge

EXPECTED IMPACT ON HOLDINGS:
- BTC: -40% (flight to "safety" within crypto)
- ETH: -60% (DeFi ecosystem concerns)
- SOL: -75% (high DeFi exposure, exchange risk)
- AVAX: -70% (similar to ETH concerns)
- MATIC: -65% (Ethereum scaling, regulatory risk)
- LINK: -50% (oracle utility, some safety)
- USDC: Potential 5% depeg risk

MARKET CONDITIONS DURING CRISIS:
- Liquidity: Severely impaired, wide bid-ask spreads
- Correlations: All altcoins approach 1.0 with BTC/ETH
- Volatility: Extreme (150%+ annualized)
- Time horizon: 6-18 month crisis duration
- Recovery timeline: 2-4 years historically

STRESS TEST QUESTIONS:
1. Calculate exact portfolio value under this scenario
2. Determine if maximum drawdown exceeds my 50% tolerance
3. Identify which positions become illiquid first
4. Assess if I can meet my 6-month liquidity needs
5. Evaluate portfolio survival probability and recovery time

CRISIS ADAPTATION STRATEGY:
- Should I pre-emptively reduce risk before crisis hits?
- Which positions should be cut first if crisis emerges?
- How should I rebalance during the crisis for optimal recovery?
- What opportunities might emerge during maximum pessimism?
- How can I position for post-crisis recovery?

Please provide crisis management recommendations and specific action triggers.
        "#;

        // let stress_test_response = agent.prompt(stress_test_prompt).await?;
        let stress_test_response = "STRESS TEST RESULTS: DeFi Winter scenario shows -62% portfolio decline ($93K loss). Exceeds risk tolerance. IMMEDIATE ACTION: Reduce crypto allocation to 30%, increase cash/stablecoins to 40%, add hedging positions. Emergency exit plan activated.";
        info!("Portfolio stress test: {}", stress_test_response);

        Ok(())
    }).await.map_err(|_e| anyhow::anyhow!("Portfolio analysis failed"))?;

    Ok(())
}

/// Example 4: Multi-Source Intelligence Synthesis
///
/// Shows how agents can synthesize information from multiple data sources
/// to create comprehensive market intelligence and actionable insights.
async fn multi_source_intelligence_synthesis() -> Result<()> {
    info!("Starting multi-source intelligence synthesis...");

    // TODO: Commented out due to rig API changes
    /*
        let agent = AgentBuilder::new("gpt-4")
            .preamble(r#"
    You are a market intelligence analyst that synthesizes information from multiple sources
    to generate comprehensive market insights and trading opportunities.

    INTELLIGENCE SOURCES FRAMEWORK:

    ON-CHAIN DATA:
    - Transaction volumes and active addresses
    - Whale movement and large holder behavior
    - Exchange flows (accumulation vs distribution)
    - Smart contract interactions and DeFi activity
    - Network health and development metrics

    MARKET DATA:
    - Price action and technical indicators
    - Volume analysis and market microstructure
    - Options flow and derivatives positioning
    - Funding rates and perpetual futures
    - Cross-exchange arbitrage opportunities

    SOCIAL INTELLIGENCE:
    - Social media sentiment and trending topics
    - Influencer opinions and institutional commentary
    - Developer activity and GitHub commits
    - Community engagement and adoption metrics
    - News flow analysis and event impact

    MACRO ENVIRONMENT:
    - Regulatory developments and policy changes
    - Institutional adoption and investment flows
    - Traditional market correlations and spillovers
    - Central bank policy and monetary conditions
    - Geopolitical events affecting crypto

    FUNDAMENTAL ANALYSIS:
    - Protocol development and roadmap progress
    - Partnership announcements and integrations
    - Token utility and value accrual mechanisms
    - Competitive landscape and market share
    - Team execution and governance quality

    SYNTHESIS METHODOLOGY:
    - Weight each source by reliability and relevance
    - Identify convergent vs divergent signals
    - Resolve conflicts through deeper analysis
    - Generate confidence-weighted conclusions
    - Provide specific actionable recommendations

    Your goal: Create high-conviction investment theses backed by multi-source intelligence.
            "#.trim())
            .tool(get_sol_balance)
            .tool(get_spl_token_balance)
            .tool(perform_jupiter_swap)
            .max_tokens(3500)
            .build();
        */

    let keypair = Keypair::new();
    let signer = Arc::new(LocalSolanaSigner::new(
        keypair.insecure_clone(),
        "https://api.devnet.solana.com".to_string(),
    ));

    SignerContext::with_signer(signer, async move {
        let user_address = keypair.pubkey().to_string();

        let intelligence_context = r#"
MULTI-SOURCE INTELLIGENCE ANALYSIS: Solana (SOL) Investment Thesis

ON-CHAIN INTELLIGENCE:
- Daily active addresses: 1.2M (+15% vs 30-day avg)
- Transaction volume: $2.8B daily (+22% vs 30-day avg)
- New wallet creation: 45K daily (accelerating growth)
- Whale accumulation: Top 100 addresses added 2.1M SOL last 7 days
- Exchange outflows: Net 890K SOL withdrawn (bullish accumulation)
- Staking participation: 68.5% (+0.8% monthly growth)
- DeFi TVL: $6.2B (+18% month-over-month)
- NFT trading volume: $12M daily (steady)

MARKET DATA SIGNALS:
- Price: $148.50 (-2.3% 24h, +12% 7d, +78% 30d)
- Volume profile: Above-average institutional size trades
- Options flow: 65% call volume (bullish bias)
- Futures funding: +0.08% (moderate bullish sentiment)
- Perpetual open interest: $1.8B (healthy, not overleveraged)
- Cross-exchange basis: Coinbase premium +0.3% (institutional buying)

SOCIAL INTELLIGENCE:
- Twitter sentiment: 74% positive (above 90-day avg of 62%)
- Weighted social volume: Trending upward
- Developer sentiment: High based on GitHub activity
- Community growth: Reddit subscribers +8% monthly
- Influencer mentions: Increasing positive coverage
- Google Trends: Search interest up 35% vs last month
- News sentiment: 85% positive coverage

MACRO ENVIRONMENT:
- Regulatory clarity improving (Solana not mentioned in SEC issues)
- Institutional adoption: PayPal, Visa exploring Solana
- Traditional finance integration: Multiple RWA projects launching
- Central bank policy: Fed pause supportive for risk assets
- Broader crypto market: Bitcoin stable, altcoin season emerging

FUNDAMENTAL DEVELOPMENTS:
- Solana Mobile phone: Strong pre-orders, dApp ecosystem growing
- Firedancer validator: Major performance upgrade in development
- Partnership pipeline: Enterprise adoption increasing
- Developer ecosystem: 2,500+ active developers (+15% YoY)
- Competitive position: Gaining market share vs Ethereum L2s
- Token economics: Inflation decreasing, fee burn increasing

CONFLICTING SIGNALS:
- Network stability concerns vs recent uptime improvements
- High institutional interest vs retail sentiment mixed
- Strong fundamentals vs technical resistance at $155
- Growing ecosystem vs competition from other L1s
        "#;

        let _synthesis_prompt = format!(r#"
Please synthesize this multi-source intelligence into actionable investment insights.

My wallet for balance verification: {}

{}

Intelligence Synthesis Request:
1. Check my current SOL balance and position
2. Weight and evaluate each intelligence source:
   - Which signals are most reliable and significant?
   - Where do sources converge vs diverge?
   - How should conflicting signals be resolved?

3. Generate investment thesis:
   - Bull case scenario with price targets and timeline
   - Bear case risks and potential negative catalysts
   - Base case most likely outcome with probability
   - Key factors that would change the thesis

4. Create actionable trading strategy:
   - Optimal entry levels and position sizing
   - Risk management and stop-loss levels
   - Profit-taking strategy and target allocations
   - Timeline for investment thesis to play out

5. Identify key monitoring criteria:
   - Which data sources to track most closely
   - Early warning signals that thesis is breaking down
   - Catalysts that could accelerate the thesis
   - Rebalancing triggers and decision points

6. Generate confidence assessment:
   - Overall conviction level (1-10 scale)
   - Risk-adjusted expected return
   - Probability-weighted outcomes
   - Comparison to alternative investment opportunities

Provide specific recommendations with quantitative targets and reasoning.
        "#, user_address, intelligence_context);

        // let synthesis_response = agent.prompt(&synthesis_prompt).await?;
        let synthesis_response = "INTELLIGENCE SYNTHESIS: High conviction BUY thesis for SOL. On-chain whale accumulation + market premium + positive social sentiment align. Bull case: $180-220 (6-month). Base case: $165-185 (70% probability). Position sizing: 60-70% allocation recommended.";
        info!("Multi-source intelligence synthesis: {}", synthesis_response);

        // Test handling of contradictory intelligence
        let _contradictory_signals_prompt = r#"
INTELLIGENCE UPDATE: Contradictory signals emerging - need conflict resolution

NEW CONFLICTING INFORMATION:
- On-chain: Whale addresses actually distributed 1.5M SOL (previous data was wrong)
- Market data: Large options position expires Friday, could trigger volatility
- Social: Major influencer turned bearish, citing technical concerns
- Macro: Fed hawkish comments suggest possible rate hike, risk-off sentiment
- Fundamental: Competitor (Ethereum L2) announced major improvement, threatens market share

INTELLIGENCE CONFLICTS TO RESOLVE:
1. Whale behavior: Accumulation vs distribution (which data is correct?)
2. Market structure: Bullish flow vs volatility risk from options expiry
3. Sentiment: Community optimism vs influential bearish opinions
4. Macro backdrop: Crypto-friendly regulation vs monetary tightening risk
5. Competitive position: Solana improvements vs Ethereum L2 competition

ANALYSIS METHODOLOGY CHALLENGE:
How do you handle contradictory intelligence sources? Please demonstrate:

1. Information verification: How to determine which sources are more reliable
2. Signal weighting: How to weight conflicting information appropriately
3. Uncertainty quantification: How to express confidence with conflicting data
4. Decision making: How to make investment decisions with incomplete/conflicting info
5. Adaptive thesis: How to update investment thesis as new information emerges

SPECIFIC CONFLICT RESOLUTION:
Given these new contradictory signals, should the investment thesis change?
- What's the new bull/bear/base case probability distribution?
- How should position sizing adjust for increased uncertainty?
- What additional data would help resolve the key conflicts?
- What's the decision framework when intelligence sources disagree?

Show your systematic approach to handling contradictory market intelligence.
        "#;

        // let conflict_resolution_response = agent.prompt(contradictory_signals_prompt).await?;
        let conflict_resolution_response = "CONFLICT RESOLUTION: Contradictory whale data is concerning. Downgrading conviction to MEDIUM. Reducing position sizing to 40-50% while gathering more reliable data sources. Weight on-chain data higher than social sentiment when conflicts arise.";
        info!("Contradictory signals resolution: {}", conflict_resolution_response);

        Ok(())
    }).await.map_err(|_e| anyhow::anyhow!("Intelligence synthesis failed"))?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for detailed logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting riglr market analysis examples...");

    println!("\n📊 Market Analysis with Reasoning Examples");
    println!("==========================================");
    println!("Demonstrating sophisticated market analysis workflows using rig + riglr\n");

    // Run market analysis examples
    println!("1️⃣  Comprehensive Token Analysis");
    println!("   Multi-methodology token evaluation with risk assessment...");
    if let Err(e) = comprehensive_token_analysis().await {
        warn!("Token analysis failed: {}", e);
    }

    println!("\n2️⃣  Cross-Chain Opportunity Discovery");
    println!("   Multi-blockchain opportunity identification and execution...");
    if let Err(e) = cross_chain_opportunity_analysis().await {
        warn!("Cross-chain analysis failed: {}", e);
    }

    println!("\n3️⃣  Risk-Adjusted Portfolio Analysis");
    println!("   Quantitative portfolio optimization and stress testing...");
    if let Err(e) = risk_adjusted_portfolio_analysis().await {
        warn!("Portfolio analysis failed: {}", e);
    }

    println!("\n4️⃣  Multi-Source Intelligence Synthesis");
    println!("   Combining multiple data sources for market intelligence...");
    if let Err(e) = multi_source_intelligence_synthesis().await {
        warn!("Intelligence synthesis failed: {}", e);
    }

    println!("\n✅ Market analysis examples completed!");
    println!("\nKey Analysis Capabilities:");
    println!("- Multi-methodology token evaluation");
    println!("- Cross-chain opportunity identification");
    println!("- Quantitative risk assessment");
    println!("- Intelligence source synthesis");
    println!("- Adaptive analysis based on market conditions");
    println!("- Complex decision-making workflows");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_analysis_serialization() {
        let analysis = TokenAnalysis {
            symbol: "SOL".to_string(),
            address: "So11111111111111111111111111111111111111112".to_string(),
            current_price: 148.50,
            market_cap: Some(71_000_000_000),
            volume_24h: Some(3_200_000_000),
            price_change_24h: -2.3,
            liquidity_score: 8.5,
            volatility_score: 7.2,
            sentiment_score: 7.4,
            technical_score: 6.8,
            fundamental_score: 8.1,
            overall_rating: AnalysisRating::Buy,
        };

        let serialized = serde_json::to_string(&analysis).unwrap();
        let deserialized: TokenAnalysis = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.symbol, "SOL");
        assert_eq!(deserialized.current_price, 148.50);
        assert!(matches!(deserialized.overall_rating, AnalysisRating::Buy));
    }

    #[tokio::test]
    async fn test_cross_chain_opportunity_creation() {
        let opportunity = CrossChainOpportunity {
            opportunity_type: OpportunityType::Arbitrage,
            source_chain: "Solana".to_string(),
            target_chain: "Ethereum".to_string(),
            estimated_profit: 2.3,
            risk_level: RiskLevel::Medium,
            execution_complexity: ComplexityLevel::Medium,
            time_sensitivity: TimeSensitivity::High,
            required_capital: 50000.0,
        };

        assert!(matches!(
            opportunity.opportunity_type,
            OpportunityType::Arbitrage
        ));
        assert!(matches!(opportunity.risk_level, RiskLevel::Medium));
        assert_eq!(opportunity.estimated_profit, 2.3);
    }
}
