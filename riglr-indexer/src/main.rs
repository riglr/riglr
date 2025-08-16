//! RIGLR Indexer Service Main Entry Point

use tracing::{info, error, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use riglr_indexer::prelude::*;
use riglr_indexer::core::ServiceLifecycle;

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
                &bind_addr,
                port,
                collector,
            ).await {
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
    info!("Waiting up to {:?} for services to shut down", shutdown_timeout);

    tokio::time::timeout(shutdown_timeout, async {
        // Wait for API server to stop
        if let Some(task) = api_task {
            task.await.ok();
        }

        // Wait for metrics server to stop  
        if let Some(task) = metrics_task {
            task.await.ok();
        }
    }).await.ok();

    info!("RIGLR Indexer Service stopped");
    Ok(())
}

/// Initialize logging based on configuration
fn init_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

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
    println!("║                        Version {}                        ║", env!("CARGO_PKG_VERSION"));
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
    info!("  Rate limiting: {} events/sec", 
          if config.processing.rate_limit.enabled {
              config.processing.rate_limit.max_events_per_second.to_string()
          } else {
              "disabled".to_string()
          });
    
    info!("Storage Configuration:");
    info!("  Backend: {:?}", config.storage.primary.backend);
    info!("  URL: {}", redact_url(&config.storage.primary.url));
    info!("  Pool size: {}-{}", 
          config.storage.primary.pool.min_connections,
          config.storage.primary.pool.max_connections);
    
    info!("API Configuration:");
    info!("  HTTP server: {}:{}", config.api.http.bind, config.api.http.port);
    info!("  WebSocket: {}", if config.api.websocket.enabled { "enabled" } else { "disabled" });
    info!("  Authentication: {:?}", config.api.auth.method);
    
    info!("Metrics Configuration:");
    info!("  Enabled: {}", config.metrics.enabled);
    if config.metrics.enabled {
        info!("  Port: {}", config.metrics.port);
        info!("  Endpoint: {}", config.metrics.endpoint);
    }
    
    info!("Features:");
    info!("  Real-time streaming: {}", config.features.realtime_streaming);
    info!("  Event archival: {}", config.features.archival);
    info!("  GraphQL API: {}", config.features.graphql_api);
    info!("  Experimental: {}", config.features.experimental);
}

/// Redact sensitive information from URLs
fn redact_url(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        let mut redacted = parsed.clone();
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
        
        let mut sigterm = signal(SignalKind::terminate())
            .expect("Failed to register SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt())
            .expect("Failed to register SIGINT handler");
        let mut sigquit = signal(SignalKind::quit())
            .expect("Failed to register SIGQUIT handler");

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
        signal::ctrl_c().await.expect("Failed to register Ctrl+C handler");
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
    fn test_redact_url() {
        assert_eq!(
            redact_url("postgresql://user:password@localhost:5432/db"),
            "postgresql://user:****@localhost:5432/db"
        );
        
        assert_eq!(
            redact_url("redis://localhost:6379"),
            "redis://localhost:6379"
        );
        
        assert_eq!(
            redact_url("invalid-url"),
            "invalid-url"
        );
    }
}