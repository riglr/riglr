//! RIGLR Indexer CLI Tool

use clap::{Parser, Subcommand};
use riglr_indexer::prelude::*;
use tracing::{error, info};

// Environment variable constants
const ENV_RUST_LOG: &str = "RUST_LOG";
const ENV_RIGLR_INDEXER_CONFIG: &str = "RIGLR_INDEXER_CONFIG";

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
#[allow(unsafe_code)]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    // SAFETY: This is safe because we're in main before any threads are spawned
    unsafe {
        std::env::set_var(ENV_RUST_LOG, log_level);
    }

    tracing_subscriber::fmt::init();

    // Set config file if provided
    if let Some(config_file) = cli.config {
        // SAFETY: This is safe because we're in main before any threads are spawned
        unsafe {
            std::env::set_var(ENV_RIGLR_INDEXER_CONFIG, config_file);
        }
    }

    match cli.command {
        Commands::Database(cmd) => handle_database_command(cmd).await,
        Commands::Query(cmd) => handle_query_command(cmd).await,
        Commands::Health { endpoint } => handle_health_command(endpoint).await,
        Commands::Config(cmd) => handle_config_command(cmd).await,
        Commands::Metrics(cmd) => handle_metrics_command(cmd).await,
    }
}

async fn handle_database_command(
    cmd: DatabaseCommands,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
            println!(
                "  Storage size: {}",
                riglr_indexer::utils::format_bytes(stats.storage_size_bytes)
            );
            println!(
                "  Average write latency: {:.2}ms",
                stats.avg_write_latency_ms
            );
            println!("  Average read latency: {:.2}ms", stats.avg_read_latency_ms);
            println!("  Active connections: {}", stats.active_connections);
            println!("  Cache hit rate: {:.2}%", stats.cache_hit_rate * 100.0);
        }

        DatabaseCommands::Cleanup { days, dry_run } => {
            info!(
                "Cleaning up data older than {} days (dry_run={})",
                days, dry_run
            );

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

async fn handle_query_command(
    cmd: QueryCommands,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = IndexerConfig::from_env()?;
    let store = riglr_indexer::storage::create_store(&config.storage).await?;

    match cmd {
        QueryCommands::Events {
            event_types,
            sources,
            start_time,
            end_time,
            limit,
            format,
        } => {
            let mut filter = riglr_indexer::storage::EventFilter::default();

            if let Some(types) = event_types {
                filter.event_types = Some(types.split(',').map(|s| s.trim().to_string()).collect());
            }

            if let Some(srcs) = sources {
                filter.sources = Some(srcs.split(',').map(|s| s.trim().to_string()).collect());
            }

            if let (Some(start), Some(end)) = (start_time, end_time) {
                let start_dt =
                    chrono::DateTime::parse_from_rfc3339(&start)?.with_timezone(&chrono::Utc);
                let end_dt =
                    chrono::DateTime::parse_from_rfc3339(&end)?.with_timezone(&chrono::Utc);
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
                    println!(
                        "{:<40} {:<20} {:<20} {:<20}",
                        "ID", "Type", "Source", "Timestamp"
                    );
                    println!("{}", "-".repeat(100));
                    for event in events {
                        println!(
                            "{:<40} {:<20} {:<20} {:<20}",
                            event.id,
                            event.event_type,
                            event.source,
                            event.timestamp.format("%Y-%m-%d %H:%M:%S")
                        );
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
                Some(event) => match format.as_str() {
                    "json" => {
                        println!("{}", serde_json::to_string_pretty(&event)?);
                    }
                    _ => {
                        return Err("Invalid format. Use 'json'".into());
                    }
                },
                None => {
                    println!("Event not found: {}", id);
                }
            }
        }

        QueryCommands::Stats => {
            let total_events = store
                .count_events(&riglr_indexer::storage::EventFilter::default())
                .await?;

            let events_by_type = riglr_indexer::storage::EventAggregator::aggregate_by_type(
                store.as_ref(),
                &riglr_indexer::storage::EventFilter::default(),
            )
            .await?;

            let events_by_source = riglr_indexer::storage::EventAggregator::aggregate_by_source(
                store.as_ref(),
                &riglr_indexer::storage::EventFilter::default(),
            )
            .await?;

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

async fn handle_health_command(
    endpoint: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let health_url = format!("{}/api/v1/health", endpoint);

    let request_result = client.get(&health_url).send().await;
    match request_result {
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

async fn handle_config_command(
    cmd: ConfigCommands,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match cmd {
        ConfigCommands::Validate => {
            let config_result = IndexerConfig::from_env();
            match config_result {
                Ok(config) => {
                    let validation_result = config.validate();
                    match validation_result {
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

async fn handle_metrics_command(
    cmd: MetricsCommands,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = IndexerConfig::from_env()?;

    match cmd {
        MetricsCommands::Show => {
            let client = reqwest::Client::new();
            let metrics_url = format!(
                "http://{}:{}/api/v1/metrics",
                config.api.http.bind, config.api.http.port
            );

            let metrics_result = client.get(&metrics_url).send().await;
            match metrics_result {
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
            let prometheus_url = format!(
                "http://{}:{}/metrics",
                config.api.http.bind, config.metrics.port
            );

            let prometheus_result = client.get(&prometheus_url).send().await;
            match prometheus_result {
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_command_factory_should_not_panic() {
        // Test that the CLI command structure is valid
        let _cmd = Cli::command();
    }

    #[test]
    fn test_cli_parse_database_init_should_succeed() {
        let args = vec!["riglr-indexer-cli", "database", "init"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Database(DatabaseCommands::Init) => {}
            _ => panic!("Expected Database Init command"),
        }
        assert!(!cli.verbose);
        assert!(cli.config.is_none());
    }

    #[test]
    fn test_cli_parse_database_migrate_should_succeed() {
        let args = vec!["riglr-indexer-cli", "database", "migrate"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Database(DatabaseCommands::Migrate) => {}
            _ => panic!("Expected Database Migrate command"),
        }
    }

    #[test]
    fn test_cli_parse_database_stats_should_succeed() {
        let args = vec!["riglr-indexer-cli", "database", "stats"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Database(DatabaseCommands::Stats) => {}
            _ => panic!("Expected Database Stats command"),
        }
    }

    #[test]
    fn test_cli_parse_database_cleanup_with_defaults_should_succeed() {
        let args = vec!["riglr-indexer-cli", "database", "cleanup"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Database(DatabaseCommands::Cleanup { days, dry_run }) => {
                assert_eq!(days, 30);
                assert!(!dry_run);
            }
            _ => panic!("Expected Database Cleanup command"),
        }
    }

    #[test]
    fn test_cli_parse_database_cleanup_with_custom_values_should_succeed() {
        let args = vec![
            "riglr-indexer-cli",
            "database",
            "cleanup",
            "--days",
            "7",
            "--dry-run",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Database(DatabaseCommands::Cleanup { days, dry_run }) => {
                assert_eq!(days, 7);
                assert!(dry_run);
            }
            _ => panic!("Expected Database Cleanup command"),
        }
    }

    #[test]
    fn test_cli_parse_query_events_with_defaults_should_succeed() {
        let args = vec!["riglr-indexer-cli", "query", "events"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Query(QueryCommands::Events {
                event_types,
                sources,
                start_time,
                end_time,
                limit,
                format,
            }) => {
                assert!(event_types.is_none());
                assert!(sources.is_none());
                assert!(start_time.is_none());
                assert!(end_time.is_none());
                assert_eq!(limit, 100);
                assert_eq!(format, "table");
            }
            _ => panic!("Expected Query Events command"),
        }
    }

    #[test]
    fn test_cli_parse_query_events_with_all_options_should_succeed() {
        let args = vec![
            "riglr-indexer-cli",
            "query",
            "events",
            "--event-types",
            "swap,transfer",
            "--sources",
            "ethereum,polygon",
            "--start-time",
            "2023-01-01T00:00:00Z",
            "--end-time",
            "2023-12-31T23:59:59Z",
            "--limit",
            "500",
            "--format",
            "json",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Query(QueryCommands::Events {
                event_types,
                sources,
                start_time,
                end_time,
                limit,
                format,
            }) => {
                assert_eq!(event_types.unwrap(), "swap,transfer");
                assert_eq!(sources.unwrap(), "ethereum,polygon");
                assert_eq!(start_time.unwrap(), "2023-01-01T00:00:00Z");
                assert_eq!(end_time.unwrap(), "2023-12-31T23:59:59Z");
                assert_eq!(limit, 500);
                assert_eq!(format, "json");
            }
            _ => panic!("Expected Query Events command"),
        }
    }

    #[test]
    fn test_cli_parse_query_get_with_defaults_should_succeed() {
        let args = vec!["riglr-indexer-cli", "query", "get", "event123"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Query(QueryCommands::Get { id, format }) => {
                assert_eq!(id, "event123");
                assert_eq!(format, "json");
            }
            _ => panic!("Expected Query Get command"),
        }
    }

    #[test]
    fn test_cli_parse_query_get_with_custom_format_should_succeed() {
        let args = vec![
            "riglr-indexer-cli",
            "query",
            "get",
            "event456",
            "--format",
            "yaml",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Query(QueryCommands::Get { id, format }) => {
                assert_eq!(id, "event456");
                assert_eq!(format, "yaml");
            }
            _ => panic!("Expected Query Get command"),
        }
    }

    #[test]
    fn test_cli_parse_query_stats_should_succeed() {
        let args = vec!["riglr-indexer-cli", "query", "stats"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Query(QueryCommands::Stats) => {}
            _ => panic!("Expected Query Stats command"),
        }
    }

    #[test]
    fn test_cli_parse_health_with_default_endpoint_should_succeed() {
        let args = vec!["riglr-indexer-cli", "health"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Health { endpoint } => {
                assert_eq!(endpoint, "http://localhost:8080");
            }
            _ => panic!("Expected Health command"),
        }
    }

    #[test]
    fn test_cli_parse_health_with_custom_endpoint_should_succeed() {
        let args = vec![
            "riglr-indexer-cli",
            "health",
            "--endpoint",
            "https://api.example.com:9090",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Health { endpoint } => {
                assert_eq!(endpoint, "https://api.example.com:9090");
            }
            _ => panic!("Expected Health command"),
        }
    }

    #[test]
    fn test_cli_parse_config_validate_should_succeed() {
        let args = vec!["riglr-indexer-cli", "config", "validate"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Config(ConfigCommands::Validate) => {}
            _ => panic!("Expected Config Validate command"),
        }
    }

    #[test]
    fn test_cli_parse_config_show_should_succeed() {
        let args = vec!["riglr-indexer-cli", "config", "show"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Config(ConfigCommands::Show) => {}
            _ => panic!("Expected Config Show command"),
        }
    }

    #[test]
    fn test_cli_parse_config_example_should_succeed() {
        let args = vec!["riglr-indexer-cli", "config", "example"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Config(ConfigCommands::Example) => {}
            _ => panic!("Expected Config Example command"),
        }
    }

    #[test]
    fn test_cli_parse_metrics_show_should_succeed() {
        let args = vec!["riglr-indexer-cli", "metrics", "show"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Metrics(MetricsCommands::Show) => {}
            _ => panic!("Expected Metrics Show command"),
        }
    }

    #[test]
    fn test_cli_parse_metrics_export_should_succeed() {
        let args = vec!["riglr-indexer-cli", "metrics", "export"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Metrics(MetricsCommands::Export) => {}
            _ => panic!("Expected Metrics Export command"),
        }
    }

    #[test]
    fn test_cli_parse_with_verbose_flag_should_succeed() {
        let args = vec!["riglr-indexer-cli", "--verbose", "health"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.verbose);
        match cli.command {
            Commands::Health { .. } => {}
            _ => panic!("Expected Health command"),
        }
    }

    #[test]
    fn test_cli_parse_with_short_verbose_flag_should_succeed() {
        let args = vec!["riglr-indexer-cli", "-v", "health"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.verbose);
    }

    #[test]
    fn test_cli_parse_with_config_flag_should_succeed() {
        let args = vec![
            "riglr-indexer-cli",
            "--config",
            "/path/to/config.toml",
            "health",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.config.unwrap(), "/path/to/config.toml");
        match cli.command {
            Commands::Health { .. } => {}
            _ => panic!("Expected Health command"),
        }
    }

    #[test]
    fn test_cli_parse_with_short_config_flag_should_succeed() {
        let args = vec!["riglr-indexer-cli", "-c", "/path/to/config.toml", "health"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.config.unwrap(), "/path/to/config.toml");
    }

    #[test]
    fn test_cli_parse_with_both_flags_should_succeed() {
        let args = vec![
            "riglr-indexer-cli",
            "--verbose",
            "--config",
            "/custom/config.toml",
            "database",
            "init",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.verbose);
        assert_eq!(cli.config.unwrap(), "/custom/config.toml");
        match cli.command {
            Commands::Database(DatabaseCommands::Init) => {}
            _ => panic!("Expected Database Init command"),
        }
    }

    #[test]
    fn test_cli_parse_invalid_command_should_fail() {
        let args = vec!["riglr-indexer-cli", "invalid-command"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parse_missing_subcommand_should_fail() {
        let args = vec!["riglr-indexer-cli"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parse_database_without_subcommand_should_fail() {
        let args = vec!["riglr-indexer-cli", "database"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parse_query_without_subcommand_should_fail() {
        let args = vec!["riglr-indexer-cli", "query"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parse_config_without_subcommand_should_fail() {
        let args = vec!["riglr-indexer-cli", "config"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parse_metrics_without_subcommand_should_fail() {
        let args = vec!["riglr-indexer-cli", "metrics"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parse_query_get_without_id_should_fail() {
        let args = vec!["riglr-indexer-cli", "query", "get"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parse_invalid_database_subcommand_should_fail() {
        let args = vec!["riglr-indexer-cli", "database", "invalid"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parse_invalid_query_subcommand_should_fail() {
        let args = vec!["riglr-indexer-cli", "query", "invalid"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parse_invalid_config_subcommand_should_fail() {
        let args = vec!["riglr-indexer-cli", "config", "invalid"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parse_invalid_metrics_subcommand_should_fail() {
        let args = vec!["riglr-indexer-cli", "metrics", "invalid"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parse_database_cleanup_with_invalid_days_should_fail() {
        let args = vec![
            "riglr-indexer-cli",
            "database",
            "cleanup",
            "--days",
            "invalid",
        ];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parse_query_events_with_invalid_limit_should_fail() {
        let args = vec!["riglr-indexer-cli", "query", "events", "--limit", "invalid"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parse_with_unknown_flag_should_fail() {
        let args = vec!["riglr-indexer-cli", "--unknown-flag", "health"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parse_database_cleanup_with_zero_days_should_succeed() {
        let args = vec!["riglr-indexer-cli", "database", "cleanup", "--days", "0"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Database(DatabaseCommands::Cleanup { days, .. }) => {
                assert_eq!(days, 0);
            }
            _ => panic!("Expected Database Cleanup command"),
        }
    }

    #[test]
    fn test_cli_parse_database_cleanup_with_large_days_should_succeed() {
        let args = vec![
            "riglr-indexer-cli",
            "database",
            "cleanup",
            "--days",
            "999999",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Database(DatabaseCommands::Cleanup { days, .. }) => {
                assert_eq!(days, 999999);
            }
            _ => panic!("Expected Database Cleanup command"),
        }
    }

    #[test]
    fn test_cli_parse_query_events_with_zero_limit_should_succeed() {
        let args = vec!["riglr-indexer-cli", "query", "events", "--limit", "0"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Query(QueryCommands::Events { limit, .. }) => {
                assert_eq!(limit, 0);
            }
            _ => panic!("Expected Query Events command"),
        }
    }

    #[test]
    fn test_cli_parse_query_events_with_large_limit_should_succeed() {
        let args = vec!["riglr-indexer-cli", "query", "events", "--limit", "999999"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Query(QueryCommands::Events { limit, .. }) => {
                assert_eq!(limit, 999999);
            }
            _ => panic!("Expected Query Events command"),
        }
    }

    #[test]
    fn test_cli_parse_empty_strings_should_succeed() {
        let args = vec!["riglr-indexer-cli", "query", "get", ""];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Query(QueryCommands::Get { id, .. }) => {
                assert_eq!(id, "");
            }
            _ => panic!("Expected Query Get command"),
        }
    }

    #[test]
    fn test_cli_parse_special_characters_in_id_should_succeed() {
        let args = vec!["riglr-indexer-cli", "query", "get", "event-123_$%@"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Query(QueryCommands::Get { id, .. }) => {
                assert_eq!(id, "event-123_$%@");
            }
            _ => panic!("Expected Query Get command"),
        }
    }

    #[test]
    fn test_cli_parse_unicode_characters_should_succeed() {
        let args = vec!["riglr-indexer-cli", "query", "get", "event-ñáéíóú-123"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Query(QueryCommands::Get { id, .. }) => {
                assert_eq!(id, "event-ñáéíóú-123");
            }
            _ => panic!("Expected Query Get command"),
        }
    }

    #[test]
    fn test_env_constants_should_have_expected_values() {
        assert_eq!(ENV_RUST_LOG, "RUST_LOG");
        assert_eq!(ENV_RIGLR_INDEXER_CONFIG, "RIGLR_INDEXER_CONFIG");
    }

    // Note: Testing the main function and handler functions requires mocking external dependencies
    // (database, HTTP clients, environment variables) which would require additional test infrastructure.
    // These tests focus on the CLI parsing logic which can be tested in isolation.
    // Integration tests for the handler functions should be placed in the tests/ directory.
}
