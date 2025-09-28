//! Configuration management for the RIGLR indexer service

use core::time::Duration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use crate::error::{ConfigError, IndexerError, IndexerResult};

const RIGLR_INDEXER_CONFIG: &str = "RIGLR_INDEXER_CONFIG";
const HOSTNAME: &str = "HOSTNAME";

/// Main configuration for the indexer service
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct IndexerConfig {
    /// API server configuration
    pub api: ApiConfig,

    /// Feature flags
    pub features: FeatureConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Metrics configuration
    pub metrics: MetricsConfig,

    /// Processing configuration
    pub processing: ProcessingConfig,

    /// Service configuration
    pub service: ServiceConfig,

    /// Storage configuration
    pub storage: StorageConfig,
}

/// Service-level configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct ServiceConfig {
    /// Environment (development, staging, production)
    pub environment: String,

    /// Health check interval
    #[serde(with = "humantime_serde")]
    pub health_check_interval: Duration,

    /// Service name for identification
    pub name: String,

    /// Node ID for clustering
    pub node_id: Option<String>,

    /// Graceful shutdown timeout
    #[serde(with = "humantime_serde")]
    pub shutdown_timeout: Duration,

    /// Service version
    pub version: String,
}

/// Storage configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct StorageConfig {
    /// Cache configuration
    pub cache: CacheConfig,

    /// Primary storage backend
    pub primary: StorageBackendConfig,

    /// Data retention policies
    pub retention: RetentionConfig,

    /// Optional secondary storage for archival
    pub secondary: Option<StorageBackendConfig>,
}

/// Storage backend configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct StorageBackendConfig {
    /// Backend type (postgres, clickhouse, etc.)
    pub backend: StorageBackend,

    /// Connection pool settings
    pub pool: ConnectionPoolConfig,

    /// Database-specific settings
    pub settings: HashMap<String, serde_json::Value>,

    /// Connection URL
    pub url: String,
}

/// Storage backend types
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    /// `ClickHouse` columnar database for high-performance analytics
    ClickHouse,
    /// `MongoDB` document database for flexible schema storage
    MongoDB,
    /// `PostgreSQL` database backend for general-purpose storage
    Postgres,
    /// `TimescaleDB` time-series database for temporal data
    TimescaleDB,
}

/// Connection pool configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct ConnectionPoolConfig {
    /// Connection timeout
    #[serde(with = "humantime_serde")]
    pub connect_timeout: Duration,

    /// Idle timeout
    #[serde(with = "humantime_serde")]
    pub idle_timeout: Duration,

    /// Maximum number of connections
    pub max_connections: u32,

    /// Maximum connection lifetime
    #[serde(with = "humantime_serde")]
    pub max_lifetime: Duration,

    /// Minimum number of idle connections
    pub min_connections: u32,
}

/// Cache configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct CacheConfig {
    /// Cache backend (redis, memory)
    pub backend: CacheBackend,

    /// Memory cache settings
    pub memory: MemoryCacheConfig,

    /// Redis URL (if using Redis)
    pub redis_url: Option<String>,

    /// Cache TTL settings
    pub ttl: CacheTtlConfig,
}

/// Cache backend types
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CacheBackend {
    /// No caching enabled
    Disabled,
    /// In-memory cache for single-instance deployments
    Memory,
    /// Redis distributed cache for shared caching across instances
    Redis,
}

/// Cache TTL configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct CacheTtlConfig {
    /// TTL for aggregated data
    #[serde(with = "humantime_serde")]
    pub aggregates: Duration,

    /// Default TTL for cached items
    #[serde(with = "humantime_serde")]
    pub default: Duration,

    /// TTL for event metadata
    #[serde(with = "humantime_serde")]
    pub events: Duration,
}

/// Memory cache settings
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct MemoryCacheConfig {
    /// Maximum number of entries
    pub max_entries: u64,

    /// Maximum memory usage in bytes
    pub max_size_bytes: u64,
}

/// Data retention configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct RetentionConfig {
    /// Archive configuration
    pub archive: ArchiveConfig,

    /// Retention by event type
    pub by_event_type: HashMap<String, Duration>,

    /// Default retention period
    #[serde(with = "humantime_serde")]
    pub default: Duration,
}

/// Archive configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct ArchiveConfig {
    /// Archive storage backend
    pub backend: Option<StorageBackendConfig>,

    /// Compression settings
    pub compression: CompressionConfig,

    /// Whether archival is enabled
    pub enabled: bool,
}

/// Compression configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct CompressionConfig {
    /// Compression algorithm
    pub algorithm: CompressionAlgorithm,

    /// Compression level (1-9)
    pub level: u8,
}

/// Compression algorithms
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CompressionAlgorithm {
    /// Gzip compression - widely compatible
    Gzip,
    /// LZ4 compression - fastest compression/decompression
    Lz4,
    /// No compression
    None,
    /// Zstandard compression - best compression ratio
    Zstd,
}

/// Processing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct ProcessingConfig {
    /// Batch processing settings
    pub batch: BatchConfig,

    /// Event queue configuration
    pub queue: QueueConfig,

    /// Rate limiting
    pub rate_limit: RateLimitConfig,

    /// Retry configuration
    pub retry: RetryConfig,

    /// Number of worker threads
    pub workers: usize,
}

/// Batch processing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct BatchConfig {
    /// Maximum batch age before forcing flush
    #[serde(with = "humantime_serde")]
    pub max_age: Duration,

    /// Maximum batch size
    pub max_size: usize,

    /// Target batch size for optimal performance
    pub target_size: usize,
}

/// Event queue configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct QueueConfig {
    /// Maximum queue capacity
    pub capacity: usize,

    /// Disk queue settings (if using disk queue)
    pub disk_settings: Option<DiskQueueConfig>,

    /// Queue type (memory, disk, hybrid)
    pub queue_type: QueueType,
}

/// Queue type options
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum QueueType {
    /// Disk-backed queue for durability
    Disk,
    /// Hybrid queue using memory with disk overflow
    Hybrid,
    /// In-memory queue for maximum performance
    Memory,
}

/// Disk queue configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct DiskQueueConfig {
    /// Directory for queue files
    pub directory: PathBuf,

    /// Maximum size per file
    pub max_file_size: u64,

    /// Sync strategy
    pub sync_strategy: SyncStrategy,
}

/// Disk sync strategies
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SyncStrategy {
    /// Sync to disk after every write for maximum durability
    Always,
    /// Never explicitly sync, rely on OS buffering
    Never,
    /// Sync to disk periodically for balanced performance
    Periodic,
}

