//! Health monitoring for stream systems.
//!
//! This module provides comprehensive health monitoring capabilities for streaming systems,
//! including component status tracking, alert generation, and health history management.
//! The health monitor continuously evaluates stream connections, event processing rates,
//! and error conditions to provide real-time system health visibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::core::{StreamHealth, StreamManager};

/// Health monitor for streams
pub struct HealthMonitor {
    /// Stream manager reference
    stream_manager: Arc<StreamManager>,
    /// Health check interval
    check_interval: Duration,
    /// Health history
    health_history: Arc<RwLock<Vec<HealthSnapshot>>>,
    /// Alert thresholds
    thresholds: HealthThresholds,
}

/// Health status levels
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Everything is working normally
    Healthy,
    /// Some issues but still functional
    Degraded,
    /// Critical issues, may not be functional
    Unhealthy,
    /// Not operational
    Down,
}

/// Health snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    /// Timestamp of the snapshot
    pub timestamp: SystemTime,
    /// Overall health status
    pub overall_status: HealthStatus,
    /// Individual component health
    pub components: HashMap<String, ComponentHealth>,
    /// Active alerts
    pub alerts: Vec<HealthAlert>,
}

/// Individual component health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Is connected
    pub is_connected: bool,
    /// Events processed
    pub events_processed: u64,
    /// Error count
    pub error_count: u64,
    /// Last event time
    pub last_event_time: Option<SystemTime>,
    /// Additional metrics
    pub metrics: HashMap<String, f64>,
}

/// Health alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    /// Alert severity
    pub severity: AlertSeverity,
    /// Component that triggered the alert
    pub component: String,
    /// Alert message
    pub message: String,
    /// When the alert was triggered
    pub triggered_at: SystemTime,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational alerts for general status updates
    Info,
    /// Warning alerts for potential issues that need attention
    Warning,
    /// Error alerts for failures that occurred but system can continue
    Error,
    /// Critical alerts for severe issues requiring immediate attention
    Critical,
}

