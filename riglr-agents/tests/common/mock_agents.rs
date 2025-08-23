use async_trait::async_trait;
use riglr_agents::*;
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
pub struct MockTradingAgent {
    id: AgentId,
    capabilities: Vec<String>,
    execution_delay: Duration,
    should_fail: bool,
    execution_count: AtomicU32,
    load_factor: f64,
    state: AgentState,
}

impl MockTradingAgent {
    pub fn new(id: &str, capabilities: Vec<String>) -> Self {
        Self {
            id: AgentId::new(id),
            capabilities,
            execution_delay: Duration::from_millis(100),
            should_fail: false,
            execution_count: AtomicU32::new(0),
            load_factor: 0.0,
            state: AgentState::Idle,
        }
    }

    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.execution_delay = delay;
        self
    }

    pub fn with_load(mut self, load: f64) -> Self {
        self.load_factor = load;
        self
    }

    pub fn with_state(mut self, state: AgentState) -> Self {
        self.state = state;
        self
    }

    pub fn execution_count(&self) -> u32 {
        self.execution_count.load(Ordering::Relaxed)
    }
}

impl Clone for MockTradingAgent {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            capabilities: self.capabilities.clone(),
            execution_delay: self.execution_delay,
            should_fail: self.should_fail,
            execution_count: AtomicU32::new(self.execution_count.load(Ordering::Relaxed)),
            load_factor: self.load_factor,
            state: self.state,
        }
    }
}

#[async_trait]
impl Agent for MockTradingAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        self.execution_count.fetch_add(1, Ordering::Relaxed);

        sleep(self.execution_delay).await;

        if self.should_fail {
            return Ok(TaskResult::failure(
                "Mock agent configured to fail".to_string(),
                true, // retriable
                self.execution_delay,
            ));
        }

        Ok(TaskResult::success(
            json!({
                "agent_id": self.id.as_str(),
                "task_id": task.id,
                "task_type": task.task_type.to_string(),
                "execution_count": self.execution_count.load(Ordering::Relaxed)
            }),
            None,
            self.execution_delay,
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }

    fn status(&self) -> AgentStatus {
        AgentStatus {
            agent_id: self.id.clone(),
            status: self.state,
            active_tasks: self.execution_count.load(Ordering::Relaxed),
            load: self.load_factor,
            last_heartbeat: chrono::Utc::now(),
            capabilities: self
                .capabilities()
                .into_iter()
                .map(|cap| Capability::new(cap, "1.0"))
                .collect(),
            metadata: HashMap::new(),
        }
    }

    fn load(&self) -> f64 {
        self.load_factor
    }

    fn is_available(&self) -> bool {
        match self.state {
            AgentState::Active | AgentState::Idle | AgentState::Busy => true,
            AgentState::Full | AgentState::Offline | AgentState::Maintenance => false,
        }
    }
}

#[derive(Debug)]
pub struct MockResearchAgent {
    id: AgentId,
    capabilities: Vec<String>,
    shared_state: Option<Arc<tokio::sync::RwLock<SharedTradingState>>>,
}

impl MockResearchAgent {
    pub fn new(id: &str) -> Self {
        Self {
            id: AgentId::new(id),
            capabilities: vec!["research".to_string(), "market_analysis".to_string()],
            shared_state: None,
        }
    }

    pub fn with_shared_state(
        mut self,
        state: Arc<tokio::sync::RwLock<SharedTradingState>>,
    ) -> Self {
        self.shared_state = Some(state);
        self
    }
}

impl Clone for MockResearchAgent {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            capabilities: self.capabilities.clone(),
            shared_state: self.shared_state.clone(),
        }
    }
}

#[async_trait]
impl Agent for MockResearchAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        sleep(Duration::from_millis(50)).await;

        let analysis = format!("Market analysis for task: {}", task.id);

        // Update shared state if available
        if let Some(ref state) = self.shared_state {
            let mut state_lock = state.write().await;
            state_lock.market_analysis = Some(analysis.clone());
        }

        Ok(TaskResult::success(
            json!({
                "agent_id": self.id.as_str(),
                "task_id": task.id,
                "analysis": analysis,
                "type": "market_analysis"
            }),
            None,
            Duration::from_millis(50),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }
}

#[derive(Debug)]
pub struct MockRiskAgent {
    id: AgentId,
    capabilities: Vec<String>,
    shared_state: Option<Arc<tokio::sync::RwLock<SharedTradingState>>>,
}

impl MockRiskAgent {
    pub fn new(id: &str) -> Self {
        Self {
            id: AgentId::new(id),
            capabilities: vec!["risk_analysis".to_string()],
            shared_state: None,
        }
    }

