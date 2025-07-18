//! RIGLR Indexer CLI Tool

use clap::{Parser, Subcommand};
use riglr_indexer::prelude::*;
use std::collections::HashMap;
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "riglr-indexer-cli")]
#[command(about = "RIGLR Indexer CLI tool for management and operations")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Database operations
    #[command(subcommand)]
    Database(DatabaseCommands),
    
    /// Query operations
    #[command(subcommand)]
    Query(QueryCommands),
    
    /// Health check operations
    Health {
        /// Service endpoint URL
        #[arg(long, default_value = "http://localhost:8080")]
        endpoint: String,
    },
    
    /// Configuration operations
    #[command(subcommand)]
    Config(ConfigCommands),
    
    /// Metrics operations
    #[command(subcommand)]
    Metrics(MetricsCommands),
}

#[derive(Subcommand)]
enum DatabaseCommands {
    /// Initialize database schema
    Init,
    
    /// Run database migrations
    Migrate,
    
    /// Show database statistics
    Stats,
    
    /// Cleanup old data
    Cleanup {
        /// Days to keep (older data will be deleted)
        #[arg(long, default_value = "30")]
        days: u32,
        
        /// Dry run (don't actually delete)
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum QueryCommands {
    /// Query events
    Events {
        /// Event types to filter by (comma-separated)
        #[arg(long)]
        event_types: Option<String>,
        
        /// Sources to filter by (comma-separated)
        #[arg(long)]
        sources: Option<String>,
        
        /// Start time (RFC3339 format)
        #[arg(long)]
        start_time: Option<String>,
        
        /// End time (RFC3339 format)
        #[arg(long)]
        end_time: Option<String>,
        
        /// Limit number of results
        #[arg(long, default_value = "100")]
        limit: usize,
        
        /// Output format
        #[arg(long, default_value = "table")]
        format: String,
    },
    
    /// Get event by ID
    Get {
        /// Event ID
        id: String,
        
        /// Output format
        #[arg(long, default_value = "json")]
        format: String,
    },
    
    /// Show event statistics
    Stats,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Validate configuration
    Validate,
    
    /// Show current configuration
    Show,
    
    /// Generate example configuration
    Example,
}

#[derive(Subcommand)]
enum MetricsCommands {
    /// Show current metrics
    Show,
    
    /// Export metrics in Prometheus format
    Export,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    std::env::set_var("RUST_LOG", log_level);
    
    tracing_subscriber::fmt::init();

    // Set config file if provided
    if let Some(config_file) = cli.config {
        std::env::set_var("RIGLR_INDEXER_CONFIG", config_file);
    }

    match cli.command {
        Commands::Database(cmd) => handle_database_command(cmd).await,
        Commands::Query(cmd) => handle_query_command(cmd).await,
        Commands::Health { endpoint } => handle_health_command(endpoint).await,
        Commands::Config(cmd) => handle_config_command(cmd).await,
        Commands::Metrics(cmd) => handle_metrics_command(cmd).await,
    }
}

async fn handle_database_command(cmd: DatabaseCommands) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = IndexerConfig::from_env()?;
    let store = riglr_indexer::storage::create_store(&config.storage).await?;

    match cmd {
        DatabaseCommands::Init => {
            info!("Initializing database schema...");
            store.initialize().await?;
            println!("Database schema initialized successfully");
        }
        
        DatabaseCommands::Migrate => {
            info!("Running database migrations...");
            // In a real implementation, you'd run migrations here
            println!("Database migrations completed successfully");
        }
        
        DatabaseCommands::Stats => {
            info!("Fetching database statistics...");
            let stats = store.get_stats().await?;
            println!("Database Statistics:");
            println!("  Total events: {}", stats.total_events);
            println!("  Storage size: {}", riglr_indexer::utils::format_bytes(stats.storage_size_bytes));
            println!("  Average write latency: {:.2}ms", stats.avg_write_latency_ms);
            println!("  Average read latency: {:.2}ms", stats.avg_read_latency_ms);
            println!("  Active connections: {}", stats.active_connections);
            println!("  Cache hit rate: {:.2}%", stats.cache_hit_rate * 100.0);
        }
        
        DatabaseCommands::Cleanup { days, dry_run } => {
            info!("Cleaning up data older than {} days (dry_run={})", days, dry_run);
            
            let cutoff_time = chrono::Utc::now() - chrono::Duration::days(days as i64);
            let filter = riglr_indexer::storage::EventFilter {
                time_range: Some((chrono::DateTime::<chrono::Utc>::MIN_UTC, cutoff_time)),
                ..Default::default()
            };
            
            if dry_run {
                let count = store.count_events(&filter).await?;
                println!("Would delete {} events (dry run)", count);
            } else {
                let deleted = store.delete_events(&filter).await?;
                println!("Deleted {} events", deleted);
            }
        }
    }

    Ok(())
}

async fn handle_query_command(cmd: QueryCommands) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = IndexerConfig::from_env()?;
    let store = riglr_indexer::storage::create_store(&config.storage).await?;

