//! Database configuration

use crate::{ConfigError, ConfigResult};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::Validate;

lazy_static! {
    static ref REDIS_URL_REGEX: Regex = Regex::new(r"^rediss?://").unwrap();
    static ref NEO4J_URL_REGEX: Regex = Regex::new(r"^(neo4j(\+s)?|bolt(\+s)?)://").unwrap();
    static ref HTTP_URL_REGEX: Regex = Regex::new(r"^https?://").unwrap();
    static ref POSTGRES_URL_REGEX: Regex = Regex::new(r"^postgres(ql)?://").unwrap();
}

/// Validate Redis URL field
fn validate_redis_url_field(url: &str) -> Result<(), validator::ValidationError> {
    if !REDIS_URL_REGEX.is_match(url) {
        return Err(validator::ValidationError::new(
            "REDIS_URL must start with redis:// or rediss://",
        ));
    }
    if url::Url::parse(url).is_err() {
        return Err(validator::ValidationError::new("Invalid Redis URL format"));
    }
    Ok(())
}

/// Database configuration
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct DatabaseConfig {
    /// Redis connection URL
    #[validate(custom(function = "validate_redis_url_field"))]
    pub redis_url: String,

    /// Neo4j connection URL (optional, for graph memory)
    #[serde(default)]
    pub neo4j_url: Option<String>,

    /// Neo4j username (if not in URL)
    #[serde(default)]
    pub neo4j_username: Option<String>,

    /// Neo4j password (if not in URL)
    #[serde(default)]
    pub neo4j_password: Option<String>,

    /// ClickHouse URL (optional, for analytics)
    #[serde(default)]
    pub clickhouse_url: Option<String>,

    /// ClickHouse database name
    #[serde(default = "default_clickhouse_db")]
    #[validate(length(min = 1))]
    pub clickhouse_database: String,

    /// PostgreSQL URL (optional, for relational data)
    #[serde(default)]
    pub postgres_url: Option<String>,

    /// Connection pool settings
    #[serde(flatten)]
    #[validate(nested)]
    pub pool: PoolConfig,
}

/// Database connection pool configuration
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    #[validate(range(min = 1, max = 1000))]
    pub max_connections: u32,

    /// Minimum number of connections to maintain
    #[serde(default = "default_min_connections")]
    #[validate(range(min = 0, max = 100))]
    pub min_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    #[validate(range(min = 1, max = 300))]
    pub connection_timeout_secs: u64,

    /// Idle timeout in seconds
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,

    /// Maximum connection lifetime in seconds
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime_secs: u64,
}

/// Validates a Redis URL format and scheme
pub fn validate_redis_url(url: &str) -> ConfigResult<()> {
    if !REDIS_URL_REGEX.is_match(url) {
        return Err(ConfigError::validation(
            "REDIS_URL must start with redis:// or rediss://",
        ));
    }
    if url::Url::parse(url).is_err() {
        return Err(ConfigError::validation("Invalid Redis URL format"));
    }
    Ok(())
}

/// Validates a Neo4j URL format and scheme
pub fn validate_neo4j_url(url: &str) -> ConfigResult<()> {
    if !NEO4J_URL_REGEX.is_match(url) {
        return Err(ConfigError::validation(
            "NEO4J_URL must start with neo4j://, neo4j+s://, bolt://, or bolt+s://",
        ));
    }
    if url::Url::parse(url).is_err() {
        return Err(ConfigError::validation("Invalid Neo4j URL format"));
    }
    Ok(())
}

/// Validates a ClickHouse URL format and scheme
pub fn validate_clickhouse_url(url: &str) -> ConfigResult<()> {
    if !HTTP_URL_REGEX.is_match(url) {
        return Err(ConfigError::validation(
            "CLICKHOUSE_URL must start with http:// or https://",
        ));
    }
    if url::Url::parse(url).is_err() {
        return Err(ConfigError::validation("Invalid ClickHouse URL format"));
    }
    Ok(())
}

