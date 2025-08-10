//! Example: Trading tokens on Uniswap V3
//!
//! This example demonstrates how to get quotes and execute swaps on Uniswap.

use riglr_evm_tools::{
    get_uniswap_quote, perform_uniswap_swap, EvmClient,
    UniswapQuote, SwapResult,
};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🦄 Uniswap V3 Trading Example\n");

    // Get private key from environment (required for swaps)
    let private_key = env::var("EVM_PRIVATE_KEY")
        .unwrap_or_else(|_| {
            println!("⚠️  Warning: EVM_PRIVATE_KEY not set. Running in read-only mode.");
            println!("   Set EVM_PRIVATE_KEY to execute actual swaps.\n");
            String::new()
        });

    // Create EVM client (with optional signer for transactions)
    let client = if !private_key.is_empty() {
        EvmClient::mainnet().await?.with_signer(&private_key)?
    } else {
        EvmClient::mainnet().await?
    };

    // Common token addresses on Ethereum mainnet
    const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    const DAI: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";

    // Example 1: Get a quote for swapping USDC to ETH
    println!("📊 Getting quote for 1000 USDC -> WETH swap...");
    let quote = get_quote_example(&client, USDC, WETH, "1000", 6, 18).await?;
    
    // Example 2: Get quotes for different fee tiers
    println!("\n📊 Comparing quotes across fee tiers...");
    compare_fee_tiers(&client, WETH, USDC, "1", 18, 6).await?;

    // Example 3: Simulate a swap (dry run)
    if client.has_signer() {
        println!("\n🔄 Simulating a swap (not executed)...");
        simulate_swap(&client, USDC, WETH, "100", 6, quote.amount_out_minimum.clone()).await?;
    }

    // Example 4: Calculate price impact
    println!("\n📈 Analyzing price impact...");
    analyze_price_impact(&client, WETH, DAI, 18, 18).await?;

    println!("\n✅ Uniswap example complete!");

    Ok(())
}

async fn get_quote_example(
    client: &EvmClient,
    token_in: &str,
    token_out: &str,
    amount: &str,
    decimals_in: u8,
    decimals_out: u8,
) -> anyhow::Result<UniswapQuote> {
    match get_uniswap_quote(
        client,
        token_in.to_string(),
        token_out.to_string(),
        amount.to_string(),
        decimals_in,
        decimals_out,
        Some(3000), // 0.3% fee tier
        Some(50),   // 0.5% slippage
    )
    .await
    {
        Ok(quote) => {
            print_quote(&quote);
            Ok(quote)
        }
        Err(e) => {
            println!("  ❌ Error getting quote: {}", e);
            Err(e.into())
        }
    }
}

async fn compare_fee_tiers(
    client: &EvmClient,
    token_in: &str,
    token_out: &str,
    amount: &str,
    decimals_in: u8,
    decimals_out: u8,
) -> anyhow::Result<()> {
    let fee_tiers = vec![
        (100, "0.01%"),
        (500, "0.05%"),
        (3000, "0.3%"),
        (10000, "1%"),
    ];

    println!("  Comparing fee tiers for {} {} swap:", amount, token_in);
    
    for (fee, description) in fee_tiers {
        match get_uniswap_quote(
            client,
            token_in.to_string(),
            token_out.to_string(),
            amount.to_string(),
            decimals_in,
            decimals_out,
            Some(fee),
            Some(50),
        )
        .await
        {
            Ok(quote) => {
                println!("  • {} tier: {} output (price: {:.6})",
                    description,
                    quote.amount_out,
                    quote.price
                );
            }
            Err(_) => {
                println!("  • {} tier: No liquidity", description);
            }
        }
    }

    Ok(())
}

async fn simulate_swap(
    client: &EvmClient,
    token_in: &str,
    token_out: &str,
    amount: &str,
    decimals_in: u8,
    amount_out_min: String,
) -> anyhow::Result<()> {
    println!("  🔄 Swap details:");
    println!("    • Token In: {}", token_in);
    println!("    • Token Out: {}", token_out);
    println!("    • Amount In: {}", amount);
    println!("    • Min Amount Out: {}", amount_out_min);
    println!("    • Fee Tier: 0.3%");
    println!("    • Deadline: 5 minutes");
    
    // In a real scenario, you would call perform_uniswap_swap here
    // For this example, we're just simulating
    println!("  ✅ Swap simulation complete (not executed)");
    
    Ok(())
}

async fn analyze_price_impact(
    client: &EvmClient,
    token_in: &str,
    token_out: &str,
    decimals_in: u8,
    decimals_out: u8,
) -> anyhow::Result<()> {
    let amounts = vec!["0.1", "1", "10", "100"];
    
    println!("  Analyzing price impact for {} -> {} swaps:", token_in, token_out);
    
    let mut base_price = 0.0;
    
    for (i, amount) in amounts.iter().enumerate() {
        match get_uniswap_quote(
            client,
            token_in.to_string(),
            token_out.to_string(),
            amount.to_string(),
            decimals_in,
            decimals_out,
            Some(3000),
            Some(50),
        )
        .await
        {
            Ok(quote) => {
                if i == 0 {
                    base_price = quote.price;
                }
                
                let impact = if base_price > 0.0 {
                    ((quote.price - base_price) / base_price * 100.0).abs()
                } else {
                    0.0
                };
                
                println!("    • {} tokens: Price {:.6}, Impact: {:.2}%",
                    amount,
                    quote.price,
                    impact
                );
            }
            Err(_) => {
                println!("    • {} tokens: Unable to get quote", amount);
            }
        }
    }
    
    Ok(())
}

fn print_quote(quote: &UniswapQuote) {
    println!("  📈 Quote Details:");
    println!("    • Input: {} {}", quote.amount_in, quote.token_in);
    println!("    • Output: {} {}", quote.amount_out, quote.token_out);
    println!("    • Price: {:.6}", quote.price);
    println!("    • Fee Tier: {:.2}%", quote.fee_tier as f64 / 10000.0);
    println!("    • Slippage: {:.1}%", quote.slippage_bps as f64 / 100.0);
    println!("    • Min Output: {}", quote.amount_out_minimum);
}

fn print_swap_result(result: &SwapResult) {
    println!("  🎯 Swap Executed:");
    println!("    • Transaction: {}", result.transaction_hash);
    println!("    • Input: {} {}", result.amount_in, result.token_in);
    println!("    • Output: {} {}", result.amount_out, result.token_out);
    if let Some(gas) = &result.gas_used {
        println!("    • Gas Used: {}", gas);
    }
    println!("    • Block: #{}", result.block_number);
}