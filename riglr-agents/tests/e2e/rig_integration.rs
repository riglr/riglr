//! End-to-end rig-core integration tests for riglr-agents.
//!
//! This module tests the integration between riglr-agents coordination system
//! and rig-core LLM-powered agents. It validates that TradingAgents can use
//! rig::Agent brains for intelligent decision-making while maintaining the
//! multi-agent coordination and SignerContext security model.
//!
//! These tests demonstrate:
//! - Natural language task processing with LLM decision-making
//! - AI-guided parameter extraction and blockchain tool selection
//! - Integration between LLM responses and riglr-solana-tools execution
//! - Mock LLM providers for deterministic testing
//! - Complete AI → Agent → SignerContext → Blockchain workflows

use crate::common::{rig_mocks::MockRigAgent, *};
use async_trait::async_trait;
use riglr_agents::*;
use riglr_core::{
    jobs::{Job, JobResult},
    signer::{SignerContext, UnifiedSigner},
};
use riglr_solana_tools::{balance::get_sol_balance, signer::LocalSolanaSigner};
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::{str::FromStr, sync::Arc, time::Duration};
use tracing::{debug, error, info, warn};

/// AI decision structure returned by the mock LLM.
///
/// This represents the structured output that an LLM would provide
/// when analyzing a natural language task and determining appropriate actions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AIDecision {
    /// The action to take (e.g., "transfer_sol", "get_balance", "token_swap")
    pub action: String,
    /// AI reasoning for why this action was chosen
    pub reasoning: String,
    /// Extracted or inferred parameters for the action
    pub parameters: serde_json::Value,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
}

/// A trading agent that uses rig-core for intelligent decision-making.
///
/// This agent demonstrates the complete integration between riglr-agents
/// coordination and rig-core LLM capabilities. It processes natural language
/// tasks, makes intelligent decisions using AI, and executes blockchain
/// operations through SignerContext.
#[derive(Debug)]
pub struct IntelligentTradingAgent {
    id: AgentId,
    unified_signer: Arc<dyn UnifiedSigner>,
    /// Mock rig-core agent simulation (in real implementation would be rig_core::Agent)
    rig_agent: MockRigAgent,
}

impl IntelligentTradingAgent {
    /// Create a new intelligent trading agent with rig-core integration.
    pub fn new(unified_signer: Arc<dyn UnifiedSigner>) -> Self {
        Self {
            id: AgentId::generate(),
            unified_signer,
            rig_agent: crate::common::rig_mocks::MockOpenAIProvider::new("test-key")
                .agent("gpt-4")
                .build(),
        }
    }

    /// Process a natural language task using the AI brain.
    ///
    /// This simulates what would happen with real rig-core integration:
    /// 1. Convert task to LLM prompt
    /// 2. Get AI decision and reasoning
    /// 3. Extract parameters and validate
    /// 4. Return structured decision
    async fn process_with_ai(&self, task: &Task) -> std::result::Result<AIDecision, String> {
        let prompt = self.create_llm_prompt(task);
        debug!("Generated LLM prompt: {}", prompt);

        // Simulate rig-core agent completion
        let ai_response = self
            .rig_agent
            .prompt(&prompt)
            .await
            .map_err(|e| format!("LLM completion failed: {}", e))?;

        debug!("AI response received: {}", ai_response);

        // Parse AI response into structured decision
        self.parse_ai_response(&ai_response, task)
    }

    /// Create an LLM prompt from the task metadata.
    fn create_llm_prompt(&self, task: &Task) -> String {
        // Get task description from metadata or create a default one
        let description = task
            .metadata
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("No description provided");

        format!(
            r#"You are an intelligent blockchain trading agent. Analyze the following task and determine the appropriate action.

Task Description: {}
Task Parameters: {}
Task Type: {}
Priority: {:?}

Available Actions:
- transfer_sol: Transfer SOL between wallets (requires: to_address, amount)
- get_balance: Check wallet SOL balance (requires: address)
- token_swap: Swap tokens using DEX protocols (requires: from_token, to_token, amount)
- market_analysis: Analyze market conditions (requires: symbol)
- risk_assessment: Assess trade risk (requires: trade_details)

Analyze the task and respond with a JSON object containing:
{{
    "action": "the_action_to_take",
    "reasoning": "why you chose this action",
    "parameters": {{"extracted": "parameters"}},
    "confidence": 0.95
}}

Focus on extracting clear, actionable parameters from the natural language description."#,
            description, task.parameters, task.task_type, task.priority
        )
    }

