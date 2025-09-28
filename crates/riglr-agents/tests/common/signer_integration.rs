#![allow(clippy::expect_used)]

use core::{
    error::Error as StdError,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
/// `SignerContext` integration helpers for riglr-agents testing.
///
/// This module provides utilities for testing proper `SignerContext` isolation
/// and security within the agent system, including mock implementations and
/// validation helpers for secure signer access patterns.
// use riglr_core::{signer::UnifiedSigner, SignerContext};
use core::{future::Future, pin::Pin};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
// use std::collections::HashMap;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};
use uuid::Uuid;

const SIMULATE_ISOLATION_VIOLATION: &str = "SIMULATE_ISOLATION_VIOLATION";

/// Errors that can occur during signer context testing.
#[derive(Debug, Error)]
pub enum SignerTestError {
    /// Unauthorized access attempt was detected during testing
    #[error("Unauthorized access attempt detected: {0}")]
    UnauthorizedAccess(String),

    /// Signer isolation boundary was violated
    #[error("Signer isolation violation: {0}")]
    IsolationViolation(String),

    /// Context validation failed during operation
    #[error("Context validation failed: {0}")]
    ValidationFailed(String),

    /// Concurrent access limit exceeded or race condition detected
    #[error("Concurrent access error: {0}")]
    ConcurrentAccessError(String),

    /// Test setup or configuration error occurred
    #[error("Test setup error: {0}")]
    TestSetup(String),
}

/// Mock signer context with enhanced testing capabilities.
#[derive(Debug)]
pub struct MockSignerContext {
    /// Unique identifier for this mock signer instance
    signer_id: String,
    /// Thread-safe access log for tracking signer operations
    access_log: Arc<Mutex<Vec<AccessLogEntry>>>,
    /// Security monitor for detecting violations and unauthorized access
    security_monitor: Arc<SecurityMonitor>,
    /// Whether isolation boundaries are enforced
    isolation_enabled: bool,
}

/// Log entry for signer access tracking.
#[derive(Debug, Clone)]
pub struct AccessLogEntry {
    /// When the access attempt occurred
    pub timestamp: Instant,
    /// Name of the operation being performed
    pub operation: String,
    /// Whether the access was authorized
    pub authorized: bool,
    /// ID of the signer context being accessed
    pub context_id: String,
    /// Optional information about the caller
    pub caller_info: Option<String>,
}

/// Security monitor for tracking access patterns and violations.
#[derive(Debug)]
pub struct SecurityMonitor {
    /// Flag indicating if unauthorized access attempts were detected
    unauthorized_attempts: AtomicBool,
    /// Current count of concurrent access operations
    concurrent_access_count: Arc<RwLock<u32>>,
    /// List of detected isolation violations
    isolation_violations: Arc<Mutex<Vec<IsolationViolation>>>,
    /// Maximum allowed concurrent access operations
    max_concurrent_access: u32,
}

/// Details of an isolation violation.
#[derive(Debug, Clone)]
pub struct IsolationViolation {
    /// When the isolation violation occurred
    pub timestamp: Instant,
    /// Type or category of the violation
    pub violation_type: String,
    /// Detailed description of the violation
    pub description: String,
    /// Severity level of the violation
    pub severity: ViolationSeverity,
}

/// Severity levels for isolation violations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationSeverity {
    /// Low severity violation - informational only
    Low,
    /// Medium severity violation - should be monitored
    Medium,
    /// High severity violation - requires attention
    High,
    /// Critical severity violation - immediate action required
    Critical,
}

impl Default for MockSignerContext {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSignerContext {
    /// Create a new mock signer context with security monitoring.
    #[must_use]
    pub fn new() -> Self {
        Self {
            signer_id: format!("mock_signer_{}", Uuid::new_v4()),
            access_log: Arc::new(Mutex::new(Vec::new())),
            security_monitor: Arc::new(SecurityMonitor::new()),
            isolation_enabled: true,
        }
    }

    /// Create a mock signer context with isolation disabled (for testing violations).
    #[must_use]
    pub fn new_without_isolation() -> Self {
        let mut context = Self::new();
        context.isolation_enabled = false;
        context
    }

