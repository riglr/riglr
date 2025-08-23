pub mod blockchain_harness;
pub mod mock_agents;
pub mod rig_mocks;
pub mod signer_integration;
pub mod test_fixtures;
// pub mod scenario_builders; // Temporarily disabled due to compilation issues
pub mod scenario_builders_fixed;

// Re-export commonly used test utilities
pub use blockchain_harness::*;
pub use mock_agents::*;
pub use test_fixtures::*;
// pub use rig_mocks::*;
// pub use signer_integration::*;
// pub use scenario_builders::*; // Temporarily disabled
pub use scenario_builders_fixed::*;

// Test configuration constants
pub const TEST_TIMEOUT_SECS: u64 = 30;
pub const TEST_AGENT_CAPACITY: usize = 10;
pub const TEST_MESSAGE_TTL_SECS: u64 = 300;

// Helper functions for test utilities
pub fn test_agent_id(name: &str) -> AgentId {
    AgentId::new(&format!("test-agent-{}", name))
}

pub fn test_task_id(name: &str) -> String {
    format!("test-task-{}", name)
}

// Test configuration struct
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub max_agents: usize,
    pub default_timeout: std::time::Duration,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            max_agents: TEST_AGENT_CAPACITY,
            default_timeout: std::time::Duration::from_secs(TEST_TIMEOUT_SECS),
        }
    }
}

// Test scenario helpers
pub struct TradingScenarios;

impl TradingScenarios {
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

    pub fn load_test_scenario(count: usize) -> Vec<Task> {
        (0..count)
            .map(|i| {
                TestTaskBuilder::new(TaskType::Trading)
                    .with_parameters(json!({"index": i, "symbol": "BTC/USD"}))
                    .build()
            })
            .collect()
    }

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
pub struct TestData;

impl TestData {
    pub fn trading_symbols() -> Vec<&'static str> {
        vec!["BTC/USD", "ETH/USD", "SOL/USD", "BONK/USD"]
    }

    pub fn trading_actions() -> Vec<&'static str> {
        vec!["buy", "sell", "hold"]
    }

    pub fn agent_capabilities() -> Vec<Vec<String>> {
        vec![
            vec!["trading".to_string()],
            vec!["research".to_string(), "market_analysis".to_string()],
            vec!["risk_analysis".to_string()],
            vec!["portfolio".to_string(), "rebalancing".to_string()],
            vec!["monitoring".to_string()],
        ]
    }
}

// Missing imports needed for the above code
use riglr_agents::*;
use serde_json::json;
use std::sync::{Arc, Mutex};

// Helper function to create a trading swarm for coordination tests
pub fn create_trading_swarm() -> (
    MockTradingAgent,
    MockResearchAgent,
    MockRiskAgent,
    MockExecutionAgent,
    Arc<Mutex<SharedTradingState>>,
) {
    use std::collections::HashMap;

    // Simple SharedTradingState for coordination testing
    #[derive(Debug, Clone, Default)]
    struct SimpleSharedState {
        market_analysis: Option<String>,
        risk_assessment: Option<String>,
        trade_executed: bool,
        execution_result: Option<String>,
    }

    impl SimpleSharedState {
        fn new() -> Self {
            Default::default()
        }

        fn is_workflow_complete(&self) -> bool {
            self.market_analysis.is_some()
                && self.risk_assessment.is_some()
                && self.trade_executed
                && self.execution_result.is_some()
        }
    }

    let shared_state = Arc::new(Mutex::new(SharedTradingState::new()));

    let trading_agent = MockTradingAgent::new("trader", vec!["trading".to_string()]);
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