    /// Parse the AI response into a structured decision.
    fn parse_ai_response(
        &self,
        _response: &str,
        task: &Task,
    ) -> std::result::Result<AIDecision, String> {
        // In real rig-core integration, this would parse actual LLM JSON responses
        // For testing, we simulate intelligent responses based on task content

        let description = task
            .metadata
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let description_lower = description.to_lowercase();
        let params = &task.parameters;

        if description_lower.contains("transfer")
            || description_lower.contains("send")
            || description_lower.contains("payment")
        {
            let amount = self
                .extract_amount_from_description(description)
                .or_else(|| params.get("amount").and_then(|v| v.as_f64()))
                .unwrap_or(1.0);

            let to_address = self
                .extract_address_from_description(description)
                .or_else(|| {
                    params
                        .get("to_address")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "11111111111111111111111111111112".to_string()); // Default to system program

            Ok(AIDecision {
                action: "transfer_sol".to_string(),
                reasoning:
                    "Task requests a SOL transfer operation based on natural language analysis"
                        .to_string(),
                parameters: serde_json::json!({
                    "to_address": to_address,
                    "amount": amount
                }),
                confidence: 0.9,
            })
        } else if description_lower.contains("balance")
            || description_lower.contains("check")
            || description_lower.contains("wallet")
        {
            let address = self
                .extract_address_from_description(description)
                .or_else(|| {
                    params
                        .get("address")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "11111111111111111111111111111112".to_string());

            Ok(AIDecision {
                action: "get_balance".to_string(),
                reasoning: "Task requests balance information based on natural language analysis"
                    .to_string(),
                parameters: serde_json::json!({
                    "address": address
                }),
                confidence: 0.95,
            })
        } else if description_lower.contains("swap")
            || description_lower.contains("exchange")
            || description_lower.contains("trade")
        {
            Ok(AIDecision {
                action: "token_swap".to_string(),
                reasoning: "Task requests token swap operation based on natural language analysis"
                    .to_string(),
                parameters: serde_json::json!({
                    "from_token": "SOL",
                    "to_token": "USDC",
                    "amount": 1.0
                }),
                confidence: 0.8,
            })
        } else {
            Ok(AIDecision {
                action: "unknown".to_string(),
                reasoning: "Could not determine appropriate action from task description"
                    .to_string(),
                parameters: serde_json::json!({}),
                confidence: 0.1,
            })
        }
    }

    /// Extract amount from natural language description.
    fn extract_amount_from_description(&self, description: &str) -> Option<f64> {
        // Simple pattern matching for SOL amounts (avoiding regex dependency for tests)
        let description_lower = description.to_lowercase();
        let words: Vec<&str> = description_lower.split_whitespace().collect();

        for (i, word) in words.iter().enumerate() {
            if word.contains("sol") && i > 0 {
                // Look for number before "sol"
                if let Ok(amount) = words[i - 1].parse::<f64>() {
                    return Some(amount);
                }
            } else if let Ok(amount) = word.parse::<f64>() {
                // Look for "SOL" after number
                if i + 1 < words.len() && words[i + 1].to_lowercase().contains("sol") {
                    return Some(amount);
                }
            }
        }
        None
    }

    /// Extract wallet address from natural language description.
    fn extract_address_from_description(&self, description: &str) -> Option<String> {
        // Look for base58-encoded addresses (simplified pattern matching)
        let words: Vec<&str> = description.split_whitespace().collect();

        for word in words {
            // Check if word looks like a Solana address (32-44 chars, base58)
            if word.len() >= 32 && word.len() <= 44 {
                // Simple check for base58 characters
                let is_base58 = word.chars().all(|c| {
                    matches!(c, '1'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z')
                });
                if is_base58 {
                    return Some(word.to_string());
                }
            }
        }
        None
    }

    /// Execute a SOL transfer based on AI decision.
    async fn execute_sol_transfer(
        &self,
        decision: &AIDecision,
    ) -> riglr_agents::Result<TaskResult> {
        let params = &decision.parameters;

        let to_address_str = params
            .get("to_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                riglr_agents::error::AgentError::task_execution(
                    "AI decision missing to_address parameter",
                )
            })?;

        let amount_sol = params
            .get("amount")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| {
                riglr_agents::error::AgentError::task_execution(
                    "AI decision missing amount parameter",
                )
            })?;

        let to_pubkey = Pubkey::from_str(to_address_str).map_err(|e| {
            riglr_agents::error::AgentError::task_execution(format!(
                "Invalid address from AI: {}",
                e
            ))
        })?;

        let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;

        info!(
            "Executing AI-guided SOL transfer: {} SOL to {}",
            amount_sol, to_pubkey
        );

        let start_time = std::time::Instant::now();
        let _job = Job::new(
            format!("ai_sol_transfer_{}", uuid::Uuid::new_v4()),
            &serde_json::json!({
                "ai_decision": decision,
                "to_address": to_address_str,
                "amount_lamports": amount_lamports
            }),
            3,
        )
        .map_err(|e| {
            riglr_agents::error::AgentError::task_execution(format!("Failed to create job: {}", e))
        })?;

        let job_result = SignerContext::with_signer(self.unified_signer.clone(), async {
            let signer = SignerContext::current_as_solana().await?;

            let mut tx = solana_sdk::transaction::Transaction::default();
            let signature = signer
                .sign_and_send_transaction(&mut tx)
                .await
                .map_err(|e| {
                    riglr_core::signer::SignerError::Configuration(format!(
                        "Transfer failed: {}",
                        e
                    ))
                })?;

            Ok(JobResult::Success {
                value: serde_json::json!({
                    "signature": signature,
                    "ai_action": "transfer_sol",
                    "ai_confidence": decision.confidence
                }),
                tx_hash: Some(signature),
            })
        })
        .await;

        let execution_time = start_time.elapsed();

        match job_result {
            Ok(JobResult::Success { value, tx_hash }) => {
                Ok(TaskResult::success(value, tx_hash, execution_time))
            }
            Ok(JobResult::Failure { error }) => Ok(TaskResult::failure(
                format!("AI-guided transfer failed: {}", error),
                true,
                execution_time,
            )),
            Err(e) => Ok(TaskResult::failure(
                format!("SignerContext execution failed: {}", e),
                false,
                execution_time,
            )),
        }
    }

