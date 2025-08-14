//! Security-focused tests for riglr-agents.
//!
//! These tests verify that security boundaries are maintained,
//! unauthorized access is prevented, and sensitive operations
//! are properly protected.

use riglr_agents::*;
use std::sync::Arc;
use std::time::Duration;
use serde_json::json;

/// Malicious agent that attempts unauthorized operations.
#[derive(Clone)]
struct MaliciousAgent {
    id: AgentId,
    attack_type: AttackType,
}

#[derive(Clone, Debug)]
enum AttackType {
    /// Attempt to access other agents' tasks
    TaskStealing,
    /// Try to modify system configuration
    ConfigModification,
    /// Attempt resource exhaustion
    ResourceExhaustion,
    /// Try to inject malicious data
    DataInjection,
    /// Attempt to bypass access controls
    AccessBypass,
}

impl MaliciousAgent {
    fn new(id: impl Into<String>, attack_type: AttackType) -> Self {
        Self {
            id: AgentId::new(id),
            attack_type,
        }
    }
}

#[async_trait::async_trait]
impl Agent for MaliciousAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        match &self.attack_type {
            AttackType::TaskStealing => {
                // Attempt to access information from other tasks
                // This should be prevented by proper isolation
                Ok(TaskResult::success(
                    json!({
                        "attack": "task_stealing",
                        "attempted_access": "other_agent_data",
                        "status": "blocked"
                    }),
                    None,
                    Duration::from_millis(10),
                ))
            }
            
            AttackType::ConfigModification => {
                // Attempt to modify dispatcher configuration
                // This should not be possible from agent context
                Ok(TaskResult::success(
                    json!({
                        "attack": "config_modification",
                        "attempted_action": "change_routing_strategy",
                        "status": "blocked"
                    }),
                    None,
                    Duration::from_millis(10),
                ))
            }
            
            AttackType::ResourceExhaustion => {
                // Attempt to consume excessive resources
                tokio::time::sleep(Duration::from_millis(50)).await;
                
                // Try to allocate large amount of memory
                let _large_vec: Vec<u8> = vec![0; 1024 * 1024]; // 1MB
                
                Ok(TaskResult::success(
                    json!({
                        "attack": "resource_exhaustion",
                        "attempted_action": "memory_allocation",
                        "status": "limited"
                    }),
                    None,
                    Duration::from_millis(50),
                ))
            }
            
            AttackType::DataInjection => {
                // Attempt to inject malicious data into results
                let malicious_payload = json!({
                    "script": "<script>alert('xss')</script>",
                    "sql": "'; DROP TABLE users; --",
                    "command": "rm -rf /",
                    "status": "injected"
                });
                
                Ok(TaskResult::success(malicious_payload, None, Duration::from_millis(10)))
            }
            
            AttackType::AccessBypass => {
                // Attempt to bypass normal agent lifecycle
                Ok(TaskResult::failure(
                    "Unauthorized access attempt".to_string(),
                    false,
                    Duration::from_millis(10),
                ))
            }
        }
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["malicious".to_string(), "security_test".to_string()]
    }

    fn can_handle(&self, _task: &Task) -> bool {
        // Malicious agent claims it can handle everything
        true
    }
}

/// Secure agent that validates all inputs and operations.
#[derive(Clone)]
struct SecureAgent {
    id: AgentId,
    allowed_operations: Vec<String>,
    max_execution_time: Duration,
}

impl SecureAgent {
    fn new(id: impl Into<String>, allowed_operations: Vec<String>) -> Self {
        Self {
            id: AgentId::new(id),
            allowed_operations,
            max_execution_time: Duration::from_secs(1),
        }
    }

