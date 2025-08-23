//! Channel-based communication implementation using tokio channels.

use super::{AgentCommunication, CommunicationConfig, CommunicationStats, MessageReceiver};
use crate::{AgentError, AgentId, AgentMessage, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, warn};

/// Channel-based communication system using tokio MPSC channels.
///
/// This implementation uses in-memory channels for message passing between
/// agents. It's suitable for single-node deployments and provides high
/// performance with low latency.
pub struct ChannelCommunication {
    /// Message channels per agent
    channels: RwLock<HashMap<AgentId, mpsc::UnboundedSender<AgentMessage>>>,
    /// Configuration
    config: CommunicationConfig,
    /// Statistics
    #[allow(dead_code)]
    stats: Arc<CommunicationStats>,
    /// Atomic counters for statistics
    messages_sent: AtomicU64,
    messages_received: AtomicU64,
    failed_deliveries: AtomicU64,
    expired_messages: AtomicU64,
}

impl fmt::Debug for ChannelCommunication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChannelCommunication")
            .field(
                "channels_count",
                &self.channels.try_read().map(|c| c.len()).unwrap_or(0),
            )
            .field("config", &self.config)
            .field("messages_sent", &self.messages_sent.load(Ordering::Relaxed))
            .field(
                "messages_received",
                &self.messages_received.load(Ordering::Relaxed),
            )
            .field(
                "failed_deliveries",
                &self.failed_deliveries.load(Ordering::Relaxed),
            )
            .field(
                "expired_messages",
                &self.expired_messages.load(Ordering::Relaxed),
            )
            .finish()
    }
}

impl ChannelCommunication {
    /// Create a new channel-based communication system with default configuration.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new channel-based communication system with configuration.
    pub fn with_config(config: CommunicationConfig) -> Self {
        Self {
            channels: RwLock::new(HashMap::default()),
            config,
            stats: Arc::new(CommunicationStats {
                active_subscriptions: 0,
                messages_sent: 0,
                messages_received: 0,
                failed_deliveries: 0,
                expired_messages: 0,
            }),
            messages_sent: AtomicU64::default(),
            messages_received: AtomicU64::default(),
            failed_deliveries: AtomicU64::default(),
            expired_messages: AtomicU64::default(),
        }
    }

    /// Get current statistics.
    pub async fn stats(&self) -> CommunicationStats {
        let channels = self.channels.read().await;
        CommunicationStats {
            active_subscriptions: channels.len(),
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            messages_received: self.messages_received.load(Ordering::Relaxed),
            failed_deliveries: self.failed_deliveries.load(Ordering::Relaxed),
            expired_messages: self.expired_messages.load(Ordering::Relaxed),
        }
    }

    /// Clean up expired messages and closed channels.
    pub async fn cleanup(&self) -> Result<usize> {
        let mut channels = self.channels.write().await;
        let initial_count = channels.len();

        // Remove closed channels
        channels.retain(|agent_id, sender| {
            if sender.is_closed() {
                debug!("Removing closed channel for agent {}", agent_id);
                false
            } else {
                true
            }
        });

        let removed_count = initial_count - channels.len();
        if removed_count > 0 {
            debug!("Cleaned up {} closed channels", removed_count);
        }

        Ok(removed_count)
    }

    /// Check if a message has expired.
    fn is_message_expired(&self, message: &AgentMessage) -> bool {
        if let Some(expires_at) = message.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            // Check against default TTL
            let age = chrono::Utc::now() - message.timestamp;
            age > chrono::Duration::from_std(self.config.message_ttl).unwrap_or_default()
        }
    }
}

impl Default for ChannelCommunication {
    fn default() -> Self {
        Self {
            channels: RwLock::new(HashMap::default()),
            config: CommunicationConfig::default(),
            stats: Arc::new(CommunicationStats {
                active_subscriptions: 0,
                messages_sent: 0,
                messages_received: 0,
                failed_deliveries: 0,
                expired_messages: 0,
            }),
            messages_sent: AtomicU64::default(),
            messages_received: AtomicU64::default(),
            failed_deliveries: AtomicU64::default(),
            expired_messages: AtomicU64::default(),
        }
    }
}

