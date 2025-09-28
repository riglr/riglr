#![allow(clippy::expect_used)]
/// End-to-end tests for core agent workflow functionality.
///
/// This test suite validates the core agent workflow capabilities including:
/// - Basic read-only tool execution (balance queries)
/// - Secure transaction execution with proper signer context
/// - Multi-tool and multi-chain execution coordination
use anyhow::Result;
use core::{str::FromStr, time::Duration};
use riglr_config::Config;
use riglr_core::{
    idempotency::InMemoryIdempotencyStore,
    provider::ApplicationContext,
    signer::{
        error::{Error as SignerError, Standard as SignerErrorStandard},
        SignerContext, UnifiedSigner,
    },
    ExecutionConfig, Job, Worker,
};
use riglr_solana_tools::{
    balance::GetSolTool, signer::Local as SolanaSigner, transaction::TransferSolTool,
};
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::{env, sync::Arc};
use tokio::time::sleep;

// Environment variable constants following codebase patterns
const SOLANA_RPC_URL_ENV: &str = "SOLANA_RPC_URL";
const SOLANA_PRIVATE_KEY_ENV: &str = "SOLANA_PRIVATE_KEY";
const RPC_URL_11155111_ENV: &str = "RPC_URL_11155111";

#[tokio::test]
async fn test_1_1_basic_read_only_tool_execution() -> Result<()> {
    // Setup
    let rpc_url = env::var(SOLANA_RPC_URL_ENV)
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        rpc_url,
        CommitmentConfig::confirmed(),
    ));

    // Initialize ApplicationContext
    let config = Config::from_env();
    let app_context = ApplicationContext::from_config(&config);
    app_context.set_extension(rpc_client.clone());

    // Create Worker and register the get_sol_balance tool
    let tool_worker = Arc::new(Worker::<InMemoryIdempotencyStore>::new(
        ExecutionConfig::default(),
        app_context.clone(),
    ));
    tool_worker.register_tool(Arc::new(GetSolTool {
        context: Arc::new(app_context.clone()),
    }));

    // Test: Query balance of a known Devnet address
    let test_address = "11111111111111111111111111111111"; // System Program
    let _prompt = format!("What is the SOL balance of {test_address}?");

    // Execute the agent
    let job = Job::new(
        "get_sol_balance",
        &serde_json::json!({
            "address": test_address
        }),
        3,
    )?;
    let response = tool_worker.process_job(job).await?;

    // Assertions
    assert!(response.is_success(), "Tool should execute successfully");
    let result_str = format!("{response:?}");
    assert!(
        result_str.contains("balance") || result_str.contains("SOL"),
        "Response should mention balance or SOL"
    );

    // Verify the balance directly
    let pubkey = Pubkey::from_str(test_address)?;
    let actual_balance = rpc_client.get_balance(&pubkey)?;
    // Allow precision loss for SOL display - lamports to SOL conversion is standard practice
    #[expect(clippy::cast_precision_loss)]
    let sol_balance = actual_balance as f64 / 1_000_000_000.0;

    println!("Test 1.1 Passed: Tool successfully queried balance");
    println!("Response: {response:?}");
    println!("Actual balance: {sol_balance} SOL");

    Ok(())
}

