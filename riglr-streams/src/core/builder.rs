use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::core::{HandlerExecutionMode, MetricsCollector, Stream, StreamManager};
use crate::evm::websocket::{ChainId, EvmStreamConfig, EvmWebSocketStream};
use crate::external::binance::{BinanceConfig, BinanceStream};
use crate::external::mempool::{BitcoinNetwork, MempoolConfig, MempoolSpaceStream};
use crate::solana::geyser::{GeyserConfig, SolanaGeyserStream};

/// Configuration for a stream
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum StreamConfig {
    SolanaGeyser {
        name: String,
        config: GeyserConfig,
    },
    EvmWebSocket {
        name: String,
        config: EvmStreamConfig,
    },
    Binance {
        name: String,
        config: BinanceConfig,
    },
    MempoolSpace {
        name: String,
        config: MempoolConfig,
    },
}

/// Builder for StreamManager with configuration loading
pub struct StreamManagerBuilder {
    execution_mode: HandlerExecutionMode,
    streams: Vec<StreamConfig>,
    enable_metrics: bool,
    metrics_collector: Option<Arc<MetricsCollector>>,
}

impl StreamManagerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            execution_mode: HandlerExecutionMode::default(),
            streams: Vec::new(),
            enable_metrics: true,
            metrics_collector: None,
        }
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
        if let Ok(mode_str) = std::env::var("STREAM_EXECUTION_MODE") {
            self.execution_mode = match mode_str.as_str() {
                "sequential" => HandlerExecutionMode::Sequential,
                "concurrent" => HandlerExecutionMode::Concurrent,
                "bounded" => {
                    let limit = std::env::var("STREAM_EXECUTION_LIMIT")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(10);
                    HandlerExecutionMode::ConcurrentBounded(limit)
                }
                _ => self.execution_mode,
            };
        }

        // Load Solana Geyser configuration
        if let Ok(ws_url) = std::env::var("SOLANA_GEYSER_WS_URL") {
            let program_ids: Vec<String> = std::env::var("SOLANA_GEYSER_PROGRAMS")
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
                        contract_addresses: Vec::new(),
                        buffer_size: 10000,
                    },
                });
            }
        }

        // Load Binance configuration
        if let Ok(streams) = std::env::var("BINANCE_STREAMS") {
            let streams: Vec<String> = streams.split(',').map(String::from).collect();
            let testnet = std::env::var("BINANCE_TESTNET")
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
        if std::env::var("MEMPOOL_ENABLED").is_ok() {
            let network = match std::env::var("MEMPOOL_NETWORK").as_deref() {
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
                .or_else(|| Some(Arc::new(MetricsCollector::new())))
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
        Self::new()
    }
}

/// Configuration structure for loading from TOML
#[derive(Debug, Deserialize)]
pub struct StreamManagerConfig {
    pub execution_mode: Option<HandlerExecutionMode>,
    pub enable_metrics: Option<bool>,
    pub streams: Vec<StreamConfig>,
}

/// Helper to load and build a StreamManager from a TOML file
pub async fn from_config_file(path: &str) -> Result<StreamManager, Box<dyn std::error::Error>> {
    StreamManagerBuilder::new()
        .from_toml_file(path)
        .await?
        .build()
        .await
}

/// Helper to load and build a StreamManager from environment variables
pub async fn from_env() -> Result<StreamManager, Box<dyn std::error::Error>> {
    StreamManagerBuilder::new().from_env()?.build().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_builder_basic() {
        let manager = StreamManagerBuilder::new()
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

        let _manager = StreamManagerBuilder::new()
            .add_stream(config)
            .with_metrics(false)
            .build()
            .await;

        // Note: This will fail to connect in test environment but validates the builder
    }
}