    /// Execute a balance check based on AI decision.
    async fn execute_balance_check(
        &self,
        decision: &AIDecision,
    ) -> riglr_agents::Result<TaskResult> {
        let params = &decision.parameters;

        let address_str = params
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                riglr_agents::error::AgentError::task_execution(
                    "AI decision missing address parameter",
                )
            })?;

        let pubkey = Pubkey::from_str(address_str).map_err(|e| {
            riglr_agents::error::AgentError::task_execution(format!(
                "Invalid address from AI: {}",
                e
            ))
        })?;

        info!("Executing AI-guided balance check for: {}", pubkey);

        // For balance checks, we don't need SignerContext - this is a read-only operation
        let start_time = std::time::Instant::now();

        // This would typically use the same RPC endpoint as the signer context
        // For this test, we'll simulate the call
        let balance_lamports = 0u64; // Placeholder - in real implementation would call get_sol_balance
        let balance_sol = balance_lamports as f64 / LAMPORTS_PER_SOL as f64;

        let execution_time = start_time.elapsed();

        Ok(TaskResult::success(
            serde_json::json!({
                "address": address_str,
                "balance_lamports": balance_lamports,
                "balance_sol": balance_sol,
                "ai_action": "get_balance",
                "ai_confidence": decision.confidence
            }),
            None,
            execution_time,
        ))
    }
}

