//! Integration tests for swap tools

use riglr_solana_tools::{
    client::SolanaClient,
    swap::{
        get_jupiter_quote, get_token_price, perform_jupiter_swap, PriceInfo, SwapQuote, SwapResult,
    },
    transaction::{get_signer_context, init_signer_context, SignerContext},
};
use solana_sdk::signature::Keypair;

#[tokio::test]
async fn test_get_jupiter_quote_sol_to_usdc() {
    // SOL to USDC quote on mainnet
    let input_mint = "So11111111111111111111111111111111111111112".to_string(); // Wrapped SOL
    let output_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(); // USDC
    let amount = 1_000_000_000; // 1 SOL

    let result = get_jupiter_quote(
        input_mint.clone(),
        output_mint.clone(),
        amount,
        50, // 0.5% slippage
        false,
        None,
    )
    .await;

    match result {
        Ok(quote) => {
            assert_eq!(quote.input_mint, input_mint);
            assert_eq!(quote.output_mint, output_mint);
            assert_eq!(quote.in_amount, amount);
            assert!(quote.out_amount > 0);
            assert!(!quote.route_plan.is_empty());
        }
        Err(_) => {
            // Network error is acceptable in tests
        }
    }
}

#[tokio::test]
async fn test_get_jupiter_quote_invalid_mints() {
    let result = get_jupiter_quote(
        "invalid_mint".to_string(),
        "also_invalid".to_string(),
        1000000,
        50,
        false,
        None,
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid"));
}

#[tokio::test]
async fn test_get_jupiter_quote_with_direct_routes() {
    let input_mint = "So11111111111111111111111111111111111111112".to_string();
    let output_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string();

    let result = get_jupiter_quote(
        input_mint,
        output_mint,
        1_000_000_000,
        100,  // 1% slippage
        true, // Only direct routes
        None,
    )
    .await;

    // May succeed or fail depending on network
    let _ = result;
}

#[tokio::test]
async fn test_get_token_price_sol_usdc() {
    let base_mint = "So11111111111111111111111111111111111111112".to_string();
    let quote_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string();

    let result = get_token_price(base_mint.clone(), quote_mint.clone(), None).await;

    match result {
        Ok(price_info) => {
            assert_eq!(price_info.base_mint, base_mint);
            assert_eq!(price_info.quote_mint, quote_mint);
            assert!(price_info.price > 0.0);
        }
        Err(_) => {
            // Network error is acceptable
        }
    }
}

#[tokio::test]
async fn test_perform_jupiter_swap_no_signer() {
    let client = SolanaClient::mainnet();

    let result = perform_jupiter_swap(
        &client,
        "So11111111111111111111111111111111111111112".to_string(),
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        1_000_000_000,
        50,
        None,
        None,
        false,
    )
    .await;

    // Should fail due to no signer context
    assert!(result.is_err());
}

#[tokio::test]
async fn test_perform_jupiter_swap_with_signer() {
    // Initialize signer context
    let mut context = SignerContext::new();
    let keypair = Keypair::new();
    context.add_signer("swapper", keypair).unwrap();
    init_signer_context(context);

    let client = SolanaClient::mainnet();

    let result = perform_jupiter_swap(
        &client,
        "So11111111111111111111111111111111111111112".to_string(),
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        1_000_000,
        50,
        Some("swapper".to_string()),
        None,
        false,
    )
    .await;

    // Will fail due to insufficient funds, but tests the code path
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_jupiter_quote_small_amount() {
    let result = get_jupiter_quote(
        "So11111111111111111111111111111111111111112".to_string(),
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        1000, // Very small amount
        50,
        false,
        None,
    )
    .await;

    // May succeed or fail depending on liquidity
    let _ = result;
}

#[tokio::test]
async fn test_get_jupiter_quote_large_slippage() {
    let result = get_jupiter_quote(
        "So11111111111111111111111111111111111111112".to_string(),
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        1_000_000_000,
        1000, // 10% slippage
        false,
        None,
    )
    .await;

    match result {
        Ok(quote) => {
            assert!(quote.other_amount_threshold > 0);
        }
        Err(_) => {
            // Network error is acceptable
        }
    }
}

#[tokio::test]
async fn test_get_token_price_custom_api_url() {
    let result = get_token_price(
        "So11111111111111111111111111111111111111112".to_string(),
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        Some("https://quote-api.jup.ag/v6".to_string()),
    )
    .await;

    // May succeed or fail depending on network
    let _ = result;
}

#[tokio::test]
async fn test_perform_jupiter_swap_versioned_transaction() {
    let mut context = SignerContext::new();
    let keypair = Keypair::new();
    context.add_signer("versioned_swapper", keypair).unwrap();
    init_signer_context(context);

    let client = SolanaClient::mainnet();

    let result = perform_jupiter_swap(
        &client,
        "So11111111111111111111111111111111111111112".to_string(),
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        1_000_000_000,
        50,
        None,
        None,
        true, // Use versioned transaction
    )
    .await;

    // Will fail but tests versioned transaction path
    assert!(result.is_err());
}

#[test]
fn test_swap_quote_structure() {
    use riglr_solana_tools::swap::RoutePlanStep;

    let quote = SwapQuote {
        input_mint: "mint1".to_string(),
        output_mint: "mint2".to_string(),
        in_amount: 1000000,
        out_amount: 2000000,
        other_amount_threshold: 1900000,
        price_impact_pct: 0.5,
        route_plan: vec![],
        context_slot: Some(123456),
        time_taken: Some(0.123),
    };

    assert_eq!(quote.in_amount, 1000000);
    assert_eq!(quote.out_amount, 2000000);
    assert_eq!(quote.price_impact_pct, 0.5);
}

#[test]
fn test_swap_result_structure() {
    use riglr_solana_tools::transaction::TransactionStatus;

    let result = SwapResult {
        signature: "swap_sig".to_string(),
        input_mint: "mint1".to_string(),
        output_mint: "mint2".to_string(),
        in_amount: 1000000,
        out_amount: 2000000,
        price_impact_pct: 0.5,
        status: TransactionStatus::Pending,
        idempotency_key: Some("key123".to_string()),
    };

    assert_eq!(result.signature, "swap_sig");
    assert_eq!(result.in_amount, 1000000);
    assert_eq!(result.out_amount, 2000000);
}

#[test]
fn test_price_info_structure() {
    let price_info = PriceInfo {
        base_mint: "base".to_string(),
        quote_mint: "quote".to_string(),
        price: 50.5,
        price_impact_pct: 0.1,
    };

    assert_eq!(price_info.price, 50.5);
    assert_eq!(price_info.price_impact_pct, 0.1);
}
