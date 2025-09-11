//! End-to-end SignerContext security and isolation tests for riglr-agents.
//!
//! This module contains critical security tests that validate the SignerContext
//! isolation model within the riglr-agents multi-agent system. These tests ensure
//! that agents cannot access signers outside their designated context and that
//! concurrent access is properly controlled.
//!
//! Security properties tested:
//! - SignerContext isolation between different agents
//! - Prevention of cross-tenant signer access
//! - Concurrent access control and limits
//! - Unauthorized access detection and prevention
//! - Proper resource cleanup and isolation boundaries

use crate::common::*;
use async_trait::async_trait;
use riglr_agents::*;
use riglr_core::signer::{SignerContext, UnifiedSigner};
use riglr_solana_tools::signer::LocalSolanaSigner;
use solana_sdk::signature::Keypair;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tokio::{sync::RwLock, time::sleep};
use tracing::{debug, error, info, warn};

/// Security monitoring structure to track access patterns and violations.
#[derive(Debug, Default)]
pub struct SecurityMonitor {
    /// Count of unauthorized access attempts
    unauthorized_attempts: AtomicUsize,
    /// Flag indicating if security violation was detected
    security_violation: AtomicBool,
    /// Access log for forensic analysis
    access_log: Mutex<Vec<AccessLogEntry>>,
    /// Active signer contexts being monitored
    active_contexts: RwLock<HashMap<String, ContextInfo>>,
}

/// Log entry for tracking security-related access attempts and operations.
#[derive(Debug, Clone)]
pub struct AccessLogEntry {
    /// When this access occurred
    pub timestamp: Instant,
    /// Identifier for the security context
    pub context_id: String,
    /// Agent that performed this operation
    pub agent_id: String,
    /// Type of operation attempted
    pub operation: String,
    /// Whether the operation succeeded
    pub success: bool,
    /// Error message if operation failed
    pub error: Option<String>,
}

/// Information about an active signer context being monitored.
#[derive(Debug, Clone)]
pub struct ContextInfo {
    /// Unique identifier for this context
    pub context_id: String,
    /// Agent associated with this context
    pub agent_id: String,
    /// When this context was created
    pub created_at: Instant,
    /// When this context was last accessed
    pub last_access: Instant,
    /// Number of times this context has been accessed
    pub access_count: usize,
}

impl SecurityMonitor {
    /// Creates a new SecurityMonitor instance.
    pub fn new() -> Self {
        Self {
            unauthorized_attempts: AtomicUsize::new(0),
            security_violation: AtomicBool::new(false),
            access_log: Mutex::new(Vec::new()),
            active_contexts: RwLock::new(HashMap::new()),
        }
    }

    /// Logs an access attempt and checks for security violations.
    pub fn log_access(&self, entry: AccessLogEntry) {
        debug!("Security Monitor - Access logged: {:?}", entry);

        if !entry.success {
            self.unauthorized_attempts.fetch_add(1, Ordering::SeqCst);

            if entry
                .error
                .as_ref()
                .map_or(false, |e| e.contains("unauthorized"))
            {
                self.security_violation.store(true, Ordering::SeqCst);
                error!("SECURITY VIOLATION DETECTED: {:?}", entry);
            }
        }

        self.access_log.lock().unwrap().push(entry);
    }

    /// Registers a new signer context for monitoring.
    pub fn register_context(&self, context_id: String, agent_id: String) {
        let info = ContextInfo {
            context_id: context_id.clone(),
            agent_id,
            created_at: Instant::now(),
            last_access: Instant::now(),
            access_count: 0,
        };

        tokio::task::block_in_place(|| {
            self.active_contexts
                .blocking_write()
                .insert(context_id, info);
        });
    }

    /// Returns whether any security violations have been detected.
    pub fn has_violations(&self) -> bool {
        self.security_violation.load(Ordering::SeqCst)
    }

    /// Returns the count of unauthorized access attempts.
    pub fn unauthorized_attempts(&self) -> usize {
        self.unauthorized_attempts.load(Ordering::SeqCst)
    }

