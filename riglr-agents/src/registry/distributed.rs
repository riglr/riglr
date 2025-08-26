//! Redis implementation of the distributed agent registry
//!
//! This module provides a Redis-backed implementation of the `DistributedAgentRegistry` trait,
//! enabling agent coordination across multiple processes or machines. The Redis backend
//! provides atomic operations, high performance, and built-in support for TTL-based
//! heartbeat monitoring.
//!
//! # Architecture
//!
//! The Redis implementation is one of potentially many distributed registry backends.
//! The abstraction through the `DistributedAgentRegistry` trait allows for future
//! implementations using other backends such as:
//! - NATS for event-driven coordination
//! - PostgreSQL for persistent storage
//! - etcd for strong consistency guarantees
//! - Consul for service mesh integration
//!
//! # Migration from Direct Usage
//!
//! If you were previously using `DistributedAgentRegistry` directly, update your code
//! to use `RedisAgentRegistry` for the concrete implementation, or better yet, use
//! the `DistributedAgentRegistry` trait for better abstraction:
//!
//! ```rust,no_run
//! // Old usage (deprecated)
//! // use riglr_agents::registry::DistributedAgentRegistry;
//!
//! // New usage (concrete type)
//! use riglr_agents::registry::RedisAgentRegistry;
//! let registry = RedisAgentRegistry::new("redis://localhost".to_string()).await?;
//!
//! // Better usage (trait abstraction)
//! use riglr_agents::registry::{DistributedAgentRegistry, RedisAgentRegistry};
//! use std::sync::Arc;
//! let registry: Arc<dyn DistributedAgentRegistry> = Arc::new(
//!     RedisAgentRegistry::new("redis://localhost".to_string()).await?
//! );
//! ```

use crate::{
    registry::{AgentRegistry, DistributedAgentRegistry, RegistryConfig},
    types::*,
    Agent, AgentError, AgentId, Result,
};
use dashmap::DashMap;
use redis::{aio::MultiplexedConnection, AsyncCommands, Client};
use serde_json;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Redis-backed distributed agent registry for multi-process coordination
///
/// This registry implementation uses Redis to:
/// - Store agent metadata that can be accessed by any process
/// - Coordinate agent discovery across distributed systems
/// - Provide atomic operations for registration/unregistration
/// - Enable load balancing and failover capabilities
///
/// # Architecture
///
/// The registry uses the following Redis data structures:
/// - HASH: `agents:{agent_id}` - Stores agent metadata
/// - SET: `agents:all` - Set of all registered agent IDs
/// - SET: `agents:capability:{capability}` - Agent IDs by capability
/// - HASH: `agents:status:{agent_id}` - Agent status information
/// - STRING: `agents:heartbeat:{agent_id}` - Agent heartbeat with TTL
///
/// # Example
///
/// ```rust,no_run
/// use riglr_agents::registry::RedisAgentRegistry;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let registry = RedisAgentRegistry::new("redis://localhost:6379".to_string())
///         .await?;
///     
///     // Use registry for agent coordination
///     Ok(())
/// }
/// ```
pub struct RedisAgentRegistry {
    /// Configuration for the registry
    config: RegistryConfig,
    /// Redis connection URL
    #[allow(dead_code)]
    redis_url: String,
    /// Redis connection
    connection: Arc<RwLock<MultiplexedConnection>>,
    /// Local cache of agents for performance
    local_agents: Arc<DashMap<AgentId, Arc<dyn Agent>>>,
}

impl RedisAgentRegistry {
    /// Create a new distributed registry with default configuration
    pub async fn new(redis_url: String) -> Result<Self> {
        let config = RegistryConfig::default();
        Self::with_config(redis_url, config).await
    }

    /// Create a new distributed registry with custom configuration
    pub async fn with_config(redis_url: String, config: RegistryConfig) -> Result<Self> {
        // Create Redis client
        let client = Client::open(redis_url.clone())
            .map_err(|e| AgentError::storage(format!("Failed to create Redis client: {}", e)))?;

        // Get multiplexed connection
        let connection = client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| AgentError::storage(format!("Failed to connect to Redis: {}", e)))?;

