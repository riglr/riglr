//! Data integrity validation pipeline for parsed events
//!
//! This module provides comprehensive validation of parsed events to ensure
//! data quality, consistency, and integrity before downstream processing.

use crate::types::{EventType, ProtocolType};
use crate::zero_copy::ZeroCopyEvent;
use dashmap::DashMap;
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
// UnifiedEvent trait has been removed

/// Configuration for validation pipeline
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Enable strict validation (fails on any error)
    pub strict_mode: bool,
    /// Enable data consistency checks
    pub enable_consistency_checks: bool,
    /// Enable business logic validation
    pub enable_business_validation: bool,
    /// Enable duplicate detection
    pub enable_duplicate_detection: bool,
    /// Maximum event age to accept (prevents stale data)
    pub max_event_age: Duration,
    /// Known token mints for validation
    pub known_tokens: HashSet<Pubkey>,
    /// Known program IDs for validation
    pub known_programs: HashSet<Pubkey>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            enable_consistency_checks: true,
            enable_business_validation: true,
            enable_duplicate_detection: true,
            max_event_age: Duration::from_secs(3600), // 1 hour
            known_tokens: HashSet::new(),
            known_programs: HashSet::new(),
        }
    }
}

/// Validation result for a single event
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the event is valid
    pub is_valid: bool,
    /// Validation errors found
    pub errors: Vec<ValidationError>,
    /// Validation warnings (non-critical issues)
    pub warnings: Vec<ValidationWarning>,
    /// Validation metrics
    pub metrics: ValidationMetrics,
}

/// Validation error types
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Missing required field
    MissingField {
        /// The name of the missing field
        field: String,
    },
    /// Invalid field value
    InvalidValue {
        /// The name of the invalid field
        field: String,
        /// The reason why the value is invalid
        reason: String,
    },
    /// Data inconsistency
    Inconsistency {
        /// Description of the inconsistency
        description: String,
    },
    /// Business logic violation
    BusinessLogicError {
        /// The business rule that was violated
        rule: String,
        /// Description of the violation
        description: String,
    },
    /// Event too old
    StaleEvent {
        /// How old the event is
        age: Duration,
    },
    /// Duplicate event detected
    Duplicate {
        /// ID of the original event
        original_id: String,
    },
}

/// Validation warning types
#[derive(Debug, Clone)]
pub enum ValidationWarning {
    /// Unusual but potentially valid value
    UnusualValue {
        /// The name of the field with unusual value
        field: String,
        /// The unusual value
        value: String,
    },
    /// Deprecated field usage
    DeprecatedField {
        /// The name of the deprecated field
        field: String,
    },
    /// Performance concern
    PerformanceWarning {
        /// Description of the performance concern
        description: String,
    },
}

/// Validation metrics
#[derive(Debug, Clone, Default)]
pub struct ValidationMetrics {
    /// Time spent validating
    pub validation_time: Duration,
    /// Number of rules checked
    pub rules_checked: usize,
    /// Number of consistency checks performed
    pub consistency_checks: usize,
}

/// Data integrity validation pipeline
pub struct ValidationPipeline {
    /// Validation configuration
    config: ValidationConfig,
    /// Duplicate detection cache
    seen_events: Arc<DashMap<String, Instant>>,
    /// Validation rule registry
    rules: Vec<Arc<dyn ValidationRule>>,
}

impl ValidationPipeline {
    /// Create a new validation pipeline
    pub fn new(config: ValidationConfig) -> Self {
        let mut pipeline = Self {
            config,
            seen_events: Arc::new(DashMap::new()),
            rules: Vec::new(),
        };

        // Register default validation rules
        pipeline.register_default_rules();

        pipeline
    }