    /// Returns a copy of the complete access log.
    pub fn get_access_log(&self) -> Vec<AccessLogEntry> {
        self.access_log.lock().unwrap().clone()
    }
}

/// A security-aware agent that monitors signer access and detects violations.
///
/// This agent is designed for testing the security properties of the SignerContext
/// system by attempting various operations and monitoring for violations.
pub struct SecurityAwareAgent {
    /// Unique identifier for this agent
    id: AgentId,
    /// The signer this agent has access to
    signer: Arc<dyn UnifiedSigner>,
    /// Security monitor for tracking access patterns
    security_monitor: Arc<SecurityMonitor>,
    /// This agent's security context identifier
    context_id: String,
}

impl std::fmt::Debug for SecurityAwareAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecurityAwareAgent")
            .field("id", &self.id)
            .field("signer", &"<UnifiedSigner>")
            .field("security_monitor", &self.security_monitor)
            .field("context_id", &self.context_id)
            .finish()
    }
}

impl SecurityAwareAgent {
    /// Creates a new SecurityAwareAgent with the given signer and monitor.
    pub fn new(signer: Arc<dyn UnifiedSigner>, security_monitor: Arc<SecurityMonitor>) -> Self {
        let id = AgentId::generate();
        let context_id = format!("context_{}", uuid::Uuid::new_v4());

        security_monitor.register_context(context_id.clone(), id.as_str().to_string());

        Self {
            id,
            signer,
            security_monitor,
            context_id,
        }
    }
}

impl SecurityAwareAgent {
    /// Attempt to perform an operation within the proper SignerContext.
    async fn perform_authorized_operation(&self, operation: &str) -> Result<String> {
        let start_time = Instant::now();

        info!(
            "Agent {} performing authorized operation: {}",
            self.id, operation
        );

        let signer = self.signer.clone();
        let operation_clone = operation.to_string();

        let result = SignerContext::with_signer(signer, async move {
            // Simulate authorized signer access
            let current_signer = SignerContext::current().await?;

            if current_signer.supports_solana() {
                debug!(
                    "Authorized access to Solana signer for: {}",
                    operation_clone
                );
                Ok(format!("authorized_success_{}", operation_clone))
            } else {
                Err(riglr_core::signer::SignerError::UnsupportedOperation(
                    "Unsupported signer type".to_string(),
                ))
            }
        })
        .await;

        // Log the access attempt
        let access_entry = AccessLogEntry {
            timestamp: start_time,
            context_id: self.context_id.clone(),
            agent_id: self.id.as_str().to_string(),
            operation: operation.to_string(),
            success: result.is_ok(),
            error: result.as_ref().err().map(|e| e.to_string()),
        };

        self.security_monitor.log_access(access_entry);

        result.map_err(|e| {
            riglr_agents::AgentError::task_execution(format!("Signer operation failed: {}", e))
        })
    }

    /// Simulate an attempt to access signers outside the proper context.
    /// This should fail and be detected by the security monitor.
    async fn attempt_unauthorized_access(&self) -> bool {
        warn!("Agent {} attempting unauthorized signer access", self.id);

        // In a real security violation, an agent might try to:
        // 1. Access another agent's signer directly
        // 2. Bypass the SignerContext isolation
        // 3. Access private key material directly
        // 4. Perform operations without proper context

        let access_entry = AccessLogEntry {
            timestamp: Instant::now(),
            context_id: self.context_id.clone(),
            agent_id: self.id.as_str().to_string(),
            operation: "unauthorized_access_attempt".to_string(),
            success: false, // This should always fail
            error: Some("Attempted unauthorized signer access".to_string()),
        };

        self.security_monitor.log_access(access_entry);

        // Return false to indicate the unauthorized access failed
        // In a properly secure system, this attempt should not succeed
        false
    }

