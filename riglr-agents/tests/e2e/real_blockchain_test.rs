//! THE GRAND UNIFIED TEST: Real blockchain integration for riglr-agents.
//!
//! This module contains the most critical test in the entire riglr ecosystem:
//! a real end-to-end test that validates the complete workflow from Task creation
//! through Agent execution to ACTUAL blockchain transactions and state changes.
//!
//! This test ACTUALLY:
//! - Starts a real solana-test-validator process
//! - Creates and funds real test wallets with SOL
//! - Uses real SignerContext with actual signers
//! - Executes real SOL transfers on the blockchain
//! - Verifies actual balance changes on-chain
//!
//! NO MOCKS. REAL BLOCKCHAIN. REAL TRANSACTIONS. REAL VERIFICATION.

use crate::common::*;
use async_trait::async_trait;
use riglr_agents::{registry::RegistryConfig, *};
use riglr_core::signer::SignerContext;
use riglr_solana_tools::signer::LocalSolanaSigner;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_system_interface::instruction as system_instruction;
use std::{str::FromStr, sync::Arc, time::Duration};
use tokio::time::sleep;
use tracing::{error, info};

/// A REAL blockchain-enabled agent that performs ACTUAL SOL transfers using SignerContext.
///
/// This agent demonstrates the complete integration between riglr-agents coordination
/// and riglr-core signer isolation. It converts Task parameters to REAL blockchain operations
/// while maintaining security through the SignerContext pattern.
#[derive(Debug)]
struct RealBlockchainTradingAgent {
    id: AgentId,
    unified_signer: Arc<dyn riglr_core::signer::UnifiedSigner>,
}

impl RealBlockchainTradingAgent {
    /// Create a new REAL blockchain trading agent with the provided unified signer.
    fn new(unified_signer: Arc<dyn riglr_core::signer::UnifiedSigner>) -> Self {
        Self {
            id: AgentId::generate(),
            unified_signer,
        }
    }

    /// Extract transfer parameters from task and validate them.
    fn extract_transfer_params(task: &Task) -> Result<(Pubkey, u64)> {
        let params = task.parameters.as_object().ok_or_else(|| {
            riglr_agents::error::AgentError::task_execution(
                "Task parameters must be a JSON object".to_string(),
            )
        })?;

        let to_address_str = params
            .get("to_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                riglr_agents::error::AgentError::task_execution(
                    "Missing required parameter: to_address".to_string(),
                )
            })?;

        let to_pubkey = Pubkey::from_str(to_address_str).map_err(|e| {
            riglr_agents::error::AgentError::task_execution(format!(
                "Invalid to_address format: {}",
                e
            ))
        })?;

        let amount_sol = params
            .get("amount")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| {
                riglr_agents::error::AgentError::task_execution(
                    "Missing or invalid parameter: amount".to_string(),
                )
            })?;

        if amount_sol <= 0.0 {
            return Err(riglr_agents::error::AgentError::task_execution(
                "Amount must be positive".to_string(),
            ));
        }

        let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;

        Ok((to_pubkey, amount_lamports))
    }
}

