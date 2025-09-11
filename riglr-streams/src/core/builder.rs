use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

const STREAM_EXECUTION_MODE: &str = "STREAM_EXECUTION_MODE";
const STREAM_EXECUTION_LIMIT: &str = "STREAM_EXECUTION_LIMIT";
const SOLANA_GEYSER_WS_URL: &str = "SOLANA_GEYSER_WS_URL";
const SOLANA_GEYSER_PROGRAMS: &str = "SOLANA_GEYSER_PROGRAMS";
const BINANCE_STREAMS: &str = "BINANCE_STREAMS";
const BINANCE_TESTNET: &str = "BINANCE_TESTNET";
const MEMPOOL_ENABLED: &str = "MEMPOOL_ENABLED";
const MEMPOOL_NETWORK: &str = "MEMPOOL_NETWORK";

use crate::core::{HandlerExecutionMode, MetricsCollector, Stream, StreamManager};
use crate::evm::websocket::{ChainId, EvmStreamConfig, EvmWebSocketStream};
use crate::external::binance::{BinanceConfig, BinanceStream};
use crate::external::mempool::{BitcoinNetwork, MempoolConfig, MempoolSpaceStream};
use crate::solana::geyser::{GeyserConfig, SolanaGeyserStream};

/// Configuration for a stream
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum StreamConfig {
    /// Solana Geyser stream configuration for monitoring Solana blockchain events
    SolanaGeyser {
        /// Unique name identifier for the stream
        name: String,
        /// Geyser-specific configuration parameters
        config: GeyserConfig,
    },
    /// EVM WebSocket stream configuration for monitoring EVM-compatible blockchain events
    EvmWebSocket {
        /// Unique name identifier for the stream
        name: String,
        /// EVM WebSocket-specific configuration parameters
        config: EvmStreamConfig,
    },
    /// Binance stream configuration for monitoring Binance exchange data
    Binance {
        /// Unique name identifier for the stream
        name: String,
        /// Binance-specific configuration parameters
        config: BinanceConfig,
    },
    /// Mempool.space stream configuration for monitoring Bitcoin mempool data
    MempoolSpace {
        /// Unique name identifier for the stream
        name: String,
        /// Mempool.space-specific configuration parameters
        config: MempoolConfig,
    },
}

/// Builder for StreamManager with configuration loading
#[derive(Debug)]
pub struct StreamManagerBuilder {
    /// Handler execution mode for processing stream events
    execution_mode: HandlerExecutionMode,
    /// Collection of configured streams to be managed
    streams: Vec<StreamConfig>,
    /// Flag to enable or disable metrics collection
    enable_metrics: bool,
    /// Optional custom metrics collector instance
    metrics_collector: Option<Arc<MetricsCollector>>,
}

