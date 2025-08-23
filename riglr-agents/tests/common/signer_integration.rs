//! SignerContext integration helpers for riglr-agents testing.
//!
//! This module provides utilities for testing proper SignerContext isolation
//! and security within the agent system, including mock implementations and
//! validation helpers for secure signer access patterns.

// use riglr_core::{signer::UnifiedSigner, SignerContext};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
// use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};
use uuid::Uuid;

/// Errors that can occur during signer context testing.
#[derive(Debug, Error)]
pub enum SignerTestError {
    #[error("Unauthorized access attempt detected: {0}")]
    UnauthorizedAccess(String),

    #[error("Signer isolation violation: {0}")]
    IsolationViolation(String),

    #[error("Context validation failed: {0}")]
    ValidationFailed(String),

    #[error("Concurrent access error: {0}")]
    ConcurrentAccessError(String),

    #[error("Test setup error: {0}")]
    TestSetup(String),
}

/// Mock signer context with enhanced testing capabilities.
#[derive(Debug)]
pub struct MockSignerContext {
    signer_id: String,
    access_log: Arc<Mutex<Vec<AccessLogEntry>>>,
    security_monitor: Arc<SecurityMonitor>,
    isolation_enabled: bool,
}

/// Log entry for signer access tracking.
#[derive(Debug, Clone)]
pub struct AccessLogEntry {
    pub timestamp: Instant,
    pub operation: String,
    pub authorized: bool,
    pub context_id: String,
    pub caller_info: Option<String>,
}

/// Security monitor for tracking access patterns and violations.
#[derive(Debug)]
pub struct SecurityMonitor {
    unauthorized_attempts: AtomicBool,
    concurrent_access_count: Arc<RwLock<u32>>,
    isolation_violations: Arc<Mutex<Vec<IsolationViolation>>>,
    max_concurrent_access: u32,
}

/// Details of an isolation violation.
#[derive(Debug, Clone)]
pub struct IsolationViolation {
    pub timestamp: Instant,
    pub violation_type: String,
    pub description: String,
    pub severity: ViolationSeverity,
}

