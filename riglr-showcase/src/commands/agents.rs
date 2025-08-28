//! Multi-agent coordination demos
//!
//! This module demonstrates the riglr-agents framework through various scenarios:
//! - Basic multi-agent coordination
//! - Real-world trading workflows
//! - Risk management systems
//! - Cross-chain agent coordination

use anyhow::Result;
use async_trait::async_trait;
use riglr_agents::communication::CommunicationConfig;
#[allow(unused_imports)]
use riglr_agents::AgentCommunication;
use riglr_agents::{
    Agent, AgentId, AgentMessage, ChannelCommunication, Task, TaskResult, TaskType,
};
use riglr_config::Config;
#[cfg(test)]
use riglr_config::{AppConfig, DatabaseConfig, FeaturesConfig, NetworkConfig, ProvidersConfig};
#[allow(unused_imports)]
use riglr_core::SignerContext;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Create a test configuration for demos and tests
#[cfg(test)]
fn create_test_config() -> Config {
    Config {
        app: AppConfig::default(),
        database: DatabaseConfig::default(),
        network: NetworkConfig::default(),
        providers: ProvidersConfig::default(),
        features: FeaturesConfig::default(),
    }
}

/// A simple risk assessment agent that evaluates trade risk based on amount thresholds.
///
/// This agent provides basic risk analysis by calculating a risk score based on the trade amount
/// and applying threshold-based approval logic. Trades with risk scores below 0.5 are approved.
#[derive(Clone, Debug)]
pub struct SimpleRiskAgent {
    /// Unique identifier for this risk agent instance
    pub id: AgentId,
}

impl SimpleRiskAgent {
    /// Create a new SimpleRiskAgent with the specified identifier.
    ///
    /// # Arguments
    /// * `id` - A string identifier for this risk agent instance
    ///
    /// # Returns
    /// A new SimpleRiskAgent instance ready for risk assessment tasks
    pub fn new(id: &str) -> Self {
        Self {
            id: AgentId::new(id),
        }
    }
}