        info!(
            "Connected to Redis at {} for distributed agent registry",
            redis_url
        );

        let registry = Self {
            config,
            redis_url,
            connection: Arc::new(RwLock::new(connection)),
            local_agents: Arc::new(DashMap::new()),
        };

        // Start background tasks if health checks are enabled
        if registry.config.enable_health_checks {
            registry.start_background_tasks();
        }

        Ok(registry)
    }

    /// Start background maintenance tasks
    fn start_background_tasks(&self) {
        // Create a weak reference to avoid circular dependencies
        let connection = Arc::downgrade(&self.connection);
        let local_agents = Arc::downgrade(&self.local_agents);

        // Heartbeat task
        let heartbeat_ttl_secs = self.config.heartbeat_ttl.as_secs();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;

                // Try to upgrade the weak references
                let connection = match connection.upgrade() {
                    Some(conn) => conn,
                    None => break, // Registry has been dropped
                };
                let local_agents = match local_agents.upgrade() {
                    Some(agents) => agents,
                    None => break, // Registry has been dropped
                };

                // Send heartbeats for all local agents
                let mut conn = connection.write().await;
                let agent_keys: Vec<AgentId> = local_agents
                    .iter()
                    .map(|entry| entry.key().clone())
                    .collect();
                for agent_id in agent_keys {
                    let key = format!("agents:heartbeat:{}", agent_id);
                    if let Err(e) = conn.set_ex::<_, _, ()>(&key, "1", heartbeat_ttl_secs).await {
                        error!("Failed to set heartbeat for {}: {}", agent_id, e);
                    }
                }
            }
        });

        // Cleanup task
        let connection = Arc::downgrade(&self.connection);
        let maintenance_interval = self.config.maintenance_interval;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(maintenance_interval);
            loop {
                interval.tick().await;

                // Try to upgrade the weak reference
                let connection = match connection.upgrade() {
                    Some(conn) => conn,
                    None => break, // Registry has been dropped
                };

                // Clean up stale agents
                let mut conn = connection.write().await;

                // Get all agent IDs
                let agent_ids: Vec<String> = match conn.smembers("agents:all").await {
                    Ok(ids) => ids,
                    Err(e) => {
                        error!("Failed to get agent list: {}", e);
                        continue;
                    }
                };

                for agent_id_str in agent_ids {
                    let heartbeat_key = format!("agents:heartbeat:{}", agent_id_str);

                    // Check if heartbeat exists
                    let exists: bool = match conn.exists(&heartbeat_key).await {
                        Ok(exists) => exists,
                        Err(e) => {
                            error!("Failed to check heartbeat for {}: {}", agent_id_str, e);
                            continue;
                        }
                    };

                    if !exists {
                        warn!(
                            "Agent {} has stale heartbeat, removing from registry",
                            agent_id_str
                        );

                        // Remove agent from all sets and hashes
                        if let Err(e) =
                            Self::remove_agent_from_redis_internal(&mut conn, &agent_id_str).await
                        {
                            error!("Failed to remove stale agent {}: {}", agent_id_str, e);
                        }
                    }
                }
            }
        });
    }

    /// Remove an agent from all Redis data structures (internal static version for background tasks)
    async fn remove_agent_from_redis_internal(
        conn: &mut MultiplexedConnection,
        agent_id_str: &str,
    ) -> Result<()> {
        // Start a transaction
        redis::pipe()
            .atomic()
            // Remove from main set
            .cmd("SREM")
            .arg("agents:all")
            .arg(agent_id_str)
            // Delete agent metadata
            .cmd("DEL")
            .arg(format!("agents:{}", agent_id_str))
            // Delete agent status
            .cmd("DEL")
            .arg(format!("agents:status:{}", agent_id_str))
            // Delete heartbeat
            .cmd("DEL")
            .arg(format!("agents:heartbeat:{}", agent_id_str))
            .query_async::<()>(conn)
            .await
            .map_err(|e| {
                AgentError::storage(format!("Failed to remove agent from Redis: {}", e))
            })?;

        // Remove from capability sets (we need to fetch capabilities first)
        let capabilities_key = format!("agents:{}:capabilities", agent_id_str);
        let capabilities: Vec<String> = conn.smembers(&capabilities_key).await.unwrap_or_default();

        for capability in capabilities {
            let cap_key = format!("agents:capability:{}", capability);
            conn.srem::<_, _, ()>(&cap_key, agent_id_str)
                .await
                .map_err(|e| {
                    AgentError::storage(format!("Failed to remove from capability set: {}", e))
                })?;
        }

        // Delete capabilities set
        conn.del::<_, ()>(&capabilities_key)
            .await
            .map_err(|e| AgentError::storage(format!("Failed to delete capabilities: {}", e)))?;

        Ok(())
    }

    /// Remove an agent from all Redis data structures
    async fn remove_agent_from_redis(&self, agent_id: &AgentId) -> Result<()> {
        let mut conn = self.connection.write().await;
        Self::remove_agent_from_redis_internal(&mut conn, agent_id.as_str()).await
    }
}