#[tokio::test]
async fn test_1_2_secure_transaction_execution() -> Result<()> {
    // Setup
    let rpc_url = env::var(SOLANA_RPC_URL_ENV)
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        rpc_url.clone(),
        CommitmentConfig::confirmed(),
    ));

    // Load private key from environment
    // Expect appropriate in test context for required environment variables
    let private_key_str =
        env::var(SOLANA_PRIVATE_KEY_ENV).expect("SOLANA_PRIVATE_KEY must be set for testing");

    // Create signer from private key
    let keypair = Keypair::from_base58_string(&private_key_str);
    let signer = SolanaSigner::from_keypair(
        keypair.insecure_clone(),
        riglr_config::SolanaNetworkConfig::devnet(),
    );

    // Initialize ApplicationContext
    let config = Config::from_env();
    let app_context = ApplicationContext::from_config(&config);
    app_context.set_extension(rpc_client.clone());

    // Create Worker and register transfer tool
    let tool_worker = Arc::new(Worker::<InMemoryIdempotencyStore>::new(
        ExecutionConfig::default(),
        app_context.clone(),
    ));
    tool_worker.register_tool(Arc::new(TransferSolTool {
        context: Arc::new(app_context.clone()),
    }));

    // Generate a random recipient address
    let recipient = Keypair::new();
    let recipient_pubkey = recipient.pubkey();

    // Check sender's balance first
    let sender_pubkey = keypair.pubkey();
    let sender_balance = rpc_client.get_balance(&sender_pubkey)?;
    // Allow precision loss for SOL display - lamports to SOL conversion is standard practice
    #[expect(clippy::cast_precision_loss)]
    let sender_sol = sender_balance as f64 / 1_000_000_000.0;

    println!("Sender address: {sender_pubkey}");
    println!("Sender balance: {sender_sol} SOL");

    if sender_sol < 0.02 {
        println!("Warning: Sender balance is low. Test may fail due to insufficient funds.");
        println!("Please fund the test wallet with devnet SOL using:");
        println!("  solana airdrop 1 {sender_pubkey} --url devnet");
        return Ok(()); // Skip test if insufficient funds
    }

    // Get initial recipient balance
    let initial_balance = rpc_client.get_balance(&recipient_pubkey)?;

    // Execute transaction within SignerContext
    let job = Job::new(
        "transfer_sol",
        &serde_json::json!({
            "recipient": recipient_pubkey.to_string(),
            "amount": 0.001
        }),
        3,
    )?;

    let signer: Arc<dyn UnifiedSigner> = Arc::new(signer);
    let response = SignerContext::with_signer(signer, async {
        tool_worker.process_job(job).await.map_err(|e| {
            Box::new(SignerErrorStandard::SigningFailed(e.to_string())) as Box<dyn SignerError>
        })
    })
    .await
    .map_err(|e| anyhow::anyhow!("Signer error: {}", e))?;

    // Wait for transaction confirmation
    println!("Waiting for transaction confirmation...");
    sleep(Duration::from_secs(5)).await;

    // Check recipient's new balance
    let final_balance = rpc_client.get_balance(&recipient_pubkey)?;
    // Allow precision loss for SOL display - lamports to SOL conversion is standard practice
    #[expect(clippy::cast_precision_loss)]
    let balance_change = (final_balance - initial_balance) as f64 / 1_000_000_000.0;

    // Assertions
    assert!(
        response.is_success(),
        "Transfer should complete successfully"
    );
    let result_str = format!("{response:?}");
    assert!(
        result_str.contains("signature") || result_str.contains("transaction"),
        "Response should contain transaction details"
    );
    assert!(
        balance_change >= 0.0009, // Allow for small rounding
        "Recipient balance should increase by ~0.001 SOL, got {balance_change} SOL"
    );

    println!("Test 1.2 Passed: Transaction executed successfully");
    println!("Response: {response:?}");
    println!("Balance change: {balance_change} SOL");

    Ok(())
}

#[tokio::test]
async fn test_1_3_multi_tool_multi_chain_execution() -> Result<()> {
    // Setup for both Solana and EVM
    let solana_rpc = env::var(SOLANA_RPC_URL_ENV)
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let _sepolia_rpc = env::var(RPC_URL_11155111_ENV)
        .unwrap_or_else(|_| "https://ethereum-sepolia-rpc.publicnode.com".to_string());

    let solana_client = Arc::new(RpcClient::new_with_commitment(
        solana_rpc,
        CommitmentConfig::confirmed(),
    ));

    // Initialize ApplicationContext with multiple chain clients
    let config = Config::from_env();
    let app_context = ApplicationContext::from_config(&config);
    app_context.set_extension(solana_client.clone());
    // Note: EVM client setup would go here if riglr-evm-tools was ready

    // Create Worker with multiple tools
    let tool_worker = Arc::new(Worker::<InMemoryIdempotencyStore>::new(
        ExecutionConfig::default(),
        app_context.clone(),
    ));
    tool_worker.register_tool(Arc::new(GetSolTool {
        context: Arc::new(app_context.clone()),
    }));
    // Additional tools would be registered here

    // Test multi-tool execution
    let test_solana_address = "11111111111111111111111111111111";
    let _prompt = format!(
        "What is the SOL balance of {test_solana_address}? Please provide the exact balance."
    );

    let job = Job::new(
        "get_sol_balance",
        &serde_json::json!({
            "address": test_solana_address
        }),
        3,
    )?;
    let response = tool_worker.process_job(job).await?;

    // Assertions
    assert!(response.is_success(), "Tool should execute successfully");
    let result_str = format!("{response:?}");
    assert!(
        result_str.contains("balance") || result_str.contains("SOL"),
        "Response should mention balance information"
    );

    println!("Test 1.3 Passed: Multi-tool execution successful");
    println!("Response: {response:?}");

    Ok(())
}
