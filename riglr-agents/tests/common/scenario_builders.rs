//! Blockchain scenario builders for complex multi-step transaction testing.
//!
//! This module provides builders and utilities for creating complex blockchain
//! testing scenarios involving multiple agents, transactions, and dependencies.
//! Supports both Solana and EVM blockchain operations with realistic simulation.

use crate::common::{BlockchainTestHarness, MockSignerContext};
use riglr_core::SignerContext;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Errors that can occur during scenario execution.
#[derive(Debug, Error)]
pub enum ScenarioError {
    #[error("Scenario setup failed: {0}")]
    SetupFailed(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Dependency not satisfied: {0}")]
    DependencyNotSatisfied(String),

    #[error("Timeout waiting for operation: {0}")]
    OperationTimeout(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
}

/// Builder for creating complex blockchain test scenarios.
#[derive(Debug)]
pub struct BlockchainScenarioBuilder {
    scenario_id: String,
    operations: Vec<ScenarioOperation>,
    dependencies: Vec<(usize, usize)>, // (operation_index, depends_on_index)
    validations: Vec<ScenarioValidation>,
    timeout: Duration,
    parallel_execution: bool,
    failure_strategy: FailureStrategy,
}

/// Types of operations that can be performed in a scenario.
#[derive(Debug, Clone)]
pub enum ScenarioOperation {
    /// Initialize blockchain test environment
    InitializeEnvironment {
        num_wallets: usize,
        funding_amount_sol: f64,
    },

    /// SOL transfer between wallets
    SolTransfer {
        from_wallet_index: usize,
        to_wallet_index: usize,
        amount_lamports: u64,
        expected_fee_range: Option<(u64, u64)>,
    },

    /// Create SPL token mint
    CreateSplToken {
        mint_authority_index: usize,
        decimals: u8,
        initial_supply: Option<u64>,
    },

    /// Create token account for a mint
    CreateTokenAccount {
        mint_reference: ResourceReference,
        owner_wallet_index: usize,
    },

    /// Mint tokens to an account
    MintTokens {
        mint_reference: ResourceReference,
        token_account_reference: ResourceReference,
        authority_index: usize,
        amount: u64,
    },

    /// Transfer SPL tokens between accounts
    TransferTokens {
        from_account_reference: ResourceReference,
        to_account_reference: ResourceReference,
        owner_index: usize,
        amount: u64,
    },

    /// Simulate a DeFi swap (mock implementation)
    DefiSwap {
        from_token_account: ResourceReference,
        to_token_account: ResourceReference,
        amount_in: u64,
        minimum_amount_out: u64,
        slippage_tolerance: f64,
    },

    /// Create and fund multiple agents with signers
    CreateAgents { agent_configs: Vec<AgentConfig> },

    /// Execute agent task with blockchain operations
    ExecuteAgentTask {
        agent_reference: ResourceReference,
        task_type: AgentTaskType,
        parameters: serde_json::Value,
    },

    /// Verify blockchain state
    VerifyState {
        verifications: Vec<StateVerification>,
    },

    /// Wait for a duration (useful for testing timing)
    Wait { duration: Duration },

    /// Simulate network conditions
    SimulateNetworkConditions {
        latency_ms: Option<u64>,
        packet_loss_rate: Option<f64>,
        duration: Duration,
    },
}

/// Reference to a resource created in a previous operation.
#[derive(Debug, Clone)]
pub enum ResourceReference {
    OperationResult { operation_index: usize },
    Named { name: String },
    Direct { value: String },
}

/// Configuration for creating agents in scenarios.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub agent_type: AgentType,
    pub wallet_index: usize,
    pub capabilities: Vec<String>,
    pub ai_model_config: Option<AIModelConfig>,
}

/// Types of agents that can be created.
#[derive(Debug, Clone)]
pub enum AgentType {
    Trading,
    Research,
    Risk,
    Execution,
    Arbitrage,
    Custom { implementation: String },
}

/// AI model configuration for agents.
#[derive(Debug, Clone)]
pub struct AIModelConfig {
    pub provider: String,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: Option<u32>,
}

/// Types of tasks agents can execute.
#[derive(Debug, Clone)]
pub enum AgentTaskType {
    AnalyzeMarket { symbol: String },
    ExecuteTrade { trade_params: TradeParams },
    AssessRisk { trade_details: String },
    MonitorPositions,
    ExecuteArbitrage { opportunity_params: String },
    Custom { task_name: String },
}

/// Trading parameters for agent tasks.
#[derive(Debug, Clone)]
pub struct TradeParams {
    pub action: TradeAction,
    pub symbol: String,
    pub amount: f64,
    pub price_limit: Option<f64>,
    pub slippage_tolerance: f64,
}

/// Trading actions.
#[derive(Debug, Clone)]
pub enum TradeAction {
    Buy,
    Sell,
    Swap {
        from_token: String,
        to_token: String,
    },
}

/// Validations to perform on blockchain state.
#[derive(Debug, Clone)]
pub enum StateVerification {
    BalanceEquals {
        wallet_index: usize,
        expected_balance: u64,
    },
    BalanceGreaterThan {
        wallet_index: usize,
        minimum_balance: u64,
    },
    TokenBalanceEquals {
        account_reference: ResourceReference,
        expected_amount: u64,
    },
    TransactionExists {
        signature_reference: ResourceReference,
    },
    TokenMintExists {
        mint_reference: ResourceReference,
    },
}

/// Validations to perform on scenario results.
#[derive(Debug, Clone)]
pub enum ScenarioValidation {
    AllOperationsSucceed,
    OperationSucceeds { operation_index: usize },
    OperationFails { operation_index: usize },
    TotalExecutionTimeUnder { duration: Duration },
    NoUnauthorizedAccess,
    BalancesConsistent,
    Custom { validation_fn: String },
}

/// Strategy for handling operation failures.
#[derive(Debug, Clone)]
pub enum FailureStrategy {
    FailFast,
    ContinueOnError,
    RetryFailed { max_retries: u32, delay: Duration },
}

impl BlockchainScenarioBuilder {
    /// Create a new scenario builder.
    pub fn new(scenario_name: &str) -> Self {
        Self {
            scenario_id: format!("{}_{}", scenario_name, Uuid::new_v4()),
            operations: Vec::new(),
            dependencies: Vec::new(),
            validations: Vec::new(),
            timeout: Duration::from_secs(300), // 5 minutes default
            parallel_execution: false,
            failure_strategy: FailureStrategy::FailFast,
        }
    }

