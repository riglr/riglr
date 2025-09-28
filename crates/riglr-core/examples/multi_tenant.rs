//! Multi-tenant example demonstrating secure signer isolation
//!
//! This shows how the `SignerContext` pattern ensures that different users/tenants
//! can use the same tools safely without any risk of signer leakage between requests.

use async_trait::async_trait;
use core::error::Error;
use riglr_config::Config;
use riglr_core::{
    idempotency::InMemoryIdempotencyStore,
    provider::ApplicationContext,
    signer::{
        error::Standard, Chain, EvmSigner, SignerBase, SignerContext, SignerError, SolanaSigner,
        UnifiedSigner,
    },
    tool::Worker,
    ExecutionConfig, Job, JobResult, Tool, ToolError,
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// A tool that accesses user-specific information from the signer context
#[derive(Clone)]
struct WalletTool;

#[async_trait]
impl Tool for WalletTool {
    type Args = serde_json::Value;
    type Output = JobResult;
    type Error = ToolError;

    fn name(&self) -> &'static str {
        "wallet"
    }

    fn description(&self) -> &'static str {
        "Wallet operations that are automatically scoped to the current user's signer context"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "amount": {"type": "number"},
                "to_user": {"type": "string"}
            }
        })
    }

    async fn call(&self, params: Self::Args) -> Result<Self::Output, Self::Error> {
        let operation = params
            .get("operation")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("info");

        // Get the current signer from context
        let signer = SignerContext::current()
            .map_err(|_| ToolError::permanent_string("This tool requires a signer context"))?;

        let user_id = signer.user_id();

        match operation {
            "info" => {
                let info = serde_json::json!({
                    "user_id": user_id,
                    "supports_solana": signer.supports_solana(),
                    "supports_evm": signer.supports_evm(),
                });
                Ok(JobResult::success(&info)
                    .map_err(|e| ToolError::permanent_string(e.to_string()))?)
            }
            "balance" => {
                // Simulate checking balance for the user
                let balance = match user_id.as_str() {
                    "alice" => 1.5,
                    "bob" => 2.3,
                    "charlie" => 0.8,
                    _ => 0.0,
                };

                let result = serde_json::json!({
                    "user_id": user_id,
                    "balance": balance,
                    "currency": if signer.supports_solana() { "SOL" } else { "ETH" }
                });
                Ok(JobResult::success(&result)
                    .map_err(|e| ToolError::permanent_string(e.to_string()))?)
            }
            "transfer" => {
                let amount = params
                    .get("amount")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0);
                let to_user = params
                    .get("to_user")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown");

                if amount <= 0.0 {
                    return Err(ToolError::invalid_input_string("Amount must be positive"));
                }

                // Simulate a transfer operation
                let tx_hash = format!(
                    "tx_{}_to_{}_{}",
                    user_id,
                    to_user,
                    chrono::Utc::now().timestamp()
                );

                let result = serde_json::json!({
                    "from": user_id,
                    "to": to_user,
                    "amount": amount,
                    "transaction_hash": tx_hash
                });

                Ok(JobResult::success_with_tx(&result, &tx_hash)
                    .map_err(|e| ToolError::permanent_string(e.to_string()))?)
            }
            _ => Err(ToolError::invalid_input_string(format!(
                "Unknown operation: {operation}"
            ))),
        }
    }
}

/// Mock signer representing different users
#[derive(Debug, Clone)]
struct UserSigner {
    user_id: String,
    supports_solana: bool,
    supports_evm: bool,
}

impl UserSigner {
    const fn new(user_id: String, supports_solana: bool, supports_evm: bool) -> Self {
        Self {
            user_id,
            supports_solana,
            supports_evm,
        }
    }
}

impl SignerBase for UserSigner {
    fn supported_chains(&self) -> &[Chain] {
        // Return supported chains based on the signer's capabilities
        if self.supports_solana && self.supports_evm {
            &[Chain::Solana, Chain::Evm]
        } else if self.supports_solana {
            &[Chain::Solana]
        } else if self.supports_evm {
            &[Chain::Evm]
        } else {
            &[]
        }
    }

    fn user_id(&self) -> String {
        self.user_id.clone()
    }
}

impl UnifiedSigner for UserSigner {
    fn supports_solana(&self) -> bool {
        self.supports_solana
    }

