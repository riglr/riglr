//! Filtered Events Example
//!
//! Demonstrates advanced event filtering and routing capabilities.
//! This example shows how to:
//! - Create complex event filters
//! - Route events to different handlers
//! - Implement pattern matching for event sequences
//! - Use windowing for time-based filtering

use anyhow::Result;
use riglr_core::ToolResult;
use riglr_events_core::prelude::*;
use riglr_showcase::config::Config;
use riglr_streams::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tracing::{info, debug, warn};

/// Event filter that routes events based on criteria
#[derive(Debug)]
pub struct EventRouter {
    filters: Vec<EventFilterRule>,
    stats: Arc<RwLock<FilterStats>>,
}

#[derive(Debug, Clone)]
pub struct EventFilterRule {
    pub name: String,
    pub predicate: fn(&dyn Event) -> bool,
    pub priority: u8,
    pub enabled: bool,
}

#[derive(Debug, Default)]
pub struct FilterStats {
    pub total_events: u64,
    pub filtered_events: HashMap<String, u64>,
    pub dropped_events: u64,
    pub processing_time_ms: HashMap<String, u64>,
}

impl EventRouter {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            stats: Arc::new(RwLock::new(FilterStats::default())),
        }
    }

    pub fn add_filter(&mut self, filter: EventFilterRule) {
        // Insert in priority order (higher priority first)
        let pos = self.filters
            .iter()
            .position(|f| f.priority < filter.priority)
            .unwrap_or(self.filters.len());
        
        self.filters.insert(pos, filter);
        info!("📋 Added filter: {} (priority: {})", self.filters[pos].name, self.filters[pos].priority);
    }

    pub async fn route_event(&self, event: &dyn Event) -> Vec<String> {
        let start = Instant::now();
        let mut matches = Vec::new();
        
        // Update total events counter
        {
            let mut stats = self.stats.write().await;
            stats.total_events += 1;
        }

        // Apply filters in priority order
        for filter in &self.filters {
            if !filter.enabled {
                continue;
            }

            if (filter.predicate)(event) {
                matches.push(filter.name.clone());
                
                // Update filter stats
                let mut stats = self.stats.write().await;
                *stats.filtered_events.entry(filter.name.clone()).or_insert(0) += 1;
            }
        }

        // Record processing time
        let processing_time = start.elapsed().as_millis() as u64;
        {
            let mut stats = self.stats.write().await;
            for filter_name in &matches {
                *stats.processing_time_ms.entry(filter_name.clone()).or_insert(0) += processing_time;
            }
        }

        if matches.is_empty() {
            let mut stats = self.stats.write().await;
            stats.dropped_events += 1;
        }

        matches
    }

    pub async fn get_stats(&self) -> FilterStats {
        self.stats.read().await.clone()
    }
}

/// Pattern-based event matcher for complex sequences
pub struct EventSequenceMatcher {
    pattern_matcher: PatternMatcher<EventSnapshot>,
    active_sequences: HashMap<String, Vec<EventSnapshot>>,
}

#[derive(Debug, Clone)]
pub struct EventSnapshot {
    pub id: String,
    pub kind: EventKind,
    pub source: String,
    pub timestamp: Instant,
    pub data: serde_json::Value,
}

impl From<&dyn Event> for EventSnapshot {
    fn from(event: &dyn Event) -> Self {
        Self {
            id: event.id().to_string(),
            kind: event.kind(),
            source: event.source().to_string(),
            timestamp: Instant::now(),
            data: event.data().unwrap_or_default(),
        }
    }
}

impl EventSequenceMatcher {
    pub fn new() -> Self {
        let patterns = vec![
            // Pattern: DEX swap followed by arbitrage within 5 seconds
            EventPattern::Sequence(vec![
                |event: &EventSnapshot| {
                    event.kind == EventKind::Custom && 
                    event.data.get("event_type") == Some(&serde_json::Value::String("swap".to_string()))
                },
                |event: &EventSnapshot| {
                    event.kind == EventKind::Custom && 
                    event.data.get("event_type") == Some(&serde_json::Value::String("arbitrage".to_string()))
                },
            ]),
            
            // Pattern: High-value transaction (> $10k)
            EventPattern::Single(|event: &EventSnapshot| {
                if let Some(amount) = event.data.get("amount").and_then(|v| v.as_f64()) {
                    amount > 10_000.0
                } else {
                    false
                }
            }),
            
            // Pattern: Multiple transactions from same address within 1 minute
            EventPattern::Within {
                pattern: Box::new(EventPattern::Single(|event: &EventSnapshot| {
                    event.kind == EventKind::Transaction
                })),
                duration: Duration::from_secs(60),
            },
        ];

        Self {
            pattern_matcher: PatternMatcher::new(patterns, 1000), // Keep 1000 events in history
            active_sequences: HashMap::new(),
        }
    }