#[async_trait]
impl Agent for RealBlockchainTradingAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        info!(
            "RealBlockchainTradingAgent executing REAL blockchain task: {}",
            task.id
        );

        // Extract and validate parameters
        let (to_pubkey, amount_lamports) = match Self::extract_transfer_params(&task) {
            Ok(params) => params,
            Err(e) => {
                error!("Parameter extraction failed: {}", e);
                return Ok(TaskResult::failure(
                    format!("Parameter validation failed: {}", e),
                    true, // retriable in case parameters can be corrected
                    Duration::from_millis(10),
                ));
            }
        };

        info!(
            "Initiating REAL SOL transfer: {} lamports to {}",
            amount_lamports, to_pubkey
        );

        // Execute the REAL transfer within SignerContext for secure signer access
        let start_time = std::time::Instant::now();
        let unified_signer = self.unified_signer.clone();

        let transfer_result = SignerContext::with_signer(unified_signer, async move {
            // Get the Solana signer from context
            let signer = SignerContext::current_as_solana().await.map_err(|e| {
                riglr_core::signer::SignerError::Configuration(format!("No Solana signer: {}", e))
            })?;

            let from_pubkey = signer.pubkey().parse::<Pubkey>().map_err(|e| {
                riglr_core::signer::SignerError::Configuration(format!("Invalid pubkey: {}", e))
            })?;

            info!(
                "Creating REAL transfer instruction from {} to {}",
                from_pubkey, to_pubkey
            );

            // Create REAL system transfer instruction
            let instruction =
                system_instruction::transfer(&from_pubkey, &to_pubkey, amount_lamports);

            // Create REAL transaction
            let transaction = Transaction::new_with_payer(&[instruction], Some(&from_pubkey));

            info!("Signing and sending REAL transaction to blockchain...");

            // Serialize transaction to bytes for the signer
            use bincode::serialize;
            let mut tx_bytes = serialize(&transaction).map_err(|e| {
                riglr_core::signer::SignerError::Configuration(format!(
                    "Failed to serialize transaction: {}",
                    e
                ))
            })?;

            // Sign and send the REAL transaction using the signer
            let signature = signer
                .sign_and_send_transaction(&mut tx_bytes)
                .await
                .map_err(|e| {
                    riglr_core::signer::SignerError::Configuration(format!(
                        "Transaction failed: {}",
                        e
                    ))
                })?;

            info!("REAL SOL transfer completed with signature: {}", signature);

            Ok(signature)
        })
        .await;

        let execution_time = start_time.elapsed();

        // Convert result to TaskResult
        match transfer_result {
            Ok(signature) => {
                info!("REAL blockchain task completed successfully: {}", signature);
                Ok(TaskResult::success(
                    serde_json::json!({
                        "status": "real_transfer_completed",
                        "signature": signature,
                        "to_address": to_pubkey.to_string(),
                        "amount_lamports": amount_lamports,
                        "agent_id": self.id.as_str(),
                        "blockchain": "solana",
                        "real_transaction": true
                    }),
                    Some(format!("REAL SOL transfer completed: {}", signature)),
                    execution_time,
                ))
            }
            Err(e) => {
                error!("REAL blockchain transaction failed: {}", e);
                Ok(TaskResult::failure(
                    format!("REAL transfer failed: {}", e),
                    true, // SOL transfers can usually be retried
                    execution_time,
                ))
            }
        }
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<CapabilityType> {
        vec![
            CapabilityType::Custom("real_solana_transfer".to_string()),
            CapabilityType::Custom("real_blockchain_operations".to_string()),
            CapabilityType::Custom("real_trading".to_string()),
        ]
    }
}

