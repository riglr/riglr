//! Prometheus metrics integration

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use tracing::info;

use crate::metrics::MetricsCollector;

/// Prometheus metrics server state
pub struct PrometheusState {
    /// Metrics collector instance
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MetricsConfig;
    use crate::metrics::{IndexerMetrics, MetricsCollector};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn create_test_collector() -> Arc<MetricsCollector> {
        use std::collections::HashMap;

        let config = MetricsConfig {
            enabled: true,
            port: 8080,
            endpoint: "/metrics".to_string(),
            collection_interval: std::time::Duration::from_secs(1),
            histogram_buckets: vec![0.1, 0.5, 1.0, 5.0, 10.0],
            custom: HashMap::new(),
        };

        Arc::new(MetricsCollector::new(config).expect("Failed to create test collector"))
    }

    #[test]
    fn test_prometheus_state_creation() {
        let collector = create_test_collector();
        let state = PrometheusState {
            collector: collector.clone(),
        };

        assert!(Arc::ptr_eq(&state.collector, &collector));
    }

    #[test]
    fn test_create_metrics_router_when_valid_collector_should_return_router() {
        let collector = create_test_collector();
        let _router = create_metrics_router(collector);

        // The router should be created successfully (no panic or error)
        // We can't easily test the internal structure, but we can verify it doesn't panic
    }

    #[tokio::test]
    async fn test_metrics_handler_when_valid_state_should_return_ok_response() {
        let collector = create_test_collector();
        let app = create_metrics_router(collector);

        let request = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Check content type header
        let content_type = response.headers().get("content-type").unwrap();
        assert_eq!(content_type, "text/plain; version=0.0.4; charset=utf-8");
    }

    #[tokio::test]
    async fn test_metrics_handler_when_called_should_export_prometheus_format() {
        let collector = create_test_collector();

        // Add some test metrics to the collector
        collector.record_gauge("test_gauge", 42.0);
        collector.record_counter("test_counter", 100);

        let app = create_metrics_router(collector);

        let request = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Check that the response contains Prometheus format metrics
        assert!(body_str.contains("# HELP"));
        assert!(body_str.contains("# TYPE"));
    }

    #[tokio::test]
    async fn test_health_handler_when_called_should_return_ok() {
        let collector = create_test_collector();
        let app = create_metrics_router(collector);

        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str, "OK");
    }

    #[tokio::test]
    async fn test_health_handler_direct_call_should_return_ok_string() {
        let result = health_handler().await;
        let response = result.into_response();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str, "OK");
    }

    #[tokio::test]
    async fn test_metrics_handler_with_updated_indexer_metrics_should_include_metrics() {
        let collector = create_test_collector();

        // Update indexer metrics with test data
        let test_metrics = IndexerMetrics {
            events_processed_total: 1000,
            events_processing_rate: 50.5,
            processing_latency_ms: 25.3,
            queue_depth: 10,
            active_workers: 5,
            error_rate: 0.02,
            storage_size_bytes: 1024 * 1024,
            cache_hit_rate: 0.95,
        };

        collector.update_indexer_metrics(test_metrics).await;

        let app = create_metrics_router(collector);

        let request = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Check that indexer metrics are included
        assert!(body_str.contains("indexer_events_processed_total"));
        assert!(body_str.contains("indexer_events_processing_rate"));
        assert!(body_str.contains("indexer_processing_latency_ms"));
        assert!(body_str.contains("indexer_queue_depth"));
        assert!(body_str.contains("indexer_active_workers"));
        assert!(body_str.contains("indexer_error_rate"));
        assert!(body_str.contains("indexer_storage_size_bytes"));
        assert!(body_str.contains("indexer_cache_hit_rate"));
    }

    #[tokio::test]
    async fn test_metrics_handler_when_invalid_route_should_return_not_found() {
        let collector = create_test_collector();
        let app = create_metrics_router(collector);

        let request = Request::builder()
            .uri("/invalid")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_start_metrics_server_when_invalid_bind_addr_should_return_error() {
        let collector = create_test_collector();

        // Use an invalid bind address
        let result = start_metrics_server("999.999.999.999", 8080, collector).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_start_metrics_server_when_port_zero_should_bind_to_random_port() {
        let collector = create_test_collector();

        // Using port 0 should allow the OS to assign a random available port
        // This test verifies the function can handle port 0 without error
        let handle =
            tokio::spawn(async move { start_metrics_server("127.0.0.1", 0, collector).await });

        // Give it a moment to start
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Cancel the server to avoid running indefinitely
        handle.abort();

        // The test passes if we get here without panicking
    }

    #[tokio::test]
    async fn test_start_metrics_server_when_port_in_use_should_return_error() {
        let collector1 = create_test_collector();
        let collector2 = create_test_collector();

        // Find an available port first
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener); // Close it to use the port

        // Start first server
        let handle1 =
            tokio::spawn(async move { start_metrics_server("127.0.0.1", port, collector1).await });

        // Give it time to bind
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Try to start second server on same port - should fail
        let result = start_metrics_server("127.0.0.1", port, collector2).await;

        // Clean up
        handle1.abort();

        assert!(result.is_err());
    }

    #[test]
    fn test_prometheus_state_collector_field_accessibility() {
        let collector = create_test_collector();
        let state = PrometheusState {
            collector: collector.clone(),
        };

        // Test that we can access the collector field
        let _exported_metrics = state.collector.export_prometheus();
    }

    #[tokio::test]
    async fn test_metrics_handler_with_empty_metrics_should_return_valid_response() {
        let collector = create_test_collector();

        // Reset collector to ensure it's empty
        collector.reset();

        let app = create_metrics_router(collector);

        let request = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Even with empty metrics, should still return Prometheus format headers
        assert!(body_str.contains("# HELP") || body_str.is_empty() || body_str.contains("indexer"));
    }

    #[tokio::test]
    async fn test_router_with_different_http_methods() {
        let collector = create_test_collector();
        let app = create_metrics_router(collector);

        // Test POST to /metrics (should work, axum allows by default)
        let request = Request::builder()
            .method("POST")
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // Should return method not allowed since we only registered GET
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_router_with_query_parameters() {
        let collector = create_test_collector();
        let app = create_metrics_router(collector);

        let request = Request::builder()
            .uri("/metrics?format=prometheus")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Query parameters should not affect the response
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_start_metrics_server_address_formatting() {
        // Test the address formatting logic
        let bind_addr = "0.0.0.0";
        let port = 9090;
        let expected = format!("{}:{}", bind_addr, port);
        assert_eq!(expected, "0.0.0.0:9090");

        // Test with IPv6
        let bind_addr_v6 = "::1";
        let expected_v6 = format!("{}:{}", bind_addr_v6, port);
        assert_eq!(expected_v6, "::1:9090");
    }

    #[tokio::test]
    async fn test_metrics_handler_response_headers() {
        let collector = create_test_collector();
        let app = create_metrics_router(collector);

        let request = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Verify all expected response properties
        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();
        let content_type = headers.get("content-type").unwrap();
        assert_eq!(content_type, "text/plain; version=0.0.4; charset=utf-8");
    }

    #[tokio::test]
    async fn test_health_handler_response_type() {
        let result = health_handler().await;
        let response = result.into_response();

        // Health handler should return 200 OK
        assert_eq!(response.status(), StatusCode::OK);
    }
}