#[async_trait]
impl Agent for IntelligentTradingAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        let description = task
            .metadata
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("No description provided");

        info!(
            "IntelligentTradingAgent processing task with AI: {}",
            description
        );

        // Use AI to analyze the task and determine the appropriate action
        let ai_decision = match self.process_with_ai(&task).await {
            Ok(decision) => {
                info!(
                    "AI decision: {} (confidence: {})",
                    decision.action, decision.confidence
                );
                info!("AI reasoning: {}", decision.reasoning);
                decision
            }
            Err(e) => {
                error!("AI processing failed: {}", e);
                return Ok(TaskResult::failure(
                    format!("AI processing failed: {}", e),
                    true,
                    Duration::from_millis(10),
                ));
            }
        };

        // Execute the AI's decision using appropriate blockchain tools
        match ai_decision.action.as_str() {
            "transfer_sol" => self.execute_sol_transfer(&ai_decision).await,
            "get_balance" => self.execute_balance_check(&ai_decision).await,
            "token_swap" => {
                // Placeholder for token swap implementation
                Ok(TaskResult::success(
                    serde_json::json!({
                        "ai_action": "token_swap",
                        "status": "not_implemented",
                        "ai_confidence": ai_decision.confidence
                    }),
                    None,
                    Duration::from_millis(100),
                ))
            }
            _ => {
                warn!("Unknown AI action: {}", ai_decision.action);
                Ok(TaskResult::failure(
                    format!(
                        "Unknown AI action: {}. Reasoning: {}",
                        ai_decision.action, ai_decision.reasoning
                    ),
                    false,
                    Duration::from_millis(10),
                ))
            }
        }
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<CapabilityType> {
        vec![
            CapabilityType::Custom("ai_trading".to_string()),
            CapabilityType::Custom("natural_language_processing".to_string()),
            CapabilityType::Custom("intelligent_blockchain_operations".to_string()),
            CapabilityType::Trading, // For capability matching
        ]
    }
}

/// Test intelligent agent decision-making with natural language tasks.
///
/// This test validates that the IntelligentTradingAgent can process
/// natural language descriptions and make appropriate blockchain operation
/// decisions using AI reasoning.
#[tokio::test]
async fn test_intelligent_agent_decision_making() {
    tracing_subscriber::fmt::try_init().ok();
    info!("Testing intelligent agent decision-making with natural language tasks");

    let harness = BlockchainTestHarness::new()
        .await
        .expect("Failed to create blockchain test harness");

    let sender_keypair = harness
        .get_funded_keypair(0)
        .expect("Failed to get sender keypair");

    let solana_signer = LocalSolanaSigner::new(
        sender_keypair.insecure_clone(),
        harness.rpc_url().to_string(),
    );
    let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(solana_signer);

    let intelligent_agent = IntelligentTradingAgent::new(unified_signer);

    // Test AI decision making with different natural language task types

    // 1. Test transfer recognition
    let mut transfer_task = Task::new(TaskType::Trading, serde_json::json!({}));
    transfer_task.metadata.insert(
        "description".to_string(),
        serde_json::json!(
            "Please send 1.5 SOL to wallet 11111111111111111111111111111112 for the payment"
        ),
    );

    let transfer_decision = intelligent_agent
        .process_with_ai(&transfer_task)
        .await
        .expect("AI should process transfer task");

    assert_eq!(transfer_decision.action, "transfer_sol");
    assert!(transfer_decision.reasoning.contains("transfer"));
    assert_eq!(transfer_decision.parameters["amount"], 1.5);
    assert_eq!(
        transfer_decision.parameters["to_address"],
        "11111111111111111111111111111112"
    );
    assert!(transfer_decision.confidence > 0.8);

    info!("✅ Transfer task AI decision validated");

    // 2. Test balance check recognition
    let mut balance_task = Task::new(TaskType::Trading, serde_json::json!({}));
    balance_task.metadata.insert(
        "description".to_string(),
        serde_json::json!("Check the current balance of wallet 11111111111111111111111111111112"),
    );

    let balance_decision = intelligent_agent
        .process_with_ai(&balance_task)
        .await
        .expect("AI should process balance task");

    assert_eq!(balance_decision.action, "get_balance");
    assert!(balance_decision.reasoning.contains("balance"));
    assert_eq!(
        balance_decision.parameters["address"],
        "11111111111111111111111111111112"
    );
    assert!(balance_decision.confidence > 0.9);

    info!("✅ Balance check AI decision validated");

    // 3. Test swap recognition
    let mut swap_task = Task::new(TaskType::Trading, serde_json::json!({}));
    swap_task.metadata.insert(
        "description".to_string(),
        serde_json::json!("I want to swap 2 SOL for USDC on the DEX"),
    );

    let swap_decision = intelligent_agent
        .process_with_ai(&swap_task)
        .await
        .expect("AI should process swap task");

    assert_eq!(swap_decision.action, "token_swap");
    assert!(swap_decision.reasoning.contains("swap"));
    assert_eq!(swap_decision.parameters["from_token"], "SOL");
    assert_eq!(swap_decision.parameters["to_token"], "USDC");

    info!("✅ Token swap AI decision validated");

    // 4. Test unknown task handling
    let mut unknown_task = Task::new(TaskType::Research, serde_json::json!({}));
    unknown_task.metadata.insert(
        "description".to_string(),
        serde_json::json!("Calculate the square root of 42"),
    );

    let unknown_decision = intelligent_agent
        .process_with_ai(&unknown_task)
        .await
        .expect("AI should process unknown task");

    assert_eq!(unknown_decision.action, "unknown");
    assert!(unknown_decision.confidence < 0.5);

    info!("✅ Unknown task AI decision validated");

    info!("Intelligent agent decision-making test completed successfully");
}

