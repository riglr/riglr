//! RIGLR Indexer Service Main Entry Point

use tracing::{error, info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use riglr_indexer::core::ServiceLifecycle;
use riglr_indexer::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing/logging
    init_logging()?;

    // Print banner
    print_banner();

    // Load configuration
    let config = match IndexerConfig::from_env() {
        Ok(config) => {
            info!("Configuration loaded successfully");
            config
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Print configuration summary
    print_config_summary(&config);

    // Initialize and start the indexer service
    let mut indexer = match IndexerService::new(config.clone()).await {
        Ok(indexer) => {
            info!("Indexer service initialized successfully");
            indexer
        }
        Err(e) => {
            error!("Failed to initialize indexer service: {}", e);
            std::process::exit(1);
        }
    };

    // Start metrics server if enabled
    let metrics_task = if config.metrics.enabled {
        let collector = indexer.context().metrics.clone();
        let bind_addr = config.api.http.bind.clone();
        let port = config.metrics.port;

        Some(tokio::spawn(async move {
            info!("Starting metrics server on {}:{}", bind_addr, port);
            if let Err(e) = riglr_indexer::metrics::prometheus::start_metrics_server(
                &bind_addr, port, collector,
            )
            .await
            {
                error!("Metrics server failed: {}", e);
            }
        }))
    } else {
        None
    };

    // Start API server if enabled
    let api_task = if config.api.http.port > 0 {
        let api_server = match riglr_indexer::api::ApiServer::new(indexer.context().clone()) {
            Ok(server) => server,
            Err(e) => {
                error!("Failed to create API server: {}", e);
                std::process::exit(1);
            }
        };

        Some(tokio::spawn(async move {
            if let Err(e) = api_server.start().await {
                error!("API server failed: {}", e);
            }
        }))
    } else {
        None
    };

    // Start the main indexer service
    let indexer_context = indexer.context().clone();
    let indexer_task = tokio::spawn(async move {
        if let Err(e) = indexer.start().await {
            error!("Indexer service failed: {}", e);
        }
    });

    // Setup signal handling
    let mut shutdown_signal = setup_signal_handling().await;

    // Wait for shutdown signal
    tokio::select! {
        _ = shutdown_signal.recv() => {
            info!("Shutdown signal received, initiating graceful shutdown");
        }
        _ = indexer_task => {
            warn!("Indexer service task completed unexpectedly");
        }
    }

    // Initiate graceful shutdown
    if let Err(e) = indexer_context.request_shutdown() {
        error!("Failed to request shutdown: {}", e);
    }

    // Wait for services to stop with timeout
    let shutdown_timeout = config.service.shutdown_timeout;
    info!(
        "Waiting up to {:?} for services to shut down",
        shutdown_timeout
    );

    tokio::time::timeout(shutdown_timeout, async {
        // Wait for API server to stop
        if let Some(task) = api_task {
            task.await.ok();
        }

        // Wait for metrics server to stop
        if let Some(task) = metrics_task {
            task.await.ok();
        }
    })
    .await
    .ok();

    info!("RIGLR Indexer Service stopped");
    Ok(())
}

/// Initialize logging based on configuration
fn init_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    Ok(())
}

/// Print startup banner
fn print_banner() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                     RIGLR INDEXER SERVICE                   ║");
    println!("║              Production-Grade Blockchain Indexing           ║");
    println!(
        "║                        Version {}                        ║",
        env!("CARGO_PKG_VERSION")
    );
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
}

/// Print configuration summary
fn print_config_summary(config: &IndexerConfig) {
    info!("Service Configuration:");
    info!("  Name: {}", config.service.name);
    info!("  Version: {}", config.service.version);
    info!("  Environment: {}", config.service.environment);
    info!("  Node ID: {}", config.node_id());

    info!("Processing Configuration:");
    info!("  Workers: {}", config.processing.workers);
    info!("  Batch size: {}", config.processing.batch.max_size);
    info!("  Queue capacity: {}", config.processing.queue.capacity);
    info!(
        "  Rate limiting: {} events/sec",
        if config.processing.rate_limit.enabled {
            config
                .processing
                .rate_limit
                .max_events_per_second
                .to_string()
        } else {
            "disabled".to_string()
        }
    );

    info!("Storage Configuration:");
    info!("  Backend: {:?}", config.storage.primary.backend);
    info!("  URL: {}", redact_url(&config.storage.primary.url));
    info!(
        "  Pool size: {}-{}",
        config.storage.primary.pool.min_connections, config.storage.primary.pool.max_connections
    );

    info!("API Configuration:");
    info!(
        "  HTTP server: {}:{}",
        config.api.http.bind, config.api.http.port
    );
    info!(
        "  WebSocket: {}",
        if config.api.websocket.enabled {
            "enabled"
        } else {
            "disabled"
        }
    );
    info!("  Authentication: {:?}", config.api.auth.method);

    info!("Metrics Configuration:");
    info!("  Enabled: {}", config.metrics.enabled);
    if config.metrics.enabled {
        info!("  Port: {}", config.metrics.port);
        info!("  Endpoint: {}", config.metrics.endpoint);
    }

    info!("Features:");
    info!(
        "  Real-time streaming: {}",
        config.features.realtime_streaming
    );
    info!("  Event archival: {}", config.features.archival);
    info!("  GraphQL API: {}", config.features.graphql_api);
    info!("  Experimental: {}", config.features.experimental);
}