    pub fn match_event(&mut self, event: &dyn Event) -> Vec<usize> {
        let snapshot = EventSnapshot::from(event);
        let matches = self.pattern_matcher.match_event(snapshot.clone());
        
        if !matches.is_empty() {
            debug!("🎯 Pattern matches for event {}: {:?}", event.id(), matches);
        }

        matches
    }
}

/// Time-based event window processor
pub struct TimeWindowProcessor {
    window_manager: WindowManager<EventSnapshot>,
    window_handlers: HashMap<String, Box<dyn WindowHandler>>,
}

trait WindowHandler: Send + Sync {
    fn handle_window(&self, window: &Window<EventSnapshot>) -> ToolResult<()>;
}

struct SwapVolumeHandler;

impl WindowHandler for SwapVolumeHandler {
    fn handle_window(&self, window: &Window<EventSnapshot>) -> ToolResult<()> {
        let swap_events: Vec<_> = window.events
            .iter()
            .filter(|e| e.data.get("event_type") == Some(&serde_json::Value::String("swap".to_string())))
            .collect();

        if !swap_events.is_empty() {
            let total_volume: f64 = swap_events
                .iter()
                .filter_map(|e| e.data.get("amount").and_then(|v| v.as_f64()))
                .sum();

            info!("💹 Window {} - {} swaps, total volume: ${:.2}", 
                 window.id, swap_events.len(), total_volume);
        }

        Ok(())
    }
}

struct LiquidityChangeHandler;

impl WindowHandler for LiquidityChangeHandler {
    fn handle_window(&self, window: &Window<EventSnapshot>) -> ToolResult<()> {
        let liquidity_events: Vec<_> = window.events
            .iter()
            .filter(|e| e.data.get("event_type") == Some(&serde_json::Value::String("liquidity_change".to_string())))
            .collect();

        if !liquidity_events.is_empty() {
            info!("🌊 Window {} - {} liquidity events", window.id, liquidity_events.len());
        }

        Ok(())
    }
}

impl TimeWindowProcessor {
    pub fn new(window_type: WindowType) -> Self {
        let mut processor = Self {
            window_manager: WindowManager::new(window_type),
            window_handlers: HashMap::new(),
        };

        // Register window handlers
        processor.window_handlers.insert("swap_volume".to_string(), Box::new(SwapVolumeHandler));
        processor.window_handlers.insert("liquidity_changes".to_string(), Box::new(LiquidityChangeHandler));

        processor
    }