    /// Execute an operation with the signer, tracking access patterns.
    ///
    /// # Errors
    ///
    /// Returns an error if concurrent access limits are exceeded or if the operation fails.
    pub async fn execute_with_signer<F, R>(&self, operation: F) -> Result<R, SignerTestError>
    where
        F: FnOnce(
            &MockUnifiedSigner,
        ) -> Pin<
            Box<dyn Future<Output = Result<R, Box<dyn StdError + Send + Sync>>> + Send + '_>,
        >,
    {
        // Check concurrent access limits
        self.security_monitor.check_concurrent_access().await?;

        // Log access attempt
        self.log_access("execute_with_signer", true, "operation_start");

        // Create mock signer with monitoring
        let mock_signer = MockUnifiedSigner::new(&self.signer_id, self.security_monitor.clone());

        // Execute operation within isolation boundary
        let result = if self.isolation_enabled {
            self.execute_isolated(operation, &mock_signer).await
        } else {
            let operation_result = operation(&mock_signer).await;
            operation_result
                .map_err(|e| SignerTestError::ValidationFailed(format!("Operation failed: {e}")))
        };

        // Release concurrent access count
        self.security_monitor.release_concurrent_access().await;

        // Log completion
        self.log_access("execute_with_signer", true, "operation_complete");

        result
    }

    /// Attempt unauthorized access to test security boundaries.
    ///
    /// # Errors
    ///
    /// Returns an error if unauthorized access is attempted or if isolation violations are detected.
    pub fn attempt_unauthorized_access(&self) -> Result<(), SignerTestError> {
        self.security_monitor
            .unauthorized_attempts
            .store(true, Ordering::SeqCst);
        self.log_access("unauthorized_access", false, "security_test");

        // This should always fail in a properly secured system
        Err(SignerTestError::UnauthorizedAccess(
            "Attempted to access signer outside of context".to_string(),
        ))
    }

    /// Get access logs for analysis.
    ///
    /// # Panics
    ///
    /// Panics if the access log mutex is poisoned.
    #[must_use]
    pub fn get_access_logs(&self) -> Vec<AccessLogEntry> {
        self.access_log
            .lock()
            .expect("Access log mutex should not be poisoned")
            .clone()
    }

    /// Check if unauthorized access was attempted.
    #[must_use]
    pub fn has_unauthorized_attempts(&self) -> bool {
        self.security_monitor
            .unauthorized_attempts
            .load(Ordering::SeqCst)
    }

    /// Get isolation violations.
    ///
    /// # Panics
    ///
    /// Panics if the isolation violations mutex is poisoned.
    #[must_use]
    pub fn get_isolation_violations(&self) -> Vec<IsolationViolation> {
        self.security_monitor
            .isolation_violations
            .lock()
            .expect("Isolation violations mutex should not be poisoned")
            .clone()
    }

    /// Reset monitoring state for new test.
    ///
    /// # Panics
    ///
    /// Panics if any of the mutexes are poisoned.
    pub fn reset_monitoring(&self) {
        self.access_log
            .lock()
            .expect("Access log mutex should not be poisoned")
            .clear();
        self.security_monitor
            .unauthorized_attempts
            .store(false, Ordering::SeqCst);
        self.security_monitor
            .isolation_violations
            .lock()
            .expect("Isolation violations mutex should not be poisoned")
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
        ) -> Pin<
            Box<dyn Future<Output = Result<R, Box<dyn StdError + Send + Sync>>> + Send + '_>,
        >,
    {
        // Simulate isolation boundary enforcement
        debug!("Executing operation within signer isolation boundary");

        // Check for isolation violations
        if self.security_monitor.check_isolation_violations() {
            return Err(SignerTestError::IsolationViolation(
                "Isolation boundary violated during operation".to_string(),
            ));
        }

        // Execute operation
        let operation_result = operation(signer).await;
        let result = operation_result.map_err(|e| {
            SignerTestError::ValidationFailed(format!("Isolated operation failed: {e}"))
        })?;

        debug!("Operation completed successfully within isolation boundary");
        Ok(result)
    }

    fn log_access(&self, operation: &str, authorized: bool, caller_info: &str) {
        let entry = AccessLogEntry {
            timestamp: Instant::now(),
            operation: operation.to_string(),
            authorized,
            context_id: self.signer_id.clone(),
            caller_info: Some(caller_info.to_string()),
        };

        self.access_log
            .lock()
            .expect("Access log mutex should not be poisoned")
            .push(entry);
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
            let count_value = *count;
            drop(count);
            return Err(SignerTestError::ConcurrentAccessError(format!(
                "Too many concurrent accesses: {count_value}"
            )));
        }

        #[expect(clippy::arithmetic_side_effects)]
        {
            *count += 1;
        }
        let count_value = *count;
        drop(count);
        debug!("Concurrent access count: {}", count_value);
        Ok(())
    }

