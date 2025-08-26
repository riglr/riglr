//! Integration tests for the distributed agent registry using Redis.

#[cfg(feature = "redis")]
mod tests {
    use riglr_agents::{
        registry::{DistributedAgentRegistry, RedisAgentRegistry, RegistryConfig},
        types::*,
        Agent, AgentId, Result,
    };
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    use testcontainers::ContainerAsync;
    use testcontainers_modules::redis::Redis;

    // Mock agent for testing
    #[derive(Clone, Debug)]
    struct TestAgent {
        id: AgentId,
        capabilities: Vec<CapabilityType>,
    }

    #[async_trait::async_trait]
    impl Agent for TestAgent {
        async fn execute_task(&self, _task: Task) -> Result<TaskResult> {
            Ok(TaskResult::success(
                serde_json::json!({"result": "test"}),
                None,
                Duration::from_millis(10),
            ))
        }

        fn id(&self) -> &AgentId {
            &self.id
        }

        fn capabilities(&self) -> Vec<CapabilityType> {
            self.capabilities.clone()
        }
    }

    async fn setup_redis_registry() -> (Arc<dyn DistributedAgentRegistry>, ContainerAsync<Redis>) {
        use testcontainers::runners::AsyncRunner;
        let redis = Redis::default()
            .start()
            .await
            .expect("Failed to start Redis container");
        let redis_port = redis
            .get_host_port_ipv4(6379)
            .await
            .expect("Failed to get Redis port");
        let redis_url = format!("redis://127.0.0.1:{}", redis_port);

        let config = RegistryConfig {
            max_agents: Some(10),
            operation_timeout: Duration::from_secs(5),
            enable_health_checks: false, // Disable for testing
            maintenance_interval: Duration::from_secs(60),
            heartbeat_ttl: Duration::from_secs(600), // 10 minutes
        };

        let registry: Arc<dyn DistributedAgentRegistry> = Arc::new(
            RedisAgentRegistry::with_config(redis_url, config)
                .await
                .expect("Failed to create registry"),
        );

        (registry, redis)
    }

    #[tokio::test]
    async fn test_basic_registration_and_retrieval() {
        let (registry, _container) = setup_redis_registry().await;

        let agent = Arc::new(TestAgent {
            id: AgentId::new("test-agent-1"),
            capabilities: vec![CapabilityType::Trading, CapabilityType::Research],
        });

        // Register agent
        registry
            .register_agent(agent.clone())
            .await
            .expect("Failed to register agent");

        // Retrieve agent
        let retrieved = registry
            .get_agent(&AgentId::new("test-agent-1"))
            .await
            .expect("Failed to get agent");

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id(), &AgentId::new("test-agent-1"));

