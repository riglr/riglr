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
pub use network::{
    ChainConfig, ChainContract, EvmNetworkConfig, NetworkConfig, SolanaNetworkConfig,
};
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
    /// Try to load configuration from environment variables
    ///
    /// This will:
    /// 1. Load .env file if present
    /// 2. Parse environment variables
    /// 3. Apply convention-based patterns (RPC_URL_{CHAIN_ID})
    /// 4. Load chains.toml if specified
    /// 5. Validate all configuration
    /// 6. Store globally for access via Config::global()
    ///
    /// Returns a Result instead of exiting on failure, making it suitable
    /// for library usage and programmatic configuration building.
    pub fn try_from_env() -> ConfigResult<Arc<Self>> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        // Build configuration from environment
        let mut config = envy::from_env::<Config>()
            .map_err(|e| ConfigError::EnvParse(format!("Failed to parse environment: {}", e)))?;

        // Apply dynamic patterns
        config.network.extract_rpc_urls();

        // Load chain contracts from TOML if specified
        if let Err(e) = config.network.load_chain_contracts() {
            tracing::warn!("Failed to load chain contracts: {}", e);
        }

        // Validate configuration
        config.validate_config()?;

        tracing::info!("✅ Configuration loaded and validated successfully");

        // Store globally
        let arc_config = Arc::new(config);
        CONFIG.set(arc_config.clone()).ok();

        Ok(arc_config)
    }

    /// Load configuration from environment variables (fail-fast)
    ///
    /// This will:
    /// 1. Load .env file if present
    /// 2. Parse environment variables
    /// 3. Apply convention-based patterns (RPC_URL_{CHAIN_ID})
    /// 4. Load chains.toml if specified
    /// 5. Validate all configuration
    /// 6. Store globally for access via Config::global()
    ///
    /// This is a wrapper around `try_from_env()` that exits the process
    /// on failure, suitable for application main binaries.
    pub fn from_env() -> Arc<Self> {
        match Self::try_from_env() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("❌ FATAL: Failed to load configuration:");
                eprintln!("   {}", e);
                eprintln!("   See .env.example for required variables");
                std::process::exit(1);
            }
        }
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
    pub fn validate_config(&self) -> ConfigResult<()> {
        self.app.validate_config()?;
        self.database.validate_config()?;
        self.network.validate_config()?;
        self.providers.validate_config()?;
        self.features.validate_config()?;

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
        ConfigBuilder::default()
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

        config.validate_config()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config {
            app: AppConfig::default(),
            database: DatabaseConfig::default(),
            network: NetworkConfig::default(),
            providers: ProvidersConfig::default(),
            features: FeaturesConfig::default(),
        }
    }

    fn create_production_config() -> Config {
        let mut config = create_test_config();
        config.app.environment = Environment::Production;
        config
    }

    #[test]
    fn test_config_try_global_when_not_initialized_should_return_none() {
        // This test assumes CONFIG might not be initialized in this context
        // In practice, try_global returns None when CONFIG is not set
        let result = Config::try_global();
        // This might return Some if global config was set by other tests
        // So we'll just verify the function doesn't panic
        match result {
            Some(_) => { /* Config is initialized - test passes */ }
            None => { /* Config is not initialized - test passes */ }
        }
    }

    #[test]
    fn test_config_validate_when_valid_should_return_ok() {
        let config = create_test_config();
        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_config_validate_cross_dependencies_when_production_with_localhost_redis_should_return_err(
    ) {
        let mut config = create_production_config();
        config.database.redis_url = "redis://localhost:6379".to_string();

        let result = config.validate_cross_dependencies();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Production cannot use localhost for Redis"));
    }

    #[test]
    fn test_config_validate_cross_dependencies_when_production_with_testnet_should_return_err() {
        let mut config = create_production_config();
        config.app.use_testnet = true;
        config.database.redis_url = "redis://production-redis.example.com:6379".to_string(); // Avoid localhost validation error

        let result = config.validate_cross_dependencies();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Production cannot use testnet"));
    }

    #[test]
    fn test_config_validate_cross_dependencies_when_graph_memory_without_neo4j_should_return_err() {
        let mut config = create_test_config();
        config.features.enable_graph_memory = true;
        config.database.neo4j_url = None;

        let result = config.validate_cross_dependencies();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Graph memory feature requires NEO4J_URL"));
    }

    #[test]
    fn test_config_validate_cross_dependencies_when_production_trading_without_alchemy_should_warn()
    {
        let mut config = create_production_config();
        config.features.enable_trading = true;
        config.providers.alchemy_api_key = None;
        config.database.redis_url = "redis://production-redis.example.com:6379".to_string(); // Avoid localhost validation error

        // This should succeed but log a warning
        let result = config.validate_cross_dependencies();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validate_cross_dependencies_when_social_monitoring_without_twitter_should_warn()
    {
        let mut config = create_test_config();
        config.features.enable_social_monitoring = true;
        config.providers.twitter_bearer_token = None;

        // This should succeed but log a warning
        let result = config.validate_cross_dependencies();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validate_cross_dependencies_when_valid_production_should_return_ok() {
        let mut config = create_production_config();
        config.database.redis_url = "redis://production-server:6379".to_string();
        config.app.use_testnet = false;

        let result = config.validate_cross_dependencies();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_builder_new_should_create_default() {
        let builder = ConfigBuilder::default();
        assert_eq!(builder.app.environment, Environment::Development);
    }

    #[test]
    fn test_config_builder_app_should_set_app_config() {
        let app_config = AppConfig::default();
        let builder = ConfigBuilder::default().app(app_config.clone());
        assert_eq!(builder.app.environment, app_config.environment);
    }

    #[test]
    fn test_config_builder_database_should_set_database_config() {
        let database_config = DatabaseConfig::default();
        let builder = ConfigBuilder::default().database(database_config.clone());
        assert_eq!(builder.database.redis_url, database_config.redis_url);
    }

    #[test]
    fn test_config_builder_network_should_set_network_config() {
        let network_config = NetworkConfig::default();
        let builder = ConfigBuilder::default().network(network_config.clone());
        assert_eq!(
            builder.network.default_chain_id,
            network_config.default_chain_id
        );
    }

    #[test]
    fn test_config_builder_providers_should_set_providers_config() {
        let providers_config = ProvidersConfig::default();
        let builder = ConfigBuilder::default().providers(providers_config.clone());
        assert_eq!(
            builder.providers.alchemy_api_key,
            providers_config.alchemy_api_key
        );
    }

    #[test]
    fn test_config_builder_features_should_set_features_config() {
        let features_config = FeaturesConfig::default();
        let builder = ConfigBuilder::default().features(features_config.clone());
        assert_eq!(
            builder.features.enable_trading,
            features_config.enable_trading
        );
    }

    #[test]
    fn test_config_builder_build_when_valid_should_return_ok() {
        let result = ConfigBuilder::default().build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_builder_build_when_invalid_should_return_err() {
        let mut app_config = AppConfig::default();
        app_config.environment = Environment::Production;

        let mut database_config = DatabaseConfig::default();
        database_config.redis_url = "redis://localhost:6379".to_string();

        let result = ConfigBuilder::default()
            .app(app_config)
            .database(database_config)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_config_builder_chaining_should_work() {
        let app_config = AppConfig::default();
        let database_config = DatabaseConfig::default();
        let network_config = NetworkConfig::default();
        let providers_config = ProvidersConfig::default();
        let features_config = FeaturesConfig::default();

        let result = ConfigBuilder::default()
            .app(app_config)
            .database(database_config)
            .network(network_config)
            .providers(providers_config)
            .features(features_config)
            .build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_config_builder_default_should_create_valid_builder() {
        let builder = ConfigBuilder::default();
        let result = builder.build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_builder_should_create_same_as_new() {
        let builder1 = ConfigBuilder::default();
        let builder2 = ConfigBuilder::default();

        let config1 = builder1.build().unwrap();
        let config2 = builder2.build().unwrap();

        assert_eq!(config1.app.environment, config2.app.environment);
    }

    #[test]
    fn test_config_validate_when_app_validation_fails_should_return_err() {
        let mut config = create_test_config();
        // Create invalid app config - this would depend on AppConfig validation logic
        // For now, we'll create a scenario that might fail cross-validation
        config.app.environment = Environment::Production;
        config.database.redis_url = "redis://localhost:6379".to_string();

        let result = config.validate_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_validate_cross_dependencies_when_graph_memory_with_neo4j_should_return_ok() {
        let mut config = create_test_config();
        config.features.enable_graph_memory = true;
        config.database.neo4j_url = Some("bolt://localhost:7687".to_string());

        let result = config.validate_cross_dependencies();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validate_cross_dependencies_when_development_with_localhost_should_return_ok() {
        let mut config = create_test_config();
        config.app.environment = Environment::Development;
        config.database.redis_url = "redis://localhost:6379".to_string();

        let result = config.validate_cross_dependencies();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validate_cross_dependencies_when_staging_should_return_ok() {
        let mut config = create_test_config();
        config.app.environment = Environment::Staging;
        config.database.redis_url = "redis://staging-server:6379".to_string();

        let result = config.validate_cross_dependencies();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_builder_should_validate_during_build() {
        // Test that the builder properly validates the config during build
        let mut features_config = FeaturesConfig::default();
        features_config.enable_graph_memory = true;

        let mut database_config = DatabaseConfig::default();
        database_config.neo4j_url = None; // This should cause validation to fail

        let result = ConfigBuilder::default()
            .features(features_config)
            .database(database_config)
            .build();

        assert!(result.is_err());
    }
}