/// THE GRAND UNIFIED TEST: Complete REAL Task → REAL Blockchain workflow validation.
///
/// This test validates the entire riglr-agents system with REAL blockchain operations:
/// 1. Starts REAL solana-test-validator process
/// 2. Creates and funds REAL test wallets with SOL
/// 3. Task creation with REAL blockchain operation parameters
/// 4. Agent registration and capability matching  
/// 5. Task dispatching through the routing system
/// 6. Agent execution with REAL SignerContext
/// 7. ACTUAL blockchain transaction execution on test validator
/// 8. REAL transaction confirmation and balance verification
/// 9. Complete workflow success validation with REAL state changes
///
/// This is the "Grand Unified Test" that proves the system works with REAL blockchain.
#[tokio::test]
async fn test_grand_unified_real_blockchain_transfer() {
    tracing_subscriber::fmt::try_init().ok();
    info!("🚀 Starting THE GRAND UNIFIED TEST: REAL end-to-end SOL transfer on blockchain");

    // Setup REAL blockchain test environment
    let harness = BlockchainTestHarness::new()
        .await
        .expect("Failed to create REAL blockchain test harness with solana-test-validator");

    info!("✅ REAL blockchain test harness initialized - solana-test-validator is running");

    // Get REAL funded test wallets
    let sender_keypair = harness
        .get_funded_keypair(0)
        .expect("Failed to get REAL sender keypair");
    let receiver_keypair = harness
        .get_funded_keypair(1)
        .expect("Failed to get REAL receiver keypair");

    info!(
        "✅ REAL test wallets obtained - Sender: {}, Receiver: {}",
        sender_keypair.pubkey(),
        receiver_keypair.pubkey()
    );

    // Record REAL initial balances for verification
    let initial_sender_balance = harness
        .get_balance(&sender_keypair.pubkey())
        .expect("Failed to get REAL initial sender balance");

    let initial_receiver_balance = harness
        .get_balance(&receiver_keypair.pubkey())
        .expect("Failed to get REAL initial receiver balance");

    info!(
        "✅ REAL initial balances - Sender: {} SOL, Receiver: {} SOL",
        initial_sender_balance as f64 / LAMPORTS_PER_SOL as f64,
        initial_receiver_balance as f64 / LAMPORTS_PER_SOL as f64
    );

    // Create REAL unified signer with sender's keypair
    let unified_signer = harness
        .create_unified_signer(0)
        .expect("Failed to create REAL unified signer");

    info!("✅ REAL SignerContext created with LocalSolanaSigner");

    // Create REAL blockchain-enabled agent
    let blockchain_agent = RealBlockchainTradingAgent::new(unified_signer);
    let agent_id = blockchain_agent.id().clone();

    info!(
        "✅ RealBlockchainTradingAgent created with ID: {}",
        agent_id
    );

    // Create agent system with registry and dispatcher
    let registry = LocalAgentRegistry::with_config(RegistryConfig {
        max_agents: Some(TEST_AGENT_CAPACITY),
        ..Default::default()
    });

    registry
        .register_agent(Arc::new(blockchain_agent))
        .await
        .expect("Failed to register REAL blockchain agent");

    info!("✅ REAL blockchain agent registered in system registry");

    let dispatcher = AgentDispatcher::new(Arc::new(registry));

    info!("✅ Agent dispatcher created and configured for REAL blockchain operations");

    // Create the REAL transfer task - this validates task parameter handling
    let transfer_amount_sol = 1.0;
    let transfer_task = Task::new(
        TaskType::Trading,
        serde_json::json!({
            "operation": "real_sol_transfer",
            "to_address": receiver_keypair.pubkey().to_string(),
            "amount": transfer_amount_sol,
            "description": "GRAND UNIFIED TEST - REAL SOL transfer on blockchain"
        }),
    )
    .with_priority(Priority::High);

    info!(
        "✅ REAL transfer task created: {} SOL to {}",
        transfer_amount_sol,
        receiver_keypair.pubkey()
    );

    // THE CRITICAL EXECUTION: Dispatch task through the complete system for REAL blockchain execution
    info!("🔥 Dispatching task through riglr-agents system for REAL blockchain execution...");
    let task_result = dispatcher
        .dispatch_task(transfer_task)
        .await
        .expect("REAL task dispatch failed");

    // Verify REAL task execution success
    assert!(
        task_result.is_success(),
        "REAL task should have succeeded: {:?}",
        task_result
    );

    let output_json = task_result
        .data()
        .unwrap()
        .as_object()
        .expect("REAL task output should be JSON object");

    let transaction_signature = output_json["signature"]
        .as_str()
        .expect("REAL task output should contain transaction signature");

    assert!(
        output_json["real_transaction"].as_bool().unwrap_or(false),
        "Task should indicate this was a REAL transaction"
    );

    info!(
        "✅ REAL task completed successfully with signature: {}",
        transaction_signature
    );

    // Wait for REAL transaction confirmation on the blockchain
    info!("⏳ Waiting for REAL transaction confirmation on blockchain...");
    sleep(Duration::from_secs(3)).await;

    // Verify REAL transaction exists on blockchain
    let client = harness.get_rpc_client();
    let signature = transaction_signature
        .parse()
        .expect("Invalid REAL transaction signature format");

    let signature_status = client
        .get_signature_status(&signature)
        .expect("Failed to get REAL signature status from blockchain")
        .expect("REAL transaction signature not found on blockchain");

    assert!(
        signature_status.is_ok(),
        "REAL transaction should be successful on blockchain: {:?}",
        signature_status
    );

    info!("✅ REAL transaction confirmed on blockchain successfully");

    // THE ULTIMATE VALIDATION: Verify ACTUAL balance changes on REAL blockchain
    let final_sender_balance = harness
        .get_balance(&sender_keypair.pubkey())
        .expect("Failed to get REAL final sender balance");

    let final_receiver_balance = harness
        .get_balance(&receiver_keypair.pubkey())
        .expect("Failed to get REAL final receiver balance");

    info!(
        "✅ REAL final balances - Sender: {} SOL, Receiver: {} SOL",
        final_sender_balance as f64 / LAMPORTS_PER_SOL as f64,
        final_receiver_balance as f64 / LAMPORTS_PER_SOL as f64
    );

    // Calculate and verify REAL balance changes
    let sender_balance_change = (initial_sender_balance as i64) - (final_sender_balance as i64);
    let receiver_balance_change =
        (final_receiver_balance as i64) - (initial_receiver_balance as i64);
    let expected_transfer_lamports = (transfer_amount_sol * LAMPORTS_PER_SOL as f64) as u64;

    info!(
        "✅ REAL balance changes - Sender: {} lamports, Receiver: {} lamports",
        sender_balance_change, receiver_balance_change
    );

    // Sender should have lost the transfer amount plus transaction fees
    assert!(
        sender_balance_change >= expected_transfer_lamports as i64,
        "REAL sender balance should decrease by at least transfer amount"
    );
    assert!(
        sender_balance_change <= (expected_transfer_lamports + 10000) as i64,
        "REAL sender balance change should not exceed transfer amount + reasonable fees"
    );

    // Receiver should have gained exactly the transfer amount
    assert_eq!(
        receiver_balance_change, expected_transfer_lamports as i64,
        "REAL receiver should gain exactly the transfer amount"
    );

    info!(
        "🎉 THE GRAND UNIFIED TEST PASSED! Complete REAL Task → REAL Blockchain workflow validated"
    );
    info!("   ✅ REAL solana-test-validator process: STARTED & RUNNING");
    info!("   ✅ REAL test wallets: CREATED & FUNDED");
    info!("   ✅ Task creation and dispatch: REAL");
    info!("   ✅ Agent capability matching: REAL");
    info!("   ✅ SignerContext integration: REAL");
    info!("   ✅ Blockchain transaction execution: REAL");
    info!("   ✅ Transaction confirmation: REAL");
    info!("   ✅ Balance change verification: REAL");
    info!("   ✅ End-to-end system validation: REAL BLOCKCHAIN");
    info!("🚀 riglr-agents PROVEN TO WORK WITH REAL BLOCKCHAIN!");
}

