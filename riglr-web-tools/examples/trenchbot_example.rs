//! Example of using the TrenchBot API integration for bundle analysis

use riglr_core::provider::ApplicationContext;
use riglr_web_tools::trenchbot::{
    analyze_creator_risk, analyze_token_bundles, check_bundle_risk, get_bundle_info,
    BundleAnalysisResult, BundleResponse, BundleRiskCheck, CreatorAnalysisResult,
};

fn display_bundle_risk(result: &BundleRiskCheck) {
    println!("  Is Bundled: {}", result.is_bundled);
    println!("  Bundle Percentage: {:.2}%", result.bundle_percentage);
    println!("  Risk Level: {}", result.risk_level);
    println!("  High Risk: {}", result.is_high_risk);
    println!("  Summary: {}", result.message);
}

fn display_token_analysis(analysis: &BundleAnalysisResult) {
    println!(
        "  Token: {} ({})",
        analysis.token,
        analysis.ticker.as_deref().unwrap_or("N/A")
    );
    println!("  Bundled: {}", analysis.is_bundled);
    println!("  Bundle Percentage: {:.2}%", analysis.bundle_percentage);
    println!("  Number of Bundles: {}", analysis.bundle_count);
    println!("  Total SOL Spent: {:.4} SOL", analysis.total_sol_spent);
    println!("  Holder Count: {}", analysis.holder_count);

    println!("\n  📊 Wallet Categories:");
    println!("    Snipers: {}", analysis.wallet_categories.snipers);
    println!("    Regular: {}", analysis.wallet_categories.regular);
    println!(
        "    Sniper Percentage: {:.2}%",
        analysis.wallet_categories.sniper_percentage
    );
    println!(
        "    Primary Category: {}",
        analysis.wallet_categories.primary_category
    );

    println!("\n  👤 Creator Risk:");
    println!(
        "    Address: {}",
        analysis
            .creator_risk
            .address
            .as_deref()
            .unwrap_or("Unknown")
    );
    println!("    Risk Level: {:?}", analysis.creator_risk.risk_level);
    println!(
        "    Total Coins Created: {}",
        analysis.creator_risk.total_coins
    );
    println!("    Rug Count: {}", analysis.creator_risk.rug_count);
    println!(
        "    Rug Percentage: {:.2}%",
        analysis.creator_risk.rug_percentage
    );
    println!(
        "    Current Holdings: {:.2}%",
        analysis.creator_risk.holding_percentage
    );

    if !analysis.warnings.is_empty() {
        println!("\n  ⚠️ Warnings:");
        for warning in &analysis.warnings {
            println!("    - {}", warning);
        }
    }

    println!("\n  💡 Recommendation:");
    println!("    {}", analysis.recommendation);
}

fn display_creator_analysis(creator_analysis: &CreatorAnalysisResult) {
    println!(
        "  Creator: {}",
        creator_analysis
            .creator_address
            .as_deref()
            .unwrap_or("Unknown")
    );
    println!("  Risk Level: {:?}", creator_analysis.risk_level);
    println!(
        "  Total Coins Created: {}",
        creator_analysis.total_coins_created
    );
    println!("  Rug Count: {}", creator_analysis.rug_count);
    println!("  Rug Percentage: {:.2}%", creator_analysis.rug_percentage);
    println!(
        "  Current Holdings: {:.2}%",
        creator_analysis.holding_percentage
    );

    if let Some(avg_cap) = creator_analysis.average_market_cap {
        println!("  Average Market Cap: ${}", avg_cap);
    }

    if !creator_analysis.red_flags.is_empty() {
        println!("\n  🚩 Red Flags:");
        for flag in &creator_analysis.red_flags {
            println!("    - {}", flag);
        }
    }

    println!("\n  Assessment: {}", creator_analysis.risk_assessment);

    if !creator_analysis.previous_coins.is_empty() {
        println!("\n  Previous Coins (showing up to 3):");
        for (i, coin) in creator_analysis.previous_coins.iter().take(3).enumerate() {
            println!(
                "    {}. {} - Market Cap: ${}, Rug: {}",
                i + 1,
                coin.symbol.as_deref().unwrap_or("Unknown"),
                coin.market_cap.unwrap_or(0),
                coin.is_rug.unwrap_or(false)
            );
        }
        if creator_analysis.previous_coins.len() > 3 {
            println!(
                "    ... and {} more",
                creator_analysis.previous_coins.len() - 3
            );
        }
    }
}