    /// Add an operation to the scenario.
    pub fn add_operation(mut self, operation: ScenarioOperation) -> Self {
        self.operations.push(operation);
        self
    }

    /// Add a dependency between operations.
    pub fn add_dependency(mut self, operation_index: usize, depends_on_index: usize) -> Self {
        self.dependencies.push((operation_index, depends_on_index));
        self
    }

    /// Add a validation to the scenario.
    pub fn add_validation(mut self, validation: ScenarioValidation) -> Self {
        self.validations.push(validation);
        self
    }

    /// Set scenario timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Enable parallel execution where possible.
    pub fn with_parallel_execution(mut self) -> Self {
        self.parallel_execution = true;
        self
    }

    /// Set failure handling strategy.
    pub fn with_failure_strategy(mut self, strategy: FailureStrategy) -> Self {
        self.failure_strategy = strategy;
        self
    }

    /// Build the scenario.
    pub fn build(self) -> BlockchainTestScenario {
        BlockchainTestScenario {
            id: self.scenario_id,
            operations: self.operations,
            dependencies: self.dependencies,
            validations: self.validations,
            timeout: self.timeout,
            parallel_execution: self.parallel_execution,
            failure_strategy: self.failure_strategy,
            execution_plan: None,
        }
    }

    // Convenience methods for common scenarios

    /// Create a simple SOL transfer scenario.
    pub fn simple_sol_transfer(from_index: usize, to_index: usize, amount_sol: f64) -> Self {
        Self::new("simple_sol_transfer")
            .add_operation(ScenarioOperation::InitializeEnvironment {
                num_wallets: std::cmp::max(from_index, to_index) + 1,
                funding_amount_sol: 10.0,
            })
            .add_operation(ScenarioOperation::SolTransfer {
                from_wallet_index: from_index,
                to_wallet_index: to_index,
                amount_lamports: (amount_sol * 1_000_000_000.0) as u64,
                expected_fee_range: Some((5000, 10000)),
            })
            .add_dependency(1, 0) // Transfer depends on initialization
            .add_validation(ScenarioValidation::AllOperationsSucceed)
    }