    pub fn with_shared_state(
        mut self,
        state: Arc<tokio::sync::RwLock<SharedTradingState>>,
    ) -> Self {
        self.shared_state = Some(state);
        self
    }
}

impl Clone for MockRiskAgent {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            capabilities: self.capabilities.clone(),
            shared_state: self.shared_state.clone(),
        }
    }
}

#[async_trait]
impl Agent for MockRiskAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        sleep(Duration::from_millis(75)).await;

        // Check if market analysis is available in shared state
        if let Some(ref state) = self.shared_state {
            let state_lock = state.read().await;
            if state_lock.market_analysis.is_none() {
                return Ok(TaskResult::failure(
                    "Cannot perform risk analysis without market analysis".to_string(),
                    true,
                    Duration::from_millis(75),
                ));
            }
        }

        let assessment = format!("Risk assessment for task: {}", task.id);

        // Update shared state
        if let Some(ref state) = self.shared_state {
            let mut state_lock = state.write().await;
            state_lock.risk_assessment = Some(assessment.clone());
        }

        Ok(TaskResult::success(
            json!({
                "agent_id": self.id.as_str(),
                "task_id": task.id,
                "assessment": assessment,
                "type": "risk_assessment"
            }),
            None,
            Duration::from_millis(75),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }
}

#[derive(Debug)]
pub struct MockExecutionAgent {
    id: AgentId,
    capabilities: Vec<String>,
    shared_state: Option<Arc<tokio::sync::RwLock<SharedTradingState>>>,
}

impl MockExecutionAgent {
    pub fn new(id: &str) -> Self {
        Self {
            id: AgentId::new(id),
            capabilities: vec!["trading".to_string(), "trade_execution".to_string()],
            shared_state: None,
        }
    }

    pub fn with_shared_state(
        mut self,
        state: Arc<tokio::sync::RwLock<SharedTradingState>>,
    ) -> Self {
        self.shared_state = Some(state);
        self
    }
}

impl Clone for MockExecutionAgent {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            capabilities: self.capabilities.clone(),
            shared_state: self.shared_state.clone(),
        }
    }
}

#[async_trait]
impl Agent for MockExecutionAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        sleep(Duration::from_millis(100)).await;

        // Check prerequisites in shared state
        if let Some(ref state) = self.shared_state {
            let state_lock = state.read().await;
            if state_lock.market_analysis.is_none() || state_lock.risk_assessment.is_none() {
                return Ok(TaskResult::failure(
                    "Cannot execute trade without analysis and risk assessment".to_string(),
                    true,
                    Duration::from_millis(100),
                ));
            }
        }

        let execution_result = format!("Trade executed for task: {}", task.id);

        // Update shared state
        if let Some(ref state) = self.shared_state {
            let mut state_lock = state.write().await;
            state_lock.trade_executed = true;
            state_lock.execution_result = Some(execution_result.clone());
        }

        Ok(TaskResult::success(
            json!({
                "agent_id": self.id.as_str(),
                "task_id": task.id,
                "result": execution_result,
                "type": "trade_execution",
                "tx_hash": "0xmock_transaction_hash"
            }),
            Some("0xmock_transaction_hash".to_string()),
            Duration::from_millis(100),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }
}

#[derive(Debug, Clone, Default)]
pub struct SharedTradingState {
    pub market_analysis: Option<String>,
    pub risk_assessment: Option<String>,
    pub trade_executed: bool,
    pub execution_result: Option<String>,
}

impl SharedTradingState {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn is_workflow_complete(&self) -> bool {
        self.market_analysis.is_some()
            && self.risk_assessment.is_some()
            && self.trade_executed
            && self.execution_result.is_some()
    }
}

// Helper function to create a set of agents for testing
pub fn create_test_agent_set() -> Vec<Arc<dyn Agent>> {
    vec![
        Arc::new(MockTradingAgent::new(
            "trader-1",
            vec!["trading".to_string()],
        )),
        Arc::new(MockResearchAgent::new("researcher-1")),
        Arc::new(MockRiskAgent::new("risk-1")),
        Arc::new(MockExecutionAgent::new("executor-1")),
    ]
}

// Helper function to create agents with shared state for workflow testing
pub fn create_workflow_agents() -> (
    Vec<Arc<dyn Agent>>,
    Arc<tokio::sync::RwLock<SharedTradingState>>,
) {
    let shared_state = Arc::new(tokio::sync::RwLock::new(SharedTradingState::new()));

    let agents: Vec<Arc<dyn Agent>> = vec![
        Arc::new(MockResearchAgent::new("researcher").with_shared_state(shared_state.clone())),
        Arc::new(MockRiskAgent::new("risk-manager").with_shared_state(shared_state.clone())),
        Arc::new(MockExecutionAgent::new("executor").with_shared_state(shared_state.clone())),
    ];

    (agents, shared_state)
}