/// Retry configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct RetryConfig {
    /// Backoff multiplier
    pub backoff_multiplier: f64,

    /// Base delay between retries
    #[serde(with = "humantime_serde")]
    pub base_delay: Duration,

    /// Jitter factor (0.0 to 1.0)
    pub jitter: f64,

    /// Maximum retry attempts
    pub max_attempts: u32,

    /// Maximum delay between retries
    #[serde(with = "humantime_serde")]
    pub max_delay: Duration,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct RateLimitConfig {
    /// Burst capacity
    pub burst_capacity: u32,

    /// Whether rate limiting is enabled
    pub enabled: bool,

    /// Maximum events per second
    pub max_events_per_second: u32,
}

/// API server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct ApiConfig {
    /// Authentication settings
    pub auth: AuthConfig,

    /// CORS settings
    pub cors: CorsConfig,

    /// GraphQL settings
    pub graphql: Option<GraphQLConfig>,

    /// HTTP server settings
    pub http: HttpConfig,

    /// WebSocket settings
    pub websocket: WebSocketConfig,
}

/// HTTP server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct HttpConfig {
    /// Server bind address
    pub bind: String,

    /// Keep-alive timeout
    #[serde(with = "humantime_serde")]
    pub keep_alive: Duration,

    /// Maximum request size
    pub max_request_size: u64,

    /// Server port
    pub port: u16,

    /// Request timeout
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
}

/// WebSocket configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct WebSocketConfig {
    /// Message buffer size
    pub buffer_size: usize,

    /// Whether WebSocket is enabled
    pub enabled: bool,

    /// Heartbeat interval
    #[serde(with = "humantime_serde")]
    pub heartbeat_interval: Duration,

    /// Maximum connections
    pub max_connections: u32,
}

/// GraphQL configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct GraphQLConfig {
    /// Query complexity limit
    pub complexity_limit: u32,

    /// Query depth limit
    pub depth_limit: u32,

    /// Whether GraphQL is enabled
    pub enabled: bool,

    /// GraphQL endpoint path
    pub endpoint: String,

    /// Playground enabled
    pub playground: bool,
}

/// Authentication configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct AuthConfig {
    /// API key settings (if using API keys)
    pub api_key: Option<ApiKeyConfig>,

    /// Whether authentication is enabled
    pub enabled: bool,

    /// JWT settings (if using JWT)
    pub jwt: Option<JwtConfig>,

    /// Authentication method
    pub method: AuthMethod,
}

/// Authentication methods
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    /// API key-based authentication
    ApiKey,
    /// JSON Web Token authentication
    Jwt,
    /// No authentication required
    None,
    /// OAuth 2.0 authentication
    OAuth2,
}

/// JWT configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct JwtConfig {
    /// Audience
    pub audience: String,

    /// Token expiration time
    #[serde(with = "humantime_serde")]
    pub expires_in: Duration,

    /// Issuer
    pub issuer: String,

    /// JWT secret key
    pub secret: String,
}

/// API key configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct ApiKeyConfig {
    /// Header name for API key
    pub header_name: String,

    /// Valid API keys
    pub keys: Vec<String>,
}

/// CORS configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct CorsConfig {
    /// Allowed headers
    pub allowed_headers: Vec<String>,

    /// Allowed methods
    pub allowed_methods: Vec<String>,

    /// Allowed origins
    pub allowed_origins: Vec<String>,

    /// Whether CORS is enabled
    pub enabled: bool,

    /// Max age for preflight requests
    #[serde(with = "humantime_serde")]
    pub max_age: Duration,
}

/// Metrics configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct MetricsConfig {
    /// Collection interval
    #[serde(with = "humantime_serde")]
    pub collection_interval: Duration,

    /// Custom metrics
    pub custom: HashMap<String, CustomMetricConfig>,

    /// Whether metrics are enabled
    pub enabled: bool,

    /// Metrics endpoint path
    pub endpoint: String,

    /// Histogram buckets
    pub histogram_buckets: Vec<f64>,

    /// Metrics server port
    pub port: u16,
}

/// Custom metric configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct CustomMetricConfig {
    /// Metric description
    pub description: String,

    /// Metric labels
    pub labels: Vec<String>,

    /// Metric type
    pub metric_type: MetricType,
}

/// Metric types
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    /// Counter metric that only increases
    Counter,
    /// Gauge metric that can go up or down
    Gauge,
    /// Histogram for observing value distributions
    Histogram,
    /// Summary for calculating quantiles
    Summary,
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct LoggingConfig {
    /// Log format (json, text)
    pub format: LogFormat,

    /// Log level
    pub level: String,

    /// Output destinations
    pub outputs: Vec<LogOutput>,

    /// Structured logging settings
    pub structured: StructuredLoggingConfig,
}

/// Log format options
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// JSON structured logging format
    Json,
    /// Pretty-printed format for development
    Pretty,
    /// Plain text logging format
    Text,
}

/// Log output destinations
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogOutput {
    /// Log to specified file path
    File(PathBuf),
    /// Standard error stream
    Stderr,
    /// Standard output stream
    Stdout,
    /// System log daemon
    Syslog,
}

/// Structured logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct StructuredLoggingConfig {
    /// Custom fields to include
    pub custom_fields: HashMap<String, String>,

    /// Include source location
    pub include_location: bool,

    /// Include service metadata
    pub include_service_metadata: bool,

    /// Include thread information
    pub include_thread: bool,
}

/// Feature configuration using enum-based approach to avoid excessive bools
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[expect(clippy::module_name_repetitions)]
pub struct FeatureConfig {
    /// Enabled API features
    pub api_features: ApiFeatureSet,

    /// Enabled core features
    pub core_features: CoreFeatureSet,

    /// Custom feature flags
    pub custom: HashMap<String, bool>,

    /// Development and experimental features
    pub dev_features: DevFeatureSet,
}

/// Core feature set configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoreFeatureSet {
    /// Event archival capability
    pub archival: FeatureState,

    /// Real-time streaming capability
    pub streaming: FeatureState,
}

/// API feature set configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiFeatureSet {
    /// GraphQL API endpoint
    pub graphql: FeatureState,
}

/// Development and experimental feature set
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DevFeatureSet {
    /// Experimental features toggle
    pub experimental: FeatureState,
}

/// State of a feature
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum FeatureState {
    /// Feature is disabled
    Disabled,
    /// Feature is enabled
    Enabled,
    /// Feature is enabled with configuration
    EnabledWithConfig(serde_json::Value),
}

impl Default for FeatureState {
    fn default() -> Self {
        Self::Disabled
    }
}

