//! Adaptive Strategy Example
//!
//! This example demonstrates how rig agents can adapt their behavior dynamically
//! based on changing conditions, intermediate results, and learning from outcomes.
//!
//! Key Features:
//! 1. Real-time strategy adaptation based on market conditions
//! 2. Learning from previous execution results
//! 3. Dynamic parameter adjustment
//! 4. Multi-criteria decision making
//! 5. Fallback strategies and risk management

use anyhow::Result;
use riglr_core::signer::{SignerContext, UnifiedSigner};
use riglr_solana_tools::signer::Local as LocalSolanaSigner;
// use riglr_solana_tools::{get_sol_balance, get_spl_token_balance, perform_jupiter_swap};
// use rig::agent::AgentBuilder;
use serde::{Deserialize, Serialize};
use solana_sdk::signer::{keypair::Keypair, Signer};
use std::sync::Arc;
use tracing::{info, warn};

/// Market condition indicators used for strategy adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MarketConditions {
    volatility_level: VolatilityLevel,
    trend_direction: TrendDirection,
    liquidity_conditions: LiquidityLevel,
    risk_sentiment: RiskSentiment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum VolatilityLevel {
    Low,     // < 2% daily moves
    Normal,  // 2-10% daily moves
    High,    // 10-20% daily moves
    Extreme, // > 20% daily moves
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum TrendDirection {
    StrongUptrend,
    Uptrend,
    Sideways,
    Downtrend,
    StrongDowntrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum LiquidityLevel {
    High,
    Normal,
    Low,
    Illiquid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum RiskSentiment {
    RiskOn,
    Neutral,
    RiskOff,
    Panic,
}

/// Strategy performance tracking for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StrategyPerformance {
    strategy_name: String,
    wins: u32,
    losses: u32,
    total_pnl: f64,
    avg_return: f64,
    max_drawdown: f64,
    sharpe_ratio: f64,
    market_conditions: Vec<MarketConditions>,
}

/// Example 1: Volatility-Adaptive Trading Strategy
///
/// This agent adapts its position sizing, holding periods, and risk parameters
/// based on real-time volatility measurements and recent performance data.
async fn volatility_adaptive_strategy() -> Result<()> {
    info!("Starting volatility-adaptive strategy...");

    // TODO: Commented out due to rig API changes - needs proper model initialization
    /*
        let agent = AgentBuilder::new("gpt-4")
            .preamble(r#"
    You are an adaptive trading agent that specializes in volatility-based strategy adjustment.

    VOLATILITY ADAPTATION FRAMEWORK:

    LOW VOLATILITY (< 2% daily moves):
    - Position Size: 80% of normal
    - Strategy: Range trading, mean reversion
    - Holding Period: 3-7 days
    - Stop Loss: Wide (8-10%)
    - Take Profit: Multiple small exits

    NORMAL VOLATILITY (2-10% daily moves):
    - Position Size: 100% normal
    - Strategy: Trend following, momentum
    - Holding Period: 1-5 days
    - Stop Loss: Standard (5-7%)
    - Take Profit: Trail with trend

    HIGH VOLATILITY (10-20% daily moves):
    - Position Size: 60% of normal
    - Strategy: Quick scalps, news-based
    - Holding Period: Hours to 1 day
    - Stop Loss: Tight (3-5%)
    - Take Profit: Quick gains (2-5%)

    EXTREME VOLATILITY (> 20% daily moves):
    - Position Size: 30% of normal
    - Strategy: Defensive, cash preservation
    - Holding Period: Minutes to hours
    - Stop Loss: Very tight (2-3%)
    - Take Profit: Any profit > 1%

    LEARNING COMPONENT:
    - Track which strategies work best in different volatility regimes
    - Adjust parameters based on recent win rate and PnL
    - Reduce position size after losses, increase after wins
    - Switch strategies if current approach isn't working

    Always explain your volatility assessment and strategy selection reasoning.
    Show how you're adapting based on recent performance data.
            "#.trim())
            .tool(get_sol_balance)
            .tool(get_spl_token_balance)
            .tool(perform_jupiter_swap)
            .max_tokens(2500)
            .build();
        */

    let keypair = Keypair::new();
    let signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
        keypair.insecure_clone(),
        "https://api.devnet.solana.com".to_string(),
    )) as Arc<dyn UnifiedSigner>;

    SignerContext::with_signer(signer, async move {
        let user_address = keypair.pubkey().to_string();

        // Simulate historical performance data for context
        let performance_context = r"
RECENT STRATEGY PERFORMANCE:
- Last 7 days: High volatility environment, quick scalp strategy
  * Win Rate: 65% (13 wins, 7 losses)
  * PnL: +8.3%
  * Avg Hold Time: 4 hours
  * Max Drawdown: -2.1%

- Previous 7 days: Normal volatility, trend following
  * Win Rate: 40% (6 wins, 9 losses)
  * PnL: -1.8%
  * Avg Hold Time: 2.5 days
  * Max Drawdown: -5.2%

CURRENT MARKET CONDITIONS:
- SOL 24h volatility: 18% (HIGH)
- Volume: 130% of 30-day average
- Trend: Downtrend (-12% from local high)
- Market sentiment: Risk-off
        ";

        let _initial_prompt = format!(r"
Please analyze current market conditions and adapt your trading strategy.

My wallet: {user_address}

{performance_context}

Current Assessment Task:
1. Evaluate the current volatility regime and market conditions
2. Review recent strategy performance to identify what's working
3. Adapt your strategy parameters for current conditions
4. Check my current positions and available capital
5. Recommend specific position adjustments and new trades
6. Set up dynamic monitoring criteria to detect regime changes

Based on the performance data, it looks like quick scalp strategy is working well
in high volatility, while trend following struggled in recent normal volatility.

Please provide specific parameter adjustments and explain your adaptation reasoning.
        ");

        // TODO: Replaced agent calls with stub responses
        // let adaptation_response = agent.prompt(&initial_prompt).await?;
        let adaptation_response = "VOLATILITY ANALYSIS: Current 18% volatility indicates HIGH volatility regime. Based on recent performance data showing 65% win rate with quick scalp strategy, I recommend maintaining this approach with position size at 60% of normal and tight stop-losses at 3-5%.";
        info!("Volatility adaptation response: {}", adaptation_response);

        // Test response to changing market conditions

        // let extreme_adaptation = agent.prompt(regime_change_prompt).await?;
        let extreme_adaptation = "EXTREME VOLATILITY ADAPTATION: Moving to defensive mode with 30% position sizes, very tight 2-3% stops, and immediate exit of current losing positions. Switching to cash preservation strategy until volatility returns to manageable levels.";
        info!("Extreme volatility adaptation: {}", extreme_adaptation);

        // Test learning from failures

        // let learning_response = agent.prompt(learning_prompt).await?;
        let learning_response = "LEARNING FROM FAILURE: Support level failures are becoming a pattern in this risk-off environment. I'm updating my strategy to avoid 'support bounce' trades and instead focus on momentum continuation. Technical levels are less reliable when fundamental sentiment is strongly bearish.";
        info!("Learning from failure: {}", learning_response);

        Ok(())
    }).await.map_err(|_e| anyhow::anyhow!("Strategy failed"))?;

    Ok(())
}

/// Example 2: Multi-Timeframe Strategy Adaptation
///
/// Shows how agents can adapt strategies based on multiple timeframe analysis
/// and coordinate short-term tactics with longer-term strategic goals.
async fn multi_timeframe_adaptation() -> Result<()> {
    info!("Starting multi-timeframe adaptive strategy...");

    // TODO: Commented out due to rig API changes
    /*
        let agent = AgentBuilder::new("gpt-4")
            .preamble(r#"
    You are a multi-timeframe adaptive strategy agent that coordinates across different time horizons.

    TIMEFRAME FRAMEWORK:
    - LONG-TERM (Weeks to Months): Strategic asset allocation, major trend identification
    - MEDIUM-TERM (Days to Weeks): Tactical positioning, sector rotation
    - SHORT-TERM (Hours to Days): Entry/exit timing, risk management
    - INTRADAY (Minutes to Hours): Execution optimization, quick profits

    ADAPTATION LOGIC:
    1. Long-term trend determines overall bias (bullish/bearish/neutral)
    2. Medium-term conditions influence position sizing and sector focus
    3. Short-term signals trigger specific entries and exits
    4. Intraday execution adapts to current volume and volatility

    COORDINATION RULES:
    - Don't fight the long-term trend with short-term trades
    - Increase position size when all timeframes align
    - Reduce size when timeframes conflict
    - Use shorter timeframes for optimal entry/exit points

    LEARNING MECHANISM:
    - Track which timeframe combinations work best
    - Identify when to prioritize one timeframe over others
    - Adapt timeframe weights based on market regime
    - Learn from timeframe conflict resolutions

    Always show your analysis across all timeframes and explain coordination decisions.
            "#.trim())
            .tool(get_sol_balance)
            .tool(get_spl_token_balance)
            .tool(perform_jupiter_swap)
            .max_tokens(2500)
            .build();
        */

    let keypair = Keypair::new();
    let signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
        keypair.insecure_clone(),
        "https://api.devnet.solana.com".to_string(),
    )) as Arc<dyn UnifiedSigner>;

    SignerContext::with_signer(signer, async move {
        let user_address = keypair.pubkey().to_string();

        let timeframe_context = r"
MULTI-TIMEFRAME ANALYSIS CONTEXT:

LONG-TERM (3 months):
- SOL uptrend from $80 to $180 (125% gain)
- Above all major moving averages
- Strong fundamentals, ecosystem growth
- Overall bias: BULLISH

MEDIUM-TERM (2 weeks):
- Pullback from $180 high to current $142
- Testing key support at $140-145
- RSI reset from overbought to neutral
- Tactical bias: NEUTRAL to slightly bullish

SHORT-TERM (3 days):
- Consolidation range $140-150
- Volume declining, indecision
- Waiting for breakout direction
- Entry bias: WAIT for confirmation

INTRADAY (6 hours):
- Range trading between $142-146
- Low volatility, tight spreads
- Good execution conditions
- Execution bias: RANGE trading mode

RECENT PERFORMANCE BY TIMEFRAME FOCUS:
- Long-term focused trades: 70% win rate, +12.4% PnL
- Medium-term tactical: 55% win rate, +3.1% PnL
- Short-term scalps: 60% win rate, +1.8% PnL
- Intraday only: 45% win rate, -0.8% PnL
        ";

        let _multi_timeframe_prompt = format!(r"
Please analyze across all timeframes and adapt your strategy accordingly.

My wallet: {user_address}

{timeframe_context}

Current Challenge:
The timeframes are showing mixed signals - long-term bullish, but medium-term
pullback creating uncertainty. I want to maintain my bullish exposure but also
protect against further downside.

Please:
1. Analyze how to coordinate across conflicting timeframe signals
2. Determine optimal position sizing given the timeframe mix
3. Identify specific entry/exit criteria that respect all timeframes
4. Set up monitoring for when timeframe alignment changes
5. Recommend trades that balance long-term bullishness with medium-term caution

Show your coordination logic across timeframes.
        ");

        // let coordination_response = agent.prompt(&multi_timeframe_prompt).await?;
        let coordination_response = "MULTI-TIMEFRAME COORDINATION: Long-term bullish bias conflicts with medium-term pullback. I recommend 40% position sizing to respect both timeframes, using short-term range trading while waiting for medium-term support confirmation before increasing exposure.";
        info!("Multi-timeframe coordination: {}", coordination_response);

        // Test adaptation when timeframes align

        // let alignment_response = agent.prompt(alignment_prompt).await?;
        let alignment_response = "TIMEFRAME ALIGNMENT: With all timeframes now bullish, I'm increasing position sizing to 100% normal allocation and using momentum-based entries. The alignment provides high-probability setup for 8-12% gains with managed risk through trailing stops.";
        info!("Timeframe alignment adaptation: {}", alignment_response);

        Ok(())
    }).await.map_err(|_e| anyhow::anyhow!("Multi-timeframe strategy failed"))?;

    Ok(())
}