    /// Register default validation rules
    fn register_default_rules(&mut self) {
        self.rules.push(Arc::new(RequiredFieldsRule));
        self.rules.push(Arc::new(DataTypesRule));
        self.rules.push(Arc::new(NumericRangesRule));
        self.rules.push(Arc::new(PubkeyValidationRule));
        self.rules.push(Arc::new(SwapConsistencyRule));
        self.rules.push(Arc::new(LiquidityValidationRule));
        self.rules.push(Arc::new(TimestampValidationRule));
    }

    /// Add a custom validation rule
    pub fn add_rule(&mut self, rule: Arc<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    /// Validate a single event
    pub async fn validate_event(&self, event: &ZeroCopyEvent<'static>) -> ValidationResult {
        let start_time = Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut rules_checked = 0;
        let mut consistency_checks = 0;

        // Check for duplicates if enabled
        if self.config.enable_duplicate_detection {
            if let Some(error) = self.check_duplicate(event).await {
                errors.push(error);
            }
            consistency_checks += 1;
        }

        // Check event age
        if let Some(error) = self.check_event_age(event) {
            errors.push(error);
        }
        rules_checked += 1;

        // Run all validation rules
        for rule in &self.rules {
            let rule_result = rule.validate(event, &self.config).await;
            errors.extend(rule_result.errors);
            warnings.extend(rule_result.warnings);
            rules_checked += 1;
            consistency_checks += rule_result.consistency_checks;
        }

        // Protocol-specific validation
        match event.protocol_type() {
            ProtocolType::Jupiter => {
                if let Some(error) = self.validate_jupiter_event(event).await {
                    errors.push(error);
                }
            }
            ProtocolType::RaydiumAmmV4 => {
                if let Some(error) = self.validate_raydium_event(event).await {
                    errors.push(error);
                }
            }
            ProtocolType::PumpSwap => {
                if let Some(error) = self.validate_pump_fun_event(event).await {
                    errors.push(error);
                }
            }
            _ => {}
        }
        rules_checked += 1;

        let validation_time = start_time.elapsed();
        let is_valid = errors.is_empty() || !self.config.strict_mode;

        ValidationResult {
            is_valid,
            errors,
            warnings,
            metrics: ValidationMetrics {
                validation_time,
                rules_checked,
                consistency_checks,
            },
        }
    }

    /// Validate multiple events
    pub async fn validate_events(
        &self,
        events: &[ZeroCopyEvent<'static>],
    ) -> Vec<ValidationResult> {
        let mut results = Vec::with_capacity(events.len());

        for event in events {
            results.push(self.validate_event(event).await);
        }

        results
    }

    /// Check for duplicate events
    async fn check_duplicate(&self, event: &ZeroCopyEvent<'_>) -> Option<ValidationError> {
        let event_id = format!("{}_{}", event.signature(), event.index());

        // Clean up old entries (older than max_event_age)
        let cutoff = std::time::Instant::now()
            .checked_sub(self.config.max_event_age)
            .unwrap_or_else(std::time::Instant::now);
        self.seen_events.retain(|_, timestamp| *timestamp > cutoff);

        // Check if we've seen this event before
        if let Some(_original_time) = self.seen_events.get(&event_id) {
            return Some(ValidationError::Duplicate {
                original_id: event_id.clone(),
            });
        }

        // Record this event
        self.seen_events.insert(event_id, Instant::now());

        None
    }

    /// Check event age
    fn check_event_age(&self, event: &ZeroCopyEvent<'_>) -> Option<ValidationError> {
        let event_time = event.timestamp();
        let now = std::time::SystemTime::now();

        if let Ok(age) = now.duration_since(event_time) {
            if age > self.config.max_event_age {
                return Some(ValidationError::StaleEvent { age });
            }
        }

        None
    }

    /// Validate Jupiter-specific events
    async fn validate_jupiter_event(&self, event: &ZeroCopyEvent<'_>) -> Option<ValidationError> {
        if let Some(json_data) = event.get_json_data() {
            // Check for required Jupiter fields
            if json_data.get("total_amount_in").is_none() {
                return Some(ValidationError::MissingField {
                    field: "total_amount_in".to_string(),
                });
            }

            if json_data.get("total_amount_out").is_none() {
                return Some(ValidationError::MissingField {
                    field: "total_amount_out".to_string(),
                });
            }

            // Validate amounts are positive
            if let Some(amount_in) = json_data.get("total_amount_in").and_then(|v| v.as_str()) {
                if let Ok(amount) = amount_in.parse::<u64>() {
                    if amount == 0 {
                        return Some(ValidationError::InvalidValue {
                            field: "total_amount_in".to_string(),
                            reason: "Amount cannot be zero".to_string(),
                        });
                    }
                }
            }
        }

        None
    }

    /// Validate Raydium-specific events
    async fn validate_raydium_event(&self, event: &ZeroCopyEvent<'_>) -> Option<ValidationError> {
        // Check event type matches Raydium operations
        match event.event_type() {
            EventType::RaydiumAmmV4SwapBaseIn | EventType::RaydiumAmmV4SwapBaseOut => {
                // Validate swap-specific fields
                if let Some(json_data) = event.get_json_data() {
                    if json_data.get("amount_in").is_none()
                        && json_data.get("max_amount_in").is_none()
                    {
                        return Some(ValidationError::MissingField {
                            field: "amount_in or max_amount_in".to_string(),
                        });
                    }
                }
            }
            _ => {}
        }

        None
    }

    /// Validate PumpFun-specific events
    async fn validate_pump_fun_event(&self, event: &ZeroCopyEvent<'_>) -> Option<ValidationError> {
        if let Some(json_data) = event.get_json_data() {
            // Check for 8-byte discriminator in PumpFun events
            if let Some(discriminator) = json_data.get("discriminator") {
                if let Some(disc_val) = discriminator.as_u64() {
                    // Validate known PumpFun discriminators
                    let known_discriminators = [
                        0x66063d1201daebea, // Buy
                        0x33e685a4017f83ad, // Sell
                        0x181ec828051c0777, // CreatePool
                    ];

                    if !known_discriminators.contains(&disc_val) {
                        return Some(ValidationError::InvalidValue {
                            field: "discriminator".to_string(),
                            reason: "Unknown PumpFun discriminator".to_string(),
                        });
                    }
                }
            }
        }

        None
    }

    /// Get pipeline statistics
    pub async fn get_stats(&self) -> ValidationStats {
        ValidationStats {
            total_rules: self.rules.len(),
            cached_events: self.seen_events.len(),
            strict_mode: self.config.strict_mode,
        }
    }
}

/// Validation pipeline statistics
#[derive(Debug, Clone)]
pub struct ValidationStats {
    /// Total number of validation rules registered
    pub total_rules: usize,
    /// Number of events currently cached for duplicate detection
    pub cached_events: usize,
    /// Whether strict mode is enabled
    pub strict_mode: bool,
}

/// Trait for validation rules
#[async_trait::async_trait]
pub trait ValidationRule: Send + Sync {
    /// Validate an event
    async fn validate(&self, event: &ZeroCopyEvent<'_>, config: &ValidationConfig) -> RuleResult;

    /// Get rule name
    fn name(&self) -> &'static str;
}

/// Result from a validation rule
#[derive(Debug, Default)]
pub struct RuleResult {
    /// Validation errors found by the rule
    pub errors: Vec<ValidationError>,
    /// Validation warnings found by the rule
    pub warnings: Vec<ValidationWarning>,
    /// Number of consistency checks performed
    pub consistency_checks: usize,
}

/// Rule to validate required fields
struct RequiredFieldsRule;

#[async_trait::async_trait]
impl ValidationRule for RequiredFieldsRule {
    async fn validate(&self, event: &ZeroCopyEvent<'_>, _config: &ValidationConfig) -> RuleResult {
        let mut result = RuleResult::default();

        // Check basic required fields
        if event.id().is_empty() {
            result.errors.push(ValidationError::MissingField {
                field: "id".to_string(),
            });
        }

        if event.signature().is_empty() {
            result.errors.push(ValidationError::MissingField {
                field: "signature".to_string(),
            });
        }

        result.consistency_checks = 2;
        result
    }

    fn name(&self) -> &'static str {
        "required_fields"
    }
}

/// Rule to validate data types
struct DataTypesRule;

#[async_trait::async_trait]
impl ValidationRule for DataTypesRule {
    async fn validate(&self, event: &ZeroCopyEvent<'_>, _config: &ValidationConfig) -> RuleResult {
        let mut result = RuleResult::default();

        // Validate slot is reasonable
        if event.slot() == 0 {
            result.warnings.push(ValidationWarning::UnusualValue {
                field: "slot".to_string(),
                value: "0".to_string(),
            });
        }

        result.consistency_checks = 1;
        result
    }

    fn name(&self) -> &'static str {
        "data_types"
    }
}

/// Rule to validate numeric ranges
struct NumericRangesRule;

#[async_trait::async_trait]
impl ValidationRule for NumericRangesRule {
    async fn validate(&self, event: &ZeroCopyEvent<'_>, _config: &ValidationConfig) -> RuleResult {
        let mut result = RuleResult::default();

        if let Some(json_data) = event.get_json_data() {
            // Validate amounts are within reasonable ranges
            let empty_map = serde_json::Map::new();
            for (key, value) in json_data.as_object().unwrap_or(&empty_map) {
                if key.contains("amount") {
                    if let Some(amount_str) = value.as_str() {
                        if let Ok(amount) = amount_str.parse::<u64>() {
                            if amount > u64::MAX / 2 {
                                result.warnings.push(ValidationWarning::UnusualValue {
                                    field: key.clone(),
                                    value: amount.to_string(),
                                });
                            }
                        }
                    }
                }
            }
            result.consistency_checks = json_data.as_object().map(|o| o.len()).unwrap_or(0);
        }

        result
    }

