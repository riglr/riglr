//! Example: Market Analyst Agent
//!
//! This example demonstrates how to combine riglr-web-tools with riglr-solana-tools
//! to create a comprehensive market analysis agent that can gather data from multiple
//! sources and provide actionable insights.

use riglr_web_tools::{
    analyze_crypto_sentiment, get_token_info, get_trending_tokens, search_tweets, search_web,
    get_crypto_news, analyze_market_sentiment,
};
use riglr_solana_tools::{get_jupiter_quote, get_sol_balance, get_token_balance};
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

    println!("🤖 Market Analyst Agent Example\n");
    println!("Combining Web Tools and Solana Tools for comprehensive analysis\n");

    // Get token to analyze from command line or use default
    let token = env::args()
        .nth(1)
        .unwrap_or_else(|| "SOL".to_string());

    println!("📊 Analyzing token: {}\n", token);

    // Example 1: Full market analysis for a token
    let analysis = perform_full_analysis(&token).await?;
    print_analysis_report(&analysis);

    // Example 2: Find trending opportunities
    println!("\n🔥 Finding Trending Opportunities...\n");
    find_trending_opportunities().await?;

    // Example 3: Monitor market sentiment shifts
    println!("\n📈 Monitoring Market Sentiment...\n");
    monitor_sentiment_shifts().await?;

    // Example 4: Arbitrage opportunity scanner
    println!("\n💰 Scanning for Arbitrage Opportunities...\n");
    scan_arbitrage_opportunities().await?;

    println!("\n✅ Market analysis complete!");

    Ok(())
}

