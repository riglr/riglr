//! Example of using the RugCheck API integration

use riglr_core::provider::ApplicationContext;
use riglr_web_tools::rugcheck::{analyze_token_risks, check_if_rugged, get_token_report};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create a mock application context (in real usage, this would come from your app)
    let context = ApplicationContext::default();

    // Example Solana token mint address (replace with actual token to test)
    let token_mint = "So11111111111111111111111111111111111111112"; // Wrapped SOL

    println!("🔍 Checking token: {}", token_mint);
    println!("{}", "=".repeat(60));

    // Example 1: Quick rug check
    println!("\n📊 Quick Rug Check:");
    match check_if_rugged(&context, token_mint.to_string()).await {
        Ok(result) => {
            println!("  Is Rugged: {}", result.is_rugged);
            println!("  Risk Score: {}/100", result.risk_score);
            println!("  Status: {}", result.status);
        }
        Err(e) => println!("  Error: {}", e),
    }

    // Example 2: Detailed risk analysis
    println!("\n🎯 Detailed Risk Analysis:");
    match analyze_token_risks(&context, token_mint.to_string()).await {
        Ok(analysis) => {
            println!("  Risk Level: {:?}", analysis.risk_level);
            println!("  Risk Score: {}/100", analysis.risk_score);
            println!("  Recommendation: {}", analysis.recommendation);

            if !analysis.critical_risks.is_empty() {
                println!("\n  ⚠️ Critical Risks:");
                for risk in &analysis.critical_risks {
                    println!("    - {}: {}", risk.name, risk.description);
                }
            }

            if let Some(insider) = &analysis.insider_analysis {
                println!("\n  👥 Insider Analysis:");
                println!("    Networks Detected: {}", insider.networks_detected);
                println!("    Insider Accounts: {}", insider.insider_accounts);
            }

            if let Some(liquidity) = &analysis.liquidity_analysis {
                println!("\n  💧 Liquidity Analysis:");
                println!("    Total Liquidity: ${:.2}", liquidity.total_liquidity_usd);
                println!("    LP Providers: {}", liquidity.provider_count);
                println!("    Concentration: {}", liquidity.concentration);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    // Example 3: Full token report (most detailed)
    println!("\n📋 Full Token Report:");
    match get_token_report(&context, token_mint.to_string()).await {
        Ok(report) => {
            println!("  Token: {}", report.mint);
            if let Some(score) = report.score {
                println!("  Score: {}/100", score);
            }
            if let Some(rugged) = report.rugged {
                println!("  Rugged: {}", rugged);
            }
            if let Some(holders) = report.total_holders {
                println!("  Total Holders: {}", holders);
            }
            if let Some(liquidity) = report.total_market_liquidity {
                println!("  Market Liquidity: ${:.2}", liquidity);
            }

            if let Some(risks) = &report.risks {
                if !risks.is_empty() {
                    println!("\n  Risk Factors ({} total):", risks.len());
                    for (i, risk) in risks.iter().take(3).enumerate() {
                        println!(
                            "    {}. {} ({}): {}",
                            i + 1,
                            risk.name,
                            risk.level,
                            risk.description
                        );
                    }
                    if risks.len() > 3 {
                        println!("    ... and {} more", risks.len() - 3);
                    }
                }
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!("\n{}", "=".repeat(60));
    println!("✅ RugCheck example completed!");

    Ok(())
}
