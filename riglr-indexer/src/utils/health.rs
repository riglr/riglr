//! Health check utilities

use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, warn};

/// Health check trait for services and components
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform a health check
    async fn health_check(
        &self,
    ) -> Result<HealthCheckResult, Box<dyn std::error::Error + Send + Sync>>;

    /// Get component name
    fn component_name(&self) -> &str;
}

/// Result of a health check
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Whether the component is healthy
    pub healthy: bool,
    /// Status message
    pub message: String,
    /// Additional details
    pub details: HashMap<String, String>,
    /// Response time for the check
    pub response_time: Duration,
    /// Timestamp when check was performed
    pub timestamp: Instant,
}

impl HealthCheckResult {
    /// Create a healthy result
    pub fn healthy(message: &str) -> Self {
        Self {
            healthy: true,
            message: message.to_string(),
            details: HashMap::new(),
            response_time: Duration::from_millis(0),
            timestamp: Instant::now(),
        }
    }

    /// Create an unhealthy result
    pub fn unhealthy(message: &str) -> Self {
        Self {
            healthy: false,
            message: message.to_string(),
            details: HashMap::new(),
            response_time: Duration::from_millis(0),
            timestamp: Instant::now(),
        }
    }

    /// Add detail information
    pub fn with_detail(mut self, key: &str, value: &str) -> Self {
        self.details.insert(key.to_string(), value.to_string());
        self
    }

    /// Set response time
    pub fn with_response_time(mut self, duration: Duration) -> Self {
        self.response_time = duration;
        self
    }
}

/// Health check coordinator that manages multiple health checks
pub struct HealthCheckCoordinator {
    /// Registered health checks
    checks: Arc<DashMap<String, Arc<dyn HealthCheck + 'static>>>,
    /// Cached results
    cached_results: Arc<DashMap<String, (HealthCheckResult, Instant)>>,
    /// Cache TTL
    cache_ttl: Duration,
}

impl HealthCheckCoordinator {
    /// Create a new health check coordinator
    pub fn new(cache_ttl: Duration) -> Self {
        Self {
            checks: Arc::new(DashMap::new()),
            cached_results: Arc::new(DashMap::new()),
            cache_ttl,
        }
    }

    /// Register a health check
    pub async fn register(&self, name: String, check: Arc<dyn HealthCheck + 'static>) {
        self.checks.insert(name, check);
    }

    /// Remove a health check
    pub async fn unregister(&self, name: &str) -> bool {
        self.checks.remove(name).is_some()
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
        Some(self.check_with_cache(name, &check).await)
    }

    /// Check if all components are healthy
    pub async fn is_healthy(&self) -> bool {
        let results = self.check_all().await;
        results.values().all(|result| result.healthy)
    }

    /// Get overall health summary
    pub async fn health_summary(&self) -> HealthSummary {
        let results = self.check_all().await;
        let total = results.len();
        let healthy = results.values().filter(|r| r.healthy).count();
        let unhealthy = total - healthy;

        let overall_healthy = unhealthy == 0;
        let status = if overall_healthy {
            "healthy".to_string()
        } else {
            format!("{}/{} components healthy", healthy, total)
        };

        HealthSummary {
            overall_healthy,
            status,
            total_components: total,
            healthy_components: healthy,
            unhealthy_components: unhealthy,
            component_results: results,
            timestamp: Instant::now(),
        }
    }

    /// Perform health check with caching
    async fn check_with_cache(
        &self,
        name: &str,
        check: &Arc<dyn HealthCheck + 'static>,
    ) -> HealthCheckResult {
        // Check cache first
        if let Some(entry) = self.cached_results.get(name) {
            let (result, cached_at) = entry.value();
            if cached_at.elapsed() < self.cache_ttl {
                debug!("Using cached health check result for {}", name);
                return result.clone();
            }
        }

        // Perform actual check
        let start = Instant::now();
        let mut result =
            match tokio::time::timeout(Duration::from_secs(10), check.health_check()).await {
                Ok(Ok(result)) => result,
                Ok(Err(_)) => HealthCheckResult::unhealthy("Health check failed"),
                Err(_) => HealthCheckResult::unhealthy("Health check timed out"),
            };

        result.response_time = start.elapsed();
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

    /// Start background health check task
    pub fn start_background_checks(&self, interval: Duration) -> tokio::task::JoinHandle<()> {
        let coordinator = self.clone();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

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

    /// Clear all cached results
    pub async fn clear_cache(&self) {
        self.cached_results.clear();
    }
}

impl Clone for HealthCheckCoordinator {
    fn clone(&self) -> Self {
        Self {
            checks: self.checks.clone(),
            cached_results: self.cached_results.clone(),
            cache_ttl: self.cache_ttl,
        }
    }
}

/// Overall health summary
#[derive(Debug, Clone)]
pub struct HealthSummary {
    /// Overall system health status
    pub overall_healthy: bool,
    /// Human-readable status description
    pub status: String,
    /// Total number of components checked
    pub total_components: usize,
    /// Number of healthy components
    pub healthy_components: usize,
    /// Number of unhealthy components
    pub unhealthy_components: usize,
    /// Individual component health check results
    pub component_results: HashMap<String, HealthCheckResult>,
    /// Health check timestamp
    pub timestamp: Instant,
}

/// Simple health check implementation for testing connections
pub struct ConnectionHealthCheck {
    name: String,
    test_fn: Arc<dyn Fn() -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync>,
}

impl ConnectionHealthCheck {
    /// Create a new connection health check
    pub fn new<F, E>(name: String, test_fn: F) -> Self
    where
        F: Fn() -> Result<(), E> + Send + Sync + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        let test_fn = Arc::new(move || {
            test_fn().map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        });

        Self { name, test_fn }
    }
}

#[async_trait::async_trait]
impl HealthCheck for ConnectionHealthCheck {
    async fn health_check(
        &self,
    ) -> Result<HealthCheckResult, Box<dyn std::error::Error + Send + Sync>> {
        match (self.test_fn)() {
            Ok(()) => Ok(HealthCheckResult::healthy("Connection OK")),
            Err(e) => Ok(HealthCheckResult::unhealthy(&format!(
                "Connection failed: {}",
                e
            ))),
        }
    }