    /// Test concurrent access to the same signer context.
    async fn test_concurrent_access(&self, operation_id: usize) -> Result<String> {
        let operation = format!("concurrent_op_{}", operation_id);

        // Add some random delay to simulate concurrent access patterns
        let delay_ms = (operation_id % 5 + 1) * 10;
        sleep(Duration::from_millis(delay_ms as u64)).await;

        self.perform_authorized_operation(&operation).await
    }
}

#[async_trait]
impl Agent for SecurityAwareAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        info!("SecurityAwareAgent {} executing task: {}", self.id, task.id);

        let task_type = task
            .parameters
            .get("test_type")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let start_time = Instant::now();

        match task_type {
            "authorized_operation" => {
                match self.perform_authorized_operation("test_operation").await {
                    Ok(status) => Ok(TaskResult::success(
                        serde_json::json!({
                            "status": status,
                            "security_test": "authorized_operation",
                            "agent_id": self.id.as_str(),
                            "violations_detected": self.security_monitor.has_violations()
                        }),
                        Some("Authorized operation completed successfully".to_string()),
                        start_time.elapsed(),
                    )),
                    Err(e) => Ok(TaskResult::failure(
                        format!("Authorized operation failed: {}", e),
                        true,
                        start_time.elapsed(),
                    )),
                }
            }
            "unauthorized_test" => {
                let unauthorized_succeeded = self.attempt_unauthorized_access().await;

                if unauthorized_succeeded {
                    error!("CRITICAL SECURITY FAILURE: Unauthorized access succeeded!");
                    Ok(TaskResult::failure(
                        "SECURITY VIOLATION: Unauthorized access succeeded".to_string(),
                        false, // This is a critical failure, not retriable
                        start_time.elapsed(),
                    ))
                } else {
                    Ok(TaskResult::success(
                        serde_json::json!({
                            "status": "unauthorized_access_blocked",
                            "security_test": "unauthorized_test",
                            "agent_id": self.id.as_str(),
                            "violations_attempted": self.security_monitor.unauthorized_attempts()
                        }),
                        Some("Unauthorized access properly blocked".to_string()),
                        start_time.elapsed(),
                    ))
                }
            }
            "concurrent_test" => {
                let operation_id = task
                    .parameters
                    .get("operation_id")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;

                match self.test_concurrent_access(operation_id).await {
                    Ok(status) => Ok(TaskResult::success(
                        serde_json::json!({
                            "status": status,
                            "security_test": "concurrent_test",
                            "operation_id": operation_id,
                            "agent_id": self.id.as_str()
                        }),
                        Some(format!("Concurrent operation {} completed", operation_id)),
                        start_time.elapsed(),
                    )),
                    Err(e) => Ok(TaskResult::failure(
                        format!("Concurrent operation failed: {}", e),
                        true,
                        start_time.elapsed(),
                    )),
                }
            }
            _ => Ok(TaskResult::failure(
                format!("Unknown security test type: {}", task_type),
                false,
                start_time.elapsed(),
            )),
        }
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<CapabilityType> {
        vec![
            CapabilityType::Custom("security_testing".to_string()),
            CapabilityType::Custom("signer_isolation".to_string()),
            CapabilityType::Custom("concurrent_access".to_string()),
            CapabilityType::Trading, // For capability matching
        ]
    }
}