#[async_trait]
impl AgentCommunication for ChannelCommunication {
    async fn send_message(&self, message: AgentMessage) -> Result<()> {
        debug!("Sending message {} to agent {:?}", message.id, message.to);

        // Check if message has expired
        if self.is_message_expired(&message) {
            warn!("Message {} has expired, not sending", message.id);
            self.expired_messages.fetch_add(1, Ordering::Relaxed);
            return Err(AgentError::communication("Message has expired"));
        }

        let target_agent = message.to.as_ref().ok_or_else(|| {
            AgentError::communication("Cannot send point-to-point message without target agent")
        })?;

        let channels = self.channels.read().await;

        if let Some(sender) = channels.get(target_agent) {
            match sender.send(message.clone()) {
                Ok(()) => {
                    debug!(
                        "Message {} sent successfully to agent {}",
                        message.id, target_agent
                    );
                    self.messages_sent.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
                Err(_) => {
                    error!(
                        "Failed to send message {} to agent {}: channel closed",
                        message.id, target_agent
                    );
                    self.failed_deliveries.fetch_add(1, Ordering::Relaxed);
                    Err(AgentError::message_delivery_failed(
                        message.id,
                        target_agent.as_str(),
                    ))
                }
            }
        } else {
            warn!("No subscription found for agent {}", target_agent);
            self.failed_deliveries.fetch_add(1, Ordering::Relaxed);
            Err(AgentError::agent_not_found(target_agent.as_str()))
        }
    }

    async fn broadcast_message(&self, message: AgentMessage) -> Result<()> {
        debug!("Broadcasting message {} to all agents", message.id);

        // Check if message has expired
        if self.is_message_expired(&message) {
            warn!("Broadcast message {} has expired, not sending", message.id);
            self.expired_messages.fetch_add(1, Ordering::Relaxed);
            return Err(AgentError::communication("Message has expired"));
        }

        let channels = self.channels.read().await;
        let mut successful_sends = 0;
        let mut failed_sends = 0;

        for (agent_id, sender) in channels.iter() {
            // Don't send to the sender itself (if specified)
            if let Some(ref from_id) = message.to {
                if agent_id == from_id {
                    continue;
                }
            }

            match sender.send(message.clone()) {
                Ok(()) => {
                    successful_sends += 1;
                }
                Err(_) => {
                    warn!(
                        "Failed to broadcast message {} to agent {}: channel closed",
                        message.id, agent_id
                    );
                    failed_sends += 1;
                }
            }
        }

        debug!(
            "Broadcast message {} sent to {} agents, {} failures",
            message.id, successful_sends, failed_sends
        );

        self.messages_sent
            .fetch_add(successful_sends, Ordering::Relaxed);
        self.failed_deliveries
            .fetch_add(failed_sends, Ordering::Relaxed);

        if successful_sends == 0 && !channels.is_empty() {
            Err(AgentError::communication(
                "Failed to deliver broadcast message to any agent",
            ))
        } else {
            Ok(())
        }
    }

    async fn subscribe(&self, agent_id: &AgentId) -> Result<Box<dyn MessageReceiver>> {
        debug!("Creating subscription for agent {}", agent_id);

        // Check subscription limits
        if let Some(max_subs) = self.config.max_subscriptions {
            let current_subs = self.channels.read().await.len();
            if current_subs >= max_subs {
                return Err(AgentError::communication(format!(
                    "Maximum subscriptions reached ({}/{})",
                    current_subs, max_subs
                )));
            }
        }

        let (sender, receiver) = mpsc::unbounded_channel();

        let mut channels = self.channels.write().await;

        // Check if agent already has a subscription
        if channels.contains_key(agent_id) {
            warn!("Agent {} already has an active subscription", agent_id);
            return Err(AgentError::communication(format!(
                "Agent {} already has an active subscription",
                agent_id
            )));
        }

        channels.insert(agent_id.clone(), sender);

        debug!("Created subscription for agent {}", agent_id);

        Ok(Box::new(ChannelReceiver::new(receiver)))
    }

    async fn unsubscribe(&self, agent_id: &AgentId) -> Result<()> {
        debug!("Removing subscription for agent {}", agent_id);

        let mut channels = self.channels.write().await;

        if channels.remove(agent_id).is_some() {
            debug!("Removed subscription for agent {}", agent_id);
            Ok(())
        } else {
            warn!("No subscription found for agent {}", agent_id);
            Err(AgentError::agent_not_found(agent_id.as_str()))
        }
    }

    async fn subscription_count(&self) -> Result<usize> {
        let channels = self.channels.read().await;
        Ok(channels.len())
    }

    async fn health_check(&self) -> Result<bool> {
        // Clean up any closed channels
        let _cleaned = self.cleanup().await?;

        // Health check passes if we can access the channels
        let _channels = self.channels.read().await;
        Ok(true)
    }
}

/// Channel-based message receiver.
#[derive(Debug)]
struct ChannelReceiver {
    receiver: mpsc::UnboundedReceiver<AgentMessage>,
    closed: bool,
}

impl ChannelReceiver {
    fn new(receiver: mpsc::UnboundedReceiver<AgentMessage>) -> Self {
        Self {
            receiver,
            closed: false,
        }
    }
}

#[async_trait]
impl MessageReceiver for ChannelReceiver {
    async fn receive(&mut self) -> Option<AgentMessage> {
        if self.closed {
            return None;
        }

        match self.receiver.recv().await {
            Some(message) => Some(message),
            None => {
                self.closed = true;
                None
            }
        }
    }