    fn component_name(&self) -> &str {
        &self.name
    }
}

/// HTTP endpoint health check
pub struct HttpHealthCheck {
    name: String,
    url: String,
    timeout: Duration,
    expected_status: u16,
}

impl HttpHealthCheck {
    /// Create a new HTTP health check
    pub fn new(name: String, url: String, timeout: Duration, expected_status: u16) -> Self {
        Self {
            name,
            url,
            timeout,
            expected_status,
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for HttpHealthCheck {
    async fn health_check(
        &self,
    ) -> Result<HealthCheckResult, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        match tokio::time::timeout(self.timeout, client.get(&self.url).send()).await {
            Ok(Ok(response)) => {
                let status = response.status().as_u16();
                if status == self.expected_status {
                    Ok(HealthCheckResult::healthy(&format!("HTTP {} OK", status)))
                } else {
                    Ok(HealthCheckResult::unhealthy(&format!(
                        "HTTP {} (expected {})",
                        status, self.expected_status
                    )))
                }
            }
            Ok(Err(e)) => Ok(HealthCheckResult::unhealthy(&format!(
                "HTTP request failed: {}",
                e
            ))),
            Err(_) => Ok(HealthCheckResult::unhealthy("HTTP request timed out")),
        }
    }

    fn component_name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysHealthyCheck {
        name: String,
    }

    #[async_trait::async_trait]
    impl HealthCheck for AlwaysHealthyCheck {
        async fn health_check(
            &self,
        ) -> Result<HealthCheckResult, Box<dyn std::error::Error + Send + Sync>> {
            Ok(HealthCheckResult::healthy("Always healthy"))
        }

        fn component_name(&self) -> &str {
            &self.name
        }
    }

    struct AlwaysUnhealthyCheck {
        name: String,
    }

    #[async_trait::async_trait]
    impl HealthCheck for AlwaysUnhealthyCheck {
        async fn health_check(
            &self,
        ) -> Result<HealthCheckResult, Box<dyn std::error::Error + Send + Sync>> {
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
        coordinator
            .register(
                "healthy".to_string(),
                Arc::new(AlwaysHealthyCheck {
                    name: "healthy".to_string(),
                }),
            )
            .await;

        coordinator
            .register(
                "unhealthy".to_string(),
                Arc::new(AlwaysUnhealthyCheck {
                    name: "unhealthy".to_string(),
                }),
            )
            .await;

        // Check all
        let results = coordinator.check_all().await;
        assert_eq!(results.len(), 2);
        assert!(results["healthy"].healthy);
        assert!(!results["unhealthy"].healthy);

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

        coordinator
            .register(
                "test".to_string(),
                Arc::new(AlwaysHealthyCheck {
                    name: "test".to_string(),
                }),
            )
            .await;

        // First check should hit the actual check
        let result1 = coordinator.check_one("test").await.unwrap();
        assert!(result1.healthy);

        // Second check should use cache (very fast)
        let start = Instant::now();
        let result2 = coordinator.check_one("test").await.unwrap();
        let elapsed = start.elapsed();

        assert!(result2.healthy);
        assert!(elapsed < Duration::from_millis(10)); // Should be very fast due to caching
    }

    #[test]
    fn test_connection_health_check() {
        let check =
            ConnectionHealthCheck::new("test".to_string(), || -> Result<(), std::io::Error> {
                Ok(())
            });

        assert_eq!(check.component_name(), "test");
    }
}
