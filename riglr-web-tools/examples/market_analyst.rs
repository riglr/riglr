//! Example: Market Analyst Agent
//!
//! This example demonstrates how to use riglr-web-tools to create a comprehensive
//! market analysis agent that can gather data from multiple sources and provide
//! actionable insights.

use riglr_core::{provider::ApplicationContext, Config};
use riglr_web_tools::dexscreener::{ChainInfo, SecurityInfo};
use riglr_web_tools::{
    analyze_crypto_sentiment, analyze_market_sentiment, get_token_info, get_trending_tokens,
    search_tweets,
};
use std::env;

/// Market analysis results combining multiple data sources
#[derive(Debug)]
struct MarketAnalysis {
    token_symbol: String,
    price_usd: f64,
    market_cap: f64,
    volume_24h: f64,
    social_sentiment: f64,
    news_sentiment: f64,
    dex_liquidity: f64,
    trending_rank: Option<u32>,
    risk_score: f64,
    recommendation: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create application context
    let config = Config::from_env();
    let context = ApplicationContext::from_config(&config);

    println!("🤖 Market Analyst Agent Example\n");
    println!("Combining Web Tools and Solana Tools for comprehensive analysis\n");

    // Get token to analyze from command line or use default
    let token = env::args().nth(1).unwrap_or_else(|| "SOL".to_string());

    println!("📊 Analyzing token: {}\n", token);

    // Example 1: Full market analysis for a token
    let analysis = perform_full_analysis(&token, &context).await?;
    print_analysis_report(&analysis);

    // Example 2: Find trending opportunities
    println!("\n🔥 Finding Trending Opportunities...\n");
    find_trending_opportunities(&context).await?;

    // Example 3: Monitor market sentiment shifts
    println!("\n📈 Monitoring Market Sentiment...\n");
    monitor_sentiment_shifts(&context).await?;

    // Example 4: Arbitrage opportunity scanner
    println!("\n💰 Scanning for Arbitrage Opportunities...\n");
    scan_arbitrage_opportunities(&context).await?;

    println!("\n✅ Market analysis complete!");

    Ok(())
}