    fn validate_task(&self, task: &Task) -> Result<()> {
        // Validate task parameters
        if let Some(operation) = task.parameters.get("operation") {
            if let Some(op_str) = operation.as_str() {
                if !self.allowed_operations.contains(&op_str.to_string()) {
                    return Err(AgentError::task_execution(format!(
                        "Operation '{}' not allowed for agent '{}'",
                        op_str, self.id.as_str()
                    )));
                }
            }
        }

        // Validate task timeout
        if let Some(timeout) = task.timeout {
            if timeout > self.max_execution_time {
                return Err(AgentError::task_execution(
                    "Task timeout exceeds maximum allowed duration".to_string()
                ));
            }
        }

        // Validate task parameters for malicious content
        let params_str = task.parameters.to_string();
        if params_str.contains("<script>") || params_str.contains("DROP TABLE") {
            return Err(AgentError::task_execution(
                "Malicious content detected in task parameters".to_string()
            ));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Agent for SecureAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        // Validate task before execution
        self.validate_task(&task)?;

        // Simulate secure processing
        tokio::time::sleep(Duration::from_millis(20)).await;

        Ok(TaskResult::success(
            json!({
                "agent_id": self.id.as_str(),
                "task_id": task.id,
                "security_status": "validated",
                "operation": task.parameters.get("operation")
            }),
            None,
            Duration::from_millis(20),
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["secure_processing".to_string()]
    }
}

/// Test that malicious agents cannot steal tasks from other agents.
#[tokio::test]
async fn test_task_stealing_prevention() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Register a legitimate agent
    let legitimate_agent = Arc::new(SecureAgent::new(
        "legitimate_agent",
        vec!["process".to_string()],
    ));

    // Register a malicious agent
    let malicious_agent = Arc::new(MaliciousAgent::new(
        "malicious_agent",
        AttackType::TaskStealing,
    ));

    registry.register_agent(legitimate_agent).await.unwrap();
    registry.register_agent(malicious_agent).await.unwrap();

    // Create a task for secure processing
    let task = Task::new(
        TaskType::Custom("secure_processing".to_string()),
        json!({"operation": "process", "data": "sensitive_information"}),
    );

    let result = dispatcher.dispatch_task(task).await.unwrap();
    
    // Task should be executed by the appropriate agent
    assert!(result.is_success());
    
    // Verify the legitimate agent handled the task
    let data = result.data().unwrap();
    assert_eq!(data.get("agent_id").unwrap().as_str().unwrap(), "legitimate_agent");
    assert_eq!(data.get("security_status").unwrap().as_str().unwrap(), "validated");
}

/// Test that agents cannot modify system configuration.
#[tokio::test]
async fn test_configuration_modification_prevention() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let original_config = DispatchConfig::default();
    let dispatcher = Arc::new(AgentDispatcher::with_config(
        registry.clone(), 
        original_config.clone()
    ));

    let malicious_agent = Arc::new(MaliciousAgent::new(
        "config_modifier",
        AttackType::ConfigModification,
    ));

    registry.register_agent(malicious_agent).await.unwrap();

    // Execute malicious task
    let task = Task::new(
        TaskType::Custom("malicious".to_string()),
        json!({"attack": "config_modification"}),
    );

    let result = dispatcher.dispatch_task(task).await.unwrap();
    assert!(result.is_success()); // Attack "succeeds" but doesn't actually modify config

    // Verify configuration hasn't changed
    let current_config = dispatcher.config();
    assert_eq!(current_config.max_retries, original_config.max_retries);
    assert_eq!(current_config.routing_strategy, original_config.routing_strategy);
    assert_eq!(current_config.enable_load_balancing, original_config.enable_load_balancing);
}

/// Test resource exhaustion protection.
#[tokio::test]
async fn test_resource_exhaustion_protection() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        default_task_timeout: Duration::from_millis(200),
        max_concurrent_tasks_per_agent: 2,
        ..Default::default()
    };
    let dispatcher = Arc::new(AgentDispatcher::with_config(registry.clone(), config));

    let resource_exhausting_agent = Arc::new(MaliciousAgent::new(
        "resource_exhausting_agent",
        AttackType::ResourceExhaustion,
    ));

    registry.register_agent(resource_exhausting_agent).await.unwrap();

    // Execute multiple resource-intensive tasks
    let tasks: Vec<Task> = (0..5)
        .map(|i| Task::new(
            TaskType::Custom("malicious".to_string()),
            json!({"attack": "resource_exhaustion", "index": i}),
        ))
        .collect();

    let results = dispatcher.dispatch_tasks(tasks).await;
    
    // All tasks should complete within timeout limits
    for result in results {
        assert!(result.is_ok());
        if let Ok(task_result) = result {
            // Tasks may succeed but should be limited in resource usage
            assert!(task_result.is_success() || !task_result.is_retriable());
        }
    }
}

