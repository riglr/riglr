//! Database configuration

use serde::{Deserialize, Serialize};
use crate::{ConfigError, ConfigResult};
use url::Url;

/// Database configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    /// Redis connection URL
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
    pub clickhouse_database: String,
    
    /// PostgreSQL URL (optional, for relational data)
    #[serde(default)]
    pub postgres_url: Option<String>,
    
    /// Connection pool settings
    #[serde(flatten)]
    pub pool: PoolConfig,
}

/// Database connection pool configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    
    /// Minimum number of connections to maintain
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    
    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_secs: u64,
    
    /// Idle timeout in seconds
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,
    
    /// Maximum connection lifetime in seconds
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime_secs: u64,
}

impl DatabaseConfig {
    pub fn validate(&self) -> ConfigResult<()> {
        // Validate Redis URL
        self.validate_redis_url()?;
        
        // Validate Neo4j URL if provided
        if let Some(ref url) = self.neo4j_url {
            self.validate_neo4j_url(url)?;
        }
        
        // Validate ClickHouse URL if provided
        if let Some(ref url) = self.clickhouse_url {
            self.validate_clickhouse_url(url)?;
        }
        
        // Validate PostgreSQL URL if provided
        if let Some(ref url) = self.postgres_url {
            self.validate_postgres_url(url)?;
        }
        
        // Validate pool configuration
        self.pool.validate()?;
        
        Ok(())
    }
    
    fn validate_redis_url(&self) -> ConfigResult<()> {
        if !self.redis_url.starts_with("redis://") && 
           !self.redis_url.starts_with("rediss://") {
            return Err(ConfigError::validation(
                "REDIS_URL must start with redis:// or rediss://"
            ));
        }
        
        // Try to parse as URL
        Url::parse(&self.redis_url)
            .map_err(|e| ConfigError::validation(
                format!("Invalid Redis URL: {}", e)
            ))?;
        
        Ok(())
    }
    
    fn validate_neo4j_url(&self, url: &str) -> ConfigResult<()> {
        if !url.starts_with("neo4j://") && 
           !url.starts_with("neo4j+s://") &&
           !url.starts_with("bolt://") &&
           !url.starts_with("bolt+s://") {
            return Err(ConfigError::validation(
                "NEO4J_URL must start with neo4j://, neo4j+s://, bolt://, or bolt+s://"
            ));
        }
        
        Url::parse(url)
            .map_err(|e| ConfigError::validation(
                format!("Invalid Neo4j URL: {}", e)
            ))?;
        
        Ok(())
    }
    
    fn validate_clickhouse_url(&self, url: &str) -> ConfigResult<()> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ConfigError::validation(
                "CLICKHOUSE_URL must start with http:// or https://"
            ));
        }
        
        Url::parse(url)
            .map_err(|e| ConfigError::validation(
                format!("Invalid ClickHouse URL: {}", e)
            ))?;
        
        Ok(())
    }
    
    fn validate_postgres_url(&self, url: &str) -> ConfigResult<()> {
        if !url.starts_with("postgres://") && 
           !url.starts_with("postgresql://") {
            return Err(ConfigError::validation(
                "POSTGRES_URL must start with postgres:// or postgresql://"
            ));
        }
        
        Url::parse(url)
            .map_err(|e| ConfigError::validation(
                format!("Invalid PostgreSQL URL: {}", e)
            ))?;
        
        Ok(())
    }
}

impl PoolConfig {
    pub fn validate(&self) -> ConfigResult<()> {
        if self.max_connections == 0 {
            return Err(ConfigError::validation(
                "max_connections must be greater than 0"
            ));
        }
        
        if self.min_connections > self.max_connections {
            return Err(ConfigError::validation(
                "min_connections cannot be greater than max_connections"
            ));
        }
        
        if self.connection_timeout_secs == 0 {
            return Err(ConfigError::validation(
                "connection_timeout_secs must be greater than 0"
            ));
        }
        
        Ok(())
    }
}

// Default value functions
fn default_clickhouse_db() -> String { "riglr".to_string() }
fn default_max_connections() -> u32 { 10 }
fn default_min_connections() -> u32 { 2 }
fn default_connection_timeout() -> u64 { 30 }
fn default_idle_timeout() -> u64 { 600 }
fn default_max_lifetime() -> u64 { 3600 }

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