/// Perform comprehensive analysis of a token
async fn perform_full_analysis(
    symbol: &str,
    context: &ApplicationContext,
) -> anyhow::Result<MarketAnalysis> {
    println!("1️⃣ Fetching token information from DexScreener...");

    // Get token info from DexScreener
    let token_info = match get_token_info(context, symbol.to_string(), None, None, None).await {
        Ok(info) => info,
        Err(e) => {
            println!("   ⚠️ Could not fetch from DexScreener: {}", e);
            // Create mock data for demo
            riglr_web_tools::TokenInfo {
                address: "mock".to_string(),
                symbol: symbol.to_string(),
                name: format!("{} Token", symbol),
                decimals: 9,
                price_usd: Some(100.0),
                market_cap: Some(1_000_000_000.0),
                volume_24h: Some(50_000_000.0),
                price_change_24h: Some(5.5),
                price_change_1h: Some(1.2),
                price_change_5m: Some(0.3),
                circulating_supply: Some(10_000_000.0),
                total_supply: Some(21_000_000.0),
                pair_count: 5,
                pairs: vec![],
                chain: ChainInfo {
                    id: "solana".to_string(),
                    name: "Solana".to_string(),
                    logo: None,
                    native_token: "SOL".to_string(),
                },
                security: SecurityInfo {
                    is_verified: true,
                    liquidity_locked: Some(true),
                    audit_status: Some("Passed".to_string()),
                    honeypot_status: Some("Safe".to_string()),
                    ownership_status: Some("Renounced".to_string()),
                    risk_score: Some(15),
                },
                socials: vec![],
                updated_at: chrono::Utc::now(),
            }
        }
    };

    println!("   ✅ Price: ${:.4}", token_info.price_usd.unwrap_or(0.0));
    println!(
        "   ✅ Market Cap: ${:.0}",
        token_info.market_cap.unwrap_or(0.0)
    );

    println!("\n2️⃣ Analyzing social sentiment on Twitter...");

    // Analyze Twitter sentiment
    let sentiment = match analyze_crypto_sentiment(
        context,
        symbol.to_string(),
        Some(24), // Last 24 hours
        Some(50), // Min engagement threshold
    )
    .await
    {
        Ok(s) => s,
        Err(e) => {
            println!("   ⚠️ Could not analyze Twitter sentiment: {}", e);
            // Create mock sentiment
            riglr_web_tools::SentimentAnalysis {
                overall_sentiment: 0.65,
                sentiment_breakdown: riglr_web_tools::SentimentBreakdown {
                    positive_pct: 65.0,
                    neutral_pct: 25.0,
                    negative_pct: 10.0,
                    positive_avg_engagement: 150.0,
                    negative_avg_engagement: 50.0,
                },
                tweet_count: 500,
                analyzed_at: chrono::Utc::now(),
                top_positive_tweets: vec![],
                top_negative_tweets: vec![],
                top_entities: vec![],
            }
        }
    };

    println!(
        "   ✅ Overall Sentiment: {:.2}",
        sentiment.overall_sentiment
    );
    println!(
        "   ✅ Positive: {:.1}%",
        sentiment.sentiment_breakdown.positive_pct
    );

    println!("\n3️⃣ Checking news sentiment...");

    // Get news sentiment
    let news_sentiment = match analyze_market_sentiment(
        context,
        Some("48h".to_string()),        // Time window
        Some(vec![symbol.to_string()]), // Asset filter
        None,                           // Source weights
        None,                           // Include social
    )
    .await
    {
        Ok(s) => s.overall_sentiment,
        Err(e) => {
            println!("   ⚠️ Could not analyze news sentiment: {}", e);
            0.5 // Neutral
        }
    };

    println!("   ✅ News Sentiment: {:.2}", news_sentiment);

    println!("\n4️⃣ Checking DEX liquidity from token info...");

    // Use token info to estimate liquidity
    let dex_liquidity = if let Some(volume) = token_info.volume_24h {
        if volume > 1_000_000.0 {
            0.8 // High liquidity
        } else if volume > 100_000.0 {
            0.6 // Medium liquidity
        } else {
            0.3 // Low liquidity
        }
    } else {
        0.5 // Unknown
    };

    println!("   ✅ DEX Liquidity Score: {:.2}", dex_liquidity);

    println!("\n5️⃣ Checking if token is trending...");

    // Check if token is trending
    let trending_rank = match get_trending_tokens(
        context,
        Some("1h".to_string()),     // Time window
        Some("solana".to_string()), // Chain filter
        None,                       // Min volume
        Some(100),                  // Limit
    )
    .await
    {
        Ok(trending) => trending
            .iter()
            .position(|t| t.symbol.to_lowercase() == symbol.to_lowercase())
            .map(|pos| (pos + 1) as u32),
        Err(_) => None,
    };

    if let Some(rank) = trending_rank {
        println!("   ✅ Trending Rank: #{}", rank);
    } else {
        println!("   ℹ️ Not in top 100 trending");
    }

    // Calculate risk score and recommendation
    let risk_score = calculate_risk_score(
        sentiment.overall_sentiment,
        news_sentiment,
        dex_liquidity,
        token_info.price_change_24h.unwrap_or(0.0),
    );

    let recommendation = generate_recommendation(
        risk_score,
        sentiment.overall_sentiment,
        trending_rank,
        token_info.price_change_24h.unwrap_or(0.0),
    );

    Ok(MarketAnalysis {
        token_symbol: symbol.to_string(),
        price_usd: token_info.price_usd.unwrap_or(0.0),
        market_cap: token_info.market_cap.unwrap_or(0.0),
        volume_24h: token_info.volume_24h.unwrap_or(0.0),
        social_sentiment: sentiment.overall_sentiment,
        news_sentiment,
        dex_liquidity,
        trending_rank,
        risk_score,
        recommendation,
    })
}

