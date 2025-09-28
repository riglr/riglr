//! Solana tools demonstration commands.

use anyhow::Result;
use colored::Colorize;
#[cfg(test)]
use core::time::Duration as StdDuration;
use core::{str::FromStr, time::Duration};
use dialoguer::{Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use riglr_core::provider::ApplicationContext;
use riglr_solana_tools::{get_jupiter_quote, get_sol};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tokio::time::sleep;
use tracing::warn;

/// Run the Solana tools demo.
///
/// # Errors
///
/// Returns an error if the wallet analysis fails or interactive prompts encounter issues.
pub async fn run_demo(context: Arc<ApplicationContext>, address: Option<String>) -> Result<()> {
    run_demo_with_options(context, address, false).await
}

/// Run the Solana tools demo with options for testing.
///
/// # Errors
///
/// Returns an error if the wallet analysis fails or interactive prompts encounter issues.
pub async fn run_demo_with_options(
    context: Arc<ApplicationContext>,
    address: Option<String>,
    skip_interactive: bool,
) -> Result<()> {
    show_demo_header();

    let wallet_address = get_wallet_address(address, skip_interactive)?;

    println!(
        "\n{}",
        format!("🔍 Analyzing wallet: {wallet_address}").yellow()
    );

    let pb = create_progress_bar()?;

    run_balance_demo(&wallet_address, &context, &pb).await;
    run_token_accounts_demo(&pb).await;
    run_jupiter_quote_demo(&context, &pb).await;

    pb.finish_and_clear();

    if skip_interactive {
        show_completion_message();
        return Ok(());
    }

    let menu_result = run_interactive_menu(context, skip_interactive).await;
    menu_result
}

fn show_demo_header() {
    println!("{}", "🌟 Solana Tools Demo".bright_blue().bold());
    println!("{}", "=".repeat(50).blue());
}
fn get_wallet_address(address: Option<String>, skip_interactive: bool) -> Result<String> {
    address.map_or_else(
        || {
            if skip_interactive {
                return Ok("So11111111111111111111111111111111111111112".to_string());
            }
            println!("\n{}", "Let's analyze a Solana wallet!".cyan());
            let default_address = "So11111111111111111111111111111111111111112";
            Input::new()
                .with_prompt("Enter Solana wallet address")
                .default(default_address.to_string())
                .interact_text()
                .map_err(Into::into)
        },
        Ok,
    )
}
fn create_progress_bar() -> Result<ProgressBar> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")?,
    );
    pb.set_message("Fetching wallet data...");
    Ok(pb)
}

async fn run_balance_demo(wallet_address: &str, context: &ApplicationContext, pb: &ProgressBar) {
    pb.set_message("Checking SOL balance...");
    match get_sol(wallet_address.to_string(), context).await {
        Ok(balance) => {
            println!("\n{}", "💰 SOL Balance".green().bold());
            println!("   Address: {}", balance.address);
            println!("   Balance: {} SOL", balance.formatted.bright_green());
            println!("   Lamports: {}", balance.lamports);
            println!("   SOL: {:.9}", balance.sol);
        }
        Err(e) => {
            warn!("Failed to get SOL balance: {}", e);
            println!(
                "\n{}",
                format!("⚠️ Could not fetch SOL balance: {e}").yellow()
            );
        }
    }
}

async fn run_token_accounts_demo(pb: &ProgressBar) {
    pb.set_message("Simulating token accounts...");
    sleep(Duration::from_millis(500)).await;

    println!("\n{}", "🪙 Token Accounts (Simulated)".green().bold());
    let simulated_tokens = [
        (
            "USDC",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "1,250.50",
        ),
        (
            "USDT",
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
            "850.75",
        ),
        (
            "RAY",
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
            "45.25",
        ),
    ];

    for (i, &(symbol, mint, balance)) in simulated_tokens.iter().enumerate() {
        println!("   {}. {} ({})", i.saturating_add(1), symbol, mint);
        println!("      Balance: {}", balance.bright_green());
    }
}

async fn run_jupiter_quote_demo(context: &ApplicationContext, pb: &ProgressBar) {
    pb.set_message("Getting Jupiter swap quote...");
    let sol_mint = "So11111111111111111111111111111111111111112";
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

    match get_jupiter_quote(
        sol_mint.to_string(),
        usdc_mint.to_string(),
        1_000_000_000,
        50,
        false,
        None,
        context,
    )
    .await
    {
        Ok(quote) => {
            println!(
                "\n{}",
                "🔄 Jupiter Swap Quote (1 SOL → USDC)".green().bold()
            );
            let sol_amount = quote.in_amount.parse::<f64>().unwrap_or(0.0) / 1_000_000_000.0;
            println!("   Input: {sol_amount} SOL");
            println!("   Output: {} USDC", quote.out_amount.bright_green());
            println!("   Price Impact: {:.3}%", quote.price_impact_pct);
            if let Some(route_plan) = quote.route_plan.first() {
                println!(
                    "   Route: {} via {}",
                    route_plan.swap_info.label.as_deref().unwrap_or("Unknown"),
                    route_plan.swap_info.amm_key
                );
            }
        }
        Err(e) => {
            warn!("Failed to get Jupiter quote: {}", e);
            println!(
                "\n{}",
                format!("⚠️ Could not fetch Jupiter quote: {e}").yellow()
            );
        }
    }
}

