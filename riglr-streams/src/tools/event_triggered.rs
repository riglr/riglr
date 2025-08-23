use async_trait::async_trait;
use riglr_events_core::prelude::Event;
use std::any::Any;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::condition::{ConditionCombinator, EventCondition};
use super::event_utils::as_event;
use crate::core::{EventHandler, StreamManager};

/// Generic tool trait for event-triggered execution
#[async_trait]
pub trait StreamingTool: Send + Sync {
    /// Execute the tool
    async fn execute(
        &self,
        event: &dyn Event,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

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
    async fn execute_with_event(
        &self,
        event: Arc<dyn Any + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Try to downcast to specific event types that implement Event
        if let Some(event_ref) = as_event(event.as_ref()) {
            info!(
                "Executing tool {} triggered by event kind {:?}",
                self.name,
                event_ref.kind()
            );

            // Increment execution count
            {
                let mut count = self.execution_count.write().await;
                *count += 1;
            }

            // Execute the underlying tool
            self.tool.execute(event_ref).await?;
        } else {
            debug!("Event is not an Event, skipping tool execution");
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

    async fn handle(
        &self,
        event: Arc<dyn Any + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        T: 'static,
    {
        let tool = Arc::new(self.build());
        manager.add_event_handler(tool.clone()).await;
        tool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::condition::ConditionCombinator;
    use riglr_events_core::prelude::{Event, EventKind, EventMetadata};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::SystemTime;

    // Mock Event implementation for testing
    #[allow(dead_code)]
    #[derive(Debug, Clone)]
    struct MockEvent {
        metadata: EventMetadata,
        kind: EventKind,
        source: String,
        timestamp: SystemTime,
        data: String,
    }

    impl MockEvent {
        fn new(kind: EventKind, source: &str, data: &str) -> Self {
            Self {
                metadata: EventMetadata::new(
                    format!("mock-{}-{}", source, data),
                    kind.clone(),
                    source.to_string(),
                ),
                kind,
                source: source.to_string(),
                timestamp: SystemTime::now(),
                data: data.to_string(),
            }
        }
    }

    impl Event for MockEvent {
        fn id(&self) -> &str {
            &self.metadata.id
        }

        fn kind(&self) -> &EventKind {
            &self.metadata.kind
        }

        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }

        fn metadata_mut(&mut self) -> &mut EventMetadata {
            &mut self.metadata
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }
    }

    // Mock StreamingTool implementation for testing
    #[derive(Debug, Clone)]
    struct MockStreamingTool {
        name: String,
        execution_count: Arc<AtomicUsize>,
        should_fail: bool,
    }

    impl MockStreamingTool {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                execution_count: Arc::new(AtomicUsize::new(0)),
                should_fail: false,
            }
        }

        fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }

        fn execution_count(&self) -> usize {
            self.execution_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl StreamingTool for MockStreamingTool {
        async fn execute(
            &self,
            _event: &dyn Event,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.execution_count.fetch_add(1, Ordering::SeqCst);
            if self.should_fail {
                Err("Mock tool failed".into())
            } else {
                Ok(())
            }
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    // Mock EventCondition that always returns a specific value
    struct MockCondition {
        result: bool,
        description: String,
    }

    impl MockCondition {
        fn new(result: bool, description: &str) -> Self {
            Self {
                result,
                description: description.to_string(),
            }
        }
    }

    #[async_trait]
    impl EventCondition for MockCondition {
        async fn matches(&self, _event: &(dyn Any + Send + Sync)) -> bool {
            self.result
        }

        fn description(&self) -> String {
            self.description.clone()
        }
    }

    // Non-Event type for testing negative cases
    #[allow(dead_code)]
    #[derive(Debug)]
    struct NonEventType {
        data: String,
    }

    #[test]
    fn test_event_triggered_tool_new_should_create_with_defaults() {
        let tool = MockStreamingTool::new("test-tool");
        let triggered_tool = EventTriggeredTool::new(tool, "test-triggered");

        assert_eq!(triggered_tool.name, "test-triggered");
        assert_eq!(triggered_tool.conditions.len(), 0);
        assert_eq!(triggered_tool.combinator, ConditionCombinator::Any);
    }

    #[test]
    fn test_event_triggered_tool_with_condition_should_add_condition() {
        let tool = MockStreamingTool::new("test-tool");
        let condition = Box::new(MockCondition::new(true, "mock condition"));
        let triggered_tool = EventTriggeredTool::new(tool, "test").with_condition(condition);

        assert_eq!(triggered_tool.conditions.len(), 1);
    }

    #[test]
    fn test_event_triggered_tool_with_combinator_should_set_combinator() {
        let tool = MockStreamingTool::new("test-tool");
        let triggered_tool =
            EventTriggeredTool::new(tool, "test").with_combinator(ConditionCombinator::All);

        assert_eq!(triggered_tool.combinator, ConditionCombinator::All);
    }

    #[tokio::test]
    async fn test_matches_conditions_when_no_conditions_should_return_true() {
        let tool = MockStreamingTool::new("test-tool");
        let triggered_tool = EventTriggeredTool::new(tool, "test");
        let event = MockEvent::new(EventKind::Transaction, "test", "data");

        let result = triggered_tool.matches_conditions(&event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_matches_conditions_with_combinator_all_when_all_match_should_return_true() {
        let tool = MockStreamingTool::new("test-tool");
        let condition1 = Box::new(MockCondition::new(true, "condition1"));
        let condition2 = Box::new(MockCondition::new(true, "condition2"));
        let triggered_tool = EventTriggeredTool::new(tool, "test")
            .with_condition(condition1)
            .with_condition(condition2)
            .with_combinator(ConditionCombinator::All);
        let event = MockEvent::new(EventKind::Transaction, "test", "data");

        let result = triggered_tool.matches_conditions(&event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_matches_conditions_with_combinator_all_when_one_fails_should_return_false() {
        let tool = MockStreamingTool::new("test-tool");
        let condition1 = Box::new(MockCondition::new(true, "condition1"));
        let condition2 = Box::new(MockCondition::new(false, "condition2"));
        let triggered_tool = EventTriggeredTool::new(tool, "test")
            .with_condition(condition1)
            .with_condition(condition2)
            .with_combinator(ConditionCombinator::All);
        let event = MockEvent::new(EventKind::Transaction, "test", "data");

        let result = triggered_tool.matches_conditions(&event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_matches_conditions_with_combinator_any_when_one_matches_should_return_true() {
        let tool = MockStreamingTool::new("test-tool");
        let condition1 = Box::new(MockCondition::new(false, "condition1"));
        let condition2 = Box::new(MockCondition::new(true, "condition2"));
        let triggered_tool = EventTriggeredTool::new(tool, "test")
            .with_condition(condition1)
            .with_condition(condition2)
            .with_combinator(ConditionCombinator::Any);
        let event = MockEvent::new(EventKind::Transaction, "test", "data");

        let result = triggered_tool.matches_conditions(&event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_matches_conditions_with_combinator_any_when_none_match_should_return_false() {
        let tool = MockStreamingTool::new("test-tool");
        let condition1 = Box::new(MockCondition::new(false, "condition1"));
        let condition2 = Box::new(MockCondition::new(false, "condition2"));
        let triggered_tool = EventTriggeredTool::new(tool, "test")
            .with_condition(condition1)
            .with_condition(condition2)
            .with_combinator(ConditionCombinator::Any);
        let event = MockEvent::new(EventKind::Transaction, "test", "data");

        let result = triggered_tool.matches_conditions(&event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_matches_conditions_with_combinator_none_when_no_matches_should_return_true() {
        let tool = MockStreamingTool::new("test-tool");
        let condition1 = Box::new(MockCondition::new(false, "condition1"));
        let condition2 = Box::new(MockCondition::new(false, "condition2"));
        let triggered_tool = EventTriggeredTool::new(tool, "test")
            .with_condition(condition1)
            .with_condition(condition2)
            .with_combinator(ConditionCombinator::None);
        let event = MockEvent::new(EventKind::Transaction, "test", "data");

        let result = triggered_tool.matches_conditions(&event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_matches_conditions_with_combinator_none_when_one_matches_should_return_false() {
        let tool = MockStreamingTool::new("test-tool");
        let condition1 = Box::new(MockCondition::new(false, "condition1"));
        let condition2 = Box::new(MockCondition::new(true, "condition2"));
        let triggered_tool = EventTriggeredTool::new(tool, "test")
            .with_condition(condition1)
            .with_condition(condition2)
            .with_combinator(ConditionCombinator::None);
        let event = MockEvent::new(EventKind::Transaction, "test", "data");

        let result = triggered_tool.matches_conditions(&event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_execute_with_event_when_valid_event_should_execute_tool() {
        let tool = MockStreamingTool::new("test-tool");
        let triggered_tool = EventTriggeredTool::new(tool.clone(), "test");
        let event = Arc::new(MockEvent::new(EventKind::Transaction, "test", "data"));

        let result = triggered_tool.execute_with_event(event).await;
        assert!(result.is_ok());
        assert_eq!(tool.execution_count(), 1);
    }

    #[tokio::test]
    async fn test_execute_with_event_when_tool_fails_should_return_error() {
        let tool = MockStreamingTool::new("test-tool").with_failure();
        let triggered_tool = EventTriggeredTool::new(tool, "test");
        let event = Arc::new(MockEvent::new(EventKind::Transaction, "test", "data"));

        let result = triggered_tool.execute_with_event(event).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Mock tool failed"));
    }

    #[tokio::test]
    async fn test_execute_with_event_when_non_event_type_should_skip_execution() {
        let tool = MockStreamingTool::new("test-tool");
        let triggered_tool = EventTriggeredTool::new(tool.clone(), "test");
        let non_event = Arc::new(NonEventType {
            data: "test".to_string(),
        });

        let result = triggered_tool.execute_with_event(non_event).await;
        assert!(result.is_ok());
        assert_eq!(tool.execution_count(), 0);
    }

    #[tokio::test]
    async fn test_execution_count_should_track_executions() {
        let tool = MockStreamingTool::new("test-tool");
        let triggered_tool = EventTriggeredTool::new(tool, "test");
        let event = Arc::new(MockEvent::new(EventKind::Transaction, "test", "data"));

        assert_eq!(triggered_tool.execution_count().await, 0);

        let _ = triggered_tool.execute_with_event(event.clone()).await;
        assert_eq!(triggered_tool.execution_count().await, 1);

        let _ = triggered_tool.execute_with_event(event).await;
        assert_eq!(triggered_tool.execution_count().await, 2);
    }

    #[tokio::test]
    async fn test_should_handle_when_conditions_match_should_return_true() {
        let tool = MockStreamingTool::new("test-tool");
        let condition = Box::new(MockCondition::new(true, "mock condition"));
        let triggered_tool = EventTriggeredTool::new(tool, "test").with_condition(condition);
        let event = MockEvent::new(EventKind::Transaction, "test", "data");

        let result = triggered_tool.should_handle(&event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_should_handle_when_conditions_do_not_match_should_return_false() {
        let tool = MockStreamingTool::new("test-tool");
        let condition = Box::new(MockCondition::new(false, "mock condition"));
        let triggered_tool = EventTriggeredTool::new(tool, "test").with_condition(condition);
        let event = MockEvent::new(EventKind::Transaction, "test", "data");

        let result = triggered_tool.should_handle(&event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_handle_when_called_should_execute_tool() {
        let tool = MockStreamingTool::new("test-tool");
        let triggered_tool = EventTriggeredTool::new(tool.clone(), "test");
        let event = Arc::new(MockEvent::new(EventKind::Transaction, "test", "data"));

        let result = triggered_tool.handle(event).await;
        assert!(result.is_ok());
        assert_eq!(tool.execution_count(), 1);
    }

    #[test]
    fn test_name_should_return_tool_name() {
        let tool = MockStreamingTool::new("test-tool");
        let triggered_tool = EventTriggeredTool::new(tool, "test-triggered");

        assert_eq!(triggered_tool.name(), "test-triggered");
    }

    #[test]
    fn test_event_trigger_builder_new_should_create_with_defaults() {
        let tool = MockStreamingTool::new("test-tool");
        let builder = EventTriggerBuilder::new(tool, "test-builder");

        assert_eq!(builder.name, "test-builder");
        assert_eq!(builder.conditions.len(), 0);
        assert_eq!(builder.combinator, ConditionCombinator::Any);
    }

    #[test]
    fn test_event_trigger_builder_condition_should_add_condition() {
        let tool = MockStreamingTool::new("test-tool");
        let condition = Box::new(MockCondition::new(true, "mock condition"));
        let builder = EventTriggerBuilder::new(tool, "test").condition(condition);

        assert_eq!(builder.conditions.len(), 1);
    }

    #[test]
    fn test_event_trigger_builder_combinator_should_set_combinator() {
        let tool = MockStreamingTool::new("test-tool");
        let builder = EventTriggerBuilder::new(tool, "test").combinator(ConditionCombinator::All);

        assert_eq!(builder.combinator, ConditionCombinator::All);
    }

    #[test]
    fn test_event_trigger_builder_build_should_create_event_triggered_tool() {
        let tool = MockStreamingTool::new("test-tool");
        let condition = Box::new(MockCondition::new(true, "mock condition"));
        let triggered_tool = EventTriggerBuilder::new(tool, "test-builder")
            .condition(condition)
            .combinator(ConditionCombinator::All)
            .build();

        assert_eq!(triggered_tool.name, "test-builder");
        assert_eq!(triggered_tool.conditions.len(), 1);
        assert_eq!(triggered_tool.combinator, ConditionCombinator::All);
    }

    #[tokio::test]
    async fn test_execution_count_increment_with_concurrent_executions() {
        let tool = MockStreamingTool::new("test-tool");
        let triggered_tool = Arc::new(EventTriggeredTool::new(tool, "test"));
        let event = Arc::new(MockEvent::new(EventKind::Transaction, "test", "data"));

        // Execute multiple times concurrently
        let mut handles = vec![];
        for _ in 0..5 {
            let tool_clone = triggered_tool.clone();
            let event_clone = event.clone();
            handles.push(tokio::spawn(async move {
                tool_clone.execute_with_event(event_clone).await
            }));
        }

        // Wait for all executions to complete
        for handle in handles {
            assert!(handle.await.unwrap().is_ok());
        }

        assert_eq!(triggered_tool.execution_count().await, 5);
    }

    #[test]
    fn test_streaming_tool_trait_implementation() {
        // This test ensures our mock tool correctly implements the trait
        let tool = MockStreamingTool::new("test-tool");
        assert_eq!(tool.name(), "test-tool");
    }

    #[tokio::test]
    async fn test_streaming_tool_execute_success_and_failure() {
        let success_tool = MockStreamingTool::new("success-tool");
        let failure_tool = MockStreamingTool::new("failure-tool").with_failure();
        let event = MockEvent::new(EventKind::Transaction, "test", "data");

        let success_result = success_tool.execute(&event).await;
        assert!(success_result.is_ok());

        let failure_result = failure_tool.execute(&event).await;
        assert!(failure_result.is_err());
    }

    #[tokio::test]
    async fn test_complex_condition_combinations() {
        let tool = MockStreamingTool::new("test-tool");

        // Test with multiple conditions using All combinator
        let condition1 = Box::new(MockCondition::new(true, "condition1"));
        let condition2 = Box::new(MockCondition::new(true, "condition2"));
        let condition3 = Box::new(MockCondition::new(true, "condition3"));

        let triggered_tool = EventTriggeredTool::new(tool, "test")
            .with_condition(condition1)
            .with_condition(condition2)
            .with_condition(condition3)
            .with_combinator(ConditionCombinator::All);

        let event = MockEvent::new(EventKind::Transaction, "test", "data");
        let result = triggered_tool.matches_conditions(&event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_empty_conditions_with_different_combinators() {
        let tool = MockStreamingTool::new("test-tool");
        let event = MockEvent::new(EventKind::Transaction, "test", "data");

        // Test All combinator with no conditions
        let triggered_tool_all = EventTriggeredTool::new(tool.clone(), "test-all")
            .with_combinator(ConditionCombinator::All);
        assert!(triggered_tool_all.matches_conditions(&event).await);

        // Test Any combinator with no conditions
        let triggered_tool_any = EventTriggeredTool::new(tool.clone(), "test-any")
            .with_combinator(ConditionCombinator::Any);
        assert!(triggered_tool_any.matches_conditions(&event).await);

        // Test None combinator with no conditions
        let triggered_tool_none =
            EventTriggeredTool::new(tool, "test-none").with_combinator(ConditionCombinator::None);
        assert!(triggered_tool_none.matches_conditions(&event).await);
    }
}