impl FeatureState {
    /// Get configuration if feature is enabled with config
    #[must_use]
    pub const fn config(&self) -> Option<&serde_json::Value> {
        match *self {
            Self::EnabledWithConfig(ref config) => Some(config),
            _ => None,
        }
    }

    /// Check if feature is enabled (either Enabled or `EnabledWithConfig`)
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        matches!(*self, Self::Enabled | Self::EnabledWithConfig(_))
    }
}

impl Default for CoreFeatureSet {
    fn default() -> Self {
        Self {
            archival: FeatureState::Enabled,
            streaming: FeatureState::Enabled,
        }
    }
}

impl Default for ApiFeatureSet {
    fn default() -> Self {
        Self {
            graphql: FeatureState::Disabled,
        }
    }
}

mod legacy {
    use super::{
        ApiFeatureSet, CoreFeatureSet, Deserialize, DevFeatureSet, FeatureConfig, FeatureState,
        HashMap,
    };

    /// Legacy feature configuration for backward compatibility
    #[derive(Debug, Clone, Deserialize)]
    pub struct LegacyFeatureConfig {
        /// Enable event archival
        pub archival: Option<bool>,

        /// Custom feature flags
        pub custom: Option<HashMap<String, bool>>,

        /// Enable experimental features
        pub experimental: Option<bool>,

        /// Enable GraphQL API
        pub graphql_api: Option<bool>,

        /// Enable real-time streaming
        pub realtime_streaming: Option<bool>,
    }

    impl From<LegacyFeatureConfig> for FeatureConfig {
        fn from(legacy: LegacyFeatureConfig) -> Self {
            Self {
                api_features: ApiFeatureSet {
                    graphql: if legacy.graphql_api.unwrap_or(false) {
                        FeatureState::Enabled
                    } else {
                        FeatureState::Disabled
                    },
                },
                core_features: CoreFeatureSet {
                    archival: if legacy.archival.unwrap_or(false) {
                        FeatureState::Enabled
                    } else {
                        FeatureState::Disabled
                    },
                    streaming: if legacy.realtime_streaming.unwrap_or(false) {
                        FeatureState::Enabled
                    } else {
                        FeatureState::Disabled
                    },
                },
                custom: legacy.custom.unwrap_or_default(),
                dev_features: DevFeatureSet {
                    experimental: if legacy.experimental.unwrap_or(false) {
                        FeatureState::Enabled
                    } else {
                        FeatureState::Disabled
                    },
                },
            }
        }
    }
}

impl Default for DevFeatureSet {
    fn default() -> Self {
        Self {
            experimental: FeatureState::Disabled,
        }
    }
}

impl IndexerConfig {
    /// Load configuration from environment variables and optional file
    ///
    /// # Errors
    ///
    /// Returns error if configuration file is invalid or required environment variables are missing
    pub fn from_env() -> IndexerResult<Self> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        // Check for custom config file
        let config_file = env::var(RIGLR_INDEXER_CONFIG).ok();

        let mut settings = config::Config::builder();

        // Load defaults first
        settings = settings.add_source(
            config::Config::try_from(&Self::default())
                .map_err(|e| IndexerError::internal(e.to_string()))?,
        );

        // Override with file if specified
        if let Some(path) = config_file {
            settings = settings.add_source(config::File::with_name(&path).required(false));
        }

        // Override with environment variables
        settings = settings.add_source(
            config::Environment::with_prefix("RIGLR_INDEXER")
                .separator("__")
                .try_parsing(true),
        );

        let config = settings.build().map_err(|e| ConfigError::LoadFailed {
            path: "environment".to_string(),
            source: Box::new(e),
        })?;

        let config: Self = config
            .try_deserialize()
            .map_err(|e| ConfigError::LoadFailed {
                path: "deserialization".to_string(),
                source: Box::new(e),
            })?;

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Get a feature flag value
    #[must_use]
    pub fn feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "archival" => self.features.core_features.archival.is_enabled(),
            "experimental" => self.features.dev_features.experimental.is_enabled(),
            "graphql_api" => self.features.api_features.graphql.is_enabled(),
            "realtime_streaming" => self.features.core_features.streaming.is_enabled(),
            _ => self.features.custom.get(feature).copied().unwrap_or(false),
        }
    }

    /// Validate the configuration
    ///
    /// # Errors
    ///
    /// Returns error if any required configuration fields are missing or invalid
    pub fn validate(&self) -> IndexerResult<()> {
        // Validate service config
        if self.service.name.is_empty() {
            return Err(ConfigError::MissingField {
                field: "service.name".to_string(),
            }
            .into());
        }

        // Validate storage config
        if self.storage.primary.url.is_empty() {
            return Err(ConfigError::MissingField {
                field: "storage.primary.url".to_string(),
            }
            .into());
        }

        // Validate processing config
        if self.processing.workers == 0 {
            return Err(ConfigError::InvalidValue {
                field: "processing.workers".to_string(),
                value: "0".to_string(),
            }
            .into());
        }

        if self.processing.batch.max_size == 0 {
            return Err(ConfigError::InvalidValue {
                field: "processing.batch.max_size".to_string(),
                value: "0".to_string(),
            }
            .into());
        }

        // Validate API config
        if self.api.http.port == 0 {
            return Err(ConfigError::InvalidValue {
                field: "api.http.port".to_string(),
                value: "0".to_string(),
            }
            .into());
        }

        // Validate metrics config if enabled
        if self.metrics.enabled && self.metrics.port == 0 {
            return Err(ConfigError::InvalidValue {
                field: "metrics.port".to_string(),
                value: "0".to_string(),
            }
            .into());
        }

        Ok(())
    }

    /// Generate node ID if not provided
    #[must_use]
    pub fn node_id(&self) -> String {
        self.service.node_id.clone().unwrap_or_else(|| {
            use core::hash::{Hash, Hasher};
            use std::collections::hash_map::DefaultHasher;

            let mut hasher = DefaultHasher::default();
            self.service.name.hash(&mut hasher);
            env::var(HOSTNAME)
                .unwrap_or_else(|_| "unknown".to_string())
                .hash(&mut hasher);

            format!("{:x}", hasher.finish())
        })
    }
}

impl IndexerConfig {
    /// Create default service configuration
    fn default_service_config() -> ServiceConfig {
        ServiceConfig {
            name: "riglr-indexer".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "development".to_string(),
            node_id: None,
            shutdown_timeout: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(30),
        }
    }

