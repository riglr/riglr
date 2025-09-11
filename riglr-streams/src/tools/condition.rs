use super::event_utils::as_event;
use async_trait::async_trait;
use riglr_events_core::prelude::EventKind;
use std::any::Any;

/// Trait for event conditions
#[async_trait]
pub trait EventCondition: Send + Sync {
    /// Check if an event matches this condition
    async fn matches(&self, event: &(dyn Any + Send + Sync)) -> bool;

    /// Get condition description
    fn description(&self) -> String;
}

/// How to combine multiple conditions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConditionCombinator {
    /// All conditions must match
    All,
    /// At least one condition must match
    Any,
    /// No conditions must match
    None,
}

/// Event kind matcher
#[derive(Debug)]
pub struct EventKindMatcher {
    event_kinds: Vec<EventKind>,
}

impl EventKindMatcher {
    /// Create a new EventKindMatcher with multiple event kinds
    pub fn new(event_kinds: Vec<EventKind>) -> Self {
        Self { event_kinds }
    }

    /// Create a new EventKindMatcher for a single event kind
    pub fn single(event_kind: EventKind) -> Self {
        Self {
            event_kinds: vec![event_kind],
        }
    }
}

#[async_trait]
impl EventCondition for EventKindMatcher {
    async fn matches(&self, event: &(dyn Any + Send + Sync)) -> bool {
        if let Some(event_ref) = as_event(event) {
            self.event_kinds.contains(event_ref.kind())
        } else {
            false
        }
    }

    fn description(&self) -> String {
        format!("EventKind in {:?}", self.event_kinds)
    }
}

/// Source matcher (matches event source)
#[derive(Debug)]
pub struct SourceMatcher {
    sources: Vec<String>,
}

impl SourceMatcher {
    /// Create a new SourceMatcher with multiple sources
    pub fn new(sources: Vec<String>) -> Self {
        Self { sources }
    }

    /// Create a new SourceMatcher for a single source
    pub fn single(source: String) -> Self {
        Self {
            sources: vec![source],
        }
    }
}

#[async_trait]
impl EventCondition for SourceMatcher {
    async fn matches(&self, event: &(dyn Any + Send + Sync)) -> bool {
        if let Some(event_ref) = as_event(event) {
            self.sources.contains(&event_ref.source().to_string())
        } else {
            false
        }
    }

    fn description(&self) -> String {
        format!("Source in {:?}", self.sources)
    }
}

/// Timestamp range matcher
#[derive(Debug)]
pub struct TimestampRangeMatcher {
    min_timestamp: Option<std::time::SystemTime>,
    max_timestamp: Option<std::time::SystemTime>,
}

impl TimestampRangeMatcher {
    /// Create a new TimestampRangeMatcher with optional min and max timestamps
    pub fn new(
        min_timestamp: Option<std::time::SystemTime>,
        max_timestamp: Option<std::time::SystemTime>,
    ) -> Self {
        Self {
            min_timestamp,
            max_timestamp,
        }
    }

    /// Create a TimestampRangeMatcher that matches events after the given timestamp
    pub fn after(timestamp: std::time::SystemTime) -> Self {
        Self {
            min_timestamp: Some(timestamp),
            max_timestamp: None,
        }
    }

    /// Create a TimestampRangeMatcher that matches events before the given timestamp
    pub fn before(timestamp: std::time::SystemTime) -> Self {
        Self {
            min_timestamp: None,
            max_timestamp: Some(timestamp),
        }
    }
}

#[async_trait]
impl EventCondition for TimestampRangeMatcher {
    async fn matches(&self, event: &(dyn Any + Send + Sync)) -> bool {
        let event_time = if let Some(event_ref) = as_event(event) {
            event_ref.timestamp()
        } else {
            return false;
        };

        if let Some(min) = self.min_timestamp {
            if event_time < min {
                return false;
            }
        }

        if let Some(max) = self.max_timestamp {
            if event_time > max {
                return false;
            }
        }

        true
    }

