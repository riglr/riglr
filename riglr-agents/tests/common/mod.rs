//! Common test utilities for riglr-agents integration tests.
//!
//! This module provides shared test fixtures, mock implementations,
//! and helper functions used across the test suite.

/// Blockchain testing harness utilities.
pub mod blockchain_harness;
/// Mock agent implementations for testing.
pub mod mock_agents;
/// Mock implementations for rig framework components.
pub mod rig_mocks;
/// Signer integration test utilities.
pub mod signer_integration;
/// Test fixture builders and utilities.
pub mod test_fixtures;
// pub mod scenario_builders; // Temporarily disabled due to compilation issues
/// Fixed scenario builders for testing.
pub mod scenario_builders_fixed;
/// Simple test agent implementations.
pub mod test_agent;

// Re-export commonly used test utilities
pub use blockchain_harness::*;
pub use mock_agents::*;
pub use rig_mocks::*;
pub use test_agent::*;
pub use test_fixtures::*;
// pub use signer_integration::*;
// pub use scenario_builders::*; // Temporarily disabled
pub use scenario_builders_fixed::*;

// Test configuration constants
/// Default timeout for test operations in seconds.
pub const TEST_TIMEOUT_SECS: u64 = 30;
/// Default maximum number of agents for testing.
pub const TEST_AGENT_CAPACITY: usize = 10;
/// Default time-to-live for test messages in seconds.
pub const TEST_MESSAGE_TTL_SECS: u64 = 300;

// Helper functions for test utilities
/// Creates a test agent ID with a standardized prefix.
pub fn test_agent_id(name: &str) -> AgentId {
    AgentId::new(format!("test-agent-{}", name))
}

/// Creates a test task ID with a standardized prefix.
pub fn test_task_id(name: &str) -> String {
    format!("test-task-{}", name)
}

// Test configuration struct
/// Configuration struct for test execution parameters.
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Maximum number of agents allowed in tests.
    pub max_agents: usize,
    /// Default timeout duration for test operations.
    pub default_timeout: std::time::Duration,
}

impl Default for TestConfig {
    /// Creates a default test configuration using standard constants.
    fn default() -> Self {
        Self {
            max_agents: TEST_AGENT_CAPACITY,
            default_timeout: std::time::Duration::from_secs(TEST_TIMEOUT_SECS),
        }
    }
}

// Test scenario helpers
/// Helper struct providing predefined trading scenarios for tests.
#[derive(Debug)]
pub struct TradingScenarios;

impl TradingScenarios {
    /// Creates a complete trading workflow scenario with research, risk analysis, and trading tasks.
    pub fn complete_trading_workflow() -> Vec<Task> {
        vec![
            TestTaskBuilder::new(TaskType::Research)
                .with_parameters(json!({"symbol": "BTC/USD", "type": "market_analysis"}))
                .build(),
            TestTaskBuilder::new(TaskType::RiskAnalysis)
                .with_parameters(json!({"symbol": "BTC/USD", "risk_level": "moderate"}))
                .build(),
            TestTaskBuilder::new(TaskType::Trading)
                .with_parameters(json!({"symbol": "BTC/USD", "action": "buy"}))
                .build(),
        ]
    }

    /// Creates a load testing scenario with the specified number of trading tasks.
    pub fn load_test_scenario(count: usize) -> Vec<Task> {
        (0..count)
            .map(|i| {
                TestTaskBuilder::new(TaskType::Trading)
                    .with_parameters(json!({"index": i, "symbol": "BTC/USD"}))
                    .build()
            })
            .collect()
    }

    /// Creates a priority testing scenario with tasks of different priorities.
    pub fn priority_test_scenario() -> Vec<Task> {
        vec![
            TestTaskBuilder::new(TaskType::Trading)
                .with_priority(Priority::Critical)
                .build(),
            TestTaskBuilder::new(TaskType::Research)
                .with_priority(Priority::High)
                .build(),
            TestTaskBuilder::new(TaskType::RiskAnalysis)
                .with_priority(Priority::Normal)
                .build(),
            TestTaskBuilder::new(TaskType::Portfolio)
                .with_priority(Priority::Low)
                .build(),
        ]
    }
}

// Test data generators
/// Helper struct providing common test data generators.
#[derive(Debug)]
pub struct TestData;

impl TestData {
    /// Returns a list of common trading symbol pairs for testing.
    pub fn trading_symbols() -> Vec<&'static str> {
        vec!["BTC/USD", "ETH/USD", "SOL/USD", "BONK/USD"]
    }

    /// Returns a list of common trading actions for testing.
    pub fn trading_actions() -> Vec<&'static str> {
        vec!["buy", "sell", "hold"]
    }

    /// Returns a list of common agent capability sets for testing.
    pub fn agent_capabilities() -> Vec<Vec<CapabilityType>> {
        vec![
            vec![CapabilityType::Trading],
            vec![
                CapabilityType::Research,
                CapabilityType::Custom("market_analysis".to_string()),
            ],
            vec![CapabilityType::RiskAnalysis],
            vec![
                CapabilityType::Portfolio,
                CapabilityType::Custom("rebalancing".to_string()),
            ],
            vec![CapabilityType::Monitoring],
        ]
    }
}

// Missing imports needed for the above code
use riglr_agents::*;
use serde_json::json;
use std::sync::{Arc, Mutex};

/// Creates a trading swarm with mock agents for coordination testing.
pub fn create_trading_swarm() -> (
    MockTradingAgent,
    MockResearchAgent,
    MockRiskAgent,
    MockExecutionAgent,
    Arc<Mutex<SharedTradingState>>,
) {
    // Simple SharedTradingState for coordination testing
    /// Simple shared state for coordination testing.
    #[derive(Debug, Clone, Default)]
    #[allow(dead_code)]
    struct SimpleSharedState {
        /// Market analysis result.
        market_analysis: Option<String>,
        /// Risk assessment result.
        risk_assessment: Option<String>,
        /// Whether trade has been executed.
        trade_executed: bool,
        /// Execution result.
        execution_result: Option<String>,
    }

    #[allow(dead_code)]
    impl SimpleSharedState {
        /// Creates a new SimpleSharedState.
        fn new() -> Self {
            SimpleSharedState::default()
        }

        /// Checks if the trading workflow is complete.
        fn is_workflow_complete(&self) -> bool {
            self.market_analysis.is_some()
                && self.risk_assessment.is_some()
                && self.trade_executed
                && self.execution_result.is_some()
        }
    }

    let shared_state = Arc::new(Mutex::new(SharedTradingState::new()));

    let trading_agent = MockTradingAgent::new("trader", vec![CapabilityType::Trading]);
    let research_agent = MockResearchAgent::new("researcher");
    let risk_agent = MockRiskAgent::new("risk-manager");
    let execution_agent = MockExecutionAgent::new("executor");

    (
        trading_agent,
        research_agent,
        risk_agent,
        execution_agent,
        shared_state,
    )
}