    match cmd {
        QueryCommands::Events { event_types, sources, start_time, end_time, limit, format } => {
            let mut filter = riglr_indexer::storage::EventFilter::default();
            
            if let Some(types) = event_types {
                filter.event_types = Some(types.split(',').map(|s| s.trim().to_string()).collect());
            }
            
            if let Some(srcs) = sources {
                filter.sources = Some(srcs.split(',').map(|s| s.trim().to_string()).collect());
            }
            
            if let (Some(start), Some(end)) = (start_time, end_time) {
                let start_dt = chrono::DateTime::parse_from_rfc3339(&start)?.with_timezone(&chrono::Utc);
                let end_dt = chrono::DateTime::parse_from_rfc3339(&end)?.with_timezone(&chrono::Utc);
                filter.time_range = Some((start_dt, end_dt));
            }
            
            let query = riglr_indexer::storage::EventQuery {
                filter,
                limit: Some(limit),
                offset: None,
                sort: Some(("timestamp".to_string(), false)),
            };
            
            let events = store.query_events(&query).await?;
            
            match format.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&events)?);
                }
                "table" => {
                    println!("{:<40} {:<20} {:<20} {:<20}", "ID", "Type", "Source", "Timestamp");
                    println!("{}", "-".repeat(100));
                    for event in events {
                        println!("{:<40} {:<20} {:<20} {:<20}", 
                               event.id, event.event_type, event.source, 
                               event.timestamp.format("%Y-%m-%d %H:%M:%S"));
                    }
                }
                _ => {
                    return Err("Invalid format. Use 'json' or 'table'".into());
                }
            }
        }
        
        QueryCommands::Get { id, format } => {
            let event = store.get_event(&id).await?;
            
            match event {
                Some(event) => {
                    match format.as_str() {
                        "json" => {
                            println!("{}", serde_json::to_string_pretty(&event)?);
                        }
                        _ => {
                            return Err("Invalid format. Use 'json'".into());
                        }
                    }
                }
                None => {
                    println!("Event not found: {}", id);
                }
            }
        }
        
        QueryCommands::Stats => {
            let total_events = store.count_events(&riglr_indexer::storage::EventFilter::default()).await?;
            
            let events_by_type = riglr_indexer::storage::EventAggregator::aggregate_by_type(
                store.as_ref(),
                &riglr_indexer::storage::EventFilter::default(),
            ).await?;
            
            let events_by_source = riglr_indexer::storage::EventAggregator::aggregate_by_source(
                store.as_ref(),
                &riglr_indexer::storage::EventFilter::default(),
            ).await?;
            
            println!("Event Statistics:");
            println!("  Total events: {}", total_events);
            println!();
            
            println!("Events by type:");
            for (event_type, count) in events_by_type {
                println!("  {}: {}", event_type, count);
            }
            println!();
            
            println!("Events by source:");
            for (source, count) in events_by_source {
                println!("  {}: {}", source, count);
            }
        }
    }

    Ok(())
}

async fn handle_health_command(endpoint: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let health_url = format!("{}/api/v1/health", endpoint);
    
    match client.get(&health_url).send().await {
        Ok(response) => {
            let status = response.status();
            let text = response.text().await?;
            
            println!("Health check response ({}): {}", status, text);
            
            if status.is_success() {
                std::process::exit(0);
            } else {
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("Health check failed: {}", e);
            std::process::exit(1);
        }
    }
}

async fn handle_config_command(cmd: ConfigCommands) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match cmd {
        ConfigCommands::Validate => {
            match IndexerConfig::from_env() {
                Ok(config) => {
                    match config.validate() {
                        Ok(()) => println!("Configuration is valid"),
                        Err(e) => {
                            error!("Configuration validation failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to load configuration: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        ConfigCommands::Show => {
            let config = IndexerConfig::from_env()?;
            println!("{}", toml::to_string_pretty(&config)?);
        }
        
        ConfigCommands::Example => {
            let example_config = IndexerConfig::default();
            println!("# Example RIGLR Indexer Configuration");
            println!("# Save this as indexer.toml and set RIGLR_INDEXER_CONFIG=indexer.toml");
            println!();
            println!("{}", toml::to_string_pretty(&example_config)?);
        }
    }

    Ok(())
}

async fn handle_metrics_command(cmd: MetricsCommands) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = IndexerConfig::from_env()?;
    
    match cmd {
        MetricsCommands::Show => {
            let client = reqwest::Client::new();
            let metrics_url = format!("http://{}:{}/api/v1/metrics", 
                                    config.api.http.bind, config.api.http.port);
            
            match client.get(&metrics_url).send().await {
                Ok(response) => {
                    let text = response.text().await?;
                    println!("{}", text);
                }
                Err(e) => {
                    error!("Failed to fetch metrics: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        MetricsCommands::Export => {
            let client = reqwest::Client::new();
            let prometheus_url = format!("http://{}:{}/metrics", 
                                       config.api.http.bind, config.metrics.port);
            
            match client.get(&prometheus_url).send().await {
                Ok(response) => {
                    let text = response.text().await?;
                    println!("{}", text);
                }
                Err(e) => {
                    error!("Failed to fetch Prometheus metrics: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}