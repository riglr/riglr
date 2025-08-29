use riglr_agents::*;
use std::sync::Arc;
use std::time::Duration;

// Import test utilities
use crate::common::*;

#[tokio::test]
async fn test_inter_agent_messaging() {
    let comm_system = ChannelCommunication::new();

    // Create agents that communicate with each other
    let agent_a_id = AgentId::new("agent-a");
    let agent_b_id = AgentId::new("agent-b");

    // Subscribe agents to communication
    let mut receiver_a = comm_system.subscribe(&agent_a_id).await.unwrap();
    let mut receiver_b = comm_system.subscribe(&agent_b_id).await.unwrap();

    // Send message from A to B
    let message = create_test_message(
        "agent-a",
        Some("agent-b"),
        "task_update",
        serde_json::json!({
            "status": "completed",
            "task_id": "task_123",
            "data": "test_data"
        }),
    );

    comm_system.send_message(message.clone()).await.unwrap();

    // Verify B received the message
    let received = receiver_b.receive().await;
    assert!(received.is_some());
    let received_msg = received.unwrap();
    assert_eq!(received_msg.id, message.id);
    assert_eq!(received_msg.message_type, "task_update");
    assert_eq!(received_msg.from, agent_a_id);

    // Verify A did not receive the message
    let no_message = receiver_a.try_receive();
    assert!(no_message.is_none());
}

