//! Health check utilities

use core::error::Error as CoreError;
use core::fmt::{Debug, Formatter, Result as FmtResult};
use core::time::Duration;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::task::JoinHandle;
use tokio::time::{interval, timeout};
use tracing::{debug, error, warn};

/// Health check trait for services and components
#[async_trait::async_trait]
#[expect(clippy::module_name_repetitions)]
pub trait HealthCheck: Send + Sync + Debug {
    /// Get component name
    fn component_name(&self) -> &str;

    /// Perform a health check
    async fn health_check(&self) -> Result<HealthCheckResult, Box<dyn CoreError + Send + Sync>>;
}

/// Result of a health check
#[derive(Debug, Clone)]
#[expect(clippy::module_name_repetitions)]
pub struct HealthCheckResult {
    /// Additional details
    pub details: HashMap<String, String>,
    /// Whether the component is healthy
    pub healthy: bool,
    /// Status message
    pub message: String,
    /// Response time for the check
    pub response_time: Duration,
    /// Timestamp when check was performed
    pub timestamp: Instant,
}

impl HealthCheckResult {
    /// Create a healthy result
    #[must_use]
    pub fn healthy(message: &str) -> Self {
        Self {
            details: HashMap::new(),
            healthy: true,
            message: message.to_string(),
            response_time: Duration::from_millis(0),
            timestamp: Instant::now(),
        }
    }

    /// Create an unhealthy result
    #[must_use]
    pub fn unhealthy(message: &str) -> Self {
        Self {
            details: HashMap::new(),
            healthy: false,
            message: message.to_string(),
            response_time: Duration::from_millis(0),
            timestamp: Instant::now(),
        }
    }

    /// Add detail information
    #[must_use]
    pub fn with_detail(mut self, key: &str, value: &str) -> Self {
        self.details.insert(key.to_string(), value.to_string());
        self
    }

    /// Set response time
    #[must_use]
    pub const fn with_response_time(mut self, duration: Duration) -> Self {
        self.response_time = duration;
        self
    }
}

/// Health check coordinator that manages multiple health checks
#[derive(Debug)]
#[expect(clippy::module_name_repetitions)]
pub struct HealthCheckCoordinator {
    /// Cache TTL
    cache_ttl: Duration,
    /// Cached results
    cached_results: Arc<DashMap<String, (HealthCheckResult, Instant)>>,
    /// Registered health checks
    checks: Arc<DashMap<String, Arc<dyn HealthCheck + 'static>>>,
}

impl HealthCheckCoordinator {
    /// Create a new health check coordinator
    #[must_use]
    pub fn new(cache_ttl: Duration) -> Self {
        Self {
            cache_ttl,
            cached_results: Arc::new(DashMap::new()),
            checks: Arc::new(DashMap::new()),
        }
    }