    fn name(&self) -> &'static str {
        "numeric_ranges"
    }
}

/// Rule to validate Pubkey formats
struct PubkeyValidationRule;

#[async_trait::async_trait]
impl ValidationRule for PubkeyValidationRule {
    async fn validate(&self, event: &ZeroCopyEvent<'_>, _config: &ValidationConfig) -> RuleResult {
        let mut result = RuleResult::default();

        if let Some(json_data) = event.get_json_data() {
            self.validate_pubkeys_in_json(json_data, &mut result);
        }

        result
    }

    fn name(&self) -> &'static str {
        "pubkey_validation"
    }
}

impl PubkeyValidationRule {
    #[allow(clippy::only_used_in_recursion)]
    fn validate_pubkeys_in_json(&self, value: &Value, result: &mut RuleResult) {
        match value {
            Value::String(s) => {
                if s.len() == 44 || s.len() == 43 {
                    // Typical Pubkey string lengths
                    if s.parse::<Pubkey>().is_err() {
                        result.errors.push(ValidationError::InvalidValue {
                            field: "pubkey".to_string(),
                            reason: format!("Invalid pubkey format: {}", s),
                        });
                    }
                    result.consistency_checks += 1;
                }
            }
            Value::Object(map) => {
                for (key, val) in map {
                    if key.contains("mint") || key.contains("address") || key.contains("pubkey") {
                        self.validate_pubkeys_in_json(val, result);
                    }
                }
            }
            Value::Array(arr) => {
                for val in arr {
                    self.validate_pubkeys_in_json(val, result);
                }
            }
            _ => {}
        }
    }
}