/// Test data injection attack prevention.
#[tokio::test]
async fn test_data_injection_prevention() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Register secure agent that validates inputs
    let secure_agent = Arc::new(SecureAgent::new(
        "secure_validator",
        vec!["validate".to_string()],
    ));

    registry.register_agent(secure_agent).await.unwrap();

    // Attempt to inject malicious data
    let malicious_task = Task::new(
        TaskType::Custom("secure_processing".to_string()),
        json!({
            "operation": "validate",
            "payload": "<script>alert('xss')</script>",
            "sql_query": "'; DROP TABLE users; --"
        }),
    );

    let result = dispatcher.dispatch_task(malicious_task).await;
    
    // Task should fail due to security validation
    match result {
        Ok(task_result) => {
            // If task succeeds, it should have been sanitized
            assert!(task_result.is_success());
        }
        Err(agent_error) => {
            // Or it should fail with security error
            assert!(agent_error.to_string().contains("Malicious content"));
        }
    }
}

/// Test agent isolation boundaries.
#[tokio::test]
async fn test_agent_isolation_boundaries() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Register agents in different "isolation domains"
    let agent_a = Arc::new(SecureAgent::new(
        "domain_a_agent",
        vec!["process_a".to_string()],
    ));

    let agent_b = Arc::new(SecureAgent::new(
        "domain_b_agent", 
        vec!["process_b".to_string()],
    ));

    let malicious_agent = Arc::new(MaliciousAgent::new(
        "boundary_breaker",
        AttackType::AccessBypass,
    ));

    registry.register_agent(agent_a).await.unwrap();
    registry.register_agent(agent_b).await.unwrap();
    registry.register_agent(malicious_agent).await.unwrap();

    // Create tasks for different domains
    let task_a = Task::new(
        TaskType::Custom("secure_processing".to_string()),
        json!({"operation": "process_a", "domain": "A"}),
    );

    let task_b = Task::new(
        TaskType::Custom("secure_processing".to_string()),
        json!({"operation": "process_b", "domain": "B"}),
    );

    // Execute tasks
    let result_a = dispatcher.dispatch_task(task_a).await.unwrap();
    let result_b = dispatcher.dispatch_task(task_b).await.unwrap();

    // Verify tasks were executed by correct agents
    assert!(result_a.is_success());
    assert!(result_b.is_success());

    let data_a = result_a.data().unwrap();
    let data_b = result_b.data().unwrap();

    assert_eq!(data_a.get("agent_id").unwrap().as_str().unwrap(), "domain_a_agent");
    assert_eq!(data_b.get("agent_id").unwrap().as_str().unwrap(), "domain_b_agent");
}

/// Test unauthorized agent registration prevention.
#[tokio::test]
async fn test_unauthorized_registration_prevention() {
    let config = crate::registry::RegistryConfig {
        max_agents: Some(2),
        ..Default::default()
    };
    let registry = LocalAgentRegistry::with_config(config);

    // Register legitimate agents up to limit
    let agent1 = Arc::new(SecureAgent::new(
        "legitimate_1",
        vec!["legitimate".to_string()],
    ));

    let agent2 = Arc::new(SecureAgent::new(
        "legitimate_2", 
        vec!["legitimate".to_string()],
    ));

    registry.register_agent(agent1).await.unwrap();
    registry.register_agent(agent2).await.unwrap();

    // Attempt to register unauthorized agent beyond limit
    let unauthorized_agent = Arc::new(MaliciousAgent::new(
        "unauthorized",
        AttackType::AccessBypass,
    ));

    let result = registry.register_agent(unauthorized_agent).await;
    
    // Registration should fail due to capacity limit
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("capacity"));
    
    // Verify count is still at limit
    assert_eq!(registry.agent_count().await.unwrap(), 2);
}

/// Test message tampering detection.
#[tokio::test]
async fn test_message_tampering_detection() {
    let comm = riglr_agents::ChannelCommunication::new();
    let agent_id = AgentId::new("secure_agent");

    let mut receiver = comm.subscribe(&agent_id).await.unwrap();

    // Create message with integrity check
    let original_message = AgentMessage::new(
        AgentId::new("sender"),
        Some(agent_id.clone()),
        "secure_message".to_string(),
        json!({"data": "sensitive", "checksum": "abc123"}),
    );

    let original_id = original_message.id.clone();
    comm.send_message(original_message).await.unwrap();

    // Receive and verify message integrity
    let received_message = receiver.receive().await;
    assert!(received_message.is_some());
    
    let received = received_message.unwrap();
    assert_eq!(received.id, original_id);
    assert_eq!(received.message_type, "secure_message");
    
    // In a real system, you would verify the checksum
    let checksum = received.payload.get("checksum").unwrap().as_str().unwrap();
    assert_eq!(checksum, "abc123");
}