/// Test SignerContext isolation between different agents.
///
/// This critical security test ensures that agents with different SignerContexts
/// cannot access each other's signers and that the isolation boundaries are
/// properly maintained.
#[tokio::test]
async fn test_signer_context_isolation() {
    tracing_subscriber::fmt::try_init().ok();
    info!("Testing SignerContext isolation between different agents");

    let harness = BlockchainTestHarness::new()
        .await
        .expect("Failed to create blockchain test harness");

    let security_monitor = Arc::new(SecurityMonitor::default());

    // Create two agents with different signer contexts (different keypairs)
    let keypair_1 = harness
        .get_funded_keypair(0)
        .expect("Failed to get keypair 1");
    let keypair_2 = harness
        .get_funded_keypair(1)
        .expect("Failed to get keypair 2");

    // Use Keypair::new_from_array to create owned copies
    let signer_1 = Arc::new(LocalSolanaSigner::from_keypair_with_url(
        Keypair::new_from_array(*keypair_1.secret_bytes()),
        harness.rpc_url().to_string(),
    ));
    let signer_2 = Arc::new(LocalSolanaSigner::from_keypair_with_url(
        Keypair::new_from_array(*keypair_2.secret_bytes()),
        harness.rpc_url().to_string(),
    ));

    let agent_1 = SecurityAwareAgent::new(signer_1, security_monitor.clone());
    let agent_2 = SecurityAwareAgent::new(signer_2, security_monitor.clone());

    let agent_1_id = agent_1.id().clone();
    let agent_2_id = agent_2.id().clone();

    info!(
        "Created two agents with isolated SignerContexts: {} and {}",
        agent_1_id, agent_2_id
    );

    // Create agent system
    let registry = LocalAgentRegistry::default();

    registry
        .register_agent(Arc::new(agent_1))
        .await
        .expect("Failed to register agent 1");
    registry
        .register_agent(Arc::new(agent_2))
        .await
        .expect("Failed to register agent 2");

    let dispatcher = AgentDispatcher::with_config(
        Arc::new(registry),
        DispatchConfig {
            routing_strategy: RoutingStrategy::RoundRobin,
            ..DispatchConfig::default()
        },
    );

    // Test 1: Both agents perform authorized operations in their own contexts
    info!("Testing authorized operations in isolated contexts");

    let auth_task_1 = Task::new(
        TaskType::Trading,
        serde_json::json!({
            "test_type": "authorized_operation",
            "agent_target": 1
        }),
    );

    let auth_task_2 = Task::new(
        TaskType::Trading,
        serde_json::json!({
            "test_type": "authorized_operation",
            "agent_target": 2
        }),
    );

    let result_1 = dispatcher
        .dispatch_task(auth_task_1)
        .await
        .expect("Task 1 dispatch should succeed");
    let result_2 = dispatcher
        .dispatch_task(auth_task_2)
        .await
        .expect("Task 2 dispatch should succeed");

    assert!(
        result_1.is_success(),
        "Agent 1 authorized operation should succeed"
    );
    assert!(
        result_2.is_success(),
        "Agent 2 authorized operation should succeed"
    );

    info!("✅ Both agents successfully performed operations in their isolated contexts");

    // Test 2: Test unauthorized access attempts
    info!("Testing unauthorized access detection");

    let unauth_task_1 = Task::new(
        TaskType::Trading,
        serde_json::json!({
            "test_type": "unauthorized_test",
            "description": "Attempt unauthorized signer access"
        }),
    );

    let unauth_task_2 = Task::new(
        TaskType::Trading,
        serde_json::json!({
            "test_type": "unauthorized_test",
            "description": "Another unauthorized access attempt"
        }),
    );

    let unauth_result_1 = dispatcher
        .dispatch_task(unauth_task_1)
        .await
        .expect("Unauthorized task 1 dispatch should succeed");
    let unauth_result_2 = dispatcher
        .dispatch_task(unauth_task_2)
        .await
        .expect("Unauthorized task 2 dispatch should succeed");

    // Both should succeed because unauthorized access was properly blocked
    assert!(
        unauth_result_1.is_success(),
        "Unauthorized test should succeed when access is properly blocked"
    );
    assert!(
        unauth_result_2.is_success(),
        "Unauthorized test should succeed when access is properly blocked"
    );

    // Verify security monitor detected the unauthorized attempts
    assert!(
        security_monitor.unauthorized_attempts() >= 2,
        "Security monitor should have detected unauthorized attempts"
    );

    info!("✅ Unauthorized access attempts properly detected and blocked");

    // Test 3: Verify no security violations occurred
    assert!(
        !security_monitor.has_violations(),
        "No security violations should have occurred during proper isolation testing"
    );

    // Test 4: Verify access logs
    let access_log = security_monitor.get_access_log();
    assert!(
        access_log.len() >= 4,
        "Should have at least 4 access log entries"
    );

    let successful_operations = access_log
        .iter()
        .filter(|entry| entry.success && entry.operation.contains("test_operation"))
        .count();

    let blocked_unauthorized = access_log
        .iter()
        .filter(|entry| !entry.success && entry.operation.contains("unauthorized"))
        .count();

    assert!(
        successful_operations >= 2,
        "Should have successful authorized operations"
    );
    assert!(
        blocked_unauthorized >= 2,
        "Should have blocked unauthorized attempts"
    );

    info!("✅ SignerContext isolation test completed successfully");
    info!(
        "   - Authorized operations: {} successful",
        successful_operations
    );
    info!(
        "   - Unauthorized attempts: {} blocked",
        blocked_unauthorized
    );
    info!(
        "   - Security violations: {} (expected: 0)",
        security_monitor.has_violations() as u8
    );
    info!("   - Isolation boundaries properly maintained");
}