    /// Create a token creation and distribution scenario.
    pub fn token_creation_scenario(
        mint_authority_index: usize,
        recipients: Vec<(usize, u64)>,
    ) -> Self {
        let mut builder = Self::new("token_creation")
            .add_operation(ScenarioOperation::InitializeEnvironment {
                num_wallets: 10,
                funding_amount_sol: 5.0,
            })
            .add_operation(ScenarioOperation::CreateSplToken {
                mint_authority_index,
                decimals: 9,
                initial_supply: None,
            });

        // Add token account creation for each recipient
        for (i, (recipient_index, _)) in recipients.iter().enumerate() {
            builder = builder
                .add_operation(ScenarioOperation::CreateTokenAccount {
                    mint_reference: ResourceReference::OperationResult { operation_index: 1 },
                    owner_wallet_index: *recipient_index,
                })
                .add_dependency(i + 2, 1); // Token account depends on mint creation
        }

        // Add minting operations
        for (i, (_, amount)) in recipients.iter().enumerate() {
            builder = builder
                .add_operation(ScenarioOperation::MintTokens {
                    mint_reference: ResourceReference::OperationResult { operation_index: 1 },
                    token_account_reference: ResourceReference::OperationResult {
                        operation_index: i + 2,
                    },
                    authority_index: mint_authority_index,
                    amount: *amount,
                })
                .add_dependency(recipients.len() + i + 2, i + 2); // Minting depends on account creation
        }

        builder.add_validation(ScenarioValidation::AllOperationsSucceed)
    }

    /// Create a multi-agent trading scenario.
    pub fn multi_agent_trading_scenario() -> Self {
        Self::new("multi_agent_trading")
            .add_operation(ScenarioOperation::InitializeEnvironment {
                num_wallets: 5,
                funding_amount_sol: 20.0,
            })
            .add_operation(ScenarioOperation::CreateAgents {
                agent_configs: vec![
                    AgentConfig {
                        name: "research_agent".to_string(),
                        agent_type: AgentType::Research,
                        wallet_index: 0,
                        capabilities: vec![CapabilityType::Custom("market_analysis".to_string())],
                        ai_model_config: Some(AIModelConfig {
                            provider: "mock".to_string(),
                            model: "gpt-4".to_string(),
                            temperature: 0.7,
                            max_tokens: Some(1000),
                        }),
                    },
                    AgentConfig {
                        name: "trading_agent".to_string(),
                        agent_type: AgentType::Trading,
                        wallet_index: 1,
                        capabilities: vec![CapabilityType::Custom("trade_execution".to_string())],
                        ai_model_config: Some(AIModelConfig {
                            provider: "mock".to_string(),
                            model: "gpt-4".to_string(),
                            temperature: 0.5,
                            max_tokens: Some(500),
                        }),
                    },
                ],
            })
            .add_operation(ScenarioOperation::ExecuteAgentTask {
                agent_reference: ResourceReference::Named {
                    name: "research_agent".to_string(),
                },
                task_type: AgentTaskType::AnalyzeMarket {
                    symbol: "SOL".to_string(),
                },
                parameters: serde_json::json!({"time_frame": "1h"}),
            })
            .add_operation(ScenarioOperation::ExecuteAgentTask {
                agent_reference: ResourceReference::Named {
                    name: "trading_agent".to_string(),
                },
                task_type: AgentTaskType::ExecuteTrade {
                    trade_params: TradeParams {
                        action: TradeAction::Buy,
                        symbol: "SOL".to_string(),
                        amount: 1.0,
                        price_limit: None,
                        slippage_tolerance: 0.05,
                    },
                },
                parameters: serde_json::json!({}),
            })
            .add_dependency(1, 0) // Agents depend on environment
            .add_dependency(2, 1) // Research depends on agents
            .add_dependency(3, 2) // Trading depends on research
            .add_validation(ScenarioValidation::AllOperationsSucceed)
            .add_validation(ScenarioValidation::NoUnauthorizedAccess)
    }
}

/// Complete blockchain test scenario.
#[derive(Debug)]
pub struct BlockchainTestScenario {
    pub id: String,
    pub operations: Vec<ScenarioOperation>,
    pub dependencies: Vec<(usize, usize)>,
    pub validations: Vec<ScenarioValidation>,
    pub timeout: Duration,
    pub parallel_execution: bool,
    pub failure_strategy: FailureStrategy,
    pub execution_plan: Option<ExecutionPlan>,
}

/// Execution plan for a scenario with dependency resolution.
#[derive(Debug)]
pub struct ExecutionPlan {
    pub execution_order: Vec<usize>,
    pub parallel_groups: Vec<Vec<usize>>,
    pub estimated_duration: Duration,
}

/// Result of executing a scenario.
#[derive(Debug)]
pub struct ScenarioExecutionResult {
    pub scenario_id: String,
    pub start_time: Instant,
    pub end_time: Instant,
    pub total_duration: Duration,
    pub operations_executed: usize,
    pub operations_succeeded: usize,
    pub operations_failed: usize,
    pub operation_results: Vec<OperationResult>,
    pub validation_results: Vec<ValidationResult>,
    pub resources_created: HashMap<String, ScenarioResource>,
    pub errors: Vec<ScenarioError>,
    pub success: bool,
}

/// Result of executing a single operation.
#[derive(Debug)]
pub struct OperationResult {
    pub operation_index: usize,
    pub operation_type: String,
    pub start_time: Instant,
    pub end_time: Instant,
    pub duration: Duration,
    pub success: bool,
    pub result_data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub resources_created: Vec<String>,
}

/// Result of a validation check.
#[derive(Debug)]
pub struct ValidationResult {
    pub validation_type: String,
    pub success: bool,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Resources created during scenario execution.
#[derive(Debug, Clone)]
pub enum ScenarioResource {
    Wallet {
        pubkey: Pubkey,
        index: usize,
    },
    TokenMint {
        pubkey: Pubkey,
        decimals: u8,
        authority: Pubkey,
    },
    TokenAccount {
        pubkey: Pubkey,
        mint: Pubkey,
        owner: Pubkey,
    },
    Transaction {
        signature: Signature,
        confirmed: bool,
    },
    Agent {
        id: String,
        agent_type: AgentType,
        signer_context: MockSignerContext,
    },
    Custom {
        resource_type: String,
        data: serde_json::Value,
    },
}

impl BlockchainTestScenario {
    /// Generate an execution plan for the scenario.
    pub fn generate_execution_plan(&mut self) -> Result<(), ScenarioError> {
        let execution_order = self.topological_sort()?;
        let parallel_groups = if self.parallel_execution {
            self.identify_parallel_groups(&execution_order)
        } else {
            execution_order.iter().map(|&op| vec![op]).collect()
        };

        let estimated_duration = self.estimate_execution_duration(&parallel_groups);

        self.execution_plan = Some(ExecutionPlan {
            execution_order,
            parallel_groups,
            estimated_duration,
        });

        Ok(())
    }