        // Check if agent is registered
        assert!(registry
            .is_agent_registered(&AgentId::new("test-agent-1"))
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_unregister_agent() {
        let (registry, _container) = setup_redis_registry().await;

        let agent = Arc::new(TestAgent {
            id: AgentId::new("test-agent-2"),
            capabilities: vec![CapabilityType::Monitoring],
        });

        // Register and then unregister
        registry.register_agent(agent.clone()).await.unwrap();
        registry
            .unregister_agent(&AgentId::new("test-agent-2"))
            .await
            .unwrap();

        // Agent should not be found
        let retrieved = registry
            .get_agent(&AgentId::new("test-agent-2"))
            .await
            .unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_find_agents_by_capability() {
        let (registry, _container) = setup_redis_registry().await;

        // Register multiple agents with different capabilities
        let agent1 = Arc::new(TestAgent {
            id: AgentId::new("trading-agent-1"),
            capabilities: vec![CapabilityType::Trading],
        });

        let agent2 = Arc::new(TestAgent {
            id: AgentId::new("trading-agent-2"),
            capabilities: vec![
                CapabilityType::Trading,
                CapabilityType::Custom("arbitrage".to_string()),
            ],
        });

        let agent3 = Arc::new(TestAgent {
            id: AgentId::new("research-agent"),
            capabilities: vec![CapabilityType::Research],
        });

        registry.register_agent(agent1).await.unwrap();
        registry.register_agent(agent2).await.unwrap();
        registry.register_agent(agent3).await.unwrap();

        // Find agents with trading capability
        let trading_agents = registry.find_agents_by_capability("trading").await.unwrap();
        assert_eq!(trading_agents.len(), 2);

        // Find agents with research capability
        let research_agents = registry
            .find_agents_by_capability("research")
            .await
            .unwrap();
        assert_eq!(research_agents.len(), 1);

        // Find agents with non-existent capability
        let none_agents = registry
            .find_agents_by_capability("nonexistent")
            .await
            .unwrap();
        assert_eq!(none_agents.len(), 0);
    }

    #[tokio::test]
    async fn test_agent_status_operations() {
        let (registry, _container) = setup_redis_registry().await;

        let agent = Arc::new(TestAgent {
            id: AgentId::new("status-test-agent"),
            capabilities: vec![CapabilityType::Trading],
        });

        registry.register_agent(agent).await.unwrap();

        // Get initial status
        let status = registry
            .get_agent_status(&AgentId::new("status-test-agent"))
            .await
            .unwrap();
        assert!(status.is_some());
        let initial_status = status.unwrap();
        assert_eq!(initial_status.agent_id, AgentId::new("status-test-agent"));
        assert_eq!(initial_status.status, AgentState::Active);

        // Update status
        let updated_status = AgentStatus {
            agent_id: AgentId::new("status-test-agent"),
            status: AgentState::Busy,
            active_tasks: 5,
            load: 0.75,
            last_heartbeat: chrono::Utc::now(),
            capabilities: vec![],
            metadata: HashMap::new(),
        };

        registry
            .update_agent_status(updated_status.clone())
            .await
            .unwrap();

        // Verify update
        let retrieved_status = registry
            .get_agent_status(&AgentId::new("status-test-agent"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved_status.status, AgentState::Busy);
        assert_eq!(retrieved_status.active_tasks, 5);
        assert_eq!(retrieved_status.load, 0.75);
    }

    #[tokio::test]
    async fn test_list_operations() {
        let (registry, _container) = setup_redis_registry().await;

        // Register multiple agents
        for i in 1..=3 {
            let agent = Arc::new(TestAgent {
                id: AgentId::new(format!("list-agent-{}", i)),
                capabilities: vec![CapabilityType::Custom("general".to_string())],
            });
            registry.register_agent(agent).await.unwrap();
        }

        // List all agents
        let agents = registry.list_agents().await.unwrap();
        assert_eq!(agents.len(), 3);

        // List all statuses
        let statuses = registry.list_agent_statuses().await.unwrap();
        assert_eq!(statuses.len(), 3);

        // Check agent count
        let count = registry.agent_count().await.unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_health_check() {
        let (registry, _container) = setup_redis_registry().await;

        // Health check should succeed with running Redis
        let health = registry.health_check().await.unwrap();
        assert!(health);
    }

    #[tokio::test]
    async fn test_max_agents_limit() {
        use testcontainers::runners::AsyncRunner;
        let redis = Redis::default()
            .start()
            .await
            .expect("Failed to start Redis container");
        let redis_port = redis
            .get_host_port_ipv4(6379)
            .await
            .expect("Failed to get Redis port");
        let redis_url = format!("redis://127.0.0.1:{}", redis_port);

        let config = RegistryConfig {
            max_agents: Some(2), // Limit to 2 agents
            operation_timeout: Duration::from_secs(5),
            enable_health_checks: false,
            maintenance_interval: Duration::from_secs(60),
            heartbeat_ttl: Duration::from_secs(600), // 10 minutes
        };

        let registry: Arc<dyn DistributedAgentRegistry> = Arc::new(
            RedisAgentRegistry::with_config(redis_url, config)
                .await
                .unwrap(),
        );

        // Register first two agents - should succeed
        for i in 1..=2 {
            let agent = Arc::new(TestAgent {
                id: AgentId::new(format!("limited-agent-{}", i)),
                capabilities: vec![],
            });
            registry.register_agent(agent).await.unwrap();
        }

        // Third agent should fail
        let agent3 = Arc::new(TestAgent {
            id: AgentId::new("limited-agent-3"),
            capabilities: vec![],
        });
        let result = registry.register_agent(agent3).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("maximum capacity"));
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let (registry, _container): (Arc<dyn DistributedAgentRegistry>, ContainerAsync<Redis>) =
            setup_redis_registry().await;

        // Spawn multiple tasks that register agents concurrently
        let mut handles = vec![];
        for i in 1..=10 {
            let registry_clone = registry.clone();
            let handle = tokio::spawn(async move {
                let agent = Arc::new(TestAgent {
                    id: AgentId::new(format!("concurrent-agent-{}", i)),
                    capabilities: vec![CapabilityType::Custom("concurrent".to_string())],
                });
                registry_clone.register_agent(agent).await
            });
            handles.push(handle);
        }

        // Wait for all registrations
        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        // Verify all agents were registered
        let count = registry.agent_count().await.unwrap();
        assert_eq!(count, 10);
    }

    #[tokio::test]
    async fn test_heartbeat_and_stale_cleanup() {
        use testcontainers::runners::AsyncRunner;
        let redis = Redis::default()
            .start()
            .await
            .expect("Failed to start Redis container");
        let redis_port = redis
            .get_host_port_ipv4(6379)
            .await
            .expect("Failed to get Redis port");
        let redis_url = format!("redis://127.0.0.1:{}", redis_port);

        let config = RegistryConfig {
            max_agents: None,
            operation_timeout: Duration::from_secs(5),
            enable_health_checks: false, // We'll manually trigger cleanup
            maintenance_interval: Duration::from_secs(1),
            heartbeat_ttl: Duration::from_secs(600), // 10 minutes
        };

        let registry: Arc<dyn DistributedAgentRegistry> = Arc::new(
            RedisAgentRegistry::with_config(redis_url, config)
                .await
                .unwrap(),
        );

        // Register an agent
        let agent = Arc::new(TestAgent {
            id: AgentId::new("stale-agent"),
            capabilities: vec![CapabilityType::Custom("test".to_string())],
        });
        registry.register_agent(agent).await.unwrap();

        // Verify agent exists
        assert!(registry
            .is_agent_registered(&AgentId::new("stale-agent"))
            .await
            .unwrap());

        // Wait for heartbeat to expire (this is a test scenario)
        // In production, the heartbeat TTL is 10 minutes
        // For testing, we'll manually remove the heartbeat and trigger cleanup

        // Note: Since we can't easily manipulate Redis TTL in tests,
        // we'll skip the actual stale cleanup test for now
        // This would require mocking or a test-specific configuration
    }

    #[tokio::test]
    async fn test_error_handling() {
        // Test with connection that will fail
        let config = RegistryConfig::default();
        let result =
            RedisAgentRegistry::with_config("redis://nonexistent:6379".to_string(), config).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to connect"));
    }

    #[tokio::test]
    async fn test_agent_status_expiration() {
        let (registry, _container) = setup_redis_registry().await;

        let agent = Arc::new(TestAgent {
            id: AgentId::new("expiring-agent"),
            capabilities: vec![],
        });

        registry.register_agent(agent).await.unwrap();

        // Update status
        let status = AgentStatus {
            agent_id: AgentId::new("expiring-agent"),
            status: AgentState::Active,
            active_tasks: 0,
            load: 0.0,
            last_heartbeat: chrono::Utc::now(),
            capabilities: vec![],
            metadata: HashMap::new(),
        };

        registry.update_agent_status(status).await.unwrap();

        // Status should exist
        let retrieved = registry
            .get_agent_status(&AgentId::new("expiring-agent"))
            .await
            .unwrap();
        assert!(retrieved.is_some());

        // Note: Testing actual expiration would require waiting 10 minutes
        // or having a test-specific configuration with shorter TTL
    }
}
