//! Channel-based communication implementation using tokio channels.

use super::{AgentCommunication, MessageReceiver, CommunicationConfig, CommunicationStats};
use crate::{AgentError, AgentId, AgentMessage, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, warn, error};

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
    stats: Arc<CommunicationStats>,
    /// Atomic counters for statistics
    messages_sent: AtomicU64,
    messages_received: AtomicU64,
    failed_deliveries: AtomicU64,
    expired_messages: AtomicU64,
}

impl ChannelCommunication {
    /// Create a new channel-based communication system.
    pub fn new() -> Self {
        Self::with_config(CommunicationConfig::default())
    }

    /// Create a new channel-based communication system with configuration.
    pub fn with_config(config: CommunicationConfig) -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
            config,
            stats: Arc::new(CommunicationStats {
                active_subscriptions: 0,
                messages_sent: 0,
                messages_received: 0,
                failed_deliveries: 0,
                expired_messages: 0,
            }),
            messages_sent: AtomicU64::new(0),
            messages_received: AtomicU64::new(0),
            failed_deliveries: AtomicU64::new(0),
            expired_messages: AtomicU64::new(0),
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
        Self::new()
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
                    debug!("Message {} sent successfully to agent {}", message.id, target_agent);
                    self.messages_sent.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
                Err(_) => {
                    error!("Failed to send message {} to agent {}: channel closed", 
                           message.id, target_agent);
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
                    warn!("Failed to broadcast message {} to agent {}: channel closed",
                          message.id, agent_id);
                    failed_sends += 1;
                }
            }
        }

        debug!("Broadcast message {} sent to {} agents, {} failures",
               message.id, successful_sends, failed_sends);

        self.messages_sent.fetch_add(successful_sends, Ordering::Relaxed);
        self.failed_deliveries.fetch_add(failed_sends, Ordering::Relaxed);

        if successful_sends == 0 && channels.len() > 0 {
            Err(AgentError::communication("Failed to deliver broadcast message to any agent"))
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
                    "Maximum subscriptions reached ({}/{})", current_subs, max_subs
                )));
            }
        }

        let (sender, receiver) = mpsc::unbounded_channel();
        
        let mut channels = self.channels.write().await;
        
        // Check if agent already has a subscription
        if channels.contains_key(agent_id) {
            warn!("Agent {} already has an active subscription", agent_id);
            return Err(AgentError::communication(format!(
                "Agent {} already has an active subscription", agent_id
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
        let comm = ChannelCommunication::new();
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
        let comm = ChannelCommunication::new();
        
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
        let comm = ChannelCommunication::new();
        
        let message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("nonexistent")),
            "test".to_string(),
            serde_json::json!({}),
        );

        let result = comm.send_message(message).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AgentError::AgentNotFound { .. }));
    }

    #[tokio::test]
    async fn test_channel_communication_duplicate_subscription() {
        let comm = ChannelCommunication::new();
        let agent_id = AgentId::new("test-agent");

        // First subscription should succeed
        let _receiver1 = comm.subscribe(&agent_id).await.unwrap();

        // Second subscription should fail
        let result = comm.subscribe(&agent_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_channel_communication_unsubscribe_nonexistent() {
        let comm = ChannelCommunication::new();
        let agent_id = AgentId::new("nonexistent");

        let result = comm.unsubscribe(&agent_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AgentError::AgentNotFound { .. }));
    }

    #[tokio::test]
    async fn test_message_receiver_try_receive() {
        let comm = ChannelCommunication::new();
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
        let comm = ChannelCommunication::new();
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
        let comm = ChannelCommunication::new();
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
        let comm = ChannelCommunication::new();
        assert!(comm.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_expired_message_handling() {
        let comm = ChannelCommunication::new();
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
}