/// Find trending opportunities across multiple chains
async fn find_trending_opportunities(context: &ApplicationContext) -> anyhow::Result<()> {
    // Get trending tokens from DexScreener
    let trending = match get_trending_tokens(
        context,
        Some("1h".to_string()), // Time window
        None,                   // All chains
        None,                   // Min volume
        Some(10),               // Top 10
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            println!("Could not fetch trending tokens: {}", e);
            return Ok(());
        }
    };

    for (idx, token) in trending.iter().take(5).enumerate() {
        println!("{}. {} ({})", idx + 1, token.symbol, token.chain.id);
        println!("   Price: ${:.6}", token.price_usd.unwrap_or(0.0));
        println!(
            "   24h Change: {:.1}%",
            token.price_change_24h.unwrap_or(0.0)
        );
        println!("   Volume: ${:.0}", token.volume_24h.unwrap_or(0.0));

        // Quick sentiment check
        let search_result = search_tweets(
            context,
            format!("${}", token.symbol),
            Some(10),
            Some(false),
            None,
            None,
            None,
        )
        .await;
        if let Ok(tweets) = search_result {
            println!("   Twitter mentions: {}", tweets.tweets.len());
        }

        println!();
    }

    Ok(())
}

/// Monitor sentiment shifts across multiple tokens
async fn monitor_sentiment_shifts(context: &ApplicationContext) -> anyhow::Result<()> {
    let tokens = vec!["BTC", "ETH", "SOL"];

    for token in tokens {
        println!("Analyzing sentiment for {}...", token);

        // Get current sentiment
        let sentiment_result = analyze_crypto_sentiment(
            context,
            token.to_string(),
            Some(6),  // Last 6 hours
            Some(20), // Lower threshold for faster results
        )
        .await;
        match sentiment_result {
            Ok(sentiment) => {
                let sentiment_emoji = if sentiment.overall_sentiment > 0.6 {
                    "🟢"
                } else if sentiment.overall_sentiment > 0.4 {
                    "🟡"
                } else {
                    "🔴"
                };

                println!(
                    "  {} {} Sentiment: {:.2} ({:.0}% positive)",
                    sentiment_emoji,
                    token,
                    sentiment.overall_sentiment,
                    sentiment.sentiment_breakdown.positive_pct
                );
            }
            Err(e) => {
                println!("  ⚠️ Could not analyze {}: {}", token, e);
            }
        }
    }

    Ok(())
}