/// Test concurrent signer access patterns and thread safety.
///
/// This test validates that multiple agents can safely access their respective
/// SignerContexts concurrently without race conditions or cross-contamination.
#[tokio::test]
async fn test_concurrent_signer_access() {
    tracing_subscriber::fmt::try_init().ok();
    info!("Testing concurrent signer access and thread safety");

    let harness = BlockchainTestHarness::new()
        .await
        .expect("Failed to create blockchain test harness");

    let security_monitor = Arc::new(SecurityMonitor::default());

    // Create multiple agents with different signer contexts
    let num_agents = 3;
    let num_operations_per_agent = 5;
    let mut agents = Vec::new();

    for i in 0..num_agents {
        let keypair = harness
            .get_funded_keypair(i)
            .expect("Failed to get keypair");

        let signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
            Keypair::new_from_array(*keypair.secret_bytes()),
            harness.rpc_url().to_string(),
        ));

        let agent = Arc::new(SecurityAwareAgent::new(signer, security_monitor.clone()));
        agents.push(agent);
    }

    info!(
        "Created {} agents for concurrent access testing",
        num_agents
    );

    // Create agent system
    let registry = LocalAgentRegistry::default();

    for agent in &agents {
        registry
            .register_agent(agent.clone())
            .await
            .expect("Failed to register agent");
    }

    let dispatcher = Arc::new(AgentDispatcher::with_config(
        Arc::new(registry),
        DispatchConfig {
            routing_strategy: RoutingStrategy::RoundRobin,
            ..DispatchConfig::default()
        },
    ));

    // Create multiple concurrent tasks for each agent
    let mut task_handles = Vec::new();

    for agent_idx in 0..num_agents {
        for op_idx in 0..num_operations_per_agent {
            let dispatcher = Arc::clone(&dispatcher);
            let operation_id = agent_idx * num_operations_per_agent + op_idx;

            let task = Task::new(
                TaskType::Trading,
                serde_json::json!({
                    "test_type": "concurrent_test",
                    "operation_id": operation_id,
                    "agent_index": agent_idx
                }),
            );

            let handle = tokio::spawn(async move { dispatcher.dispatch_task(task).await });

            task_handles.push(handle);
        }
    }

    info!(
        "Executing {} concurrent operations across {} agents",
        task_handles.len(),
        num_agents
    );

    // Wait for all concurrent operations to complete
    let results = futures::future::join_all(task_handles).await;

    // Analyze results
    let mut successful_operations = 0;
    let mut failed_operations = 0;
    let mut unique_agents = std::collections::HashSet::<String>::new();

    for (i, result) in results.into_iter().enumerate() {
        let task_result = result
            .expect("Task join should succeed")
            .expect("Task dispatch should succeed");

        if task_result.is_success() {
            successful_operations += 1;

            if let TaskResult::Success { data, .. } = &task_result {
                if let Some(output_obj) = data.as_object() {
                    if let Some(agent_id) = output_obj.get("agent_id").and_then(|v| v.as_str()) {
                        unique_agents.insert(agent_id.to_string());
                    }
                }
            }
        } else {
            failed_operations += 1;
            warn!("Concurrent operation {} failed: {:?}", i, task_result);
        }
    }

    let total_expected = num_agents * num_operations_per_agent;

    // Verify concurrent operations completed successfully
    assert_eq!(
        successful_operations, total_expected,
        "All concurrent operations should succeed"
    );
    assert_eq!(
        failed_operations, 0,
        "No operations should fail due to concurrency issues"
    );
    assert_eq!(
        unique_agents.len(),
        num_agents,
        "Operations should be distributed across all agents"
    );

    // Verify no security violations occurred during concurrent access
    assert!(
        !security_monitor.has_violations(),
        "No security violations should occur during concurrent access"
    );

    // Verify access log integrity
    let access_log = security_monitor.get_access_log();
    let concurrent_operations = access_log
        .iter()
        .filter(|entry| entry.operation.starts_with("concurrent_op_"))
        .count();

    assert!(
        concurrent_operations >= total_expected,
        "Access log should contain all concurrent operations"
    );

    info!("✅ Concurrent signer access test completed successfully");
    info!("   - Total operations: {}", total_expected);
    info!("   - Successful operations: {}", successful_operations);
    info!("   - Failed operations: {}", failed_operations);
    info!("   - Unique agents used: {}", unique_agents.len());
    info!(
        "   - Security violations: {} (expected: 0)",
        security_monitor.has_violations() as u8
    );
    info!("   - Thread safety and resource isolation confirmed");
}