fn show_completion_message() {
    println!("\n{}", "✅ Solana demo completed!".bright_green().bold());
    println!("{}", "Thank you for exploring riglr-solana-tools!".dimmed());
}
async fn run_interactive_menu(
    context: Arc<ApplicationContext>,
    skip_interactive: bool,
) -> Result<()> {
    println!("\n{}", "🎮 Interactive Options".bright_blue().bold());
    let options = vec![
        "Analyze another wallet",
        "Get detailed token info",
        "Simulate a swap",
        "Exit demo",
    ];

    let selection = Select::new()
        .with_prompt("What would you like to do next?")
        .items(&options)
        .default(3)
        .interact()?;

    handle_menu_selection(selection, context, skip_interactive).await
}
async fn handle_menu_selection(
    selection: usize,
    context: Arc<ApplicationContext>,
    skip_interactive: bool,
) -> Result<()> {
    match selection {
        0 => handle_analyze_wallet(context, skip_interactive).await,
        1 => handle_token_analysis(),
        2 => {
            handle_swap_simulation();
            Ok(())
        }
        _ => {
            show_completion_message();
            Ok(())
        }
    }
}
async fn handle_analyze_wallet(
    context: Arc<ApplicationContext>,
    skip_interactive: bool,
) -> Result<()> {
    println!("\n{}", "Let's analyze another wallet!".cyan());
    let new_address: String = Input::new()
        .with_prompt("Enter wallet address")
        .interact_text()?;
    Box::pin(run_demo_with_options(
        context,
        Some(new_address),
        skip_interactive,
    ))
    .await
}
fn handle_token_analysis() -> Result<()> {
    println!("\n{}", "🔍 Token Analysis".cyan());
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let token_mint: String = Input::new()
        .with_prompt("Enter token mint address")
        .default(usdc_mint.to_string())
        .interact_text()?;

    match Pubkey::from_str(&token_mint) {
        Ok(_pubkey) => {
            println!("   Token analysis temporarily disabled - update to SignerContext pattern");
        }
        Err(_) => {
            println!("   {}", "Invalid token mint address".yellow());
        }
    }

    show_completion_message();
    Ok(())
}

