/// Live Fire Demo Example with Real Blockchain Operations
///
/// A comprehensive end-to-end demonstration that showcases:
/// - AI-powered blockchain tool calling with Gemini LLM integration
/// - Real Solana devnet operations with cryptographic signing
/// - Clean agent architecture using the riglr framework
/// - Both READ operations (balance queries) and WRITE operations (SOL transfers)
/// - On-chain verification of successful transactions
/// - Secure signer context management for blockchain operations
///
/// This example performs actual blockchain transactions on Solana devnet,
/// demonstrating the complete agent workflow from AI decision-making to
/// cryptographic execution and on-chain confirmation.
///
/// ## Prerequisites
/// - `.env.test` file with `GEMINI_API_KEY`
/// - Private key file at `~/.riglr/keys/solana.key` or `SOLANA_PRIVATE_KEY` env var
/// - Solana devnet SOL in the signing wallet for transaction fees
///
/// ## Security Note
/// This example demonstrates secure key loading from files instead of environment
/// variables. Place your private key in `~/.riglr/keys/solana.key` with restricted
/// permissions (chmod 600).
///
/// Run with: `cargo run --example live_fire_demo`
use anyhow::{Context, Result};
use core::time::Duration;
use rig::client::CompletionClient;
use rig::providers::gemini;
use riglr_agents::agents::tool_calling::{DebuggableCompletionModel, ToolAgentBuilder};
use riglr_agents::toolset::Toolset;
use riglr_agents::{AgentRegistry, Dispatcher, LocalAgentRegistry, Task, TaskType};
use riglr_core::provider::ApplicationContext;
use riglr_core::signer::error::{Error as SignerError, Standard as StandardSignerError};
use riglr_core::signer::{SignerContext, UnifiedSigner};
use riglr_core::util::{ensure_key_directory, load_private_key_with_fallback};
use riglr_solana_tools::signer::Local as SolanaSigner;
use serde_json::json;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    signature::{Keypair, Signer},
};
use std::{env, sync::Arc};
use tokio::time;
use tracing::info;
use tracing_subscriber::fmt;

// Environment variable constants
const GEMINI_API_KEY: &str = "GEMINI_API_KEY";
const ANTHROPIC_API_KEY: &str = "ANTHROPIC_API_KEY";

/// Setup logging and load configuration from environment
fn setup_environment() -> Result<String> {
    dotenvy::from_filename(".env.test").context("Failed to load .env.test file")?;
    fmt::init();

    // Load API key from environment (this is OK for API keys)
    env::var(GEMINI_API_KEY).context("GEMINI_API_KEY must be set")
}

/// Load Solana private key securely from file with env var fallback
fn load_solana_keypair() -> Result<Keypair> {
    info!("Loading Solana private key...");
    let key_dir = ensure_key_directory().context("Failed to create key directory")?;
    let key_path = key_dir.join("solana.key");

    let solana_b58 = load_private_key_with_fallback(&key_path, "SOLANA_PRIVATE_KEY").context(
        "Private key not found. Place it in ~/.riglr/keys/solana.key or set SOLANA_PRIVATE_KEY",
    )?;

    Ok(Keypair::from_base58_string(&solana_b58))
}

/// Setup application context and configure agents
fn setup_application_context(devnet_rpc: &str) -> Result<ApplicationContext> {
    // Set dummy ANTHROPIC_API_KEY if not set for ApplicationContext
    if env::var(ANTHROPIC_API_KEY).is_err() {
        // Expect unsafe code for controlled test environment setup
        #[expect(unsafe_code)]
        // SAFETY: Setting environment variables in examples is safe in controlled execution environment
        unsafe {
            env::set_var(ANTHROPIC_API_KEY, "dummy-key-for-testing");
        }
    }

    let config = riglr_core::Config::from_env();

    // Validate basic configuration (skip blockchain-specific address validation for this demo)
    config
        .network
        .validate_config(None)
        .context("Configuration validation failed")?;

    let app_context = ApplicationContext::from_config(&config);
    app_context.set_extension(Arc::new(RpcClient::new(devnet_rpc)));

    Ok(app_context)
}

/// Execute read operation demo
async fn run_read_demo(
    dispatcher: &Dispatcher<LocalAgentRegistry>,
) -> Result<(), Box<dyn SignerError>> {
    info!("\n\n--- DEMO 1: AI-Powered READ Operation ---");
    let read_prompt = format!(
        "What is the SOL balance of the wallet {}?",
        "Vote111111111111111111111111111111111111111"
    );
    let read_task = Task::new(
        TaskType::Custom("tool_calling".to_string()),
        json!({
            "prompt": read_prompt
        }),
    );
    let read_result = dispatcher.dispatch_task(read_task).await.map_err(|e| {
        Box::new(StandardSignerError::Generic(e.to_string())) as Box<dyn SignerError>
    })?;
    info!(
        "✅ READ Task Complete. Result: {}",
        serde_json::to_string_pretty(&read_result.data())
            .unwrap_or_else(|_| "Error formatting result".to_string())
    );
    Ok(())
}