#[tokio::test]
async fn test_broadcast_messaging() {
    let comm_system = ChannelCommunication::new();

    // Create multiple agents
    let agent_ids = vec![
        AgentId::new("agent-1"),
        AgentId::new("agent-2"),
        AgentId::new("agent-3"),
    ];

    let mut receivers = Vec::new();
    for agent_id in &agent_ids {
        let receiver = comm_system.subscribe(agent_id).await.unwrap();
        receivers.push(receiver);
    }

    // Broadcast message to all agents
    let broadcast_msg = create_broadcast_message(
        "broadcaster",
        "market_alert",
        serde_json::json!({
            "alert_type": "high_volatility",
            "symbol": "BTC/USD",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
    );

    comm_system
        .broadcast_message(broadcast_msg.clone())
        .await
        .unwrap();

    // Verify all agents received the broadcast
    for mut receiver in receivers {
        let received = receiver.receive().await;
        assert!(received.is_some());
        let received_msg = received.unwrap();
        assert_eq!(received_msg.id, broadcast_msg.id);
        assert_eq!(received_msg.message_type, "market_alert");
    }
}

#[tokio::test]
async fn test_agent_coordination_through_messaging() {
    let comm_system = ChannelCommunication::new();
    let _registry = Arc::new(LocalAgentRegistry::new());

    // Create coordinating agents
    let coordinator_id = AgentId::new("coordinator");
    let worker_a_id = AgentId::new("worker-a");
    let worker_b_id = AgentId::new("worker-b");

    let mut coordinator_receiver = comm_system.subscribe(&coordinator_id).await.unwrap();
    let mut worker_a_receiver = comm_system.subscribe(&worker_a_id).await.unwrap();
    let mut worker_b_receiver = comm_system.subscribe(&worker_b_id).await.unwrap();

    // Coordinator sends work assignment to workers
    let work_assignment_a = create_test_message(
        "coordinator",
        Some("worker-a"),
        "work_assignment",
        serde_json::json!({
            "task_type": "analysis",
            "data": "dataset_part_1"
        }),
    );

    let work_assignment_b = create_test_message(
        "coordinator",
        Some("worker-b"),
        "work_assignment",
        serde_json::json!({
            "task_type": "analysis",
            "data": "dataset_part_2"
        }),
    );

    comm_system.send_message(work_assignment_a).await.unwrap();
    comm_system.send_message(work_assignment_b).await.unwrap();

    // Workers receive assignments
    let assignment_a = worker_a_receiver.receive().await.unwrap();
    let assignment_b = worker_b_receiver.receive().await.unwrap();

    assert_eq!(assignment_a.message_type, "work_assignment");
    assert_eq!(assignment_b.message_type, "work_assignment");

    // Workers send completion notifications back to coordinator
    let completion_a = create_test_message(
        "worker-a",
        Some("coordinator"),
        "work_completed",
        serde_json::json!({
            "task_id": assignment_a.id,
            "result": "analysis_result_1"
        }),
    );

    let completion_b = create_test_message(
        "worker-b",
        Some("coordinator"),
        "work_completed",
        serde_json::json!({
            "task_id": assignment_b.id,
            "result": "analysis_result_2"
        }),
    );

    comm_system.send_message(completion_a).await.unwrap();
    comm_system.send_message(completion_b).await.unwrap();

    // Coordinator receives completion notifications
    let result_a = coordinator_receiver.receive().await.unwrap();
    let result_b = coordinator_receiver.receive().await.unwrap();

    assert_eq!(result_a.message_type, "work_completed");
    assert_eq!(result_b.message_type, "work_completed");
}

#[tokio::test]
async fn test_message_priority_handling() {
    let comm_system = ChannelCommunication::new();
    let agent_id = AgentId::new("priority-agent");

    let mut receiver = comm_system.subscribe(&agent_id).await.unwrap();

    // Send messages with different priorities
    let low_priority_msg = create_test_message(
        "sender",
        Some("priority-agent"),
        "low_priority",
        serde_json::json!({"data": "low"}),
    )
    .with_priority(Priority::Low);

    let critical_msg = create_test_message(
        "sender",
        Some("priority-agent"),
        "critical",
        serde_json::json!({"data": "critical"}),
    )
    .with_priority(Priority::Critical);

    let normal_msg = create_test_message(
        "sender",
        Some("priority-agent"),
        "normal",
        serde_json::json!({"data": "normal"}),
    )
    .with_priority(Priority::Normal);

    // Send in non-priority order
    comm_system
        .send_message(low_priority_msg.clone())
        .await
        .unwrap();
    comm_system
        .send_message(critical_msg.clone())
        .await
        .unwrap();
    comm_system.send_message(normal_msg.clone()).await.unwrap();

    // Receive messages (order depends on channel implementation)
    let msg1 = receiver.receive().await.unwrap();
    let msg2 = receiver.receive().await.unwrap();
    let msg3 = receiver.receive().await.unwrap();

    // Verify all messages were received
    let received_ids = vec![msg1.id, msg2.id, msg3.id];
    assert!(received_ids.contains(&low_priority_msg.id));
    assert!(received_ids.contains(&critical_msg.id));
    assert!(received_ids.contains(&normal_msg.id));
}

#[tokio::test]
async fn test_message_expiration() {
    let comm_system = ChannelCommunication::new();
    let agent_id = AgentId::new("test-agent");

    let _receiver = comm_system.subscribe(&agent_id).await.unwrap();

    // Create message that expires immediately
    let mut expired_msg = create_test_message(
        "sender",
        Some("test-agent"),
        "expired",
        serde_json::json!({"data": "should_expire"}),
    );
    expired_msg.expires_at = Some(chrono::Utc::now() - chrono::Duration::seconds(1));

    // Try to send expired message
    let result = comm_system.send_message(expired_msg).await;
    assert!(result.is_err());

    // Create message with future expiration
    let mut valid_msg = create_test_message(
        "sender",
        Some("test-agent"),
        "valid",
        serde_json::json!({"data": "should_deliver"}),
    );
    valid_msg.expires_at = Some(chrono::Utc::now() + chrono::Duration::minutes(1));

    // Should deliver successfully
    let result = comm_system.send_message(valid_msg).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_communication_stats_tracking() {
    let comm_system = ChannelCommunication::new();

    let agent_a_id = AgentId::new("agent-a");
    let agent_b_id = AgentId::new("agent-b");

    let mut _receiver_a = comm_system.subscribe(&agent_a_id).await.unwrap();
    let mut _receiver_b = comm_system.subscribe(&agent_b_id).await.unwrap();

    // Check initial stats
    let initial_stats = comm_system.stats().await;
    assert_eq!(initial_stats.active_subscriptions, 2);
    assert_eq!(initial_stats.messages_sent, 0);

    // Send some messages
    let message1 = create_test_message("agent-a", Some("agent-b"), "test1", serde_json::json!({}));
    let message2 = create_test_message("agent-b", Some("agent-a"), "test2", serde_json::json!({}));

    comm_system.send_message(message1).await.unwrap();
    comm_system.send_message(message2).await.unwrap();

    // Check updated stats
    let updated_stats = comm_system.stats().await;
    assert_eq!(updated_stats.messages_sent, 2);
    assert_eq!(updated_stats.failed_deliveries, 0);

    // Send to non-existent agent
    let failed_message = create_test_message(
        "agent-a",
        Some("nonexistent"),
        "fail",
        serde_json::json!({}),
    );
    let _ = comm_system.send_message(failed_message).await;

    // Check failure stats
    let final_stats = comm_system.stats().await;
    assert_eq!(final_stats.failed_deliveries, 1);
}

#[tokio::test]
async fn test_large_scale_message_passing() {
    let comm_system = ChannelCommunication::new();

    // Create many agents
    let num_agents = 20;
    let mut agent_ids = Vec::new();
    let mut receivers = Vec::new();

    for i in 0..num_agents {
        let agent_id = AgentId::new(&format!("agent-{}", i));
        let receiver = comm_system.subscribe(&agent_id).await.unwrap();
        agent_ids.push(agent_id);
        receivers.push(receiver);
    }

    // Send messages from each agent to every other agent
    let mut total_messages = 0;
    for sender_idx in 0..num_agents {
        for receiver_idx in 0..num_agents {
            if sender_idx != receiver_idx {
                let message = create_test_message(
                    &format!("agent-{}", sender_idx),
                    Some(&format!("agent-{}", receiver_idx)),
                    "peer_message",
                    serde_json::json!({
                        "from": sender_idx,
                        "to": receiver_idx,
                        "sequence": total_messages
                    }),
                );

                comm_system.send_message(message).await.unwrap();
                total_messages += 1;
            }
        }
    }

    // Verify message delivery stats
    let stats = comm_system.stats().await;
    assert_eq!(stats.messages_sent, total_messages);
    assert_eq!(stats.failed_deliveries, 0);

    // Each agent should receive messages from all other agents
    for (_agent_idx, mut receiver) in receivers.into_iter().enumerate() {
        let mut received_count = 0;

        // Try to receive all messages for this agent
        while let Some(_message) = receiver.try_receive() {
            received_count += 1;
        }

        // Each agent should receive num_agents - 1 messages
        assert_eq!(received_count, num_agents - 1);
    }
}

#[tokio::test]
async fn test_message_delivery_reliability() {
    let comm_system = ChannelCommunication::new();

    let _sender_id = AgentId::new("reliable-sender");
    let receiver_id = AgentId::new("reliable-receiver");

    let mut receiver = comm_system.subscribe(&receiver_id).await.unwrap();

    // Send a batch of messages rapidly
    let num_messages = 100;
    let mut sent_message_ids = Vec::new();

    for i in 0..num_messages {
        let message = create_test_message(
            "reliable-sender",
            Some("reliable-receiver"),
            "reliability_test",
            serde_json::json!({
                "sequence": i,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        );

        sent_message_ids.push(message.id.clone());
        comm_system.send_message(message).await.unwrap();
    }

    // Receive all messages
    let mut received_message_ids = Vec::new();
    for _ in 0..num_messages {
        if let Some(message) = receiver.receive().await {
            received_message_ids.push(message.id);
        }
    }

    // Verify all messages were delivered
    assert_eq!(received_message_ids.len(), num_messages);
    for sent_id in sent_message_ids {
        assert!(received_message_ids.contains(&sent_id));
    }
}

#[tokio::test]
async fn test_concurrent_message_passing() {
    let comm_system = Arc::new(ChannelCommunication::new());

    let num_senders = 5;
    let messages_per_sender = 20;

    // Create receivers
    let mut receivers = Vec::new();
    for i in 0..num_senders {
        let receiver_id = AgentId::new(&format!("receiver-{}", i));
        let receiver = comm_system.subscribe(&receiver_id).await.unwrap();
        receivers.push(receiver);
    }

    // Spawn concurrent senders
    let mut sender_handles = Vec::new();
    for sender_idx in 0..num_senders {
        let comm_system_clone = comm_system.clone();
        let handle = tokio::spawn(async move {
            for msg_idx in 0..messages_per_sender {
                for receiver_idx in 0..num_senders {
                    if sender_idx != receiver_idx {
                        let message = create_test_message(
                            &format!("sender-{}", sender_idx),
                            Some(&format!("receiver-{}", receiver_idx)),
                            "concurrent_test",
                            serde_json::json!({
                                "sender": sender_idx,
                                "message": msg_idx
                            }),
                        );

                        comm_system_clone.send_message(message).await.unwrap();
                    }
                }
            }
        });
        sender_handles.push(handle);
    }

    // Wait for all senders to complete
    for handle in sender_handles {
        handle.await.unwrap();
    }

    // Verify message delivery
    let stats = comm_system.stats().await;
    let expected_messages = num_senders * messages_per_sender * (num_senders - 1);
    assert_eq!(stats.messages_sent, expected_messages);
    assert_eq!(stats.failed_deliveries, 0);
}

#[tokio::test]
async fn test_agent_message_handling_workflow() {
    // Test the full message handling workflow from dispatch to agent processing
    let registry = Arc::new(LocalAgentRegistry::new());
    let comm_system = ChannelCommunication::new();

    // Create a custom agent that handles messages
    #[derive(Debug)]
    struct MessageHandlingAgent {
        id: AgentId,
        capabilities: Vec<CapabilityType>,
        received_messages: Arc<tokio::sync::Mutex<Vec<AgentMessage>>>,
    }

    impl MessageHandlingAgent {
        fn new(id: &str) -> Self {
            Self {
                id: AgentId::new(id),
                capabilities: vec![CapabilityType::Custom("message_handler".to_string())],
                received_messages: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            }
        }

        async fn get_received_messages(&self) -> Vec<AgentMessage> {
            self.received_messages.lock().await.clone()
        }
    }

    #[async_trait::async_trait]
    impl Agent for MessageHandlingAgent {
        async fn execute_task(&self, task: Task) -> Result<TaskResult> {
            Ok(TaskResult::success(
                serde_json::json!({"agent_id": self.id.as_str(), "task_id": task.id}),
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

        async fn handle_message(&self, message: AgentMessage) -> Result<()> {
            let mut messages = self.received_messages.lock().await;
            messages.push(message);
            Ok(())
        }
    }

    let message_agent = Arc::new(MessageHandlingAgent::new("message-handler"));
    let agent_clone = message_agent.clone();

    registry.register_agent(message_agent).await.unwrap();

    let mut receiver = comm_system.subscribe(agent_clone.id()).await.unwrap();

    // Send message to the agent
    let test_message = create_test_message(
        "external-sender",
        Some("message-handler"),
        "test_workflow",
        serde_json::json!({"workflow_step": "1", "data": "test_data"}),
    );

    comm_system
        .send_message(test_message.clone())
        .await
        .unwrap();

    // Agent receives and processes the message
    let received_msg = receiver.receive().await.unwrap();
    agent_clone.handle_message(received_msg).await.unwrap();

    // Verify the agent processed the message
    let processed_messages = agent_clone.get_received_messages().await;
    assert_eq!(processed_messages.len(), 1);
    assert_eq!(processed_messages[0].id, test_message.id);
    assert_eq!(processed_messages[0].message_type, "test_workflow");
}