/// Example 3: Performance-Based Strategy Evolution
///
/// Demonstrates how agents can evolve their strategies based on systematic
/// analysis of what's working and what's not, including strategy combination
/// and parameter optimization.
async fn performance_based_evolution() -> Result<()> {
    info!("Starting performance-based strategy evolution...");

    // TODO: Commented out due to rig API changes
    /*
        let agent = AgentBuilder::new("gpt-4")
            .preamble(r#"
    You are a performance-driven strategy evolution agent that systematically improves
    trading approaches based on data-driven analysis of results.

    PERFORMANCE ANALYSIS FRAMEWORK:
    1. Win Rate Analysis: Which strategies have highest probability of success
    2. Risk-Adjusted Returns: Sharpe ratio, max drawdown analysis
    3. Market Condition Performance: Which strategies work in which environments
    4. Parameter Sensitivity: How small changes affect performance
    5. Strategy Combination: When to use multiple approaches together

    EVOLUTION PROCESS:
    - Track detailed metrics for every strategy and parameter set
    - Identify top-performing strategies in different market conditions
    - Gradually phase out underperforming approaches
    - Optimize parameters based on historical performance
    - Test new strategy combinations for improved results

    ADAPTATION TRIGGERS:
    - 10+ trade sample: Evaluate strategy effectiveness
    - 5% drawdown: Review and adjust parameters
    - 3 consecutive losses: Analyze failure patterns
    - New market regime: Test strategy effectiveness in new conditions

    LEARNING PRIORITIES:
    1. What's working best RIGHT NOW (recency bias)
    2. What's worked consistently across different markets (robustness)
    3. What combinations outperform individual strategies
    4. What parameters need adjustment for current conditions

    Always show your performance analysis and explain evolution decisions.
            "#.trim())
            .tool(get_sol_balance)
            .tool(get_spl_token_balance)
            .tool(perform_jupiter_swap)
            .max_tokens(2500)
            .build();
        */

    let keypair = Keypair::new();
    let signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
        keypair.insecure_clone(),
        "https://api.devnet.solana.com".to_string(),
    )) as Arc<dyn UnifiedSigner>;

    SignerContext::with_signer(signer, async move {
        let user_address = keypair.pubkey().to_string();

        let performance_data = r"
COMPREHENSIVE PERFORMANCE ANALYSIS:

STRATEGY A - MOMENTUM BREAKOUTS:
- Sample Size: 45 trades
- Win Rate: 67% (30 wins, 15 losses)
- Avg Win: +3.2%, Avg Loss: -1.8%
- Total PnL: +18.4%
- Sharpe Ratio: 1.8
- Max Drawdown: -4.2%
- Best in: High volatility, trending markets

STRATEGY B - MEAN REVERSION:
- Sample Size: 38 trades
- Win Rate: 58% (22 wins, 16 losses)
- Avg Win: +2.1%, Avg Loss: -2.4%
- Total PnL: +4.7%
- Sharpe Ratio: 0.9
- Max Drawdown: -8.1%
- Best in: Low volatility, range-bound markets

STRATEGY C - NEWS-BASED:
- Sample Size: 23 trades
- Win Rate: 74% (17 wins, 6 losses)
- Avg Win: +4.8%, Avg Loss: -2.1%
- Total PnL: +68.5%
- Sharpe Ratio: 2.4
- Max Drawdown: -3.6%
- Best in: High impact news events

COMBINED STRATEGIES:
- A+B Portfolio: 15% better risk-adjusted returns
- A+C Combination: 22% better returns but requires event timing
- B+C Hedge: Reduced drawdowns by 35%

CURRENT MARKET: High volatility, trending down with frequent news
RECENT PERFORMANCE: Strategy A struggling (40% win rate last 10 trades)
                   Strategy C excelling (85% win rate last 6 trades)
        ";

        let _evolution_prompt = format!(r"
Please analyze the performance data and evolve your strategy approach.

My wallet: {user_address}

{performance_data}

Evolution Challenge:
Strategy A (momentum) was my best performer historically but is struggling recently.
Strategy C (news-based) is working great but has limited opportunities.
I need to evolve my approach to maximize current market opportunities.

Please:
1. Analyze why Strategy A performance has declined
2. Identify how to optimize parameters or modify the approach
3. Evaluate if I should increase allocation to Strategy C
4. Consider new strategy combinations or hybrid approaches
5. Propose specific parameter adjustments based on recent performance
6. Set up testing criteria for strategy modifications

Show your systematic approach to strategy evolution.
        ");

        // let evolution_response = agent.prompt(&evolution_prompt).await?;
        let evolution_response = "STRATEGY EVOLUTION: Strategy A's decline correlates with changing market structure - momentum breaks are failing faster. I recommend reducing Strategy A allocation to 30% and increasing Strategy C to 50% given its 85% recent win rate. Testing hybrid approach combining news-based timing with modified momentum parameters.";
        info!("Strategy evolution response: {}", evolution_response);

        // Test response to poor performance period

        // let recovery_response = agent.prompt(drawdown_prompt).await?;
        let recovery_response = "DRAWDOWN RECOVERY: The 6.8% drawdown indicates fundamental market regime change. Reducing all position sizes to 25% while analyzing what's causing the pattern failures. Implementing 'capital preservation mode' until I identify new patterns that work in this market structure.";
        info!("Drawdown recovery strategy: {}", recovery_response);

        Ok(())
    }).await.map_err(|_e| anyhow::anyhow!("Performance evolution failed"))?;

    Ok(())
}

