//! Configuration management for the RIGLR indexer service

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::error::{ConfigError, IndexerResult};

/// Main configuration for the indexer service
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IndexerConfig {
    /// Service configuration
    pub service: ServiceConfig,
    
    /// Storage configuration
    pub storage: StorageConfig,
    
    /// Processing configuration  
    pub processing: ProcessingConfig,
    
    /// API server configuration
    pub api: ApiConfig,
    
    /// Metrics configuration
    pub metrics: MetricsConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
    
    /// Feature flags
    pub features: FeatureConfig,
}

/// Service-level configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceConfig {
    /// Service name for identification
    pub name: String,
    
    /// Service version
    pub version: String,
    
    /// Environment (development, staging, production)
    pub environment: String,
    
    /// Node ID for clustering
    pub node_id: Option<String>,
    
    /// Graceful shutdown timeout
    #[serde(with = "humantime_serde")]
    pub shutdown_timeout: Duration,
    
    /// Health check interval
    #[serde(with = "humantime_serde")]
    pub health_check_interval: Duration,
}

/// Storage configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// Primary storage backend
    pub primary: StorageBackendConfig,
    
    /// Optional secondary storage for archival
    pub secondary: Option<StorageBackendConfig>,
    
    /// Cache configuration
    pub cache: CacheConfig,
    
    /// Data retention policies
    pub retention: RetentionConfig,
}

/// Storage backend configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageBackendConfig {
    /// Backend type (postgres, clickhouse, etc.)
    pub backend: StorageBackend,
    
    /// Connection URL
    pub url: String,
    
    /// Connection pool settings
    pub pool: ConnectionPoolConfig,
    
    /// Database-specific settings
    pub settings: HashMap<String, serde_json::Value>,
}

/// Storage backend types
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Postgres,
    ClickHouse,
    TimescaleDB,
    MongoDB,
}

/// Connection pool configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections
    pub max_connections: u32,
    
    /// Minimum number of idle connections
    pub min_connections: u32,
    
    /// Connection timeout
    #[serde(with = "humantime_serde")]
    pub connect_timeout: Duration,
    
    /// Idle timeout
    #[serde(with = "humantime_serde")]
    pub idle_timeout: Duration,
    
    /// Maximum connection lifetime
    #[serde(with = "humantime_serde")]
    pub max_lifetime: Duration,
}

/// Cache configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    /// Cache backend (redis, memory)
    pub backend: CacheBackend,
    
    /// Redis URL (if using Redis)
    pub redis_url: Option<String>,
    
    /// Cache TTL settings
    pub ttl: CacheTtlConfig,
    
    /// Memory cache settings
    pub memory: MemoryCacheConfig,
}

/// Cache backend types
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheBackend {
    Redis,
    Memory,
    Disabled,
}

/// Cache TTL configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheTtlConfig {
    /// Default TTL for cached items
    #[serde(with = "humantime_serde")]
    pub default: Duration,
    
    /// TTL for event metadata
    #[serde(with = "humantime_serde")]
    pub events: Duration,
    
    /// TTL for aggregated data
    #[serde(with = "humantime_serde")]
    pub aggregates: Duration,
}

/// Memory cache settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryCacheConfig {
    /// Maximum memory usage in bytes
    pub max_size_bytes: u64,
    
    /// Maximum number of entries
    pub max_entries: u64,
}

/// Data retention configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetentionConfig {
    /// Default retention period
    #[serde(with = "humantime_serde")]
    pub default: Duration,
    
    /// Retention by event type
    pub by_event_type: HashMap<String, Duration>,
    
    /// Archive configuration
    pub archive: ArchiveConfig,
}

/// Archive configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArchiveConfig {
    /// Whether archival is enabled
    pub enabled: bool,
    
    /// Archive storage backend
    pub backend: Option<StorageBackendConfig>,
    
    /// Compression settings
    pub compression: CompressionConfig,
}

/// Compression configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompressionConfig {
    /// Compression algorithm
    pub algorithm: CompressionAlgorithm,
    
    /// Compression level (1-9)
    pub level: u8,
}

/// Compression algorithms
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionAlgorithm {
    Zstd,
    Gzip,
    Lz4,
    None,
}

/// Processing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProcessingConfig {
    /// Number of worker threads
    pub workers: usize,
    
    /// Batch processing settings
    pub batch: BatchConfig,
    
    /// Event queue configuration
    pub queue: QueueConfig,
    
    /// Retry configuration
    pub retry: RetryConfig,
    
    /// Rate limiting
    pub rate_limit: RateLimitConfig,
}

/// Batch processing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchConfig {
    /// Maximum batch size
    pub max_size: usize,
    
    /// Maximum batch age before forcing flush
    #[serde(with = "humantime_serde")]
    pub max_age: Duration,
    
    /// Target batch size for optimal performance
    pub target_size: usize,
}

