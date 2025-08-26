//! Live Fire Demo Example with Real Blockchain Operations
//!
//! A comprehensive end-to-end demonstration that showcases:
//! - AI-powered blockchain tool calling with Gemini LLM integration
//! - Real Solana devnet operations with cryptographic signing
//! - Clean agent architecture using the riglr framework
//! - Both READ operations (balance queries) and WRITE operations (SOL transfers)
//! - On-chain verification of successful transactions
//! - Secure signer context management for blockchain operations
//!
//! This example performs actual blockchain transactions on Solana devnet,
//! demonstrating the complete agent workflow from AI decision-making to
//! cryptographic execution and on-chain confirmation.
//!
//! ## Prerequisites
//! - `.env.test` file with `GEMINI_API_KEY`
//! - Private key file at `~/.riglr/keys/solana.key` or `SOLANA_PRIVATE_KEY` env var
//! - Solana devnet SOL in the signing wallet for transaction fees
//!
//! ## Security Note
//! This example demonstrates secure key loading from files instead of environment
//! variables. Place your private key in `~/.riglr/keys/solana.key` with restricted
//! permissions (chmod 600).
//!
//! Run with: `cargo run --example live_fire_demo`

use rig::client::CompletionClient;
use riglr_agents::agents::tool_calling::{DebuggableCompletionModel, ToolCallingAgentBuilder};
use riglr_agents::toolset::Toolset;
use riglr_agents::*;
use riglr_core::provider::ApplicationContext;
use riglr_core::signer::{SignerContext, UnifiedSigner};
use riglr_core::util::{ensure_key_directory, load_private_key_with_fallback};
use riglr_solana_tools::signer::LocalSolanaSigner;
use serde_json::json;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    signature::{Keypair, Signer},
};
use std::sync::Arc;
use tracing::info;

// Environment variable constants
const GEMINI_API_KEY: &str = "GEMINI_API_KEY";
const ANTHROPIC_API_KEY: &str = "ANTHROPIC_API_KEY";

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // --- 1. SETUP & LOAD SECRETS ---
    dotenvy::from_filename(".env.test").expect("Failed to load .env.test file.");
    tracing_subscriber::fmt::init();

    // Load API key from environment (this is OK for API keys)
    let gemini_api_key = std::env::var(GEMINI_API_KEY).expect("GEMINI_API_KEY must be set.");

    // Load private key securely from file with env var fallback
    info!("Loading Solana private key...");
    let key_dir = ensure_key_directory().expect("Failed to create key directory");
    let key_path = key_dir.join("solana.key");

    let solana_b58 = load_private_key_with_fallback(&key_path, "SOLANA_PRIVATE_KEY").expect(
        "Private key not found. Place it in ~/.riglr/keys/solana.key or set SOLANA_PRIVATE_KEY",
    );

    let signer_keypair = Keypair::from_base58_string(&solana_b58);
    let devnet_rpc = "https://api.devnet.solana.com";

    info!("--- STARTING LIVE FIRE DEMO (CLEAN ARCHITECTURE) ---");
    info!("Signer Wallet: {}", signer_keypair.pubkey());

    // --- 2. BUILD THE AGENT SYSTEM (THE CLEAN WAY) ---
    // Set dummy ANTHROPIC_API_KEY if not set for ApplicationContext
    if std::env::var(ANTHROPIC_API_KEY).is_err() {
        std::env::set_var(ANTHROPIC_API_KEY, "dummy-key-for-testing");
    }

    let config = riglr_core::Config::from_env();

    // Validate configuration with EVM address validator if needed
    // This shows the new pattern for address validation after Task 1 architectural changes
    #[cfg(feature = "evm")]
    {
        use riglr_evm_common::validation::EvmAddressValidator;
        config
            .network
            .validate_config(Some(&EvmAddressValidator))
            .expect("Invalid EVM addresses in configuration");
    }
    #[cfg(not(feature = "evm"))]
    {
        // Skip EVM address validation when EVM support is disabled
        config
            .network
            .validate_config(None)
            .expect("Configuration validation failed");
    }

    let app_context = ApplicationContext::from_config(&config);
    app_context.set_extension(Arc::new(solana_client::rpc_client::RpcClient::new(
        devnet_rpc,
    )));

    // 1. Discover all available tools with one line.
    let toolset = Toolset::default().with_solana_tools();

    // 2. Build the entire agent with one clean, fluent call.
    // This handles creating the ToolWorker, the rig::Agent, and registering tools everywhere.
    let gemini_client = rig::providers::gemini::Client::new(&gemini_api_key);
    let model = gemini_client.completion_model("gemini-1.5-flash");
    let debuggable_model = DebuggableCompletionModel::new(model);
    let live_agent = ToolCallingAgentBuilder::new(toolset, app_context)
        .build(debuggable_model)
        .await?;

    // --- 3. RUN THE DEMO ---
    let solana_signer = LocalSolanaSigner::new(signer_keypair, devnet_rpc.to_string());
    let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(solana_signer);

    SignerContext::with_signer(unified_signer, async {
        let registry = LocalAgentRegistry::new();
        registry.register_agent(live_agent).await.unwrap();
        let dispatcher = AgentDispatcher::new(Arc::new(registry));

        // DEMO 1: READ Operation
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
        let read_result = dispatcher.dispatch_task(read_task).await.unwrap();
        info!(
            "✅ READ Task Complete. Result: {}",
            serde_json::to_string_pretty(&read_result.data())
                .unwrap_or_else(|_| "Error formatting result".to_string())
        );

        // DEMO 2: WRITE Operation
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
        let write_result = dispatcher.dispatch_task(write_task).await.unwrap();
        info!(
            "✅ WRITE Task Complete. Result: {}",
            serde_json::to_string_pretty(&write_result.data())
                .unwrap_or_else(|_| "Error formatting result".to_string())
        );

        // FINAL PROOF: Verify On-Chain State
        info!("\n\n--- FINAL PROOF: Verifying On-Chain State ---");
        info!("Waiting 20 seconds for Devnet to confirm...");
        tokio::time::sleep(std::time::Duration::from_secs(20)).await;
        let rpc_client = solana_client::rpc_client::RpcClient::new("https://api.devnet.solana.com");
        let final_balance = rpc_client.get_balance(&temp_receiver.pubkey())?;
        let expected_balance = (0.001 * LAMPORTS_PER_SOL as f64) as u64;
        assert_eq!(
            final_balance, expected_balance,
            "ON-CHAIN BALANCE MISMATCH!"
        );
        info!("✅✅✅ On-chain balance confirmed! The architecture works.");

        Ok::<(), riglr_core::signer::SignerError>(())
    })
    .await
    .unwrap();

    Ok(())
}