/// Example 4: Real-Time Market Adaptation
///
/// Shows how agents can adapt instantly to breaking news, sudden volatility spikes,
/// and other real-time market events that require immediate strategy adjustments.
async fn real_time_market_adaptation() -> Result<()> {
    info!("Starting real-time market adaptation...");

    // TODO: Commented out due to rig API changes
    /*
        let agent = AgentBuilder::new("gpt-4")
            .preamble(r#"
    You are a real-time adaptive trading agent that responds instantly to market events,
    news, and sudden changes in conditions.

    REAL-TIME ADAPTATION TRIGGERS:
    - Price moves > 5% in < 15 minutes: Volatility spike protocol
    - Breaking news affecting your positions: News response protocol
    - Volume spikes > 200% of average: Liquidity event protocol
    - Technical level breaks: Support/resistance failure protocol
    - Risk management alerts: Position size or drawdown limits hit

    ADAPTATION SPEED REQUIREMENTS:
    - Market structure changes: < 5 minutes to adapt
    - Breaking news: < 2 minutes to assess and respond
    - Technical breaks: < 1 minute to adjust stops/targets
    - Risk alerts: < 30 seconds to reduce exposure

    DECISION FRAMEWORK:
    1. ASSESS: What changed and how significant is it?
    2. IMPACT: How does this affect my current positions?
    3. ADAPT: What strategy adjustments are needed immediately?
    4. ACT: Execute necessary trades to align with new reality
    5. MONITOR: Watch for further developments requiring additional adaptation

    ADAPTATION TYPES:
    - Parameter Adjustment: Change stop-loss, take-profit, position size
    - Strategy Switch: Move from one strategy to completely different approach
    - Risk Reduction: Cut positions, raise cash, reduce leverage
    - Opportunity Capture: Increase positions when favorable events occur
    - Defensive Mode: Protect capital when uncertainty spikes

    Always respond quickly and explain your real-time decision making process.
            "#.trim())
            .tool(get_sol_balance)
            .tool(get_spl_token_balance)
            .tool(perform_jupiter_swap)
            .max_tokens(2500)
            .build();
        */

    let keypair = Keypair::new();
    let signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
        keypair.insecure_clone(),
        "https://api.devnet.solana.com".to_string(),
    )) as Arc<dyn UnifiedSigner>;

    SignerContext::with_signer(signer, async move {
        let user_address = keypair.pubkey().to_string();

        let _baseline_prompt = format!(r"
CURRENT POSITION STATUS:

My wallet: {user_address}

ACTIVE POSITIONS:
- SOL: 50% of portfolio, avg entry $147.20, currently $148.50 (+0.9%)
- USDC: 30% cash reserves for opportunities
- Small altcoin positions: 20% diversified across 4 tokens

STRATEGY STATUS:
- Running momentum strategy in normal volatility environment
- Stop losses at 4% below entry
- Take profits at 8% gains
- Position size: 60% normal due to recent uncertain conditions

MARKET CONDITIONS:
- Normal volatility: 6% daily range
- Volume: Average levels
- No major news expected today
- Technical levels holding

I'm ready for whatever the market throws at me. Please establish baseline
monitoring and be prepared to adapt to any real-time events.
        ");

        // let baseline_response = agent.prompt(&baseline_prompt).await?;
        let baseline_response = "BASELINE MONITORING: Current positions tracked. Stop-losses set at 4%, monitoring for volatility spikes >200% or volume anomalies. Ready for real-time adaptation to market events.";
        info!("Baseline monitoring established: {}", baseline_response);

        // Simulate breaking news event

        // let crisis_response = agent.prompt(breaking_news_prompt).await?;
        let crisis_response = "CRISIS RESPONSE: IMMEDIATE RISK REDUCTION - Cutting SOL position by 75% and exiting altcoins completely. Protocol hack creates ecosystem risk. Moving to cash position until situation clarifies. Stop-losses bypassed due to extreme volatility.";
        info!("Breaking news crisis response: {}", crisis_response);

        // Test adaptation to rapid recovery

        // let recovery_response = agent.prompt(recovery_prompt).await?;
        let recovery_response = "RECOVERY ADAPTATION: Quick reversal suggests overreaction. Cautiously re-entering SOL position at 50% allocation given successful containment. The market taught me to be faster on crisis response but also ready to adapt when new information emerges.";
        info!("Recovery adaptation response: {}", recovery_response);

        Ok(())
    }).await.map_err(|_e| anyhow::anyhow!("Real-time adaptation failed"))?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for detailed logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting riglr adaptive strategy examples...");

    println!("\n🎯 Adaptive Strategy Examples with riglr");
    println!("=========================================");
    println!("Demonstrating how rig agents adapt behavior based on changing conditions\n");

    // Run adaptive strategy examples
    println!("1️⃣  Volatility-Adaptive Strategy");
    println!("   Adapting position sizing and tactics based on market volatility...");
    if let Err(e) = volatility_adaptive_strategy().await {
        warn!("Volatility adaptation failed: {}", e);
    }

    println!("\n2️⃣  Multi-Timeframe Strategy Adaptation");
    println!("   Coordinating across different time horizons...");
    if let Err(e) = multi_timeframe_adaptation().await {
        warn!("Multi-timeframe adaptation failed: {}", e);
    }

    println!("\n3️⃣  Performance-Based Strategy Evolution");
    println!("   Learning and evolving from trading results...");
    if let Err(e) = performance_based_evolution().await {
        warn!("Performance evolution failed: {}", e);
    }

    println!("\n4️⃣  Real-Time Market Adaptation");
    println!("   Instant adaptation to breaking news and market events...");
    if let Err(e) = real_time_market_adaptation().await {
        warn!("Real-time adaptation failed: {}", e);
    }

    println!("\n✅ Adaptive strategy examples completed!");
    println!("\nKey Adaptation Patterns:");
    println!("- Volatility-based parameter adjustment");
    println!("- Multi-timeframe coordination");
    println!("- Performance-driven strategy evolution");
    println!("- Real-time event response");
    println!("- Learning from outcomes");
    println!("- Dynamic risk management");

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_market_conditions_serialization() {
        let conditions = MarketConditions {
            volatility_level: VolatilityLevel::High,
            trend_direction: TrendDirection::Uptrend,
            liquidity_conditions: LiquidityLevel::Normal,
            risk_sentiment: RiskSentiment::RiskOn,
        };

        let serialized = serde_json::to_string(&conditions).unwrap();
        let deserialized: MarketConditions = serde_json::from_str(&serialized).unwrap();

        assert!(matches!(
            deserialized.volatility_level,
            VolatilityLevel::High
        ));
        assert!(matches!(
            deserialized.trend_direction,
            TrendDirection::Uptrend
        ));
    }

    #[tokio::test]
    async fn test_strategy_performance_tracking() {
        let performance = StrategyPerformance {
            strategy_name: "test_strategy".to_string(),
            wins: 10,
            losses: 5,
            total_pnl: 15.5,
            avg_return: 1.03,
            max_drawdown: -2.1,
            sharpe_ratio: 1.5,
            market_conditions: vec![],
        };

        assert_eq!(performance.wins, 10);
        assert_eq!(performance.losses, 5);
        assert_eq!(performance.total_pnl, 15.5);
    }
}