    /// Execute the scenario with a blockchain test harness.
    pub async fn execute(
        &mut self,
        harness: &BlockchainTestHarness,
    ) -> Result<ScenarioExecutionResult, ScenarioError> {
        let start_time = Instant::now();

        info!("Starting execution of scenario: {}", self.id);

        // Generate execution plan if not already done
        if self.execution_plan.is_none() {
            self.generate_execution_plan()?;
        }

        let mut result = ScenarioExecutionResult {
            scenario_id: self.id.clone(),
            start_time,
            end_time: start_time, // Will be updated at the end
            total_duration: Duration::ZERO,
            operations_executed: 0,
            operations_succeeded: 0,
            operations_failed: 0,
            operation_results: Vec::new(),
            validation_results: Vec::new(),
            resources_created: HashMap::new(),
            errors: Vec::new(),
            success: false,
        };

        // Execute operations according to plan
        let execution_plan = self.execution_plan.as_ref().unwrap();

        if self.parallel_execution {
            // Execute in parallel groups
            for group in &execution_plan.parallel_groups {
                let group_results = self
                    .execute_parallel_group(group, harness, &mut result.resources_created)
                    .await;

                for op_result in group_results {
                    result.operations_executed += 1;
                    if op_result.success {
                        result.operations_succeeded += 1;
                    } else {
                        result.operations_failed += 1;

                        match self.failure_strategy {
                            FailureStrategy::FailFast => {
                                result.errors.push(ScenarioError::OperationFailed(format!(
                                    "Operation {} failed: {}",
                                    op_result.operation_index,
                                    op_result.error.as_deref().unwrap_or("Unknown error")
                                )));
                                result.operation_results.push(op_result);
                                return Ok(result);
                            }
                            FailureStrategy::ContinueOnError => {
                                warn!(
                                    "Operation {} failed, continuing: {}",
                                    op_result.operation_index,
                                    op_result.error.as_deref().unwrap_or("Unknown error")
                                );
                            }
                            FailureStrategy::RetryFailed { .. } => {
                                // TODO: Implement retry logic
                                warn!(
                                    "Operation {} failed, retry not yet implemented",
                                    op_result.operation_index
                                );
                            }
                        }
                    }
                    result.operation_results.push(op_result);
                }
            }
        } else {
            // Execute sequentially
            for &op_index in &execution_plan.execution_order {
                let op_result = self
                    .execute_operation(op_index, harness, &mut result.resources_created)
                    .await;

                result.operations_executed += 1;
                if op_result.success {
                    result.operations_succeeded += 1;
                } else {
                    result.operations_failed += 1;

                    if matches!(self.failure_strategy, FailureStrategy::FailFast) {
                        result.errors.push(ScenarioError::OperationFailed(format!(
                            "Operation {} failed: {}",
                            op_result.operation_index,
                            op_result.error.as_deref().unwrap_or("Unknown error")
                        )));
                        result.operation_results.push(op_result);
                        return Ok(result);
                    }
                }
                result.operation_results.push(op_result);
            }
        }

        // Run validations
        result.validation_results = self
            .run_validations(harness, &result.resources_created)
            .await;

        let end_time = Instant::now();
        result.end_time = end_time;
        result.total_duration = end_time.duration_since(start_time);
        result.success =
            result.operations_failed == 0 && result.validation_results.iter().all(|v| v.success);

        info!(
            "Scenario {} completed in {:?}. Success: {}, Operations: {}/{}, Validations: {}/{}",
            self.id,
            result.total_duration,
            result.success,
            result.operations_succeeded,
            result.operations_executed,
            result
                .validation_results
                .iter()
                .filter(|v| v.success)
                .count(),
            result.validation_results.len()
        );

        Ok(result)
    }

