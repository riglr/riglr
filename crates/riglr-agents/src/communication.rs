//! Inter-agent communication system for message passing.
//!
//! This module provides the infrastructure for agents to communicate with
//! each other through structured messages. It supports both point-to-point
//! and broadcast messaging patterns.

extern crate alloc;

use crate::{types::Priority, AgentError, AgentId, AgentMessage, Result};
use alloc::vec::Vec;
use async_trait::async_trait;
use core::{
    fmt::{Debug, Formatter, Result as FmtResult},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, warn};

/// Trait for agent communication implementations.
///
/// The communication system enables agents to send messages to each other
/// for coordination, status updates, and data sharing. Implementations
/// can use various transport mechanisms (channels, queues, etc.).
#[async_trait]
pub trait Communication: Send + Sync {
    /// Send a message to a specific agent.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// Ok(()) if the message was sent successfully, Err otherwise.
    /// Broadcast a message to all agents.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to broadcast
    ///
    /// # Returns
    ///
    /// Ok(()) if the message was broadcast successfully, Err otherwise.
    async fn broadcast_message(&self, message: AgentMessage) -> Result<()>;

    /// Health check for the communication system.
    async fn health_check(&self) -> Result<bool>;

    /// Send a message to a specific agent.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// Ok(()) if the message was sent successfully, Err otherwise.
    async fn send_message(&self, message: AgentMessage) -> Result<()>;

    /// Subscribe to messages for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent ID to receive messages for
    ///
    /// # Returns
    ///
    /// A message receiver that yields incoming messages.
    async fn subscribe(&self, agent_id: &AgentId) -> Result<Box<dyn MessageReceiver>>;

    /// Get the number of active subscriptions.
    async fn subscription_count(&self) -> Result<usize>;

    /// Unsubscribe from messages for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent ID to stop receiving messages for
    async fn unsubscribe(&self, agent_id: &AgentId) -> Result<()>;
}

/// Trait for receiving messages from the communication system.
#[async_trait]
pub trait MessageReceiver: Send + Sync + Debug {
    /// Close the receiver.
    async fn close(&mut self);

    /// Check if the receiver is closed.
    fn is_closed(&self) -> bool;

    /// Receive the next message.
    ///
    /// # Returns
    ///
    /// The next message, or None if the receiver is closed.
    async fn receive(&mut self) -> Option<AgentMessage>;

    /// Try to receive a message without blocking.
    ///
    /// # Returns
    ///
    /// The next message if available, None if no message is ready.
    fn try_receive(&mut self) -> Option<AgentMessage>;
}

/// Configuration for communication systems.
#[derive(Debug, Clone)]
pub struct Config {
    /// Buffer size for channels
    pub channel_buffer_size: usize,
    /// Enable message persistence
    pub enable_persistence: bool,
    /// Maximum number of pending messages per agent
    pub max_pending_messages: usize,
    /// Maximum number of concurrent subscriptions
    pub max_subscriptions: Option<usize>,
    /// Message time-to-live
    pub message_ttl: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            channel_buffer_size: 100,
            enable_persistence: false,
            max_pending_messages: 1000,
            max_subscriptions: None,
            message_ttl: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Statistics about the communication system.
#[derive(Debug, Clone)]
pub struct Stats {
    /// Number of active subscriptions
    pub active_subscriptions: usize,
    /// Number of expired messages
    pub expired_messages: u64,
    /// Number of failed message deliveries
    pub failed_deliveries: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Total messages sent
    pub messages_sent: u64,
}

/// Message filter for selective message reception.
#[derive(Debug, Clone)]
pub enum MessageFilter {
    /// Accept all messages
    All,
    /// Combine multiple filters with AND logic
    And(Vec<MessageFilter>),
    /// Filter by message type
    MessageType(String),
    /// Combine multiple filters with OR logic
    Or(Vec<MessageFilter>),
    /// Filter by priority
    Priority(Priority),
    /// Filter by sender
    Sender(AgentId),
}

impl MessageFilter {
    /// Check if a message matches this filter.
    #[must_use]
    pub fn matches(&self, message: &AgentMessage) -> bool {
        match *self {
            Self::All => true,
            Self::And(ref filters) => filters.iter().all(|f| f.matches(message)),
            Self::MessageType(ref msg_type) => message.message_type == *msg_type,
            Self::Or(ref filters) => filters.iter().any(|f| f.matches(message)),
            Self::Priority(ref priority) => message.priority >= *priority,
            Self::Sender(ref sender_id) => message.from == *sender_id,
        }
    }
}

/// Channel-based communication system using tokio MPSC channels.
///
/// This implementation uses in-memory channels for message passing between
/// agents. It's suitable for single-node deployments and provides high
/// performance with low latency.
pub struct Channel {
    /// Message channels per agent
    channels: RwLock<HashMap<AgentId, mpsc::UnboundedSender<AgentMessage>>>,
    /// Configuration
    config: Config,
    /// Atomic counter for expired messages
    expired_messages: AtomicU64,
    /// Atomic counter for failed deliveries
    failed_deliveries: AtomicU64,
    /// Atomic counter for received messages
    messages_received: AtomicU64,
    /// Atomic counter for sent messages
    messages_sent: AtomicU64,
}

impl Debug for Channel {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("Channel")
            .field(
                "channels_count",
                &self
                    .channels
                    .try_read()
                    .map(|channels_ref| channels_ref.len())
                    .unwrap_or(0),
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
            .finish_non_exhaustive()
    }
}

impl Channel {
    /// Clean up expired messages and closed channels.
    ///
    /// # Errors
    ///
    /// Returns an error if unable to access the channels for cleanup.
    #[inline]
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