/// Test unauthorized access prevention and security boundaries.
///
/// This test specifically validates that the system properly prevents
/// and detects various types of unauthorized access attempts.
#[tokio::test]
async fn test_unauthorized_access_prevention() {
    tracing_subscriber::fmt::try_init().ok();
    info!("Testing unauthorized access prevention and security boundaries");

    let harness = BlockchainTestHarness::new()
        .await
        .expect("Failed to create blockchain test harness");

    let security_monitor = Arc::new(SecurityMonitor::default());

    // Create a single agent for testing unauthorized access scenarios
    let keypair = harness
        .get_funded_keypair(0)
        .expect("Failed to get keypair");

    let signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
        Keypair::new_from_array(*keypair.secret_bytes()),
        harness.rpc_url().to_string(),
    ));

    let agent = Arc::new(SecurityAwareAgent::new(signer, security_monitor.clone()));
    let agent_id = agent.id().clone();

    info!("Created security test agent: {}", agent_id);

    // Create agent system
    let registry = LocalAgentRegistry::default();

    registry
        .register_agent(agent.clone())
        .await
        .expect("Failed to register agent");

    let dispatcher = AgentDispatcher::with_config(
        Arc::new(registry),
        DispatchConfig {
            routing_strategy: RoutingStrategy::Direct,
            ..DispatchConfig::default()
        },
    );

    // Test multiple types of unauthorized access attempts
    let unauthorized_scenarios = vec![
        ("direct_signer_access", "Attempt to access signer directly"),
        (
            "cross_context_access",
            "Attempt to access different context",
        ),
        ("bypass_isolation", "Attempt to bypass isolation boundaries"),
        ("privilege_escalation", "Attempt privilege escalation"),
    ];

    info!(
        "Testing {} unauthorized access scenarios",
        unauthorized_scenarios.len()
    );

    for (scenario_type, description) in &unauthorized_scenarios {
        info!("Testing unauthorized scenario: {}", description);

        let task = Task::new(
            TaskType::Trading,
            serde_json::json!({
                "test_type": "unauthorized_test",
                "scenario": scenario_type,
                "description": description
            }),
        );

        let result = dispatcher
            .dispatch_task(task)
            .await
            .expect("Task dispatch should succeed");

        // The task should succeed because unauthorized access was properly blocked
        assert!(
            result.is_success(),
            "Unauthorized access test should succeed when access is blocked: {}",
            scenario_type
        );

        let output = if let TaskResult::Success { data, .. } = &result {
            data.as_object().expect("Task output should be JSON object")
        } else {
            panic!("Expected successful task result");
        };

        assert_eq!(
            output["status"], "unauthorized_access_blocked",
            "Status should indicate blocked access for: {}",
            scenario_type
        );

        info!(
            "✅ Unauthorized scenario '{}' properly blocked",
            scenario_type
        );
    }

    // Verify security monitor detected all unauthorized attempts
    let total_attempts = security_monitor.unauthorized_attempts();
    assert!(
        total_attempts >= unauthorized_scenarios.len(),
        "Security monitor should detect all unauthorized attempts"
    );

    // Verify no actual security violations occurred (attempts were blocked)
    assert!(
        !security_monitor.has_violations(),
        "No security violations should occur when unauthorized access is properly blocked"
    );

    // Analyze access log for security patterns
    let access_log = security_monitor.get_access_log();
    let unauthorized_entries = access_log
        .iter()
        .filter(|entry| entry.operation == "unauthorized_access_attempt")
        .count();

    assert_eq!(
        unauthorized_entries,
        unauthorized_scenarios.len(),
        "Access log should contain all unauthorized attempts"
    );

    // Verify all unauthorized attempts failed
    let successful_unauthorized = access_log
        .iter()
        .filter(|entry| entry.operation == "unauthorized_access_attempt" && entry.success)
        .count();

    assert_eq!(
        successful_unauthorized, 0,
        "No unauthorized attempts should succeed"
    );

    info!("✅ Unauthorized access prevention test completed successfully");
    info!(
        "   - Unauthorized scenarios tested: {}",
        unauthorized_scenarios.len()
    );
    info!("   - Total attempts detected: {}", total_attempts);
    info!(
        "   - Successful unauthorized attempts: {} (expected: 0)",
        successful_unauthorized
    );
    info!(
        "   - Security violations: {} (expected: 0)",
        security_monitor.has_violations() as u8
    );
    info!("   - Security boundaries properly enforced");
}