// Manual Clone implementation needed because MultiplexedConnection doesn't implement Clone
impl Clone for RedisAgentRegistry {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            redis_url: self.redis_url.clone(),
            connection: Arc::clone(&self.connection),
            local_agents: Arc::clone(&self.local_agents),
        }
    }
}

// Manual Debug implementation needed because MultiplexedConnection doesn't implement Debug
impl std::fmt::Debug for RedisAgentRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisAgentRegistry")
            .field("config", &self.config)
            .field("redis_url", &self.redis_url)
            .field("connection", &"<redis_connection>")
            .field("local_agents", &"<local_agents>")
            .finish()
    }
}

#[async_trait::async_trait]
impl AgentRegistry for RedisAgentRegistry {
    async fn register_agent(&self, agent: Arc<dyn Agent>) -> Result<()> {
        // Check capacity limit
        if let Some(max) = self.config.max_agents {
            let count = self.agent_count().await?;
            if count >= max {
                return Err(AgentError::capacity(format!(
                    "Registry at maximum capacity ({} agents)",
                    max
                )));
            }
        }

        let agent_id = agent.id();
        let agent_id_str = agent_id.as_str();

        // Store in local cache
        self.local_agents
            .insert(agent_id.clone(), Arc::clone(&agent));

        // Store in Redis
        let mut conn = self.connection.write().await;

        // Prepare agent metadata
        let metadata = serde_json::json!({
            "id": agent_id_str,
            "capabilities": agent.capabilities(),
            "registered_at": chrono::Utc::now().to_rfc3339(),
        });

        // Use Redis transaction for atomic operations
        redis::pipe()
            .atomic()
            // Add to main agent set
            .cmd("SADD")
            .arg("agents:all")
            .arg(agent_id_str)
            // Store agent metadata
            .cmd("HSET")
            .arg(format!("agents:{}", agent_id_str))
            .arg("metadata")
            .arg(metadata.to_string())
            // Set initial heartbeat
            .cmd("SETEX")
            .arg(format!("agents:heartbeat:{}", agent_id_str))
            .arg(self.config.heartbeat_ttl.as_secs())
            .arg("1")
            .query_async::<()>(&mut *conn)
            .await
            .map_err(|e| {
                AgentError::storage(format!("Failed to register agent in Redis: {}", e))
            })?;

        // Add to capability sets
        for capability in agent.capabilities() {
            let cap_key = format!("agents:capability:{}", capability);
            conn.sadd::<_, _, ()>(&cap_key, agent_id_str)
                .await
                .map_err(|e| {
                    AgentError::storage(format!("Failed to add to capability set: {}", e))
                })?;

            // Also store in a dedicated capabilities set for the agent
            let agent_cap_key = format!("agents:{}:capabilities", agent_id_str);
            conn.sadd::<_, _, ()>(&agent_cap_key, &capability.to_string())
                .await
                .map_err(|e| {
                    AgentError::storage(format!("Failed to store agent capabilities: {}", e))
                })?;
        }

        // Create initial status
        let status = AgentStatus {
            agent_id: agent_id.clone(),
            status: AgentState::Active,
            active_tasks: 0,
            load: 0.0,
            last_heartbeat: chrono::Utc::now(),
            capabilities: agent
                .capabilities()
                .into_iter()
                .map(|cap| Capability {
                    name: cap.to_string(),
                    version: "1.0.0".to_string(),
                    parameters: HashMap::new(),
                    schema: None,
                })
                .collect(),
            metadata: HashMap::new(),
        };
        self.update_agent_status(status).await?;

        info!("Registered agent {} in distributed registry", agent_id_str);
        Ok(())
    }