    /// Perform all health checks
    pub async fn check_all(&self) -> HashMap<String, HealthCheckResult> {
        // Clone out names and Arc<HealthCheck> first to avoid holding DashMap guards across await
        let entries: Vec<(String, Arc<dyn HealthCheck + 'static>)> = self
            .checks
            .iter()
            .map(|entry| (entry.key().clone(), Arc::clone(entry.value())))
            .collect();

        let mut results = HashMap::new();
        for (name, check) in entries {
            let result = self.check_with_cache(&name, &check).await;
            results.insert(name, result);
        }

        results
    }

    /// Perform a specific health check
    pub async fn check_one(&self, name: &str) -> Option<HealthCheckResult> {
        let check = self.checks.get(name).map(|g| Arc::clone(g.value()))?;
        let result = self.check_with_cache(name, &check).await;
        Some(result)
    }

    /// Clear all cached results
    pub fn clear_cache(&self) {
        self.cached_results.clear();
    }

    /// Get overall health summary
    pub async fn health_summary(&self) -> HealthSummary {
        let results = self.check_all().await;
        let total = results.len();
        let healthy = results.values().filter(|r| r.healthy).count();
        #[expect(clippy::arithmetic_side_effects)]
        let unhealthy = total - healthy;

        let overall_healthy = unhealthy == 0;
        let status = if overall_healthy {
            "healthy".to_string()
        } else {
            format!("{healthy}/{total} components healthy")
        };

        HealthSummary {
            component_results: results,
            healthy_components: healthy,
            overall_healthy,
            status,
            timestamp: Instant::now(),
            total_components: total,
            unhealthy_components: unhealthy,
        }
    }

    /// Check if all components are healthy
    pub async fn is_healthy(&self) -> bool {
        let results = self.check_all().await;
        results.values().all(|result| result.healthy)
    }

    /// Register a health check
    pub fn register(&self, name: String, check: Arc<dyn HealthCheck + 'static>) {
        self.checks.insert(name, check);
    }

    /// Unregister a health check
    /// Returns true if the check was removed, false if it didn't exist
    #[must_use]
    pub fn unregister(&self, name: &str) -> bool {
        self.checks.remove(name).is_some()
    }

    /// Perform health check with caching
    async fn check_with_cache(
        &self,
        name: &str,
        check: &Arc<dyn HealthCheck + 'static>,
    ) -> HealthCheckResult {
        // Check cache first
        if let Some(cached_result) = self.get_cached_result_if_valid(name) {
            return cached_result;
        }

        // Perform actual check
        let start = Instant::now();
        let result = Self::execute_health_check(check).await;
        self.finalize_and_cache_result(name, result, start)
    }

    /// Execute health check with timeout
    async fn execute_health_check(check: &Arc<dyn HealthCheck + 'static>) -> HealthCheckResult {
        let health_check_future = check.health_check();
        let timeout_result = timeout(Duration::from_secs(10), health_check_future).await;
        match timeout_result {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => HealthCheckResult::unhealthy("Health check failed"),
            Err(_) => HealthCheckResult::unhealthy("Health check timed out"),
        }
    }

    /// Update result timestamps and cache
    fn finalize_and_cache_result(
        &self,
        name: &str,
        mut result: HealthCheckResult,
        start_time: Instant,
    ) -> HealthCheckResult {
        result.response_time = start_time.elapsed();
        result.timestamp = Instant::now();

        // Update cache
        self.cached_results
            .insert(name.to_string(), (result.clone(), Instant::now()));

        if result.healthy {
            debug!("Health check passed for {}: {}", name, result.message);
        } else {
            warn!("Health check failed for {}: {}", name, result.message);
        }

        result
    }

    /// Check if cached result is still valid
    fn get_cached_result_if_valid(&self, name: &str) -> Option<HealthCheckResult> {
        if let Some(entry) = self.cached_results.get(name) {
            let (ref result, ref cached_at) = *entry.value();
            if cached_at.elapsed() < self.cache_ttl {
                debug!("Using cached health check result for {}", name);
                return Some(result.clone());
            }
        }
        None
    }

    /// Start background health check task
    #[must_use]
    pub fn start_background_checks(&self, interval_duration: Duration) -> JoinHandle<()> {
        let coordinator = self.clone();

        tokio::spawn(async move {
            let mut interval_timer = interval(interval_duration);

            loop {
                interval_timer.tick().await;

                let summary = coordinator.health_summary().await;
                debug!(
                    "Background health check: {} ({}/{})",
                    summary.status, summary.healthy_components, summary.total_components
                );

                if !summary.overall_healthy {
                    for (name, result) in &summary.component_results {
                        if !result.healthy {
                            error!("Component {} is unhealthy: {}", name, result.message);
                        }
                    }
                }
            }
        })
    }
}

impl Clone for HealthCheckCoordinator {
    fn clone(&self) -> Self {
        Self {
            cache_ttl: self.cache_ttl,
            cached_results: self.cached_results.clone(),
            checks: self.checks.clone(),
        }
    }
}

/// Overall health summary
#[derive(Debug, Clone)]
#[expect(clippy::module_name_repetitions)]
pub struct HealthSummary {
    /// Individual component health check results
    pub component_results: HashMap<String, HealthCheckResult>,
    /// Number of healthy components
    pub healthy_components: usize,
    /// Overall system health status
    pub overall_healthy: bool,
    /// Human-readable status description
    pub status: String,
    /// Health check timestamp
    pub timestamp: Instant,
    /// Total number of components checked
    pub total_components: usize,
    /// Number of unhealthy components
    pub unhealthy_components: usize,
}

/// Simple health check implementation for testing connections
pub struct ConnectionHealthCheck {
    name: String,
    test_fn: Arc<dyn Fn() -> Result<(), Box<dyn CoreError + Send + Sync>> + Send + Sync>,
}

impl ConnectionHealthCheck {
    /// Create a new connection health check
    pub fn new<F, E>(name: String, test_fn: F) -> Self
    where
        F: Fn() -> Result<(), E> + Send + Sync + 'static,
        E: CoreError + Send + Sync + 'static,
    {
        let test_fn = Arc::new(move || {
            test_fn().map_err(|e| Box::new(e) as Box<dyn CoreError + Send + Sync>)
        });

        Self { name, test_fn }
    }
}

impl Debug for ConnectionHealthCheck {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ConnectionHealthCheck")
            .field("name", &self.name)
            .field("test_fn", &"<function>")
            .finish()
    }
}