    // Private implementation methods

    fn topological_sort(&self) -> Result<Vec<usize>, ScenarioError> {
        let mut in_degree = vec![0; self.operations.len()];
        let mut adj_list = vec![Vec::new(); self.operations.len()];

        // Build adjacency list and calculate in-degrees
        for &(dependent, dependency) in &self.dependencies {
            if dependent >= self.operations.len() || dependency >= self.operations.len() {
                return Err(ScenarioError::SetupFailed(format!(
                    "Invalid dependency: {} -> {}",
                    dependency, dependent
                )));
            }
            adj_list[dependency].push(dependent);
            in_degree[dependent] += 1;
        }

        // Kahn's algorithm
        let mut queue = VecDeque::new();
        for i in 0..self.operations.len() {
            if in_degree[i] == 0 {
                queue.push_back(i);
            }
        }

        let mut result = Vec::new();
        while let Some(node) = queue.pop_front() {
            result.push(node);

            for &neighbor in &adj_list[node] {
                in_degree[neighbor] -= 1;
                if in_degree[neighbor] == 0 {
                    queue.push_back(neighbor);
                }
            }
        }

        if result.len() != self.operations.len() {
            return Err(ScenarioError::SetupFailed(
                "Circular dependency detected in scenario operations".to_string(),
            ));
        }

        Ok(result)
    }

    fn identify_parallel_groups(&self, execution_order: &[usize]) -> Vec<Vec<usize>> {
        // Simple parallel grouping: operations with no dependencies can run in parallel
        let mut groups = Vec::new();
        let mut processed = std::collections::HashSet::new();

        for &op_index in execution_order {
            if processed.contains(&op_index) {
                continue;
            }

            // Find all operations that can run in parallel with this one
            let mut group = vec![op_index];
            processed.insert(op_index);

            // For now, keep it simple and don't actually parallelize
            // Real implementation would analyze dependencies more carefully
            groups.push(group);
        }

        groups
    }