        let removed_count = initial_count.saturating_sub(channels.len());
        drop(channels);

        if removed_count > 0 {
            debug!("Cleaned up {} closed channels", removed_count);
        }

        Ok(removed_count)
    }

    /// Check if a message has expired.
    fn is_message_expired(&self, message: &AgentMessage) -> bool {
        message.expires_at.map_or_else(
            || {
                // Check against default TTL
                let age = chrono::Utc::now().signed_duration_since(message.timestamp);
                age > chrono::Duration::from_std(self.config.message_ttl).unwrap_or_default()
            },
            |expires_at| chrono::Utc::now() > expires_at,
        )
    }

    /// Create a new channel-based communication system with default configuration.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get current statistics.
    #[inline]
    pub async fn stats(&self) -> Stats {
        let channels = self.channels.read().await;
        Stats {
            active_subscriptions: channels.len(),
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            messages_received: self.messages_received.load(Ordering::Relaxed),
            failed_deliveries: self.failed_deliveries.load(Ordering::Relaxed),
            expired_messages: self.expired_messages.load(Ordering::Relaxed),
        }
    }

    /// Create a new channel-based communication system with configuration.
    #[must_use]
    #[inline]
    pub fn with_config(config: Config) -> Self {
        Self {
            channels: RwLock::new(HashMap::default()),
            config,
            expired_messages: AtomicU64::default(),
            failed_deliveries: AtomicU64::default(),
            messages_received: AtomicU64::default(),
            messages_sent: AtomicU64::default(),
        }
    }
}

impl Default for Channel {
    #[inline]
    fn default() -> Self {
        Self {
            channels: RwLock::new(HashMap::default()),
            config: Config::default(),
            expired_messages: AtomicU64::default(),
            failed_deliveries: AtomicU64::default(),
            messages_received: AtomicU64::default(),
            messages_sent: AtomicU64::default(),
        }
    }
}

#[async_trait]
impl Communication for Channel {
    #[inline]
    async fn broadcast_message(&self, message: AgentMessage) -> Result<()> {
        debug!("Broadcasting message {} to all agents", message.id);

        // Check if message has expired
        if self.is_message_expired(&message) {
            warn!("Broadcast message {} has expired, not sending", message.id);
            self.expired_messages.fetch_add(1, Ordering::Relaxed);
            return Err(AgentError::communication("Message has expired"));
        }

        let channels = self.channels.read().await;
        let mut successful_sends: usize = 0;
        let mut failed_sends: usize = 0;
        let channels_empty = channels.is_empty();

        for (agent_id, sender) in channels.iter() {
            // Don't send to the sender itself (if specified)
            if let Some(from_id) = message.to.as_ref() {
                if agent_id == from_id {
                    continue;
                }
            }

            if matches!(sender.send(message.clone()), Ok(())) {
                successful_sends = successful_sends.saturating_add(1);
            } else {
                warn!(
                    "Failed to broadcast message {} to agent {}: channel closed",
                    message.id, agent_id
                );
                failed_sends = failed_sends.saturating_add(1);
            }
        }
        drop(channels);

        debug!(
            "Broadcast message {} sent to {} agents, {} failures",
            message.id, successful_sends, failed_sends
        );

        self.messages_sent
            .fetch_add(successful_sends.try_into().unwrap_or(0), Ordering::Relaxed);
        self.failed_deliveries
            .fetch_add(failed_sends.try_into().unwrap_or(0), Ordering::Relaxed);

        if successful_sends == 0 && !channels_empty {
            return Err(AgentError::communication(
                "Failed to deliver broadcast message to any agent",
            ));
        }
        Ok(())
    }

    #[inline]
    async fn health_check(&self) -> Result<bool> {
        // Clean up any closed channels
        let _cleaned = self.cleanup().await?;

        // Health check passes if we can access the channels
        let _channels = self.channels.read().await;
        Ok(true)
    }