    fn description(&self) -> String {
        match (self.min_timestamp, self.max_timestamp) {
            (Some(min), Some(max)) => format!("Timestamp in range [{:?}, {:?}]", min, max),
            (Some(min), None) => format!("Timestamp >= {:?}", min),
            (None, Some(max)) => format!("Timestamp <= {:?}", max),
            (None, None) => "Any timestamp".to_string(),
        }
    }
}

/// Custom condition with closure
pub struct CustomCondition<F>
where
    F: Fn(&(dyn Any + Send + Sync)) -> bool + Send + Sync,
{
    predicate: F,
    description: String,
}

impl<F> std::fmt::Debug for CustomCondition<F>
where
    F: Fn(&(dyn Any + Send + Sync)) -> bool + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomCondition")
            .field("description", &self.description)
            .finish_non_exhaustive()
    }
}

impl<F> CustomCondition<F>
where
    F: Fn(&(dyn Any + Send + Sync)) -> bool + Send + Sync,
{
    /// Create a new CustomCondition with a predicate function and description
    pub fn new(predicate: F, description: impl Into<String>) -> Self {
        Self {
            predicate,
            description: description.into(),
        }
    }
}

#[async_trait]
impl<F> EventCondition for CustomCondition<F>
where
    F: Fn(&(dyn Any + Send + Sync)) -> bool + Send + Sync,
{
    async fn matches(&self, event: &(dyn Any + Send + Sync)) -> bool {
        (self.predicate)(event)
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}

/// Composite condition that combines multiple conditions
pub struct CompositeCondition {
    conditions: Vec<Box<dyn EventCondition>>,
    combinator: ConditionCombinator,
}

impl std::fmt::Debug for CompositeCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositeCondition")
            .field("combinator", &self.combinator)
            .field("conditions_count", &self.conditions.len())
            .finish()
    }
}

impl CompositeCondition {
    /// Create a new CompositeCondition with the specified combinator
    pub fn new(combinator: ConditionCombinator) -> Self {
        Self {
            conditions: Vec::new(),
            combinator,
        }
    }

    /// Add a condition to this composite condition
    pub fn add_condition(mut self, condition: Box<dyn EventCondition>) -> Self {
        self.conditions.push(condition);
        self
    }
}

#[async_trait]
impl EventCondition for CompositeCondition {
    async fn matches(&self, event: &(dyn Any + Send + Sync)) -> bool {
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

    fn description(&self) -> String {
        let conditions_desc: Vec<String> =
            self.conditions.iter().map(|c| c.description()).collect();

        format!("{:?}({})", self.combinator, conditions_desc.join(", "))
    }
}

/// Helper trait for pattern matching
pub trait EventMatcher {
    /// Match event kind
    fn event_kind(event_kind: EventKind) -> Box<dyn EventCondition>
    where
        Self: Sized,
    {
        Box::new(EventKindMatcher::single(event_kind))
    }

    /// Match source
    fn source(source: String) -> Box<dyn EventCondition>
    where
        Self: Sized,
    {
        Box::new(SourceMatcher::single(source))
    }

    /// Match timestamp range
    fn timestamp_range(
        min: Option<std::time::SystemTime>,
        max: Option<std::time::SystemTime>,
    ) -> Box<dyn EventCondition>
    where
        Self: Sized,
    {
        Box::new(TimestampRangeMatcher::new(min, max))
    }

    /// Combine conditions with ALL
    fn all(conditions: Vec<Box<dyn EventCondition>>) -> Box<dyn EventCondition>
    where
        Self: Sized,
    {
        let mut composite = CompositeCondition::new(ConditionCombinator::All);
        for condition in conditions {
            composite = composite.add_condition(condition);
        }
        Box::new(composite)
    }

    /// Combine conditions with ANY
    fn any(conditions: Vec<Box<dyn EventCondition>>) -> Box<dyn EventCondition>
    where
        Self: Sized,
    {
        let mut composite = CompositeCondition::new(ConditionCombinator::Any);
        for condition in conditions {
            composite = composite.add_condition(condition);
        }
        Box::new(composite)
    }
}

/// Empty struct to implement EventMatcher
#[derive(Debug)]
pub struct Matcher;

impl EventMatcher for Matcher {}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_events_core::prelude::{EventMetadata, GenericEvent};
    use serde_json::json;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    // Helper function to create test events
    fn create_test_event(
        id: &str,
        kind: EventKind,
        source: &str,
        timestamp_offset_secs: u64,
    ) -> GenericEvent {
        let timestamp =
            chrono::Utc::now() - chrono::Duration::seconds(timestamp_offset_secs as i64);
        let metadata =
            EventMetadata::with_timestamp(id.to_string(), kind, source.to_string(), timestamp);
        GenericEvent::with_metadata(metadata, json!({}))
    }