/// Validates a PostgreSQL URL format and scheme
pub fn validate_postgres_url(url: &str) -> ConfigResult<()> {
    if !POSTGRES_URL_REGEX.is_match(url) {
        return Err(ConfigError::validation(
            "POSTGRES_URL must start with postgres:// or postgresql://",
        ));
    }
    if url::Url::parse(url).is_err() {
        return Err(ConfigError::validation("Invalid PostgreSQL URL format"));
    }
    Ok(())
}

impl DatabaseConfig {
    /// Validates all database configuration settings
    ///
    /// This method validates:
    /// - Redis URL format and connectivity
    /// - Neo4j URL format (if provided)
    /// - ClickHouse URL format (if provided)
    /// - PostgreSQL URL format (if provided)
    /// - Connection pool configuration
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if any validation fails
    pub fn validate_config(&self) -> ConfigResult<()> {
        // Use validator crate for field validation
        Validate::validate(self)
            .map_err(|e| ConfigError::validation(format!("Validation failed: {}", e)))?;

        // Additional custom validation for URLs
        validate_redis_url(&self.redis_url)?;

        if let Some(ref neo4j_url) = self.neo4j_url {
            validate_neo4j_url(neo4j_url)?;
        }

        if let Some(ref clickhouse_url) = self.clickhouse_url {
            validate_clickhouse_url(clickhouse_url)?;
        }

        if let Some(ref postgres_url) = self.postgres_url {
            validate_postgres_url(postgres_url)?;
        }

        self.pool.validate_config()?;

        Ok(())
    }
}

impl PoolConfig {
    /// Validates connection pool configuration settings
    ///
    /// This method validates:
    /// - Maximum connections is greater than 0
    /// - Minimum connections doesn't exceed maximum connections
    /// - Connection timeout is greater than 0
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if any validation fails
    pub fn validate_config(&self) -> ConfigResult<()> {
        // Use validator crate for field validation
        Validate::validate(self)
            .map_err(|e| ConfigError::validation(format!("Validation failed: {}", e)))?;

        // Additional custom validation
        if self.min_connections > self.max_connections {
            return Err(ConfigError::validation(
                "min_connections cannot be greater than max_connections",
            ));
        }

        Ok(())
    }
}