/// Test complete AI agent integration workflow with real blockchain operations.
///
/// This test validates the entire pipeline from natural language task
/// through AI decision-making to actual blockchain execution.
#[tokio::test]
async fn test_ai_agent_integration_workflow() {
    tracing_subscriber::fmt::try_init().ok();
    info!("Testing complete AI agent integration workflow");

    let harness = BlockchainTestHarness::new()
        .await
        .expect("Failed to create blockchain test harness");

    let sender_keypair = harness
        .get_funded_keypair(0)
        .expect("Failed to get sender keypair");
    let receiver_keypair = harness
        .get_funded_keypair(1)
        .expect("Failed to get receiver keypair");

    // Record initial balances
    let config = riglr_core::Config::from_env();
    let app_context = riglr_core::provider::ApplicationContext::from_config(&config);
    // Override the default RPC client with the test harness client
    app_context.set_extension(harness.get_rpc_client());
    let initial_receiver_balance =
        get_sol_balance(receiver_keypair.pubkey().to_string(), &app_context)
            .await
            .expect("Failed to get initial receiver balance");

    let initial_lamports = initial_receiver_balance.lamports;
    info!(
        "Initial receiver balance: {} SOL",
        initial_lamports as f64 / LAMPORTS_PER_SOL as f64
    );

    let solana_signer = LocalSolanaSigner::new(
        sender_keypair.insecure_clone(),
        harness.rpc_url().to_string(),
    );
    let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(solana_signer);

    let intelligent_agent = IntelligentTradingAgent::new(unified_signer);

    // Create agent system
    let registry = LocalAgentRegistry::new();
    registry
        .register_agent(Arc::new(intelligent_agent))
        .await
        .expect("Failed to register intelligent agent");

    let dispatcher = AgentDispatcher::new(Arc::new(registry));

    // Create natural language task for blockchain operation
    let mut natural_language_task = Task::new(
        TaskType::Trading,
        serde_json::json!({
            "context": "This is for testing the AI integration"
        }),
    );
    natural_language_task.metadata.insert(
        "description".to_string(),
        serde_json::json!(format!(
            "I need to send 0.5 SOL to my friend's wallet {} for lunch money",
            receiver_keypair.pubkey()
        )),
    );

    let description = natural_language_task
        .metadata
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("No description");
    info!("Dispatching natural language task: '{}'", description);

    // Execute the complete workflow through the agent system
    let task_result = dispatcher
        .dispatch_task(natural_language_task)
        .await
        .expect("Task dispatch should succeed");

    // Verify AI agent processed and executed the task correctly
    assert!(
        task_result.is_success(),
        "AI agent should successfully process and execute the task: {:?}",
        task_result
    );

    let output = task_result
        .data()
        .expect("Task should have data")
        .as_object()
        .expect("Task output should be JSON object");

    assert!(
        output.contains_key("signature"),
        "Output should contain transaction signature"
    );
    assert_eq!(
        output["ai_action"], "transfer_sol",
        "AI should have chosen transfer_sol action"
    );
    assert!(
        output["ai_confidence"].as_f64().unwrap() > 0.8,
        "AI should be confident in its decision"
    );

    let signature = output["signature"]
        .as_str()
        .expect("Should have transaction signature");

    info!(
        "AI-guided transaction completed with signature: {}",
        signature
    );

    // Wait for blockchain confirmation
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Verify blockchain state changes
    let final_receiver_balance =
        get_sol_balance(receiver_keypair.pubkey().to_string(), &app_context)
            .await
            .expect("Failed to get final receiver balance");

    let final_lamports = final_receiver_balance.lamports;
    let balance_increase = final_lamports - initial_lamports;
    let expected_increase = (0.5 * LAMPORTS_PER_SOL as f64) as u64;

    assert_eq!(
        balance_increase, expected_increase,
        "Receiver should have gained exactly 0.5 SOL from AI-guided transfer"
    );

    info!(
        "Final receiver balance: {} SOL (increased by {} lamports)",
        final_lamports as f64 / LAMPORTS_PER_SOL as f64,
        balance_increase
    );

    info!("✅ Complete AI agent integration workflow validated");
    info!("   - Natural language task processing: ✓");
    info!("   - AI decision-making: ✓");
    info!("   - Blockchain operation execution: ✓");
    info!("   - Transaction confirmation: ✓");
    info!("   - Balance change verification: ✓");
}