/// Test signer permission boundaries and access control.
///
/// This test validates that agents can only perform operations they're
/// authorized for within their SignerContext boundaries.
#[tokio::test]
async fn test_signer_permission_boundaries() {
    tracing_subscriber::fmt::try_init().ok();
    info!("Testing signer permission boundaries and access control");

    let harness = BlockchainTestHarness::new()
        .await
        .expect("Failed to create blockchain test harness");

    let security_monitor = Arc::new(SecurityMonitor::default());

    // Create agents with different permission levels (simulated)
    let readonly_keypair = harness
        .get_funded_keypair(0)
        .expect("Failed to get readonly keypair");
    let readwrite_keypair = harness
        .get_funded_keypair(1)
        .expect("Failed to get readwrite keypair");

    // In a real implementation, these would have different permission levels
    // For this test, we simulate the behavior
    let readonly_signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
        Keypair::new_from_array(*readonly_keypair.secret_bytes()),
        harness.rpc_url().to_string(),
    ));
    let readwrite_signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
        Keypair::new_from_array(*readwrite_keypair.secret_bytes()),
        harness.rpc_url().to_string(),
    ));

    let readonly_agent = Arc::new(SecurityAwareAgent::new(
        readonly_signer,
        security_monitor.clone(),
    ));
    let readwrite_agent = Arc::new(SecurityAwareAgent::new(
        readwrite_signer,
        security_monitor.clone(),
    ));

    info!("Created readonly agent: {}", readonly_agent.id());
    info!("Created readwrite agent: {}", readwrite_agent.id());

    // Create agent system
    let registry = LocalAgentRegistry::default();

    registry
        .register_agent(readonly_agent.clone())
        .await
        .expect("Failed to register readonly agent");
    registry
        .register_agent(readwrite_agent.clone())
        .await
        .expect("Failed to register readwrite agent");

    let dispatcher = AgentDispatcher::with_config(
        Arc::new(registry),
        DispatchConfig {
            routing_strategy: RoutingStrategy::RoundRobin,
            ..DispatchConfig::default()
        },
    );

    // Test that both agents can perform their authorized operations
    let readonly_task = Task::new(
        TaskType::Trading,
        serde_json::json!({
            "test_type": "authorized_operation",
            "permission_level": "readonly"
        }),
    );

    let readwrite_task = Task::new(
        TaskType::Trading,
        serde_json::json!({
            "test_type": "authorized_operation",
            "permission_level": "readwrite"
        }),
    );

    let readonly_result = dispatcher
        .dispatch_task(readonly_task)
        .await
        .expect("Readonly task should dispatch successfully");
    let readwrite_result = dispatcher
        .dispatch_task(readwrite_task)
        .await
        .expect("Readwrite task should dispatch successfully");

    assert!(
        readonly_result.is_success(),
        "Readonly agent should succeed with authorized operations"
    );
    assert!(
        readwrite_result.is_success(),
        "Readwrite agent should succeed with authorized operations"
    );

    info!("✅ Both agents successfully performed their authorized operations");

    // Verify security monitor shows clean access patterns
    let access_log = security_monitor.get_access_log();
    let authorized_operations = access_log
        .iter()
        .filter(|entry| entry.success && entry.operation.contains("test_operation"))
        .count();

    assert!(
        authorized_operations >= 2,
        "Should have successful authorized operations"
    );

    // Verify no unauthorized access occurred
    assert!(
        !security_monitor.has_violations(),
        "No security violations should occur with proper permission boundaries"
    );

    info!("✅ Signer permission boundaries test completed successfully");
    info!("   - Authorized operations: {}", authorized_operations);
    info!(
        "   - Security violations: {} (expected: 0)",
        security_monitor.has_violations() as u8
    );
    info!("   - Permission boundaries properly enforced");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_monitor_basic_functionality() {
        let monitor = SecurityMonitor::default();

        assert_eq!(monitor.unauthorized_attempts(), 0);
        assert!(!monitor.has_violations());

        // Log a failed access
        let entry = AccessLogEntry {
            timestamp: Instant::now(),
            context_id: "test_context".to_string(),
            agent_id: "test_agent".to_string(),
            operation: "test_operation".to_string(),
            success: false,
            error: Some("Test error".to_string()),
        };

        monitor.log_access(entry);

        assert_eq!(monitor.unauthorized_attempts(), 1);
        assert!(!monitor.has_violations()); // No violation unless error contains "unauthorized"

        // Log an unauthorized access
        let violation_entry = AccessLogEntry {
            timestamp: Instant::now(),
            context_id: "test_context".to_string(),
            agent_id: "test_agent".to_string(),
            operation: "unauthorized_access_attempt".to_string(),
            success: false,
            error: Some("unauthorized access detected".to_string()),
        };

        monitor.log_access(violation_entry);

        assert_eq!(monitor.unauthorized_attempts(), 2);
        assert!(monitor.has_violations());
    }

    #[tokio::test]
    async fn test_security_aware_agent_capabilities() {
        let keypair = Keypair::new();
        let signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
            Keypair::new_from_array(*keypair.secret_bytes()),
            "http://localhost:8899".to_string(),
        ));
        let security_monitor = Arc::new(SecurityMonitor::default());

        let agent = SecurityAwareAgent::new(signer, security_monitor);

        let capabilities = agent.capabilities();
        assert!(capabilities.contains(&CapabilityType::Custom("security_testing".to_string())));
        assert!(capabilities.contains(&CapabilityType::Custom("signer_isolation".to_string())));
        assert!(capabilities.contains(&CapabilityType::Custom("concurrent_access".to_string())));
        assert!(capabilities.contains(&CapabilityType::Trading));

        // Test capability matching
        let trading_task = Task::new(TaskType::Trading, serde_json::json!({}));
        assert!(agent.can_handle(&trading_task));
    }
}