/// Scan for potential arbitrage opportunities
async fn scan_arbitrage_opportunities(_context: &ApplicationContext) -> anyhow::Result<()> {
    println!("Scanning DEX prices across chains...\n");

    // In a real implementation, this would compare prices across multiple DEXs
    // For demo purposes, we'll show the concept

    let tokens = vec![
        ("USDC", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
        ("USDT", "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
    ];

    for (symbol, address) in tokens {
        println!("Checking {} arbitrage opportunities:", symbol);

        // Get price on Ethereum
        let config = Config::from_env();
        let context = ApplicationContext::from_config(&config);
        let eth_result = get_token_info(
            &context,
            address.to_string(),
            Some("ethereum".to_string()),
            None,
            None,
        )
        .await;
        if let Ok(eth_info) = eth_result {
            println!("  Ethereum: ${:.4}", eth_info.price_usd.unwrap_or(0.0));
        }

        // Get price on other chains (would need different addresses in reality)
        let config = Config::from_env();
        let context = ApplicationContext::from_config(&config);
        let bsc_result = get_token_info(
            &context,
            address.to_string(),
            Some("bsc".to_string()),
            None,
            None,
        )
        .await;
        if let Ok(bsc_info) = bsc_result {
            println!("  BSC: ${:.4}", bsc_info.price_usd.unwrap_or(0.0));

            // Calculate potential arbitrage
            // In reality, would need to account for gas fees, slippage, etc.
        }

        println!();
    }

    println!("ℹ️ Note: Real arbitrage detection requires accounting for:");
    println!("  - Gas fees on each chain");
    println!("  - Slippage and liquidity");
    println!("  - Bridge fees and time");

    Ok(())
}

/// Calculate risk score based on multiple factors
fn calculate_risk_score(
    social_sentiment: f64,
    news_sentiment: f64,
    liquidity: f64,
    price_change_24h: f64,
) -> f64 {
    let sentiment_score = (social_sentiment + news_sentiment) / 2.0;
    let volatility_risk = (price_change_24h.abs() / 100.0).min(1.0);

    // Higher sentiment and liquidity = lower risk
    // Higher volatility = higher risk
    let risk = 1.0 - (sentiment_score * 0.4 + liquidity * 0.4 - volatility_risk * 0.2);

    (risk * 100.0).clamp(0.0, 100.0)
}

/// Generate trading recommendation based on analysis
fn generate_recommendation(
    risk_score: f64,
    sentiment: f64,
    trending_rank: Option<u32>,
    price_change: f64,
) -> String {
    let mut recommendation = String::default();

    if risk_score < 30.0 && sentiment > 0.7 {
        recommendation.push_str("🟢 STRONG BUY - ");
        recommendation.push_str("Low risk with very positive sentiment. ");
    } else if risk_score < 50.0 && sentiment > 0.5 {
        recommendation.push_str("🟢 BUY - ");
        recommendation.push_str("Moderate risk with positive sentiment. ");
    } else if risk_score > 70.0 || sentiment < 0.3 {
        recommendation.push_str("🔴 AVOID - ");
        recommendation.push_str("High risk or negative sentiment. ");
    } else {
        recommendation.push_str("🟡 HOLD/WATCH - ");
        recommendation.push_str("Mixed signals, monitor closely. ");
    }

    if let Some(rank) = trending_rank {
        if rank <= 10 {
            recommendation.push_str(&format!("Currently trending (#{})! ", rank));
        }
    }

    if price_change.abs() > 20.0 {
        recommendation.push_str("High volatility detected. ");
    }

    recommendation
}

/// Print formatted analysis report
fn print_analysis_report(analysis: &MarketAnalysis) {
    println!("\n{}", "=".repeat(60));
    println!("📊 MARKET ANALYSIS REPORT: {}", analysis.token_symbol);
    println!("{}", "=".repeat(60));

    println!("\n💵 PRICE METRICS:");
    println!("  • Current Price: ${:.4}", analysis.price_usd);
    println!("  • Market Cap: ${:.0}", analysis.market_cap);
    println!("  • 24h Volume: ${:.0}", analysis.volume_24h);

    println!("\n🎭 SENTIMENT ANALYSIS:");
    println!(
        "  • Social Sentiment: {:.2} ({})",
        analysis.social_sentiment,
        if analysis.social_sentiment > 0.6 {
            "Positive"
        } else if analysis.social_sentiment > 0.4 {
            "Neutral"
        } else {
            "Negative"
        }
    );
    println!("  • News Sentiment: {:.2}", analysis.news_sentiment);

    println!("\n📈 MARKET POSITION:");
    println!("  • DEX Liquidity Score: {:.2}", analysis.dex_liquidity);
    if let Some(rank) = analysis.trending_rank {
        println!("  • Trending Rank: #{}", rank);
    }

    println!("\n⚠️ RISK ASSESSMENT:");
    println!("  • Risk Score: {:.0}/100", analysis.risk_score);
    println!(
        "  • Risk Level: {}",
        if analysis.risk_score < 30.0 {
            "Low"
        } else if analysis.risk_score < 60.0 {
            "Medium"
        } else {
            "High"
        }
    );

    println!("\n💡 RECOMMENDATION:");
    println!("  {}", analysis.recommendation);

    println!("\n{}", "=".repeat(60));
}
