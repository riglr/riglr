//! Example: Get Uniswap quotes and execute swaps

use riglr_core::provider::ApplicationContext;
use riglr_evm_tools::get_uniswap_quote;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create application context
    let context = ApplicationContext::from_env();

    // Token addresses on Ethereum mainnet
    let usdc = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    let dai = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();

    println!("Getting Uniswap V3 quotes...\n");

    // Get quote for swapping 1000 USDC to WETH
    println!("Quote: 1000 USDC -> WETH");
    match get_uniswap_quote(
        usdc.clone(),
        weth.clone(),
        "1000".to_string(), // 1000 USDC
        6,                  // USDC decimals
        18,                 // WETH decimals
        Some(3000),         // 0.3% fee tier
        Some(50),           // 0.5% slippage
        Some(1),            // Ethereum mainnet
        &context,
    )
    .await
    {
        Ok(quote) => {
            println!("✓ Input: {} {}", quote.amount_in, quote.token_in);
            println!("  Output: {} {}", quote.amount_out, quote.token_out);
            println!("  Price: {:.6}", quote.price);
            println!(
                "  Fee tier: {:.2}%",
                quote.fee_tier.parse::<f64>().unwrap_or(0.0) / 10000.0
            );
            println!("  Min output: {}", quote.amount_out_minimum);
        }
        Err(e) => {
            println!("✗ Failed to get quote: {}", e);
        }
    }

    // Get additional quotes
    println!("\nAdditional Quotes:");

    // Quote for 1 WETH to USDC
    println!("\nQuote: 1 WETH -> USDC");
    match get_uniswap_quote(
        weth.clone(),
        usdc.clone(),
        "1".to_string(), // 1 WETH
        18,              // WETH decimals
        6,               // USDC decimals
        Some(3000),      // 0.3% fee tier
        Some(50),        // 0.5% slippage
        Some(1),         // Ethereum mainnet
        &context,
    )
    .await
    {
        Ok(quote) => {
            println!("✓ Input: {} {}", quote.amount_in, quote.token_in);
            println!("  Output: {} {}", quote.amount_out, quote.token_out);
            println!("  Price: {:.2} USDC per WETH", quote.price);
        }
        Err(e) => {
            println!("✗ Failed to get quote: {}", e);
        }
    }

    // Quote for DAI to USDC (stablecoin pair)
    println!("\nQuote: 1000 DAI -> USDC");
    match get_uniswap_quote(
        dai.clone(),
        usdc.clone(),
        "1000".to_string(), // 1000 DAI
        18,                 // DAI decimals
        6,                  // USDC decimals
        Some(500),          // 0.05% fee tier for stablecoins
        Some(10),           // 0.1% slippage for stablecoins
        Some(1),            // Ethereum mainnet
        &context,
    )
    .await
    {
        Ok(quote) => {
            println!("✓ Input: {} {}", quote.amount_in, quote.token_in);
            println!("  Output: {} {}", quote.amount_out, quote.token_out);
            println!("  Price: {:.6} USDC per DAI", quote.price);
        }
        Err(e) => {
            println!("✗ Failed to get quote: {}", e);
        }
    }

    // Example of how to execute a swap (requires signer)
    println!("\nTo execute a swap, you would need to:");
    println!("1. Create a client with a signer (private key)");
    println!("2. Call perform_uniswap_swap with the quote parameters");
    println!("3. Wait for transaction confirmation");

    // Example code (DO NOT use real private keys in examples!)
    /*
    // Create client with signer
    let private_key = "your_private_key_here";
    let client_with_signer = EvmClient::mainnet().await?
        .with_signer(private_key)?;

    // Execute swap
    let swap_result = perform_uniswap_swap(
        usdc,
        weth,
        "1000".to_string(),  // 1000 USDC
        6,                    // USDC decimals
        "950000".to_string(), // Amount out minimum (example)
        Some(3000),           // 0.3% fee tier
        Some(300),            // Deadline in seconds
    ).await?;

    println!("Swap transaction: {}", swap_result.tx_hash);
    println!("Amount out: {}", swap_result.amount_out);
    */

    Ok(())
}
