//! # RIGLR Configuration
//!
//! Unified, hierarchical configuration management for RIGLR applications.
//!
//! ## Features
//!
//! - **Single source of truth**: All configuration in one place
//! - **Environment-based**: Supports dev, staging, and production environments
//! - **Convention over configuration**: RPC_URL_{CHAIN_ID} pattern for dynamic chain support
//! - **Fail-fast validation**: Catches configuration errors at startup
//! - **Hierarchical structure**: Organized into logical sections
//! - **Type-safe**: Strongly typed configuration with serde
//!
//! ## Usage
//!
//! ```rust,no_run
//! use riglr_config::Config;
//!
//! // Load configuration from environment (fail-fast)
//! let config = Config::from_env();
//!
//! // Access configuration values
//! println!("Redis URL: {}", config.database.redis_url);
//! println!("Environment: {:?}", config.app.environment);
//!
//! // Get RPC URL for a specific chain
//! if let Some(rpc_url) = config.network.get_rpc_url(1) {
//!     println!("Ethereum RPC: {}", rpc_url);
//! }
//! ```

mod app;
mod database;
mod environment;
mod error;
mod features;
mod network;
mod providers;
mod validation;

pub use app::{AppConfig, Environment, RetryConfig};
pub use database::DatabaseConfig;
pub use environment::EnvironmentSource;
pub use error::{ConfigError, ConfigResult};
pub use features::{Feature, FeaturesConfig};
pub use network::{ChainConfig, ChainContract, NetworkConfig};
pub use providers::{AiProvider, BlockchainProvider, DataProvider, ProvidersConfig};
pub use validation::Validator;

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Global configuration instance
static CONFIG: OnceCell<Arc<Config>> = OnceCell::new();

/// Main configuration structure that aggregates all subsystems
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Application-level configuration
    #[serde(flatten)]
    pub app: AppConfig,

    /// Database connections
    #[serde(flatten)]
    pub database: DatabaseConfig,

    /// Network and blockchain configuration
    #[serde(flatten)]
    pub network: NetworkConfig,

    /// External API providers
    #[serde(flatten)]
    pub providers: ProvidersConfig,

    /// Feature flags
    #[serde(flatten)]
    pub features: FeaturesConfig,
}

impl Config {
    /// Load configuration from environment variables (fail-fast)
    ///
    /// This will:
    /// 1. Load .env file if present
    /// 2. Parse environment variables
    /// 3. Apply convention-based patterns (RPC_URL_{CHAIN_ID})
    /// 4. Load chains.toml if specified
    /// 5. Validate all configuration
    /// 6. Store globally for access via Config::global()
    pub fn from_env() -> Arc<Self> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        // Build configuration from environment
        let mut config = match envy::from_env::<Config>() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("❌ FATAL: Failed to load configuration from environment:");
                eprintln!("   {}", e);
                eprintln!("   See .env.example for required variables");
                std::process::exit(1);
            }
        };

        // Apply dynamic patterns
        config.network.extract_rpc_urls();

        // Load chain contracts from TOML if specified
        if let Err(e) = config.network.load_chain_contracts() {
            tracing::warn!("Failed to load chain contracts: {}", e);
        }

        // Validate configuration
        if let Err(e) = config.validate() {
            eprintln!("❌ Configuration validation failed:");
            eprintln!("   {}", e);
            std::process::exit(1);
        }

        tracing::info!("✅ Configuration loaded and validated successfully");

        // Store globally
        let arc_config = Arc::new(config);
        CONFIG.set(arc_config.clone()).ok();

        arc_config
    }

    /// Get the global configuration instance
    ///
    /// Panics if configuration hasn't been loaded via from_env()
    pub fn global() -> Arc<Self> {
        CONFIG
            .get()
            .expect("Configuration not initialized. Call Config::from_env() first")
            .clone()
    }

    /// Try to get the global configuration instance
    pub fn try_global() -> Option<Arc<Self>> {
        CONFIG.get().cloned()
    }

    /// Validate the entire configuration
    pub fn validate(&self) -> ConfigResult<()> {
        self.app.validate()?;
        self.database.validate()?;
        self.network.validate()?;
        self.providers.validate()?;
        self.features.validate()?;

        // Cross-validation
        self.validate_cross_dependencies()?;

        Ok(())
    }

    /// Validate cross-dependencies between configuration sections
    fn validate_cross_dependencies(&self) -> ConfigResult<()> {
        // Production environment checks
        if self.app.environment == Environment::Production {
            // Ensure not using localhost in production
            if self.database.redis_url.contains("localhost") {
                return Err(ConfigError::validation(
                    "Production cannot use localhost for Redis",
                ));
            }

            // Ensure not using testnet in production
            if self.app.use_testnet {
                return Err(ConfigError::validation("Production cannot use testnet"));
            }

            // Ensure critical features are configured
            if self.features.enable_trading && self.providers.alchemy_api_key.is_none() {
                tracing::warn!("Trading enabled in production without Alchemy API key");
            }
        }

        // Feature dependencies
        if self.features.enable_graph_memory && self.database.neo4j_url.is_none() {
            return Err(ConfigError::validation(
                "Graph memory feature requires NEO4J_URL to be configured",
            ));
        }

        if self.features.enable_social_monitoring && self.providers.twitter_bearer_token.is_none() {
            tracing::warn!("Social monitoring enabled without Twitter bearer token");
        }

        Ok(())
    }

    /// Create a builder for constructing configuration programmatically
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

/// Builder for constructing configuration programmatically
#[derive(Default)]
pub struct ConfigBuilder {
    app: AppConfig,
    database: DatabaseConfig,
    network: NetworkConfig,
    providers: ProvidersConfig,
    features: FeaturesConfig,
}

impl ConfigBuilder {
    /// Create a new configuration builder with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set application configuration
    pub fn app(mut self, config: AppConfig) -> Self {
        self.app = config;
        self
    }

    /// Set database configuration
    pub fn database(mut self, config: DatabaseConfig) -> Self {
        self.database = config;
        self
    }

    /// Set network configuration
    pub fn network(mut self, config: NetworkConfig) -> Self {
        self.network = config;
        self
    }

    /// Set providers configuration
    pub fn providers(mut self, config: ProvidersConfig) -> Self {
        self.providers = config;
        self
    }

    /// Set features configuration
    pub fn features(mut self, config: FeaturesConfig) -> Self {
        self.features = config;
        self
    }

    /// Build the configuration
    pub fn build(self) -> ConfigResult<Config> {
        let config = Config {
            app: self.app,
            database: self.database,
            network: self.network,
            providers: self.providers,
            features: self.features,
        };

        config.validate()?;
        Ok(config)
    }
}

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        AppConfig, ChainConfig, Config, ConfigBuilder, ConfigError, ConfigResult, DatabaseConfig,
        Environment, Feature, FeaturesConfig, NetworkConfig, ProvidersConfig, RetryConfig,
    };
}