/// Test natural language task processing with various input formats.
///
/// This validates the AI agent's ability to handle different ways
/// users might express blockchain operation requests.
#[tokio::test]
async fn test_natural_language_task_processing() {
    tracing_subscriber::fmt::try_init().ok();
    info!("Testing natural language task processing variations");

    let harness = BlockchainTestHarness::new()
        .await
        .expect("Failed to create blockchain test harness");

    let keypair = harness
        .get_funded_keypair(0)
        .expect("Failed to get keypair");

    let solana_signer =
        LocalSolanaSigner::new(keypair.insecure_clone(), harness.rpc_url().to_string());
    let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(solana_signer);

    let intelligent_agent = IntelligentTradingAgent::new(unified_signer.clone());

    // Test various natural language expressions for transfers
    let transfer_variations = vec![
        "Transfer 2 SOL to 11111111111111111111111111111112",
        "Send 1.5 SOL to wallet 11111111111111111111111111111112",
        "Please transfer 3.0 SOL to the address 11111111111111111111111111111112",
        "I want to send 0.5 SOL to 11111111111111111111111111111112",
        "Pay 2.5 SOL to 11111111111111111111111111111112",
    ];

    for (i, description) in transfer_variations.iter().enumerate() {
        info!("Testing transfer variation {}: '{}'", i + 1, description);

        let mut task = Task::new(TaskType::Trading, serde_json::json!({}));
        task.metadata.insert(
            "description".to_string(),
            serde_json::json!(description.to_string()),
        );

        let decision = intelligent_agent
            .process_with_ai(&task)
            .await
            .expect("AI should process transfer variation");

        assert_eq!(
            decision.action, "transfer_sol",
            "Should recognize transfer intent in: '{}'",
            description
        );
        assert!(
            decision.confidence > 0.8,
            "Should be confident about transfer recognition"
        );
        assert!(
            decision.parameters.get("amount").is_some(),
            "Should extract amount from: '{}'",
            description
        );
        assert_eq!(
            decision.parameters["to_address"], "11111111111111111111111111111112",
            "Should extract correct address"
        );

        info!("✅ Transfer variation {} processed correctly", i + 1);
    }

    // Test balance check variations
    let balance_variations = vec![
        "Check balance of 11111111111111111111111111111112",
        "What's the balance of wallet 11111111111111111111111111111112?",
        "Show me the SOL balance for 11111111111111111111111111111112",
        "How much SOL does 11111111111111111111111111111112 have?",
    ];

    for (i, description) in balance_variations.iter().enumerate() {
        info!("Testing balance variation {}: '{}'", i + 1, description);

        let mut task = Task::new(TaskType::Trading, serde_json::json!({}));
        task.metadata.insert(
            "description".to_string(),
            serde_json::json!(description.to_string()),
        );

        let decision = intelligent_agent
            .process_with_ai(&task)
            .await
            .expect("AI should process balance variation");

        assert_eq!(
            decision.action, "get_balance",
            "Should recognize balance intent in: '{}'",
            description
        );
        assert!(
            decision.confidence > 0.9,
            "Should be very confident about balance recognition"
        );
        assert_eq!(
            decision.parameters["address"], "11111111111111111111111111111112",
            "Should extract correct address"
        );

        info!("✅ Balance variation {} processed correctly", i + 1);
    }

    info!("Natural language task processing variations test completed successfully");
}

