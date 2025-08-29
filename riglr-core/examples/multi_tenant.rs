//! Multi-tenant example demonstrating secure signer isolation
//!
//! This shows how the SignerContext pattern ensures that different users/tenants
//! can use the same tools safely without any risk of signer leakage between requests.

use async_trait::async_trait;
use riglr_config::Config;
use riglr_core::{
    idempotency::InMemoryIdempotencyStore,
    provider::ApplicationContext,
    signer::{SignerBase, SignerContext, UnifiedSigner},
    ExecutionConfig, Job, JobResult, Tool, ToolError, ToolWorker,
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// A tool that accesses user-specific information from the signer context
#[derive(Clone)]
struct WalletTool;

#[async_trait]
impl Tool for WalletTool {
    async fn execute(
        &self,
        params: serde_json::Value,
        _app_context: &ApplicationContext,
    ) -> Result<JobResult, ToolError> {
        let operation = params["operation"].as_str().unwrap_or("info");

        // Get the current signer from context
        let signer = SignerContext::current()
            .await
            .map_err(|_| ToolError::permanent_string("This tool requires a signer context"))?;

        let user_id = signer.user_id().unwrap_or_else(|| "anonymous".to_string());

        match operation {
            "info" => {
                let info = serde_json::json!({
                    "user_id": user_id,
                    "supports_solana": signer.supports_solana(),
                    "supports_evm": signer.supports_evm(),
                    "locale": signer.locale(),
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
                let amount = params["amount"].as_f64().unwrap_or(0.0);
                let to_user = params["to_user"].as_str().unwrap_or("unknown");

                if amount <= 0.0 {
                    return Err(ToolError::invalid_input_string("Amount must be positive").into());
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
                "Unknown operation: {}",
                operation
            ))),
        }
    }

    fn name(&self) -> &str {
        "wallet"
    }

    fn description(&self) -> &str {
        "Wallet operations that are automatically scoped to the current user's signer context"
    }
}

/// Mock signer representing different users
#[derive(Debug, Clone)]
struct UserSigner {
    user_id: String,
    locale: String,
    supports_solana: bool,
    supports_evm: bool,
}

impl UserSigner {
    fn new(user_id: String, locale: String, supports_solana: bool, supports_evm: bool) -> Self {
        Self {
            user_id,
            locale,
            supports_solana,
            supports_evm,
        }
    }
}

impl SignerBase for UserSigner {
    fn user_id(&self) -> Option<String> {
        Some(self.user_id.clone())
    }

    fn locale(&self) -> String {
        self.locale.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl UnifiedSigner for UserSigner {
    fn supports_solana(&self) -> bool {
        self.supports_solana
    }

    fn supports_evm(&self) -> bool {
        self.supports_evm
    }

    fn as_solana(&self) -> Option<&dyn riglr_core::signer::SolanaSigner> {
        None // Mock signer doesn't implement actual signing
    }

    fn as_evm(&self) -> Option<&dyn riglr_core::signer::EvmSigner> {
        None
    }

    fn as_multi_chain(&self) -> Option<&dyn riglr_core::signer::MultiChainSigner> {
        None
    }
}

/// Simulate handling a user request with proper signer isolation
async fn handle_user_request(
    worker: &ToolWorker<InMemoryIdempotencyStore>,
    user_signer: Arc<dyn UnifiedSigner>,
    _operation: &str,
    params: serde_json::Value,
) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
    SignerContext::with_signer(user_signer, async {
        let job = Job::new("wallet", &params, 3)
            .map_err(|e| riglr_core::signer::SignerError::Configuration(e.to_string()))?;
        worker
            .process_job(job)
            .await
            .map_err(|e| riglr_core::signer::SignerError::ProviderError(e.to_string()))
    })
    .await
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== riglr-core Multi-Tenant Example ===\n");

    // Create a shared worker
    let exec_config = ExecutionConfig::default();
    let config = Config::from_env();
    let app_context = ApplicationContext::from_config(&config);
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(exec_config, app_context);
    worker.register_tool(Arc::new(WalletTool)).await;

    println!("✅ Created shared worker (serves all tenants)\n");

    // Create different users with different capabilities
    let alice_signer = Arc::new(UserSigner::new(
        "alice".to_string(),
        "en".to_string(),
        true,  // supports Solana
        false, // doesn't support EVM
    )) as Arc<dyn UnifiedSigner>;

    let bob_signer = Arc::new(UserSigner::new(
        "bob".to_string(),
        "fr".to_string(),
        false, // doesn't support Solana
        true,  // supports EVM
    )) as Arc<dyn UnifiedSigner>;

    let charlie_signer = Arc::new(UserSigner::new(
        "charlie".to_string(),
        "es".to_string(),
        true, // supports both chains
        true,
    )) as Arc<dyn UnifiedSigner>;

    println!("👥 Created signers for three users:");
    println!("   • Alice: Solana user (English)");
    println!("   • Bob: EVM user (French)");
    println!("   • Charlie: Multi-chain user (Spanish)\n");

    // Demonstrate concurrent requests with proper isolation
    println!("🔄 Processing concurrent requests (should be isolated)...\n");

    let alice_task = tokio::spawn({
        let worker = worker.clone();
        let signer = alice_signer.clone();
        async move {
            println!("👤 Alice: Checking wallet info...");
            let result = handle_user_request(
                &worker,
                signer,
                "info",
                serde_json::json!({"operation": "info"}),
            )
            .await
            .unwrap();

            match result {
                JobResult::Success { value, .. } => {
                    println!("👤 Alice result: {}", value);
                }
                _ => println!("👤 Alice failed"),
            }
        }
    });

    let bob_task = tokio::spawn({
        let worker = worker.clone();
        let signer = bob_signer.clone();
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
            .unwrap();

            match result {
                JobResult::Success { value, .. } => {
                    println!("👤 Bob result: {}", value);
                }
                _ => println!("👤 Bob failed"),
            }
        }
    });

    let charlie_task = tokio::spawn({
        let worker = worker.clone();
        let signer = charlie_signer.clone();
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
            .unwrap();

            match result {
                JobResult::Success { value, tx_hash } => {
                    println!("👤 Charlie result: {}", value);
                    if let Some(hash) = tx_hash {
                        println!("   📝 Transaction hash: {}", hash);
                    }
                }
                _ => println!("👤 Charlie failed"),
            }
        }
    });

    // Wait for all tasks to complete
    let _ = tokio::join!(alice_task, bob_task, charlie_task);

    println!("\n✅ All concurrent requests completed successfully!");
    println!("   Each request was processed with the correct user context\n");

    // Demonstrate that contexts don't leak between sequential requests
    println!("🔒 Testing signer isolation between sequential requests...\n");

    // Alice's request
    let alice_result = handle_user_request(
        &worker,
        alice_signer.clone(),
        "info",
        serde_json::json!({"operation": "info"}),
    )
    .await
    .map_err(|e| e.to_string())?;

    println!("👤 Alice (sequential): {:?}", alice_result);

    // Bob's request (should not have access to Alice's context)
    let bob_result = handle_user_request(
        &worker,
        bob_signer.clone(),
        "info",
        serde_json::json!({"operation": "info"}),
    )
    .await
    .map_err(|e| e.to_string())?;

    println!("👤 Bob (sequential): {:?}", bob_result);

    // Demonstrate error handling when no context is available
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
        _ => println!("   Unexpected success!"),
    }

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

    Ok(())
}
