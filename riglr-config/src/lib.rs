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
//! if let Some(rpc_url) = config.network.get_rpc_url("1") {
//!     println!("Ethereum RPC: {}", rpc_url);
//! }
//! ```

mod app;
mod builder;
mod database;
mod environment;
mod error;
mod features;
mod network;
mod providers;

pub use app::{AppConfig, Environment, LogLevel, RetryConfig};
pub use builder::ConfigBuilder;
pub use database::DatabaseConfig;
pub use environment::EnvironmentSource;
pub use error::{ConfigError, ConfigResult};
pub use features::{Feature, FeaturesConfig};
pub use network::{
    AddressValidator, ChainConfig, ChainContract, EvmNetworkConfig, NetworkConfig,
    SolanaNetworkConfig,
};
pub use providers::{AiProvider, BlockchainProvider, DataProvider, ProvidersConfig};
// Validator trait removed - configs now directly implement validate_config()

#[cfg(test)]
pub mod test_helpers;

use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

        Ok(Arc::new(config))
    }

    /// Load configuration from environment variables (fail-fast)
    ///
    /// This will:
    /// 1. Load .env file if present
    /// 2. Parse environment variables
    /// 3. Apply convention-based patterns (RPC_URL_{CHAIN_ID})
    /// 4. Load chains.toml if specified
    /// 5. Validate all configuration
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

    /// Validate the entire configuration
    pub fn validate_config(&self) -> ConfigResult<()> {
        self.app.validate_config()?;
        self.database.validate_config()?;
        self.network.validate_config(None)?;
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
                return Err(ConfigError::validation(
                    "Trading is enabled in production, but ALCHEMY_API_KEY is not set",
                ));
            }
        }

        // Feature dependencies
        if self.features.enable_graph_memory && self.database.neo4j_url.is_none() {
            return Err(ConfigError::validation(
                "Graph memory feature requires NEO4J_URL to be configured",
            ));
        }

        if self.features.enable_bridging && self.providers.lifi_api_key.is_none() {
            return Err(ConfigError::validation(
                "Bridging feature requires LIFI_API_KEY to be configured",
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

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        AddressValidator, AppConfig, ChainConfig, Config, ConfigBuilder, ConfigError, ConfigResult,
        DatabaseConfig, Environment, Feature, FeaturesConfig, NetworkConfig, ProvidersConfig,
        RetryConfig,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        let mut features = FeaturesConfig::default();
        // Disable features that require external dependencies for basic tests
        features.enable_bridging = false; // Requires lifi_api_key
        features.enable_graph_memory = false; // Requires neo4j_url

        Config {
            app: AppConfig::default(),
            database: DatabaseConfig::default(),
            network: NetworkConfig::default(),
            providers: ProvidersConfig::default(),
            features,
        }
    }

    fn create_production_config() -> Config {
        let mut config = create_test_config();
        config.app.environment = Environment::Production;
        config
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
    fn test_prod_config_missing_alchemy_key_fails() {
        let mut config = create_production_config();
        config.features.enable_trading = true;
        config.providers.alchemy_api_key = None;
        config.database.redis_url = "redis://production-redis.example.com:6379".to_string(); // Avoid localhost validation error

        // This should now fail in production
        let result = config.validate_cross_dependencies();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Trading is enabled in production, but ALCHEMY_API_KEY is not set"));
    }

    #[test]
    fn test_dev_config_missing_alchemy_key_succeeds() {
        let mut config = create_test_config();
        config.app.environment = Environment::Development;
        config.features.enable_trading = true;
        config.providers.alchemy_api_key = None;

        // This should succeed in development (no error)
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
        // Either provide an Alchemy key or disable trading for production validation
        config.providers.alchemy_api_key = Some("test-alchemy-key".to_string());

        let result = config.validate_cross_dependencies();
        assert!(result.is_ok(), "Validation failed: {:?}", result);
    }

    #[test]
    fn test_config_builder_build_when_valid_should_return_ok() {
        let result = ConfigBuilder::new().build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_builder_build_when_invalid_should_return_err() {
        let mut app_config = AppConfig::default();
        app_config.environment = Environment::Production;

        let mut database_config = DatabaseConfig::default();
        database_config.redis_url = "redis://localhost:6379".to_string();

        let result = ConfigBuilder::new()
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

        let result = ConfigBuilder::new()
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
        let builder = ConfigBuilder::new();
        let result = builder.build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_builder_should_create_same_as_new() {
        let builder1 = ConfigBuilder::new();
        let builder2 = ConfigBuilder::new();

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

        let result = ConfigBuilder::new()
            .features(features_config)
            .database(database_config)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_config_validate_cross_dependencies_when_bridging_without_lifi_key_should_return_err() {
        let mut config = create_test_config();
        config.features.enable_bridging = true;
        config.providers.lifi_api_key = None;

        let result = config.validate_cross_dependencies();
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("LIFI_API_KEY"));
        }
    }

    #[test]
    fn test_config_validate_cross_dependencies_when_bridging_with_lifi_key_should_return_ok() {
        let mut config = create_test_config();
        config.features.enable_bridging = true;
        config.providers.lifi_api_key = Some("test-lifi-key".to_string());

        let result = config.validate_cross_dependencies();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validate_cross_dependencies_when_social_monitoring_without_twitter_token_should_return_ok(
    ) {
        let mut config = create_test_config();
        config.features.enable_social_monitoring = true;
        config.providers.twitter_bearer_token = None;

        // Should return Ok but log a warning (warning is not testable here)
        let result = config.validate_cross_dependencies();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validate_cross_dependencies_when_social_monitoring_with_twitter_token_should_return_ok(
    ) {
        let mut config = create_test_config();
        config.features.enable_social_monitoring = true;
        config.providers.twitter_bearer_token = Some("test-twitter-token".to_string());

        let result = config.validate_cross_dependencies();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validate_cross_dependencies_when_bridging_disabled_without_lifi_key_should_return_ok(
    ) {
        let mut config = create_test_config();
        config.features.enable_bridging = false;
        config.providers.lifi_api_key = None;

        // Bridging is disabled, so missing key should be OK
        let result = config.validate_cross_dependencies();
        assert!(result.is_ok());
    }
}
