//! Test module for AsNumeric trait
use std::any::Any;
use std::time::SystemTime;
use crate::core::financial_operators::{AsNumeric, FinancialEvent};
use riglr_events_core::prelude::{Event, EventKind, EventMetadata};
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct TestPriceEvent {
    pub metadata: EventMetadata,
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp: SystemTime,
}

impl Event for TestPriceEvent {
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
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
    
    fn to_json(&self) -> riglr_events_core::EventResult<serde_json::Value> {
        Ok(serde_json::json!({
            "metadata": self.metadata,
            "symbol": self.symbol,
            "price": self.price,
            "volume": self.volume,
            "timestamp": self.timestamp
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        }))
    }
}

impl AsNumeric for TestPriceEvent {
    fn as_price(&self) -> Option<f64> {
        Some(self.price)
    }
    
    fn as_volume(&self) -> Option<f64> {
        Some(self.volume)
    }
    
    fn as_timestamp_ms(&self) -> Option<i64> {
        Some(self.timestamp.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default().as_millis() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_numeric_trait() {
        let metadata = EventMetadata::new(
            "SOL/USDT".to_string(),
            EventKind::Price,
            "test".to_string(),
        );
        
        let event = TestPriceEvent {
            metadata,
            symbol: "SOL/USDT".to_string(),
            price: 150.0,
            volume: 1000.0,
            timestamp: SystemTime::now(),
        };
        
        assert_eq!(event.as_price(), Some(150.0));
        assert_eq!(event.as_volume(), Some(1000.0));
        assert!(event.as_timestamp_ms().is_some());
        assert_eq!(event.as_market_cap(), None); // Default implementation
    }
    
    #[test]
    fn test_financial_event_wrapper() {
        let metadata = EventMetadata::new(
            "SOL/USDT".to_string(),
            EventKind::Price,
            "test".to_string(),
        );
        
        let test_event = TestPriceEvent {
            metadata,
            symbol: "SOL/USDT".to_string(),
            price: 150.0,
            volume: 1000.0,
            timestamp: SystemTime::now(),
        };
        
        let metadata = EventMetadata::new(
            "vwap-test".to_string(),
            EventKind::Price,
            "test-vwap".to_string(),
        );
        
        let financial_event = FinancialEvent {
            metadata,
            indicator_value: 152.5, // VWAP calculation result
            original_event: std::sync::Arc::new(test_event),
            indicator_type: "VWAP".to_string(),
            timestamp: SystemTime::now(),
        };
        
        // The financial event should preserve the original event data
        assert_eq!(financial_event.original_event.as_price(), Some(150.0));
        assert_eq!(financial_event.indicator_value, 152.5);
        assert_eq!(financial_event.indicator_type, "VWAP");
    }
}