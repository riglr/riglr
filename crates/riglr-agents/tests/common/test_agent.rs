/// Simple test agent implementation for integration tests.
use async_trait::async_trait;
use core::time::Duration;
use riglr_agents::{
    types::{Capability, CapabilityType},
    Agent, AgentError, AgentId, Task, TaskResult,
};

/// A simple test agent for integration tests.
#[derive(Debug)]
pub struct TestAgent {
    id: AgentId,
    capabilities: Vec<CapabilityType>,
}

impl TestAgent {
    /// Create a new `TestAgent` with the given ID and capabilities.
    #[must_use]
    pub fn new(id: &str, capabilities: Vec<CapabilityType>) -> Self {
        Self {
            id: AgentId::new(id),
            capabilities,
        }
    }
}

#[async_trait]
impl Agent for TestAgent {
    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<CapabilityType> {
        self.capabilities.clone()
    }

    async fn execute_task(&self, task: Task) -> Result<TaskResult, AgentError> {
        // Simple test implementation
        Ok(TaskResult::Success {
            data: serde_json::json!({
                "agent_id": self.id.as_str(),
                "task_id": task.id,
                "message": "Test task completed"
            }),
            tx_hash: None,
            duration: Duration::from_millis(10),
        })
    }

    fn is_available(&self) -> bool {
        true
    }

    fn can_handle(&self, _task: &Task) -> bool {
        true
    }

    fn load(&self) -> f64 {
        0.1
    }

    fn status(&self) -> riglr_agents::AgentStatus {
        use riglr_agents::types::AgentState;
        riglr_agents::AgentStatus::new(
            self.id.clone(),
            AgentState::Active,
            0,
            0.1,
            self.capabilities
                .iter()
                .map(|s| Capability::new(s.to_string(), "1.0"))
                .collect(),
        )
    }
}
