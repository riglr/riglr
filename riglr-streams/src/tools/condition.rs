use std::any::Any;
use async_trait::async_trait;
use riglr_solana_events::{UnifiedEvent, EventType, ProtocolType};
use super::event_utils::as_unified_event;

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

/// Event type matcher
pub struct EventTypeMatcher {
    event_types: Vec<EventType>,
}

impl EventTypeMatcher {
    pub fn new(event_types: Vec<EventType>) -> Self {
        Self { event_types }
    }
    
    pub fn single(event_type: EventType) -> Self {
        Self {
            event_types: vec![event_type],
        }
    }
}

#[async_trait]
impl EventCondition for EventTypeMatcher {
    async fn matches(&self, event: &(dyn Any + Send + Sync)) -> bool {
        if let Some(unified_event) = as_unified_event(event) {
            self.event_types.contains(&unified_event.event_type())
        } else {
            false
        }
    }
    
    fn description(&self) -> String {
        format!("EventType in {:?}", self.event_types)
    }
}

/// Protocol type matcher
pub struct ProtocolMatcher {
    protocols: Vec<ProtocolType>,
}

impl ProtocolMatcher {
    pub fn new(protocols: Vec<ProtocolType>) -> Self {
        Self { protocols }
    }
    
    pub fn single(protocol: ProtocolType) -> Self {
        Self {
            protocols: vec![protocol],
        }
    }
}

#[async_trait]
impl EventCondition for ProtocolMatcher {
    async fn matches(&self, event: &(dyn Any + Send + Sync)) -> bool {
        if let Some(unified_event) = as_unified_event(event) {
            self.protocols.contains(&unified_event.protocol_type())
        } else {
            false
        }
    }
    
    fn description(&self) -> String {
        format!("Protocol in {:?}", self.protocols)
    }
}

/// Slot/block range matcher
pub struct BlockRangeMatcher {
    min_block: Option<u64>,
    max_block: Option<u64>,
}

impl BlockRangeMatcher {
    pub fn new(min_block: Option<u64>, max_block: Option<u64>) -> Self {
        Self { min_block, max_block }
    }
    
    pub fn after(block: u64) -> Self {
        Self {
            min_block: Some(block),
            max_block: None,
        }
    }
    
    pub fn before(block: u64) -> Self {
        Self {
            min_block: None,
            max_block: Some(block),
        }
    }
}

#[async_trait]
impl EventCondition for BlockRangeMatcher {
    async fn matches(&self, event: &(dyn Any + Send + Sync)) -> bool {
        let slot = if let Some(unified_event) = as_unified_event(event) {
            unified_event.slot()
        } else {
            return false;
        };
        
        if let Some(min) = self.min_block {
            if slot < min {
                return false;
            }
        }
        
        if let Some(max) = self.max_block {
            if slot > max {
                return false;
            }
        }
        
        true
    }
    
    fn description(&self) -> String {
        match (self.min_block, self.max_block) {
            (Some(min), Some(max)) => format!("Block in range [{}, {}]", min, max),
            (Some(min), None) => format!("Block >= {}", min),
            (None, Some(max)) => format!("Block <= {}", max),
            (None, None) => "Any block".to_string(),
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
        let conditions_desc: Vec<String> = self.conditions
            .iter()
            .map(|c| c.description())
            .collect();
        
        format!("{:?}({})", self.combinator, conditions_desc.join(", "))
    }
}

/// Helper trait for pattern matching
pub trait EventMatcher {
    /// Match event type
    fn event_type(event_type: EventType) -> Box<dyn EventCondition> {
        Box::new(EventTypeMatcher::single(event_type))
    }
    
    /// Match protocol
    fn protocol(protocol: ProtocolType) -> Box<dyn EventCondition> {
        Box::new(ProtocolMatcher::single(protocol))
    }
    
    /// Match block range
    fn block_range(min: Option<u64>, max: Option<u64>) -> Box<dyn EventCondition> {
        Box::new(BlockRangeMatcher::new(min, max))
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