//! REST API handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::core::ServiceContext;
use crate::error::IndexerResult;
use crate::storage::{EventFilter, EventQuery, StoredEvent};

/// REST API handler
pub struct RestHandler {
    context: Arc<ServiceContext>,
}

/// API response wrapper
#[derive(Serialize)]
pub struct ApiResponse<T> {
    /// Whether the request was successful
    pub success: bool,
    /// Response data if successful
    pub data: Option<T>,
    /// Error message if unsuccessful
    pub error: Option<String>,
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    /// Create an error response
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: Utc::now(),
        }
    }
}

/// Query parameters for events endpoint
#[derive(Debug, Deserialize)]
pub struct EventsQueryParams {
    /// Event types to filter by
    pub event_types: Option<String>, // Comma-separated
    /// Sources to filter by
    pub sources: Option<String>, // Comma-separated
    /// Start time for filtering
    pub start_time: Option<DateTime<Utc>>,
    /// End time for filtering
    pub end_time: Option<DateTime<Utc>>,
    /// Minimum block height
    pub min_block: Option<u64>,
    /// Maximum block height
    pub max_block: Option<u64>,
    /// Transaction hash
    pub tx_hash: Option<String>,
    /// Number of results to return
    pub limit: Option<usize>,
    /// Number of results to skip
    pub offset: Option<usize>,
    /// Sort field
    pub sort_by: Option<String>,
    /// Sort direction (asc/desc)
    pub sort_dir: Option<String>,
}

/// Event statistics response
#[derive(Serialize)]
pub struct EventStatsResponse {
    /// Total number of events indexed
    pub total_events: u64,
    /// Event counts grouped by type
    pub events_by_type: HashMap<String, u64>,
    /// Event counts grouped by source
    pub events_by_source: HashMap<String, u64>,
    /// Storage layer statistics
    pub storage_stats: crate::storage::StorageStats,
}

/// Service health response
#[derive(Serialize)]
pub struct HealthResponse {
    /// Overall health status (healthy/unhealthy/degraded)
    pub status: String,
    /// Health check timestamp
    pub timestamp: DateTime<Utc>,
    /// Individual component health statuses
    pub components: HashMap<String, ComponentStatus>,
    /// Service uptime in seconds
    pub uptime_seconds: u64,
    /// Service version
    pub version: String,
}

/// Component status
#[derive(Serialize)]
pub struct ComponentStatus {
    /// Whether the component is healthy
    pub healthy: bool,
    /// Status message describing component state
    pub message: String,
    /// Timestamp of last successful operation
    pub last_success: Option<DateTime<Utc>>,
}

impl RestHandler {
    /// Create a new REST handler
    pub fn new(context: Arc<ServiceContext>) -> IndexerResult<Self> {
        Ok(Self { context })
    }

    /// Create the router for REST endpoints
    pub fn create_router(&self) -> Router<Arc<ServiceContext>> {
        Router::new()
            .route("/api/v1/events", get(get_events))
            .route("/api/v1/events/:id", get(get_event))
            .route("/api/v1/events/stats", get(get_event_stats))
            .route("/api/v1/health", get(get_health))
            .route("/api/v1/metrics", get(get_metrics))
            .with_state(self.context.clone())
    }
}

/// Get events endpoint
pub async fn get_events(
    State(context): State<Arc<ServiceContext>>,
    Query(params): Query<EventsQueryParams>,
) -> Result<Json<ApiResponse<Vec<StoredEvent>>>, StatusCode> {
    debug!("GET /api/v1/events with params: {:?}", params);

    // Build filter from query parameters
    let mut filter = EventFilter::default();

    if let Some(event_types) = params.event_types {
        filter.event_types = Some(
            event_types
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        );
    }

    if let Some(sources) = params.sources {
        filter.sources = Some(sources.split(',').map(|s| s.trim().to_string()).collect());
    }

    if let (Some(start), Some(end)) = (params.start_time, params.end_time) {
        filter.time_range = Some((start, end));
    }

    if let (Some(min_block), Some(max_block)) = (params.min_block, params.max_block) {
        filter.block_height_range = Some((min_block, max_block));
    }

    if let Some(tx_hash) = params.tx_hash {
        filter.transaction_hash = Some(tx_hash);
    }

    // Build query
    let query = EventQuery {
        filter,
        limit: params.limit.or(Some(100)), // Default limit
        offset: params.offset,
        sort: params.sort_by.map(|field| {
            let ascending = params
                .sort_dir
                .as_deref()
                .unwrap_or("desc")
                .eq_ignore_ascii_case("asc");
            (field, ascending)
        }),
    };

    // Execute query
    match context.store.query_events(&query).await {
        Ok(events) => {
            info!("Returned {} events", events.len());
            Ok(Json(ApiResponse::success(events)))
        }
        Err(e) => {
            error!("Failed to query events: {}", e);
            Ok(Json(ApiResponse::<Vec<StoredEvent>>::error(e.to_string())))
        }
    }
}