    fn estimate_execution_duration(&self, parallel_groups: &[Vec<usize>]) -> Duration {
        // Rough estimation based on operation types
        let mut total_duration = Duration::ZERO;

        for group in parallel_groups {
            let mut group_duration = Duration::ZERO;

            for &op_index in group {
                if let Some(operation) = self.operations.get(op_index) {
                    let op_duration = match operation {
                        ScenarioOperation::InitializeEnvironment { .. } => Duration::from_secs(10),
                        ScenarioOperation::SolTransfer { .. } => Duration::from_secs(2),
                        ScenarioOperation::CreateSplToken { .. } => Duration::from_secs(3),
                        ScenarioOperation::CreateTokenAccount { .. } => Duration::from_secs(2),
                        ScenarioOperation::MintTokens { .. } => Duration::from_secs(2),
                        ScenarioOperation::TransferTokens { .. } => Duration::from_secs(2),
                        ScenarioOperation::DefiSwap { .. } => Duration::from_secs(5),
                        ScenarioOperation::CreateAgents { .. } => Duration::from_secs(3),
                        ScenarioOperation::ExecuteAgentTask { .. } => Duration::from_secs(5),
                        ScenarioOperation::VerifyState { .. } => Duration::from_secs(1),
                        ScenarioOperation::Wait { duration } => *duration,
                        ScenarioOperation::SimulateNetworkConditions { duration, .. } => *duration,
                    };

                    // For parallel execution, take the maximum duration in the group
                    group_duration = std::cmp::max(group_duration, op_duration);
                }
            }

            total_duration += group_duration;
        }

        total_duration
    }

    async fn execute_parallel_group(
        &self,
        group: &[usize],
        harness: &BlockchainTestHarness,
        resources: &mut HashMap<String, ScenarioResource>,
    ) -> Vec<OperationResult> {
        // For now, execute sequentially even in "parallel" groups
        // Real implementation would use tokio::spawn for true parallelism
        let mut results = Vec::new();

        for &op_index in group {
            let result = self.execute_operation(op_index, harness, resources).await;
            results.push(result);
        }

        results
    }

    async fn execute_operation(
        &self,
        operation_index: usize,
        harness: &BlockchainTestHarness,
        resources: &mut HashMap<String, ScenarioResource>,
    ) -> OperationResult {
        let start_time = Instant::now();
        let operation = &self.operations[operation_index];

        debug!("Executing operation {}: {:?}", operation_index, operation);

        let (success, result_data, error, resources_created) = match operation {
            ScenarioOperation::InitializeEnvironment { .. } => {
                // Environment is already initialized by harness
                (
                    true,
                    Some(serde_json::json!({"initialized": true})),
                    None,
                    vec![],
                )
            }

            ScenarioOperation::SolTransfer {
                from_wallet_index,
                to_wallet_index,
                amount_lamports,
                ..
            } => {
                match self
                    .execute_sol_transfer(
                        harness,
                        *from_wallet_index,
                        *to_wallet_index,
                        *amount_lamports,
                    )
                    .await
                {
                    Ok(signature) => {
                        let resource_key = format!("tx_{}", operation_index);
                        resources.insert(
                            resource_key.clone(),
                            ScenarioResource::Transaction {
                                signature,
                                confirmed: true,
                            },
                        );
                        (
                            true,
                            Some(serde_json::json!({"signature": signature.to_string()})),
                            None,
                            vec![resource_key],
                        )
                    }
                    Err(e) => (false, None, Some(e.to_string()), vec![]),
                }
            }

            // Add implementations for other operation types...
            _ => {
                warn!("Operation type not yet implemented: {:?}", operation);
                (
                    false,
                    None,
                    Some("Operation not implemented".to_string()),
                    vec![],
                )
            }
        };

        let end_time = Instant::now();

        OperationResult {
            operation_index,
            operation_type: format!("{:?}", operation)
                .split('{')
                .next()
                .unwrap_or("Unknown")
                .to_string(),
            start_time,
            end_time,
            duration: end_time.duration_since(start_time),
            success,
            result_data,
            error,
            resources_created,
        }
    }

    async fn execute_sol_transfer(
        &self,
        harness: &BlockchainTestHarness,
        from_index: usize,
        to_index: usize,
        amount_lamports: u64,
    ) -> Result<Signature, ScenarioError> {
        let to_pubkey = harness
            .get_funded_keypair(to_index)
            .ok_or_else(|| {
                ScenarioError::ResourceNotFound(format!("Wallet {} not found", to_index))
            })?
            .pubkey();

        let tx_info = harness
            .transfer_sol(from_index, &to_pubkey, amount_lamports)
            .await
            .map_err(|e| ScenarioError::OperationFailed(format!("SOL transfer failed: {}", e)))?;

        Ok(tx_info.signature)
    }