/// Redact sensitive information from URLs
fn redact_url(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        let mut redacted = parsed;
        if redacted.password().is_some() {
            let _ = redacted.set_password(Some("****"));
        }
        redacted.to_string()
    } else {
        // If URL parsing fails, just redact after '@' if present
        if let Some(_at_pos) = url.find('@') {
            let parts: Vec<&str> = url.split('@').collect();
            if parts.len() >= 2 {
                format!("****@{}", parts[1..].join("@"))
            } else {
                url.to_string()
            }
        } else {
            url.to_string()
        }
    }
}

/// Setup signal handling for graceful shutdown
async fn setup_signal_handling() -> tokio::signal::unix::Signal {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm =
            signal(SignalKind::terminate()).expect("Failed to register SIGTERM handler");
        let mut sigint =
            signal(SignalKind::interrupt()).expect("Failed to register SIGINT handler");
        let mut sigquit = signal(SignalKind::quit()).expect("Failed to register SIGQUIT handler");

        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM");
            }
            _ = sigint.recv() => {
                info!("Received SIGINT (Ctrl+C)");
            }
            _ = sigquit.recv() => {
                info!("Received SIGQUIT");
            }
        }

        sigterm
    }

    #[cfg(not(unix))]
    {
        signal::ctrl_c()
            .await
            .expect("Failed to register Ctrl+C handler");
        info!("Received Ctrl+C");

        // Return a dummy signal that's immediately ready
        use tokio::signal::unix::{signal, SignalKind};
        signal(SignalKind::interrupt()).expect("Failed to create dummy signal")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logging_when_valid_should_return_ok() {
        // Happy path - should initialize logging successfully
        let result = init_logging();
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_banner_when_called_should_execute_without_panic() {
        // Test that print_banner executes without panicking
        // This function prints to stdout, so we just verify it doesn't panic
        print_banner();
    }

    #[test]
    fn test_print_config_summary_when_valid_config_should_execute_without_panic() {
        // Create a mock config for testing
        use riglr_indexer::config::*;

        // We need to create a minimal valid config
        // This tests the function execution without panicking
        let config = IndexerConfig::default();
        print_config_summary(&config);
    }

    #[test]
    fn test_redact_url_when_valid_url_with_password_should_redact_password() {
        // Happy path - URL with password should be redacted
        assert_eq!(
            redact_url("postgresql://user:password@localhost:5432/db"),
            "postgresql://user:****@localhost:5432/db"
        );

        assert_eq!(
            redact_url("mysql://admin:secret123@database.example.com:3306/mydb"),
            "mysql://admin:****@database.example.com:3306/mydb"
        );
    }

    #[test]
    fn test_redact_url_when_valid_url_without_password_should_remain_unchanged() {
        // URL without password should remain unchanged
        assert_eq!(
            redact_url("redis://localhost:6379"),
            "redis://localhost:6379"
        );

        assert_eq!(
            redact_url("http://example.com:8080"),
            "http://example.com:8080"
        );
    }

    #[test]
    fn test_redact_url_when_invalid_url_should_remain_unchanged() {
        // Invalid URL without @ symbol should remain unchanged
        assert_eq!(redact_url("invalid-url"), "invalid-url");
        assert_eq!(redact_url("just-a-string"), "just-a-string");
        assert_eq!(redact_url(""), "");
    }

    #[test]
    fn test_redact_url_when_invalid_url_with_at_symbol_should_redact_before_at() {
        // Invalid URL with @ symbol should redact everything before @
        assert_eq!(
            redact_url("user:pass@some-invalid-url"),
            "****@some-invalid-url"
        );

        assert_eq!(
            redact_url("complex:password:with:colons@server"),
            "****@server"
        );
    }

    #[test]
    fn test_redact_url_when_multiple_at_symbols_should_handle_correctly() {
        // Test edge case with multiple @ symbols
        assert_eq!(redact_url("user:pass@server@domain"), "****@server@domain");
    }

    #[test]
    fn test_redact_url_when_only_at_symbol_should_handle_gracefully() {
        // Edge case: just an @ symbol
        assert_eq!(redact_url("@"), "****@");
    }

    #[test]
    fn test_redact_url_when_url_with_username_only_should_remain_unchanged() {
        // URL with username but no password
        let result = redact_url("postgresql://user@localhost:5432/db");
        assert_eq!(result, "postgresql://user@localhost:5432/db");
    }

    #[test]
    fn test_redact_url_when_complex_valid_url_with_query_params_should_redact_password() {
        // Complex URL with query parameters and fragments
        let result =
            redact_url("https://user:password@api.example.com:443/path?param=value#fragment");
        assert_eq!(
            result,
            "https://user:****@api.example.com:443/path?param=value#fragment"
        );
    }

    #[test]
    fn test_redact_url_when_url_with_empty_password_should_redact() {
        // URL with empty password (user:@host)
        let result = redact_url("mongodb://user:@localhost:27017/db");
        assert_eq!(result, "mongodb://user:****@localhost:27017/db");
    }

    #[test]
    fn test_redact_url_when_special_characters_in_password_should_redact() {
        // Password with special characters
        let result = redact_url("redis://user:p@ss!w0rd@redis.example.com:6379");
        assert_eq!(result, "redis://user:****@redis.example.com:6379");
    }

    #[test]
    fn test_setup_signal_handling_when_compiled_should_not_panic() {
        // This function is async and platform-specific, so we just test that it compiles
        // We can't easily test the actual signal handling in a unit test environment
        // The function will be tested through integration tests or manual testing

        // This is a compile-time test - if the function compiles, the test passes
        // We just verify the function exists and can be referenced
        let _function_exists = setup_signal_handling;
        // If this compiles, the test passes
    }
}
