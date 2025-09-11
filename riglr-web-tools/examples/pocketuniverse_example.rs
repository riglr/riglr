//! Example of using the PocketUniverse API integration for rug pull detection

use riglr_core::provider::ApplicationContext;
use riglr_web_tools::pocketuniverse::{
    analyze_rug_risk, check_rug_pull, check_rug_pull_raw, is_token_safe, RiskTolerance,
    RugApiResponse,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create a mock application context (in real usage, this would come from your app)
    let context = ApplicationContext::default();

    // Note: API key is now managed through ApplicationContext
    // Set POCKET_UNIVERSE_API_KEY environment variable before running

    // Example Solana token address (replace with actual token to test)
    let token_address = "Ci6FSXsfctsnXKcPqZxWkMahnApqkgwGroVjU5YPcgJ9"; // Example from spec

    println!("🔍 Analyzing token: {}", token_address);
    println!("{}", "=".repeat(60));

    // Example 1: Raw API response
    println!("\n📊 Raw API Response:");
    let raw_response = check_rug_pull_raw(&context, token_address.to_string()).await;
    match raw_response {
        Ok(response) => match response {
            RugApiResponse::NotProcessed { message } => {
                println!("  Status: Not Processed");
                println!("  Message: {}", message);
            }
            RugApiResponse::Processed {
                message,
                is_scam,
                rug_percent,
                fresh_percent,
            } => {
                println!("  Status: Processed");
                println!("  Is Scam: {}", is_scam);
                println!("  Rug Puller Volume: {:.2}%", rug_percent * 100.0);
                println!("  Fresh Wallet Volume: {:.2}%", fresh_percent * 100.0);
                println!("  Message: {}", message);
            }
        },
        Err(e) => println!("  Error: {}", e),
    }

    // Example 2: Simplified rug check
    println!("\n🎯 Simplified Rug Check:");
    let rug_check_result = check_rug_pull(&context, token_address.to_string()).await;
    match rug_check_result {
        Ok(result) => {
            println!("  Processed: {}", result.is_processed);
            if result.is_processed {
                println!("  Is Scam: {:?}", result.is_scam);
                println!(
                    "  Rug Percentage: {:.2}%",
                    result.rug_percentage.unwrap_or(0.0)
                );
                println!(
                    "  Fresh Percentage: {:.2}%",
                    result.fresh_percentage.unwrap_or(0.0)
                );
            }
            println!("  Risk Level: {:?}", result.risk_level);
            println!("  Message: {}", result.message);
            println!("  Recommendation: {}", result.recommendation);
        }
        Err(e) => println!("  Error: {}", e),
    }

    // Example 3: Detailed rug analysis
    println!("\n🔬 Detailed Rug Analysis:");
    let analysis_result = analyze_rug_risk(&context, token_address.to_string()).await;
    match analysis_result {
        Ok(analysis) => {
            println!("  Status: {:?}", analysis.status);

            if let Some(scam) = &analysis.scam_detection {
                println!("\n  🚨 Scam Detection:");
                println!("    Is Scam: {}", scam.is_scam);
                println!("    Confidence: {:.1}%", scam.confidence);
                println!("    Reason: {}", scam.reason);
            }

            if let Some(volume) = &analysis.volume_analysis {
                println!("\n  📊 Volume Analysis:");
                println!(
                    "    Rug Puller Volume: {:.2}%",
                    volume.rug_puller_percentage
                );
                println!(
                    "    Fresh Wallet Volume: {:.2}%",
                    volume.fresh_wallet_percentage
                );
                println!(
                    "    Regular Trader Volume: {:.2}%",
                    volume.regular_trader_percentage
                );
                println!("    Concentration: {:?}", volume.concentration);
            }

            println!("\n  ⚠️ Risk Assessment:");
            println!("    Risk Level: {:?}", analysis.risk_assessment.level);
            println!("    Risk Score: {:.1}/100", analysis.risk_assessment.score);
            println!("    Action: {}", analysis.risk_assessment.action);

            if !analysis.risk_assessment.factors.is_empty() {
                println!("    Risk Factors:");
                for factor in &analysis.risk_assessment.factors {
                    println!("      - {}", factor);
                }
            }

            if !analysis.warnings.is_empty() {
                println!("\n  🚩 Warnings:");
                for warning in &analysis.warnings {
                    println!("    - {}", warning);
                }
            }

            println!("\n  💡 Recommendation:");
            println!("    {}", analysis.recommendation);
        }
        Err(e) => println!("  Error: {}", e),
    }

    // Example 4: Quick safety check with different risk tolerances
    println!("\n✅ Safety Check (Different Risk Tolerances):");

    for tolerance in [
        RiskTolerance::Low,
        RiskTolerance::Medium,
        RiskTolerance::High,
    ] {
        println!("\n  Risk Tolerance: {:?}", tolerance);
        let safety_result =
            is_token_safe(&context, token_address.to_string(), Some(tolerance)).await;
        match safety_result {
            Ok(safety) => {
                println!("    Is Safe: {}", safety.is_safe);
                println!("    Risk Level: {:?}", safety.risk_level);
                println!("    Safety Score: {:.1}/100", safety.safety_score);
                println!("    Verdict: {}", safety.verdict);
            }
            Err(e) => println!("    Error: {}", e),
        }
    }

    // Example with a different token (you can test with various addresses)
    println!("\n{}", "=".repeat(60));
    println!("\n📝 Testing with Wrapped SOL (should be safe):");
    let wsol = "So11111111111111111111111111111111111111112";

    let wsol_check_result = check_rug_pull(&context, wsol.to_string()).await;
    match wsol_check_result {
        Ok(result) => {
            println!("  Token: {}", wsol);
            println!("  Risk Level: {:?}", result.risk_level);
            if result.is_processed {
                println!("  Is Scam: {:?}", result.is_scam);
                if let Some(rug_pct) = result.rug_percentage {
                    println!("  Rug Percentage: {:.2}%", rug_pct);
                }
            }
            println!("  Recommendation: {}", result.recommendation);
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!("\n{}", "=".repeat(60));
    println!("✅ PocketUniverse example completed!");
    println!("\nNote: To use this with real tokens, you'll need to:");
    println!("1. Get an API key from PocketUniverse");
    println!("2. Set POCKET_UNIVERSE_API_KEY environment variable");
    println!("3. Replace the example token addresses with ones you want to check");

    Ok(())
}