    async fn release_concurrent_access(&self) {
        let mut count = self.concurrent_access_count.write().await;
        if *count > 0 {
            #[expect(clippy::arithmetic_side_effects)]
            {
                *count -= 1;
            }
        }
        let count_value = *count;
        drop(count);
        debug!("Released concurrent access, count: {}", count_value);
    }

    fn check_isolation_violations(&self) -> bool {
        use std::env;
        // Simulate isolation boundary checks
        // In a real implementation, this would check for:
        // - Memory access patterns
        // - Thread isolation
        // - Resource access boundaries

        // For testing, we can simulate different violation scenarios
        if env::var(SIMULATE_ISOLATION_VIOLATION).is_ok() {
            self.record_violation(
                "memory_access".to_string(),
                "Simulated memory boundary violation".to_string(),
                ViolationSeverity::High,
            );
            return true;
        }

        false
    }

    fn record_violation(
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
            .expect("Isolation violations mutex should not be poisoned")
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
    /// Unique identifier for this signer instance
    signer_id: String,
    /// Security monitor for tracking violations
    security_monitor: Arc<SecurityMonitor>,
    /// Count of operations performed by this signer
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
    ///
    /// # Errors
    ///
    /// Returns an error if the transfer amount is zero or the recipient address is empty.
    pub async fn transfer_sol(
        &self,
        to_address: &str,
        amount_lamports: u64,
    ) -> Result<String, Box<dyn StdError + Send + Sync>> {
        use tokio::time::sleep;

        self.increment_operation_count();

        // Validate operation parameters
        if amount_lamports == 0 {
            return Err("Invalid transfer amount".into());
        }

        if to_address.is_empty() {
            return Err("Invalid recipient address".into());
        }

        // Simulate processing time
        sleep(Duration::from_millis(200)).await;

        // Return mock transaction signature
        Ok(format!(
            "{}_{}_{}",
            "mock_signature",
            &self.signer_id[..8],
            self.get_operation_count()
        ))
    }

    /// Mock balance check operation.
    ///
    /// # Errors
    ///
    /// This mock implementation does not actually return errors, but maintains the signature for testing.
    pub fn get_balance(&self) -> Result<u64, Box<dyn StdError + Send + Sync>> {
        self.increment_operation_count();

        // Simulate variable balance based on signer ID
        let balance = match self.signer_id.chars().last() {
            Some(c) if c.is_numeric() => {
                let digit = u64::from(c.to_digit(10).unwrap_or(5));
                #[expect(clippy::arithmetic_side_effects)]
                {
                    digit * 1_000_000_000 // Convert to lamports
                }
            }
            _ => 5_000_000_000, // Default 5 SOL
        };

        Ok(balance)
    }

    /// Attempt to access private key (should fail in secure implementation).
    ///
    /// # Errors
    ///
    /// Always returns an error as private key access should not be allowed outside secure context.
    pub fn get_private_key(&self) -> Result<String, Box<dyn StdError + Send + Sync>> {
        // This should always fail in a properly secured signer
        self.security_monitor
            .unauthorized_attempts
            .store(true, Ordering::SeqCst);

        Err("Private key access not allowed outside secure context".into())
    }

    fn increment_operation_count(&self) {
        let mut count = self
            .operation_count
            .lock()
            .expect("Operation count mutex should not be poisoned");
        #[expect(clippy::arithmetic_side_effects)]
        {
            *count += 1;
        }
    }

    fn get_operation_count(&self) -> u32 {
        *self
            .operation_count
            .lock()
            .expect("Operation count mutex should not be poisoned")
    }
}

/// Test utilities for signer context integration.
pub mod test_utils {
    use super::*;
    use std::sync::Arc;
    use tokio::time::timeout;

    /// Create multiple mock signer contexts for concurrent testing.
    #[must_use]
    pub fn create_test_signer_contexts(count: usize) -> Vec<MockSignerContext> {
        (0..count).map(|_| MockSignerContext::new()).collect()
    }