// Default value functions
fn default_clickhouse_db() -> String {
    "riglr".to_string()
}
fn default_max_connections() -> u32 {
    10
}
fn default_min_connections() -> u32 {
    2
}
fn default_connection_timeout() -> u64 {
    30
}
fn default_idle_timeout() -> u64 {
    600
}
fn default_max_lifetime() -> u64 {
    3600
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            neo4j_url: None,
            neo4j_username: None,
            neo4j_password: None,
            clickhouse_url: None,
            clickhouse_database: default_clickhouse_db(),
            postgres_url: None,
            pool: PoolConfig::default(),
        }
    }
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connection_timeout_secs: default_connection_timeout(),
            idle_timeout_secs: default_idle_timeout(),
            max_lifetime_secs: default_max_lifetime(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test default value functions
    #[test]
    fn test_default_clickhouse_db_should_return_riglr() {
        assert_eq!(default_clickhouse_db(), "riglr");
    }

    #[test]
    fn test_default_max_connections_should_return_10() {
        assert_eq!(default_max_connections(), 10);
    }

    #[test]
    fn test_default_min_connections_should_return_2() {
        assert_eq!(default_min_connections(), 2);
    }

    #[test]
    fn test_default_connection_timeout_should_return_30() {
        assert_eq!(default_connection_timeout(), 30);
    }

    #[test]
    fn test_default_idle_timeout_should_return_600() {
        assert_eq!(default_idle_timeout(), 600);
    }

    #[test]
    fn test_default_max_lifetime_should_return_3600() {
        assert_eq!(default_max_lifetime(), 3600);
    }

    // Test Default implementations
    #[test]
    fn test_pool_config_default_should_use_default_values() {
        let pool = PoolConfig::default();
        assert_eq!(pool.max_connections, 10);
        assert_eq!(pool.min_connections, 2);
        assert_eq!(pool.connection_timeout_secs, 30);
        assert_eq!(pool.idle_timeout_secs, 600);
        assert_eq!(pool.max_lifetime_secs, 3600);
    }

    #[test]
    fn test_database_config_default_should_use_default_values() {
        let config = DatabaseConfig::default();
        assert_eq!(config.redis_url, "redis://localhost:6379");
        assert_eq!(config.neo4j_url, None);
        assert_eq!(config.neo4j_username, None);
        assert_eq!(config.neo4j_password, None);
        assert_eq!(config.clickhouse_url, None);
        assert_eq!(config.clickhouse_database, "riglr");
        assert_eq!(config.postgres_url, None);
        assert_eq!(config.pool.max_connections, 10);
    }

    // Test PoolConfig validation - Happy Path
    #[test]
    fn test_pool_config_validate_when_valid_should_return_ok() {
        let pool = PoolConfig {
            max_connections: 10,
            min_connections: 2,
            connection_timeout_secs: 30,
            idle_timeout_secs: 600,
            max_lifetime_secs: 3600,
        };
        assert!(pool.validate_config().is_ok());
    }

    #[test]
    fn test_pool_config_validate_when_min_equals_max_should_return_ok() {
        let pool = PoolConfig {
            max_connections: 5,
            min_connections: 5,
            connection_timeout_secs: 30,
            idle_timeout_secs: 600,
            max_lifetime_secs: 3600,
        };
        assert!(pool.validate_config().is_ok());
    }

    // Test PoolConfig validation - Error Paths
    #[test]
    fn test_pool_config_validate_when_max_connections_zero_should_return_err() {
        let pool = PoolConfig {
            max_connections: 0,
            min_connections: 0,
            connection_timeout_secs: 30,
            idle_timeout_secs: 600,
            max_lifetime_secs: 3600,
        };
        let result = pool.validate_config();
        assert!(result.is_err());
        // The validator crate generates different error messages
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Validation failed") || err_msg.contains("max_connections"));
    }

    #[test]
    fn test_pool_config_validate_when_min_greater_than_max_should_return_err() {
        let pool = PoolConfig {
            max_connections: 5,
            min_connections: 10,
            connection_timeout_secs: 30,
            idle_timeout_secs: 600,
            max_lifetime_secs: 3600,
        };
        let result = pool.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("min_connections cannot be greater than max_connections"));
    }

    #[test]
    fn test_pool_config_validate_when_connection_timeout_zero_should_return_err() {
        let pool = PoolConfig {
            max_connections: 10,
            min_connections: 2,
            connection_timeout_secs: 0,
            idle_timeout_secs: 600,
            max_lifetime_secs: 3600,
        };
        let result = pool.validate_config();
        assert!(result.is_err());
        // The validator crate generates different error messages
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Validation failed") || err_msg.contains("connection_timeout_secs")
        );
    }

    // Test Redis URL validation - Happy Path
    #[test]
    fn test_validate_redis_url_when_redis_scheme_should_return_ok() {
        assert!(validate_redis_url("redis://localhost:6379").is_ok());
    }

    #[test]
    fn test_validate_redis_url_when_rediss_scheme_should_return_ok() {
        assert!(validate_redis_url("rediss://localhost:6380").is_ok());
    }

    #[test]
    fn test_validate_redis_url_when_complex_url_should_return_ok() {
        assert!(validate_redis_url("redis://user:pass@localhost:6379/0").is_ok());
    }

    // Test Redis URL validation - Error Paths
    #[test]
    fn test_validate_redis_url_when_invalid_scheme_should_return_err() {
        let result = validate_redis_url("http://localhost:6379");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("REDIS_URL must start with redis:// or rediss://"));
    }

    #[test]
    fn test_validate_redis_url_when_empty_should_return_err() {
        let result = validate_redis_url("");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("REDIS_URL must start with redis:// or rediss://"));
    }

    #[test]
    fn test_validate_redis_url_when_malformed_url_should_return_err() {
        let result = validate_redis_url("redis://[invalid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid Redis URL"));
    }

    // Test Neo4j URL validation - Happy Path
    #[test]
    fn test_validate_neo4j_url_when_neo4j_scheme_should_return_ok() {
        assert!(validate_neo4j_url("neo4j://localhost:7687").is_ok());
    }

    #[test]
    fn test_validate_neo4j_url_when_neo4j_secure_scheme_should_return_ok() {
        assert!(validate_neo4j_url("neo4j+s://localhost:7687").is_ok());
    }

    #[test]
    fn test_validate_neo4j_url_when_bolt_scheme_should_return_ok() {
        assert!(validate_neo4j_url("bolt://localhost:7687").is_ok());
    }

    #[test]
    fn test_validate_neo4j_url_when_bolt_secure_scheme_should_return_ok() {
        assert!(validate_neo4j_url("bolt+s://localhost:7687").is_ok());
    }

    // Test Neo4j URL validation - Error Paths
    #[test]
    fn test_validate_neo4j_url_when_invalid_scheme_should_return_err() {
        let result = validate_neo4j_url("http://localhost:7687");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("NEO4J_URL must start with neo4j://"));
    }

    #[test]
    fn test_validate_neo4j_url_when_malformed_url_should_return_err() {
        let result = validate_neo4j_url("neo4j://[invalid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid Neo4j URL"));
    }

    // Test ClickHouse URL validation - Happy Path
    #[test]
    fn test_validate_clickhouse_url_when_http_scheme_should_return_ok() {
        assert!(validate_clickhouse_url("http://localhost:8123").is_ok());
    }

    #[test]
    fn test_validate_clickhouse_url_when_https_scheme_should_return_ok() {
        assert!(validate_clickhouse_url("https://localhost:8443").is_ok());
    }

    // Test ClickHouse URL validation - Error Paths
    #[test]
    fn test_validate_clickhouse_url_when_invalid_scheme_should_return_err() {
        let result = validate_clickhouse_url("tcp://localhost:8123");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("CLICKHOUSE_URL must start with http:// or https://"));
    }

    #[test]
    fn test_validate_clickhouse_url_when_malformed_url_should_return_err() {
        let result = validate_clickhouse_url("http://[invalid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid ClickHouse URL"));
    }

    // Test PostgreSQL URL validation - Happy Path
    #[test]
    fn test_validate_postgres_url_when_postgres_scheme_should_return_ok() {
        assert!(validate_postgres_url("postgres://localhost:5432/db").is_ok());
    }

    #[test]
    fn test_validate_postgres_url_when_postgresql_scheme_should_return_ok() {
        assert!(validate_postgres_url("postgresql://localhost:5432/db").is_ok());
    }

    // Test PostgreSQL URL validation - Error Paths
    #[test]
    fn test_validate_postgres_url_when_invalid_scheme_should_return_err() {
        let result = validate_postgres_url("mysql://localhost:5432/db");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("POSTGRES_URL must start with postgres:// or postgresql://"));
    }

    #[test]
    fn test_validate_postgres_url_when_malformed_url_should_return_err() {
        let result = validate_postgres_url("postgres://[invalid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid PostgreSQL URL"));
    }

    // Test DatabaseConfig validate - Happy Path
    #[test]
    fn test_database_config_validate_when_minimal_config_should_return_ok() {
        let config = DatabaseConfig {
            redis_url: "redis://localhost:6379".to_string(),
            neo4j_url: None,
            neo4j_username: None,
            neo4j_password: None,
            clickhouse_url: None,
            clickhouse_database: "riglr".to_string(),
            postgres_url: None,
            pool: PoolConfig::default(),
        };
        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_database_config_validate_when_all_urls_provided_should_return_ok() {
        let config = DatabaseConfig {
            redis_url: "redis://localhost:6379".to_string(),
            neo4j_url: Some("neo4j://localhost:7687".to_string()),
            neo4j_username: Some("neo4j".to_string()),
            neo4j_password: Some("password".to_string()),
            clickhouse_url: Some("http://localhost:8123".to_string()),
            clickhouse_database: "riglr".to_string(),
            postgres_url: Some("postgres://localhost:5432/db".to_string()),
            pool: PoolConfig::default(),
        };
        assert!(config.validate_config().is_ok());
    }

    // Test DatabaseConfig validate - Error Paths
    #[test]
    fn test_database_config_validate_when_invalid_redis_url_should_return_err() {
        let config = DatabaseConfig {
            redis_url: "http://localhost:6379".to_string(),
            ..Default::default()
        };
        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("REDIS_URL must start with redis:// or rediss://"));
    }

    #[test]
    fn test_database_config_validate_when_invalid_neo4j_url_should_return_err() {
        let config = DatabaseConfig {
            redis_url: "redis://localhost:6379".to_string(),
            neo4j_url: Some("http://localhost:7687".to_string()),
            ..Default::default()
        };
        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("NEO4J_URL must start with neo4j://"));
    }

    #[test]
    fn test_database_config_validate_when_invalid_clickhouse_url_should_return_err() {
        let config = DatabaseConfig {
            redis_url: "redis://localhost:6379".to_string(),
            clickhouse_url: Some("tcp://localhost:8123".to_string()),
            ..Default::default()
        };
        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("CLICKHOUSE_URL must start with http:// or https://"));
    }

    #[test]
    fn test_database_config_validate_when_invalid_postgres_url_should_return_err() {
        let config = DatabaseConfig {
            redis_url: "redis://localhost:6379".to_string(),
            postgres_url: Some("mysql://localhost:5432/db".to_string()),
            ..Default::default()
        };
        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("POSTGRES_URL must start with postgres:// or postgresql://"));
    }

    #[test]
    fn test_database_config_validate_when_invalid_pool_config_should_return_err() {
        let config = DatabaseConfig {
            redis_url: "redis://localhost:6379".to_string(),
            pool: PoolConfig {
                max_connections: 0,
                min_connections: 0,
                connection_timeout_secs: 30,
                idle_timeout_secs: 600,
                max_lifetime_secs: 3600,
            },
            ..Default::default()
        };
        let result = config.validate_config();
        assert!(result.is_err());
        // The validator crate generates different error messages
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Validation failed") || err_msg.contains("max_connections"));
    }

    // Test edge cases for URL validation
    #[test]
    fn test_validate_redis_url_when_redis_prefix_in_middle_should_return_err() {
        let result = validate_redis_url("http://redis://localhost:6379");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("REDIS_URL must start with redis:// or rediss://"));
    }

    #[test]
    fn test_validate_neo4j_url_when_neo4j_prefix_in_middle_should_return_err() {
        let result = validate_neo4j_url("http://neo4j://localhost:7687");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("NEO4J_URL must start with neo4j://"));
    }

    #[test]
    fn test_validate_clickhouse_url_when_http_prefix_in_middle_should_return_err() {
        let result = validate_clickhouse_url("tcp://http://localhost:8123");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("CLICKHOUSE_URL must start with http:// or https://"));
    }

    #[test]
    fn test_validate_postgres_url_when_postgres_prefix_in_middle_should_return_err() {
        let result = validate_postgres_url("mysql://postgres://localhost:5432/db");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("POSTGRES_URL must start with postgres:// or postgresql://"));
    }

    // Test with edge case values for pool config
    #[test]
    fn test_pool_config_validate_when_very_large_values_should_return_ok() {
        let pool = PoolConfig {
            max_connections: 1000,        // Maximum allowed by validation
            min_connections: 100,         // Maximum allowed by validation
            connection_timeout_secs: 300, // Maximum allowed by validation
            idle_timeout_secs: u64::MAX,  // No validation limit
            max_lifetime_secs: u64::MAX,  // No validation limit
        };
        assert!(pool.validate_config().is_ok());
    }

    #[test]
    fn test_pool_config_validate_when_min_zero_max_one_should_return_ok() {
        let pool = PoolConfig {
            max_connections: 1,
            min_connections: 0,
            connection_timeout_secs: 1,
            idle_timeout_secs: 0,
            max_lifetime_secs: 0,
        };
        assert!(pool.validate_config().is_ok());
    }
}