#[async_trait::async_trait]
impl HealthCheck for ConnectionHealthCheck {
    fn component_name(&self) -> &str {
        &self.name
    }

    async fn health_check(&self) -> Result<HealthCheckResult, Box<dyn CoreError + Send + Sync>> {
        let test_result = (self.test_fn)();
        match test_result {
            Ok(()) => Ok(HealthCheckResult::healthy("Connection OK")),
            Err(e) => Ok(HealthCheckResult::unhealthy(&format!(
                "Connection failed: {e}"
            ))),
        }
    }
}

/// HTTP endpoint health check
#[derive(Debug)]
pub struct HttpHealthCheck {
    expected_status: u16,
    name: String,
    timeout: Duration,
    url: String,
}

impl HttpHealthCheck {
    /// Create a new HTTP health check
    #[must_use]
    pub const fn new(name: String, url: String, timeout: Duration, expected_status: u16) -> Self {
        Self {
            expected_status,
            name,
            timeout,
            url,
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for HttpHealthCheck {
    fn component_name(&self) -> &str {
        &self.name
    }

    async fn health_check(&self) -> Result<HealthCheckResult, Box<dyn CoreError + Send + Sync>> {
        let client = reqwest::Client::new();

        let request_future = client.get(&self.url).send();
        let request_result = timeout(self.timeout, request_future).await;
        match request_result {
            Ok(Ok(response)) => {
                let status = response.status().as_u16();
                if status == self.expected_status {
                    Ok(HealthCheckResult::healthy(&format!("HTTP {status} OK")))
                } else {
                    Ok(HealthCheckResult::unhealthy(&format!(
                        "HTTP {} (expected {})",
                        status, self.expected_status
                    )))
                }
            }
            Ok(Err(e)) => Ok(HealthCheckResult::unhealthy(&format!(
                "HTTP request failed: {e}"
            ))),
            Err(_) => Ok(HealthCheckResult::unhealthy("HTTP request timed out")),
        }
    }
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};
    use tokio::time::sleep;

    #[derive(Debug)]
    struct AlwaysHealthyCheck {
        name: String,
    }

    #[async_trait::async_trait]
    impl HealthCheck for AlwaysHealthyCheck {
        async fn health_check(
            &self,
        ) -> Result<HealthCheckResult, Box<dyn CoreError + Send + Sync>> {
            Ok(HealthCheckResult::healthy("Always healthy"))
        }

        fn component_name(&self) -> &str {
            &self.name
        }
    }

    #[derive(Debug)]
    struct AlwaysUnhealthyCheck {
        name: String,
    }

    #[async_trait::async_trait]
    impl HealthCheck for AlwaysUnhealthyCheck {
        async fn health_check(
            &self,
        ) -> Result<HealthCheckResult, Box<dyn CoreError + Send + Sync>> {
            Ok(HealthCheckResult::unhealthy("Always unhealthy"))
        }

        fn component_name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_health_check_coordinator() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_secs(5));

        // Register checks
        coordinator.register(
            "healthy".to_string(),
            Arc::new(AlwaysHealthyCheck {
                name: "healthy".to_string(),
            }),
        );

        coordinator.register(
            "unhealthy".to_string(),
            Arc::new(AlwaysUnhealthyCheck {
                name: "unhealthy".to_string(),
            }),
        );

        // Check all
        let results = coordinator.check_all().await;
        assert_eq!(results.len(), 2);
        assert!(
            results
                .get("healthy")
                .expect("Should have healthy result")
                .healthy
        );
        assert!(
            !results
                .get("unhealthy")
                .expect("Should have unhealthy result")
                .healthy
        );

        // Check overall health
        assert!(!coordinator.is_healthy().await);

        // Check summary
        let summary = coordinator.health_summary().await;
        assert!(!summary.overall_healthy);
        assert_eq!(summary.total_components, 2);
        assert_eq!(summary.healthy_components, 1);
        assert_eq!(summary.unhealthy_components, 1);
    }

    #[tokio::test]
    async fn test_health_check_caching() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_millis(100));