/// Perform comprehensive analysis of a token
async fn perform_full_analysis(symbol: &str) -> anyhow::Result<MarketAnalysis> {
    println!("1️⃣ Fetching token information from DexScreener...");
    
    // Get token info from DexScreener
    let token_info = match get_token_info(symbol.to_string(), None).await {
        Ok(info) => info,
        Err(e) => {
            println!("   ⚠️ Could not fetch from DexScreener: {}", e);
            // Create mock data for demo
            riglr_web_tools::TokenInfo {
                address: "mock".to_string(),
                symbol: symbol.to_string(),
                name: format!("{} Token", symbol),
                price_usd: 100.0,
                market_cap: Some(1_000_000_000.0),
                volume_24h: Some(50_000_000.0),
                liquidity_usd: Some(10_000_000.0),
                price_change_24h: Some(5.5),
                holders: Some(100000),
                chain: "solana".to_string(),
            }
        }
    };

    println!("   ✅ Price: ${:.4}", token_info.price_usd);
    println!("   ✅ Market Cap: ${:.0}", token_info.market_cap.unwrap_or(0.0));

    println!("\n2️⃣ Analyzing social sentiment on Twitter...");
    
    // Analyze Twitter sentiment
    let sentiment = match analyze_crypto_sentiment(
        symbol.to_string(),
        Some(24),  // Last 24 hours
        Some(50),  // Min engagement threshold
    ).await {
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

    println!("   ✅ Overall Sentiment: {:.2}", sentiment.overall_sentiment);
    println!("   ✅ Positive: {:.1}%", sentiment.sentiment_breakdown.positive_pct);

    println!("\n3️⃣ Checking news sentiment...");
    
    // Get news sentiment
    let news_sentiment = match analyze_market_sentiment(
        Some(symbol.to_string()),
        Some(48),  // Last 48 hours
    ).await {
        Ok(s) => s.overall_sentiment,
        Err(e) => {
            println!("   ⚠️ Could not analyze news sentiment: {}", e);
            0.5 // Neutral
        }
    };

    println!("   ✅ News Sentiment: {:.2}", news_sentiment);

    println!("\n4️⃣ Checking DEX liquidity on Solana...");
    
    // Check Jupiter (Solana DEX) for liquidity
    let jupiter_quote = if symbol == "SOL" || symbol.to_lowercase().contains("sol") {
        match get_jupiter_quote(
            "So11111111111111111111111111111111111112".to_string(), // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            1_000_000_000, // 1 SOL in lamports
            Some(50),      // 0.5% slippage
        ).await {
            Ok(quote) => Some(quote),
            Err(e) => {
                println!("   ⚠️ Could not fetch Jupiter quote: {}", e);
                None
            }
        }
    } else {
        None
    };

    let dex_liquidity = jupiter_quote
        .as_ref()
        .and_then(|q| q.price_impact_pct)
        .map(|impact| (100.0 - impact.abs()) / 100.0) // Convert impact to liquidity score
        .unwrap_or(0.5);

    println!("   ✅ DEX Liquidity Score: {:.2}", dex_liquidity);

    println!("\n5️⃣ Checking if token is trending...");
    
    // Check if token is trending
    let trending_rank = match get_trending_tokens(
        Some("solana".to_string()),
        Some(100),
    ).await {
        Ok(trending) => {
            trending.tokens
                .iter()
                .position(|t| t.symbol.to_lowercase() == symbol.to_lowercase())
                .map(|pos| (pos + 1) as u32)
        }
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
        price_usd: token_info.price_usd,
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
async fn find_trending_opportunities() -> anyhow::Result<()> {
    // Get trending tokens from DexScreener
    let trending = match get_trending_tokens(
        None,      // All chains
        Some(10),  // Top 10
    ).await {
        Ok(t) => t,
        Err(e) => {
            println!("Could not fetch trending tokens: {}", e);
            return Ok(());
        }
    };

    for (idx, token) in trending.tokens.iter().take(5).enumerate() {
        println!("{}. {} ({})", idx + 1, token.symbol, token.chain);
        println!("   Price: ${:.6}", token.price_usd);
        println!("   24h Change: {:.1}%", token.price_change_24h.unwrap_or(0.0));
        println!("   Volume: ${:.0}", token.volume_24h.unwrap_or(0.0));
        
        // Quick sentiment check
        if let Ok(tweets) = search_tweets(
            format!("${}", token.symbol),
            Some(10),
            Some(false),
            None,
            None,
            None,
        ).await {
            println!("   Twitter mentions: {}", tweets.tweets.len());
        }
        
        println!();
    }

    Ok(())
}

/// Monitor sentiment shifts across multiple tokens
async fn monitor_sentiment_shifts() -> anyhow::Result<()> {
    let tokens = vec!["BTC", "ETH", "SOL"];
    
    for token in tokens {
        println!("Analyzing sentiment for {}...", token);
        
        // Get current sentiment
        match analyze_crypto_sentiment(
            token.to_string(),
            Some(6),   // Last 6 hours
            Some(20),  // Lower threshold for faster results
        ).await {
            Ok(sentiment) => {
                let sentiment_emoji = if sentiment.overall_sentiment > 0.6 {
                    "🟢"
                } else if sentiment.overall_sentiment > 0.4 {
                    "🟡"
                } else {
                    "🔴"
                };
                
                println!("  {} {} Sentiment: {:.2} ({:.0}% positive)",
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
async fn scan_arbitrage_opportunities() -> anyhow::Result<()> {
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
        if let Ok(eth_info) = get_token_info(
            address.to_string(),
            Some("ethereum".to_string()),
        ).await {
            println!("  Ethereum: ${:.4}", eth_info.price_usd);
        }
        
        // Get price on other chains (would need different addresses in reality)
        if let Ok(bsc_info) = get_token_info(
            address.to_string(),
            Some("bsc".to_string()),
        ).await {
            println!("  BSC: ${:.4}", bsc_info.price_usd);
            
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
    
    (risk * 100.0).max(0.0).min(100.0)
}

/// Generate trading recommendation based on analysis
fn generate_recommendation(
    risk_score: f64,
    sentiment: f64,
    trending_rank: Option<u32>,
    price_change: f64,
) -> String {
    let mut recommendation = String::new();
    
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
    println!("  • Social Sentiment: {:.2} ({})",
        analysis.social_sentiment,
        if analysis.social_sentiment > 0.6 { "Positive" } 
        else if analysis.social_sentiment > 0.4 { "Neutral" }
        else { "Negative" }
    );
    println!("  • News Sentiment: {:.2}", analysis.news_sentiment);
    
    println!("\n📈 MARKET POSITION:");
    println!("  • DEX Liquidity Score: {:.2}", analysis.dex_liquidity);
    if let Some(rank) = analysis.trending_rank {
        println!("  • Trending Rank: #{}", rank);
    }
    
    println!("\n⚠️ RISK ASSESSMENT:");
    println!("  • Risk Score: {:.0}/100", analysis.risk_score);
    println!("  • Risk Level: {}",
        if analysis.risk_score < 30.0 { "Low" }
        else if analysis.risk_score < 60.0 { "Medium" }
        else { "High" }
    );
    
    println!("\n💡 RECOMMENDATION:");
    println!("  {}", analysis.recommendation);
    
    println!("\n{}", "=".repeat(60));
}