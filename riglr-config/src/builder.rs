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
        Self::default()
    }

    /// Merge another configuration into this builder
    ///
    /// Non-default values from the other config will override current values.
    /// This allows for layered configuration where later sources take precedence.
    #[must_use]
    pub fn merge(mut self, other: Config) -> Self {
        self.merge_app_config(other.app);
        self.merge_database_config(other.database);
        self.merge_network_config(other.network);
        self.merge_providers_config(other.providers);
        self.merge_features_config(other.features);
        self
    }

    /// Merge AppConfig, taking non-default values from other
    fn merge_app_config(&mut self, other: AppConfig) {
        let default_app = AppConfig::default();
        
        // Use non-default values from other, falling back to current values
        if other.port != default_app.port {
            self.app.port = other.port;
        }
        if other.environment != default_app.environment {
            self.app.environment = other.environment;
        }
        if other.log_level != default_app.log_level {
            self.app.log_level = other.log_level;
        }
        if other.use_testnet != default_app.use_testnet {
            self.app.use_testnet = other.use_testnet;
        }
        // Merge nested configs - for simplicity, replace if they have any non-default values
        if other.transaction.max_gas_price_gwei != default_app.transaction.max_gas_price_gwei
            || other.transaction.priority_fee_gwei != default_app.transaction.priority_fee_gwei
            || other.transaction.slippage_tolerance_percent
                != default_app.transaction.slippage_tolerance_percent
            || other.transaction.deadline_seconds != default_app.transaction.deadline_seconds
        {
            self.app.transaction = other.transaction;
        }
        if other.retry.max_retry_attempts != default_app.retry.max_retry_attempts
            || other.retry.retry_delay_ms != default_app.retry.retry_delay_ms
            || other.retry.retry_backoff_multiplier != default_app.retry.retry_backoff_multiplier
            || other.retry.max_retry_delay_ms != default_app.retry.max_retry_delay_ms
        {
            self.app.retry = other.retry;
        }
    }

    /// Merge DatabaseConfig, taking non-default values from other
    fn merge_database_config(&mut self, other: DatabaseConfig) {
        let default_database = DatabaseConfig::default();
        
        if other.redis_url != default_database.redis_url {
            self.database.redis_url = other.redis_url;
        }
        if other.neo4j_url != default_database.neo4j_url {
            self.database.neo4j_url = other.neo4j_url;
        }
    }

    /// Merge NetworkConfig, taking non-default values from other
    fn merge_network_config(&mut self, other: NetworkConfig) {
        let default_network = NetworkConfig::default();
        if other.solana_rpc_url != default_network.solana_rpc_url {
            self.network.solana_rpc_url = other.solana_rpc_url;
        }
        if other.solana_ws_url != default_network.solana_ws_url {
            self.network.solana_ws_url = other.solana_ws_url;
        }
        if other.default_chain_id != default_network.default_chain_id {
            self.network.default_chain_id = other.default_chain_id;
        }
        // Merge RPC URLs and chains - non-empty collections replace defaults
        if !other.rpc_urls.is_empty() {
            self.network.rpc_urls = other.rpc_urls;
        }
        if !other.chains.is_empty() {
            self.network.chains = other.chains;
        }
        // Merge timeouts if different from default
        if other.timeouts.rpc_timeout_secs != default_network.timeouts.rpc_timeout_secs
            || other.timeouts.ws_timeout_secs != default_network.timeouts.ws_timeout_secs
            || other.timeouts.http_timeout_secs != default_network.timeouts.http_timeout_secs
        {
            self.network.timeouts = other.timeouts;
        }
    }

    /// Merge ProvidersConfig, taking non-default values from other
    fn merge_providers_config(&mut self, other: ProvidersConfig) {
        if other.anthropic_api_key.is_some() {
            self.providers.anthropic_api_key = other.anthropic_api_key;
        }
        if other.openai_api_key.is_some() {
            self.providers.openai_api_key = other.openai_api_key;
        }
        if other.alchemy_api_key.is_some() {
            self.providers.alchemy_api_key = other.alchemy_api_key;
        }
        if other.lifi_api_key.is_some() {
            self.providers.lifi_api_key = other.lifi_api_key;
        }
        if other.twitter_bearer_token.is_some() {
            self.providers.twitter_bearer_token = other.twitter_bearer_token;
        }
        if other.dexscreener_api_key.is_some() {
            self.providers.dexscreener_api_key = other.dexscreener_api_key;
        }
    }

    /// Merge FeaturesConfig, taking non-default values from other
    fn merge_features_config(&mut self, other: FeaturesConfig) {
        let default_features = FeaturesConfig::default();
        if other.enable_trading != default_features.enable_trading {
            self.features.enable_trading = other.enable_trading;
        }
        if other.enable_graph_memory != default_features.enable_graph_memory {
            self.features.enable_graph_memory = other.enable_graph_memory;
        }
        if other.enable_bridging != default_features.enable_bridging {
            self.features.enable_bridging = other.enable_bridging;
        }
        if other.enable_social_monitoring != default_features.enable_social_monitoring {
            self.features.enable_social_monitoring = other.enable_social_monitoring;
        }
        if other.enable_streaming != default_features.enable_streaming {
            self.features.enable_streaming = other.enable_streaming;
        }
        if other.enable_webhooks != default_features.enable_webhooks {
            self.features.enable_webhooks = other.enable_webhooks;
        }
        if other.enable_analytics != default_features.enable_analytics {
            self.features.enable_analytics = other.enable_analytics;
        }
        if other.debug_mode != default_features.debug_mode {
            self.features.debug_mode = other.debug_mode;
        }
        if other.experimental != default_features.experimental {
            self.features.experimental = other.experimental;
        }
        // Merge custom feature flags
        if !other.custom.is_empty() {
            self.features.custom.extend(other.custom);
        }
    }

    // ==== Bulk Setters ====

    /// Set application configuration
    #[must_use]
    pub fn app(mut self, config: AppConfig) -> Self {
        self.app = config;
        self
    }

    /// Set database configuration
    #[must_use]
    pub fn database(mut self, config: DatabaseConfig) -> Self {
        self.database = config;
        self
    }

    /// Set network configuration
    #[must_use]
    pub fn network(mut self, config: NetworkConfig) -> Self {
        self.network = config;
        self
    }

    /// Set providers configuration
    #[must_use]
    pub fn providers(mut self, config: ProvidersConfig) -> Self {
        self.providers = config;
        self
    }

    /// Set features configuration
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn openai_api_key(mut self, key: Option<String>) -> Self {
        self.providers.openai_api_key = key;
        self
    }

    // ==== Convenience Environment Setters ====

    /// Configure for development environment
    ///
    /// Sets up common development defaults:
    /// - Environment::Development
    /// - LogLevel::Debug
    /// - use_testnet = true
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::{ConfigBuilder, Environment, LogLevel};
    ///
    /// let config = ConfigBuilder::new()
    ///     .development()
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(config.app.environment, Environment::Development);
    /// assert_eq!(config.app.log_level, LogLevel::Debug);
    /// assert!(config.app.use_testnet);
    /// ```
    #[must_use]
    pub fn development(mut self) -> Self {
        use crate::{Environment, LogLevel};
        self.app.environment = Environment::Development;
        self.app.log_level = LogLevel::Debug;
        self.app.use_testnet = true;
        self
    }

    /// Configure for staging environment
    ///
    /// Sets up common staging defaults:
    /// - Environment::Staging
    /// - LogLevel::Info
    /// - use_testnet = true
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::{ConfigBuilder, Environment, LogLevel};
    ///
    /// let config = ConfigBuilder::new()
    ///     .staging()
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(config.app.environment, Environment::Staging);
    /// assert_eq!(config.app.log_level, LogLevel::Info);
    /// assert!(config.app.use_testnet);
    /// ```
    #[must_use]
    pub fn staging(mut self) -> Self {
        use crate::{Environment, LogLevel};
        self.app.environment = Environment::Staging;
        self.app.log_level = LogLevel::Info;
        self.app.use_testnet = true;
        self
    }

    /// Configure for production environment
    ///
    /// Sets up common production defaults:
    /// - Environment::Production
    /// - LogLevel::Info
    /// - use_testnet = false
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::{ConfigBuilder, Environment, LogLevel};
    ///
    /// let config = ConfigBuilder::new()
    ///     .production()
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(config.app.environment, Environment::Production);
    /// assert_eq!(config.app.log_level, LogLevel::Info);
    /// assert!(!config.app.use_testnet);
    /// ```
    #[must_use]
    pub fn production(mut self) -> Self {
        use crate::{Environment, LogLevel};
        self.app.environment = Environment::Production;
        self.app.log_level = LogLevel::Info;
        self.app.use_testnet = false;
        self
    }

    /// Configure for testnet usage
    ///
    /// Sets up testnet-specific defaults:
    /// - use_testnet = true
    /// - solana_rpc_url = devnet RPC
    /// - default_chain_id = 5 (Goerli)
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::new()
    ///     .testnet()
    ///     .build()
    ///     .unwrap();
    /// assert!(config.app.use_testnet);
    /// assert_eq!(config.network.default_chain_id, 5); // Goerli
    /// ```
    #[must_use]
    pub fn testnet(mut self) -> Self {
        self.app.use_testnet = true;
        self.network.solana_rpc_url = "https://api.devnet.solana.com".to_string();
        self.network.default_chain_id = 5; // Goerli testnet
        self
    }

    /// Configure for mainnet usage
    ///
    /// Sets up mainnet-specific defaults:
    /// - use_testnet = false
    /// - solana_rpc_url = mainnet-beta RPC
    /// - default_chain_id = 1 (Ethereum Mainnet)
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::new()
    ///     .mainnet()
    ///     .build()
    ///     .unwrap();
    /// assert!(!config.app.use_testnet);
    /// assert_eq!(config.network.default_chain_id, 1); // Ethereum Mainnet
    /// ```
    #[must_use]
    pub fn mainnet(mut self) -> Self {
        self.app.use_testnet = false;
        self.network.solana_rpc_url = "https://api.mainnet-beta.solana.com".to_string();
        self.network.default_chain_id = 1; // Ethereum mainnet
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
    #[must_use]
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
    #[must_use]
    pub fn enable_graph_memory(mut self, enabled: bool) -> Self {
        self.features.enable_graph_memory = enabled;
        self
    }

    // ==== Utility Methods ====

    /// Load configuration from a TOML file
    ///
    /// This method loads configuration from a TOML file and merges it
    /// with any existing builder settings.
    ///
    /// # Example
    /// ```rust,no_run
    /// use riglr_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::new()
    ///     .from_file("config.toml")
    ///     .unwrap()
    ///     .port(8080) // Can override file settings
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn from_file<P: AsRef<std::path::Path>>(mut self, path: P) -> ConfigResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::ConfigError::validation(format!("Failed to read config file: {}", e))
        })?;

        let file_config: Config = toml::from_str(&content).map_err(|e| {
            crate::ConfigError::validation(format!("Failed to parse TOML config: {}", e))
        })?;

        // Merge file config with current builder state (non-destructive)
        self.merge_app_config(file_config.app);
        self.merge_database_config(file_config.database);
        self.merge_network_config(file_config.network);
        self.merge_providers_config(file_config.providers);
        self.merge_features_config(file_config.features);

        Ok(self)
    }

    /// Load configuration from environment variables
    ///
    /// This method loads configuration from environment variables
    /// and merges it with any existing builder settings.
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::new()
    ///     .development()
    ///     .from_env()
    ///     .unwrap()
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn from_env(mut self) -> ConfigResult<Self> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        let env_config = envy::from_env::<Config>().map_err(|e| {
            crate::ConfigError::EnvParse(format!("Failed to parse environment: {}", e))
        })?;

        // Merge environment config with current builder state (non-destructive)
        self.merge_app_config(env_config.app);
        self.merge_database_config(env_config.database);
        self.merge_network_config(env_config.network);
        self.merge_providers_config(env_config.providers);
        self.merge_features_config(env_config.features);

        Ok(self)
    }

    /// Validate the current configuration without building
    ///
    /// This allows you to check if the configuration is valid
    /// before calling build().
    ///
    /// # Example
    /// ```rust
    /// use riglr_config::ConfigBuilder;
    ///
    /// let builder = ConfigBuilder::new()
    ///     .development();
    ///
    /// // Check if valid before building
    /// if builder.validate().is_ok() {
    ///     let config = builder.build().unwrap();
    /// }
    /// ```
    pub fn validate(&self) -> ConfigResult<()> {
        let config = Config {
            app: self.app.clone(),
            database: self.database.clone(),
            network: self.network.clone(),
            providers: self.providers.clone(),
            features: self.features.clone(),
        };
        config.validate_config()
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

    #[test]
    fn test_config_builder_development_preset() {
        let config = ConfigBuilder::new().development().build().unwrap();

        assert_eq!(config.app.environment, Environment::Development);
        assert_eq!(config.app.log_level, LogLevel::Debug);
        assert!(config.app.use_testnet);
    }

    #[test]
    fn test_config_builder_staging_preset() {
        let config = ConfigBuilder::new().staging().build().unwrap();

        assert_eq!(config.app.environment, Environment::Staging);
        assert_eq!(config.app.log_level, LogLevel::Info);
        assert!(config.app.use_testnet);
    }

    #[test]
    fn test_config_builder_production_preset() {
        // Use a proper Redis URL for production and disable trading to avoid Alchemy key requirement
        let config = ConfigBuilder::new()
            .production()
            .redis_url("redis://production-redis:6379".to_string())
            .enable_trading(false) // Disable trading to avoid Alchemy key requirement
            .build()
            .unwrap();

        assert_eq!(config.app.environment, Environment::Production);
        assert_eq!(config.app.log_level, LogLevel::Info);
        assert!(!config.app.use_testnet);
    }

    #[test]
    fn test_config_builder_testnet_preset() {
        let config = ConfigBuilder::new().testnet().build().unwrap();

        assert!(config.app.use_testnet);
        assert_eq!(config.network.default_chain_id, 5); // Goerli
        assert_eq!(
            config.network.solana_rpc_url,
            "https://api.devnet.solana.com"
        );
    }

    #[test]
    fn test_config_builder_mainnet_preset() {
        let config = ConfigBuilder::new().mainnet().build().unwrap();

        assert!(!config.app.use_testnet);
        assert_eq!(config.network.default_chain_id, 1); // Ethereum mainnet
        assert_eq!(
            config.network.solana_rpc_url,
            "https://api.mainnet-beta.solana.com"
        );
    }

    #[test]
    fn test_config_builder_validate_without_building() {
        let builder = ConfigBuilder::new().development();

        // Should be able to validate without building
        assert!(builder.validate().is_ok());

        // Should still be able to build after validation
        let config = builder.build().unwrap();
        assert_eq!(config.app.environment, Environment::Development);
    }

    #[test]
    fn test_config_builder_preset_chaining() {
        let config = ConfigBuilder::new()
            .development() // Set development defaults
            .port(9000) // Override specific values
            .build()
            .unwrap();

        assert_eq!(config.app.environment, Environment::Development);
        assert_eq!(config.app.log_level, LogLevel::Debug);
        assert!(config.app.use_testnet);
        assert_eq!(config.app.port, 9000); // Should override the default
    }

    #[test]
    fn test_config_builder_chaining_with_overrides() {
        let config = ConfigBuilder::new()
            .development() // Set development defaults
            .production() // Override with production
            .port(9000) // Override specific values
            .redis_url("redis://prod:6379".to_string()) // Avoid localhost validation error
            .enable_trading(false) // Disable trading to avoid Alchemy key requirement
            .build()
            .unwrap();

        assert_eq!(config.app.environment, Environment::Production);
        assert_eq!(config.app.log_level, LogLevel::Info);
        assert!(!config.app.use_testnet); // Production overrode development
        assert_eq!(config.app.port, 9000); // Should override the default
    }

    #[test]
    fn test_config_builder_comprehensive_layering() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary TOML config file with required fields
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config_content = r#"
# App config
port = 8081
environment = "development"
log_level = "info"
use_testnet = true

# Database config  
redis_url = "redis://file-redis:6379"

# Network config
solana_rpc_url = "https://file-solana-rpc.com"
default_chain_id = 1

# Features config
enable_trading = false
enable_graph_memory = false
enable_bridging = false
enable_social_monitoring = false
enable_streaming = false
enable_webhooks = false
enable_analytics = false
debug_mode = false
experimental = false
"#;
        temp_file
            .write_all(config_content.as_bytes())
            .expect("Failed to write to temp file");

        let temp_path = temp_file.path();

        // Test the layering without environment variables first to verify basic file loading works
        let file_only_config = ConfigBuilder::new()
            .from_file(temp_path)
            .expect("Failed to load from file")
            .build()
            .unwrap();

        // Verify file values are loaded
        assert_eq!(
            file_only_config.app.port, 8081,
            "Port should come from file"
        );
        assert_eq!(
            file_only_config.database.redis_url, "redis://file-redis:6379",
            "Redis URL should come from file"
        );

        // Now test with programmatic overrides
        let config = ConfigBuilder::new()
            .from_file(temp_path)
            .expect("Failed to load from file")
            .port(8083) // Programmatic setter should override all: port -> 8083
            .redis_url("redis://programmatic:6379".to_string()) // Override file value
            .anthropic_api_key(Some("programmatic-key".to_string())) // Set a new value not in file
            .build()
            .unwrap();

        // Verify the layering:
        // port: Default <- File (8081) <- Programmatic (8083) = 8083
        assert_eq!(
            config.app.port, 8083,
            "Port should be set by programmatic setter"
        );

        // redis_url: Default <- File (file-redis) <- Programmatic (programmatic) = programmatic
        assert_eq!(
            config.database.redis_url, "redis://programmatic:6379",
            "Redis URL should be set by programmatic setter"
        );

        // solana_rpc_url: Default <- File (file-solana-rpc) = file-solana-rpc (no override)
        assert_eq!(
            config.network.solana_rpc_url, "https://file-solana-rpc.com",
            "Solana RPC URL should be set by file"
        );

        // anthropic_api_key: Default (None) <- Programmatic (Some("key")) = Some("key")
        assert_eq!(
            config.providers.anthropic_api_key,
            Some("programmatic-key".to_string()),
            "Anthropic API key should be set by programmatic setter"
        );

        // enable_trading: Default (true) <- File (false) = false
        assert!(
            !config.features.enable_trading,
            "Trading should be disabled by file config"
        );
    }

    #[test]
    fn test_config_builder_merge_behavior() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Test that merge() preserves non-default values correctly

        // Create a temporary TOML config file with some settings
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config_content = r#"
port = 9001
redis_url = "redis://merge-test:6379"
solana_rpc_url = "https://file-rpc.com"
"#;
        temp_file
            .write_all(config_content.as_bytes())
            .expect("Failed to write to temp file");

        let temp_path = temp_file.path();

        // Build a base config with some values
        let base_config = ConfigBuilder::new()
            .port(7000) // Should be overridden by file
            .anthropic_api_key(Some("base-key".to_string())) // Should remain
            .from_file(temp_path)
            .expect("Failed to load from file")
            .build()
            .unwrap();

        // Verify that file values override base values
        assert_eq!(base_config.app.port, 9001, "Port should come from file");
        assert_eq!(
            base_config.database.redis_url, "redis://merge-test:6379",
            "Redis URL should come from file"
        );
        assert_eq!(
            base_config.network.solana_rpc_url, "https://file-rpc.com",
            "Solana RPC should come from file"
        );
        assert_eq!(
            base_config.providers.anthropic_api_key,
            Some("base-key".to_string()),
            "Anthropic key should remain from base builder"
        );
    }

    #[test]
    fn test_config_builder_layering_with_none_values() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Test that None/empty values don't override non-empty values inappropriately

        // Create a TOML file with required fields and an optional field set
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config_content = r#"
redis_url = "redis://required:6379"
solana_rpc_url = "https://api.devnet.solana.com"
neo4j_url = "bolt://file-neo4j:7687"
"#;
        temp_file
            .write_all(config_content.as_bytes())
            .expect("Failed to write to temp file");

        let temp_path = temp_file.path();

        // First test: load from file only to verify file parsing works
        let file_only_config = ConfigBuilder::new()
            .from_file(temp_path)
            .expect("Failed to load from file")
            .build()
            .unwrap();

        assert_eq!(
            file_only_config.database.neo4j_url,
            Some("bolt://file-neo4j:7687".to_string()),
            "Neo4j URL should come from file when loaded directly"
        );

        // Second test: verify layering behavior with programmatic settings
        let config_with_override = ConfigBuilder::new()
            .redis_url("redis://base:6379".to_string()) // Base value
            .from_file(temp_path)
            .expect("Failed to load from file")
            .redis_url("redis://override:6379".to_string()) // Override after file
            .build()
            .unwrap();

        // Verify programmatic override takes precedence
        assert_eq!(
            config_with_override.database.redis_url, "redis://override:6379",
            "Redis URL should be overridden by programmatic setter"
        );

        // Verify file value is preserved for other fields
        assert_eq!(
            config_with_override.database.neo4j_url,
            Some("bolt://file-neo4j:7687".to_string()),
            "Neo4j URL should still come from file"
        );
    }
}