/// Event queue configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueueConfig {
    /// Maximum queue capacity
    pub capacity: usize,
    
    /// Queue type (memory, disk, hybrid)
    pub queue_type: QueueType,
    
    /// Disk queue settings (if using disk queue)
    pub disk_settings: Option<DiskQueueConfig>,
}

/// Queue type options
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum QueueType {
    Memory,
    Disk,
    Hybrid,
}

/// Disk queue configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiskQueueConfig {
    /// Directory for queue files
    pub directory: PathBuf,
    
    /// Maximum size per file
    pub max_file_size: u64,
    
    /// Sync strategy
    pub sync_strategy: SyncStrategy,
}

/// Disk sync strategies
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncStrategy {
    Always,
    Periodic,
    Never,
}

/// Retry configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    
    /// Base delay between retries
    #[serde(with = "humantime_serde")]
    pub base_delay: Duration,
    
    /// Maximum delay between retries
    #[serde(with = "humantime_serde")]
    pub max_delay: Duration,
    
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    
    /// Jitter factor (0.0 to 1.0)
    pub jitter: f64,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitConfig {
    /// Whether rate limiting is enabled
    pub enabled: bool,
    
    /// Maximum events per second
    pub max_events_per_second: u32,
    
    /// Burst capacity
    pub burst_capacity: u32,
}

/// API server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiConfig {
    /// HTTP server settings
    pub http: HttpConfig,
    
    /// WebSocket settings
    pub websocket: WebSocketConfig,
    
    /// GraphQL settings
    pub graphql: Option<GraphQLConfig>,
    
    /// Authentication settings
    pub auth: AuthConfig,
    
    /// CORS settings
    pub cors: CorsConfig,
}

/// HTTP server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpConfig {
    /// Server bind address
    pub bind: String,
    
    /// Server port
    pub port: u16,
    
    /// Request timeout
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    
    /// Maximum request size
    pub max_request_size: u64,
    
    /// Keep-alive timeout
    #[serde(with = "humantime_serde")]
    pub keep_alive: Duration,
}

/// WebSocket configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebSocketConfig {
    /// Whether WebSocket is enabled
    pub enabled: bool,
    
    /// Maximum connections
    pub max_connections: u32,
    
    /// Message buffer size
    pub buffer_size: usize,
    
    /// Heartbeat interval
    #[serde(with = "humantime_serde")]
    pub heartbeat_interval: Duration,
}

/// GraphQL configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphQLConfig {
    /// Whether GraphQL is enabled
    pub enabled: bool,
    
    /// GraphQL endpoint path
    pub endpoint: String,
    
    /// Playground enabled
    pub playground: bool,
    
    /// Query complexity limit
    pub complexity_limit: u32,
    
    /// Query depth limit
    pub depth_limit: u32,
}

/// Authentication configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// Whether authentication is enabled
    pub enabled: bool,
    
    /// Authentication method
    pub method: AuthMethod,
    
    /// JWT settings (if using JWT)
    pub jwt: Option<JwtConfig>,
    
    /// API key settings (if using API keys)
    pub api_key: Option<ApiKeyConfig>,
}

/// Authentication methods
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    None,
    ApiKey,
    Jwt,
    OAuth2,
}

/// JWT configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtConfig {
    /// JWT secret key
    pub secret: String,
    
    /// Token expiration time
    #[serde(with = "humantime_serde")]
    pub expires_in: Duration,
    
    /// Issuer
    pub issuer: String,
    
    /// Audience
    pub audience: String,
}

/// API key configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiKeyConfig {
    /// Header name for API key
    pub header_name: String,
    
    /// Valid API keys
    pub keys: Vec<String>,
}

/// CORS configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorsConfig {
    /// Whether CORS is enabled
    pub enabled: bool,
    
    /// Allowed origins
    pub allowed_origins: Vec<String>,
    
    /// Allowed methods
    pub allowed_methods: Vec<String>,
    
    /// Allowed headers
    pub allowed_headers: Vec<String>,
    
    /// Max age for preflight requests
    #[serde(with = "humantime_serde")]
    pub max_age: Duration,
}

/// Metrics configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricsConfig {
    /// Whether metrics are enabled
    pub enabled: bool,
    
    /// Metrics server port
    pub port: u16,
    
    /// Metrics endpoint path
    pub endpoint: String,
    
    /// Collection interval
    #[serde(with = "humantime_serde")]
    pub collection_interval: Duration,
    
    /// Histogram buckets
    pub histogram_buckets: Vec<f64>,
    
    /// Custom metrics
    pub custom: HashMap<String, CustomMetricConfig>,
}

/// Custom metric configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomMetricConfig {
    /// Metric type
    pub metric_type: MetricType,
    
    /// Metric description
    pub description: String,
    
    /// Metric labels
    pub labels: Vec<String>,
}

/// Metric types
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    
    /// Log format (json, text)
    pub format: LogFormat,
    
    /// Output destinations
    pub outputs: Vec<LogOutput>,
    
    /// Structured logging settings
    pub structured: StructuredLoggingConfig,
}

/// Log format options
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Text,
    Pretty,
}