/// Test REAL error handling when attempting transfers with insufficient balance.
///
/// This validates that the system properly handles and reports REAL blockchain-level
/// errors while maintaining task execution consistency.
#[tokio::test]
async fn test_real_insufficient_balance_error_handling() {
    tracing_subscriber::fmt::try_init().ok();
    info!("Testing REAL insufficient balance error handling on blockchain");

    let harness = BlockchainTestHarness::new()
        .await
        .expect("Failed to create REAL blockchain test harness");

    // Create REAL unified signer with funded wallet
    let unified_signer = harness
        .create_unified_signer(0)
        .expect("Failed to create REAL unified signer");

    let receiver_keypair = harness
        .get_funded_keypair(1)
        .expect("Failed to get REAL receiver keypair");

    let blockchain_agent = RealBlockchainTradingAgent::new(unified_signer);

    // Create agent system
    let registry = LocalAgentRegistry::with_config(RegistryConfig {
        max_agents: Some(TEST_AGENT_CAPACITY),
        ..Default::default()
    });
    registry
        .register_agent(Arc::new(blockchain_agent))
        .await
        .expect("Failed to register agent");

    let dispatcher = AgentDispatcher::new(Arc::new(registry));

    // Attempt to transfer more SOL than available (wallets are funded with 10 SOL)
    let excessive_transfer_task = Task::new(
        TaskType::Trading,
        serde_json::json!({
            "operation": "real_sol_transfer",
            "to_address": receiver_keypair.pubkey().to_string(),
            "amount": 100.0, // More than the 10 SOL funding
            "description": "REAL insufficient balance test transfer"
        }),
    )
    .with_priority(Priority::High);

    info!("Attempting REAL transfer of 100 SOL (should fail with REAL blockchain error)");
    let task_result = dispatcher
        .dispatch_task(excessive_transfer_task)
        .await
        .expect("Task dispatch should succeed even if REAL execution fails");

    // Verify task failed with appropriate REAL error
    assert!(
        !task_result.is_success(),
        "REAL task should fail due to insufficient balance on blockchain"
    );
    assert!(
        task_result.is_retriable(),
        "REAL insufficient balance errors should be retriable"
    );

    let error_message = task_result.error().unwrap_or("unknown error");

    assert!(
        error_message.to_lowercase().contains("insufficient")
            || error_message.to_lowercase().contains("balance")
            || error_message.to_lowercase().contains("funds"),
        "REAL error message should indicate insufficient balance: {}",
        error_message
    );

    info!(
        "✅ REAL insufficient balance error handling validated: {}",
        error_message
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_transfer_params_valid() {
        let task = Task::new(
            TaskType::Trading,
            serde_json::json!({
                "to_address": "11111111111111111111111111111112", // System program ID
                "amount": 1.5
            }),
        );

        let (pubkey, lamports) = RealBlockchainTradingAgent::extract_transfer_params(&task)
            .expect("Should extract valid parameters");

        assert_eq!(pubkey.to_string(), "11111111111111111111111111111112");
        assert_eq!(lamports, (1.5 * LAMPORTS_PER_SOL as f64) as u64);
    }

    #[test]
    fn test_extract_transfer_params_invalid_address() {
        let task = Task::new(
            TaskType::Trading,
            serde_json::json!({
                "to_address": "invalid_address",
                "amount": 1.0
            }),
        );

        let result = RealBlockchainTradingAgent::extract_transfer_params(&task);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid to_address format"));
    }

    #[test]
    fn test_extract_transfer_params_negative_amount() {
        let task = Task::new(
            TaskType::Trading,
            serde_json::json!({
                "to_address": "11111111111111111111111111111112",
                "amount": -1.0
            }),
        );

        let result = RealBlockchainTradingAgent::extract_transfer_params(&task);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Amount must be positive"));
    }

    #[tokio::test]
    async fn test_real_blockchain_agent_capabilities() {
        let keypair = Keypair::new();
        let signer =
            LocalSolanaSigner::from_keypair_with_url(keypair, "http://localhost:8899".to_string());
        let unified_signer: Arc<dyn riglr_core::signer::UnifiedSigner> = Arc::new(signer);

        let agent = RealBlockchainTradingAgent::new(unified_signer);

        let capabilities = agent.capabilities();
        assert!(capabilities.contains(&CapabilityType::Custom("real_solana_transfer".to_string())));
        assert!(capabilities.contains(&CapabilityType::Custom(
            "real_blockchain_operations".to_string()
        )));
        assert!(capabilities.contains(&CapabilityType::Custom("real_trading".to_string())));

        // Test capability matching
        let trading_task = Task::new(TaskType::Trading, serde_json::json!({}));
        assert!(agent.can_handle(&trading_task));

        let research_task = Task::new(TaskType::Research, serde_json::json!({}));
        assert!(!agent.can_handle(&research_task));
    }
}
