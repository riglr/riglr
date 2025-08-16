//! Inter-agent communication system for message passing.
//!
//! This module provides the infrastructure for agents to communicate with
//! each other through structured messages. It supports both point-to-point
//! and broadcast messaging patterns.

pub mod channels;

use crate::{AgentId, AgentMessage, Result};
use async_trait::async_trait;

pub use channels::ChannelCommunication;

/// Trait for agent communication implementations.
///
/// The communication system enables agents to send messages to each other
/// for coordination, status updates, and data sharing. Implementations
/// can use various transport mechanisms (channels, queues, etc.).
#[async_trait]
pub trait AgentCommunication: Send + Sync {
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

    /// Unsubscribe from messages for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent ID to stop receiving messages for
    async fn unsubscribe(&self, agent_id: &AgentId) -> Result<()>;

    /// Get the number of active subscriptions.
    async fn subscription_count(&self) -> Result<usize>;

    /// Health check for the communication system.
    async fn health_check(&self) -> Result<bool>;
}

/// Trait for receiving messages from the communication system.
#[async_trait]
pub trait MessageReceiver: Send + Sync {
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

    /// Close the receiver.
    async fn close(&mut self);

    /// Check if the receiver is closed.
    fn is_closed(&self) -> bool;
}

/// Configuration for communication systems.
#[derive(Debug, Clone)]
pub struct CommunicationConfig {
    /// Maximum number of pending messages per agent
    pub max_pending_messages: usize,
    /// Message time-to-live
    pub message_ttl: std::time::Duration,
    /// Enable message persistence
    pub enable_persistence: bool,
    /// Buffer size for channels
    pub channel_buffer_size: usize,
    /// Maximum number of concurrent subscriptions
    pub max_subscriptions: Option<usize>,
}

impl Default for CommunicationConfig {
    fn default() -> Self {
        Self {
            max_pending_messages: 1000,
            message_ttl: std::time::Duration::from_secs(3600), // 1 hour
            enable_persistence: false,
            channel_buffer_size: 100,
            max_subscriptions: None,
        }
    }
}

/// Statistics about the communication system.
#[derive(Debug, Clone)]
pub struct CommunicationStats {
    /// Number of active subscriptions
    pub active_subscriptions: usize,
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Number of failed message deliveries
    pub failed_deliveries: u64,
    /// Number of expired messages
    pub expired_messages: u64,
}

/// Message filter for selective message reception.
#[derive(Debug, Clone)]
pub enum MessageFilter {
    /// Accept all messages
    All,
    /// Filter by message type
    MessageType(String),
    /// Filter by sender
    Sender(AgentId),
    /// Filter by priority
    Priority(crate::types::Priority),
    /// Combine multiple filters with AND logic
    And(Vec<MessageFilter>),
    /// Combine multiple filters with OR logic
    Or(Vec<MessageFilter>),
}

impl MessageFilter {
    /// Check if a message matches this filter.
    pub fn matches(&self, message: &AgentMessage) -> bool {
        match self {
            MessageFilter::All => true,
            MessageFilter::MessageType(msg_type) => message.message_type == *msg_type,
            MessageFilter::Sender(sender_id) => message.from == *sender_id,
            MessageFilter::Priority(priority) => message.priority >= *priority,
            MessageFilter::And(filters) => filters.iter().all(|f| f.matches(message)),
            MessageFilter::Or(filters) => filters.iter().any(|f| f.matches(message)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

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
            sender_id.clone(),
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
}