#[async_trait]
impl Agent for SimpleRiskAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        println!("⚖️ Risk agent {} assessing trade risk", self.id);

        let amount = task
            .parameters
            .get("amount")
            .and_then(|a| a.as_f64())
            .unwrap_or(0.0);

        // Simple risk assessment
        let risk_score = amount / 10000.0; // Simple calculation
        let approved = risk_score < 0.5;

        let result = json!({
            "approved": approved,
            "risk_score": risk_score,
            "recommendation": if approved { "APPROVE" } else { "REJECT" }
        });

        Ok(TaskResult::success(
            result,
            None,
            Duration::from_millis(100),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<riglr_agents::CapabilityType> {
        vec![riglr_agents::CapabilityType::RiskAnalysis]
    }
}

/// A coordinator agent that orchestrates multi-step workflows across multiple agents.
///
/// This agent manages complex workflows by breaking them down into sequential steps
/// and broadcasting coordination messages to worker agents. It serves as the central
/// orchestration point for multi-agent collaboration scenarios.
#[derive(Clone, Debug)]
pub struct CoordinatorAgent {
    /// Unique identifier for this coordinator agent instance
    pub id: AgentId,
    /// Shared communication channel for broadcasting messages to other agents
    pub communication: Arc<ChannelCommunication>,
}

impl CoordinatorAgent {
    /// Create a new CoordinatorAgent with the specified identifier and communication channel.
    ///
    /// # Arguments
    /// * `id` - A string identifier for this coordinator agent instance
    /// * `communication` - Shared communication channel for inter-agent messaging
    ///
    /// # Returns
    /// A new CoordinatorAgent instance ready for workflow orchestration
    pub fn new(id: &str, communication: Arc<ChannelCommunication>) -> Self {
        Self {
            id: AgentId::new(id),
            communication,
        }
    }
}

#[async_trait]
impl Agent for CoordinatorAgent {
    async fn execute_task(&self, _task: Task) -> riglr_agents::Result<TaskResult> {
        println!("👑 Coordinator {} orchestrating workflow", self.id);

        let workflow_steps = [
            "data_collection",
            "analysis",
            "decision_making",
            "execution",
        ];

        for (i, step) in workflow_steps.iter().enumerate() {
            println!("  📋 Step {}: {}", i + 1, step);

            // Send message to workers
            let message = AgentMessage::new(
                self.id.clone(),
                None, // Broadcast
                "workflow_step".to_string(),
                json!({"step": step, "sequence": i + 1}),
            );

            self.communication
                .broadcast_message(message)
                .await
                .map_err(|e| riglr_agents::AgentError::generic(e.to_string()))?;

            sleep(Duration::from_millis(100)).await;
        }

        Ok(TaskResult::success(
            json!({"workflow": "completed", "steps": workflow_steps.len()}),
            None,
            Duration::from_millis(400),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<riglr_agents::CapabilityType> {
        vec![
            riglr_agents::CapabilityType::Portfolio,
            riglr_agents::CapabilityType::Custom("coordination".to_string()),
        ]
    }
}

/// A worker agent that performs specialized tasks and responds to coordination messages.
///
/// Worker agents handle specific task types such as research and monitoring while
/// participating in coordinated workflows by responding to messages from coordinator agents.
/// They can execute tasks independently or as part of larger orchestrated processes.
#[derive(Clone, Debug)]
pub struct WorkerAgent {
    /// Unique identifier for this worker agent instance
    pub id: AgentId,
    /// Communication channel for receiving coordination messages (currently unused)
    pub _communication: Arc<ChannelCommunication>,
}

impl WorkerAgent {
    /// Create a new WorkerAgent with the specified identifier and communication channel.
    ///
    /// # Arguments
    /// * `id` - A string identifier for this worker agent instance
    /// * `communication` - Communication channel for receiving coordination messages
    ///
    /// # Returns
    /// A new WorkerAgent instance ready for task execution and coordination
    pub fn new(id: &str, communication: Arc<ChannelCommunication>) -> Self {
        Self {
            id: AgentId::new(id),
            _communication: communication,
        }
    }
}

#[async_trait]
impl Agent for WorkerAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        println!("🔧 Worker {} processing task", self.id);

        let work_type = task
            .parameters
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("general");

        // Simulate work
        sleep(Duration::from_millis(50)).await;

        Ok(TaskResult::success(
            json!({
                "worker": self.id.as_str(),
                "work_type": work_type,
                "status": "completed"
            }),
            None,
            Duration::from_millis(50),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<riglr_agents::CapabilityType> {
        vec![
            riglr_agents::CapabilityType::Research,
            riglr_agents::CapabilityType::Monitoring,
        ]
    }

    async fn handle_message(&self, message: AgentMessage) -> riglr_agents::Result<()> {
        if message.message_type == "workflow_step" {
            let step = message
                .payload
                .get("step")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown");
            let sequence = message
                .payload
                .get("sequence")
                .and_then(|s| s.as_u64())
                .unwrap_or(0);

            println!(
                "    🔧 Worker {} handling step {} ({})",
                self.id, sequence, step
            );
        }
        Ok(())
    }
}

/// Runs a multi-agent coordination demonstration based on the specified scenario.
///
/// This function demonstrates the riglr-agents framework through various predefined scenarios:
/// - `"trading"`: Real-world trading coordination with blockchain operations
/// - `"risk"`: Risk management system with coordinated assessment across multiple agents  
/// - `"basic"`: Fundamental multi-agent communication and workflow patterns
///
/// # Arguments
/// * `config` - Shared configuration for all agents and blockchain operations
/// * `scenario` - The demonstration scenario to execute
///
/// # Returns
/// Returns `Ok(())` on successful demonstration completion, or an error if the scenario
/// is unknown or the demonstration fails.
///
/// # Examples
/// ```
/// use std::sync::Arc;
/// use riglr_config::Config;
///
/// # async fn example() -> anyhow::Result<()> {
/// let config = Arc::new(create_test_config());
/// run_demo(config, "basic".to_string()).await?;
/// # Ok(())
/// # }
/// ```
pub async fn run_demo(config: Arc<Config>, scenario: String) -> Result<()> {
    println!("🤖 Starting Multi-Agent Coordination Demo");
    println!("📋 Scenario: {}", scenario);

    match scenario.as_str() {
        "trading" => run_trading_coordination_demo(config).await,
        "risk" => run_risk_management_demo(config).await,
        "basic" => run_basic_coordination_demo(config).await,
        _ => {
            println!("❌ Unknown scenario: {}", scenario);
            println!("Available scenarios: trading, risk, basic");
            Err(anyhow::anyhow!("Unknown scenario: {}", scenario))
        }
    }
}

async fn run_trading_coordination_demo(_config: Arc<Config>) -> Result<()> {
    println!("\n🔄 Running Real-World Trading Coordination Demo");
    println!("This demo shows agents working together for actual blockchain operations");

    // Setup signer context for real blockchain operations
    // TODO: Create proper signer factory
    // let signer_factory = MemorySignerFactory::new();

    // TODO: Re-enable when proper signer factory is available
    // SignerContext::new(&signer_factory).execute(async {
    //     // Run the comprehensive trading coordination example
    //     trading_coordination::demonstrate_trading_coordination(config).await?;
    //
    //     Ok::<(), riglr_core::ToolError>(())
    // }).await?;

    println!("⚠️  Agent demo temporarily disabled - signer factory needs implementation");

    Ok(())
}

async fn run_risk_management_demo(_config: Arc<Config>) -> Result<()> {
    println!("\n⚖️ Running Risk Management System Demo");
    println!("This demo shows coordinated risk assessment across multiple agents");

    // Import and run the risk management example
    // Note: In a real implementation, you would import from the examples
    // For now, we'll show a simplified version

    #[allow(unused_imports)]
    use riglr_agents::AgentCommunication;
    use riglr_agents::{
        AgentDispatcher, AgentRegistry, DispatchConfig, LocalAgentRegistry, Priority,
        RoutingStrategy, Task,
    };
    use std::time::Duration;

    // TODO: Create proper signer factory
    // let signer_factory = MemorySignerFactory::new();

    // TODO: Re-enable when proper signer factory is available
    // SignerContext::new(&signer_factory).execute(async {
    let _communication = Arc::new(ChannelCommunication::with_config(
        CommunicationConfig::default(),
    ));
    let risk_agent = Arc::new(SimpleRiskAgent::new("risk-demo-1"));

    let registry = Arc::new(LocalAgentRegistry::new());
    registry.register_agent(risk_agent).await?;

    let dispatcher = AgentDispatcher::with_config(
        registry,
        DispatchConfig {
            routing_strategy: RoutingStrategy::Capability,
            max_retries: 2,
            default_task_timeout: Duration::from_secs(30),
            retry_delay: Duration::from_secs(1),
            max_concurrent_tasks_per_agent: 3,
            enable_load_balancing: false,
            response_wait_timeout: Duration::from_secs(300),
        },
    );

    println!("📊 Testing risk assessment for different trade sizes");

    for (trade_size, expected) in [(1000.0, "APPROVE"), (8000.0, "REJECT")] {
        let task = Task::new(
            TaskType::RiskAnalysis,
            json!({"amount": trade_size, "symbol": "BTC"}),
        )
        .with_priority(Priority::High);

        let result = dispatcher.dispatch_task(task).await?;
        let default_json = serde_json::json!({});
        let decision = result
            .data()
            .unwrap_or(&default_json)
            .get("recommendation")
            .and_then(|r| r.as_str())
            .unwrap_or("UNKNOWN");

        println!("  💰 Trade size: ${:.0} -> {}", trade_size, decision);
        assert_eq!(
            decision, expected,
            "Risk assessment mismatch for trade size {}",
            trade_size
        );
    }

    println!("✅ Risk management demo completed successfully");

    // Ok::<(), riglr_core::ToolError>(())
    // }).await?;

    println!("⚠️  Function temporarily disabled - needs signer factory implementation");

    Ok(())
}

async fn run_basic_coordination_demo(_config: Arc<Config>) -> Result<()> {
    println!("\n🔄 Running Basic Agent Coordination Demo");
    println!("This demo shows fundamental multi-agent communication patterns");

    use riglr_agents::{
        AgentDispatcher, AgentRegistry, DispatchConfig, LocalAgentRegistry, Priority,
        RoutingStrategy, Task,
    };
    use std::time::Duration;

    // TODO: Create proper signer factory
    // let signer_factory = MemorySignerFactory::new();

    // TODO: Re-enable when proper signer factory is available
    // SignerContext::new(&signer_factory).execute(async {
    let communication = Arc::new(ChannelCommunication::with_config(
        CommunicationConfig::default(),
    ));

    let coordinator = Arc::new(CoordinatorAgent::new(
        "coordinator-1",
        communication.clone(),
    ));

    let worker1 = Arc::new(WorkerAgent::new("worker-1", communication.clone()));

    let worker2 = Arc::new(WorkerAgent::new("worker-2", communication.clone()));

    let registry = Arc::new(LocalAgentRegistry::new());
    registry.register_agent(coordinator).await?;
    registry.register_agent(worker1).await?;
    registry.register_agent(worker2).await?;

    let agent_count = registry.list_agents().await?.len();
    println!("✅ Registered {} agents for coordination demo", agent_count);

    let dispatcher = AgentDispatcher::with_config(
        registry,
        DispatchConfig {
            routing_strategy: RoutingStrategy::Capability,
            max_retries: 1,
            default_task_timeout: Duration::from_secs(10),
            retry_delay: Duration::from_secs(1),
            max_concurrent_tasks_per_agent: 2,
            enable_load_balancing: true,
            response_wait_timeout: Duration::from_secs(300),
        },
    );

    // Test coordination workflow
    let coordination_task = Task::new(TaskType::Portfolio, json!({"workflow": "multi_agent_demo"}))
        .with_priority(Priority::High);

    let coord_result = dispatcher.dispatch_task(coordination_task).await?;
    println!(
        "✅ Coordination completed: {} steps",
        coord_result
            .data()
            .unwrap_or(&serde_json::json!({}))
            .get("steps")
            .and_then(|s| s.as_u64())
            .unwrap_or(0)
    );

    sleep(Duration::from_millis(200)).await;

    // Test worker tasks
    println!("\n🔧 Testing individual worker capabilities");

    let research_task = Task::new(TaskType::Research, json!({"type": "market_research"}));

    let monitor_task = Task::new(TaskType::Monitoring, json!({"type": "system_monitoring"}));

    let (research_result, monitor_result) = tokio::join!(
        dispatcher.dispatch_task(research_task),
        dispatcher.dispatch_task(monitor_task)
    );

    match (research_result, monitor_result) {
        (Ok(r), Ok(m)) => {
            println!(
                "✅ Research completed by: {}",
                r.data()
                    .unwrap_or(&serde_json::json!({}))
                    .get("worker")
                    .and_then(|w| w.as_str())
                    .unwrap_or("unknown")
            );
            println!(
                "✅ Monitoring completed by: {}",
                m.data()
                    .unwrap_or(&serde_json::json!({}))
                    .get("worker")
                    .and_then(|w| w.as_str())
                    .unwrap_or("unknown")
            );
        }
        (Err(e), _) | (_, Err(e)) => {
            return Err(anyhow::anyhow!("Task execution failed: {}", e));
        }
    }

    println!("\n🎉 Basic coordination demo completed successfully!");
    println!("Demonstrated:");
    println!("  ✅ Multi-agent task routing");
    println!("  ✅ Inter-agent communication");
    println!("  ✅ Workflow orchestration");
    println!("  ✅ Parallel task execution");

    // Ok::<(), riglr_core::ToolError>(())
    // }).await?;

    println!("⚠️  Function temporarily disabled - needs signer factory implementation");

    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_run_demo_when_trading_scenario_should_call_trading_demo() {
        let config = Arc::new(create_test_config());
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(run_demo(config, "trading".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_demo_when_risk_scenario_should_call_risk_demo() {
        let config = Arc::new(create_test_config());
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(run_demo(config, "risk".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_demo_when_basic_scenario_should_call_basic_demo() {
        let config = Arc::new(create_test_config());
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(run_demo(config, "basic".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_demo_when_unknown_scenario_should_return_error() {
        let config = Arc::new(create_test_config());
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(run_demo(config, "unknown".to_string()));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Unknown scenario: unknown");
    }

    #[test]
    fn test_run_demo_when_empty_scenario_should_return_error() {
        let config = Arc::new(create_test_config());
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(run_demo(config, "".to_string()));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Unknown scenario: ");
    }

    #[test]
    fn test_run_trading_coordination_demo_should_return_ok() {
        let config = Arc::new(create_test_config());
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(run_trading_coordination_demo(config));
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_risk_management_demo_should_return_ok() {
        let config = Arc::new(create_test_config());
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(run_risk_management_demo(config));
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_basic_coordination_demo_should_return_ok() {
        let config = Arc::new(create_test_config());
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(run_basic_coordination_demo(config));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_simple_risk_agent_execute_task_when_low_amount_should_approve() {
        use riglr_agents::{Task, TaskType};
        use serde_json::json;

        let agent = SimpleRiskAgent::new("test-risk-agent");

        let task = Task::new(TaskType::RiskAnalysis, json!({"amount": 1000.0}));

        let result = agent.execute_task(task).await;
        assert!(result.is_ok());

        let task_result = result.unwrap();
        let data = task_result.data().unwrap();
        assert!(data.get("approved").unwrap().as_bool().unwrap());
        assert_eq!(
            data.get("recommendation").unwrap().as_str().unwrap(),
            "APPROVE"
        );
    }

    #[tokio::test]
    async fn test_simple_risk_agent_execute_task_when_high_amount_should_reject() {
        use riglr_agents::{Task, TaskType};
        use serde_json::json;

        let agent = SimpleRiskAgent::new("test-risk-agent");

        let task = Task::new(TaskType::RiskAnalysis, json!({"amount": 8000.0}));

        let result = agent.execute_task(task).await;
        assert!(result.is_ok());

        let task_result = result.unwrap();
        let data = task_result.data().unwrap();
        assert!(!data.get("approved").unwrap().as_bool().unwrap());
        assert_eq!(
            data.get("recommendation").unwrap().as_str().unwrap(),
            "REJECT"
        );
    }

    #[tokio::test]
    async fn test_simple_risk_agent_execute_task_when_no_amount_should_use_default() {
        use riglr_agents::{Task, TaskType};
        use serde_json::json;

        let agent = SimpleRiskAgent::new("test-risk-agent");

        let task = Task::new(TaskType::RiskAnalysis, json!({}));

        let result = agent.execute_task(task).await;
        assert!(result.is_ok());

        let task_result = result.unwrap();
        let data = task_result.data().unwrap();
        assert!(data.get("approved").unwrap().as_bool().unwrap());
        assert_eq!(
            data.get("recommendation").unwrap().as_str().unwrap(),
            "APPROVE"
        );
    }

    #[test]
    fn test_simple_risk_agent_id_should_return_correct_id() {
        let agent = SimpleRiskAgent {
            id: riglr_agents::AgentId::new("test-agent"),
        };

        assert_eq!(agent.id().as_str(), "test-agent");
    }

    #[test]
    fn test_simple_risk_agent_capabilities_should_return_risk_analysis() {
        let agent = SimpleRiskAgent {
            id: riglr_agents::AgentId::new("test-agent"),
        };

        let capabilities = agent.capabilities();
        assert_eq!(capabilities.len(), 1);
        assert_eq!(capabilities[0], riglr_agents::CapabilityType::RiskAnalysis);
    }

    #[tokio::test]
    async fn test_coordinator_agent_execute_task_should_broadcast_workflow_steps() {
        use riglr_agents::{ChannelCommunication, Task, TaskType};
        use serde_json::json;

        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let agent = CoordinatorAgent::new("test-coordinator", communication);

        let task = Task::new(TaskType::Portfolio, json!({"workflow": "test"}));

        let result = agent.execute_task(task).await;
        assert!(result.is_ok());

        let task_result = result.unwrap();
        let data = task_result.data().unwrap();
        assert_eq!(data.get("workflow").unwrap().as_str().unwrap(), "completed");
        assert_eq!(data.get("steps").unwrap().as_u64().unwrap(), 4);
    }

    #[test]
    fn test_coordinator_agent_id_should_return_correct_id() {
        let communication = Arc::new(riglr_agents::ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let agent = CoordinatorAgent::new("test-coordinator", communication);

        assert_eq!(agent.id().as_str(), "test-coordinator");
    }

    #[test]
    fn test_coordinator_agent_capabilities_should_return_portfolio_and_coordination() {
        let communication = Arc::new(riglr_agents::ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let agent = CoordinatorAgent::new("test-coordinator", communication);

        let capabilities = agent.capabilities();
        assert_eq!(capabilities.len(), 2);
        assert!(capabilities.contains(&riglr_agents::CapabilityType::Portfolio));
        assert!(capabilities.contains(&riglr_agents::CapabilityType::Custom(
            "coordination".to_string()
        )));
    }

    #[tokio::test]
    async fn test_worker_agent_execute_task_when_type_provided_should_use_type() {
        use riglr_agents::{ChannelCommunication, Task, TaskType};
        use serde_json::json;

        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let agent = WorkerAgent::new("test-worker", communication);

        let task = Task::new(TaskType::Research, json!({"type": "market_analysis"}));

        let result = agent.execute_task(task).await;
        assert!(result.is_ok());

        let task_result = result.unwrap();
        let data = task_result.data().unwrap();
        assert_eq!(data.get("worker").unwrap().as_str().unwrap(), "test-worker");
        assert_eq!(
            data.get("work_type").unwrap().as_str().unwrap(),
            "market_analysis"
        );
        assert_eq!(data.get("status").unwrap().as_str().unwrap(), "completed");
    }

    #[tokio::test]
    async fn test_worker_agent_execute_task_when_no_type_should_use_general() {
        use riglr_agents::{ChannelCommunication, Task, TaskType};
        use serde_json::json;

        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let agent = WorkerAgent::new("test-worker", communication);

        let task = Task::new(TaskType::Research, json!({}));

        let result = agent.execute_task(task).await;
        assert!(result.is_ok());

        let task_result = result.unwrap();
        let data = task_result.data().unwrap();
        assert_eq!(data.get("work_type").unwrap().as_str().unwrap(), "general");
    }

    #[test]
    fn test_worker_agent_id_should_return_correct_id() {
        let communication = Arc::new(riglr_agents::ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let agent = WorkerAgent::new("test-worker", communication);

        assert_eq!(agent.id().as_str(), "test-worker");
    }

    #[test]
    fn test_worker_agent_capabilities_should_return_research_and_monitoring() {
        let communication = Arc::new(riglr_agents::ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let agent = WorkerAgent::new("test-worker", communication);

        let capabilities = agent.capabilities();
        assert_eq!(capabilities.len(), 2);
        assert!(capabilities.contains(&riglr_agents::CapabilityType::Research));
        assert!(capabilities.contains(&riglr_agents::CapabilityType::Monitoring));
    }

    #[tokio::test]
    async fn test_worker_agent_handle_message_when_workflow_step_should_handle_correctly() {
        use riglr_agents::{AgentMessage, ChannelCommunication};
        use serde_json::json;

        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let agent = WorkerAgent::new("test-worker", communication);

        let message = AgentMessage::new(
            riglr_agents::AgentId::new("sender"),
            Some(riglr_agents::AgentId::new("test-worker")),
            "workflow_step".to_string(),
            json!({"step": "analysis", "sequence": 2}),
        );

        let result = agent.handle_message(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_worker_agent_handle_message_when_missing_step_should_use_unknown() {
        use riglr_agents::{AgentMessage, ChannelCommunication};
        use serde_json::json;

        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let agent = WorkerAgent::new("test-worker", communication);

        let message = AgentMessage::new(
            riglr_agents::AgentId::new("sender"),
            Some(riglr_agents::AgentId::new("test-worker")),
            "workflow_step".to_string(),
            json!({"sequence": 1}),
        );

        let result = agent.handle_message(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_worker_agent_handle_message_when_missing_sequence_should_use_zero() {
        use riglr_agents::{AgentMessage, ChannelCommunication};
        use serde_json::json;

        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let agent = WorkerAgent::new("test-worker", communication);

        let message = AgentMessage::new(
            riglr_agents::AgentId::new("sender"),
            Some(riglr_agents::AgentId::new("test-worker")),
            "workflow_step".to_string(),
            json!({"step": "analysis"}),
        );

        let result = agent.handle_message(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_worker_agent_handle_message_when_non_workflow_step_should_handle_gracefully() {
        use riglr_agents::{AgentMessage, ChannelCommunication};
        use serde_json::json;

        let communication = Arc::new(ChannelCommunication::with_config(
            CommunicationConfig::default(),
        ));
        let agent = WorkerAgent::new("test-worker", communication);

        let message = AgentMessage::new(
            riglr_agents::AgentId::new("sender"),
            Some(riglr_agents::AgentId::new("test-worker")),
            "other_message".to_string(),
            json!({"data": "test"}),
        );

        let result = agent.handle_message(message).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_demo_when_case_sensitive_scenario_should_return_error() {
        let config = Arc::new(create_test_config());
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(run_demo(config, "TRADING".to_string()));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Unknown scenario: TRADING");
    }

    #[test]
    fn test_run_demo_when_whitespace_scenario_should_return_error() {
        let config = Arc::new(create_test_config());
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(run_demo(config, " trading ".to_string()));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unknown scenario:  trading "
        );
    }

    #[tokio::test]
    async fn test_simple_risk_agent_execute_task_when_boundary_amount_should_handle_correctly() {
        use riglr_agents::{Task, TaskType};
        use serde_json::json;

        let agent = SimpleRiskAgent::new("test-risk-agent");

        // Test exactly at the boundary (5000.0 / 10000.0 = 0.5)
        let task = Task::new(TaskType::RiskAnalysis, json!({"amount": 5000.0}));

        let result = agent.execute_task(task).await;
        assert!(result.is_ok());

        let task_result = result.unwrap();
        let data = task_result.data().unwrap();
        // At exactly 0.5, it should be rejected (risk_score < 0.5 is the condition for approval)
        assert!(!data.get("approved").unwrap().as_bool().unwrap());
        assert_eq!(
            data.get("recommendation").unwrap().as_str().unwrap(),
            "REJECT"
        );
    }

    #[tokio::test]
    async fn test_simple_risk_agent_execute_task_when_invalid_amount_type_should_use_default() {
        use riglr_agents::{Task, TaskType};
        use serde_json::json;

        let agent = SimpleRiskAgent::new("test-risk-agent");

        let task = Task::new(TaskType::RiskAnalysis, json!({"amount": "invalid"}));

        let result = agent.execute_task(task).await;
        assert!(result.is_ok());

        let task_result = result.unwrap();
        let data = task_result.data().unwrap();
        assert!(data.get("approved").unwrap().as_bool().unwrap());
        assert_eq!(
            data.get("recommendation").unwrap().as_str().unwrap(),
            "APPROVE"
        );
    }
}
