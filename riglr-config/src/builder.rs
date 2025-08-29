//! Configuration builder for programmatic config construction
//!
//! Provides a fluent API for building configuration objects with validation.

use crate::{
    AppConfig, Config, ConfigResult, DatabaseConfig, FeaturesConfig, LogLevel, NetworkConfig,
    ProvidersConfig,
};

/// Builder for creating Config instances programmatically
///
/// This builder provides a fluent API for constructing configuration objects
/// piece by piece. The builder validates the configuration when `build()` is called.
///
/// # Examples
///
/// ```rust
/// use riglr_config::{ConfigBuilder, AppConfig, Environment};
///
/// let config = ConfigBuilder::default()
///     .app(AppConfig {
///         environment: Environment::Development,
///         log_level: LogLevel::Debug,
///         ..Default::default()
///     })
///     .build()
///     .expect("Failed to build config");
/// ```
#[derive(Default)]
pub struct ConfigBuilder {
    app: AppConfig,
    database: DatabaseConfig,
    network: NetworkConfig,
    providers: ProvidersConfig,
    features: FeaturesConfig,
}

impl ConfigBuilder {
    /// Create a new ConfigBuilder with default values
    ///
    /// This is equivalent to `ConfigBuilder::default()` but provides a more explicit API
    /// for builder pattern initialization.
    pub fn new() -> Self {
        Self {
            app: AppConfig::default(),
            database: DatabaseConfig::default(),
            network: NetworkConfig::default(),
            providers: ProvidersConfig::default(),
            features: FeaturesConfig::default(),
        }
    }

    // ==== Bulk Setters ====

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

    // ==== AppConfig Individual Setters ====

    /// Set the server port
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::new()
    ///     .port(8080)
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(config.app.port, 8080);
    /// ```
    pub fn port(mut self, port: u16) -> Self {
        self.app.port = port;
        self
    }

