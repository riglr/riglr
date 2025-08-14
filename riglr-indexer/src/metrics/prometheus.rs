//! Prometheus metrics integration

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use tracing::{info, error};

use crate::metrics::MetricsCollector;

/// Prometheus metrics server state
pub struct PrometheusState {
    pub collector: Arc<MetricsCollector>,
}

/// Create Prometheus metrics router
pub fn create_metrics_router(collector: Arc<MetricsCollector>) -> Router {
    let state = PrometheusState { collector };
    
    Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .with_state(Arc::new(state))
}

/// Prometheus metrics endpoint handler
async fn metrics_handler(
    State(state): State<Arc<PrometheusState>>,
) -> Result<Response, StatusCode> {
    let metrics_data = state.collector.export_prometheus();
    
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/plain; version=0.0.4; charset=utf-8")
        .body(metrics_data.into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(response)
}

/// Health check endpoint
async fn health_handler() -> impl IntoResponse {
    "OK"
}

/// Start Prometheus metrics server
pub async fn start_metrics_server(
    bind_addr: &str,
    port: u16,
    collector: Arc<MetricsCollector>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("{}:{}", bind_addr, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    info!("Starting Prometheus metrics server on {}", addr);
    
    let app = create_metrics_router(collector);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}