    fn supports_evm(&self) -> bool {
        self.supports_evm
    }

    fn as_solana(&self) -> Option<&dyn SolanaSigner> {
        None // Mock signer doesn't implement actual signing
    }

    fn as_evm(&self) -> Option<&dyn EvmSigner> {
        None
    }

    // Note: as_multi_chain method not available in current UnifiedSigner trait
}

/// Simulate handling a user request with proper signer isolation
async fn handle_user_request(
    worker: &Worker<InMemoryIdempotencyStore>,
    user_signer: Arc<dyn UnifiedSigner>,
    _operation: &str,
    params: serde_json::Value,
) -> Result<JobResult, Box<dyn Error + Send + Sync>> {
    SignerContext::with_signer(user_signer, async {
        let job = Job::new("wallet", &params, 3).map_err(|e| {
            Box::new(Standard::Configuration(e.to_string())) as Box<dyn SignerError>
        })?;
        worker
            .process_job(job)
            .await
            .map_err(|e| Box::new(Standard::Generic(e.to_string())) as Box<dyn SignerError>)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Signer error: {}", e).into())
}

/// Set up the worker and register tools
fn setup_worker_and_tools() -> Worker<InMemoryIdempotencyStore> {
    let exec_config = ExecutionConfig::default();
    let config = Config::from_env();
    let app_context = ApplicationContext::from_config(&config);
    let worker = Worker::<InMemoryIdempotencyStore>::new(exec_config, app_context);
    worker.register_tool(Arc::new(WalletTool));
    println!("✅ Created shared worker (serves all tenants)\n");
    worker
}

/// Create user signers with different capabilities
fn create_user_signers() -> (
    Arc<dyn UnifiedSigner>,
    Arc<dyn UnifiedSigner>,
    Arc<dyn UnifiedSigner>,
) {
    let alice_signer = Arc::new(UserSigner::new(
        "alice".to_string(),
        true,  // supports Solana
        false, // doesn't support EVM
    )) as Arc<dyn UnifiedSigner>;

    let bob_signer = Arc::new(UserSigner::new(
        "bob".to_string(),
        false, // doesn't support Solana
        true,  // supports EVM
    )) as Arc<dyn UnifiedSigner>;

    let charlie_signer = Arc::new(UserSigner::new(
        "charlie".to_string(),
        true, // supports both chains
        true,
    )) as Arc<dyn UnifiedSigner>;

    println!("👥 Created signers for three users:");
    println!("   • Alice: Solana user");
    println!("   • Bob: EVM user");
    println!("   • Charlie: Multi-chain user\n");

    (alice_signer, bob_signer, charlie_signer)
}

/// Demonstrate concurrent requests with proper isolation
async fn run_concurrent_requests_demo(
    worker: &Worker<InMemoryIdempotencyStore>,
    alice_signer: Arc<dyn UnifiedSigner>,
    bob_signer: Arc<dyn UnifiedSigner>,
    charlie_signer: Arc<dyn UnifiedSigner>,
) {
    println!("🔄 Processing concurrent requests (should be isolated)...\n");

    let alice_task = tokio::spawn({
        let worker = worker.clone();
        let signer = alice_signer.clone();
        #[expect(clippy::expect_used)]
        async move {
            println!("👤 Alice: Checking wallet info...");
            let result = handle_user_request(
                &worker,
                signer,
                "info",
                serde_json::json!({"operation": "info"}),
            )
            .await
            .expect("Alice's info request should succeed in this example");

            match result {
                JobResult::Success { value, .. } => {
                    println!("👤 Alice result: {value}");
                }
                JobResult::Failure { .. } => println!("👤 Alice failed"),
                _ => println!("👤 Alice: Unexpected result type"),
            }
        }
    });

    let bob_task = tokio::spawn({
        let worker = worker.clone();
        let signer = bob_signer.clone();
        #[expect(clippy::expect_used)]
        async move {
            // Add a small delay to show concurrent execution
            sleep(Duration::from_millis(10)).await;

            println!("👤 Bob: Checking balance...");
            let result = handle_user_request(
                &worker,
                signer,
                "balance",
                serde_json::json!({"operation": "balance"}),
            )
            .await
            .expect("Bob's balance request should succeed in this example");

            match result {
                JobResult::Success { value, .. } => {
                    println!("👤 Bob result: {value}");
                }
                JobResult::Failure { .. } => println!("👤 Bob failed"),
                _ => println!("👤 Bob: Unexpected result type"),
            }
        }
    });

    let charlie_task = tokio::spawn({
        let worker = worker.clone();
        let signer = charlie_signer.clone();
        #[expect(clippy::expect_used)]
        async move {
            // Add a different delay
            sleep(Duration::from_millis(20)).await;

            println!("👤 Charlie: Making a transfer...");
            let result = handle_user_request(
                &worker,
                signer,
                "transfer",
                serde_json::json!({
                    "operation": "transfer",
                    "amount": 0.5,
                    "to_user": "alice"
                }),
            )
            .await
            .expect("Charlie's transfer request should succeed in this example");

            match result {
                JobResult::Success { value, tx_hash } => {
                    println!("👤 Charlie result: {value}");
                    if let Some(hash) = tx_hash {
                        println!("   📝 Transaction hash: {hash}");
                    }
                }
                JobResult::Failure { .. } => println!("👤 Charlie failed"),
                _ => println!("👤 Charlie: Unexpected result type"),
            }
        }
    });

    // Wait for all tasks to complete
    let _ = tokio::join!(alice_task, bob_task, charlie_task);

    println!("\n✅ All concurrent requests completed successfully!");
    println!("   Each request was processed with the correct user context\n");
}

/// Demonstrate that contexts don't leak between sequential requests
async fn run_sequential_requests_demo(
    worker: &Worker<InMemoryIdempotencyStore>,
    alice_signer: Arc<dyn UnifiedSigner>,
    bob_signer: Arc<dyn UnifiedSigner>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("🔒 Testing signer isolation between sequential requests...\n");

    // Alice's request
    let alice_result = handle_user_request(
        worker,
        alice_signer.clone(),
        "info",
        serde_json::json!({"operation": "info"}),
    )
    .await
    .map_err(|e| e.to_string())?;

    println!("👤 Alice (sequential): {alice_result:?}");

    // Bob's request (should not have access to Alice's context)
    let bob_result = handle_user_request(
        worker,
        bob_signer.clone(),
        "info",
        serde_json::json!({"operation": "info"}),
    )
    .await
    .map_err(|e| e.to_string())?;

    println!("👤 Bob (sequential): {bob_result:?}");

    Ok(())
}

/// Test error handling when no signer context is available
async fn test_no_context_error(
    worker: &Worker<InMemoryIdempotencyStore>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n❌ Testing error when no signer context is available...");
    let job_no_context = Job::new("wallet", &serde_json::json!({"operation": "info"}), 1)?;

    let no_context_result = worker.process_job(job_no_context).await?;
    match no_context_result {
        JobResult::Failure { ref error } => {
            println!(
                "   Expected error: {} (retriable: {})",
                error,
                no_context_result.is_retriable()
            );
        }
        JobResult::Success { .. } => println!("   Unexpected success!"),
        _ => println!("   Unexpected result type"),
    }

    Ok(())
}

/// Print example completion summary
fn print_summary() {
    println!("\n🎉 Multi-tenant example completed!");
    println!("\n🔒 Security features demonstrated:");
    println!("   • Complete signer isolation between different users");
    println!("   • No context leakage in concurrent processing");
    println!("   • Proper error handling when no context is available");
    println!("   • Thread-safe multi-tenant operations");
    println!("\n🏗️ Architecture benefits:");
    println!("   • One worker can serve multiple tenants safely");
    println!("   • Tools automatically get the right user context");
    println!("   • No risk of accidentally accessing another user's data");
    println!("   • Clean separation of concerns");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("=== riglr-core Multi-Tenant Example ===\n");

    // Set up the worker and tools
    let worker = setup_worker_and_tools();

    // Create user signers
    let (alice_signer, bob_signer, charlie_signer) = create_user_signers();

    // Run concurrent requests demonstration
    run_concurrent_requests_demo(
        &worker,
        alice_signer.clone(),
        bob_signer.clone(),
        charlie_signer,
    )
    .await;

    // Run sequential requests demonstration
    run_sequential_requests_demo(&worker, alice_signer, bob_signer).await?;

    // Test error handling when no context is available
    test_no_context_error(&worker).await?;

    // Print completion summary
    print_summary();

    Ok(())
}