    pub fn process_event(&mut self, event: &dyn Event) -> ToolResult<()> {
        let snapshot = EventSnapshot::from(event);
        let completed_windows = self.window_manager.add_event(snapshot);

        // Process completed windows
        for window in completed_windows {
            debug!("⏰ Processing completed window {} with {} events", 
                  window.id, window.events.len());

            for (_name, handler) in &self.window_handlers {
                if let Err(e) = handler.handle_window(&window) {
                    warn!("Failed to process window with handler: {}", e);
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("debug,riglr_streams=trace")
        .init();

    info!("🎯 Starting Filtered Events Example");

    // Load configuration
    let config = Config::from_env();
    config.validate()?;

    // Create event router with custom filters
    let mut event_router = EventRouter::new();

    // Add high-value transaction filter
    event_router.add_filter(EventFilterRule {
        name: "high_value_transactions".to_string(),
        predicate: |event| {
            if let Some(data) = event.data() {
                if let Some(amount) = data.get("amount").and_then(|v| v.as_f64()) {
                    return amount > 1000.0;
                }
            }
            false
        },
        priority: 100,
        enabled: true,
    });

    // Add DEX swap filter
    event_router.add_filter(EventFilterRule {
        name: "dex_swaps".to_string(),
        predicate: |event| {
            event.kind() == EventKind::Custom &&
            event.data()
                .and_then(|d| d.get("event_type"))
                .and_then(|t| t.as_str()) == Some("swap")
        },
        priority: 80,
        enabled: true,
    });

    // Add error event filter
    event_router.add_filter(EventFilterRule {
        name: "error_events".to_string(),
        predicate: |event| {
            event.data()
                .and_then(|d| d.get("status"))
                .and_then(|s| s.as_str()) == Some("error")
        },
        priority: 90,
        enabled: true,
    });

    // Create pattern matcher for complex sequences
    let mut sequence_matcher = EventSequenceMatcher::new();

    // Create time window processor (5-second tumbling windows)
    let mut window_processor = TimeWindowProcessor::new(
        WindowType::Tumbling { duration: Duration::from_secs(5) }
    );

    // Simulate event processing
    let mut event_count = 0u64;
    let start_time = Instant::now();

    info!("🔄 Starting event processing simulation...");

    // Generate sample events for demonstration
    let sample_events = generate_sample_events();

    for event in sample_events {
        event_count += 1;

        // Route event through filters
        let matched_filters = event_router.route_event(&*event).await;
        if !matched_filters.is_empty() {
            info!("✅ Event {} matched filters: {:?}", event.id(), matched_filters);
        }

        // Check for pattern matches
        let pattern_matches = sequence_matcher.match_event(&*event);
        if !pattern_matches.is_empty() {
            info!("🎯 Event {} matched patterns: {:?}", event.id(), pattern_matches);
        }

        // Process through time windows
        if let Err(e) = window_processor.process_event(&*event) {
            warn!("Window processing error: {}", e);
        }

        // Add some delay to simulate real-time processing
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Display final statistics
    let stats = event_router.get_stats().await;
    info!("📊 Final Statistics:");
    info!("   Total events processed: {}", stats.total_events);
    info!("   Events dropped: {}", stats.dropped_events);
    
    for (filter_name, count) in &stats.filtered_events {
        let avg_processing_time = stats.processing_time_ms.get(filter_name).unwrap_or(&0) / count.max(1);
        info!("   Filter '{}': {} matches (avg: {}ms)", filter_name, count, avg_processing_time);
    }

    let elapsed = start_time.elapsed();
    info!("⏱️  Processing completed in {:?} ({:.2} events/sec)", 
         elapsed, event_count as f64 / elapsed.as_secs_f64());

    Ok(())
}

fn generate_sample_events() -> Vec<Box<dyn Event>> {
    let mut events: Vec<Box<dyn Event>> = Vec::new();

    // High-value transaction
    events.push(Box::new(GenericEvent::new(
        "tx_high_value_1".into(),
        EventKind::Transaction,
        serde_json::json!({
            "amount": 15000.0,
            "from": "alice",
            "to": "bob",
            "status": "success"
        }),
    )));

    // DEX swap event
    events.push(Box::new(GenericEvent::new(
        "swap_jupiter_1".into(),
        EventKind::Custom,
        serde_json::json!({
            "event_type": "swap",
            "protocol": "jupiter",
            "amount_in": 1000.0,
            "amount_out": 995.0,
            "fee": 5.0
        }),
    )));

    // Error event
    events.push(Box::new(GenericEvent::new(
        "tx_failed_1".into(),
        EventKind::Transaction,
        serde_json::json!({
            "amount": 500.0,
            "from": "charlie",
            "to": "dave",
            "status": "error",
            "error": "insufficient_funds"
        }),
    )));

    // Regular transaction (should be dropped)
    events.push(Box::new(GenericEvent::new(
        "tx_regular_1".into(),
        EventKind::Transaction,
        serde_json::json!({
            "amount": 50.0,
            "from": "eve",
            "to": "frank",
            "status": "success"
        }),
    )));

    // Liquidity change event
    events.push(Box::new(GenericEvent::new(
        "liquidity_1".into(),
        EventKind::Custom,
        serde_json::json!({
            "event_type": "liquidity_change",
            "protocol": "raydium",
            "pool": "SOL-USDC",
            "change": 10000.0
        }),
    )));

    // Arbitrage event (should match sequence pattern)
    events.push(Box::new(GenericEvent::new(
        "arbitrage_1".into(),
        EventKind::Custom,
        serde_json::json!({
            "event_type": "arbitrage",
            "profit": 125.50,
            "dexes": ["jupiter", "orca"]
        }),
    )));

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_router() {
        let mut router = EventRouter::new();

        router.add_filter(EventFilterRule {
            name: "test_filter".to_string(),
            predicate: |event| event.kind() == EventKind::Transaction,
            priority: 50,
            enabled: true,
        });

        let event = GenericEvent::new(
            "test".into(),
            EventKind::Transaction,
            serde_json::json!({"amount": 100}),
        );

        let matches = router.route_event(&event).await;
        assert_eq!(matches, vec!["test_filter"]);

        let stats = router.get_stats().await;
        assert_eq!(stats.total_events, 1);
        assert_eq!(*stats.filtered_events.get("test_filter").unwrap(), 1);
    }

    #[test]
    fn test_pattern_matcher() {
        let mut matcher = EventSequenceMatcher::new();

        let swap_event = GenericEvent::new(
            "swap".into(),
            EventKind::Custom,
            serde_json::json!({"event_type": "swap"}),
        );

        let matches = matcher.match_event(&swap_event);
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_time_window_processor() {
        let mut processor = TimeWindowProcessor::new(
            WindowType::Count { size: 2 }
        );

        let event1 = GenericEvent::new(
            "event1".into(),
            EventKind::Transaction,
            serde_json::json!({"amount": 100}),
        );

        let event2 = GenericEvent::new(
            "event2".into(),
            EventKind::Transaction,
            serde_json::json!({"amount": 200}),
        );

        assert!(processor.process_event(&event1).is_ok());
        assert!(processor.process_event(&event2).is_ok());
    }
}