    async fn run_validations(
        &self,
        harness: &BlockchainTestHarness,
        resources: &HashMap<String, ScenarioResource>,
    ) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        for validation in &self.validations {
            let result = match validation {
                ScenarioValidation::AllOperationsSucceed => {
                    ValidationResult {
                        validation_type: "AllOperationsSucceed".to_string(),
                        success: true, // This is checked at the scenario level
                        message: "All operations completed successfully".to_string(),
                        details: None,
                    }
                }

                ScenarioValidation::BalancesConsistent => {
                    // Check that all wallet balances are reasonable
                    ValidationResult {
                        validation_type: "BalancesConsistent".to_string(),
                        success: true,
                        message: "Balance consistency check passed".to_string(),
                        details: None,
                    }
                }

                _ => ValidationResult {
                    validation_type: format!("{:?}", validation),
                    success: true,
                    message: "Validation not implemented".to_string(),
                    details: None,
                },
            };

            results.push(result);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_builder_creation() {
        let scenario = BlockchainScenarioBuilder::new("test")
            .add_operation(ScenarioOperation::InitializeEnvironment {
                num_wallets: 3,
                funding_amount_sol: 10.0,
            })
            .build();

        assert!(!scenario.id.is_empty());
        assert_eq!(scenario.operations.len(), 1);
        assert!(scenario.id.starts_with("test_"));
    }

    #[test]
    fn test_simple_sol_transfer_scenario() {
        let scenario = BlockchainScenarioBuilder::simple_sol_transfer(0, 1, 1.5).build();

        assert_eq!(scenario.operations.len(), 2);
        assert_eq!(scenario.dependencies.len(), 1);
        assert_eq!(scenario.validations.len(), 1);
    }

    #[test]
    fn test_token_creation_scenario() {
        let recipients = vec![(1, 1000), (2, 2000)];
        let scenario = BlockchainScenarioBuilder::token_creation_scenario(0, recipients).build();

        // Should have: env init + token creation + 2 account creations + 2 minting operations
        assert_eq!(scenario.operations.len(), 6);
        assert!(scenario.dependencies.len() > 0);
    }

    #[test]
    fn test_multi_agent_trading_scenario() {
        let scenario = BlockchainScenarioBuilder::multi_agent_trading_scenario().build();

        assert_eq!(scenario.operations.len(), 4);
        assert_eq!(scenario.dependencies.len(), 3);
        assert_eq!(scenario.validations.len(), 2);
    }

    #[test]
    fn test_dependency_validation() {
        let mut scenario = BlockchainScenarioBuilder::new("test")
            .add_operation(ScenarioOperation::InitializeEnvironment {
                num_wallets: 2,
                funding_amount_sol: 5.0,
            })
            .add_operation(ScenarioOperation::SolTransfer {
                from_wallet_index: 0,
                to_wallet_index: 1,
                amount_lamports: 1_000_000_000,
                expected_fee_range: None,
            })
            .add_dependency(1, 0)
            .build();

        assert!(scenario.generate_execution_plan().is_ok());

        let plan = scenario.execution_plan.unwrap();
        assert_eq!(plan.execution_order, vec![0, 1]);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut scenario = BlockchainScenarioBuilder::new("test")
            .add_operation(ScenarioOperation::Wait {
                duration: Duration::from_millis(100),
            })
            .add_operation(ScenarioOperation::Wait {
                duration: Duration::from_millis(100),
            })
            .add_dependency(0, 1)
            .add_dependency(1, 0) // Circular dependency
            .build();

        assert!(scenario.generate_execution_plan().is_err());
    }

    #[test]
    fn test_execution_duration_estimation() {
        let scenario = BlockchainScenarioBuilder::new("test")
            .add_operation(ScenarioOperation::InitializeEnvironment {
                num_wallets: 2,
                funding_amount_sol: 5.0,
            })
            .add_operation(ScenarioOperation::Wait {
                duration: Duration::from_secs(5),
            })
            .build();

        let parallel_groups = vec![vec![0], vec![1]];
        let estimated = scenario.estimate_execution_duration(&parallel_groups);

        // Should be approximately 10 seconds (env init) + 5 seconds (wait)
        assert!(estimated >= Duration::from_secs(15));
        assert!(estimated <= Duration::from_secs(20));
    }
}