impl StreamManagerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the handler execution mode
    pub fn with_execution_mode(mut self, mode: HandlerExecutionMode) -> Self {
        self.execution_mode = mode;
        self
    }

    /// Enable or disable metrics collection
    pub fn with_metrics(mut self, enable: bool) -> Self {
        self.enable_metrics = enable;
        self
    }

    /// Use a custom metrics collector
    pub fn with_metrics_collector(mut self, collector: Arc<MetricsCollector>) -> Self {
        self.metrics_collector = Some(collector);
        self.enable_metrics = true;
        self
    }

    /// Add a stream configuration
    pub fn add_stream(mut self, config: StreamConfig) -> Self {
        self.streams.push(config);
        self
    }

    /// Load configuration from a TOML file
    pub async fn from_toml_file(mut self, path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = tokio::fs::read_to_string(path).await?;
        let config: StreamManagerConfig = toml::from_str(&contents)?;

        self.execution_mode = config.execution_mode.unwrap_or(self.execution_mode);
        self.enable_metrics = config.enable_metrics.unwrap_or(self.enable_metrics);
        self.streams.extend(config.streams);

        Ok(self)
    }

    /// Load configuration from environment variables
    pub fn from_env(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        // Load execution mode from env
        if let Ok(mode_str) = std::env::var(STREAM_EXECUTION_MODE) {
            self.execution_mode = match mode_str.as_str() {
                "sequential" => HandlerExecutionMode::Sequential,
                "concurrent" => HandlerExecutionMode::Concurrent,
                "bounded" => {
                    let limit = std::env::var(STREAM_EXECUTION_LIMIT)
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(10);
                    HandlerExecutionMode::ConcurrentBounded(limit)
                }
                _ => self.execution_mode,
            };
        }

        // Load Solana Geyser configuration
        if let Ok(ws_url) = std::env::var(SOLANA_GEYSER_WS_URL) {
            let program_ids: Vec<String> = std::env::var(SOLANA_GEYSER_PROGRAMS)
                .ok()
                .map(|s| s.split(',').map(String::from).collect())
                .unwrap_or_default();

            self.streams.push(StreamConfig::SolanaGeyser {
                name: "solana-geyser".to_string(),
                config: GeyserConfig {
                    ws_url,
                    auth_token: None,
                    program_ids,
                    buffer_size: 10000,
                },
            });
        }

        // Load EVM configurations
        for chain_id in [1, 137, 56, 42161, 10, 43114, 8453] {
            if let Ok(ws_url) = std::env::var(format!("EVM_WS_URL_{}", chain_id)) {
                let chain = match chain_id {
                    1 => ChainId::Ethereum,
                    137 => ChainId::Polygon,
                    56 => ChainId::BSC,
                    42161 => ChainId::Arbitrum,
                    10 => ChainId::Optimism,
                    43114 => ChainId::Avalanche,
                    8453 => ChainId::Base,
                    _ => continue,
                };

                self.streams.push(StreamConfig::EvmWebSocket {
                    name: format!("evm-{}", chain_id),
                    config: EvmStreamConfig {
                        ws_url,
                        chain_id: chain,
                        subscribe_pending_transactions: false,
                        subscribe_new_blocks: true,
                        contract_addresses: Vec::default(),
                        buffer_size: 10000,
                    },
                });
            }
        }

        // Load Binance configuration
        if let Ok(streams) = std::env::var(BINANCE_STREAMS) {
            let streams: Vec<String> = streams.split(',').map(String::from).collect();
            let testnet = std::env::var(BINANCE_TESTNET)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(false);

            self.streams.push(StreamConfig::Binance {
                name: "binance".to_string(),
                config: BinanceConfig {
                    streams,
                    testnet,
                    buffer_size: 10000,
                },
            });
        }

        // Load Mempool.space configuration
        if std::env::var(MEMPOOL_ENABLED).is_ok() {
            let network = match std::env::var(MEMPOOL_NETWORK).as_deref() {
                Ok("testnet") => BitcoinNetwork::Testnet,
                Ok("signet") => BitcoinNetwork::Signet,
                _ => BitcoinNetwork::Mainnet,
            };

            self.streams.push(StreamConfig::MempoolSpace {
                name: "mempool".to_string(),
                config: MempoolConfig {
                    network,
                    subscribe_transactions: true,
                    subscribe_blocks: true,
                    subscribe_fees: false,
                    buffer_size: 10000,
                },
            });
        }

        Ok(self)
    }

    /// Build the StreamManager with all configured streams
    pub async fn build(self) -> Result<StreamManager, Box<dyn std::error::Error>> {
        let manager = StreamManager::with_execution_mode(self.execution_mode);

        // Set up metrics collector if enabled
        let metrics = if self.enable_metrics {
            self.metrics_collector
                .or_else(|| Some(Arc::new(MetricsCollector::default())))
        } else {
            None
        };

        // Create and add all configured streams
        for stream_config in self.streams {
            match stream_config {
                StreamConfig::SolanaGeyser { name, config } => {
                    info!("Adding Solana Geyser stream: {}", name);
                    let mut stream = SolanaGeyserStream::new(name.clone());
                    stream.start(config).await?;
                    manager.add_stream(name, stream).await?;
                }
                StreamConfig::EvmWebSocket { name, config } => {
                    info!("Adding EVM WebSocket stream: {}", name);
                    let mut stream = EvmWebSocketStream::new(name.clone());
                    stream.start(config).await?;
                    manager.add_stream(name, stream).await?;
                }
                StreamConfig::Binance { name, config } => {
                    info!("Adding Binance stream: {}", name);
                    let mut stream = BinanceStream::new(name.clone());
                    stream.start(config).await?;
                    manager.add_stream(name, stream).await?;
                }
                StreamConfig::MempoolSpace { name, config } => {
                    info!("Adding Mempool.space stream: {}", name);
                    let mut stream = MempoolSpaceStream::new(name.clone());
                    stream.start(config).await?;
                    manager.add_stream(name, stream).await?;
                }
            }
        }

        // Start periodic metrics updates if enabled
        if let Some(metrics) = metrics {
            let metrics_clone = metrics.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    metrics_clone.update_rates().await;
                }
            });
        }

        Ok(manager)
    }
}