/// Get single event endpoint
pub async fn get_event(
    State(context): State<Arc<ServiceContext>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Option<StoredEvent>>>, StatusCode> {
    debug!("GET /api/v1/events/{}", id);

    match context.store.get_event(&id).await {
        Ok(event) => {
            if event.is_some() {
                info!("Found event {}", id);
            } else {
                info!("Event {} not found", id);
            }
            Ok(Json(ApiResponse::success(event)))
        }
        Err(e) => {
            error!("Failed to get event {}: {}", id, e);
            Ok(Json(ApiResponse::<Option<StoredEvent>>::error(
                e.to_string(),
            )))
        }
    }
}

/// Get event statistics endpoint
pub async fn get_event_stats(
    State(context): State<Arc<ServiceContext>>,
) -> Result<Json<ApiResponse<EventStatsResponse>>, StatusCode> {
    debug!("GET /api/v1/events/stats");

    // Get total event count
    let total_events = match context.store.count_events(&EventFilter::default()).await {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to get event count: {}", e);
            return Ok(Json(ApiResponse::<EventStatsResponse>::error(
                e.to_string(),
            )));
        }
    };

    // Get aggregated statistics
    let events_by_type = match crate::storage::EventAggregator::aggregate_by_type(
        context.store.as_ref(),
        &EventFilter::default(),
    )
    .await
    {
        Ok(stats) => stats,
        Err(e) => {
            error!("Failed to get events by type: {}", e);
            HashMap::new()
        }
    };

    let events_by_source = match crate::storage::EventAggregator::aggregate_by_source(
        context.store.as_ref(),
        &EventFilter::default(),
    )
    .await
    {
        Ok(stats) => stats,
        Err(e) => {
            error!("Failed to get events by source: {}", e);
            HashMap::new()
        }
    };

    // Get storage statistics
    let storage_stats = match context.store.get_stats().await {
        Ok(stats) => stats,
        Err(e) => {
            error!("Failed to get storage stats: {}", e);
            return Ok(Json(ApiResponse::<EventStatsResponse>::error(
                e.to_string(),
            )));
        }
    };

    let response = EventStatsResponse {
        total_events,
        events_by_type,
        events_by_source,
        storage_stats,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get service health endpoint
pub async fn get_health(
    State(context): State<Arc<ServiceContext>>,
) -> Result<Json<ApiResponse<HealthResponse>>, StatusCode> {
    debug!("GET /api/v1/health");

    match context.health_check().await {
        Ok(health_status) => {
            let mut components = HashMap::new();

            for (name, component_health) in health_status.components {
                components.insert(
                    name,
                    ComponentStatus {
                        healthy: component_health.healthy,
                        message: component_health.message,
                        last_success: component_health.last_success,
                    },
                );
            }

            let response = HealthResponse {
                status: if health_status.healthy {
                    "healthy".to_string()
                } else {
                    "unhealthy".to_string()
                },
                timestamp: health_status.timestamp,
                components,
                uptime_seconds: 0, // Would need to track service start time
                version: context.config.service.version.clone(),
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Health check failed: {}", e);
            Ok(Json(ApiResponse::<HealthResponse>::error(e.to_string())))
        }
    }
}

/// Get metrics endpoint
pub async fn get_metrics(
    State(context): State<Arc<ServiceContext>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    debug!("GET /api/v1/metrics");

    match context.metrics.export_json() {
        Ok(metrics) => Ok(Json(ApiResponse::success(metrics))),
        Err(e) => {
            error!("Failed to export metrics: {}", e);
            Ok(Json(ApiResponse::<serde_json::Value>::error(e.to_string())))
        }
    }
}