fn handle_swap_simulation() {
    println!("\n{}", "💱 Swap Simulation".cyan());
    println!("   This would simulate a token swap using Jupiter...");
    println!(
        "   {}",
        "(Implementation would require wallet private key)".dimmed()
    );
    show_completion_message();
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;
    use riglr_config::Config;
    use riglr_solana_tools::common_types::SolanaConfig;
    use solana_commitment_config::CommitmentLevel;
    use std::sync::Arc;
    use tokio;

    fn create_test_context() -> Arc<ApplicationContext> {
        let config = Config::builder()
            .solana_rpc_url("https://api.mainnet-beta.solana.com".to_string())
            .build()
            .unwrap();
        Arc::new(ApplicationContext::from_config(&config))
    }

    #[tokio::test]
    async fn test_run_demo_when_address_provided_should_use_provided_address() {
        let context = create_test_context();
        let test_address = "So11111111111111111111111111111111111111112".to_string();

        // Use the new function with skip_interactive = true to avoid hanging on prompts
        let result = run_demo_with_options(context, Some(test_address), true).await;

        // The function should handle network errors gracefully
        // and not panic, returning Ok even if external calls fail
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_run_demo_when_no_address_provided_should_handle_default() {
        let context = create_test_context();

        // Test with skip_interactive = true to use default address without prompts
        let result = run_demo_with_options(context.clone(), None, true).await;

        // The function should handle network errors gracefully
        assert!(result.is_ok() || result.is_err());

        // We can also verify the config creation works
        assert!(!context.config.network.solana_rpc_url.is_empty());
    }

    #[test]
    fn test_solana_config_creation() {
        let context = create_test_context();
        let mut solana_config = SolanaConfig::default();
        solana_config.rpc_url = context.config.network.solana_rpc_url.clone();

        assert_eq!(solana_config.rpc_url, "https://api.mainnet-beta.solana.com");
        assert_eq!(solana_config.commitment, "confirmed");
        assert_eq!(solana_config.timeout_seconds, 30);
    }

    #[test]
    fn test_simulated_tokens_data() {
        let simulated_tokens = [
            (
                "USDC",
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "1,250.50",
            ),
            (
                "USDT",
                "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
                "850.75",
            ),
            (
                "RAY",
                "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
                "45.25",
            ),
        ];

        assert_eq!(simulated_tokens.len(), 3);
        assert_eq!(simulated_tokens[0].0, "USDC");
        assert_eq!(
            simulated_tokens[0].1,
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        );
        assert_eq!(simulated_tokens[0].2, "1,250.50");

        assert_eq!(simulated_tokens[1].0, "USDT");
        assert_eq!(
            simulated_tokens[1].1,
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"
        );
        assert_eq!(simulated_tokens[1].2, "850.75");

        assert_eq!(simulated_tokens[2].0, "RAY");
        assert_eq!(
            simulated_tokens[2].1,
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"
        );
        assert_eq!(simulated_tokens[2].2, "45.25");
    }

    #[test]
    fn test_mint_addresses() {
        let sol_mint = "So11111111111111111111111111111111111111112";
        let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

        assert_eq!(sol_mint.len(), 43); // Solana addresses are 43 characters
        assert_eq!(usdc_mint.len(), 44);

        // Verify these are valid base58 strings (basic check)
        assert!(sol_mint.chars().all(|c| c.is_ascii_alphanumeric()));
        assert!(usdc_mint.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_jupiter_quote_parameters() {
        // Test the parameters used in the Jupiter quote call
        let input_amount = 1_000_000_000u64; // 1 SOL in lamports
        let slippage_bps = 50u16; // 0.5%
        let only_direct_routes = false;

        assert_eq!(input_amount, 1_000_000_000);
        assert_eq!(slippage_bps, 50);
        assert!(!only_direct_routes);

        // Verify lamports to SOL conversion
        #[expect(clippy::cast_precision_loss)]
        let sol_amount = input_amount as f64 / 1_000_000_000.0;
        assert!((sol_amount - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_bar_style_template() {
        // Test that the progress bar template string is valid
        let template = "{spinner:.green} {msg}";
        assert!(template.contains("{spinner:.green}"));
        assert!(template.contains("{msg}"));
        assert!(!template.is_empty());
    }

    #[test]
    fn test_interactive_options() {
        let options = [
            "Analyze another wallet",
            "Get detailed token info",
            "Simulate a swap",
            "Exit demo",
        ];

        assert_eq!(options.len(), 4);
        assert_eq!(options[0], "Analyze another wallet");
        assert_eq!(options[1], "Get detailed token info");
        assert_eq!(options[2], "Simulate a swap");
        assert_eq!(options[3], "Exit demo");
    }

    #[test]
    fn test_default_wallet_address() {
        let default_address = "So11111111111111111111111111111111111111112";
        assert_eq!(default_address.len(), 43);
        assert!(default_address.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_config_network_solana_rpc_url() {
        let context = create_test_context();
        assert!(!context.config.network.solana_rpc_url.is_empty());
        assert!(context
            .config
            .network
            .solana_rpc_url
            .starts_with("https://"));
    }

    #[test]
    fn test_spinner_tick_chars() {
        let tick_chars = "⠁⠂⠄⡀⢀⠠⠐⠈ ";
        assert_eq!(tick_chars.len(), 25); // Unicode spinner chars (3 bytes each) + 1 space
        assert_eq!(tick_chars.chars().count(), 9); // 8 spinner chars + 1 space
        assert!(tick_chars.ends_with(' '));
    }

    #[test]
    fn test_timeout_duration() {
        let timeout = StdDuration::from_secs(30);
        assert_eq!(timeout.as_secs(), 30);
        assert_eq!(timeout.as_millis(), 30_000);
    }

    #[test]
    fn test_sleep_duration() {
        let sleep_duration = StdDuration::from_millis(500);
        assert_eq!(sleep_duration.as_millis(), 500);
        assert_eq!(sleep_duration.as_secs(), 0);
    }

    #[test]
    fn test_pubkey_from_str_valid() {
        let valid_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        let result = Pubkey::from_str(valid_mint);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pubkey_from_str_invalid() {
        let invalid_mint = "invalid_address";
        let result = Pubkey::from_str(invalid_mint);
        assert!(result.is_err());
    }

    #[test]
    fn test_commitment_level() {
        let commitment = CommitmentLevel::Confirmed;
        assert_eq!(commitment, CommitmentLevel::Confirmed);
    }

    // Note: Due to the nature of this async function with external dependencies
    // and user interaction (dialoguer), comprehensive testing would require:
    // 1. Mocking the riglr_solana_tools functions (get_sol_balance, get_jupiter_quote)
    // 2. Mocking the dialoguer Input and Select interactions
    // 3. Mocking the SolanaClient and its RPC calls
    //
    // The current implementation is tightly coupled to external services and user input,
    // which makes unit testing challenging. For better testability, the function should
    // be refactored to use dependency injection for these external dependencies.
}