fn display_bundle_data(bundle_data: &BundleResponse, token_mint: &str) {
    println!("  Token: {}", token_mint);
    if let Some(ticker) = &bundle_data.ticker {
        println!("  Ticker: {}", ticker);
    }
    if let Some(bonded) = bundle_data.bonded {
        println!("  Bonded: {}", bonded);
    }
    if let Some(bundles) = bundle_data.total_bundles {
        println!("  Total Bundles: {}", bundles);
    }
    if let Some(bundled_pct) = bundle_data.total_percentage_bundled {
        println!("  Bundled Percentage: {:.2}%", bundled_pct);
    }
    if let Some(sol_spent) = bundle_data.total_sol_spent {
        println!("  Total SOL Spent: {:.4} SOL", sol_spent);
    }
    if let Some(distributed) = bundle_data.distributed_wallets {
        println!("  Distributed to Wallets: {}", distributed);
    }

    if let Some(bundles) = &bundle_data.bundles {
        println!("\n  Bundle Details (showing up to 2):");
        for (i, (id, details)) in bundles.iter().take(2).enumerate() {
            println!("    Bundle {}:", i + 1);
            println!("      ID: {}", id);
            if let Some(tokens) = details.total_tokens {
                println!("      Total Tokens: {}", tokens);
            }
            if let Some(sol) = details.total_sol {
                println!("      Total SOL: {:.4}", sol);
            }
            if let Some(wallets) = details.unique_wallets {
                println!("      Unique Wallets: {}", wallets);
            }
            if let Some(analysis) = &details.bundle_analysis {
                if let Some(is_bundle) = analysis.is_likely_bundle {
                    println!("      Likely Bundle: {}", is_bundle);
                }
                if let Some(category) = &analysis.primary_category {
                    println!("      Primary Category: {}", category);
                }
            }
        }
        if bundles.len() > 2 {
            println!("    ... and {} more bundles", bundles.len() - 2);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create a mock application context (in real usage, this would come from your app)
    let context = ApplicationContext::default();

    // Example Solana token mint address (replace with actual token to test)
    let token_mint = "So11111111111111111111111111111111111111112"; // Wrapped SOL

    println!("🔍 Analyzing token bundles: {}", token_mint);
    println!("{}", "=".repeat(60));

    // Example 1: Quick bundle risk check
    println!("\n📊 Quick Bundle Risk Check:");
    match check_bundle_risk(&context, token_mint.to_string()).await {
        Ok(result) => display_bundle_risk(&result),
        Err(e) => println!("  Error: {}", e),
    }

    // Example 2: Comprehensive bundle analysis
    println!("\n🎯 Comprehensive Bundle Analysis:");
    match analyze_token_bundles(&context, token_mint.to_string()).await {
        Ok(analysis) => display_token_analysis(&analysis),
        Err(e) => println!("  Error: {}", e),
    }

    // Example 3: Creator risk analysis
    println!("\n👤 Creator Risk Analysis:");
    match analyze_creator_risk(&context, token_mint.to_string()).await {
        Ok(creator_analysis) => display_creator_analysis(&creator_analysis),
        Err(e) => println!("  Error: {}", e),
    }

    // Example 4: Raw bundle data (most detailed)
    println!("\n📋 Raw Bundle Data:");
    match get_bundle_info(&context, token_mint.to_string()).await {
        Ok(bundle_data) => display_bundle_data(&bundle_data, token_mint),
        Err(e) => println!("  Error: {}", e),
    }

    println!("\n{}", "=".repeat(60));
    println!("✅ TrenchBot example completed!");

    Ok(())
}