/// Test AI agent error handling when tasks are ambiguous or invalid.
#[tokio::test]
async fn test_ai_agent_error_handling() {
    tracing_subscriber::fmt::try_init().ok();
    info!("Testing AI agent error handling for ambiguous tasks");

    let harness = BlockchainTestHarness::new()
        .await
        .expect("Failed to create blockchain test harness");

    let keypair = harness
        .get_funded_keypair(0)
        .expect("Failed to get keypair");

    let solana_signer =
        LocalSolanaSigner::new(keypair.insecure_clone(), harness.rpc_url().to_string());
    let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(solana_signer);

    let intelligent_agent = IntelligentTradingAgent::new(unified_signer.clone());

    // Create agent system
    let registry = LocalAgentRegistry::new();
    registry
        .register_agent(Arc::new(intelligent_agent))
        .await
        .expect("Failed to register intelligent agent");

    let dispatcher = AgentDispatcher::new(Arc::new(registry));

    // Test ambiguous task
    let mut ambiguous_task = Task::new(TaskType::Trading, serde_json::json!({}));
    ambiguous_task.metadata.insert(
        "description".to_string(),
        serde_json::json!("Do something with blockchain"),
    );

    let result = dispatcher
        .dispatch_task(ambiguous_task)
        .await
        .expect("Task dispatch should succeed");

    // Should fail gracefully with unknown action
    assert!(!result.is_success(), "Ambiguous task should not succeed");

    let output_str = result
        .error()
        .unwrap_or_else(|| result.data().and_then(|v| v.as_str()).unwrap_or(""));
    assert!(
        output_str.contains("Unknown AI action") || output_str.contains("unknown"),
        "Error should indicate unknown action"
    );

    // Test invalid address task
    let mut invalid_address_task = Task::new(TaskType::Trading, serde_json::json!({}));
    invalid_address_task.metadata.insert(
        "description".to_string(),
        serde_json::json!("Send 1 SOL to invalid_address"),
    );

    let result = dispatcher
        .dispatch_task(invalid_address_task)
        .await
        .expect("Task dispatch should succeed");

    // Should fail gracefully with address validation error
    assert!(
        !result.is_success(),
        "Invalid address task should not succeed"
    );

    info!("✅ AI agent error handling validated for ambiguous and invalid tasks");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_intelligent_agent_capabilities() {
        let keypair = Keypair::new();
        let signer = LocalSolanaSigner::new(keypair, "http://localhost:8899".to_string());
        let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(signer);

        let agent = IntelligentTradingAgent::new(unified_signer);

        let capabilities = agent.capabilities();
        assert!(capabilities.contains(&CapabilityType::Custom("ai_trading".to_string())));
        assert!(capabilities.contains(&CapabilityType::Custom(
            "natural_language_processing".to_string()
        )));
        assert!(capabilities.contains(&CapabilityType::Custom(
            "intelligent_blockchain_operations".to_string()
        )));
        assert!(capabilities.contains(&CapabilityType::Trading));

        // Test capability matching
        let trading_task = Task::new(TaskType::Trading, serde_json::json!({}));
        assert!(agent.can_handle(&trading_task));
    }

    #[test]
    fn test_amount_extraction() {
        let agent_keypair = Keypair::new();
        let signer = LocalSolanaSigner::new(agent_keypair, "http://localhost:8899".to_string());
        let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(signer);

        let agent = IntelligentTradingAgent::new(unified_signer);

        // Test various amount formats
        assert_eq!(
            agent.extract_amount_from_description("Send 2.5 SOL"),
            Some(2.5)
        );
        assert_eq!(
            agent.extract_amount_from_description("Transfer 10 SOL"),
            Some(10.0)
        );
        assert_eq!(
            agent.extract_amount_from_description("Pay 0.1 sol"),
            Some(0.1)
        );
        assert_eq!(
            agent.extract_amount_from_description("No amount here"),
            None
        );
    }

    #[test]
    fn test_address_extraction() {
        let agent_keypair = Keypair::new();
        let signer = LocalSolanaSigner::new(agent_keypair, "http://localhost:8899".to_string());
        let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(signer);

        let agent = IntelligentTradingAgent::new(unified_signer);

        let test_address = "11111111111111111111111111111112";
        let description = format!("Send SOL to {}", test_address);

        assert_eq!(
            agent.extract_address_from_description(&description),
            Some(test_address.to_string())
        );
        assert_eq!(
            agent.extract_address_from_description("No address here"),
            None
        );
    }
}
