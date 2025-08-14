use std::sync::Arc;
use std::any::Any;
use tokio::sync::RwLock;
use async_trait::async_trait;
use riglr_solana_events::UnifiedEvent;
use tracing::{info, debug, warn};

use crate::core::{EventHandler, StreamManager};
use super::condition::{EventCondition, ConditionCombinator};
use super::event_utils::as_unified_event;

/// Generic tool trait for event-triggered execution
#[async_trait]
pub trait StreamingTool: Send + Sync {
    /// Execute the tool
    async fn execute(&self, event: &dyn UnifiedEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get tool name
    fn name(&self) -> &str;
}

/// Wrapper that makes any Tool event-triggered
pub struct EventTriggeredTool<T: StreamingTool> {
    /// The underlying tool
    tool: Arc<T>,
    /// Event conditions that trigger this tool
    conditions: Vec<Box<dyn EventCondition>>,
    /// Combinator for multiple conditions
    combinator: ConditionCombinator,
    /// Tool name
    name: String,
    /// Execution count
    execution_count: Arc<RwLock<u64>>,
}

impl<T: StreamingTool + 'static> EventTriggeredTool<T> {
    /// Create a new event-triggered tool
    pub fn new(tool: T, name: impl Into<String>) -> Self {
        Self {
            tool: Arc::new(tool),
            conditions: Vec::new(),
            combinator: ConditionCombinator::Any,
            name: name.into(),
            execution_count: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Add a condition
    pub fn with_condition(mut self, condition: Box<dyn EventCondition>) -> Self {
        self.conditions.push(condition);
        self
    }
    
    /// Set condition combinator
    pub fn with_combinator(mut self, combinator: ConditionCombinator) -> Self {
        self.combinator = combinator;
        self
    }
    
    /// Check if event matches conditions
    async fn matches_conditions(&self, event: &(dyn Any + Send + Sync)) -> bool {
        if self.conditions.is_empty() {
            return true; // No conditions means always trigger
        }
        
        match self.combinator {
            ConditionCombinator::All => {
                for condition in &self.conditions {
                    if !condition.matches(event).await {
                        return false;
                    }
                }
                true
            }
            ConditionCombinator::Any => {
                for condition in &self.conditions {
                    if condition.matches(event).await {
                        return true;
                    }
                }
                false
            }
            ConditionCombinator::None => {
                for condition in &self.conditions {
                    if condition.matches(event).await {
                        return false;
                    }
                }
                true
            }
        }
    }
    
    /// Execute the tool with event context
    async fn execute_with_event(&self, event: Arc<dyn Any + Send + Sync>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Use centralized helper to get UnifiedEvent
        let unified_event = as_unified_event(event.as_ref());
        
        if let Some(unified_event) = unified_event {
            info!(
                "Executing tool {} triggered by event type {:?}",
                self.name,
                unified_event.event_type()
            );
            
            // Increment execution count
            {
                let mut count = self.execution_count.write().await;
                *count += 1;
            }
            
            // Execute the underlying tool
            self.tool.execute(unified_event).await?;
        } else {
            debug!("Event is not a UnifiedEvent, skipping tool execution");
        }
        
        Ok(())
    }
    
    /// Get execution count
    pub async fn execution_count(&self) -> u64 {
        *self.execution_count.read().await
    }
}

#[async_trait]
impl<T: StreamingTool + 'static> EventHandler for EventTriggeredTool<T> {
    async fn should_handle(&self, event: &(dyn Any + Send + Sync)) -> bool {
        self.matches_conditions(event).await
    }
    
    async fn handle(&self, event: Arc<dyn Any + Send + Sync>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.execute_with_event(event).await
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Builder for event-triggered tools
pub struct EventTriggerBuilder<T: StreamingTool> {
    tool: T,
    name: String,
    conditions: Vec<Box<dyn EventCondition>>,
    combinator: ConditionCombinator,
}

impl<T: StreamingTool + 'static> EventTriggerBuilder<T> {
    /// Create a new builder
    pub fn new(tool: T, name: impl Into<String>) -> Self {
        Self {
            tool,
            name: name.into(),
            conditions: Vec::new(),
            combinator: ConditionCombinator::Any,
        }
    }
    
    /// Add a condition
    pub fn condition(mut self, condition: Box<dyn EventCondition>) -> Self {
        self.conditions.push(condition);
        self
    }
    
    /// Set combinator
    pub fn combinator(mut self, combinator: ConditionCombinator) -> Self {
        self.combinator = combinator;
        self
    }
    
    /// Build the event-triggered tool
    pub fn build(self) -> EventTriggeredTool<T> {
        let mut tool = EventTriggeredTool::new(self.tool, self.name);
        tool.conditions = self.conditions;
        tool.combinator = self.combinator;
        tool
    }
    
    /// Register with a stream manager
    pub async fn register(self, manager: &StreamManager) -> Arc<EventTriggeredTool<T>> 
    where
        T: 'static
    {
        let tool = Arc::new(self.build());
        manager.add_event_handler(tool.clone()).await;
        tool
    }
}