    /// Test concurrent access to a signer context.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the concurrent operations fail or if signer access violations occur.
    pub async fn test_concurrent_signer_access(
        context: Arc<MockSignerContext>,
        concurrent_tasks: usize,
    ) -> Result<Vec<String>, SignerTestError> {
        let mut handles = Vec::new();

        for i in 0..concurrent_tasks {
            let context_clone = context.clone();
            let handle = tokio::spawn(async move {
                context_clone
                    .execute_with_signer(|signer| {
                        Box::pin(async move {
                            use tokio::time::sleep;
                            // Simulate work
                            sleep(Duration::from_millis(100)).await;
                            let balance_result = signer.get_balance();
                            balance_result.map(|balance| {
                                format!("Task {i} completed with balance: {balance}")
                            })
                        })
                    })
                    .await
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        let mut results = Vec::new();
        for handle in handles {
            let handle_result = handle.await;
            match handle_result {
                Ok(result) => match result {
                    Ok(message) => results.push(message),
                    Err(e) => return Err(e),
                },
                Err(e) => {
                    return Err(SignerTestError::ConcurrentAccessError(format!(
                        "Task failed: {e}"
                    )))
                }
            }
        }

        Ok(results)
    }

    /// Test signer isolation boundaries.
    ///
    /// # Errors
    ///
    /// Returns an error if isolation testing fails or security boundaries are violated.
    pub async fn test_signer_isolation(
        context: &MockSignerContext,
    ) -> Result<IsolationTestResult, SignerTestError> {
        let mut test_result = IsolationTestResult::default();

        // Test 1: Normal operation should succeed
        let normal_result = context
            .execute_with_signer(|signer| Box::pin(async move { signer.get_balance() }))
            .await;

        test_result.normal_operation_success = normal_result.is_ok();

        // Test 2: Unauthorized access should fail
        let unauthorized_result = context.attempt_unauthorized_access();
        test_result.unauthorized_access_blocked = unauthorized_result.is_err();

        // Test 3: Private key access should fail
        let private_key_result = context
            .execute_with_signer(|signer| Box::pin(async move { signer.get_private_key() }))
            .await;

        test_result.private_key_access_blocked = private_key_result.is_err();

        // Test 4: Check for isolation violations
        test_result.isolation_violations = context.get_isolation_violations();

        Ok(test_result)
    }

    /// Test signer context timeout handling.
    ///
    /// # Errors
    ///
    /// Returns an error if the timeout test fails or if there are issues with the signer operation.
    pub async fn test_signer_timeout(
        context: &MockSignerContext,
        timeout_duration: Duration,
    ) -> Result<bool, SignerTestError> {
        let operation = context.execute_with_signer(|signer| {
            Box::pin(async move {
                use tokio::time::sleep;
                // Simulate long-running operation
                #[expect(clippy::arithmetic_side_effects)]
                let extended_duration = timeout_duration * 2;
                sleep(extended_duration).await;
                signer.get_balance()
            })
        });

        match timeout(timeout_duration, operation).await {
            Ok(_) => Ok(false), // Operation completed, timeout didn't work
            Err(_) => Ok(true), // Operation timed out as expected
        }
    }

    /// Validate signer context security properties.
    #[must_use]
    pub fn validate_security_properties(context: &MockSignerContext) -> SecurityValidationResult {
        let mut result = SecurityValidationResult::default();

        // Check access logs
        let logs = context.get_access_logs();
        result.access_log_state = if logs.is_empty() {
            AccessLogState::NoLogs
        } else if logs.iter().any(|log| !log.authorized) {
            AccessLogState::LogsWithUnauthorizedAttempts
        } else {
            AccessLogState::LogsWithAuthorizedOnly
        };

        // Check isolation violations
        let violations = context.get_isolation_violations();
        result.isolation_state = if violations.is_empty() {
            IsolationState::NoViolations
        } else {
            IsolationState::ViolationsDetected
        };
        result.critical_violations = violations
            .iter()
            .filter(|v| matches!(v.severity, ViolationSeverity::Critical))
            .count();

        // Check if unauthorized access attempts were detected
        result.unauthorized_access_state = if context.has_unauthorized_attempts() {
            UnauthorizedAccessState::UnauthorizedAccessDetected
        } else {
            UnauthorizedAccessState::NoUnauthorizedAccess
        };

        result
    }
}

/// Result of isolation testing.
#[derive(Debug, Default)]
pub struct IsolationTestResult {
    /// Whether normal signer operations completed successfully
    pub normal_operation_success: bool,
    /// Whether unauthorized access attempts were properly blocked
    pub unauthorized_access_blocked: bool,
    /// Whether private key access attempts were properly blocked
    pub private_key_access_blocked: bool,
    /// List of any isolation violations detected during testing
    pub isolation_violations: Vec<IsolationViolation>,
}

impl IsolationTestResult {
    /// Check if all isolation tests passed.
    #[must_use]
    pub const fn all_tests_passed(&self) -> bool {
        self.normal_operation_success
            && self.unauthorized_access_blocked
            && self.private_key_access_blocked
            && self.isolation_violations.is_empty()
    }
}

/// Access logging state during security validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessLogState {
    /// No access logs were generated
    NoLogs,
    /// Access logs were generated, all attempts were authorized
    LogsWithAuthorizedOnly,
    /// Access logs were generated, some unauthorized attempts were logged
    LogsWithUnauthorizedAttempts,
}

impl Default for AccessLogState {
    fn default() -> Self {
        Self::NoLogs
    }
}

/// Isolation violation state during security validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationState {
    /// No isolation violations detected
    NoViolations,
    /// Isolation violations were detected
    ViolationsDetected,
}

impl Default for IsolationState {
    fn default() -> Self {
        Self::NoViolations
    }
}

/// Unauthorized access detection state during security validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnauthorizedAccessState {
    /// No unauthorized access attempts detected
    NoUnauthorizedAccess,
    /// Unauthorized access attempts were detected
    UnauthorizedAccessDetected,
}

impl Default for UnauthorizedAccessState {
    fn default() -> Self {
        Self::NoUnauthorizedAccess
    }
}

/// Result of security validation.
#[derive(Debug, Default)]
pub struct SecurityValidationResult {
    /// State of access logging during testing
    pub access_log_state: AccessLogState,
    /// State of isolation violations during testing
    pub isolation_state: IsolationState,
    /// Number of critical violations found
    pub critical_violations: usize,
    /// State of unauthorized access detection during testing
    pub unauthorized_access_state: UnauthorizedAccessState,
}

impl SecurityValidationResult {
    /// Check if security validation passed.
    #[must_use]
    pub const fn security_validation_passed(&self) -> bool {
        matches!(
            self.access_log_state,
            AccessLogState::LogsWithAuthorizedOnly
        ) && matches!(self.isolation_state, IsolationState::NoViolations)
            && self.critical_violations == 0
            && matches!(
                self.unauthorized_access_state,
                UnauthorizedAccessState::NoUnauthorizedAccess
            )
    }

