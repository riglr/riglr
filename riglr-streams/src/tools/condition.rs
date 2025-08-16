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
pub struct EventKindMatcher {
    event_kinds: Vec<EventKind>,
}

impl EventKindMatcher {
    pub fn new(event_kinds: Vec<EventKind>) -> Self {
        Self { event_kinds }
    }

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
pub struct SourceMatcher {
    sources: Vec<String>,
}

impl SourceMatcher {
    pub fn new(sources: Vec<String>) -> Self {
        Self { sources }
    }

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
pub struct TimestampRangeMatcher {
    min_timestamp: Option<std::time::SystemTime>,
    max_timestamp: Option<std::time::SystemTime>,
}

impl TimestampRangeMatcher {
    pub fn new(
        min_timestamp: Option<std::time::SystemTime>,
        max_timestamp: Option<std::time::SystemTime>,
    ) -> Self {
        Self {
            min_timestamp,
            max_timestamp,
        }
    }

    pub fn after(timestamp: std::time::SystemTime) -> Self {
        Self {
            min_timestamp: Some(timestamp),
            max_timestamp: None,
        }
    }

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

impl<F> CustomCondition<F>
where
    F: Fn(&(dyn Any + Send + Sync)) -> bool + Send + Sync,
{
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

impl CompositeCondition {
    pub fn new(combinator: ConditionCombinator) -> Self {
        Self {
            conditions: Vec::new(),
            combinator,
        }
    }

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
    fn event_kind(event_kind: EventKind) -> Box<dyn EventCondition> {
        Box::new(EventKindMatcher::single(event_kind))
    }

    /// Match source
    fn source(source: String) -> Box<dyn EventCondition> {
        Box::new(SourceMatcher::single(source))
    }

    /// Match timestamp range
    fn timestamp_range(
        min: Option<std::time::SystemTime>,
        max: Option<std::time::SystemTime>,
    ) -> Box<dyn EventCondition> {
        Box::new(TimestampRangeMatcher::new(min, max))
    }

    /// Combine conditions with ALL
    fn all(conditions: Vec<Box<dyn EventCondition>>) -> Box<dyn EventCondition> {
        let mut composite = CompositeCondition::new(ConditionCombinator::All);
        for condition in conditions {
            composite = composite.add_condition(condition);
        }
        Box::new(composite)
    }

    /// Combine conditions with ANY
    fn any(conditions: Vec<Box<dyn EventCondition>>) -> Box<dyn EventCondition> {
        let mut composite = CompositeCondition::new(ConditionCombinator::Any);
        for condition in conditions {
            composite = composite.add_condition(condition);
        }
        Box::new(composite)
    }
}

/// Empty struct to implement EventMatcher
pub struct Matcher;

impl EventMatcher for Matcher {}
