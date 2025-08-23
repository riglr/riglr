// riglr-agents/examples/live_fire_demo.rs

use riglr_agents::agents::tool_calling::ToolCallingAgentBuilder;
use riglr_agents::toolset::Toolset;
use riglr_agents::*;
use riglr_core::provider::ApplicationContext;
use riglr_core::signer::{SignerContext, UnifiedSigner};
use riglr_solana_tools::signer::LocalSolanaSigner;
use serde_json::json;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    signature::{Keypair, Signer},
};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- 1. SETUP & LOAD SECRETS ---
    dotenvy::from_filename(".env.test").expect("Failed to load .env.test file.");
    tracing_subscriber::fmt::init();

    let openai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set.");
    let solana_b58 = std::env::var("SOLANA_PRIVATE_KEY").expect("SOLANA_PRIVATE_KEY must be set.");
    let signer_keypair = Keypair::from_base58_string(&solana_b58);
    let devnet_rpc = "https://api.devnet.solana.com".to_string();

    info!("--- STARTING LIVE FIRE DEMO (CLEAN ARCHITECTURE) ---");
    info!("Signer Wallet: {}", signer_keypair.pubkey());

    // --- 2. BUILD THE AGENT SYSTEM (THE CLEAN WAY) ---
    // Set dummy ANTHROPIC_API_KEY if not set for ApplicationContext
    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        std::env::set_var("ANTHROPIC_API_KEY", "dummy-key-for-testing");
    }

    let app_context = ApplicationContext::from_env();
    app_context.set_extension(Arc::new(solana_client::rpc_client::RpcClient::new(
        devnet_rpc.clone(),
    )));

    // 1. Discover all available tools with one line.
    let toolset = Toolset::new().with_solana_tools();

    // 2. Build the entire agent with one clean, fluent call.
    // This handles creating the ToolWorker, the rig::Agent, and registering tools everywhere.
    let openai_client = rig::providers::openai::Client::new(&openai_api_key);
    let live_agent = ToolCallingAgentBuilder::new(toolset, app_context)
        .build(openai_client.agent("gpt-4o"))
        .await?;

    // --- 3. RUN THE DEMO ---
    let solana_signer = LocalSolanaSigner::new(signer_keypair, devnet_rpc);
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
            serde_json::to_string_pretty(&read_result.data())?
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
            serde_json::to_string_pretty(&write_result.data())?
        );

        // FINAL PROOF: Verify On-Chain State
        info!("\n\n--- FINAL PROOF: Verifying On-Chain State ---");
        info!("Waiting 20 seconds for Devnet to confirm...");
        tokio::time::sleep(std::time::Duration::from_secs(20)).await;
        let rpc_client =
            solana_client::rpc_client::RpcClient::new("https://api.devnet.solana.com".to_string());
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