/// Health check thresholds
#[derive(Debug, Clone)]
pub struct HealthThresholds {
    /// Max time without events before considered unhealthy
    pub max_event_age: Duration,
    /// Max error rate (errors per minute)
    pub max_error_rate: f64,
    /// Min events per minute
    pub min_event_rate: f64,
    /// Max consecutive errors
    pub max_consecutive_errors: u64,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            max_event_age: Duration::from_secs(300), // 5 minutes
            max_error_rate: 10.0,                    // 10 errors per minute
            min_event_rate: 0.1,                     // At least 1 event per 10 minutes
            max_consecutive_errors: 5,
        }
    }
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(stream_manager: Arc<StreamManager>) -> Self {
        Self {
            stream_manager,
            check_interval: Duration::from_secs(30),
            health_history: Arc::new(RwLock::new(Vec::new())),
            thresholds: HealthThresholds::default(),
        }
    }

    /// Set custom thresholds
    pub fn with_thresholds(mut self, thresholds: HealthThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    /// Set check interval
    pub fn with_check_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// Start monitoring
    pub async fn start(&self) {
        info!("Starting health monitoring");

        let stream_manager = self.stream_manager.clone();
        let health_history = self.health_history.clone();
        let thresholds = self.thresholds.clone();
        let check_interval = self.check_interval;

        tokio::spawn(async move {
            loop {
                // Perform health check
                let snapshot = Self::perform_health_check(&stream_manager, &thresholds).await;

                // Log status
                match snapshot.overall_status {
                    HealthStatus::Healthy => {
                        info!("System health: HEALTHY");
                    }
                    HealthStatus::Degraded => {
                        warn!("System health: DEGRADED - {} alerts", snapshot.alerts.len());
                    }
                    HealthStatus::Unhealthy => {
                        error!(
                            "System health: UNHEALTHY - {} alerts",
                            snapshot.alerts.len()
                        );
                    }
                    HealthStatus::Down => {
                        error!("System health: DOWN");
                    }
                }

                // Store snapshot
                {
                    let mut history = health_history.write().await;
                    history.push(snapshot.clone());

                    // Keep only last 100 snapshots
                    if history.len() > 100 {
                        history.remove(0);
                    }
                }

                // Wait for next check
                tokio::time::sleep(check_interval).await;
            }
        });
    }

    /// Perform a health check
    async fn perform_health_check(
        stream_manager: &StreamManager,
        thresholds: &HealthThresholds,
    ) -> HealthSnapshot {
        let mut components = HashMap::new();
        let mut alerts = Vec::new();

        // Get health of all streams
        let stream_health = stream_manager.health().await;

        for (name, health) in stream_health {
            let component_health =
                Self::check_component_health(&name, &health, thresholds, &mut alerts);
            components.insert(name, component_health);
        }

        // Determine overall status
        let overall_status = Self::determine_overall_status(&components);

        HealthSnapshot {
            timestamp: SystemTime::now(),
            overall_status,
            components,
            alerts,
        }
    }

    /// Check individual component health
    fn check_component_health(
        name: &str,
        health: &StreamHealth,
        thresholds: &HealthThresholds,
        alerts: &mut Vec<HealthAlert>,
    ) -> ComponentHealth {
        let mut status = HealthStatus::Healthy;
        let mut metrics = HashMap::new();

        // Check connection
        if !health.is_connected {
            status = HealthStatus::Down;
            alerts.push(HealthAlert {
                severity: AlertSeverity::Critical,
                component: name.to_string(),
                message: "Stream is not connected".to_string(),
                triggered_at: SystemTime::now(),
            });
        }

        // Check event age
        if let Some(last_event_time) = health.last_event_time {
            let age = SystemTime::now()
                .duration_since(last_event_time)
                .unwrap_or(Duration::ZERO);

            metrics.insert("event_age_seconds".to_string(), age.as_secs_f64());

            if age > thresholds.max_event_age && status == HealthStatus::Healthy {
                status = HealthStatus::Degraded;
                alerts.push(HealthAlert {
                    severity: AlertSeverity::Warning,
                    component: name.to_string(),
                    message: format!("No events for {} seconds", age.as_secs()),
                    triggered_at: SystemTime::now(),
                });
            }
        }

        // Check error rate
        if health.error_count > thresholds.max_consecutive_errors {
            status = HealthStatus::Unhealthy;
            alerts.push(HealthAlert {
                severity: AlertSeverity::Error,
                component: name.to_string(),
                message: format!("High error count: {}", health.error_count),
                triggered_at: SystemTime::now(),
            });
        }

        // Calculate event rate
        let event_rate = health.events_processed as f64 / 60.0; // Events per minute approximation
        metrics.insert("event_rate".to_string(), event_rate);

        ComponentHealth {
            name: name.to_string(),
            status,
            is_connected: health.is_connected,
            events_processed: health.events_processed,
            error_count: health.error_count,
            last_event_time: health.last_event_time,
            metrics,
        }
    }

    /// Determine overall status from component statuses
    fn determine_overall_status(components: &HashMap<String, ComponentHealth>) -> HealthStatus {
        let mut has_down = false;
        let mut has_unhealthy = false;
        let mut has_degraded = false;

        for component in components.values() {
            match component.status {
                HealthStatus::Down => has_down = true,
                HealthStatus::Unhealthy => has_unhealthy = true,
                HealthStatus::Degraded => has_degraded = true,
                HealthStatus::Healthy => {}
            }
        }

        if has_down {
            HealthStatus::Down
        } else if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Get current health snapshot
    pub async fn current_health(&self) -> HealthSnapshot {
        Self::perform_health_check(&self.stream_manager, &self.thresholds).await
    }

    /// Get health history
    pub async fn health_history(&self) -> Vec<HealthSnapshot> {
        self.health_history.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{StreamHealth, StreamManager};
    use std::collections::HashMap;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn create_test_stream_manager() -> Arc<StreamManager> {
        Arc::new(StreamManager::new())
    }

    #[test]
    fn test_health_status_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_eq!(HealthStatus::Degraded, HealthStatus::Degraded);
        assert_eq!(HealthStatus::Unhealthy, HealthStatus::Unhealthy);
        assert_eq!(HealthStatus::Down, HealthStatus::Down);

        assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
        assert_ne!(HealthStatus::Degraded, HealthStatus::Unhealthy);
        assert_ne!(HealthStatus::Unhealthy, HealthStatus::Down);
    }

    #[test]
    fn test_alert_severity_equality() {
        assert_eq!(AlertSeverity::Info, AlertSeverity::Info);
        assert_eq!(AlertSeverity::Warning, AlertSeverity::Warning);
        assert_eq!(AlertSeverity::Error, AlertSeverity::Error);
        assert_eq!(AlertSeverity::Critical, AlertSeverity::Critical);

        assert_ne!(AlertSeverity::Info, AlertSeverity::Warning);
        assert_ne!(AlertSeverity::Warning, AlertSeverity::Error);
        assert_ne!(AlertSeverity::Error, AlertSeverity::Critical);
    }

    #[test]
    fn test_health_thresholds_default() {
        let thresholds = HealthThresholds::default();

        assert_eq!(thresholds.max_event_age, Duration::from_secs(300));
        assert_eq!(thresholds.max_error_rate, 10.0);
        assert_eq!(thresholds.min_event_rate, 0.1);
        assert_eq!(thresholds.max_consecutive_errors, 5);
    }

    #[test]
    fn test_health_monitor_new() {
        let stream_manager = create_test_stream_manager();
        let monitor = HealthMonitor::new(stream_manager.clone());

        assert_eq!(monitor.check_interval, Duration::from_secs(30));
        assert!(Arc::ptr_eq(&monitor.stream_manager, &stream_manager));
    }

    #[test]
    fn test_health_monitor_with_thresholds() {
        let stream_manager = create_test_stream_manager();
        let custom_thresholds = HealthThresholds {
            max_event_age: Duration::from_secs(600),
            max_error_rate: 20.0,
            min_event_rate: 0.5,
            max_consecutive_errors: 10,
        };

        let monitor = HealthMonitor::new(stream_manager).with_thresholds(custom_thresholds.clone());

        assert_eq!(monitor.thresholds.max_event_age, Duration::from_secs(600));
        assert_eq!(monitor.thresholds.max_error_rate, 20.0);
        assert_eq!(monitor.thresholds.min_event_rate, 0.5);
        assert_eq!(monitor.thresholds.max_consecutive_errors, 10);
    }

    #[test]
    fn test_health_monitor_with_check_interval() {
        let stream_manager = create_test_stream_manager();
        let custom_interval = Duration::from_secs(60);

        let monitor = HealthMonitor::new(stream_manager).with_check_interval(custom_interval);

        assert_eq!(monitor.check_interval, custom_interval);
    }

    #[test]
    fn test_health_monitor_builder_chaining() {
        let stream_manager = create_test_stream_manager();
        let custom_thresholds = HealthThresholds {
            max_event_age: Duration::from_secs(600),
            max_error_rate: 20.0,
            min_event_rate: 0.5,
            max_consecutive_errors: 10,
        };
        let custom_interval = Duration::from_secs(45);

        let monitor = HealthMonitor::new(stream_manager)
            .with_thresholds(custom_thresholds.clone())
            .with_check_interval(custom_interval);

        assert_eq!(monitor.check_interval, custom_interval);
        assert_eq!(monitor.thresholds.max_event_age, Duration::from_secs(600));
    }

    #[tokio::test]
    async fn test_current_health_when_no_streams_should_return_healthy() {
        let stream_manager = create_test_stream_manager();
        let monitor = HealthMonitor::new(stream_manager);

        let snapshot = monitor.current_health().await;

        assert_eq!(snapshot.overall_status, HealthStatus::Healthy);
        assert!(snapshot.components.is_empty());
        assert!(snapshot.alerts.is_empty());
    }

    #[tokio::test]
    async fn test_health_history_when_empty_should_return_empty_vector() {
        let stream_manager = create_test_stream_manager();
        let monitor = HealthMonitor::new(stream_manager);

        let history = monitor.health_history().await;

        assert!(history.is_empty());
    }

    #[test]
    fn test_check_component_health_when_connected_and_healthy_should_return_healthy() {
        let health = StreamHealth {
            is_connected: true,
            events_processed: 100,
            error_count: 0,
            last_event_time: Some(SystemTime::now()),
            backlog_size: None,
            custom_metrics: None,
            stream_info: None,
        };
        let thresholds = HealthThresholds::default();
        let mut alerts = Vec::new();

        let component_health =
            HealthMonitor::check_component_health("test_stream", &health, &thresholds, &mut alerts);

        assert_eq!(component_health.status, HealthStatus::Healthy);
        assert_eq!(component_health.name, "test_stream");
        assert!(component_health.is_connected);
        assert_eq!(component_health.events_processed, 100);
        assert_eq!(component_health.error_count, 0);
        assert!(component_health.last_event_time.is_some());
        assert!(alerts.is_empty());
        assert!(component_health.metrics.contains_key("event_age_seconds"));
        assert!(component_health.metrics.contains_key("event_rate"));
    }

    #[test]
    fn test_check_component_health_when_not_connected_should_return_down() {
        let health = StreamHealth {
            is_connected: false,
            events_processed: 50,
            error_count: 0,
            last_event_time: Some(SystemTime::now()),
            backlog_size: None,
            custom_metrics: None,
            stream_info: None,
        };
        let thresholds = HealthThresholds::default();
        let mut alerts = Vec::new();

        let component_health =
            HealthMonitor::check_component_health("test_stream", &health, &thresholds, &mut alerts);

        assert_eq!(component_health.status, HealthStatus::Down);
        assert!(!component_health.is_connected);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Critical);
        assert_eq!(alerts[0].component, "test_stream");
        assert_eq!(alerts[0].message, "Stream is not connected");
    }

    #[test]
    fn test_check_component_health_when_old_events_should_return_degraded() {
        let old_time = SystemTime::now() - Duration::from_secs(400); // Older than 300s threshold
        let health = StreamHealth {
            is_connected: true,
            events_processed: 100,
            error_count: 0,
            last_event_time: Some(old_time),
            backlog_size: None,
            custom_metrics: None,
            stream_info: None,
        };
        let thresholds = HealthThresholds::default();
        let mut alerts = Vec::new();

        let component_health =
            HealthMonitor::check_component_health("test_stream", &health, &thresholds, &mut alerts);

        assert_eq!(component_health.status, HealthStatus::Degraded);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Warning);
        assert!(alerts[0].message.contains("No events for"));
    }

    #[test]
    fn test_check_component_health_when_no_last_event_time_should_not_create_age_alert() {
        let health = StreamHealth {
            is_connected: true,
            events_processed: 100,
            error_count: 0,
            last_event_time: None,
            backlog_size: None,
            custom_metrics: None,
            stream_info: None,
        };
        let thresholds = HealthThresholds::default();
        let mut alerts = Vec::new();

        let component_health =
            HealthMonitor::check_component_health("test_stream", &health, &thresholds, &mut alerts);

        assert_eq!(component_health.status, HealthStatus::Healthy);
        assert_eq!(component_health.last_event_time, None);
        assert!(alerts.is_empty());
        assert!(!component_health.metrics.contains_key("event_age_seconds"));
        assert!(component_health.metrics.contains_key("event_rate"));
    }

    #[test]
    fn test_check_component_health_when_high_error_count_should_return_unhealthy() {
        let health = StreamHealth {
            is_connected: true,
            events_processed: 100,
            error_count: 10, // Higher than default threshold of 5
            last_event_time: Some(SystemTime::now()),
            backlog_size: None,
            custom_metrics: None,
            stream_info: None,
        };
        let thresholds = HealthThresholds::default();
        let mut alerts = Vec::new();

        let component_health =
            HealthMonitor::check_component_health("test_stream", &health, &thresholds, &mut alerts);

        assert_eq!(component_health.status, HealthStatus::Unhealthy);
        assert_eq!(component_health.error_count, 10);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Error);
        assert!(alerts[0].message.contains("High error count: 10"));
    }

    #[test]
    fn test_check_component_health_when_disconnected_takes_precedence_over_errors() {
        let health = StreamHealth {
            is_connected: false,
            events_processed: 100,
            error_count: 10, // High errors, but disconnected should take precedence
            last_event_time: Some(SystemTime::now()),
            backlog_size: None,
            custom_metrics: None,
            stream_info: None,
        };
        let thresholds = HealthThresholds::default();
        let mut alerts = Vec::new();

        let component_health =
            HealthMonitor::check_component_health("test_stream", &health, &thresholds, &mut alerts);

        assert_eq!(component_health.status, HealthStatus::Down);
        // Should have both alerts - one for disconnection, one for errors
        assert_eq!(alerts.len(), 2);
    }

    #[test]
    fn test_check_component_health_when_old_events_but_disconnected_should_return_down() {
        let old_time = SystemTime::now() - Duration::from_secs(400);
        let health = StreamHealth {
            is_connected: false,
            events_processed: 100,
            error_count: 0,
            last_event_time: Some(old_time),
            backlog_size: None,
            custom_metrics: None,
            stream_info: None,
        };
        let thresholds = HealthThresholds::default();
        let mut alerts = Vec::new();

        let component_health =
            HealthMonitor::check_component_health("test_stream", &health, &thresholds, &mut alerts);

        assert_eq!(component_health.status, HealthStatus::Down);
        // Should only have disconnect alert, event age check shouldn't run for Down status
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Critical);
    }

    #[test]
    fn test_check_component_health_when_duration_since_fails_should_use_zero() {
        // Test edge case where duration_since might fail
        let future_time = SystemTime::now() + Duration::from_secs(100);
        let health = StreamHealth {
            is_connected: true,
            events_processed: 100,
            error_count: 0,
            last_event_time: Some(future_time),
            backlog_size: None,
            custom_metrics: None,
            stream_info: None,
        };
        let thresholds = HealthThresholds::default();
        let mut alerts = Vec::new();

        let component_health =
            HealthMonitor::check_component_health("test_stream", &health, &thresholds, &mut alerts);

        assert_eq!(component_health.status, HealthStatus::Healthy);
        assert!(component_health.metrics.contains_key("event_age_seconds"));
        // Should be 0.0 when duration_since fails
        assert_eq!(component_health.metrics["event_age_seconds"], 0.0);
    }

    #[test]
    fn test_determine_overall_status_when_empty_components_should_return_healthy() {
        let components = HashMap::new();

        let status = HealthMonitor::determine_overall_status(&components);

        assert_eq!(status, HealthStatus::Healthy);
    }

    #[test]
    fn test_determine_overall_status_when_all_healthy_should_return_healthy() {
        let mut components = HashMap::new();
        components.insert(
            "stream1".to_string(),
            ComponentHealth {
                name: "stream1".to_string(),
                status: HealthStatus::Healthy,
                is_connected: true,
                events_processed: 100,
                error_count: 0,
                last_event_time: Some(SystemTime::now()),
                metrics: HashMap::new(),
            },
        );
        components.insert(
            "stream2".to_string(),
            ComponentHealth {
                name: "stream2".to_string(),
                status: HealthStatus::Healthy,
                is_connected: true,
                events_processed: 50,
                error_count: 0,
                last_event_time: Some(SystemTime::now()),
                metrics: HashMap::new(),
            },
        );

        let status = HealthMonitor::determine_overall_status(&components);

        assert_eq!(status, HealthStatus::Healthy);
    }

    #[test]
    fn test_determine_overall_status_when_has_degraded_should_return_degraded() {
        let mut components = HashMap::new();
        components.insert(
            "stream1".to_string(),
            ComponentHealth {
                name: "stream1".to_string(),
                status: HealthStatus::Healthy,
                is_connected: true,
                events_processed: 100,
                error_count: 0,
                last_event_time: Some(SystemTime::now()),
                metrics: HashMap::new(),
            },
        );
        components.insert(
            "stream2".to_string(),
            ComponentHealth {
                name: "stream2".to_string(),
                status: HealthStatus::Degraded,
                is_connected: true,
                events_processed: 50,
                error_count: 0,
                last_event_time: Some(SystemTime::now()),
                metrics: HashMap::new(),
            },
        );

        let status = HealthMonitor::determine_overall_status(&components);

        assert_eq!(status, HealthStatus::Degraded);
    }

    #[test]
    fn test_determine_overall_status_when_has_unhealthy_should_return_unhealthy() {
        let mut components = HashMap::new();
        components.insert(
            "stream1".to_string(),
            ComponentHealth {
                name: "stream1".to_string(),
                status: HealthStatus::Degraded,
                is_connected: true,
                events_processed: 100,
                error_count: 0,
                last_event_time: Some(SystemTime::now()),
                metrics: HashMap::new(),
            },
        );
        components.insert(
            "stream2".to_string(),
            ComponentHealth {
                name: "stream2".to_string(),
                status: HealthStatus::Unhealthy,
                is_connected: true,
                events_processed: 50,
                error_count: 10,
                last_event_time: Some(SystemTime::now()),
                metrics: HashMap::new(),
            },
        );

        let status = HealthMonitor::determine_overall_status(&components);

        assert_eq!(status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_determine_overall_status_when_has_down_should_return_down() {
        let mut components = HashMap::new();
        components.insert(
            "stream1".to_string(),
            ComponentHealth {
                name: "stream1".to_string(),
                status: HealthStatus::Unhealthy,
                is_connected: true,
                events_processed: 100,
                error_count: 10,
                last_event_time: Some(SystemTime::now()),
                metrics: HashMap::new(),
            },
        );
        components.insert(
            "stream2".to_string(),
            ComponentHealth {
                name: "stream2".to_string(),
                status: HealthStatus::Down,
                is_connected: false,
                events_processed: 50,
                error_count: 0,
                last_event_time: Some(SystemTime::now()),
                metrics: HashMap::new(),
            },
        );

        let status = HealthMonitor::determine_overall_status(&components);

        assert_eq!(status, HealthStatus::Down);
    }

    #[test]
    fn test_determine_overall_status_priority_order() {
        // Test that Down has highest priority, then Unhealthy, then Degraded
        let mut components = HashMap::new();
        components.insert(
            "stream1".to_string(),
            ComponentHealth {
                name: "stream1".to_string(),
                status: HealthStatus::Healthy,
                is_connected: true,
                events_processed: 100,
                error_count: 0,
                last_event_time: Some(SystemTime::now()),
                metrics: HashMap::new(),
            },
        );
        components.insert(
            "stream2".to_string(),
            ComponentHealth {
                name: "stream2".to_string(),
                status: HealthStatus::Degraded,
                is_connected: true,
                events_processed: 50,
                error_count: 0,
                last_event_time: Some(SystemTime::now()),
                metrics: HashMap::new(),
            },
        );
        components.insert(
            "stream3".to_string(),
            ComponentHealth {
                name: "stream3".to_string(),
                status: HealthStatus::Unhealthy,
                is_connected: true,
                events_processed: 25,
                error_count: 10,
                last_event_time: Some(SystemTime::now()),
                metrics: HashMap::new(),
            },
        );
        components.insert(
            "stream4".to_string(),
            ComponentHealth {
                name: "stream4".to_string(),
                status: HealthStatus::Down,
                is_connected: false,
                events_processed: 0,
                error_count: 0,
                last_event_time: None,
                metrics: HashMap::new(),
            },
        );

        let status = HealthMonitor::determine_overall_status(&components);

        assert_eq!(status, HealthStatus::Down);
    }

    #[test]
    fn test_health_snapshot_serialization() {
        let snapshot = HealthSnapshot {
            timestamp: UNIX_EPOCH,
            overall_status: HealthStatus::Healthy,
            components: HashMap::new(),
            alerts: Vec::new(),
        };

        // Test that it can be serialized (this will panic if serialization fails)
        let _serialized = serde_json::to_string(&snapshot).expect("Should serialize");
    }

    #[test]
    fn test_component_health_serialization() {
        let component = ComponentHealth {
            name: "test".to_string(),
            status: HealthStatus::Healthy,
            is_connected: true,
            events_processed: 100,
            error_count: 0,
            last_event_time: Some(UNIX_EPOCH),
            metrics: HashMap::new(),
        };

        let _serialized = serde_json::to_string(&component).expect("Should serialize");
    }

    #[test]
    fn test_health_alert_serialization() {
        let alert = HealthAlert {
            severity: AlertSeverity::Warning,
            component: "test".to_string(),
            message: "Test message".to_string(),
            triggered_at: UNIX_EPOCH,
        };

        let _serialized = serde_json::to_string(&alert).expect("Should serialize");
    }

    #[test]
    fn test_debug_implementations() {
        // Test Debug implementations
        let status = HealthStatus::Healthy;
        let _debug_str = format!("{:?}", status);

        let severity = AlertSeverity::Warning;
        let _debug_str = format!("{:?}", severity);

        let thresholds = HealthThresholds::default();
        let _debug_str = format!("{:?}", thresholds);

        let snapshot = HealthSnapshot {
            timestamp: UNIX_EPOCH,
            overall_status: HealthStatus::Healthy,
            components: HashMap::new(),
            alerts: Vec::new(),
        };
        let _debug_str = format!("{:?}", snapshot);

        let component = ComponentHealth {
            name: "test".to_string(),
            status: HealthStatus::Healthy,
            is_connected: true,
            events_processed: 100,
            error_count: 0,
            last_event_time: Some(UNIX_EPOCH),
            metrics: HashMap::new(),
        };
        let _debug_str = format!("{:?}", component);

        let alert = HealthAlert {
            severity: AlertSeverity::Warning,
            component: "test".to_string(),
            message: "Test message".to_string(),
            triggered_at: UNIX_EPOCH,
        };
        let _debug_str = format!("{:?}", alert);
    }

    #[test]
    fn test_clone_implementations() {
        // Test Clone implementations
        let status = HealthStatus::Healthy;
        let _cloned = status.clone();

        let severity = AlertSeverity::Warning;
        let _cloned = severity.clone();

        let thresholds = HealthThresholds::default();
        let _cloned = thresholds.clone();

        let snapshot = HealthSnapshot {
            timestamp: UNIX_EPOCH,
            overall_status: HealthStatus::Healthy,
            components: HashMap::new(),
            alerts: Vec::new(),
        };
        let _cloned = snapshot.clone();

        let component = ComponentHealth {
            name: "test".to_string(),
            status: HealthStatus::Healthy,
            is_connected: true,
            events_processed: 100,
            error_count: 0,
            last_event_time: Some(UNIX_EPOCH),
            metrics: HashMap::new(),
        };
        let _cloned = component.clone();

        let alert = HealthAlert {
            severity: AlertSeverity::Warning,
            component: "test".to_string(),
            message: "Test message".to_string(),
            triggered_at: UNIX_EPOCH,
        };
        let _cloned = alert.clone();
    }

    #[test]
    fn test_copy_implementations() {
        // Test Copy implementations for enums
        let status = HealthStatus::Healthy;
        let _copied = status;
        let _also_copied = status; // Should work due to Copy trait

        let severity = AlertSeverity::Warning;
        let _copied = severity;
        let _also_copied = severity; // Should work due to Copy trait
    }
}
