//! Production-ready utilities for stream management and monitoring.
//!
//! This module provides essential production features for managing streaming data pipelines:
//! - Health monitoring and status tracking
//! - Performance metrics collection and reporting  
//! - Resilience patterns including circuit breakers and retry policies
//!
//! These components work together to ensure reliable operation of streaming systems
//! in production environments with proper observability and fault tolerance.

pub mod health;
pub mod monitoring;
pub mod resilience;

pub use health::{ComponentHealth, HealthMonitor, HealthStatus};
pub use monitoring::{MetricsCollector, PerformanceStats, StreamMetrics};
pub use resilience::{BackoffStrategy, CircuitBreaker, RetryPolicy};
