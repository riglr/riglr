//! Example: Get Uniswap quotes and execute swaps

use riglr_evm_tools::swap::{get_token_price, get_uniswap_quote};
use riglr_evm_tools::transaction::{init_evm_signer_context, EvmSignerContext};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

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
        "1000000000".to_string(), // 1000 USDC (6 decimals)
        3000,                     // 0.3% fee tier
        None,
        None,
    )
    .await
    {
        Ok(quote) => {
            let weth_amount = quote.amount_out as f64 / 1e18; // WETH has 18 decimals
            println!("✓ Input: {} USDC", quote.amount_in as f64 / 1e6);
            println!("  Output: {:.6} WETH", weth_amount);
            println!("  Price impact: {:.2}%", quote.price_impact_pct);
            println!("  Fee tier: {}bps", quote.fee_tier / 100);
            println!("  Router: {}", quote.router_address);
        }
        Err(e) => {
            println!("✗ Failed to get quote: {}", e);
        }
    }

    // Get token prices
    println!("\nToken Prices:");

    // USDC price in WETH
    match get_token_price(usdc.clone(), weth.clone(), Some(3000), None).await {
        Ok(price_info) => {
            println!("✓ USDC/WETH: {:.8} WETH per USDC", price_info.price);
        }
        Err(e) => {
            println!("✗ Failed to get USDC price: {}", e);
        }
    }

    // DAI price in USDC
    match get_token_price(
        dai.clone(),
        usdc.clone(),
        Some(500), // 0.05% fee tier for stablecoins
        None,
    )
    .await
    {
        Ok(price_info) => {
            println!("✓ DAI/USDC: {:.6} USDC per DAI", price_info.price);
        }
        Err(e) => {
            println!("✗ Failed to get DAI price: {}", e);
        }
    }

    // Example of how to execute a swap (requires signer context)
    println!("\nTo execute a swap, you would need to:");
    println!("1. Initialize a signer context with your private key");
    println!("2. Call perform_uniswap_swap with the quote parameters");
    println!("3. Wait for transaction confirmation");

    // Example code (DO NOT use real private keys in examples!)
    /*
    let mut context = EvmSignerContext::new();
    context.add_signer("main", [0u8; 32])?; // Replace with actual key
    init_evm_signer_context(context);

    let swap_result = perform_uniswap_swap(
        usdc,
        weth,
        "1000000000".to_string(),
        "500000000000000000".to_string(), // Min 0.5 WETH
        3000,
        None,
        None,
        None,
        None,
        None,
        Some("swap-example-1".to_string()),
    ).await?;

    println!("Swap transaction: {}", swap_result.tx_hash);
    */

    Ok(())
}