    fn try_receive(&mut self) -> Option<AgentMessage> {
        if self.closed {
            return None;
        }

        match self.receiver.try_recv() {
            Ok(message) => Some(message),
            Err(mpsc::error::TryRecvError::Empty) => None,
            Err(mpsc::error::TryRecvError::Disconnected) => {
                self.closed = true;
                None
            }
        }
    }

    async fn close(&mut self) {
        self.closed = true;
        self.receiver.close();
    }

    fn is_closed(&self) -> bool {
        self.closed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[tokio::test]
    async fn test_channel_communication_basic() {
        let comm = ChannelCommunication::default();
        let agent_id = AgentId::new("test-agent");

        // Subscribe
        let mut receiver = comm.subscribe(&agent_id).await.unwrap();
        assert_eq!(comm.subscription_count().await.unwrap(), 1);

        // Send message
        let message = AgentMessage::new(
            AgentId::new("sender"),
            Some(agent_id.clone()),
            "test_message".to_string(),
            serde_json::json!({"data": "test"}),
        );

        comm.send_message(message.clone()).await.unwrap();

        // Receive message
        let received = receiver.receive().await;
        assert!(received.is_some());
        let received = received.unwrap();
        assert_eq!(received.id, message.id);
        assert_eq!(received.message_type, "test_message");

        // Unsubscribe
        comm.unsubscribe(&agent_id).await.unwrap();
        assert_eq!(comm.subscription_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_channel_communication_broadcast() {
        let comm = ChannelCommunication::default();

        // Subscribe multiple agents
        let agent1 = AgentId::new("agent1");
        let agent2 = AgentId::new("agent2");

        let mut receiver1 = comm.subscribe(&agent1).await.unwrap();
        let mut receiver2 = comm.subscribe(&agent2).await.unwrap();

        // Broadcast message
        let message = AgentMessage::broadcast(
            AgentId::new("broadcaster"),
            "broadcast_test".to_string(),
            serde_json::json!({"announcement": "hello all"}),
        );

        comm.broadcast_message(message.clone()).await.unwrap();

        // Both agents should receive the message
        let received1 = receiver1.receive().await;
        let received2 = receiver2.receive().await;

        assert!(received1.is_some());
        assert!(received2.is_some());
        assert_eq!(received1.unwrap().id, message.id);
        assert_eq!(received2.unwrap().id, message.id);
    }

    #[tokio::test]
    async fn test_channel_communication_send_to_nonexistent() {
        let comm = ChannelCommunication::default();

        let message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("nonexistent")),
            "test".to_string(),
            serde_json::json!({}),
        );

        let result = comm.send_message(message).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AgentError::AgentNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn test_channel_communication_duplicate_subscription() {
        let comm = ChannelCommunication::default();
        let agent_id = AgentId::new("test-agent");

        // First subscription should succeed
        let _receiver1 = comm.subscribe(&agent_id).await.unwrap();

        // Second subscription should fail
        let result = comm.subscribe(&agent_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_channel_communication_unsubscribe_nonexistent() {
        let comm = ChannelCommunication::default();
        let agent_id = AgentId::new("nonexistent");

        let result = comm.unsubscribe(&agent_id).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AgentError::AgentNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn test_message_receiver_try_receive() {
        let comm = ChannelCommunication::default();
        let agent_id = AgentId::new("test-agent");

        let mut receiver = comm.subscribe(&agent_id).await.unwrap();

        // Should return None when no message is available
        assert!(receiver.try_receive().is_none());

        // Send a message
        let message = AgentMessage::new(
            AgentId::new("sender"),
            Some(agent_id.clone()),
            "test".to_string(),
            serde_json::json!({}),
        );
        comm.send_message(message.clone()).await.unwrap();

        // Should return the message immediately
        let received = receiver.try_receive();
        assert!(received.is_some());
        assert_eq!(received.unwrap().id, message.id);
    }

    #[tokio::test]
    async fn test_message_receiver_close() {
        let comm = ChannelCommunication::default();
        let agent_id = AgentId::new("test-agent");

        let mut receiver = comm.subscribe(&agent_id).await.unwrap();
        assert!(!receiver.is_closed());

        receiver.close().await;
        assert!(receiver.is_closed());

        // Should return None after closing
        assert!(receiver.receive().await.is_none());
        assert!(receiver.try_receive().is_none());
    }

    #[tokio::test]
    async fn test_channel_communication_stats() {
        let comm = ChannelCommunication::default();
        let agent_id = AgentId::new("test-agent");

        let mut _receiver = comm.subscribe(&agent_id).await.unwrap();

        let stats = comm.stats().await;
        assert_eq!(stats.active_subscriptions, 1);
        assert_eq!(stats.messages_sent, 0);

        // Send a message
        let message = AgentMessage::new(
            AgentId::new("sender"),
            Some(agent_id.clone()),
            "test".to_string(),
            serde_json::json!({}),
        );
        comm.send_message(message).await.unwrap();

        let stats = comm.stats().await;
        assert_eq!(stats.messages_sent, 1);
        assert_eq!(stats.failed_deliveries, 0);
    }

    #[tokio::test]
    async fn test_channel_communication_health_check() {
        let comm = ChannelCommunication::default();
        assert!(comm.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_expired_message_handling() {
        let comm = ChannelCommunication::default();
        let agent_id = AgentId::new("test-agent");

        let _receiver = comm.subscribe(&agent_id).await.unwrap();

        // Create an expired message
        let mut message = AgentMessage::new(
            AgentId::new("sender"),
            Some(agent_id.clone()),
            "test".to_string(),
            serde_json::json!({}),
        );
        message.expires_at = Some(chrono::Utc::now() - chrono::Duration::seconds(1));

        let result = comm.send_message(message).await;
        assert!(result.is_err());

        let stats = comm.stats().await;
        assert_eq!(stats.expired_messages, 1);
    }

    #[test]
    fn test_default_constructor() {
        let comm = ChannelCommunication::default();
        assert_eq!(
            comm.config.message_ttl,
            CommunicationConfig::default().message_ttl
        );
    }

    #[test]
    fn test_with_config_constructor() {
        let config = CommunicationConfig {
            message_ttl: std::time::Duration::from_secs(300),
            max_subscriptions: Some(10),
            ..Default::default()
        };
        let comm = ChannelCommunication::with_config(config.clone());
        assert_eq!(comm.config.message_ttl, config.message_ttl);
        assert_eq!(comm.config.max_subscriptions, config.max_subscriptions);
    }

    #[test]
    fn test_is_message_expired_with_expires_at() {
        let comm = ChannelCommunication::default();

        // Test message with expires_at in the future
        let mut message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("target")),
            "test".to_string(),
            serde_json::json!({}),
        );
        message.expires_at = Some(chrono::Utc::now() + chrono::Duration::seconds(10));
        assert!(!comm.is_message_expired(&message));

        // Test message with expires_at in the past
        message.expires_at = Some(chrono::Utc::now() - chrono::Duration::seconds(10));
        assert!(comm.is_message_expired(&message));
    }

    #[test]
    fn test_is_message_expired_with_default_ttl() {
        let config = CommunicationConfig {
            message_ttl: std::time::Duration::from_millis(1), // Very short TTL
            ..Default::default()
        };
        let comm = ChannelCommunication::with_config(config);

        // Create an old message without expires_at
        let mut message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("target")),
            "test".to_string(),
            serde_json::json!({}),
        );
        message.timestamp = chrono::Utc::now() - chrono::Duration::seconds(10);
        message.expires_at = None;

        assert!(comm.is_message_expired(&message));
    }

    #[test]
    fn test_is_message_expired_fresh_message() {
        let comm = ChannelCommunication::default();

        // Test fresh message without expires_at
        let message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("target")),
            "test".to_string(),
            serde_json::json!({}),
        );

        assert!(!comm.is_message_expired(&message));
    }

    #[tokio::test]
    async fn test_cleanup_with_closed_channels() {
        let comm = ChannelCommunication::default();
        let agent_id = AgentId::new("test-agent");

        // Subscribe and then drop the receiver to close the channel
        let receiver = comm.subscribe(&agent_id).await.unwrap();
        drop(receiver);

        // Allow some time for channel to be marked as closed
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Cleanup should remove the closed channel
        let removed_count = comm.cleanup().await.unwrap();
        assert_eq!(removed_count, 1);
        assert_eq!(comm.subscription_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_cleanup_with_no_closed_channels() {
        let comm = ChannelCommunication::default();
        let agent_id = AgentId::new("test-agent");

        let _receiver = comm.subscribe(&agent_id).await.unwrap();

        // Cleanup should not remove any channels
        let removed_count = comm.cleanup().await.unwrap();
        assert_eq!(removed_count, 0);
        assert_eq!(comm.subscription_count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_send_message_without_target() {
        let comm = ChannelCommunication::default();

        // Create message without target
        let message = AgentMessage::broadcast(
            AgentId::new("sender"),
            "test".to_string(),
            serde_json::json!({}),
        );
        // This should have to = None for broadcast

        let result = comm.send_message(message).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot send point-to-point message without target agent"));
    }

    #[tokio::test]
    async fn test_send_message_to_closed_channel() {
        let comm = ChannelCommunication::default();
        let agent_id = AgentId::new("test-agent");

        // Subscribe and then drop the receiver to close the channel
        let receiver = comm.subscribe(&agent_id).await.unwrap();
        drop(receiver);

        // Allow some time for channel to be marked as closed
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let message = AgentMessage::new(
            AgentId::new("sender"),
            Some(agent_id.clone()),
            "test".to_string(),
            serde_json::json!({}),
        );

        let result = comm.send_message(message).await;
        assert!(result.is_err());

        let stats = comm.stats().await;
        assert_eq!(stats.failed_deliveries, 1);
    }

    #[tokio::test]
    async fn test_broadcast_message_expired() {
        let comm = ChannelCommunication::default();

        // Create an expired broadcast message
        let mut message = AgentMessage::broadcast(
            AgentId::new("sender"),
            "test".to_string(),
            serde_json::json!({}),
        );
        message.expires_at = Some(chrono::Utc::now() - chrono::Duration::seconds(1));

        let result = comm.broadcast_message(message).await;
        assert!(result.is_err());

        let stats = comm.stats().await;
        assert_eq!(stats.expired_messages, 1);
    }

    #[tokio::test]
    async fn test_broadcast_message_with_self_filtering() {
        let comm = ChannelCommunication::default();

        let sender_id = AgentId::new("sender");
        let agent1 = AgentId::new("agent1");
        let agent2 = AgentId::new("agent2");

        let mut receiver_sender = comm.subscribe(&sender_id).await.unwrap();
        let mut receiver1 = comm.subscribe(&agent1).await.unwrap();
        let mut receiver2 = comm.subscribe(&agent2).await.unwrap();

        // Create broadcast message with 'to' field set (this is used for self-filtering)
        let mut message =
            AgentMessage::broadcast(sender_id.clone(), "test".to_string(), serde_json::json!({}));
        message.to = Some(sender_id.clone()); // Set to sender to test self-filtering

        comm.broadcast_message(message.clone()).await.unwrap();

        // Sender should not receive the message due to self-filtering
        assert!(receiver_sender.try_receive().is_none());

        // Other agents should receive the message
        assert!(receiver1.try_receive().is_some());
        assert!(receiver2.try_receive().is_some());
    }

    #[tokio::test]
    async fn test_broadcast_message_to_empty_channels() {
        let comm = ChannelCommunication::default();

        let message = AgentMessage::broadcast(
            AgentId::new("sender"),
            "test".to_string(),
            serde_json::json!({}),
        );

        // Broadcasting to no channels should succeed
        let result = comm.broadcast_message(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_broadcast_message_all_channels_closed() {
        let comm = ChannelCommunication::default();
        let agent1 = AgentId::new("agent1");
        let agent2 = AgentId::new("agent2");

        // Subscribe and then drop receivers to close channels
        let receiver1 = comm.subscribe(&agent1).await.unwrap();
        let receiver2 = comm.subscribe(&agent2).await.unwrap();
        drop(receiver1);
        drop(receiver2);

        // Allow some time for channels to be marked as closed
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let message = AgentMessage::broadcast(
            AgentId::new("sender"),
            "test".to_string(),
            serde_json::json!({}),
        );

        let result = comm.broadcast_message(message).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to deliver broadcast message to any agent"));

        let stats = comm.stats().await;
        assert_eq!(stats.failed_deliveries, 2);
    }

    #[tokio::test]
    async fn test_subscription_limit_exceeded() {
        let config = CommunicationConfig {
            max_subscriptions: Some(1),
            ..Default::default()
        };
        let comm = ChannelCommunication::with_config(config);

        let agent1 = AgentId::new("agent1");
        let agent2 = AgentId::new("agent2");

        // First subscription should succeed
        let _receiver1 = comm.subscribe(&agent1).await.unwrap();

        // Second subscription should fail due to limit
        let result = comm.subscribe(&agent2).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Maximum subscriptions reached"));
    }

    #[tokio::test]
    async fn test_subscription_with_no_limit() {
        let config = CommunicationConfig {
            max_subscriptions: None,
            ..Default::default()
        };
        let comm = ChannelCommunication::with_config(config);

        let agent1 = AgentId::new("agent1");
        let agent2 = AgentId::new("agent2");

        // Both subscriptions should succeed
        let _receiver1 = comm.subscribe(&agent1).await.unwrap();
        let _receiver2 = comm.subscribe(&agent2).await.unwrap();

        assert_eq!(comm.subscription_count().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_channel_receiver_after_disconnection() {
        let comm = ChannelCommunication::default();
        let agent_id = AgentId::new("test-agent");

        let mut receiver = comm.subscribe(&agent_id).await.unwrap();

        // Unsubscribe to close the channel
        comm.unsubscribe(&agent_id).await.unwrap();

        // Receiver should detect disconnection
        let result = receiver.receive().await;
        assert!(result.is_none());
        assert!(receiver.is_closed());

        // Further receives should return None
        assert!(receiver.receive().await.is_none());
        assert!(receiver.try_receive().is_none());
    }

    #[tokio::test]
    async fn test_channel_receiver_try_receive_after_disconnection() {
        let comm = ChannelCommunication::default();
        let agent_id = AgentId::new("test-agent");

        let mut receiver = comm.subscribe(&agent_id).await.unwrap();

        // Unsubscribe to close the channel
        comm.unsubscribe(&agent_id).await.unwrap();

        // try_receive should detect disconnection
        let result = receiver.try_receive();
        assert!(result.is_none());
        assert!(receiver.is_closed());
    }

    #[test]
    fn test_channel_receiver_new() {
        let (_, rx) = mpsc::unbounded_channel();
        let receiver = ChannelReceiver::new(rx);
        assert!(!receiver.is_closed());
    }

    #[tokio::test]
    async fn test_stats_with_multiple_operations() {
        let comm = ChannelCommunication::default();
        let agent1 = AgentId::new("agent1");
        let agent2 = AgentId::new("agent2");

        let mut _receiver1 = comm.subscribe(&agent1).await.unwrap();
        let mut _receiver2 = comm.subscribe(&agent2).await.unwrap();

        // Send successful message
        let message1 = AgentMessage::new(
            AgentId::new("sender"),
            Some(agent1.clone()),
            "test".to_string(),
            serde_json::json!({}),
        );
        comm.send_message(message1).await.unwrap();

        // Send to non-existent agent
        let message2 = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("nonexistent")),
            "test".to_string(),
            serde_json::json!({}),
        );
        let _ = comm.send_message(message2).await;

        // Send expired message
        let mut message3 = AgentMessage::new(
            AgentId::new("sender"),
            Some(agent2.clone()),
            "test".to_string(),
            serde_json::json!({}),
        );
        message3.expires_at = Some(chrono::Utc::now() - chrono::Duration::seconds(1));
        let _ = comm.send_message(message3).await;

        let stats = comm.stats().await;
        assert_eq!(stats.active_subscriptions, 2);
        assert_eq!(stats.messages_sent, 1);
        assert_eq!(stats.failed_deliveries, 1);
        assert_eq!(stats.expired_messages, 1);
    }

    #[tokio::test]
    async fn test_broadcast_stats_tracking() {
        let comm = ChannelCommunication::default();
        let agent1 = AgentId::new("agent1");
        let agent2 = AgentId::new("agent2");

        let mut _receiver1 = comm.subscribe(&agent1).await.unwrap();
        let mut _receiver2 = comm.subscribe(&agent2).await.unwrap();

        let message = AgentMessage::broadcast(
            AgentId::new("sender"),
            "test".to_string(),
            serde_json::json!({}),
        );

        comm.broadcast_message(message).await.unwrap();

        let stats = comm.stats().await;
        assert_eq!(stats.messages_sent, 2); // Sent to both agents
        assert_eq!(stats.failed_deliveries, 0);
    }
}