    /// Set the log level
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::new()
    ///     .log_level(LogLevel::Warn)
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(config.app.log_level, LogLevel::Warn);
    /// ```
    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.app.log_level = level;
        self
    }

    /// Set whether to use testnet
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::new()
    ///     .use_testnet(true)
    ///     .build()
    ///     .unwrap();
    /// assert!(config.app.use_testnet);
    /// ```
    pub fn use_testnet(mut self, use_testnet: bool) -> Self {
        self.app.use_testnet = use_testnet;
        self
    }

    // ==== DatabaseConfig Individual Setters ====

    /// Set the Redis URL
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::new()
    ///     .redis_url("redis://localhost:6379".to_string())
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(config.database.redis_url, "redis://localhost:6379");
    /// ```
    pub fn redis_url(mut self, url: String) -> Self {
        self.database.redis_url = url;
        self
    }

    /// Set the Neo4j URL (optional)
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::new()
    ///     .neo4j_url(Some("bolt://localhost:7687".to_string()))
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(config.database.neo4j_url, Some("bolt://localhost:7687".to_string()));
    /// ```
    pub fn neo4j_url(mut self, url: Option<String>) -> Self {
        self.database.neo4j_url = url;
        self
    }

    // ==== NetworkConfig Individual Setters ====

    /// Set the Solana RPC URL
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::new()
    ///     .solana_rpc_url("https://api.mainnet-beta.solana.com".to_string())
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(config.network.solana_rpc_url, "https://api.mainnet-beta.solana.com");
    /// ```
    pub fn solana_rpc_url(mut self, url: String) -> Self {
        self.network.solana_rpc_url = url;
        self
    }

    /// Set the default chain ID
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::new()
    ///     .default_chain_id(137) // Polygon mainnet
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(config.network.default_chain_id, 137);
    /// ```
    pub fn default_chain_id(mut self, id: u64) -> Self {
        self.network.default_chain_id = id;
        self
    }

    // ==== ProvidersConfig Individual Setters ====

    /// Set the Anthropic API key
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::new()
    ///     .anthropic_api_key(Some("sk-ant-...".to_string()))
    ///     .build()
    ///     .unwrap();
    /// assert!(config.providers.anthropic_api_key.is_some());
    /// ```
    pub fn anthropic_api_key(mut self, key: Option<String>) -> Self {
        self.providers.anthropic_api_key = key;
        self
    }

    /// Set the OpenAI API key
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::new()
    ///     .openai_api_key(Some("sk-...".to_string()))
    ///     .build()
    ///     .unwrap();
    /// assert!(config.providers.openai_api_key.is_some());
    /// ```
    pub fn openai_api_key(mut self, key: Option<String>) -> Self {
        self.providers.openai_api_key = key;
        self
    }

    // ==== FeaturesConfig Individual Setters ====

    /// Set whether trading is enabled
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::new()
    ///     .enable_trading(false)
    ///     .build()
    ///     .unwrap();
    /// assert!(!config.features.enable_trading);
    /// ```
    pub fn enable_trading(mut self, enabled: bool) -> Self {
        self.features.enable_trading = enabled;
        self
    }

    /// Set whether graph memory is enabled
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::new()
    ///     .neo4j_url(Some("bolt://localhost:7687".to_string())) // Required for graph memory
    ///     .enable_graph_memory(true)
    ///     .build()
    ///     .unwrap();
    /// assert!(config.features.enable_graph_memory);
    /// ```
    pub fn enable_graph_memory(mut self, enabled: bool) -> Self {
        self.features.enable_graph_memory = enabled;
        self
    }

    /// Build the configuration
    ///
    /// This validates all configuration and returns an error if any validation fails.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Environment;

    #[test]
    fn test_builder_default() {
        let config = ConfigBuilder::default().build().unwrap();
        assert_eq!(config.app.environment, Environment::Development);
    }

    #[test]
    fn test_builder_with_app_config() {
        // Use development environment to avoid validation failures
        let config = ConfigBuilder::default()
            .app(AppConfig {
                environment: Environment::Development,
                log_level: LogLevel::Warn,
                ..Default::default()
            })
            .build()
            .unwrap();

        assert_eq!(config.app.environment, Environment::Development);
        assert_eq!(config.app.log_level, LogLevel::Warn);
    }

    #[test]
    fn test_builder_chaining() {
        let config = ConfigBuilder::default().build().unwrap();

        assert_eq!(config.app.environment, Environment::Development);
    }

    #[test]
    fn test_builder_individual_setters() {
        let config = ConfigBuilder::default()
            // AppConfig setters
            .port(8080)
            .log_level(LogLevel::Warn)
            .use_testnet(true)
            // DatabaseConfig setters
            .redis_url("redis://localhost:6379".to_string())
            .neo4j_url(Some("bolt://localhost:7687".to_string()))
            // NetworkConfig setters
            .solana_rpc_url("https://api.mainnet-beta.solana.com".to_string())
            .default_chain_id(137)
            // ProvidersConfig setters
            .anthropic_api_key(Some("test_anthropic_key".to_string()))
            .openai_api_key(Some("test_openai_key".to_string()))
            // FeaturesConfig setters
            .enable_trading(false)
            .enable_graph_memory(true)
            .build()
            .unwrap();

        // Verify AppConfig
        assert_eq!(config.app.port, 8080);
        assert_eq!(config.app.log_level, LogLevel::Warn);
        assert!(config.app.use_testnet);

        // Verify DatabaseConfig
        assert_eq!(config.database.redis_url, "redis://localhost:6379");
        assert_eq!(
            config.database.neo4j_url,
            Some("bolt://localhost:7687".to_string())
        );

        // Verify NetworkConfig
        assert_eq!(
            config.network.solana_rpc_url,
            "https://api.mainnet-beta.solana.com"
        );
        assert_eq!(config.network.default_chain_id, 137);

        // Verify ProvidersConfig
        assert_eq!(
            config.providers.anthropic_api_key,
            Some("test_anthropic_key".to_string())
        );
        assert_eq!(
            config.providers.openai_api_key,
            Some("test_openai_key".to_string())
        );

        // Verify FeaturesConfig
        assert!(!config.features.enable_trading);
        assert!(config.features.enable_graph_memory);
    }

    #[test]
    fn test_individual_setters_override_defaults() {
        // Test that individual setters properly override default values
        let config = ConfigBuilder::default().port(9000).build().unwrap();

        assert_eq!(config.app.port, 9000);
        // Other fields should retain default values
        assert_eq!(config.app.environment, Environment::Development);
    }

    #[test]
    fn test_mixed_bulk_and_individual_setters() {
        // Test that individual setters can be mixed with bulk setters
        let mut app_config = AppConfig::default();
        app_config.environment = Environment::Development; // Use development to avoid validation issues

        let config = ConfigBuilder::default()
            .app(app_config)
            .port(3333) // This should override the port from the bulk setter
            .redis_url("redis://custom:6379".to_string())
            .build()
            .unwrap();

        assert_eq!(config.app.port, 3333);
        assert_eq!(config.app.environment, Environment::Development);
        assert_eq!(config.database.redis_url, "redis://custom:6379");
    }
}