    #[inline]
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

        let result = {
            let channels = self.channels.read().await;
            channels
                .get(target_agent)
                .map(|sender| sender.send(message.clone()))
        };

        match result {
            Some(Ok(())) => {
                debug!(
                    "Message {} sent successfully to agent {}",
                    message.id, target_agent
                );
                self.messages_sent.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Some(Err(_)) => {
                error!(
                    "Failed to send message {} to agent {}: channel closed",
                    message.id, target_agent
                );
                self.failed_deliveries.fetch_add(1, Ordering::Relaxed);
                return Err(AgentError::message_delivery_failed(
                    message.id,
                    target_agent.as_str(),
                ));
            }
            None => {
                warn!("No subscription found for agent {}", target_agent);
                self.failed_deliveries.fetch_add(1, Ordering::Relaxed);
                Err(AgentError::agent_not_found(target_agent.as_str()))
            }
        }
    }

    #[inline]
    async fn subscribe(&self, agent_id: &AgentId) -> Result<Box<dyn MessageReceiver>> {
        debug!("Creating subscription for agent {}", agent_id);

        // Check subscription limits
        if let Some(max_subs) = self.config.max_subscriptions {
            let current_subs = self.channels.read().await.len();
            if current_subs >= max_subs {
                return Err(AgentError::communication(format!(
                    "Maximum subscriptions reached ({current_subs}/{max_subs})"
                )));
            }
        }

        let (sender, receiver) = mpsc::unbounded_channel();

        let mut channels = self.channels.write().await;

        // Check if agent already has a subscription
        if channels.contains_key(agent_id) {
            warn!("Agent {} already has an active subscription", agent_id);
            return Err(AgentError::communication(format!(
                "Agent {agent_id} already has an active subscription"
            )));
        }

        channels.insert(agent_id.clone(), sender);
        drop(channels);

        debug!("Created subscription for agent {}", agent_id);

        return Ok(Box::new(ChannelReceiver {
            closed: false,
            receiver,
        }));
    }

    #[inline]
    async fn subscription_count(&self) -> Result<usize> {
        let channels = self.channels.read().await;
        Ok(channels.len())
    }

    #[inline]
    async fn unsubscribe(&self, agent_id: &AgentId) -> Result<()> {
        debug!("Removing subscription for agent {}", agent_id);

        let removed = {
            let mut channels = self.channels.write().await;
            channels.remove(agent_id)
        };

        if removed.is_some() {
            debug!("Removed subscription for agent {}", agent_id);
            return Ok(());
        }
        warn!("No subscription found for agent {}", agent_id);
        Err(AgentError::agent_not_found(agent_id.as_str()))
    }
}

/// Channel-based message receiver.
#[derive(Debug)]
struct ChannelReceiver {
    /// Whether the receiver has been closed
    closed: bool,
    /// The underlying channel receiver
    receiver: mpsc::UnboundedReceiver<AgentMessage>,
}

#[async_trait]
impl MessageReceiver for ChannelReceiver {
    async fn close(&mut self) {
        self.closed = true;
        self.receiver.close();
    }
    fn is_closed(&self) -> bool {
        self.closed
    }

    async fn receive(&mut self) -> Option<AgentMessage> {
        if self.closed {
            return None;
        }

        if let Some(message) = self.receiver.recv().await {
            Some(message)
        } else {
            self.closed = true;
            None
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
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::types::*;
    use tokio::time::sleep;

    #[test]
    fn test_message_filter_all() {
        let filter = MessageFilter::All;
        let message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test".to_string(),
            serde_json::json!({}),
        );

        assert!(filter.matches(&message));
    }

    #[test]
    fn test_message_filter_message_type() {
        let filter = MessageFilter::MessageType("test_type".to_string());

        let matching_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test_type".to_string(),
            serde_json::json!({}),
        );

        let non_matching_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "other_type".to_string(),
            serde_json::json!({}),
        );