    async fn unregister_agent(&self, agent_id: &AgentId) -> Result<()> {
        // Remove from local cache
        self.local_agents.remove(agent_id);

        // Remove from Redis
        self.remove_agent_from_redis(agent_id).await?;

        info!(
            "Unregistered agent {} from distributed registry",
            agent_id.as_str()
        );
        Ok(())
    }

    async fn get_agent(&self, agent_id: &AgentId) -> Result<Option<Arc<dyn Agent>>> {
        // Check local cache first
        if let Some(agent) = self.local_agents.get(agent_id) {
            return Ok(Some(Arc::clone(agent.value())));
        }

        // Agent not in local cache - check if it exists in Redis
        let mut conn = self.connection.write().await;
        let exists: bool = conn
            .sismember("agents:all", agent_id.as_str())
            .await
            .map_err(|e| AgentError::storage(format!("Failed to check agent existence: {}", e)))?;

        if exists {
            // Agent exists in Redis but not locally
            // This means it's registered on another node
            debug!(
                "Agent {} exists in Redis but not in local cache",
                agent_id.as_str()
            );
        }

        Ok(None)
    }

    async fn list_agents(&self) -> Result<Vec<Arc<dyn Agent>>> {
        // For distributed registry, we can only return locally registered agents
        // Remote agents on other nodes are not accessible as Arc<dyn Agent>
        // For distributed discovery, use list_agent_statuses() instead
        Ok(self
            .local_agents
            .iter()
            .map(|entry| Arc::clone(entry.value()))
            .collect())
    }

    async fn find_agents_by_capability(&self, capability: &str) -> Result<Vec<Arc<dyn Agent>>> {
        // For distributed registry, we filter local agents by capability
        // Remote agents are not directly accessible
        let matching_agents: Vec<Arc<dyn Agent>> = self
            .local_agents
            .iter()
            .filter(|entry| {
                let cap_type = crate::CapabilityType::from_str(capability).unwrap();
                entry.value().capabilities().contains(&cap_type)
            })
            .map(|entry| Arc::clone(entry.value()))
            .collect();

        Ok(matching_agents)
    }

    async fn get_agent_status(&self, agent_id: &AgentId) -> Result<Option<AgentStatus>> {
        let mut conn = self.connection.write().await;
        let status_key = format!("agents:status:{}", agent_id.as_str());

        let status_json: Option<String> = conn
            .get(&status_key)
            .await
            .map_err(|e| AgentError::storage(format!("Failed to get agent status: {}", e)))?;

        match status_json {
            Some(json) => {
                let status = serde_json::from_str(&json).map_err(|e| {
                    AgentError::storage(format!("Failed to parse agent status: {}", e))
                })?;
                Ok(Some(status))
            }
            None => Ok(None),
        }
    }

    async fn update_agent_status(&self, status: AgentStatus) -> Result<()> {
        let mut conn = self.connection.write().await;
        let status_key = format!("agents:status:{}", status.agent_id.as_str());

        let status_json = serde_json::to_string(&status)
            .map_err(|e| AgentError::storage(format!("Failed to serialize status: {}", e)))?;

        // Set with TTL to auto-expire stale statuses
        conn.set_ex::<_, _, ()>(&status_key, status_json, 600) // 10 minute TTL
            .await
            .map_err(|e| AgentError::storage(format!("Failed to update agent status: {}", e)))?;

        Ok(())
    }