    /// Create default storage configuration
    fn default_storage_config() -> StorageConfig {
        StorageConfig {
            primary: StorageBackendConfig {
                backend: StorageBackend::Postgres,
                url: "postgresql://localhost:5432/riglr_indexer".to_string(),
                pool: ConnectionPoolConfig {
                    max_connections: 20,
                    min_connections: 5,
                    connect_timeout: Duration::from_secs(30),
                    idle_timeout: Duration::from_secs(300),
                    max_lifetime: Duration::from_secs(1800),
                },
                settings: HashMap::new(),
            },
            secondary: None,
            cache: CacheConfig {
                backend: CacheBackend::Redis,
                redis_url: Some("redis://localhost:6379".to_string()),
                ttl: CacheTtlConfig {
                    default: Duration::from_secs(300),
                    events: Duration::from_secs(3600),
                    aggregates: Duration::from_secs(1800),
                },
                memory: MemoryCacheConfig {
                    max_size_bytes: 100_000_000, // 100MB
                    max_entries: 10_000,
                },
            },
            retention: RetentionConfig {
                default: Duration::from_secs(30 * 24 * 3600), // 30 days
                by_event_type: HashMap::new(),
                archive: ArchiveConfig {
                    enabled: false,
                    backend: None,
                    compression: CompressionConfig {
                        algorithm: CompressionAlgorithm::Zstd,
                        level: 3,
                    },
                },
            },
        }
    }

