//! Test module for AsNumeric trait
use std::any::Any;
use std::time::SystemTime;
use crate::core::financial_operators::{AsNumeric, FinancialEvent};
use riglr_solana_events::{UnifiedEvent, EventType};

#[derive(Debug, Clone)]
pub struct TestPriceEvent {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp: SystemTime,
}

impl UnifiedEvent for TestPriceEvent {
    fn id(&self) -> &str {
        &self.symbol
    }
    
    fn event_type(&self) -> EventType {
        EventType::PriceUpdate
    }
    
    fn signature(&self) -> &str {
        "test-price-event"
    }
    
    fn slot(&self) -> u64 {
        0 // Test events don't have slots
    }
    
    fn program_received_time_ms(&self) -> i64 {
        self.timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }
    
    fn program_handle_time_consuming_ms(&self) -> i64 {
        0 // No processing time for test events
    }
    
    fn set_program_handle_time_consuming_ms(&mut self, _time: i64) {
        // No-op for test events
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    fn clone_boxed(&self) -> Box<dyn UnifiedEvent> {
        Box::new(self.clone())
    }
    
    fn set_transfer_data(
        &mut self,
        _transfer_data: Vec<riglr_solana_events::TransferData>,
        _swap_data: Option<riglr_solana_events::SwapData>,
    ) {
        // No-op for test events
    }
    
    fn index(&self) -> String {
        "0".to_string()
    }
    
    fn protocol_type(&self) -> riglr_solana_events::ProtocolType {
        riglr_solana_events::ProtocolType::Other("Test".to_string())
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
        let event = TestPriceEvent {
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
        let test_event = TestPriceEvent {
            symbol: "SOL/USDT".to_string(),
            price: 150.0,
            volume: 1000.0,
            timestamp: SystemTime::now(),
        };
        
        let financial_event = FinancialEvent {
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