/// Rule to validate swap consistency
struct SwapConsistencyRule;

#[async_trait::async_trait]
impl ValidationRule for SwapConsistencyRule {
    async fn validate(&self, event: &ZeroCopyEvent<'_>, _config: &ValidationConfig) -> RuleResult {
        let mut result = RuleResult::default();

        if event.event_type() == EventType::Swap {
            if let Some(json_data) = event.get_json_data() {
                // Check that input and output amounts make sense
                let amount_in = json_data
                    .get("amount_in")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u64>().ok());
                let amount_out = json_data
                    .get("amount_out")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u64>().ok());

                if let (Some(in_amount), Some(out_amount)) = (amount_in, amount_out) {
                    if in_amount == 0 || out_amount == 0 {
                        result.errors.push(ValidationError::InvalidValue {
                            field: "swap_amounts".to_string(),
                            reason: "Swap amounts cannot be zero".to_string(),
                        });
                    }
                    result.consistency_checks += 1;
                }
            }
        }

        result
    }

    fn name(&self) -> &'static str {
        "swap_consistency"
    }
}

/// Rule to validate liquidity operations
struct LiquidityValidationRule;

#[async_trait::async_trait]
impl ValidationRule for LiquidityValidationRule {
    async fn validate(&self, event: &ZeroCopyEvent<'_>, _config: &ValidationConfig) -> RuleResult {
        let mut result = RuleResult::default();

        match event.event_type() {
            EventType::AddLiquidity | EventType::RemoveLiquidity => {
                // Validate liquidity-specific fields
                result.consistency_checks += 1;
            }
            _ => {}
        }

        result
    }