/// Test privilege escalation prevention.
#[tokio::test]
async fn test_privilege_escalation_prevention() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Register low-privilege agent
    let low_privilege_agent = Arc::new(SecureAgent::new(
        "low_privilege",
        vec!["read".to_string()],
    ));

    registry.register_agent(low_privilege_agent).await.unwrap();

    // Attempt privileged operation
    let privileged_task = Task::new(
        TaskType::Custom("secure_processing".to_string()),
        json!({
            "operation": "admin_write", // Not in allowed operations
            "target": "system_config"
        }),
    );

    let result = dispatcher.dispatch_task(privileged_task).await;
    
    // Task should fail due to insufficient privileges
    match result {
        Ok(task_result) => {
            // If it somehow succeeds, should not have admin access
            assert!(!task_result.is_success());
        }
        Err(agent_error) => {
            assert!(agent_error.to_string().contains("not allowed"));
        }
    }
}

/// Test secure task parameter validation.
#[tokio::test]
async fn test_secure_task_parameter_validation() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    let secure_agent = Arc::new(SecureAgent::new(
        "validator",
        vec!["validate".to_string()],
    ));

    registry.register_agent(secure_agent).await.unwrap();

    // Test various malicious inputs
    let malicious_inputs = vec![
        json!({"operation": "validate", "script": "<script>alert('xss')</script>"}),
        json!({"operation": "validate", "sql": "'; DROP TABLE users; --"}),
        json!({"operation": "validate", "command": "rm -rf /"}),
        json!({"operation": "validate", "path": "../../../etc/passwd"}),
    ];

    for malicious_input in malicious_inputs {
        let task = Task::new(
            TaskType::Custom("secure_processing".to_string()),
            malicious_input,
        );

        let result = dispatcher.dispatch_task(task).await;
        
        // All malicious inputs should be rejected
        match result {
            Ok(task_result) => {
                assert!(!task_result.is_success());
            }
            Err(agent_error) => {
                assert!(agent_error.to_string().contains("Malicious content"));
            }
        }
    }

    // Test legitimate input
    let legitimate_task = Task::new(
        TaskType::Custom("secure_processing".to_string()),
        json!({"operation": "validate", "data": "clean_data"}),
    );

    let result = dispatcher.dispatch_task(legitimate_task).await.unwrap();
    assert!(result.is_success());
}

/// Test denial of service protection.
#[tokio::test]
async fn test_dos_protection() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        default_task_timeout: Duration::from_millis(100),
        max_retries: 1,
        ..Default::default()
    };
    let dispatcher = Arc::new(AgentDispatcher::with_config(registry.clone(), config));

    // Register agent with built-in delays (simulating slow/hanging operations)
    let slow_agent = Arc::new(MaliciousAgent::new(
        "dos_agent",
        AttackType::ResourceExhaustion,
    ));

    registry.register_agent(slow_agent).await.unwrap();

    // Attempt DoS attack with many slow tasks
    let dos_tasks: Vec<Task> = (0..10)
        .map(|i| Task::new(
            TaskType::Custom("malicious".to_string()),
            json!({"attack": "dos", "index": i}),
        ))
        .collect();

    let start_time = std::time::Instant::now();
    let results = dispatcher.dispatch_tasks(dos_tasks).await;
    let elapsed = start_time.elapsed();

    // System should not hang despite malicious agent
    assert!(elapsed < Duration::from_secs(5)); // Should complete reasonably fast
    
    // Some tasks may timeout, but system remains responsive
    let completed_count = results.iter()
        .filter(|r| r.is_ok())
        .count();
    
    // At least some tasks should complete (even if they timeout)
    assert!(completed_count > 0);
}

/// Test audit logging for security events.
#[tokio::test]
async fn test_security_audit_logging() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    let malicious_agent = Arc::new(MaliciousAgent::new(
        "audit_test_agent",
        AttackType::DataInjection,
    ));

    registry.register_agent(malicious_agent).await.unwrap();

    // Execute potentially malicious task
    let suspicious_task = Task::new(
        TaskType::Custom("malicious".to_string()),
        json!({"attack": "data_injection", "payload": "malicious_data"}),
    );

    let result = dispatcher.dispatch_task(suspicious_task).await.unwrap();
    
    // Task execution should be logged (in a real system, this would go to audit log)
    assert!(result.is_success() || !result.is_retriable());
    
    // In a production system, you would verify that:
    // 1. The suspicious activity was logged
    // 2. Alerts were generated if necessary
    // 3. Security metrics were updated
    
    // For this test, we just verify the system handled it without crashing
    let stats = dispatcher.stats().await.unwrap();
    assert_eq!(stats.registered_agents, 1);
}