    /// Create default processing configuration
    fn default_processing_config() -> ProcessingConfig {
        ProcessingConfig {
            workers: num_cpus::get(),
            batch: BatchConfig {
                max_size: 1000,
                max_age: Duration::from_secs(5),
                target_size: 100,
            },
            queue: QueueConfig {
                capacity: 10000,
                queue_type: QueueType::Memory,
                disk_settings: None,
            },
            retry: RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(30),
                backoff_multiplier: 2.0,
                jitter: 0.1,
            },
            rate_limit: RateLimitConfig {
                enabled: true,
                max_events_per_second: 10000,
                burst_capacity: 1000,
            },
        }
    }

    /// Create default API configuration
    fn default_api_config() -> ApiConfig {
        ApiConfig {
            http: HttpConfig {
                bind: "0.0.0.0".to_string(),
                port: 8080,
                timeout: Duration::from_secs(30),
                max_request_size: 1_000_000, // 1MB
                keep_alive: Duration::from_secs(60),
            },
            websocket: WebSocketConfig {
                enabled: true,
                max_connections: 1000,
                buffer_size: 1024,
                heartbeat_interval: Duration::from_secs(30),
            },
            graphql: None,
            auth: AuthConfig {
                enabled: false,
                method: AuthMethod::None,
                jwt: None,
                api_key: None,
            },
            cors: CorsConfig {
                enabled: true,
                allowed_origins: vec!["*".to_string()],
                allowed_methods: vec!["GET".to_string(), "POST".to_string()],
                allowed_headers: vec!["*".to_string()],
                max_age: Duration::from_secs(3600),
            },
        }
    }

    /// Create default metrics configuration
    fn default_metrics_config() -> MetricsConfig {
        MetricsConfig {
            enabled: true,
            port: 9090,
            endpoint: "/metrics".to_string(),
            collection_interval: Duration::from_secs(15),
            histogram_buckets: vec![
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
            custom: HashMap::new(),
        }
    }

    /// Create default logging configuration
    fn default_logging_config() -> LoggingConfig {
        LoggingConfig {
            level: "info".to_string(),
            format: LogFormat::Json,
            outputs: vec![LogOutput::Stdout],
            structured: StructuredLoggingConfig {
                include_location: false,
                include_thread: true,
                include_service_metadata: true,
                custom_fields: HashMap::new(),
            },
        }
    }

    /// Create default feature configuration
    fn default_features_config() -> FeatureConfig {
        FeatureConfig {
            core_features: CoreFeatureSet {
                streaming: FeatureState::Enabled,
                archival: FeatureState::Enabled,
            },
            api_features: ApiFeatureSet {
                graphql: FeatureState::Disabled,
            },
            dev_features: DevFeatureSet {
                experimental: FeatureState::Disabled,
            },
            custom: HashMap::new(),
        }
    }
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            service: Self::default_service_config(),
            storage: Self::default_storage_config(),
            processing: Self::default_processing_config(),
            api: Self::default_api_config(),
            metrics: Self::default_metrics_config(),
            logging: Self::default_logging_config(),
            features: Self::default_features_config(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::time::Duration;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_indexer_config_default() {
        let config = IndexerConfig::default();

        assert_eq!(config.service.name, "riglr-indexer");
        assert_eq!(config.service.environment, "development");
        assert_eq!(config.service.shutdown_timeout, Duration::from_secs(30));
        assert_eq!(config.storage.primary.backend, StorageBackend::Postgres);
        assert_eq!(config.processing.workers, num_cpus::get());
        assert_eq!(config.api.http.port, 8080);
        assert!(config.metrics.enabled);
        assert!(config.feature_enabled("realtime_streaming"));
    }

    #[test]
    fn test_indexer_config_validate_when_valid_should_return_ok() {
        let config = IndexerConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    #[allow(clippy::expect_used, clippy::panic)]
    fn test_indexer_config_validate_when_empty_service_name_should_return_err() {
        let mut config = IndexerConfig::default();
        config.service.name = String::default();

        let result = config.validate();
        assert!(result.is_err());
        let error = result.expect_err("Expected validation to fail for empty service name");
        match error {
            IndexerError::Config(ConfigError::MissingField { field }) => {
                assert_eq!(field, "service.name");
            }
            _ => panic!("Expected ConfigError::MissingField, got: {error:?}"),
        }
    }

    #[test]
    #[allow(clippy::expect_used, clippy::panic)]
    fn test_indexer_config_validate_when_empty_storage_url_should_return_err() {
        let mut config = IndexerConfig::default();
        config.storage.primary.url = String::default();

        let result = config.validate();
        assert!(result.is_err());
        let error = result.expect_err("Expected validation to fail for empty storage URL");
        match error {
            IndexerError::Config(ConfigError::MissingField { field }) => {
                assert_eq!(field, "storage.primary.url");
            }
            _ => panic!("Expected ConfigError::MissingField, got: {error:?}"),
        }
    }

    #[test]
    #[allow(clippy::expect_used, clippy::panic)]
    fn test_indexer_config_validate_when_zero_workers_should_return_err() {
        let mut config = IndexerConfig::default();
        config.processing.workers = 0;

        let result = config.validate();
        assert!(result.is_err());
        let error = result.expect_err("Expected validation to fail for zero workers");
        match error {
            IndexerError::Config(ConfigError::InvalidValue { field, value }) => {
                assert_eq!(field, "processing.workers");
                assert_eq!(value, "0");
            }
            _ => panic!("Expected ConfigError::InvalidValue, got: {error:?}"),
        }
    }

    #[test]
    #[allow(clippy::expect_used, clippy::panic)]
    fn test_indexer_config_validate_when_zero_batch_size_should_return_err() {
        let mut config = IndexerConfig::default();
        config.processing.batch.max_size = 0;

        let result = config.validate();
        assert!(result.is_err());
        let error = result.expect_err("Expected validation to fail for zero batch size");
        match error {
            IndexerError::Config(ConfigError::InvalidValue { field, value }) => {
                assert_eq!(field, "processing.batch.max_size");
                assert_eq!(value, "0");
            }
            _ => panic!("Expected ConfigError::InvalidValue, got: {error:?}"),
        }
    }

    #[test]
    #[allow(clippy::expect_used, clippy::panic)]
    fn test_indexer_config_validate_when_zero_http_port_should_return_err() {
        let mut config = IndexerConfig::default();
        config.api.http.port = 0;

        let result = config.validate();
        assert!(result.is_err());
        let error = result.expect_err("Expected validation to fail for zero HTTP port");
        match error {
            IndexerError::Config(ConfigError::InvalidValue { field, value }) => {
                assert_eq!(field, "api.http.port");
                assert_eq!(value, "0");
            }
            _ => panic!("Expected ConfigError::InvalidValue, got: {error:?}"),
        }
    }

    #[test]
    #[allow(clippy::expect_used, clippy::panic)]
    fn test_indexer_config_validate_when_metrics_enabled_but_zero_port_should_return_err() {
        let mut config = IndexerConfig::default();
        config.metrics.enabled = true;
        config.metrics.port = 0;

        let result = config.validate();
        assert!(result.is_err());
        let error =
            result.expect_err("Expected validation to fail for zero metrics port when enabled");
        match error {
            IndexerError::Config(ConfigError::InvalidValue { field, value }) => {
                assert_eq!(field, "metrics.port");
                assert_eq!(value, "0");
            }
            _ => panic!("Expected ConfigError::InvalidValue, got: {error:?}"),
        }
    }

    #[test]
    fn test_indexer_config_validate_when_metrics_disabled_and_zero_port_should_return_ok() {
        let mut config = IndexerConfig::default();
        config.metrics.enabled = false;
        config.metrics.port = 0;

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_indexer_config_feature_enabled_when_builtin_features() {
        let config = IndexerConfig::default();

        assert!(config.feature_enabled("realtime_streaming"));
        assert!(config.feature_enabled("archival"));
        assert!(!config.feature_enabled("graphql_api"));
        assert!(!config.feature_enabled("experimental"));
    }

    #[test]
    fn test_indexer_config_feature_enabled_when_custom_feature_exists() {
        let mut config = IndexerConfig::default();
        config
            .features
            .custom
            .insert("my_feature".to_string(), true);

        assert!(config.feature_enabled("my_feature"));
    }

    #[test]
    fn test_indexer_config_feature_enabled_when_custom_feature_not_exists() {
        let config = IndexerConfig::default();

        assert!(!config.feature_enabled("nonexistent_feature"));
    }

    #[test]
    fn test_indexer_config_feature_enabled_when_custom_feature_disabled() {
        let mut config = IndexerConfig::default();
        config
            .features
            .custom
            .insert("my_feature".to_string(), false);

        assert!(!config.feature_enabled("my_feature"));
    }

    #[test]
    fn test_indexer_config_node_id_when_provided() {
        let mut config = IndexerConfig::default();
        config.service.node_id = Some("custom-node-id".to_string());

        assert_eq!(config.node_id(), "custom-node-id");
    }

    #[test]
    fn test_indexer_config_node_id_when_not_provided() {
        let config = IndexerConfig::default();

        let node_id = config.node_id();
        assert!(!node_id.is_empty());
        // Should be a hex string (hash of service name + hostname)
        assert!(node_id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_storage_backend_serialization() {
        assert_eq!(
            serde_json::to_string(&StorageBackend::Postgres).unwrap(),
            "\"postgres\""
        );
        assert_eq!(
            serde_json::to_string(&StorageBackend::ClickHouse).unwrap(),
            "\"clickhouse\""
        );
        assert_eq!(
            serde_json::to_string(&StorageBackend::TimescaleDB).unwrap(),
            "\"timescaledb\""
        );
        assert_eq!(
            serde_json::to_string(&StorageBackend::MongoDB).unwrap(),
            "\"mongodb\""
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_storage_backend_deserialization() {
        assert_eq!(
            serde_json::from_str::<StorageBackend>("\"postgres\"").unwrap(),
            StorageBackend::Postgres
        );
        assert_eq!(
            serde_json::from_str::<StorageBackend>("\"clickhouse\"").unwrap(),
            StorageBackend::ClickHouse
        );
        assert_eq!(
            serde_json::from_str::<StorageBackend>("\"timescaledb\"").unwrap(),
            StorageBackend::TimescaleDB
        );
        assert_eq!(
            serde_json::from_str::<StorageBackend>("\"mongodb\"").unwrap(),
            StorageBackend::MongoDB
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_cache_backend_serialization() {
        assert_eq!(
            serde_json::to_string(&CacheBackend::Redis).unwrap(),
            "\"redis\""
        );
        assert_eq!(
            serde_json::to_string(&CacheBackend::Memory).unwrap(),
            "\"memory\""
        );
        assert_eq!(
            serde_json::to_string(&CacheBackend::Disabled).unwrap(),
            "\"disabled\""
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_compression_algorithm_serialization() {
        assert_eq!(
            serde_json::to_string(&CompressionAlgorithm::Zstd).unwrap(),
            "\"zstd\""
        );
        assert_eq!(
            serde_json::to_string(&CompressionAlgorithm::Gzip).unwrap(),
            "\"gzip\""
        );
        assert_eq!(
            serde_json::to_string(&CompressionAlgorithm::Lz4).unwrap(),
            "\"lz4\""
        );
        assert_eq!(
            serde_json::to_string(&CompressionAlgorithm::None).unwrap(),
            "\"none\""
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_queue_type_serialization() {
        assert_eq!(
            serde_json::to_string(&QueueType::Memory).unwrap(),
            "\"memory\""
        );
        assert_eq!(serde_json::to_string(&QueueType::Disk).unwrap(), "\"disk\"");
        assert_eq!(
            serde_json::to_string(&QueueType::Hybrid).unwrap(),
            "\"hybrid\""
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_sync_strategy_serialization() {
        assert_eq!(
            serde_json::to_string(&SyncStrategy::Always).unwrap(),
            "\"always\""
        );
        assert_eq!(
            serde_json::to_string(&SyncStrategy::Periodic).unwrap(),
            "\"periodic\""
        );
        assert_eq!(
            serde_json::to_string(&SyncStrategy::Never).unwrap(),
            "\"never\""
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_auth_method_serialization() {
        assert_eq!(
            serde_json::to_string(&AuthMethod::None).unwrap(),
            "\"none\""
        );
        assert_eq!(
            serde_json::to_string(&AuthMethod::ApiKey).unwrap(),
            "\"apikey\""
        );
        assert_eq!(serde_json::to_string(&AuthMethod::Jwt).unwrap(), "\"jwt\"");
        assert_eq!(
            serde_json::to_string(&AuthMethod::OAuth2).unwrap(),
            "\"oauth2\""
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_metric_type_serialization() {
        assert_eq!(
            serde_json::to_string(&MetricType::Counter).unwrap(),
            "\"counter\""
        );
        assert_eq!(
            serde_json::to_string(&MetricType::Gauge).unwrap(),
            "\"gauge\""
        );
        assert_eq!(
            serde_json::to_string(&MetricType::Histogram).unwrap(),
            "\"histogram\""
        );
        assert_eq!(
            serde_json::to_string(&MetricType::Summary).unwrap(),
            "\"summary\""
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_log_format_serialization() {
        assert_eq!(serde_json::to_string(&LogFormat::Json).unwrap(), "\"json\"");
        assert_eq!(serde_json::to_string(&LogFormat::Text).unwrap(), "\"text\"");
        assert_eq!(
            serde_json::to_string(&LogFormat::Pretty).unwrap(),
            "\"pretty\""
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_log_output_serialization() {
        assert_eq!(
            serde_json::to_string(&LogOutput::Stdout).unwrap(),
            "\"stdout\""
        );
        assert_eq!(
            serde_json::to_string(&LogOutput::Stderr).unwrap(),
            "\"stderr\""
        );
        assert_eq!(
            serde_json::to_string(&LogOutput::Syslog).unwrap(),
            "\"syslog\""
        );

        let file_output = LogOutput::File(PathBuf::from("/var/log/app.log"));
        let serialized = serde_json::to_string(&file_output).unwrap();
        assert!(serialized.contains("/var/log/app.log"));
    }

    #[test]
    fn test_struct_cloning() {
        let config = IndexerConfig::default();
        let cloned = config.clone();

        assert_eq!(config.service.name, cloned.service.name);
        assert_eq!(
            config.storage.primary.backend,
            cloned.storage.primary.backend
        );
        assert_eq!(config.processing.workers, cloned.processing.workers);
    }

    #[test]
    fn test_struct_debug_formatting() {
        let config = IndexerConfig::default();
        let debug_output = format!("{config:?}");

        assert!(debug_output.contains("IndexerConfig"));
        assert!(debug_output.contains("riglr-indexer"));
    }

    #[test]
    fn test_service_config_creation() {
        let service = ServiceConfig {
            name: "test-service".to_string(),
            version: "1.0.0".to_string(),
            environment: "test".to_string(),
            node_id: Some("test-node".to_string()),
            shutdown_timeout: Duration::from_secs(60),
            health_check_interval: Duration::from_secs(10),
        };

        assert_eq!(service.name, "test-service");
        assert_eq!(service.version, "1.0.0");
        assert_eq!(service.environment, "test");
        assert_eq!(service.node_id, Some("test-node".to_string()));
        assert_eq!(service.shutdown_timeout, Duration::from_secs(60));
        assert_eq!(service.health_check_interval, Duration::from_secs(10));
    }

    #[test]
    fn test_storage_backend_config_creation() {
        let mut settings = HashMap::new();
        settings.insert(
            "max_connections".to_string(),
            serde_json::Value::Number(serde_json::Number::from(100)),
        );

        let storage = StorageBackendConfig {
            backend: StorageBackend::ClickHouse,
            url: "http://localhost:8123".to_string(),
            pool: ConnectionPoolConfig {
                max_connections: 50,
                min_connections: 10,
                connect_timeout: Duration::from_secs(10),
                idle_timeout: Duration::from_secs(60),
                max_lifetime: Duration::from_secs(300),
            },
            settings,
        };

        assert_eq!(storage.backend, StorageBackend::ClickHouse);
        assert_eq!(storage.url, "http://localhost:8123");
        assert_eq!(storage.pool.max_connections, 50);
        assert!(storage.settings.contains_key("max_connections"));
    }

    #[test]
    fn test_cache_config_creation() {
        let cache = CacheConfig {
            backend: CacheBackend::Memory,
            redis_url: None,
            ttl: CacheTtlConfig {
                default: Duration::from_secs(600),
                events: Duration::from_secs(1200),
                aggregates: Duration::from_secs(3600),
            },
            memory: MemoryCacheConfig {
                max_size_bytes: 50_000_000,
                max_entries: 5_000,
            },
        };

        assert_eq!(cache.backend, CacheBackend::Memory);
        assert_eq!(cache.redis_url, None);
        assert_eq!(cache.ttl.default, Duration::from_secs(600));
        assert_eq!(cache.memory.max_size_bytes, 50_000_000);
    }

    #[test]
    fn test_retention_config_creation() {
        let mut by_event_type = HashMap::new();
        by_event_type.insert("error".to_string(), Duration::from_secs(86400 * 7)); // 7 days

        let retention = RetentionConfig {
            default: Duration::from_secs(86400 * 30), // 30 days
            by_event_type,
            archive: ArchiveConfig {
                enabled: true,
                backend: None,
                compression: CompressionConfig {
                    algorithm: CompressionAlgorithm::Lz4,
                    level: 5,
                },
            },
        };

        assert_eq!(retention.default, Duration::from_secs(86400 * 30));
        assert!(retention.by_event_type.contains_key("error"));
        assert!(retention.archive.enabled);
        assert_eq!(
            retention.archive.compression.algorithm,
            CompressionAlgorithm::Lz4
        );
        assert_eq!(retention.archive.compression.level, 5);
    }

    #[test]
    fn test_processing_config_creation() {
        let processing = ProcessingConfig {
            workers: 8,
            batch: BatchConfig {
                max_size: 500,
                max_age: Duration::from_secs(10),
                target_size: 50,
            },
            queue: QueueConfig {
                capacity: 5000,
                queue_type: QueueType::Disk,
                disk_settings: Some(DiskQueueConfig {
                    directory: tempfile::tempdir().map_or_else(
                        |_| PathBuf::from("/tmp/riglr-test"),
                        |dir| dir.path().to_path_buf(),
                    ),
                    max_file_size: 1_000_000,
                    sync_strategy: SyncStrategy::Periodic,
                }),
            },
            retry: RetryConfig {
                max_attempts: 5,
                base_delay: Duration::from_millis(200),
                max_delay: Duration::from_secs(60),
                backoff_multiplier: 1.5,
                jitter: 0.2,
            },
            rate_limit: RateLimitConfig {
                enabled: false,
                max_events_per_second: 5000,
                burst_capacity: 500,
            },
        };

        assert_eq!(processing.workers, 8);
        assert_eq!(processing.batch.max_size, 500);
        assert_eq!(processing.queue.queue_type, QueueType::Disk);
        assert!(processing.queue.disk_settings.is_some());
        assert_eq!(processing.retry.max_attempts, 5);
        assert!(!processing.rate_limit.enabled);
    }

    #[test]
    fn test_api_config_creation() {
        let api = ApiConfig {
            http: HttpConfig {
                bind: "127.0.0.1".to_string(),
                port: 3000,
                timeout: Duration::from_secs(60),
                max_request_size: 5_000_000,
                keep_alive: Duration::from_secs(120),
            },
            websocket: WebSocketConfig {
                enabled: false,
                max_connections: 500,
                buffer_size: 2048,
                heartbeat_interval: Duration::from_secs(60),
            },
            graphql: Some(GraphQLConfig {
                enabled: true,
                endpoint: "/graphql".to_string(),
                playground: true,
                complexity_limit: 1000,
                depth_limit: 20,
            }),
            auth: AuthConfig {
                enabled: true,
                method: AuthMethod::Jwt,
                jwt: Some(JwtConfig {
                    secret: "secret-key".to_string(),
                    expires_in: Duration::from_secs(3600),
                    issuer: "riglr".to_string(),
                    audience: "api".to_string(),
                }),
                api_key: None,
            },
            cors: CorsConfig {
                enabled: false,
                allowed_origins: vec!["https://example.com".to_string()],
                allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string()],
                allowed_headers: vec!["Authorization".to_string()],
                max_age: Duration::from_secs(7200),
            },
        };

        assert_eq!(api.http.bind, "127.0.0.1");
        assert_eq!(api.http.port, 3000);
        assert!(!api.websocket.enabled);
        assert!(api.graphql.is_some());
        assert!(api.auth.enabled);
        assert_eq!(api.auth.method, AuthMethod::Jwt);
        assert!(!api.cors.enabled);
    }

    #[test]
    fn test_metrics_config_creation() {
        let mut custom = HashMap::new();
        custom.insert(
            "request_duration".to_string(),
            CustomMetricConfig {
                metric_type: MetricType::Histogram,
                description: "Request duration in seconds".to_string(),
                labels: vec!["method".to_string(), "status".to_string()],
            },
        );

        let metrics = MetricsConfig {
            enabled: true,
            port: 9091,
            endpoint: "/custom-metrics".to_string(),
            collection_interval: Duration::from_secs(30),
            histogram_buckets: vec![0.1, 0.5, 1.0, 5.0],
            custom,
        };

        assert!(metrics.enabled);
        assert_eq!(metrics.port, 9091);
        assert_eq!(metrics.endpoint, "/custom-metrics");
        assert_eq!(metrics.histogram_buckets, vec![0.1, 0.5, 1.0, 5.0]);
        assert!(metrics.custom.contains_key("request_duration"));
    }

    #[test]
    fn test_logging_config_creation() {
        let mut custom_fields = HashMap::new();
        custom_fields.insert("app".to_string(), "riglr-indexer".to_string());

        let logging = LoggingConfig {
            level: "debug".to_string(),
            format: LogFormat::Pretty,
            outputs: vec![
                LogOutput::Stderr,
                LogOutput::File(PathBuf::from("/var/log/app.log")),
            ],
            structured: StructuredLoggingConfig {
                include_location: true,
                include_thread: false,
                include_service_metadata: false,
                custom_fields,
            },
        };

        assert_eq!(logging.level, "debug");
        assert_eq!(logging.format, LogFormat::Pretty);
        assert_eq!(logging.outputs.len(), 2);
        assert!(logging.structured.include_location);
        assert!(!logging.structured.include_thread);
        assert!(logging.structured.custom_fields.contains_key("app"));
    }

    #[test]
    fn test_feature_config_creation() {
        let mut custom = HashMap::new();
        custom.insert("beta_feature".to_string(), true);
        custom.insert("legacy_support".to_string(), false);

        let features = FeatureConfig {
            core_features: CoreFeatureSet {
                streaming: FeatureState::Disabled,
                archival: FeatureState::Disabled,
            },
            api_features: ApiFeatureSet {
                graphql: FeatureState::Enabled,
            },
            dev_features: DevFeatureSet {
                experimental: FeatureState::Enabled,
            },
            custom,
        };

        assert!(!features.core_features.streaming.is_enabled());
        assert!(!features.core_features.archival.is_enabled());
        assert!(features.api_features.graphql.is_enabled());
        assert!(features.dev_features.experimental.is_enabled());
        assert_eq!(features.custom.get("beta_feature"), Some(&true));
        assert_eq!(features.custom.get("legacy_support"), Some(&false));
    }

    #[test]
    fn test_enum_equality() {
        assert_eq!(StorageBackend::Postgres, StorageBackend::Postgres);
        assert_ne!(StorageBackend::Postgres, StorageBackend::ClickHouse);

        assert_eq!(CacheBackend::Redis, CacheBackend::Redis);
        assert_ne!(CacheBackend::Redis, CacheBackend::Memory);

        assert_eq!(CompressionAlgorithm::Zstd, CompressionAlgorithm::Zstd);
        assert_ne!(CompressionAlgorithm::Zstd, CompressionAlgorithm::Gzip);
    }

    #[test]
    fn test_partial_eq_derive() {
        let backend1 = StorageBackend::Postgres;
        let backend2 = StorageBackend::Postgres;
        let backend3 = StorageBackend::ClickHouse;

        assert_eq!(backend1, backend2);
        assert_ne!(backend1, backend3);
    }

    #[test]
    fn test_compression_config_edge_cases() {
        let compression = CompressionConfig {
            algorithm: CompressionAlgorithm::None,
            level: 0,
        };

        assert_eq!(compression.algorithm, CompressionAlgorithm::None);
        assert_eq!(compression.level, 0);
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_auth_config_with_api_key() {
        let auth = AuthConfig {
            enabled: true,
            method: AuthMethod::ApiKey,
            jwt: None,
            api_key: Some(ApiKeyConfig {
                header_name: "X-API-Key".to_string(),
                keys: vec!["key1".to_string(), "key2".to_string()],
            }),
        };

        assert!(auth.enabled);
        assert_eq!(auth.method, AuthMethod::ApiKey);
        assert!(auth.jwt.is_none());
        assert!(auth.api_key.is_some());

        if let Some(api_key_config) = auth.api_key {
            assert_eq!(api_key_config.header_name, "X-API-Key");
            assert_eq!(api_key_config.keys.len(), 2);
        } else {
            panic!("Expected API key config to be present in test");
        }
    }

    #[test]
    fn test_memory_cache_config_values() {
        let memory_config = MemoryCacheConfig {
            max_size_bytes: u64::MAX,
            max_entries: 0,
        };

        assert_eq!(memory_config.max_size_bytes, u64::MAX);
        assert_eq!(memory_config.max_entries, 0);
    }

    #[test]
    fn test_retry_config_edge_values() {
        let retry = RetryConfig {
            max_attempts: u32::MAX,
            base_delay: Duration::from_nanos(1),
            max_delay: Duration::from_secs(u64::MAX / 1000), // Large but valid duration
            backoff_multiplier: 0.0,
            jitter: 1.0,
        };

        assert_eq!(retry.max_attempts, u32::MAX);
        assert_eq!(retry.base_delay, Duration::from_nanos(1));
        {
            #[expect(clippy::float_cmp)] // Testing exact values from literals
            {
                assert_eq!(retry.backoff_multiplier, 0.0);
                assert_eq!(retry.jitter, 1.0);
            }
        }
    }

    #[test]
    fn test_rate_limit_config_disabled() {
        let rate_limit = RateLimitConfig {
            enabled: false,
            max_events_per_second: 0,
            burst_capacity: 0,
        };

        assert!(!rate_limit.enabled);
        assert_eq!(rate_limit.max_events_per_second, 0);
        assert_eq!(rate_limit.burst_capacity, 0);
    }

    #[test]
    fn test_http_config_minimal_values() {
        let http = HttpConfig {
            bind: String::default(),
            port: 1,
            timeout: Duration::from_nanos(1),
            max_request_size: 0,
            keep_alive: Duration::from_nanos(1),
        };

        assert!(http.bind.is_empty());
        assert_eq!(http.port, 1);
        assert_eq!(http.timeout, Duration::from_nanos(1));
        assert_eq!(http.max_request_size, 0);
        assert_eq!(http.keep_alive, Duration::from_nanos(1));
    }

    #[test]
    fn test_websocket_config_large_values() {
        let ws = WebSocketConfig {
            enabled: true,
            max_connections: u32::MAX,
            buffer_size: usize::MAX,
            heartbeat_interval: Duration::from_secs(86400), // 1 day
        };

        assert!(ws.enabled);
        assert_eq!(ws.max_connections, u32::MAX);
        assert_eq!(ws.buffer_size, usize::MAX);
        assert_eq!(ws.heartbeat_interval, Duration::from_secs(86400));
    }

    #[test]
    fn test_graphql_config_limits() {
        let graphql = GraphQLConfig {
            enabled: false,
            endpoint: "/".to_string(),
            playground: false,
            complexity_limit: 0,
            depth_limit: 0,
        };

        assert!(!graphql.enabled);
        assert_eq!(graphql.endpoint, "/");
        assert!(!graphql.playground);
        assert_eq!(graphql.complexity_limit, 0);
        assert_eq!(graphql.depth_limit, 0);
    }

    #[test]
    fn test_cors_config_empty_lists() {
        let cors = CorsConfig {
            enabled: true,
            allowed_origins: vec![],
            allowed_methods: vec![],
            allowed_headers: vec![],
            max_age: Duration::from_secs(0),
        };

        assert!(cors.enabled);
        assert!(cors.allowed_origins.is_empty());
        assert!(cors.allowed_methods.is_empty());
        assert!(cors.allowed_headers.is_empty());
        assert_eq!(cors.max_age, Duration::from_secs(0));
    }

    #[test]
    fn test_jwt_config_empty_strings() {
        let jwt = JwtConfig {
            secret: String::default(),
            expires_in: Duration::from_secs(0),
            issuer: String::default(),
            audience: String::default(),
        };

        assert!(jwt.secret.is_empty());
        assert_eq!(jwt.expires_in, Duration::from_secs(0));
        assert!(jwt.issuer.is_empty());
        assert!(jwt.audience.is_empty());
    }

    #[test]
    fn test_disk_queue_config_edge_cases() {
        let temp_dir = tempfile::tempdir().map_or_else(
            |_| PathBuf::from("/tmp/riglr-test"),
            |dir| dir.path().to_path_buf(),
        );

        let disk_queue = DiskQueueConfig {
            directory: temp_dir.clone(),
            max_file_size: 0,
            sync_strategy: SyncStrategy::Never,
        };

        assert_eq!(disk_queue.directory, temp_dir);
        assert_eq!(disk_queue.max_file_size, 0);
        assert_eq!(disk_queue.sync_strategy, SyncStrategy::Never);
    }

    #[test]
    fn test_custom_metric_config_empty_labels() {
        let metric = CustomMetricConfig {
            metric_type: MetricType::Counter,
            description: String::default(),
            labels: vec![],
        };

        assert_eq!(metric.metric_type, MetricType::Counter);
        assert!(metric.description.is_empty());
        assert!(metric.labels.is_empty());
    }

    #[test]
    fn test_structured_logging_config_all_false() {
        let structured = StructuredLoggingConfig {
            include_location: false,
            include_thread: false,
            include_service_metadata: false,
            custom_fields: HashMap::new(),
        };

        assert!(!structured.include_location);
        assert!(!structured.include_thread);
        assert!(!structured.include_service_metadata);
        assert!(structured.custom_fields.is_empty());
    }

    #[test]
    fn test_batch_config_zero_values() {
        let batch = BatchConfig {
            max_size: 0,
            max_age: Duration::from_secs(0),
            target_size: 0,
        };

        assert_eq!(batch.max_size, 0);
        assert_eq!(batch.max_age, Duration::from_secs(0));
        assert_eq!(batch.target_size, 0);
    }

    #[test]
    fn test_connection_pool_config_minimal() {
        let pool = ConnectionPoolConfig {
            max_connections: 1,
            min_connections: 0,
            connect_timeout: Duration::from_nanos(1),
            idle_timeout: Duration::from_nanos(1),
            max_lifetime: Duration::from_nanos(1),
        };

        assert_eq!(pool.max_connections, 1);
        assert_eq!(pool.min_connections, 0);
        assert_eq!(pool.connect_timeout, Duration::from_nanos(1));
        assert_eq!(pool.idle_timeout, Duration::from_nanos(1));
        assert_eq!(pool.max_lifetime, Duration::from_nanos(1));
    }
}