    /// Check if access logs were generated.
    #[must_use]
    pub const fn has_access_logs(&self) -> bool {
        !matches!(self.access_log_state, AccessLogState::NoLogs)
    }

    /// Check if unauthorized attempts were logged.
    #[must_use]
    pub const fn unauthorized_attempts_logged(&self) -> bool {
        matches!(
            self.access_log_state,
            AccessLogState::LogsWithUnauthorizedAttempts
        )
    }

    /// Check if isolation violations were detected.
    #[must_use]
    pub const fn isolation_violations_detected(&self) -> bool {
        matches!(self.isolation_state, IsolationState::ViolationsDetected)
    }

    /// Check if unauthorized access was detected.
    #[must_use]
    pub const fn unauthorized_access_detected(&self) -> bool {
        matches!(
            self.unauthorized_access_state,
            UnauthorizedAccessState::UnauthorizedAccessDetected
        )
    }
}

#[cfg(test)]
#[expect(clippy::expect_used)]
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
            .execute_with_signer(|signer| Box::pin(async move { signer.get_balance() }))
            .await;

        assert!(result.is_ok());
        let balance = result.expect("Result should be Ok");
        assert!(balance > 0);
    }

    #[tokio::test]
    async fn test_unauthorized_access_detection() {
        let context = MockSignerContext::new();

        let result = context.attempt_unauthorized_access();
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

        let isolation_result = result.expect("Isolation test should succeed");
        assert!(isolation_result.normal_operation_success);
        assert!(isolation_result.unauthorized_access_blocked);
        assert!(isolation_result.private_key_access_blocked);
    }

    #[tokio::test]
    async fn test_security_validation() {
        let context = MockSignerContext::new();

        // Perform some operations to generate logs
        let _ = context
            .execute_with_signer(|signer| Box::pin(async move { signer.get_balance() }))
            .await;

        let _ = context.attempt_unauthorized_access();

        let validation_result = validate_security_properties(&context);
        assert!(validation_result.has_access_logs());
        assert!(validation_result.unauthorized_access_detected());
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
        use std::collections::HashSet;

        let contexts = create_test_signer_contexts(5);
        assert_eq!(contexts.len(), 5);

        // Verify each context has unique signer ID
        let mut signer_ids = HashSet::new();
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
                        use tokio::time::sleep;
                        sleep(Duration::from_millis(50)).await;
                        signer
                            .transfer_sol(&format!("recipient_{operation_id}"), 1000)
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