/// Log output destinations
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogOutput {
    Stdout,
    Stderr,
    File(PathBuf),
    Syslog,
}

/// Structured logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructuredLoggingConfig {
    /// Include source location
    pub include_location: bool,
    
    /// Include thread information
    pub include_thread: bool,
    
    /// Include service metadata
    pub include_service_metadata: bool,
    
    /// Custom fields to include
    pub custom_fields: HashMap<String, String>,
}

/// Feature flags configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeatureConfig {
    /// Enable real-time streaming
    pub realtime_streaming: bool,
    
    /// Enable event archival
    pub archival: bool,
    
    /// Enable GraphQL API
    pub graphql_api: bool,
    
    /// Enable experimental features
    pub experimental: bool,
    
    /// Custom feature flags
    pub custom: HashMap<String, bool>,
}

impl IndexerConfig {
    /// Load configuration from environment variables and optional file
    pub fn from_env() -> IndexerResult<Self> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        // Check for custom config file
        let config_file = std::env::var("RIGLR_INDEXER_CONFIG").ok();
        
        let mut settings = config::Config::builder();
        
        // Load defaults first
        settings = settings.add_source(config::Config::try_from(&Self::default())?);
        
        // Override with file if specified
        if let Some(path) = config_file {
            settings = settings.add_source(config::File::with_name(&path).required(false));
        }
        
        // Override with environment variables
        settings = settings.add_source(
            config::Environment::with_prefix("RIGLR_INDEXER")
                .separator("__")
                .try_parsing(true)
        );

        let config = settings.build()
            .map_err(|e| ConfigError::LoadFailed {
                path: "environment".to_string(),
                source: Box::new(e),
            })?;

        let config: IndexerConfig = config.try_deserialize()
            .map_err(|e| ConfigError::LoadFailed {
                path: "deserialization".to_string(), 
                source: Box::new(e),
            })?;

        // Validate configuration
        config.validate()?;
        
        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> IndexerResult<()> {
        // Validate service config
        if self.service.name.is_empty() {
            return Err(ConfigError::MissingField {
                field: "service.name".to_string(),
            }.into());
        }

        // Validate storage config
        if self.storage.primary.url.is_empty() {
            return Err(ConfigError::MissingField {
                field: "storage.primary.url".to_string(),
            }.into());
        }

        // Validate processing config
        if self.processing.workers == 0 {
            return Err(ConfigError::InvalidValue {
                field: "processing.workers".to_string(),
                value: "0".to_string(),
            }.into());
        }

        if self.processing.batch.max_size == 0 {
            return Err(ConfigError::InvalidValue {
                field: "processing.batch.max_size".to_string(),
                value: "0".to_string(),
            }.into());
        }

        // Validate API config
        if self.api.http.port == 0 {
            return Err(ConfigError::InvalidValue {
                field: "api.http.port".to_string(),
                value: "0".to_string(),
            }.into());
        }

        // Validate metrics config if enabled
        if self.metrics.enabled && self.metrics.port == 0 {
            return Err(ConfigError::InvalidValue {
                field: "metrics.port".to_string(),
                value: "0".to_string(),
            }.into());
        }

        Ok(())
    }

    /// Get a feature flag value
    pub fn feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "realtime_streaming" => self.features.realtime_streaming,
            "archival" => self.features.archival,
            "graphql_api" => self.features.graphql_api,
            "experimental" => self.features.experimental,
            _ => self.features.custom.get(feature).copied().unwrap_or(false),
        }
    }

    /// Generate node ID if not provided
    pub fn node_id(&self) -> String {
        self.service.node_id
            .clone()
            .unwrap_or_else(|| {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                
                let mut hasher = DefaultHasher::new();
                self.service.name.hash(&mut hasher);
                std::env::var("HOSTNAME")
                    .unwrap_or_else(|_| "unknown".to_string())
                    .hash(&mut hasher);
                    
                format!("{:x}", hasher.finish())
            })
    }
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            service: ServiceConfig {
                name: "riglr-indexer".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                environment: "development".to_string(),
                node_id: None,
                shutdown_timeout: Duration::from_secs(30),
                health_check_interval: Duration::from_secs(30),
            },
            storage: StorageConfig {
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
            },
            processing: ProcessingConfig {
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
            },
            api: ApiConfig {
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
            },
            metrics: MetricsConfig {
                enabled: true,
                port: 9090,
                endpoint: "/metrics".to_string(),
                collection_interval: Duration::from_secs(15),
                histogram_buckets: vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
                custom: HashMap::new(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: LogFormat::Json,
                outputs: vec![LogOutput::Stdout],
                structured: StructuredLoggingConfig {
                    include_location: false,
                    include_thread: true,
                    include_service_metadata: true,
                    custom_fields: HashMap::new(),
                },
            },
            features: FeatureConfig {
                realtime_streaming: true,
                archival: true,
                graphql_api: false,
                experimental: false,
                custom: HashMap::new(),
            },
        }
    }
}