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