    fn create_test_event_with_system_time(
        id: &str,
        kind: EventKind,
        source: &str,
        system_time: SystemTime,
    ) -> GenericEvent {
        let timestamp: chrono::DateTime<chrono::Utc> = system_time.into();
        let metadata =
            EventMetadata::with_timestamp(id.to_string(), kind, source.to_string(), timestamp);
        GenericEvent::with_metadata(metadata, json!({}))
    }

    // Test EventKindMatcher

    #[tokio::test]
    async fn test_event_kind_matcher_new_when_multiple_kinds_should_store_all() {
        let kinds = vec![EventKind::Transaction, EventKind::Block, EventKind::Swap];
        let matcher = EventKindMatcher::new(kinds.clone());
        assert_eq!(matcher.event_kinds, kinds);
    }

    #[tokio::test]
    async fn test_event_kind_matcher_single_when_given_kind_should_store_single() {
        let matcher = EventKindMatcher::single(EventKind::Contract);
        assert_eq!(matcher.event_kinds, vec![EventKind::Contract]);
    }

    #[tokio::test]
    async fn test_event_kind_matcher_matches_when_event_kind_in_list_should_return_true() {
        let matcher = EventKindMatcher::new(vec![EventKind::Transaction, EventKind::Block]);
        let event = create_test_event("test1", EventKind::Transaction, "test-source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = matcher.matches(any_event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_event_kind_matcher_matches_when_event_kind_not_in_list_should_return_false() {
        let matcher = EventKindMatcher::new(vec![EventKind::Transaction, EventKind::Block]);
        let event = create_test_event("test1", EventKind::Swap, "test-source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = matcher.matches(any_event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_event_kind_matcher_matches_when_not_event_should_return_false() {
        let matcher = EventKindMatcher::single(EventKind::Transaction);
        let non_event = "not an event";
        let any_event: &(dyn Any + Send + Sync) = &non_event;

        let result = matcher.matches(any_event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_event_kind_matcher_description_when_multiple_kinds_should_format_correctly() {
        let kinds = vec![EventKind::Transaction, EventKind::Block];
        let matcher = EventKindMatcher::new(kinds.clone());
        let description = matcher.description();
        assert!(description.contains("EventKind in"));
        assert!(description.contains("Transaction"));
        assert!(description.contains("Block"));
    }

    // Test SourceMatcher

    #[tokio::test]
    async fn test_source_matcher_new_when_multiple_sources_should_store_all() {
        let sources = vec!["source1".to_string(), "source2".to_string()];
        let matcher = SourceMatcher::new(sources.clone());
        assert_eq!(matcher.sources, sources);
    }

    #[tokio::test]
    async fn test_source_matcher_single_when_given_source_should_store_single() {
        let matcher = SourceMatcher::single("test-source".to_string());
        assert_eq!(matcher.sources, vec!["test-source".to_string()]);
    }

    #[tokio::test]
    async fn test_source_matcher_matches_when_event_source_in_list_should_return_true() {
        let matcher = SourceMatcher::new(vec!["source1".to_string(), "source2".to_string()]);
        let event = create_test_event("test1", EventKind::Transaction, "source1", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = matcher.matches(any_event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_source_matcher_matches_when_event_source_not_in_list_should_return_false() {
        let matcher = SourceMatcher::new(vec!["source1".to_string(), "source2".to_string()]);
        let event = create_test_event("test1", EventKind::Transaction, "source3", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = matcher.matches(any_event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_source_matcher_matches_when_not_event_should_return_false() {
        let matcher = SourceMatcher::single("test-source".to_string());
        let non_event = 42u32;
        let any_event: &(dyn Any + Send + Sync) = &non_event;

        let result = matcher.matches(any_event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_source_matcher_description_when_multiple_sources_should_format_correctly() {
        let sources = vec!["source1".to_string(), "source2".to_string()];
        let matcher = SourceMatcher::new(sources.clone());
        let description = matcher.description();
        assert!(description.contains("Source in"));
        assert!(description.contains("source1"));
        assert!(description.contains("source2"));
    }

    // Test TimestampRangeMatcher

    #[tokio::test]
    async fn test_timestamp_range_matcher_new_when_both_bounds_should_store_both() {
        let min_time = SystemTime::now();
        let max_time = min_time + Duration::from_secs(3600);
        let matcher = TimestampRangeMatcher::new(Some(min_time), Some(max_time));
        assert_eq!(matcher.min_timestamp, Some(min_time));
        assert_eq!(matcher.max_timestamp, Some(max_time));
    }

    #[tokio::test]
    async fn test_timestamp_range_matcher_new_when_only_min_should_store_min_only() {
        let min_time = SystemTime::now();
        let matcher = TimestampRangeMatcher::new(Some(min_time), None);
        assert_eq!(matcher.min_timestamp, Some(min_time));
        assert_eq!(matcher.max_timestamp, None);
    }

    #[tokio::test]
    async fn test_timestamp_range_matcher_new_when_neither_bound_should_store_none() {
        let matcher = TimestampRangeMatcher::new(None, None);
        assert_eq!(matcher.min_timestamp, None);
        assert_eq!(matcher.max_timestamp, None);
    }

    #[tokio::test]
    async fn test_timestamp_range_matcher_after_when_given_timestamp_should_set_min_only() {
        let timestamp = SystemTime::now();
        let matcher = TimestampRangeMatcher::after(timestamp);
        assert_eq!(matcher.min_timestamp, Some(timestamp));
        assert_eq!(matcher.max_timestamp, None);
    }

    #[tokio::test]
    async fn test_timestamp_range_matcher_before_when_given_timestamp_should_set_max_only() {
        let timestamp = SystemTime::now();
        let matcher = TimestampRangeMatcher::before(timestamp);
        assert_eq!(matcher.min_timestamp, None);
        assert_eq!(matcher.max_timestamp, Some(timestamp));
    }

    #[tokio::test]
    async fn test_timestamp_range_matcher_matches_when_in_range_should_return_true() {
        let base_time = UNIX_EPOCH + Duration::from_secs(1000);
        let min_time = base_time;
        let max_time = base_time + Duration::from_secs(100);
        let event_time = base_time + Duration::from_secs(50);

        let matcher = TimestampRangeMatcher::new(Some(min_time), Some(max_time));
        let event = create_test_event_with_system_time(
            "test1",
            EventKind::Transaction,
            "source",
            event_time,
        );
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = matcher.matches(any_event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_timestamp_range_matcher_matches_when_before_min_should_return_false() {
        let base_time = UNIX_EPOCH + Duration::from_secs(1000);
        let min_time = base_time + Duration::from_secs(100);
        let event_time = base_time + Duration::from_secs(50);

        let matcher = TimestampRangeMatcher::new(Some(min_time), None);
        let event = create_test_event_with_system_time(
            "test1",
            EventKind::Transaction,
            "source",
            event_time,
        );
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = matcher.matches(any_event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_timestamp_range_matcher_matches_when_after_max_should_return_false() {
        let base_time = UNIX_EPOCH + Duration::from_secs(1000);
        let max_time = base_time + Duration::from_secs(100);
        let event_time = base_time + Duration::from_secs(150);

        let matcher = TimestampRangeMatcher::new(None, Some(max_time));
        let event = create_test_event_with_system_time(
            "test1",
            EventKind::Transaction,
            "source",
            event_time,
        );
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = matcher.matches(any_event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_timestamp_range_matcher_matches_when_no_bounds_should_return_true() {
        let matcher = TimestampRangeMatcher::new(None, None);
        let event = create_test_event("test1", EventKind::Transaction, "source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = matcher.matches(any_event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_timestamp_range_matcher_matches_when_not_event_should_return_false() {
        let matcher = TimestampRangeMatcher::new(None, None);
        let non_event = vec![1, 2, 3];
        let any_event: &(dyn Any + Send + Sync) = &non_event;

        let result = matcher.matches(any_event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_timestamp_range_matcher_description_when_both_bounds_should_format_range() {
        let min_time = UNIX_EPOCH + Duration::from_secs(1000);
        let max_time = min_time + Duration::from_secs(100);
        let matcher = TimestampRangeMatcher::new(Some(min_time), Some(max_time));
        let description = matcher.description();
        assert!(description.contains("Timestamp in range"));
    }

    #[tokio::test]
    async fn test_timestamp_range_matcher_description_when_only_min_should_format_gte() {
        let min_time = UNIX_EPOCH + Duration::from_secs(1000);
        let matcher = TimestampRangeMatcher::new(Some(min_time), None);
        let description = matcher.description();
        assert!(description.contains("Timestamp >="));
    }

    #[tokio::test]
    async fn test_timestamp_range_matcher_description_when_only_max_should_format_lte() {
        let max_time = UNIX_EPOCH + Duration::from_secs(1000);
        let matcher = TimestampRangeMatcher::new(None, Some(max_time));
        let description = matcher.description();
        assert!(description.contains("Timestamp <="));
    }

    #[tokio::test]
    async fn test_timestamp_range_matcher_description_when_no_bounds_should_format_any() {
        let matcher = TimestampRangeMatcher::new(None, None);
        let description = matcher.description();
        assert_eq!(description, "Any timestamp");
    }

    // Test CustomCondition

    #[tokio::test]
    async fn test_custom_condition_new_when_given_predicate_should_store_both() {
        let predicate = |_: &(dyn Any + Send + Sync)| -> bool { true };
        let description = "Always true";
        let condition = CustomCondition::new(predicate, description);
        assert_eq!(condition.description, description);
    }

    #[tokio::test]
    async fn test_custom_condition_matches_when_predicate_returns_true_should_return_true() {
        let predicate = |_: &(dyn Any + Send + Sync)| -> bool { true };
        let condition = CustomCondition::new(predicate, "Always true");
        let event = create_test_event("test1", EventKind::Transaction, "source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = condition.matches(any_event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_custom_condition_matches_when_predicate_returns_false_should_return_false() {
        let predicate = |_: &(dyn Any + Send + Sync)| -> bool { false };
        let condition = CustomCondition::new(predicate, "Always false");
        let event = create_test_event("test1", EventKind::Transaction, "source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = condition.matches(any_event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_custom_condition_description_when_given_string_should_return_string() {
        let predicate = |_: &(dyn Any + Send + Sync)| -> bool { true };
        let description = "Custom test condition";
        let condition = CustomCondition::new(predicate, description);
        assert_eq!(condition.description(), description);
    }

    #[tokio::test]
    async fn test_custom_condition_description_when_given_string_ref_should_convert_to_string() {
        let predicate = |_: &(dyn Any + Send + Sync)| -> bool { true };
        let condition = CustomCondition::new(predicate, "Test description");
        assert_eq!(condition.description(), "Test description");
    }

    // Test CompositeCondition

    #[tokio::test]
    async fn test_composite_condition_new_when_given_combinator_should_store_combinator() {
        let condition = CompositeCondition::new(ConditionCombinator::All);
        assert_eq!(condition.combinator, ConditionCombinator::All);
        assert!(condition.conditions.is_empty());
    }

    #[tokio::test]
    async fn test_composite_condition_add_condition_when_given_condition_should_add_to_list() {
        let mut condition = CompositeCondition::new(ConditionCombinator::All);
        let sub_condition: Box<dyn EventCondition> =
            Box::new(EventKindMatcher::single(EventKind::Transaction));
        condition = condition.add_condition(sub_condition);
        assert_eq!(condition.conditions.len(), 1);
    }

    #[tokio::test]
    async fn test_composite_condition_matches_when_all_combinator_and_all_match_should_return_true()
    {
        let kind_condition: Box<dyn EventCondition> =
            Box::new(EventKindMatcher::single(EventKind::Transaction));
        let source_condition: Box<dyn EventCondition> =
            Box::new(SourceMatcher::single("test-source".to_string()));

        let condition = CompositeCondition::new(ConditionCombinator::All)
            .add_condition(kind_condition)
            .add_condition(source_condition);

        let event = create_test_event("test1", EventKind::Transaction, "test-source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = condition.matches(any_event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_composite_condition_matches_when_all_combinator_and_one_fails_should_return_false(
    ) {
        let kind_condition: Box<dyn EventCondition> =
            Box::new(EventKindMatcher::single(EventKind::Transaction));
        let source_condition: Box<dyn EventCondition> =
            Box::new(SourceMatcher::single("other-source".to_string()));

        let condition = CompositeCondition::new(ConditionCombinator::All)
            .add_condition(kind_condition)
            .add_condition(source_condition);

        let event = create_test_event("test1", EventKind::Transaction, "test-source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = condition.matches(any_event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_composite_condition_matches_when_all_combinator_and_empty_should_return_true() {
        let condition = CompositeCondition::new(ConditionCombinator::All);
        let event = create_test_event("test1", EventKind::Transaction, "test-source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = condition.matches(any_event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_composite_condition_matches_when_any_combinator_and_one_matches_should_return_true(
    ) {
        let kind_condition: Box<dyn EventCondition> =
            Box::new(EventKindMatcher::single(EventKind::Block));
        let source_condition: Box<dyn EventCondition> =
            Box::new(SourceMatcher::single("test-source".to_string()));

        let condition = CompositeCondition::new(ConditionCombinator::Any)
            .add_condition(kind_condition)
            .add_condition(source_condition);

        let event = create_test_event("test1", EventKind::Transaction, "test-source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = condition.matches(any_event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_composite_condition_matches_when_any_combinator_and_none_match_should_return_false(
    ) {
        let kind_condition: Box<dyn EventCondition> =
            Box::new(EventKindMatcher::single(EventKind::Block));
        let source_condition: Box<dyn EventCondition> =
            Box::new(SourceMatcher::single("other-source".to_string()));

        let condition = CompositeCondition::new(ConditionCombinator::Any)
            .add_condition(kind_condition)
            .add_condition(source_condition);

        let event = create_test_event("test1", EventKind::Transaction, "test-source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = condition.matches(any_event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_composite_condition_matches_when_any_combinator_and_empty_should_return_false() {
        let condition = CompositeCondition::new(ConditionCombinator::Any);
        let event = create_test_event("test1", EventKind::Transaction, "test-source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = condition.matches(any_event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_composite_condition_matches_when_none_combinator_and_all_fail_should_return_true()
    {
        let kind_condition: Box<dyn EventCondition> =
            Box::new(EventKindMatcher::single(EventKind::Block));
        let source_condition: Box<dyn EventCondition> =
            Box::new(SourceMatcher::single("other-source".to_string()));

        let condition = CompositeCondition::new(ConditionCombinator::None)
            .add_condition(kind_condition)
            .add_condition(source_condition);

        let event = create_test_event("test1", EventKind::Transaction, "test-source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = condition.matches(any_event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_composite_condition_matches_when_none_combinator_and_one_matches_should_return_false(
    ) {
        let kind_condition: Box<dyn EventCondition> =
            Box::new(EventKindMatcher::single(EventKind::Transaction));
        let source_condition: Box<dyn EventCondition> =
            Box::new(SourceMatcher::single("other-source".to_string()));

        let condition = CompositeCondition::new(ConditionCombinator::None)
            .add_condition(kind_condition)
            .add_condition(source_condition);

        let event = create_test_event("test1", EventKind::Transaction, "test-source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = condition.matches(any_event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_composite_condition_matches_when_none_combinator_and_empty_should_return_true() {
        let condition = CompositeCondition::new(ConditionCombinator::None);
        let event = create_test_event("test1", EventKind::Transaction, "test-source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = condition.matches(any_event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_composite_condition_description_when_all_combinator_should_format_correctly() {
        let kind_condition: Box<dyn EventCondition> =
            Box::new(EventKindMatcher::single(EventKind::Transaction));
        let source_condition: Box<dyn EventCondition> =
            Box::new(SourceMatcher::single("test-source".to_string()));

        let condition = CompositeCondition::new(ConditionCombinator::All)
            .add_condition(kind_condition)
            .add_condition(source_condition);

        let description = condition.description();
        assert!(description.contains("All("));
        assert!(description.contains("EventKind"));
        assert!(description.contains("Source"));
    }

    #[tokio::test]
    async fn test_composite_condition_description_when_any_combinator_should_format_correctly() {
        let kind_condition: Box<dyn EventCondition> =
            Box::new(EventKindMatcher::single(EventKind::Transaction));

        let condition =
            CompositeCondition::new(ConditionCombinator::Any).add_condition(kind_condition);

        let description = condition.description();
        assert!(description.contains("Any("));
        assert!(description.contains("EventKind"));
    }

    #[tokio::test]
    async fn test_composite_condition_description_when_none_combinator_should_format_correctly() {
        let kind_condition: Box<dyn EventCondition> =
            Box::new(EventKindMatcher::single(EventKind::Transaction));

        let condition =
            CompositeCondition::new(ConditionCombinator::None).add_condition(kind_condition);

        let description = condition.description();
        assert!(description.contains("None("));
        assert!(description.contains("EventKind"));
    }

    #[tokio::test]
    async fn test_composite_condition_description_when_empty_should_format_empty() {
        let condition = CompositeCondition::new(ConditionCombinator::All);
        let description = condition.description();
        assert!(description.contains("All()"));
    }

    // Test ConditionCombinator enum

    #[test]
    fn test_condition_combinator_equality() {
        assert_eq!(ConditionCombinator::All, ConditionCombinator::All);
        assert_eq!(ConditionCombinator::Any, ConditionCombinator::Any);
        assert_eq!(ConditionCombinator::None, ConditionCombinator::None);
        assert_ne!(ConditionCombinator::All, ConditionCombinator::Any);
        assert_ne!(ConditionCombinator::Any, ConditionCombinator::None);
        assert_ne!(ConditionCombinator::All, ConditionCombinator::None);
    }

    #[test]
    fn test_condition_combinator_clone() {
        let combinator = ConditionCombinator::All;
        let cloned = combinator.clone();
        assert_eq!(combinator, cloned);
    }

    #[test]
    fn test_condition_combinator_copy() {
        let combinator = ConditionCombinator::Any;
        let copied = combinator;
        assert_eq!(combinator, copied);
    }

    #[test]
    fn test_condition_combinator_debug() {
        let combinator = ConditionCombinator::All;
        let debug_str = format!("{:?}", combinator);
        assert_eq!(debug_str, "All");
    }

    // Test EventMatcher trait

    #[test]
    fn test_matcher_event_kind_when_given_kind_should_return_boxed_matcher() {
        let condition = Matcher::event_kind(EventKind::Transaction);
        let description = condition.description();
        assert!(description.contains("EventKind"));
        assert!(description.contains("Transaction"));
    }

    #[test]
    fn test_matcher_source_when_given_source_should_return_boxed_matcher() {
        let condition = Matcher::source("test-source".to_string());
        let description = condition.description();
        assert!(description.contains("Source"));
        assert!(description.contains("test-source"));
    }

    #[test]
    fn test_matcher_timestamp_range_when_given_bounds_should_return_boxed_matcher() {
        let min_time = Some(UNIX_EPOCH + Duration::from_secs(1000));
        let max_time = Some(UNIX_EPOCH + Duration::from_secs(2000));
        let condition = Matcher::timestamp_range(min_time, max_time);
        let description = condition.description();
        assert!(description.contains("Timestamp in range"));
    }

    #[test]
    fn test_matcher_timestamp_range_when_no_bounds_should_return_any_timestamp() {
        let condition = Matcher::timestamp_range(None, None);
        let description = condition.description();
        assert_eq!(description, "Any timestamp");
    }

    #[test]
    fn test_matcher_all_when_given_conditions_should_return_composite_all() {
        let kind_condition = Matcher::event_kind(EventKind::Transaction);
        let source_condition = Matcher::source("test-source".to_string());
        let conditions = vec![kind_condition, source_condition];

        let composite = Matcher::all(conditions);
        let description = composite.description();
        assert!(description.contains("All("));
        assert!(description.contains("EventKind"));
        assert!(description.contains("Source"));
    }

    #[test]
    fn test_matcher_all_when_empty_conditions_should_return_empty_composite() {
        let conditions = vec![];
        let composite = Matcher::all(conditions);
        let description = composite.description();
        assert!(description.contains("All()"));
    }

    #[test]
    fn test_matcher_any_when_given_conditions_should_return_composite_any() {
        let kind_condition = Matcher::event_kind(EventKind::Transaction);
        let source_condition = Matcher::source("test-source".to_string());
        let conditions = vec![kind_condition, source_condition];

        let composite = Matcher::any(conditions);
        let description = composite.description();
        assert!(description.contains("Any("));
        assert!(description.contains("EventKind"));
        assert!(description.contains("Source"));
    }

    #[test]
    fn test_matcher_any_when_empty_conditions_should_return_empty_composite() {
        let conditions = vec![];
        let composite = Matcher::any(conditions);
        let description = composite.description();
        assert!(description.contains("Any()"));
    }

    // Test with edge cases

    #[tokio::test]
    async fn test_event_kind_matcher_empty_list_should_never_match() {
        let matcher = EventKindMatcher::new(vec![]);
        let event = create_test_event("test1", EventKind::Transaction, "source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = matcher.matches(any_event).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_source_matcher_empty_list_should_never_match() {
        let matcher = SourceMatcher::new(vec![]);
        let event = create_test_event("test1", EventKind::Transaction, "source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = matcher.matches(any_event).await;
        assert!(!result);
    }

    #[test]
    fn test_matcher_struct_instantiation() {
        let _matcher = Matcher;
        // Just testing that we can create the empty struct
    }

    // Test Custom EventKind

    #[tokio::test]
    async fn test_event_kind_matcher_with_custom_event_kind() {
        let custom_kind = EventKind::Custom("my-custom-event".to_string());
        let matcher = EventKindMatcher::single(custom_kind.clone());
        let event = create_test_event("test1", custom_kind, "source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = matcher.matches(any_event).await;
        assert!(result);
    }

    // Test edge case: exact timestamp boundary conditions

    #[tokio::test]
    async fn test_timestamp_range_matcher_exact_min_boundary_should_return_true() {
        let boundary_time = UNIX_EPOCH + Duration::from_secs(1000);
        let matcher = TimestampRangeMatcher::new(Some(boundary_time), None);
        let event = create_test_event_with_system_time(
            "test1",
            EventKind::Transaction,
            "source",
            boundary_time,
        );
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = matcher.matches(any_event).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_timestamp_range_matcher_exact_max_boundary_should_return_true() {
        let boundary_time = UNIX_EPOCH + Duration::from_secs(1000);
        let matcher = TimestampRangeMatcher::new(None, Some(boundary_time));
        let event = create_test_event_with_system_time(
            "test1",
            EventKind::Transaction,
            "source",
            boundary_time,
        );
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = matcher.matches(any_event).await;
        assert!(result);
    }

    // Test complex composite conditions

    #[tokio::test]
    async fn test_nested_composite_conditions() {
        let kind_condition1: Box<dyn EventCondition> =
            Box::new(EventKindMatcher::single(EventKind::Transaction));
        let kind_condition2: Box<dyn EventCondition> =
            Box::new(EventKindMatcher::single(EventKind::Block));

        let any_kinds = CompositeCondition::new(ConditionCombinator::Any)
            .add_condition(kind_condition1)
            .add_condition(kind_condition2);

        let source_condition: Box<dyn EventCondition> =
            Box::new(SourceMatcher::single("test-source".to_string()));

        let final_condition = CompositeCondition::new(ConditionCombinator::All)
            .add_condition(Box::new(any_kinds))
            .add_condition(source_condition);

        let event = create_test_event("test1", EventKind::Transaction, "test-source", 0);
        let any_event: &(dyn Any + Send + Sync) = &event;

        let result = final_condition.matches(any_event).await;
        assert!(result);
    }
}