/// Execute write operation demo
async fn run_write_demo(
    dispatcher: &Dispatcher<LocalAgentRegistry>,
) -> Result<Keypair, Box<dyn SignerError>> {
    info!("\n\n--- DEMO 2: AI-Powered WRITE Operation ---");
    let temp_receiver = Keypair::new();
    info!(
        "Creating temporary receiver wallet: {}",
        temp_receiver.pubkey()
    );
    let write_prompt = format!("Send 0.001 SOL to {}", temp_receiver.pubkey());
    let write_task = Task::new(
        TaskType::Custom("tool_calling".to_string()),
        json!({
            "prompt": write_prompt
        }),
    );
    let write_result = dispatcher.dispatch_task(write_task).await.map_err(|e| {
        Box::new(StandardSignerError::Generic(e.to_string())) as Box<dyn SignerError>
    })?;
    info!(
        "✅ WRITE Task Complete. Result: {}",
        serde_json::to_string_pretty(&write_result.data())
            .unwrap_or_else(|_| "Error formatting result".to_string())
    );
    Ok(temp_receiver)
}

/// Verify the transaction on-chain
async fn verify_on_chain_state(temp_receiver: &Keypair) -> Result<(), Box<dyn SignerError>> {
    info!("\n\n--- FINAL PROOF: Verifying On-Chain State ---");
    info!("Waiting 20 seconds for Devnet to confirm...");
    time::sleep(Duration::from_secs(20)).await;
    let rpc_client = RpcClient::new("https://api.devnet.solana.com");
    let final_balance = rpc_client
        .get_balance(&temp_receiver.pubkey())
        .map_err(|e| {
            Box::new(StandardSignerError::Generic(e.to_string())) as Box<dyn SignerError>
        })?;

    // Casting with explicit truncation and sign loss expectations for financial calculations
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let expected_balance = (0.001 * LAMPORTS_PER_SOL as f64) as u64;

    assert_eq!(
        final_balance, expected_balance,
        "ON-CHAIN BALANCE MISMATCH!"
    );
    info!("✅✅✅ On-chain balance confirmed! The architecture works.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // --- 1. SETUP & LOAD SECRETS ---
    let gemini_api_key = setup_environment()?;
    let signer_keypair = load_solana_keypair()?;
    let devnet_rpc = "https://api.devnet.solana.com";

    info!("--- STARTING LIVE FIRE DEMO (CLEAN ARCHITECTURE) ---");
    info!("Signer Wallet: {}", signer_keypair.pubkey());

    // --- 2. BUILD THE AGENT SYSTEM ---
    let app_context = setup_application_context(devnet_rpc)?;

    // 1. Discover all available tools with the application context.
    let toolset = Toolset::new(Arc::new(app_context));

    // 2. Build the entire agent with one clean, fluent call.
    let gemini_client = gemini::Client::new(&gemini_api_key);
    let model = gemini_client.completion_model("gemini-1.5-flash");
    let debuggable_model = DebuggableCompletionModel::new(model);
    let live_agent = ToolAgentBuilder::new(toolset).build(debuggable_model)?;

    // --- 3. RUN THE DEMO ---
    let solana_b58 = signer_keypair.to_base58_string();
    let solana_signer = SolanaSigner::new(&solana_b58, riglr_config::SolanaNetworkConfig::devnet())
        .map_err(|e| anyhow::anyhow!("Failed to create Solana signer: {}", e))?;
    let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(solana_signer);

    SignerContext::with_signer(unified_signer, async {
        let registry = LocalAgentRegistry::new();
        registry.register_agent(live_agent).await.map_err(|e| {
            Box::new(StandardSignerError::Generic(e.to_string())) as Box<dyn SignerError>
        })?;
        let dispatcher = Dispatcher::new(Arc::new(registry));

        // Execute demos
        run_read_demo(&dispatcher).await?;
        let temp_receiver = run_write_demo(&dispatcher).await?;
        verify_on_chain_state(&temp_receiver).await?;

        Ok::<(), Box<dyn SignerError>>(())
    })
    .await
    .map_err(|e| anyhow::anyhow!("SignerContext execution failed: {}", e))?;

    Ok(())
}