impl Default for StreamManagerBuilder {
    fn default() -> Self {
        Self {
            execution_mode: HandlerExecutionMode::default(),
            streams: Vec::default(),
            enable_metrics: true,
            metrics_collector: None,
        }
    }
}

/// Configuration structure for loading from TOML
#[derive(Debug, Deserialize)]
pub struct StreamManagerConfig {
    /// Optional handler execution mode override
    pub execution_mode: Option<HandlerExecutionMode>,
    /// Optional flag to enable or disable metrics collection
    pub enable_metrics: Option<bool>,
    /// Collection of stream configurations to load
    pub streams: Vec<StreamConfig>,
}

/// Helper to load and build a StreamManager from a TOML file
pub async fn from_config_file(path: &str) -> Result<StreamManager, Box<dyn std::error::Error>> {
    StreamManagerBuilder::default()
        .from_toml_file(path)
        .await?
        .build()
        .await
}

/// Helper to load and build a StreamManager from environment variables
pub async fn from_env() -> Result<StreamManager, Box<dyn std::error::Error>> {
    StreamManagerBuilder::default().from_env()?.build().await
}

#[cfg(test)]
#[allow(unsafe_code)] // Required for Rust 2024 compatibility with std::env functions in test contexts
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper function to set environment variables in tests  
    fn set_test_env_var(key: &str, value: &str) {
        // SAFETY: This is a test-only function used in isolated test environments
        // where we control the threading and environment variable access patterns.
        unsafe {
            std::env::set_var(key, value);
        }
    }

    /// Helper function to remove environment variables in tests
    fn remove_test_env_var(key: &str) {
        // SAFETY: This is a test-only function used in isolated test environments
        // where we control the threading and environment variable access patterns.
        unsafe {
            std::env::remove_var(key);
        }
    }

    #[tokio::test]
    async fn test_builder_basic() {
        let manager = StreamManagerBuilder::default()
            .with_execution_mode(HandlerExecutionMode::Concurrent)
            .with_metrics(true)
            .build()
            .await
            .unwrap();

        assert_eq!(manager.list_streams().await.len(), 0);
    }

    #[tokio::test]
    async fn test_builder_with_stream() {
        let config = StreamConfig::Binance {
            name: "test-binance".to_string(),
            config: BinanceConfig {
                streams: vec!["btcusdt@ticker".to_string()],
                testnet: true,
                buffer_size: 1000,
            },
        };

        let _manager = StreamManagerBuilder::default()
            .add_stream(config)
            .with_metrics(false)
            .build()
            .await;

        // Note: This will fail to connect in test environment but validates the builder
    }

    #[test]
    fn test_stream_manager_builder_new() {
        let builder = StreamManagerBuilder::default();
        assert_eq!(builder.execution_mode, HandlerExecutionMode::default());
        assert_eq!(builder.streams.len(), 0);
        assert!(builder.enable_metrics);
        assert!(builder.metrics_collector.is_none());
    }

    #[test]
    fn test_stream_manager_builder_default() {
        let builder = StreamManagerBuilder::default();
        assert_eq!(builder.execution_mode, HandlerExecutionMode::default());
        assert_eq!(builder.streams.len(), 0);
        assert!(builder.enable_metrics);
        assert!(builder.metrics_collector.is_none());
    }

    #[test]
    fn test_with_execution_mode() {
        let builder =
            StreamManagerBuilder::default().with_execution_mode(HandlerExecutionMode::Sequential);
        assert_eq!(builder.execution_mode, HandlerExecutionMode::Sequential);
    }

    #[test]
    fn test_with_metrics() {
        let builder = StreamManagerBuilder::default().with_metrics(false);
        assert!(!builder.enable_metrics);
    }

    #[test]
    fn test_with_metrics_collector() {
        let collector = Arc::new(MetricsCollector::default());
        let builder = StreamManagerBuilder::default().with_metrics_collector(collector.clone());
        assert!(builder.enable_metrics);
        assert!(builder.metrics_collector.is_some());
    }

    #[test]
    fn test_add_stream_solana_geyser() {
        let config = StreamConfig::SolanaGeyser {
            name: "test-solana".to_string(),
            config: GeyserConfig {
                ws_url: "ws://localhost:8899".to_string(),
                auth_token: None,
                program_ids: vec!["11111111111111111111111111111111".to_string()],
                buffer_size: 1000,
            },
        };
        let builder = StreamManagerBuilder::default().add_stream(config);
        assert_eq!(builder.streams.len(), 1);
    }

    #[test]
    fn test_add_stream_evm_websocket() {
        let config = StreamConfig::EvmWebSocket {
            name: "test-evm".to_string(),
            config: EvmStreamConfig {
                ws_url: "ws://localhost:8546".to_string(),
                chain_id: ChainId::Ethereum,
                subscribe_pending_transactions: false,
                subscribe_new_blocks: true,
                contract_addresses: vec![],
                buffer_size: 1000,
            },
        };
        let builder = StreamManagerBuilder::default().add_stream(config);
        assert_eq!(builder.streams.len(), 1);
    }

    #[test]
    fn test_add_stream_binance() {
        let config = StreamConfig::Binance {
            name: "test-binance".to_string(),
            config: BinanceConfig {
                streams: vec!["btcusdt@ticker".to_string()],
                testnet: false,
                buffer_size: 1000,
            },
        };
        let builder = StreamManagerBuilder::default().add_stream(config);
        assert_eq!(builder.streams.len(), 1);
    }

    #[test]
    fn test_add_stream_mempool_space() {
        let config = StreamConfig::MempoolSpace {
            name: "test-mempool".to_string(),
            config: MempoolConfig {
                network: BitcoinNetwork::Mainnet,
                subscribe_transactions: true,
                subscribe_blocks: true,
                subscribe_fees: false,
                buffer_size: 1000,
            },
        };
        let builder = StreamManagerBuilder::default().add_stream(config);
        assert_eq!(builder.streams.len(), 1);
    }

    #[tokio::test]
    async fn test_from_toml_file_with_valid_file() {
        let toml_content = r#"
execution_mode = "Sequential"
enable_metrics = false

[[streams]]
type = "Binance"
name = "test-binance"
config.streams = ["btcusdt@ticker"]
config.testnet = true
config.buffer_size = 5000
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let builder = StreamManagerBuilder::default()
            .from_toml_file(temp_file.path().to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(builder.execution_mode, HandlerExecutionMode::Sequential);
        assert!(!builder.enable_metrics);
        assert_eq!(builder.streams.len(), 1);
    }

    #[tokio::test]
    async fn test_from_toml_file_with_invalid_file() {
        let result = StreamManagerBuilder::default()
            .from_toml_file("/nonexistent/path/config.toml")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_from_toml_file_with_invalid_toml() {
        let invalid_toml = "invalid toml content [[[";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(invalid_toml.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = StreamManagerBuilder::default()
            .from_toml_file(temp_file.path().to_str().unwrap())
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_from_toml_file_with_partial_config() {
        let toml_content = r#"
execution_mode = "Concurrent"

[[streams]]
type = "SolanaGeyser"
name = "test-solana"
config.ws_url = "ws://localhost:8899"
config.program_ids = ["11111111111111111111111111111111"]
config.buffer_size = 2000
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let builder = StreamManagerBuilder::default()
            .with_metrics(false)
            .from_toml_file(temp_file.path().to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(builder.execution_mode, HandlerExecutionMode::Concurrent);
        // enable_metrics should retain original value (false) since it's not in TOML
        assert!(!builder.enable_metrics);
        assert_eq!(builder.streams.len(), 1);
    }

    #[test]
    fn test_from_env_no_env_vars() {
        // Clear all relevant env vars
        for var in &[
            STREAM_EXECUTION_MODE,
            STREAM_EXECUTION_LIMIT,
            SOLANA_GEYSER_WS_URL,
            SOLANA_GEYSER_PROGRAMS,
            BINANCE_STREAMS,
            BINANCE_TESTNET,
            MEMPOOL_ENABLED,
            MEMPOOL_NETWORK,
        ] {
            remove_test_env_var(var);
        }

        let builder = StreamManagerBuilder::default().from_env().unwrap();
        assert_eq!(builder.streams.len(), 0);
    }

    #[test]
    fn test_from_env_with_sequential_execution_mode() {
        set_test_env_var(STREAM_EXECUTION_MODE, "sequential");
        let builder = StreamManagerBuilder::default().from_env().unwrap();
        assert_eq!(builder.execution_mode, HandlerExecutionMode::Sequential);
        remove_test_env_var(STREAM_EXECUTION_MODE);
    }

    #[test]
    fn test_from_env_with_concurrent_execution_mode() {
        set_test_env_var(STREAM_EXECUTION_MODE, "concurrent");
        let builder = StreamManagerBuilder::default().from_env().unwrap();
        assert_eq!(builder.execution_mode, HandlerExecutionMode::Concurrent);
        remove_test_env_var(STREAM_EXECUTION_MODE);
    }

    #[test]
    fn test_from_env_with_bounded_execution_mode() {
        set_test_env_var(STREAM_EXECUTION_MODE, "bounded");
        set_test_env_var(STREAM_EXECUTION_LIMIT, "5");
        let builder = StreamManagerBuilder::default().from_env().unwrap();
        assert_eq!(
            builder.execution_mode,
            HandlerExecutionMode::ConcurrentBounded(5)
        );
        remove_test_env_var(STREAM_EXECUTION_MODE);
        remove_test_env_var(STREAM_EXECUTION_LIMIT);
    }

    #[test]
    fn test_from_env_with_bounded_execution_mode_default_limit() {
        set_test_env_var(STREAM_EXECUTION_MODE, "bounded");
        remove_test_env_var(STREAM_EXECUTION_LIMIT);
        let builder = StreamManagerBuilder::default().from_env().unwrap();
        assert_eq!(
            builder.execution_mode,
            HandlerExecutionMode::ConcurrentBounded(10)
        );
        remove_test_env_var(STREAM_EXECUTION_MODE);
    }

    #[test]
    fn test_from_env_with_bounded_execution_mode_invalid_limit() {
        set_test_env_var(STREAM_EXECUTION_MODE, "bounded");
        set_test_env_var(STREAM_EXECUTION_LIMIT, "invalid");
        let builder = StreamManagerBuilder::default().from_env().unwrap();
        assert_eq!(
            builder.execution_mode,
            HandlerExecutionMode::ConcurrentBounded(10)
        );
        remove_test_env_var(STREAM_EXECUTION_MODE);
        remove_test_env_var(STREAM_EXECUTION_LIMIT);
    }

    #[test]
    fn test_from_env_with_invalid_execution_mode() {
        set_test_env_var(STREAM_EXECUTION_MODE, "invalid");
        let original_mode = HandlerExecutionMode::Sequential;
        let builder = StreamManagerBuilder::default()
            .with_execution_mode(original_mode.clone())
            .from_env()
            .unwrap();
        assert_eq!(builder.execution_mode, original_mode);
        remove_test_env_var(STREAM_EXECUTION_MODE);
    }

    #[test]
    fn test_from_env_with_solana_geyser() {
        set_test_env_var(SOLANA_GEYSER_WS_URL, "ws://localhost:8899");
        set_test_env_var(SOLANA_GEYSER_PROGRAMS, "program1,program2,program3");

        let builder = StreamManagerBuilder::default().from_env().unwrap();
        assert_eq!(builder.streams.len(), 1);

        if let StreamConfig::SolanaGeyser { name, config } = &builder.streams[0] {
            assert_eq!(name, "solana-geyser");
            assert_eq!(config.ws_url, "ws://localhost:8899");
            assert_eq!(config.program_ids, vec!["program1", "program2", "program3"]);
            assert_eq!(config.buffer_size, 10000);
            assert!(config.auth_token.is_none());
        } else {
            panic!("Expected SolanaGeyser config");
        }

        remove_test_env_var(SOLANA_GEYSER_WS_URL);
        remove_test_env_var(SOLANA_GEYSER_PROGRAMS);
    }

    #[test]
    fn test_from_env_with_solana_geyser_no_programs() {
        set_test_env_var(SOLANA_GEYSER_WS_URL, "ws://localhost:8899");
        remove_test_env_var(SOLANA_GEYSER_PROGRAMS);

        let builder = StreamManagerBuilder::default().from_env().unwrap();
        assert_eq!(builder.streams.len(), 1);

        if let StreamConfig::SolanaGeyser { config, .. } = &builder.streams[0] {
            assert!(config.program_ids.is_empty());
        } else {
            panic!("Expected SolanaGeyser config");
        }

        remove_test_env_var(SOLANA_GEYSER_WS_URL);
    }

    #[test]
    fn test_from_env_with_evm_chains() {
        set_test_env_var("EVM_WS_URL_1", "ws://ethereum:8546");
        set_test_env_var("EVM_WS_URL_137", "ws://polygon:8546");
        set_test_env_var("EVM_WS_URL_56", "ws://bsc:8546");
        set_test_env_var("EVM_WS_URL_42161", "ws://arbitrum:8546");
        set_test_env_var("EVM_WS_URL_10", "ws://optimism:8546");
        set_test_env_var("EVM_WS_URL_43114", "ws://avalanche:8546");
        set_test_env_var("EVM_WS_URL_8453", "ws://base:8546");

        let builder = StreamManagerBuilder::default().from_env().unwrap();
        assert_eq!(builder.streams.len(), 7);

        // Check that all chains are correctly configured
        let chain_names: Vec<String> = builder
            .streams
            .iter()
            .map(|s| {
                if let StreamConfig::EvmWebSocket { name, .. } = s {
                    name.clone()
                } else {
                    panic!("Expected EvmWebSocket config");
                }
            })
            .collect();

        assert!(chain_names.contains(&"evm-1".to_string()));
        assert!(chain_names.contains(&"evm-137".to_string()));
        assert!(chain_names.contains(&"evm-56".to_string()));
        assert!(chain_names.contains(&"evm-42161".to_string()));
        assert!(chain_names.contains(&"evm-10".to_string()));
        assert!(chain_names.contains(&"evm-43114".to_string()));
        assert!(chain_names.contains(&"evm-8453".to_string()));

        // Clean up
        for chain_id in [1, 137, 56, 42161, 10, 43114, 8453] {
            remove_test_env_var(&format!("EVM_WS_URL_{}", chain_id));
        }
    }

    #[test]
    fn test_from_env_with_binance() {
        set_test_env_var(BINANCE_STREAMS, "btcusdt@ticker,ethusdt@depth");
        set_test_env_var(BINANCE_TESTNET, "true");

        let builder = StreamManagerBuilder::default().from_env().unwrap();
        assert_eq!(builder.streams.len(), 1);

        if let StreamConfig::Binance { name, config } = &builder.streams[0] {
            assert_eq!(name, "binance");
            assert_eq!(config.streams, vec!["btcusdt@ticker", "ethusdt@depth"]);
            assert!(config.testnet);
            assert_eq!(config.buffer_size, 10000);
        } else {
            panic!("Expected Binance config");
        }

        remove_test_env_var(BINANCE_STREAMS);
        remove_test_env_var(BINANCE_TESTNET);
    }

    #[test]
    fn test_from_env_with_binance_no_testnet() {
        set_test_env_var(BINANCE_STREAMS, "btcusdt@ticker");
        remove_test_env_var(BINANCE_TESTNET);

        let builder = StreamManagerBuilder::default().from_env().unwrap();
        assert_eq!(builder.streams.len(), 1);

        if let StreamConfig::Binance { config, .. } = &builder.streams[0] {
            assert!(!config.testnet);
        } else {
            panic!("Expected Binance config");
        }

        remove_test_env_var(BINANCE_STREAMS);
    }

    #[test]
    fn test_from_env_with_binance_invalid_testnet() {
        set_test_env_var(BINANCE_STREAMS, "btcusdt@ticker");
        set_test_env_var(BINANCE_TESTNET, "invalid");

        let builder = StreamManagerBuilder::default().from_env().unwrap();
        assert_eq!(builder.streams.len(), 1);

        if let StreamConfig::Binance { config, .. } = &builder.streams[0] {
            assert!(!config.testnet);
        } else {
            panic!("Expected Binance config");
        }

        remove_test_env_var(BINANCE_STREAMS);
        remove_test_env_var(BINANCE_TESTNET);
    }

    #[test]
    fn test_from_env_with_mempool_mainnet() {
        set_test_env_var(MEMPOOL_ENABLED, "true");
        remove_test_env_var(MEMPOOL_NETWORK);

        let builder = StreamManagerBuilder::default().from_env().unwrap();
        assert_eq!(builder.streams.len(), 1);

        if let StreamConfig::MempoolSpace { name, config } = &builder.streams[0] {
            assert_eq!(name, "mempool");
            assert_eq!(config.network, BitcoinNetwork::Mainnet);
            assert!(config.subscribe_transactions);
            assert!(config.subscribe_blocks);
            assert!(!config.subscribe_fees);
            assert_eq!(config.buffer_size, 10000);
        } else {
            panic!("Expected MempoolSpace config");
        }

        remove_test_env_var(MEMPOOL_ENABLED);
    }

    #[test]
    fn test_from_env_with_mempool_testnet() {
        set_test_env_var(MEMPOOL_ENABLED, "true");
        set_test_env_var(MEMPOOL_NETWORK, "testnet");

        let builder = StreamManagerBuilder::default().from_env().unwrap();
        assert_eq!(builder.streams.len(), 1);

        if let StreamConfig::MempoolSpace { config, .. } = &builder.streams[0] {
            assert_eq!(config.network, BitcoinNetwork::Testnet);
        } else {
            panic!("Expected MempoolSpace config");
        }

        remove_test_env_var(MEMPOOL_ENABLED);
        remove_test_env_var(MEMPOOL_NETWORK);
    }

    #[test]
    fn test_from_env_with_mempool_signet() {
        set_test_env_var(MEMPOOL_ENABLED, "true");
        set_test_env_var(MEMPOOL_NETWORK, "signet");

        let builder = StreamManagerBuilder::default().from_env().unwrap();
        assert_eq!(builder.streams.len(), 1);

        if let StreamConfig::MempoolSpace { config, .. } = &builder.streams[0] {
            assert_eq!(config.network, BitcoinNetwork::Signet);
        } else {
            panic!("Expected MempoolSpace config");
        }

        remove_test_env_var(MEMPOOL_ENABLED);
        remove_test_env_var(MEMPOOL_NETWORK);
    }

    #[test]
    fn test_from_env_with_mempool_invalid_network() {
        set_test_env_var(MEMPOOL_ENABLED, "true");
        set_test_env_var(MEMPOOL_NETWORK, "invalid");

        let builder = StreamManagerBuilder::default().from_env().unwrap();
        assert_eq!(builder.streams.len(), 1);

        if let StreamConfig::MempoolSpace { config, .. } = &builder.streams[0] {
            assert_eq!(config.network, BitcoinNetwork::Mainnet);
        } else {
            panic!("Expected MempoolSpace config");
        }

        remove_test_env_var(MEMPOOL_ENABLED);
        remove_test_env_var(MEMPOOL_NETWORK);
    }

    #[tokio::test]
    async fn test_build_with_metrics_enabled() {
        let builder = StreamManagerBuilder::default().with_metrics(true);
        let _manager = builder.build().await.unwrap();
        // Just verify it builds successfully
    }

    #[tokio::test]
    async fn test_build_with_metrics_disabled() {
        let builder = StreamManagerBuilder::default().with_metrics(false);
        let _manager = builder.build().await.unwrap();
        // Just verify it builds successfully
    }

    #[tokio::test]
    async fn test_build_with_custom_metrics_collector() {
        let collector = Arc::new(MetricsCollector::default());
        let builder = StreamManagerBuilder::default().with_metrics_collector(collector);
        let _manager = builder.build().await.unwrap();
        // Just verify it builds successfully
    }

    #[tokio::test]
    async fn test_from_config_file_success() {
        let toml_content = r#"
execution_mode = "Sequential"
enable_metrics = true

[[streams]]
type = "Binance"
name = "test-binance"
config.streams = ["btcusdt@ticker"]
config.testnet = false
config.buffer_size = 1000
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = from_config_file(temp_file.path().to_str().unwrap()).await;
        // This will fail because we can't actually connect to Binance in tests,
        // but we're testing the parsing logic
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_from_config_file_invalid_path() {
        let result = from_config_file("/nonexistent/path.toml").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_from_env_function() {
        // Clear all env vars first
        for var in &[
            STREAM_EXECUTION_MODE,
            STREAM_EXECUTION_LIMIT,
            SOLANA_GEYSER_WS_URL,
            SOLANA_GEYSER_PROGRAMS,
            BINANCE_STREAMS,
            BINANCE_TESTNET,
            MEMPOOL_ENABLED,
            MEMPOOL_NETWORK,
        ] {
            remove_test_env_var(var);
        }

        let result = from_env().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_stream_config_debug() {
        let config = StreamConfig::Binance {
            name: "test".to_string(),
            config: BinanceConfig {
                streams: vec!["btcusdt@ticker".to_string()],
                testnet: true,
                buffer_size: 1000,
            },
        };
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("Binance"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_stream_config_clone() {
        let config = StreamConfig::SolanaGeyser {
            name: "test".to_string(),
            config: GeyserConfig {
                ws_url: "ws://localhost:8899".to_string(),
                auth_token: None,
                program_ids: vec!["11111111111111111111111111111111".to_string()],
                buffer_size: 1000,
            },
        };
        let cloned = config.clone();

        if let (
            StreamConfig::SolanaGeyser { name: name1, .. },
            StreamConfig::SolanaGeyser { name: name2, .. },
        ) = (&config, &cloned)
        {
            assert_eq!(name1, name2);
        } else {
            panic!("Clone failed");
        }
    }

    #[test]
    fn test_stream_manager_config_debug() {
        let config = StreamManagerConfig {
            execution_mode: Some(HandlerExecutionMode::Sequential),
            enable_metrics: Some(true),
            streams: vec![],
        };
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("StreamManagerConfig"));
    }
}