    async fn list_agent_statuses(&self) -> Result<Vec<AgentStatus>> {
        // Use the distributed discovery method we already implemented
        self.list_agent_statuses_distributed().await
    }

    async fn find_agent_statuses_by_capability(
        &self,
        capability: &str,
    ) -> Result<Vec<AgentStatus>> {
        // Use the distributed discovery method we already implemented
        self.find_agent_statuses_by_capability_distributed(capability)
            .await
    }

    async fn agent_count(&self) -> Result<usize> {
        let mut conn = self.connection.write().await;
        let count: usize = conn
            .scard("agents:all")
            .await
            .map_err(|e| AgentError::storage(format!("Failed to count agents: {}", e)))?;

        Ok(count)
    }

    async fn is_agent_registered(&self, agent_id: &AgentId) -> Result<bool> {
        let mut conn = self.connection.write().await;
        let exists: bool = conn
            .sismember("agents:all", agent_id.as_str())
            .await
            .map_err(|e| {
                AgentError::storage(format!("Failed to check agent registration: {}", e))
            })?;

        Ok(exists)
    }

    async fn health_check(&self) -> Result<bool> {
        let mut conn = self.connection.write().await;

        // Simple Redis PING to check connectivity
        let pong: String = conn
            .ping()
            .await
            .map_err(|e| AgentError::storage(format!("Health check failed: {}", e)))?;

        Ok(pong == "PONG")
    }
}

#[async_trait::async_trait]
impl DistributedAgentRegistry for RedisAgentRegistry {
    async fn list_agent_statuses_distributed(&self) -> Result<Vec<AgentStatus>> {
        let mut conn = self.connection.write().await;

        // Get all agent IDs from the global set
        let agent_ids: Vec<String> = conn
            .smembers("agents:all")
            .await
            .map_err(|e| AgentError::storage(format!("Failed to get agent IDs: {}", e)))?;

        let mut statuses = Vec::new();

        // Fetch status for each agent
        for agent_id in agent_ids {
            let status_key = format!("agents:status:{}", agent_id);

            // Get the serialized status
            let status_json: Option<String> = conn
                .get(&status_key)
                .await
                .map_err(|e| AgentError::storage(format!("Failed to get agent status: {}", e)))?;

            if let Some(json) = status_json {
                match serde_json::from_str::<AgentStatus>(&json) {
                    Ok(status) => statuses.push(status),
                    Err(e) => {
                        warn!("Failed to deserialize status for agent {}: {}", agent_id, e);
                        // Continue with other agents even if one fails
                    }
                }
            }
        }

        Ok(statuses)
    }

    async fn find_agent_statuses_by_capability_distributed(
        &self,
        capability: &str,
    ) -> Result<Vec<AgentStatus>> {
        let mut conn = self.connection.write().await;

        // Get agent IDs with this capability
        let capability_key = format!("agents:capability:{}", capability);
        let agent_ids: Vec<String> = conn.smembers(&capability_key).await.map_err(|e| {
            AgentError::storage(format!("Failed to get agents by capability: {}", e))
        })?;

        let mut statuses = Vec::new();

        // Fetch status for each agent with this capability
        for agent_id in agent_ids {
            let status_key = format!("agents:status:{}", agent_id);

            // Get the serialized status
            let status_json: Option<String> = conn
                .get(&status_key)
                .await
                .map_err(|e| AgentError::storage(format!("Failed to get agent status: {}", e)))?;

            if let Some(json) = status_json {
                match serde_json::from_str::<AgentStatus>(&json) {
                    Ok(status) => {
                        // Double-check that the status has the capability we're looking for
                        if status.capabilities.iter().any(|cap| cap.name == capability) {
                            statuses.push(status);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to deserialize status for agent {}: {}", agent_id, e);
                        // Continue with other agents even if one fails
                    }
                }
            }
        }

        Ok(statuses)
    }
}