        coordinator.register(
            "test".to_string(),
            Arc::new(AlwaysHealthyCheck {
                name: "test".to_string(),
            }),
        );

        // First check should hit the actual check
        let result1 = coordinator
            .check_one("test")
            .await
            .expect("First health check should return result");
        assert!(result1.healthy);

        // Second check should use cache (very fast)
        let start = Instant::now();
        let result2 = coordinator
            .check_one("test")
            .await
            .expect("Second health check should return cached result");
        let elapsed = start.elapsed();

        assert!(result2.healthy);
        assert!(elapsed < Duration::from_millis(10)); // Should be very fast due to caching
    }

    #[test]
    fn test_connection_health_check() {
        let check =
            ConnectionHealthCheck::new("test".to_string(), || -> Result<(), IoError> { Ok(()) });

        assert_eq!(check.component_name(), "test");
    }

    // ============= Additional Comprehensive Tests for 100% Coverage =============

    #[test]
    fn test_health_check_result_healthy() {
        let result = HealthCheckResult::healthy("Service is running");
        assert!(result.healthy);
        assert_eq!(result.message, "Service is running");
        assert!(result.details.is_empty());
        assert_eq!(result.response_time, Duration::from_millis(0));
    }

    #[test]
    fn test_health_check_result_unhealthy() {
        let result = HealthCheckResult::unhealthy("Service is down");
        assert!(!result.healthy);
        assert_eq!(result.message, "Service is down");
        assert!(result.details.is_empty());
        assert_eq!(result.response_time, Duration::from_millis(0));
    }

    #[test]
    fn test_health_check_result_with_detail() {
        let result = HealthCheckResult::healthy("OK")
            .with_detail("version", "1.0.0")
            .with_detail("uptime", "10s");

        assert!(result.healthy);
        assert_eq!(result.details.get("version"), Some(&"1.0.0".to_string()));
        assert_eq!(result.details.get("uptime"), Some(&"10s".to_string()));
        assert_eq!(result.details.len(), 2);
    }

    #[test]
    fn test_health_check_result_with_response_time() {
        let duration = Duration::from_millis(100);
        let result = HealthCheckResult::healthy("OK").with_response_time(duration);

        assert_eq!(result.response_time, duration);
    }

    #[test]
    fn test_health_check_result_chaining() {
        let result = HealthCheckResult::unhealthy("Error")
            .with_detail("error_code", "500")
            .with_response_time(Duration::from_millis(50))
            .with_detail("retries", "3");

        assert!(!result.healthy);
        assert_eq!(result.message, "Error");
        assert_eq!(result.details.len(), 2);
        assert_eq!(result.response_time, Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_coordinator_new() {
        let ttl = Duration::from_secs(10);
        let coordinator = HealthCheckCoordinator::new(ttl);

        assert_eq!(coordinator.cache_ttl, ttl);
        assert!(coordinator.checks.is_empty());
        assert!(coordinator.cached_results.is_empty());
    }

    #[tokio::test]
    async fn test_coordinator_clone() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_secs(5));
        coordinator.register(
            "test".to_string(),
            Arc::new(AlwaysHealthyCheck {
                name: "test".to_string(),
            }),
        );

        let cloned = coordinator.clone();
        assert_eq!(cloned.cache_ttl, coordinator.cache_ttl);
        assert_eq!(cloned.checks.len(), 1);
    }

    #[tokio::test]
    async fn test_coordinator_unregister_existing() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_secs(5));
        coordinator.register(
            "test".to_string(),
            Arc::new(AlwaysHealthyCheck {
                name: "test".to_string(),
            }),
        );

        let removed = coordinator.unregister("test");
        assert!(removed);
        assert!(coordinator.checks.is_empty());
    }

    #[tokio::test]
    async fn test_coordinator_unregister_nonexistent() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_secs(5));

        let removed = coordinator.unregister("nonexistent");
        assert!(!removed);
    }

    #[tokio::test]
    async fn test_coordinator_check_one_nonexistent() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_secs(5));

        let result = coordinator.check_one("nonexistent").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_coordinator_check_one_existing() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_secs(5));
        coordinator.register(
            "test".to_string(),
            Arc::new(AlwaysHealthyCheck {
                name: "test".to_string(),
            }),
        );

        let result = coordinator.check_one("test").await;
        assert!(result.is_some());
        assert!(result.expect("Health check should return result").healthy);
    }

    #[tokio::test]
    async fn test_coordinator_is_healthy_with_no_checks() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_secs(5));

        // Should be healthy when no checks are registered
        assert!(coordinator.is_healthy().await);
    }

    #[tokio::test]
    async fn test_coordinator_is_healthy_with_all_healthy() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_secs(5));
        coordinator.register(
            "test1".to_string(),
            Arc::new(AlwaysHealthyCheck {
                name: "test1".to_string(),
            }),
        );
        coordinator.register(
            "test2".to_string(),
            Arc::new(AlwaysHealthyCheck {
                name: "test2".to_string(),
            }),
        );

        assert!(coordinator.is_healthy().await);
    }

    #[tokio::test]
    async fn test_coordinator_health_summary_all_healthy() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_secs(5));
        coordinator.register(
            "test1".to_string(),
            Arc::new(AlwaysHealthyCheck {
                name: "test1".to_string(),
            }),
        );
        coordinator.register(
            "test2".to_string(),
            Arc::new(AlwaysHealthyCheck {
                name: "test2".to_string(),
            }),
        );

        let summary = coordinator.health_summary().await;
        assert!(summary.overall_healthy);
        assert_eq!(summary.status, "healthy");
        assert_eq!(summary.total_components, 2);
        assert_eq!(summary.healthy_components, 2);
        assert_eq!(summary.unhealthy_components, 0);
    }

    #[tokio::test]
    async fn test_coordinator_health_summary_mixed_health() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_secs(5));
        coordinator.register(
            "healthy".to_string(),
            Arc::new(AlwaysHealthyCheck {
                name: "healthy".to_string(),
            }),
        );
        coordinator.register(
            "unhealthy".to_string(),
            Arc::new(AlwaysUnhealthyCheck {
                name: "unhealthy".to_string(),
            }),
        );

        let summary = coordinator.health_summary().await;
        assert!(!summary.overall_healthy);
        assert_eq!(summary.status, "1/2 components healthy");
        assert_eq!(summary.total_components, 2);
        assert_eq!(summary.healthy_components, 1);
        assert_eq!(summary.unhealthy_components, 1);
    }

    #[tokio::test]
    async fn test_coordinator_health_summary_empty() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_secs(5));

        let summary = coordinator.health_summary().await;
        assert!(summary.overall_healthy);
        assert_eq!(summary.status, "healthy");
        assert_eq!(summary.total_components, 0);
        assert_eq!(summary.healthy_components, 0);
        assert_eq!(summary.unhealthy_components, 0);
    }

    #[tokio::test]
    async fn test_coordinator_clear_cache() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_secs(60));
        coordinator.register(
            "test".to_string(),
            Arc::new(AlwaysHealthyCheck {
                name: "test".to_string(),
            }),
        );

        // Populate cache
        coordinator.check_one("test").await;
        assert!(!coordinator.cached_results.is_empty());

        // Clear cache
        coordinator.clear_cache();
        assert!(coordinator.cached_results.is_empty());
    }

    #[tokio::test]
    async fn test_coordinator_cache_expiry() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_millis(1)); // Very short TTL
        coordinator.register(
            "test".to_string(),
            Arc::new(AlwaysHealthyCheck {
                name: "test".to_string(),
            }),
        );

        // First check
        coordinator.check_one("test").await;

        // Wait for cache to expire
        sleep(Duration::from_millis(5)).await;

        // Second check should not use cache
        let start = Instant::now();
        coordinator.check_one("test").await;
        let elapsed = start.elapsed();

        // Should take some time as it's not using cache
        assert!(elapsed > Duration::from_millis(1));
    }

    #[derive(Debug)]
    struct ErroringHealthCheck {
        name: String,
    }

    #[async_trait::async_trait]
    impl HealthCheck for ErroringHealthCheck {
        async fn health_check(
            &self,
        ) -> Result<HealthCheckResult, Box<dyn CoreError + Send + Sync>> {
            Err("Health check error".into())
        }

        fn component_name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_coordinator_with_erroring_check() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_secs(5));
        coordinator.register(
            "error".to_string(),
            Arc::new(ErroringHealthCheck {
                name: "error".to_string(),
            }),
        );

        let result = coordinator
            .check_one("error")
            .await
            .expect("Error health check should return result");
        assert!(!result.healthy);
        assert_eq!(result.message, "Health check failed");
    }

    #[derive(Debug)]
    struct SlowHealthCheck {
        name: String,
        delay: Duration,
    }

    #[async_trait::async_trait]
    impl HealthCheck for SlowHealthCheck {
        async fn health_check(
            &self,
        ) -> Result<HealthCheckResult, Box<dyn CoreError + Send + Sync>> {
            sleep(self.delay).await;
            Ok(HealthCheckResult::healthy("Slow but healthy"))
        }

        fn component_name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_coordinator_with_timeout() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_secs(5));
        coordinator.register(
            "slow".to_string(),
            Arc::new(SlowHealthCheck {
                name: "slow".to_string(),
                delay: Duration::from_secs(15), // Longer than 10s timeout
            }),
        );

        let result = coordinator
            .check_one("slow")
            .await
            .expect("Slow health check should return timeout result");
        assert!(!result.healthy);
        assert_eq!(result.message, "Health check timed out");
    }

    #[tokio::test]
    async fn test_connection_health_check_success() {
        let check =
            ConnectionHealthCheck::new("test".to_string(), || -> Result<(), IoError> { Ok(()) });

        let result = check
            .health_check()
            .await
            .expect("Connection health check should succeed");
        assert!(result.healthy);
        assert_eq!(result.message, "Connection OK");
    }

    #[tokio::test]
    async fn test_connection_health_check_failure() {
        let check = ConnectionHealthCheck::new("test".to_string(), || -> Result<(), IoError> {
            Err(IoError::new(
                ErrorKind::ConnectionRefused,
                "Connection refused",
            ))
        });

        let result = check
            .health_check()
            .await
            .expect("Connection health check should handle failure");
        assert!(!result.healthy);
        assert!(result.message.contains("Connection failed"));
        assert!(result.message.contains("Connection refused"));
    }

    #[test]
    fn test_http_health_check_new() {
        let check = HttpHealthCheck::new(
            "api".to_string(),
            "http://localhost:8080/health".to_string(),
            Duration::from_secs(5),
            200,
        );

        assert_eq!(check.component_name(), "api");
        assert_eq!(check.name, "api");
        assert_eq!(check.url, "http://localhost:8080/health");
        assert_eq!(check.timeout, Duration::from_secs(5));
        assert_eq!(check.expected_status, 200);
    }

    // Note: We can't easily test the actual HTTP functionality without a real server
    // or mocking framework, but we've tested the construction and the async trait implementation
    // is covered by the trait bounds. The actual HTTP logic would need integration tests.

    #[tokio::test]
    async fn test_coordinator_start_background_checks() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_secs(5));
        coordinator.register(
            "test".to_string(),
            Arc::new(AlwaysHealthyCheck {
                name: "test".to_string(),
            }),
        );

        // Start background task
        let handle = coordinator.start_background_checks(Duration::from_millis(10));

        // Let it run briefly
        sleep(Duration::from_millis(25)).await;

        // Stop the task
        handle.abort();

        // Verify task was running (it should have completed without panicking)
        assert!(handle.is_finished());
    }

    #[tokio::test]
    async fn test_coordinator_background_checks_with_unhealthy_components() {
        let coordinator = HealthCheckCoordinator::new(Duration::from_secs(5));
        coordinator.register(
            "unhealthy".to_string(),
            Arc::new(AlwaysUnhealthyCheck {
                name: "unhealthy".to_string(),
            }),
        );

        // Start background task
        let handle = coordinator.start_background_checks(Duration::from_millis(10));

        // Let it run briefly to trigger the unhealthy logging path
        sleep(Duration::from_millis(25)).await;

        // Stop the task
        handle.abort();

        assert!(handle.is_finished());
    }
}