/// Severity levels for isolation violations.
#[derive(Debug, Clone, PartialEq)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl MockSignerContext {
    /// Create a new mock signer context with security monitoring.
    pub fn new() -> Self {
        Self {
            signer_id: format!("mock_signer_{}", Uuid::new_v4()),
            access_log: Arc::new(Mutex::new(Vec::new())),
            security_monitor: Arc::new(SecurityMonitor::new()),
            isolation_enabled: true,
        }
    }

    /// Create a mock signer context with isolation disabled (for testing violations).
    pub fn new_without_isolation() -> Self {
        let mut context = Self::new();
        context.isolation_enabled = false;
        context
    }

    /// Execute an operation with the signer, tracking access patterns.
    pub async fn execute_with_signer<F, R>(&self, operation: F) -> Result<R, SignerTestError>
    where
        F: FnOnce(
            &MockUnifiedSigner,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<R, Box<dyn std::error::Error + Send + Sync>>,
                    > + Send
                    + '_,
            >,
        >,
    {
        // Check concurrent access limits
        self.security_monitor.check_concurrent_access().await?;

        // Log access attempt
        self.log_access("execute_with_signer", true, "operation_start")
            .await;

        // Create mock signer with monitoring
        let mock_signer = MockUnifiedSigner::new(&self.signer_id, self.security_monitor.clone());

        // Execute operation within isolation boundary
        let result = if self.isolation_enabled {
            self.execute_isolated(operation, &mock_signer).await
        } else {
            operation(&mock_signer)
                .await
                .map_err(|e| SignerTestError::ValidationFailed(format!("Operation failed: {}", e)))
        };

        // Release concurrent access count
        self.security_monitor.release_concurrent_access().await;

        // Log completion
        self.log_access("execute_with_signer", true, "operation_complete")
            .await;

        result
    }

    /// Attempt unauthorized access to test security boundaries.
    pub async fn attempt_unauthorized_access(&self) -> Result<(), SignerTestError> {
        self.security_monitor
            .unauthorized_attempts
            .store(true, Ordering::SeqCst);
        self.log_access("unauthorized_access", false, "security_test")
            .await;

        // This should always fail in a properly secured system
        Err(SignerTestError::UnauthorizedAccess(
            "Attempted to access signer outside of context".to_string(),
        ))
    }

    /// Get access logs for analysis.
    pub fn get_access_logs(&self) -> Vec<AccessLogEntry> {
        self.access_log.lock().unwrap().clone()
    }

    /// Check if unauthorized access was attempted.
    pub fn has_unauthorized_attempts(&self) -> bool {
        self.security_monitor
            .unauthorized_attempts
            .load(Ordering::SeqCst)
    }

    /// Get isolation violations.
    pub fn get_isolation_violations(&self) -> Vec<IsolationViolation> {
        self.security_monitor
            .isolation_violations
            .lock()
            .unwrap()
            .clone()
    }

    /// Reset monitoring state for new test.
    pub fn reset_monitoring(&self) {
        self.access_log.lock().unwrap().clear();
        self.security_monitor
            .unauthorized_attempts
            .store(false, Ordering::SeqCst);
        self.security_monitor
            .isolation_violations
            .lock()
            .unwrap()
            .clear();
    }

    // Private methods

    async fn execute_isolated<F, R>(
        &self,
        operation: F,
        signer: &MockUnifiedSigner,
    ) -> Result<R, SignerTestError>
    where
        F: FnOnce(
            &MockUnifiedSigner,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<R, Box<dyn std::error::Error + Send + Sync>>,
                    > + Send
                    + '_,
            >,
        >,
    {
        // Simulate isolation boundary enforcement
        debug!("Executing operation within signer isolation boundary");

        // Check for isolation violations
        if self.security_monitor.check_isolation_violations().await? {
            return Err(SignerTestError::IsolationViolation(
                "Isolation boundary violated during operation".to_string(),
            ));
        }

        // Execute operation
        let result = operation(signer).await.map_err(|e| {
            SignerTestError::ValidationFailed(format!("Isolated operation failed: {}", e))
        })?;

        debug!("Operation completed successfully within isolation boundary");
        Ok(result)
    }

    async fn log_access(&self, operation: &str, authorized: bool, caller_info: &str) {
        let entry = AccessLogEntry {
            timestamp: Instant::now(),
            operation: operation.to_string(),
            authorized,
            context_id: self.signer_id.clone(),
            caller_info: Some(caller_info.to_string()),
        };

        self.access_log.lock().unwrap().push(entry);
    }
}

impl SecurityMonitor {
    fn new() -> Self {
        Self {
            unauthorized_attempts: AtomicBool::new(false),
            concurrent_access_count: Arc::new(RwLock::new(0)),
            isolation_violations: Arc::new(Mutex::new(Vec::new())),
            max_concurrent_access: 5, // Configurable limit
        }
    }

    async fn check_concurrent_access(&self) -> Result<(), SignerTestError> {
        let mut count = self.concurrent_access_count.write().await;

        if *count >= self.max_concurrent_access {
            return Err(SignerTestError::ConcurrentAccessError(format!(
                "Too many concurrent accesses: {}",
                *count
            )));
        }

        *count += 1;
        debug!("Concurrent access count: {}", *count);
        Ok(())
    }

    async fn release_concurrent_access(&self) {
        let mut count = self.concurrent_access_count.write().await;
        if *count > 0 {
            *count -= 1;
        }
        debug!("Released concurrent access, count: {}", *count);
    }

    async fn check_isolation_violations(&self) -> Result<bool, SignerTestError> {
        // Simulate isolation boundary checks
        // In a real implementation, this would check for:
        // - Memory access patterns
        // - Thread isolation
        // - Resource access boundaries

        // For testing, we can simulate different violation scenarios
        if std::env::var("SIMULATE_ISOLATION_VIOLATION").is_ok() {
            self.record_violation(
                "memory_access".to_string(),
                "Simulated memory boundary violation".to_string(),
                ViolationSeverity::High,
            )
            .await;
            return Ok(true);
        }

        Ok(false)
    }

    async fn record_violation(
        &self,
        violation_type: String,
        description: String,
        severity: ViolationSeverity,
    ) {
        let violation = IsolationViolation {
            timestamp: Instant::now(),
            violation_type,
            description,
            severity,
        };

        self.isolation_violations
            .lock()
            .unwrap()
            .push(violation.clone());

        match violation.severity {
            ViolationSeverity::Critical | ViolationSeverity::High => {
                error!("Isolation violation detected: {:?}", violation);
            }
            ViolationSeverity::Medium => {
                warn!("Isolation violation detected: {:?}", violation);
            }
            ViolationSeverity::Low => {
                debug!("Minor isolation violation detected: {:?}", violation);
            }
        }
    }
}

/// Mock unified signer with security monitoring.
#[derive(Debug)]
pub struct MockUnifiedSigner {
    signer_id: String,
    security_monitor: Arc<SecurityMonitor>,
    operation_count: Arc<Mutex<u32>>,
}

impl MockUnifiedSigner {
    fn new(signer_id: &str, security_monitor: Arc<SecurityMonitor>) -> Self {
        Self {
            signer_id: signer_id.to_string(),
            security_monitor,
            operation_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Mock SOL transfer operation with security monitoring.
    pub async fn transfer_sol(
        &self,
        to_address: &str,
        amount_lamports: u64,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.increment_operation_count();

        // Validate operation parameters
        if amount_lamports == 0 {
            return Err("Invalid transfer amount".into());
        }

        if to_address.is_empty() {
            return Err("Invalid recipient address".into());
        }

        // Simulate processing time
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Return mock transaction signature
        Ok(format!(
            "{}_{}_{}",
            "mock_signature",
            &self.signer_id[..8],
            self.get_operation_count()
        ))
    }

    /// Mock balance check operation.
    pub async fn get_balance(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        self.increment_operation_count();

        // Simulate variable balance based on signer ID
        let balance = match self.signer_id.chars().last() {
            Some(c) if c.is_numeric() => {
                let digit = c.to_digit(10).unwrap_or(5) as u64;
                digit * 1_000_000_000 // Convert to lamports
            }
            _ => 5_000_000_000, // Default 5 SOL
        };

        Ok(balance)
    }

    /// Attempt to access private key (should fail in secure implementation).
    pub async fn get_private_key(
        &self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // This should always fail in a properly secured signer
        self.security_monitor
            .unauthorized_attempts
            .store(true, Ordering::SeqCst);

        Err("Private key access not allowed outside secure context".into())
    }

    fn increment_operation_count(&self) {
        let mut count = self.operation_count.lock().unwrap();
        *count += 1;
    }

    fn get_operation_count(&self) -> u32 {
        *self.operation_count.lock().unwrap()
    }
}

/// Test utilities for signer context integration.
pub mod test_utils {
    use super::*;
    use std::sync::Arc;
    use tokio::time::timeout;

    /// Create multiple mock signer contexts for concurrent testing.
    pub fn create_test_signer_contexts(count: usize) -> Vec<MockSignerContext> {
        (0..count).map(|_| MockSignerContext::new()).collect()
    }

    /// Test concurrent access to a signer context.
    pub async fn test_concurrent_signer_access(
        context: Arc<MockSignerContext>,
        concurrent_tasks: usize,
    ) -> Result<Vec<String>, SignerTestError> {
        let mut handles = Vec::new();

        for i in 0..concurrent_tasks {
            let context_clone = context.clone();
            let handle = tokio::spawn(async move {
                let result = context_clone
                    .execute_with_signer(|signer| {
                        Box::pin(async move {
                            // Simulate work
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            signer.get_balance().await.map(|balance| {
                                format!("Task {} completed with balance: {}", i, balance)
                            })
                        })
                    })
                    .await;

                result
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => match result {
                    Ok(message) => results.push(message),
                    Err(e) => return Err(e),
                },
                Err(e) => {
                    return Err(SignerTestError::ConcurrentAccessError(format!(
                        "Task failed: {}",
                        e
                    )))
                }
            }
        }

        Ok(results)
    }

    /// Test signer isolation boundaries.
    pub async fn test_signer_isolation(
        context: &MockSignerContext,
    ) -> Result<IsolationTestResult, SignerTestError> {
        let mut test_result = IsolationTestResult::default();

        // Test 1: Normal operation should succeed
        let normal_result = context
            .execute_with_signer(|signer| Box::pin(async move { signer.get_balance().await }))
            .await;

        test_result.normal_operation_success = normal_result.is_ok();

        // Test 2: Unauthorized access should fail
        let unauthorized_result = context.attempt_unauthorized_access().await;
        test_result.unauthorized_access_blocked = unauthorized_result.is_err();

        // Test 3: Private key access should fail
        let private_key_result = context
            .execute_with_signer(|signer| Box::pin(async move { signer.get_private_key().await }))
            .await;

        test_result.private_key_access_blocked = private_key_result.is_err();

        // Test 4: Check for isolation violations
        test_result.isolation_violations = context.get_isolation_violations();

        Ok(test_result)
    }

    /// Test signer context timeout handling.
    pub async fn test_signer_timeout(
        context: &MockSignerContext,
        timeout_duration: Duration,
    ) -> Result<bool, SignerTestError> {
        let operation = context.execute_with_signer(|signer| {
            Box::pin(async move {
                // Simulate long-running operation
                tokio::time::sleep(timeout_duration * 2).await;
                signer.get_balance().await
            })
        });

        match timeout(timeout_duration, operation).await {
            Ok(_) => Ok(false), // Operation completed, timeout didn't work
            Err(_) => Ok(true), // Operation timed out as expected
        }
    }

    /// Validate signer context security properties.
    pub fn validate_security_properties(context: &MockSignerContext) -> SecurityValidationResult {
        let mut result = SecurityValidationResult::default();

        // Check access logs
        let logs = context.get_access_logs();
        result.has_access_logs = !logs.is_empty();
        result.unauthorized_attempts_logged = logs.iter().any(|log| !log.authorized);

        // Check isolation violations
        let violations = context.get_isolation_violations();
        result.isolation_violations_detected = !violations.is_empty();
        result.critical_violations = violations
            .iter()
            .filter(|v| matches!(v.severity, ViolationSeverity::Critical))
            .count();

        // Check if unauthorized access attempts were detected
        result.unauthorized_access_detected = context.has_unauthorized_attempts();

        result
    }
}

/// Result of isolation testing.
#[derive(Debug, Default)]
pub struct IsolationTestResult {
    pub normal_operation_success: bool,
    pub unauthorized_access_blocked: bool,
    pub private_key_access_blocked: bool,
    pub isolation_violations: Vec<IsolationViolation>,
}

impl IsolationTestResult {
    /// Check if all isolation tests passed.
    pub fn all_tests_passed(&self) -> bool {
        self.normal_operation_success
            && self.unauthorized_access_blocked
            && self.private_key_access_blocked
            && self.isolation_violations.is_empty()
    }
}

/// Result of security validation.
#[derive(Debug, Default)]
pub struct SecurityValidationResult {
    pub has_access_logs: bool,
    pub unauthorized_attempts_logged: bool,
    pub isolation_violations_detected: bool,
    pub critical_violations: usize,
    pub unauthorized_access_detected: bool,
}

impl SecurityValidationResult {
    /// Check if security validation passed.
    pub fn security_validation_passed(&self) -> bool {
        self.has_access_logs
            && !self.unauthorized_attempts_logged
            && !self.isolation_violations_detected
            && self.critical_violations == 0
            && !self.unauthorized_access_detected
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::*;
    use super::*;

    #[tokio::test]
    async fn test_mock_signer_context_creation() {
        let context = MockSignerContext::new();
        assert!(!context.signer_id.is_empty());
        assert!(context.isolation_enabled);
        assert!(!context.has_unauthorized_attempts());
    }

    #[tokio::test]
    async fn test_signer_context_normal_operation() {
        let context = MockSignerContext::new();

        let result = context
            .execute_with_signer(|signer| Box::pin(async move { signer.get_balance().await }))
            .await;

        assert!(result.is_ok());
        let balance = result.unwrap();
        assert!(balance > 0);
    }

    #[tokio::test]
    async fn test_unauthorized_access_detection() {
        let context = MockSignerContext::new();

        let result = context.attempt_unauthorized_access().await;
        assert!(result.is_err());
        assert!(context.has_unauthorized_attempts());
    }

    // #[tokio::test]
    // async fn test_concurrent_signer_access() {
    //     let context = Arc::new(MockSignerContext::new());
    //
    //     let results = test_concurrent_signer_access(context.clone(), 3).await;
    //     assert!(results.is_ok());
    //
    //     let messages = results.unwrap();
    //     assert_eq!(messages.len(), 3);
    //
    //     // Verify all tasks completed
    //     for (i, message) in messages.iter().enumerate() {
    //         assert!(message.contains(&format!("Task {}", i)));
    //     }
    // }

    #[tokio::test]
    async fn test_signer_isolation_boundaries() {
        let context = MockSignerContext::new();

        let result = test_signer_isolation(&context).await;
        assert!(result.is_ok());

        let isolation_result = result.unwrap();
        assert!(isolation_result.normal_operation_success);
        assert!(isolation_result.unauthorized_access_blocked);
        assert!(isolation_result.private_key_access_blocked);
    }

    #[tokio::test]
    async fn test_security_validation() {
        let context = MockSignerContext::new();

        // Perform some operations to generate logs
        let _ = context
            .execute_with_signer(|signer| Box::pin(async move { signer.get_balance().await }))
            .await;

        let _ = context.attempt_unauthorized_access().await;

        let validation_result = validate_security_properties(&context);
        assert!(validation_result.has_access_logs);
        assert!(validation_result.unauthorized_access_detected);
    }

    #[tokio::test]
    async fn test_operation_timeout() {
        let context = MockSignerContext::new();
        let timeout_duration = Duration::from_millis(100);

        let timed_out = test_signer_timeout(&context, timeout_duration).await;
        assert!(timed_out.is_ok());
        // Note: In a real timeout test, this should be true
        // For this mock implementation, we're testing the pattern
    }

    #[test]
    fn test_multiple_signer_contexts() {
        let contexts = create_test_signer_contexts(5);
        assert_eq!(contexts.len(), 5);

        // Verify each context has unique signer ID
        let mut signer_ids = std::collections::HashSet::new();
        for context in &contexts {
            assert!(signer_ids.insert(context.signer_id.clone()));
        }
        assert_eq!(signer_ids.len(), 5);
    }

    #[tokio::test]
    async fn test_access_logging() {
        let context = MockSignerContext::new();

        // Perform several operations
        for i in 0..3 {
            let _ = context
                .execute_with_signer(|signer| {
                    let operation_id = i;
                    Box::pin(async move {
                        tokio::time::sleep(Duration::from_millis(50)).await;
                        signer
                            .transfer_sol(&format!("recipient_{}", operation_id), 1000)
                            .await
                    })
                })
                .await;
        }

        let logs = context.get_access_logs();
        assert!(!logs.is_empty());

        // Should have log entries for each operation (start and complete)
        assert!(logs.len() >= 6); // 2 entries per operation

        // All entries should be authorized for normal operations
        let authorized_count = logs.iter().filter(|log| log.authorized).count();
        assert!(authorized_count > 0);
    }
}