        assert!(filter.matches(&matching_message));
        assert!(!filter.matches(&non_matching_message));
    }

    #[test]
    fn test_message_filter_sender() {
        let sender_id = AgentId::new("specific_sender");
        let filter = MessageFilter::Sender(sender_id.clone());

        let matching_message = AgentMessage::new(
            sender_id,
            Some(AgentId::new("receiver")),
            "test".to_string(),
            serde_json::json!({}),
        );

        let non_matching_message = AgentMessage::new(
            AgentId::new("other_sender"),
            Some(AgentId::new("receiver")),
            "test".to_string(),
            serde_json::json!({}),
        );

        assert!(filter.matches(&matching_message));
        assert!(!filter.matches(&non_matching_message));
    }

    #[test]
    fn test_message_filter_priority() {
        let filter = MessageFilter::Priority(Priority::High);

        let high_priority_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test".to_string(),
            serde_json::json!({}),
        )
        .with_priority(Priority::High);

        let critical_priority_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test".to_string(),
            serde_json::json!({}),
        )
        .with_priority(Priority::Critical);

        let low_priority_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test".to_string(),
            serde_json::json!({}),
        )
        .with_priority(Priority::Low);

        assert!(filter.matches(&high_priority_message));
        assert!(filter.matches(&critical_priority_message));
        assert!(!filter.matches(&low_priority_message));
    }

    #[test]
    fn test_message_filter_and() {
        let filter = MessageFilter::And(vec![
            MessageFilter::MessageType("test_type".to_string()),
            MessageFilter::Priority(Priority::High),
        ]);

        let matching_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test_type".to_string(),
            serde_json::json!({}),
        )
        .with_priority(Priority::High);

        let non_matching_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test_type".to_string(),
            serde_json::json!({}),
        )
        .with_priority(Priority::Low);

        assert!(filter.matches(&matching_message));
        assert!(!filter.matches(&non_matching_message));
    }

    #[test]
    fn test_message_filter_or() {
        let filter = MessageFilter::Or(vec![
            MessageFilter::MessageType("type1".to_string()),
            MessageFilter::MessageType("type2".to_string()),
        ]);

        let message1 = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "type1".to_string(),
            serde_json::json!({}),
        );

        let message2 = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "type2".to_string(),
            serde_json::json!({}),
        );

        let message3 = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "type3".to_string(),
            serde_json::json!({}),
        );

        assert!(filter.matches(&message1));
        assert!(filter.matches(&message2));
        assert!(!filter.matches(&message3));
    }

    #[test]
    fn test_message_filter_and_empty() {
        let filter = MessageFilter::And(vec![]);
        let message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test".to_string(),
            serde_json::json!({}),
        );

        // Empty And filter should return true (all conditions met vacuously)
        assert!(filter.matches(&message));
    }

    #[test]
    fn test_message_filter_or_empty() {
        let filter = MessageFilter::Or(vec![]);
        let message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test".to_string(),
            serde_json::json!({}),
        );

        // Empty Or filter should return false (no conditions to satisfy)
        assert!(!filter.matches(&message));
    }

    #[test]
    fn test_message_filter_nested_combinations() {
        let filter = MessageFilter::And(vec![
            MessageFilter::Or(vec![
                MessageFilter::MessageType("type1".to_string()),
                MessageFilter::MessageType("type2".to_string()),
            ]),
            MessageFilter::Priority(Priority::Normal),
        ]);

        let matching_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "type1".to_string(),
            serde_json::json!({}),
        )
        .with_priority(Priority::High);

        let non_matching_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "type3".to_string(),
            serde_json::json!({}),
        )
        .with_priority(Priority::High);

        assert!(filter.matches(&matching_message));
        assert!(!filter.matches(&non_matching_message));
    }

    #[test]
    fn test_message_filter_priority_exact_match() {
        let filter = MessageFilter::Priority(Priority::Normal);

        let medium_priority_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test".to_string(),
            serde_json::json!({}),
        )
        .with_priority(Priority::Normal);

        assert!(filter.matches(&medium_priority_message));
    }

    #[test]
    fn test_communication_config_default() {
        let config = Config::default();

        assert_eq!(config.max_pending_messages, 1000);
        assert_eq!(config.message_ttl, Duration::from_secs(3600));
        assert!(!config.enable_persistence);
        assert_eq!(config.channel_buffer_size, 100);
        assert_eq!(config.max_subscriptions, None);
    }

    #[test]
    fn test_communication_config_custom() {
        let config = Config {
            channel_buffer_size: 50,
            enable_persistence: true,
            max_pending_messages: 500,
            max_subscriptions: Some(10),
            message_ttl: Duration::from_secs(1800),
        };

        assert_eq!(config.max_pending_messages, 500);
        assert_eq!(config.message_ttl, Duration::from_secs(1800));
        assert!(config.enable_persistence);
        assert_eq!(config.channel_buffer_size, 50);
        assert_eq!(config.max_subscriptions, Some(10));
    }

    #[test]
    fn test_communication_config_clone() {
        let config = Config::default();
        let cloned_config = config.clone();

        assert_eq!(
            config.max_pending_messages,
            cloned_config.max_pending_messages
        );
        assert_eq!(config.message_ttl, cloned_config.message_ttl);
        assert_eq!(config.enable_persistence, cloned_config.enable_persistence);
        assert_eq!(
            config.channel_buffer_size,
            cloned_config.channel_buffer_size
        );
        assert_eq!(config.max_subscriptions, cloned_config.max_subscriptions);
    }

    #[test]
    fn test_communication_stats_creation() {
        let stats = Stats {
            active_subscriptions: 5,
            expired_messages: 2,
            failed_deliveries: 3,
            messages_received: 95,
            messages_sent: 100,
        };

        assert_eq!(stats.active_subscriptions, 5);
        assert_eq!(stats.messages_sent, 100);
        assert_eq!(stats.messages_received, 95);
        assert_eq!(stats.failed_deliveries, 3);
        assert_eq!(stats.expired_messages, 2);
    }

    #[test]
    fn test_communication_stats_clone() {
        let stats = Stats {
            active_subscriptions: 5,
            expired_messages: 2,
            failed_deliveries: 3,
            messages_received: 95,
            messages_sent: 100,
        };

        let cloned_stats = stats.clone();

        assert_eq!(
            stats.active_subscriptions,
            cloned_stats.active_subscriptions
        );
        assert_eq!(stats.messages_sent, cloned_stats.messages_sent);
        assert_eq!(stats.messages_received, cloned_stats.messages_received);
        assert_eq!(stats.failed_deliveries, cloned_stats.failed_deliveries);
        assert_eq!(stats.expired_messages, cloned_stats.expired_messages);
    }

    #[test]
    fn test_message_filter_clone() {
        let filter = MessageFilter::MessageType("test".to_string());
        let cloned_filter = filter.clone();

        let message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test".to_string(),
            serde_json::json!({}),
        );

        assert!(filter.matches(&message));
        assert!(cloned_filter.matches(&message));
    }

    #[test]
    fn test_message_filter_and_single_condition() {
        let filter = MessageFilter::And(vec![MessageFilter::MessageType("test".to_string())]);

        let matching_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test".to_string(),
            serde_json::json!({}),
        );

        let non_matching_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "other".to_string(),
            serde_json::json!({}),
        );

        assert!(filter.matches(&matching_message));
        assert!(!filter.matches(&non_matching_message));
    }

    #[test]
    fn test_message_filter_or_single_condition() {
        let filter = MessageFilter::Or(vec![MessageFilter::MessageType("test".to_string())]);

        let matching_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test".to_string(),
            serde_json::json!({}),
        );

        let non_matching_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "other".to_string(),
            serde_json::json!({}),
        );

        assert!(filter.matches(&matching_message));
        assert!(!filter.matches(&non_matching_message));
    }

    #[test]
    fn test_message_filter_priority_all_levels() {
        let low_filter = MessageFilter::Priority(Priority::Low);
        let medium_filter = MessageFilter::Priority(Priority::Normal);
        let high_filter = MessageFilter::Priority(Priority::High);
        let critical_filter = MessageFilter::Priority(Priority::Critical);

        let low_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test".to_string(),
            serde_json::json!({}),
        )
        .with_priority(Priority::Low);

        let medium_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test".to_string(),
            serde_json::json!({}),
        )
        .with_priority(Priority::Normal);

        let high_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test".to_string(),
            serde_json::json!({}),
        )
        .with_priority(Priority::High);

        let critical_message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("receiver")),
            "test".to_string(),
            serde_json::json!({}),
        )
        .with_priority(Priority::Critical);

        // Low filter accepts all
        assert!(low_filter.matches(&low_message));
        assert!(low_filter.matches(&medium_message));
        assert!(low_filter.matches(&high_message));
        assert!(low_filter.matches(&critical_message));

        // Medium filter accepts medium and above
        assert!(!medium_filter.matches(&low_message));
        assert!(medium_filter.matches(&medium_message));
        assert!(medium_filter.matches(&high_message));
        assert!(medium_filter.matches(&critical_message));

        // High filter accepts high and above
        assert!(!high_filter.matches(&low_message));
        assert!(!high_filter.matches(&medium_message));
        assert!(high_filter.matches(&high_message));
        assert!(high_filter.matches(&critical_message));

        // Critical filter accepts only critical
        assert!(!critical_filter.matches(&low_message));
        assert!(!critical_filter.matches(&medium_message));
        assert!(!critical_filter.matches(&high_message));
        assert!(critical_filter.matches(&critical_message));
    }

    // Channel communication tests start here
    #[tokio::test]
    async fn test_channel_communication_basic() {
        let comm = Channel::default();
        let agent_id = AgentId::new("test-agent");

        // Subscribe
        let mut receiver = comm
            .subscribe(&agent_id)
            .await
            .expect("Failed to subscribe agent");
        assert_eq!(
            comm.subscription_count()
                .await
                .expect("Failed to get subscription count"),
            1
        );

        // Send message
        let message = AgentMessage::new(
            AgentId::new("sender"),
            Some(agent_id.clone()),
            "test_message".to_string(),
            serde_json::json!({"data": "test"}),
        );

        comm.send_message(message.clone())
            .await
            .expect("Failed to send message");

        // Receive message
        let message_received = receiver.receive().await;
        assert!(message_received.is_some());
        let received_msg = message_received.expect("Expected to receive message");
        assert_eq!(received_msg.id, message.id);
        assert_eq!(received_msg.message_type, "test_message");

        // Unsubscribe
        comm.unsubscribe(&agent_id)
            .await
            .expect("Failed to unsubscribe agent");
        assert_eq!(
            comm.subscription_count()
                .await
                .expect("Failed to get subscription count"),
            0
        );
    }

    #[tokio::test]
    async fn test_channel_communication_broadcast() {
        let comm = Channel::default();

        // Subscribe multiple agents
        let agent1 = AgentId::new("agent1");
        let agent2 = AgentId::new("agent2");

        let mut receiver1 = comm
            .subscribe(&agent1)
            .await
            .expect("Failed to subscribe agent1");
        let mut receiver2 = comm
            .subscribe(&agent2)
            .await
            .expect("Failed to subscribe agent2");

        // Broadcast message
        let message = AgentMessage::broadcast(
            AgentId::new("broadcaster"),
            "broadcast_test".to_string(),
            serde_json::json!({"announcement": "hello all"}),
        );

        comm.broadcast_message(message.clone())
            .await
            .expect("Failed to broadcast message");

        // Both agents should receive the message
        let msg1 = receiver1.receive().await;
        let msg2 = receiver2.receive().await;

        assert!(msg1.is_some());
        assert!(msg2.is_some());
        assert_eq!(
            msg1.expect("Expected receiver1 to get message").id,
            message.id
        );
        assert_eq!(
            msg2.expect("Expected receiver2 to get message").id,
            message.id
        );
    }

    #[tokio::test]
    async fn test_channel_communication_send_to_nonexistent() {
        let comm = Channel::default();

        let message = AgentMessage::new(
            AgentId::new("sender"),
            Some(AgentId::new("nonexistent")),
            "test".to_string(),
            serde_json::json!({}),
        );

        let result = comm.send_message(message).await;
        assert!(result.is_err());
        assert!(matches!(
            result.expect_err("Expected error for nonexistent agent"),
            AgentError::AgentNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn test_channel_communication_duplicate_subscription() {
        let comm = Channel::default();
        let agent_id = AgentId::new("test-agent");

        // First subscription should succeed
        let _receiver1 = comm
            .subscribe(&agent_id)
            .await
            .expect("Failed to subscribe agent");

        // Second subscription should fail
        let result = comm.subscribe(&agent_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_channel_communication_unsubscribe_nonexistent() {
        let comm = Channel::default();
        let agent_id = AgentId::new("nonexistent");

        let result = comm.unsubscribe(&agent_id).await;
        assert!(result.is_err());
        assert!(matches!(
            result.expect_err("Expected error for unsubscribing nonexistent agent"),
            AgentError::AgentNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn test_message_receiver_try_receive() {
        let comm = Channel::default();
        let agent_id = AgentId::new("test-agent");

        let mut receiver = comm
            .subscribe(&agent_id)
            .await
            .expect("Failed to subscribe agent");

        // Should return None when no message is available
        assert!(receiver.try_receive().is_none());

        // Send a message
        let message = AgentMessage::new(
            AgentId::new("sender"),
            Some(agent_id.clone()),
            "test".to_string(),
            serde_json::json!({}),
        );
        comm.send_message(message.clone())
            .await
            .expect("Failed to send message");

        // Should return the message immediately
        let msg = receiver.try_receive();
        assert!(msg.is_some());
        assert_eq!(msg.expect("Expected to receive message").id, message.id);
    }

    #[tokio::test]
    async fn test_message_receiver_close() {
        let comm = Channel::default();
        let agent_id = AgentId::new("test-agent");

        let mut receiver = comm
            .subscribe(&agent_id)
            .await
            .expect("Failed to subscribe agent");
        assert!(!receiver.is_closed());

        receiver.close().await;
        assert!(receiver.is_closed());

        // Should return None after closing
        assert!(receiver.receive().await.is_none());
        assert!(receiver.try_receive().is_none());
    }

    #[tokio::test]
    async fn test_channel_communication_stats() {
        let comm = Channel::default();
        let agent_id = AgentId::new("test-agent");

        let mut _receiver = comm
            .subscribe(&agent_id)
            .await
            .expect("Failed to subscribe agent");

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
        comm.send_message(message)
            .await
            .expect("Failed to send message");

        let stats = comm.stats().await;
        assert_eq!(stats.messages_sent, 1);
        assert_eq!(stats.failed_deliveries, 0);
    }

    #[tokio::test]
    async fn test_channel_communication_health_check() {
        let comm = Channel::default();
        assert!(comm.health_check().await.expect("Health check failed"));
    }

    #[tokio::test]
    async fn test_expired_message_handling() {
        let comm = Channel::default();
        let agent_id = AgentId::new("test-agent");

        let _receiver = comm
            .subscribe(&agent_id)
            .await
            .expect("Failed to subscribe agent");

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
        let comm = Channel::default();
        assert_eq!(comm.config.message_ttl, Config::default().message_ttl);
    }

    #[test]
    fn test_with_config_constructor() {
        let config = Config {
            message_ttl: Duration::from_secs(300),
            max_subscriptions: Some(10),
            ..Default::default()
        };
        let comm = Channel::with_config(config.clone());
        assert_eq!(comm.config.message_ttl, config.message_ttl);
        assert_eq!(comm.config.max_subscriptions, config.max_subscriptions);
    }

    #[test]
    fn test_is_message_expired_with_expires_at() {
        let comm = Channel::default();

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
        let config = Config {
            message_ttl: Duration::from_millis(1), // Very short TTL
            ..Default::default()
        };
        let comm = Channel::with_config(config);

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
        let comm = Channel::default();

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
        let comm = Channel::default();
        let agent_id = AgentId::new("test-agent");

        // Subscribe and then drop the receiver to close the channel
        let receiver = comm
            .subscribe(&agent_id)
            .await
            .expect("Failed to subscribe agent");
        drop(receiver);

        // Allow some time for channel to be marked as closed
        sleep(Duration::from_millis(10)).await;

        // Cleanup should remove the closed channel
        let removed_count = comm.cleanup().await.expect("Failed to cleanup channels");
        assert_eq!(removed_count, 1);
        assert_eq!(
            comm.subscription_count()
                .await
                .expect("Failed to get subscription count"),
            0
        );
    }

    #[tokio::test]
    async fn test_cleanup_with_no_closed_channels() {
        let comm = Channel::default();
        let agent_id = AgentId::new("test-agent");

        let _receiver = comm
            .subscribe(&agent_id)
            .await
            .expect("Failed to subscribe agent");

        // Cleanup should not remove any channels
        let removed_count = comm.cleanup().await.expect("Failed to cleanup channels");
        assert_eq!(removed_count, 0);
        assert_eq!(
            comm.subscription_count()
                .await
                .expect("Failed to get subscription count"),
            1
        );
    }

    #[tokio::test]
    async fn test_send_message_without_target() {
        let comm = Channel::default();

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
            .expect_err("Expected error for message without target")
            .to_string()
            .contains("Cannot send point-to-point message without target agent"));
    }

    #[tokio::test]
    async fn test_send_message_to_closed_channel() {
        let comm = Channel::default();
        let agent_id = AgentId::new("test-agent");

        // Subscribe and then drop the receiver to close the channel
        let receiver = comm
            .subscribe(&agent_id)
            .await
            .expect("Failed to subscribe agent");
        drop(receiver);

        // Allow some time for channel to be marked as closed
        sleep(Duration::from_millis(10)).await;

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
        let comm = Channel::default();

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
        let comm = Channel::default();

        let sender_id = AgentId::new("sender");
        let agent1 = AgentId::new("agent1");
        let agent2 = AgentId::new("agent2");

        let mut receiver_sender = comm
            .subscribe(&sender_id)
            .await
            .expect("Failed to subscribe sender");
        let mut receiver1 = comm
            .subscribe(&agent1)
            .await
            .expect("Failed to subscribe agent1");
        let mut receiver2 = comm
            .subscribe(&agent2)
            .await
            .expect("Failed to subscribe agent2");

        // Create broadcast message with 'to' field set (this is used for self-filtering)
        let mut message =
            AgentMessage::broadcast(sender_id.clone(), "test".to_string(), serde_json::json!({}));
        message.to = Some(sender_id.clone()); // Set to sender to test self-filtering

        comm.broadcast_message(message.clone())
            .await
            .expect("Failed to broadcast message");

        // Sender should not receive the message due to self-filtering
        assert!(receiver_sender.try_receive().is_none());

        // Other agents should receive the message
        assert!(receiver1.try_receive().is_some());
        assert!(receiver2.try_receive().is_some());
    }

    #[tokio::test]
    async fn test_broadcast_message_to_empty_channels() {
        let comm = Channel::default();

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
        let comm = Channel::default();
        let agent1 = AgentId::new("agent1");
        let agent2 = AgentId::new("agent2");

        // Subscribe and then drop receivers to close channels
        let receiver1 = comm
            .subscribe(&agent1)
            .await
            .expect("Failed to subscribe agent1");
        let receiver2 = comm
            .subscribe(&agent2)
            .await
            .expect("Failed to subscribe agent2");
        drop(receiver1);
        drop(receiver2);

        // Allow some time for channels to be marked as closed
        sleep(Duration::from_millis(10)).await;

        let message = AgentMessage::broadcast(
            AgentId::new("sender"),
            "test".to_string(),
            serde_json::json!({}),
        );

        let result = comm.broadcast_message(message).await;
        assert!(result.is_err());
        assert!(result
            .expect_err("Expected error for all channels closed")
            .to_string()
            .contains("Failed to deliver broadcast message to any agent"));

        let stats = comm.stats().await;
        assert_eq!(stats.failed_deliveries, 2);
    }

    #[tokio::test]
    async fn test_subscription_limit_exceeded() {
        let config = Config {
            max_subscriptions: Some(1),
            ..Default::default()
        };
        let comm = Channel::with_config(config);

        let agent1 = AgentId::new("agent1");
        let agent2 = AgentId::new("agent2");

        // First subscription should succeed
        let _receiver1 = comm
            .subscribe(&agent1)
            .await
            .expect("Failed to subscribe agent1");

        // Second subscription should fail due to limit
        let result = comm.subscribe(&agent2).await;
        assert!(result.is_err());
        assert!(result
            .expect_err("Expected error for subscription limit exceeded")
            .to_string()
            .contains("Maximum subscriptions reached"));
    }

    #[tokio::test]
    async fn test_subscription_with_no_limit() {
        let config = Config {
            max_subscriptions: None,
            ..Default::default()
        };
        let comm = Channel::with_config(config);

        let agent1 = AgentId::new("agent1");
        let agent2 = AgentId::new("agent2");

        // Both subscriptions should succeed
        let _receiver1 = comm
            .subscribe(&agent1)
            .await
            .expect("Failed to subscribe agent1");
        let _receiver2 = comm
            .subscribe(&agent2)
            .await
            .expect("Failed to subscribe agent2");

        assert_eq!(
            comm.subscription_count()
                .await
                .expect("Failed to get subscription count"),
            2
        );
    }

    #[tokio::test]
    async fn test_channel_receiver_after_disconnection() {
        let comm = Channel::default();
        let agent_id = AgentId::new("test-agent");

        let mut receiver = comm
            .subscribe(&agent_id)
            .await
            .expect("Failed to subscribe agent");

        // Unsubscribe to close the channel
        comm.unsubscribe(&agent_id)
            .await
            .expect("Failed to unsubscribe agent");

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
        let comm = Channel::default();
        let agent_id = AgentId::new("test-agent");

        let mut receiver = comm
            .subscribe(&agent_id)
            .await
            .expect("Failed to subscribe agent");

        // Unsubscribe to close the channel
        comm.unsubscribe(&agent_id)
            .await
            .expect("Failed to unsubscribe agent");

        // try_receive should detect disconnection
        let result = receiver.try_receive();
        assert!(result.is_none());
        assert!(receiver.is_closed());
    }

    #[test]
    fn test_channel_receiver_new() {
        let (_, rx) = mpsc::unbounded_channel();
        let receiver = ChannelReceiver {
            closed: false,
            receiver: rx,
        };
        assert!(!receiver.is_closed());
    }

    #[tokio::test]
    async fn test_stats_with_multiple_operations() {
        let comm = Channel::default();
        let agent1 = AgentId::new("agent1");
        let agent2 = AgentId::new("agent2");

        let mut _receiver1 = comm
            .subscribe(&agent1)
            .await
            .expect("Failed to subscribe agent1");
        let mut _receiver2 = comm
            .subscribe(&agent2)
            .await
            .expect("Failed to subscribe agent2");

        // Send successful message
        let message1 = AgentMessage::new(
            AgentId::new("sender"),
            Some(agent1.clone()),
            "test".to_string(),
            serde_json::json!({}),
        );
        comm.send_message(message1)
            .await
            .expect("Failed to send message1");

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
        let comm = Channel::default();
        let agent1 = AgentId::new("agent1");
        let agent2 = AgentId::new("agent2");

        let mut _receiver1 = comm
            .subscribe(&agent1)
            .await
            .expect("Failed to subscribe agent1");
        let mut _receiver2 = comm
            .subscribe(&agent2)
            .await
            .expect("Failed to subscribe agent2");

        let message = AgentMessage::broadcast(
            AgentId::new("sender"),
            "test".to_string(),
            serde_json::json!({}),
        );

        comm.broadcast_message(message)
            .await
            .expect("Failed to broadcast message");

        let stats = comm.stats().await;
        assert_eq!(stats.messages_sent, 2); // Sent to both agents
        assert_eq!(stats.failed_deliveries, 0);
    }
}