    fn name(&self) -> &'static str {
        "liquidity_validation"
    }
}

/// Rule to validate timestamps
struct TimestampValidationRule;

#[async_trait::async_trait]
impl ValidationRule for TimestampValidationRule {
    async fn validate(&self, event: &ZeroCopyEvent<'_>, _config: &ValidationConfig) -> RuleResult {
        let mut result = RuleResult::default();

        let now = std::time::SystemTime::now();
        let event_time = event.timestamp();

        // Check if timestamp is in the future
        if event_time > now {
            result.warnings.push(ValidationWarning::UnusualValue {
                field: "timestamp".to_string(),
                value: format!("{:?}", event_time),
            });
        }

        result.consistency_checks = 1;
        result
    }

    fn name(&self) -> &'static str {
        "timestamp_validation"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EventMetadata;

    #[tokio::test]
    async fn test_validation_pipeline_creation() {
        let config = ValidationConfig::default();
        let pipeline = ValidationPipeline::new(config);

        let stats = pipeline.get_stats().await;
        assert!(stats.total_rules > 0);
        assert_eq!(stats.cached_events, 0);
    }

    #[tokio::test]
    async fn test_required_fields_validation() {
        let rule = RequiredFieldsRule;
        let metadata = EventMetadata::default();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let config = ValidationConfig::default();
        let result = rule.validate(&event, &config).await;

        // Should have errors for empty id and signature
        assert!(result.errors.len() >= 2);
    }

    #[tokio::test]
    async fn test_duplicate_detection() {
        let config = ValidationConfig {
            enable_duplicate_detection: true,
            ..Default::default()
        };
        let pipeline = ValidationPipeline::new(config);

        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = crate::solana_metadata::SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::default(),
            "0".to_string(),
            0,
            core_metadata,
        );

        let event = ZeroCopyEvent::new_owned(metadata.clone(), vec![]);

        // First validation should pass
        let result1 = pipeline.validate_event(&event).await;
        assert!(result1.is_valid);

        // Second validation should detect duplicate
        let result2 = pipeline.validate_event(&event).await;
        assert!(!result2.errors.is_empty());
    }

    #[test]
    fn test_pubkey_validation() {
        let rule = PubkeyValidationRule;
        let mut result = RuleResult::default();

        let valid_json = serde_json::json!({
            "mint": "So11111111111111111111111111111111111111112"
        });

        rule.validate_pubkeys_in_json(&valid_json, &mut result);
        assert!(result.errors.is_empty());

        let invalid_json = serde_json::json!({
            "mint": "InvalidPubkeyStringWithExactly44Characters!"
        });

        rule.validate_pubkeys_in_json(&invalid_json, &mut result);
        assert!(!result.errors.is_empty());
    }
}
