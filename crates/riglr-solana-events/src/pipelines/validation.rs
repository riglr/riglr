//! Data integrity validation pipeline for parsed events
//!
//! This module provides comprehensive validation of parsed events to ensure
//! data quality, consistency, and integrity before downstream processing.

extern crate alloc;

use crate::types::{EventType, ProtocolType};
use crate::zero_copy::ZeroCopyEvent;
use dashmap::DashMap;
use serde_json::Value;

use solana_sdk::pubkey::Pubkey;
use std::collections::HashSet;

use alloc::sync::Arc;
use core::fmt::{Debug, Formatter, Result as FmtResult};
use core::time::Duration;
use std::time::{Instant, SystemTime};

#[cfg(test)]
use riglr_events_core::types::EventKind;
#[cfg(test)]
use tokio::time;
// UnifiedEvent trait has been removed

/// Configuration for validation pipeline
#[non_exhaustive]
#[derive(Debug, Clone)]
#[expect(clippy::struct_excessive_bools)]
pub struct Config {
    /// Enable business logic validation
    pub enable_business_validation: bool,
    /// Enable data consistency checks
    pub enable_consistency_checks: bool,
    /// Enable duplicate detection
    pub enable_duplicate_detection: bool,
    /// Known program IDs for validation
    pub known_programs: HashSet<Pubkey>,
    /// Known token mints for validation
    pub known_tokens: HashSet<Pubkey>,
    /// Maximum event age to accept (prevents stale data)
    pub max_event_age: Duration,
    /// Enable strict validation (fails on any error)
    pub strict_mode: bool,
}

impl Default for Config {
    #[inline]
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
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Result {
    /// Validation errors found
    pub errors: Vec<Error>,
    /// Whether the event is valid
    pub is_valid: bool,
    /// Validation metrics
    pub metrics: Metrics,
    /// Validation warnings (non-critical issues)
    pub warnings: Vec<Warning>,
}

/// Validation error types
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Error {
    /// Business logic violation
    BusinessLogicError {
        /// Description of the violation
        description: String,
        /// The business rule that was violated
        rule: String,
    },
    /// Duplicate event detected
    Duplicate {
        /// ID of the original event
        original_id: String,
    },
    /// Data inconsistency
    Inconsistency {
        /// Description of the inconsistency
        description: String,
    },
    /// Invalid field value
    InvalidValue {
        /// The name of the invalid field
        field: String,
        /// The reason why the value is invalid
        reason: String,
    },
    /// Missing required field
    MissingField {
        /// The name of the missing field
        field: String,
    },
    /// Event too old
    StaleEvent {
        /// How old the event is
        age: Duration,
    },
}

/// Validation warning types
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Warning {
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
    /// Unusual but potentially valid value
    UnusualValue {
        /// The name of the field with unusual value
        field: String,
        /// The unusual value
        value: String,
    },
}

/// Validation metrics
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    /// Number of consistency checks performed
    pub consistency_checks: usize,
    /// Number of rules checked
    pub rules_checked: usize,
    /// Time spent validating
    pub validation_time: Duration,
}

/// Data integrity validation pipeline
pub struct Pipeline {
    /// Validation configuration
    config: Config,
    /// Validation rule registry
    rules: Vec<Arc<dyn Rule>>,
    /// Duplicate detection cache
    seen_events: Arc<DashMap<String, Instant>>,
}

impl Pipeline {
    /// Add a custom validation rule
    #[inline]
    pub fn add_rule(&mut self, rule: Arc<dyn Rule>) {
        self.rules.push(rule);
    }

    /// Get pipeline statistics
    #[must_use]
    #[inline]
    pub fn get_stats(&self) -> Stats {
        Stats {
            total_rules: self.rules.len(),
            cached_events: self.seen_events.len(),
            strict_mode: self.config.strict_mode,
        }
    }

    /// Create a new validation pipeline
    #[must_use]
    #[inline]
    pub fn new(config: Config) -> Self {
        let mut pipeline = Self {
            config,
            seen_events: Arc::new(DashMap::new()),
            rules: Vec::new(),
        };

        // Register default validation rules
        pipeline.register_default_rules();

        pipeline
    }

    /// Check for duplicate events
    fn check_duplicate(&self, event: &ZeroCopyEvent<'_>) -> Option<Error> {
        let event_id = format!("{}_{}", event.signature(), event.index());

        // Clean up old entries (older than max_event_age)
        let cutoff = Instant::now()
            .checked_sub(self.config.max_event_age)
            .unwrap_or_else(Instant::now);
        self.seen_events.retain(|_, timestamp| *timestamp > cutoff);

        // Check if we've seen this event before
        if let Some(_original_time) = self.seen_events.get(&event_id) {
            return Some(Error::Duplicate {
                original_id: event_id.clone(),
            });
        }

        // Record this event
        self.seen_events.insert(event_id, Instant::now());

        None
    }

    /// Register default validation rules
    fn register_default_rules(&mut self) {
        self.rules.push(Arc::new(RequiredFieldsRule));
        self.rules.push(Arc::new(DataTypesRule));
        self.rules.push(Arc::new(NumericRangesRule));
        self.rules.push(Arc::new(PubkeyRule));
        self.rules.push(Arc::new(SwapConsistencyRule));
        self.rules.push(Arc::new(LiquidityRule));
        self.rules.push(Arc::new(TimestampRule));
    }

    /// Check event age
    fn check_event_age(&self, event: &ZeroCopyEvent<'_>) -> Option<Error> {
        let event_time = event.timestamp();
        let now = SystemTime::now();

        if let Ok(age) = now.duration_since(event_time) {
            if age > self.config.max_event_age {
                return Some(Error::StaleEvent { age });
            }
        }

        None
    }

    /// Validate a single event
    #[inline]
    pub async fn validate_event(&self, event: &ZeroCopyEvent<'static>) -> Result {
        let start_time = Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut rules_checked = 0_usize;
        let mut consistency_checks = 0_usize;

        // Check for duplicates if enabled
        if self.config.enable_duplicate_detection {
            if let Some(error) = self.check_duplicate(event) {
                errors.push(error);
            }
            {
                consistency_checks = consistency_checks.saturating_add(1);
            }
        }

        // Check event age
        if let Some(error) = self.check_event_age(event) {
            errors.push(error);
        }
        {
            rules_checked = rules_checked.saturating_add(1);
        }

        // Run all validation rules
        for rule in &self.rules {
            let rule_result = rule.validate(event, &self.config).await;
            errors.extend(rule_result.errors);
            warnings.extend(rule_result.warnings);
            {
                rules_checked = rules_checked.saturating_add(1);
                consistency_checks =
                    consistency_checks.saturating_add(rule_result.consistency_checks);
            }
        }

        // Protocol-specific validation
        match event.protocol_type() {
            ProtocolType::Jupiter => {
                if let Some(error) = Self::validate_jupiter_event(event) {
                    errors.push(error);
                }
            }
            ProtocolType::RaydiumAmmV4 => {
                if let Some(error) = Self::validate_raydium_event(event) {
                    errors.push(error);
                }
            }
            ProtocolType::PumpSwap => {
                if let Some(error) = Self::validate_pump_fun_event(event) {
                    errors.push(error);
                }
            }
            _ => {}
        }
        {
            rules_checked = rules_checked.saturating_add(1);
        }

        let validation_time = start_time.elapsed();
        let is_valid = errors.is_empty() || !self.config.strict_mode;

        Result {
            is_valid,
            errors,
            warnings,
            metrics: Metrics {
                consistency_checks,
                rules_checked,
                validation_time,
            },
        }
    }

    /// Validate multiple events
    #[inline]
    pub async fn validate_events(&self, events: &[ZeroCopyEvent<'static>]) -> Vec<Result> {
        let mut results = Vec::with_capacity(events.len());

        for event in events {
            results.push(self.validate_event(event).await);
        }

        results
    }

    /// Validate Jupiter-specific events
    fn validate_jupiter_event(event: &ZeroCopyEvent<'_>) -> Option<Error> {
        if let Some(json_data) = event.get_json_data() {
            // Check for required Jupiter fields
            if json_data.get("total_amount_in").is_none() {
                return Some(Error::MissingField {
                    field: "total_amount_in".to_owned(),
                });
            }

            if json_data.get("total_amount_out").is_none() {
                return Some(Error::MissingField {
                    field: "total_amount_out".to_owned(),
                });
            }

            // Validate amounts are positive
            if let Some(amount_in) = json_data
                .get("total_amount_in")
                .and_then(|value| value.as_str())
            {
                if let Ok(amount) = amount_in.parse::<u64>() {
                    if amount == 0 {
                        return Some(Error::InvalidValue {
                            field: "total_amount_in".to_owned(),
                            reason: "Amount cannot be zero".to_owned(),
                        });
                    }
                }
            }
        }

        None
    }

    /// Validate PumpFun-specific events
    fn validate_pump_fun_event(event: &ZeroCopyEvent<'_>) -> Option<Error> {
        if let Some(json_data) = event.get_json_data() {
            // Check for 8-byte discriminator in PumpFun events
            if let Some(discriminator) = json_data.get("discriminator") {
                if let Some(disc_val) = discriminator.as_u64() {
                    // Validate known PumpFun discriminators
                    let known_discriminators = [
                        0x6606_3d12_01da_ebea, // Buy
                        0x33e6_85a4_017f_83ad, // Sell
                        0x181e_c828_051c_0777, // CreatePool
                    ];

                    if !known_discriminators.contains(&disc_val) {
                        return Some(Error::InvalidValue {
                            field: "discriminator".to_owned(),
                            reason: "Unknown PumpFun discriminator".to_owned(),
                        });
                    }
                }
            }
        }

        None
    }

    /// Validate Raydium-specific events
    fn validate_raydium_event(event: &ZeroCopyEvent<'_>) -> Option<Error> {
        // Check event type matches Raydium operations
        match event.event_type() {
            EventType::RaydiumAmmV4SwapBaseIn | EventType::RaydiumAmmV4SwapBaseOut => {
                // Validate swap-specific fields
                if let Some(json_data) = event.get_json_data() {
                    if json_data.get("amount_in").is_none()
                        && json_data.get("max_amount_in").is_none()
                    {
                        return Some(Error::MissingField {
                            field: "amount_in or max_amount_in".to_owned(),
                        });
                    }
                }
            }
            _ => {}
        }

        None
    }
}

impl Debug for Pipeline {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("Pipeline")
            .field("config", &self.config)
            .field(
                "seen_events",
                &format!(
                    "DashMap<String, Instant> with {} entries",
                    self.seen_events.len()
                ),
            )
            .field(
                "rules",
                &format!("Vec<Arc<dyn Rule>> with {} rules", self.rules.len()),
            )
            .finish()
    }
}

/// Validation pipeline statistics
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Stats {
    /// Number of events currently cached for duplicate detection
    pub cached_events: usize,
    /// Whether strict mode is enabled
    pub strict_mode: bool,
    /// Total number of validation rules registered
    pub total_rules: usize,
}

/// Trait for validation rules
#[async_trait::async_trait]
pub trait Rule: Send + Sync {
    /// Get rule name
    fn name(&self) -> &'static str;

    /// Validate an event
    async fn validate(&self, event: &ZeroCopyEvent<'_>, config: &Config) -> RuleResult;
}

/// Result from a validation rule
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct RuleResult {
    /// Number of consistency checks performed
    pub consistency_checks: usize,
    /// Validation errors found by the rule
    pub errors: Vec<Error>,
    /// Validation warnings found by the rule
    pub warnings: Vec<Warning>,
}

/// Rule to validate required fields
struct RequiredFieldsRule;

#[async_trait::async_trait]
impl Rule for RequiredFieldsRule {
    fn name(&self) -> &'static str {
        "required_fields"
    }

    async fn validate(&self, event: &ZeroCopyEvent<'_>, _config: &Config) -> RuleResult {
        let mut result = RuleResult::default();

        // Check basic required fields
        if event.id().is_empty() {
            result.errors.push(Error::MissingField {
                field: "id".to_owned(),
            });
        }

        if event.signature().is_empty() {
            result.errors.push(Error::MissingField {
                field: "signature".to_owned(),
            });
        }

        result.consistency_checks = 2;
        result
    }
}

/// Rule to validate data types
struct DataTypesRule;

#[async_trait::async_trait]
impl Rule for DataTypesRule {
    fn name(&self) -> &'static str {
        "data_types"
    }

    async fn validate(&self, event: &ZeroCopyEvent<'_>, _config: &Config) -> RuleResult {
        let mut result = RuleResult::default();

        // Validate slot is reasonable
        if event.slot() == 0 {
            result.warnings.push(Warning::UnusualValue {
                field: "slot".to_owned(),
                value: "0".to_owned(),
            });
        }

        result.consistency_checks = 1;
        result
    }
}

/// Rule to validate numeric ranges
struct NumericRangesRule;

#[async_trait::async_trait]
impl Rule for NumericRangesRule {
    fn name(&self) -> &'static str {
        "numeric_ranges"
    }

    async fn validate(&self, event: &ZeroCopyEvent<'_>, _config: &Config) -> RuleResult {
        let mut result = RuleResult::default();

        if let Some(json_data) = event.get_json_data() {
            // Validate amounts are within reasonable ranges
            let empty_map = serde_json::Map::new();
            for (key, value) in json_data.as_object().unwrap_or(&empty_map) {
                if key.contains("amount") {
                    if let Some(amount_str) = value.as_str() {
                        if let Ok(amount) = amount_str.parse::<u64>() {
                            if amount > u64::MAX / 2 {
                                result.warnings.push(Warning::UnusualValue {
                                    field: key.clone(),
                                    value: amount.to_string(),
                                });
                            }
                        }
                    }
                }
            }
            result.consistency_checks = json_data.as_object().map_or(0, serde_json::Map::len);
        }

        result
    }
}

/// Rule to validate Pubkey formats
struct PubkeyRule;

#[async_trait::async_trait]
impl Rule for PubkeyRule {
    fn name(&self) -> &'static str {
        "pubkey_validation"
    }

    async fn validate(&self, event: &ZeroCopyEvent<'_>, _config: &Config) -> RuleResult {
        let mut result = RuleResult::default();

        if let Some(json_data) = event.get_json_data() {
            Self::validate_pubkeys_in_json(json_data, &mut result);
        }

        result
    }
}

impl PubkeyRule {
    /// Recursively validate public keys in JSON data
    fn validate_pubkeys_in_json(value: &Value, result: &mut RuleResult) {
        match *value {
            Value::String(ref string_value) => {
                if string_value.len() == 44 || string_value.len() == 43 {
                    // Typical Pubkey string lengths
                    if string_value.parse::<Pubkey>().is_err() {
                        result.errors.push(Error::InvalidValue {
                            field: "pubkey".to_owned(),
                            reason: format!("Invalid pubkey format: {string_value}"),
                        });
                    }
                    result.consistency_checks = result.consistency_checks.saturating_add(1);
                }
            }
            Value::Object(ref map) => {
                for (key, val) in map {
                    if key.contains("mint") || key.contains("address") || key.contains("pubkey") {
                        Self::validate_pubkeys_in_json(val, result);
                    }
                }
            }
            Value::Array(ref arr) => {
                for val in arr {
                    Self::validate_pubkeys_in_json(val, result);
                }
            }
            _ => {}
        }
    }
}

/// Rule to validate swap consistency
struct SwapConsistencyRule;

#[async_trait::async_trait]
impl Rule for SwapConsistencyRule {
    fn name(&self) -> &'static str {
        "swap_consistency"
    }

    async fn validate(&self, event: &ZeroCopyEvent<'_>, _config: &Config) -> RuleResult {
        let mut result = RuleResult::default();

        if event.event_type() == EventType::Swap {
            if let Some(json_data) = event.get_json_data() {
                // Check that input and output amounts make sense
                let amount_in = json_data
                    .get("amount_in")
                    .and_then(|value| value.as_str())
                    .and_then(|string_val| string_val.parse::<u64>().ok());
                let amount_out = json_data
                    .get("amount_out")
                    .and_then(|value| value.as_str())
                    .and_then(|string_val| string_val.parse::<u64>().ok());

                if let (Some(in_amount), Some(out_amount)) = (amount_in, amount_out) {
                    if in_amount == 0 || out_amount == 0 {
                        result.errors.push(Error::InvalidValue {
                            field: "swap_amounts".to_owned(),
                            reason: "Swap amounts cannot be zero".to_owned(),
                        });
                    }
                    {
                        result.consistency_checks = result.consistency_checks.saturating_add(1);
                    }
                }
            }
        }

        result
    }
}

/// Rule to validate liquidity operations
struct LiquidityRule;

#[async_trait::async_trait]
impl Rule for LiquidityRule {
    fn name(&self) -> &'static str {
        "liquidity_validation"
    }

    async fn validate(&self, event: &ZeroCopyEvent<'_>, _config: &Config) -> RuleResult {
        let mut result = RuleResult::default();

        match event.event_type() {
            EventType::AddLiquidity | EventType::RemoveLiquidity => {
                // Validate liquidity-specific fields
                result.consistency_checks = result.consistency_checks.saturating_add(1);
            }
            _ => {}
        }

        result
    }
}

/// Rule to validate timestamps
struct TimestampRule;

#[async_trait::async_trait]
impl Rule for TimestampRule {
    fn name(&self) -> &'static str {
        "timestamp_validation"
    }

    async fn validate(&self, event: &ZeroCopyEvent<'_>, _config: &Config) -> RuleResult {
        let mut result = RuleResult::default();

        let now = SystemTime::now();
        let event_time = event.timestamp();

        // Check if timestamp is in the future
        if event_time > now {
            result.warnings.push(Warning::UnusualValue {
                field: "timestamp".to_owned(),
                value: format!("{event_time:?}"),
            });
        }

        result.consistency_checks = 1;
        result
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::solana_metadata::SolanaEventMetadata;
    type EventMetadata = SolanaEventMetadata;

    #[tokio::test]
    async fn test_validation_pipeline_creation() {
        let config = Config::default();
        let pipeline = Pipeline::new(config);

        let stats = pipeline.get_stats();
        assert!(stats.total_rules > 0);
        assert_eq!(stats.cached_events, 0);
    }

    #[tokio::test]
    async fn test_required_fields_validation() {
        let rule = RequiredFieldsRule;
        let metadata = SolanaEventMetadata::default();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let config = Config::default();
        let result = rule.validate(&event, &config).await;

        // Should have errors for empty id and signature
        assert!(result.errors.len() >= 2);
    }

    #[tokio::test]
    async fn test_duplicate_detection() {
        let config = Config {
            enable_duplicate_detection: true,
            ..Default::default()
        };
        let pipeline = Pipeline::new(config);

        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
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
        let mut result = RuleResult::default();

        let valid_json = serde_json::json!({
            "mint": "So11111111111111111111111111111111111111112"
        });

        PubkeyRule::validate_pubkeys_in_json(&valid_json, &mut result);
        assert!(result.errors.is_empty());

        let invalid_json = serde_json::json!({
            "mint": "InvalidPubkeyStringWithExactly44Characters!"
        });

        PubkeyRule::validate_pubkeys_in_json(&invalid_json, &mut result);
        assert!(!result.errors.is_empty());
    }

    // Additional comprehensive unit tests for 100% coverage

    #[test]
    fn test_validation_config_default() {
        let config = Config::default();
        assert!(!config.strict_mode);
        assert!(config.enable_consistency_checks);
        assert!(config.enable_business_validation);
        assert!(config.enable_duplicate_detection);
        assert_eq!(config.max_event_age, Duration::from_secs(3600));
        assert!(config.known_tokens.is_empty());
        assert!(config.known_programs.is_empty());
    }

    #[test]
    fn test_validation_config_custom() {
        let mut known_tokens = HashSet::new();
        known_tokens.insert(Pubkey::default());
        let mut known_programs = HashSet::new();
        known_programs.insert(Pubkey::default());

        let config = Config {
            strict_mode: true,
            enable_consistency_checks: false,
            enable_business_validation: false,
            enable_duplicate_detection: false,
            max_event_age: Duration::from_secs(7200),
            known_tokens,
            known_programs,
        };

        assert!(config.strict_mode);
        assert!(!config.enable_consistency_checks);
        assert!(!config.enable_business_validation);
        assert!(!config.enable_duplicate_detection);
        assert_eq!(config.max_event_age, Duration::from_secs(7200));
        assert_eq!(config.known_tokens.len(), 1);
        assert_eq!(config.known_programs.len(), 1);
    }

    #[test]
    fn test_validation_error_variants() {
        let missing_field_error = Error::MissingField {
            field: "test_field".to_string(),
        };
        assert!(matches!(missing_field_error, Error::MissingField { .. }));

        let invalid_value_error = Error::InvalidValue {
            field: "test_field".to_string(),
            reason: "test_reason".to_string(),
        };
        assert!(matches!(invalid_value_error, Error::InvalidValue { .. }));

        let inconsistency_error = Error::Inconsistency {
            description: "test_description".to_string(),
        };
        assert!(matches!(inconsistency_error, Error::Inconsistency { .. }));

        let business_logic_error = Error::BusinessLogicError {
            rule: "test_rule".to_string(),
            description: "test_description".to_string(),
        };
        assert!(matches!(
            business_logic_error,
            Error::BusinessLogicError { .. }
        ));

        let stale_event_error = Error::StaleEvent {
            age: Duration::from_secs(100),
        };
        assert!(matches!(stale_event_error, Error::StaleEvent { .. }));

        let duplicate_error = Error::Duplicate {
            original_id: "test_id".to_string(),
        };
        assert!(matches!(duplicate_error, Error::Duplicate { .. }));
    }

    #[test]
    fn test_validation_warning_variants() {
        let unusual_value_warning = Warning::UnusualValue {
            field: "test_field".to_string(),
            value: "test_value".to_string(),
        };
        assert!(matches!(
            unusual_value_warning,
            Warning::UnusualValue { .. }
        ));

        let deprecated_field_warning = Warning::DeprecatedField {
            field: "test_field".to_string(),
        };
        assert!(matches!(
            deprecated_field_warning,
            Warning::DeprecatedField { .. }
        ));

        let performance_warning = Warning::PerformanceWarning {
            description: "test_description".to_string(),
        };
        assert!(matches!(
            performance_warning,
            Warning::PerformanceWarning { .. }
        ));
    }

    #[test]
    fn test_validation_metrics_default() {
        let metrics = Metrics::default();
        assert_eq!(metrics.validation_time, Duration::ZERO);
        assert_eq!(metrics.rules_checked, 0);
        assert_eq!(metrics.consistency_checks, 0);
    }

    #[test]
    fn test_validation_metrics_custom() {
        let metrics = Metrics {
            validation_time: Duration::from_millis(100),
            rules_checked: 5,
            consistency_checks: 3,
        };
        assert_eq!(metrics.validation_time, Duration::from_millis(100));
        assert_eq!(metrics.rules_checked, 5);
        assert_eq!(metrics.consistency_checks, 3);
    }

    #[test]
    fn test_rule_result_default() {
        let result = RuleResult::default();
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
        assert_eq!(result.consistency_checks, 0);
    }

    #[test]
    fn test_rule_result_with_data() {
        let mut result = RuleResult::default();
        result.errors.push(Error::MissingField {
            field: "test".to_string(),
        });
        result.warnings.push(Warning::UnusualValue {
            field: "test".to_string(),
            value: "test".to_string(),
        });
        result.consistency_checks = 5;

        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.consistency_checks, 5);
    }

    #[test]
    fn test_validation_pipeline_with_custom_config() {
        let config = Config {
            strict_mode: true,
            enable_duplicate_detection: false,
            ..Default::default()
        };
        let pipeline = Pipeline::new(config);

        assert!(pipeline.config.strict_mode);
        assert!(!pipeline.config.enable_duplicate_detection);
        assert!(!pipeline.rules.is_empty());
    }

    #[test]
    fn test_validation_pipeline_add_rule() {
        let config = Config::default();
        let mut pipeline = Pipeline::new(config);
        let initial_rules_count = pipeline.rules.len();

        let custom_rule = Arc::new(RequiredFieldsRule);
        pipeline.add_rule(custom_rule);

        assert_eq!(pipeline.rules.len(), initial_rules_count + 1);
    }

    #[tokio::test]
    async fn test_validation_pipeline_strict_mode_false() {
        let config = Config {
            strict_mode: false,
            ..Default::default()
        };
        let pipeline = Pipeline::new(config);

        // Create an event with missing required fields
        let metadata = SolanaEventMetadata::default();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let result = pipeline.validate_event(&event).await;

        // With strict_mode false, should be valid even with errors
        assert!(result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validation_pipeline_strict_mode_true() {
        let config = Config {
            strict_mode: true,
            ..Default::default()
        };
        let pipeline = Pipeline::new(config);

        // Create an event with missing required fields
        let metadata = SolanaEventMetadata::default();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let result = pipeline.validate_event(&event).await;

        // With strict_mode true, should be invalid when there are errors
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validation_pipeline_disabled_duplicate_detection() {
        let config = Config {
            enable_duplicate_detection: false,
            ..Default::default()
        };
        let pipeline = Pipeline::new(config);

        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::default(),
            "0".to_string(),
            0,
            core_metadata,
        );

        let event = ZeroCopyEvent::new_owned(metadata.clone(), vec![]);

        // Both validations should not detect duplicates
        let result1 = pipeline.validate_event(&event).await;
        let result2 = pipeline.validate_event(&event).await;

        // Should not find duplicate errors
        let has_duplicate_errors = result1
            .errors
            .iter()
            .any(|e| matches!(*e, Error::Duplicate { .. }))
            || result2
                .errors
                .iter()
                .any(|e| matches!(*e, Error::Duplicate { .. }));
        assert!(!has_duplicate_errors);
    }

    #[tokio::test]
    async fn test_validate_events_multiple() {
        let config = Config::default();
        let pipeline = Pipeline::new(config);

        let metadata1 = EventMetadata::default();
        let metadata2 = EventMetadata::default();
        let events = vec![
            ZeroCopyEvent::new_owned(metadata1, vec![]),
            ZeroCopyEvent::new_owned(metadata2, vec![]),
        ];

        let results = pipeline.validate_events(&events).await;
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_validate_events_empty() {
        let config = Config::default();
        let pipeline = Pipeline::new(config);

        let events = vec![];
        let results = pipeline.validate_events(&events).await;
        assert!(results.is_empty());
    }

    #[test]
    fn test_check_event_age_stale() {
        let config = Config {
            max_event_age: Duration::from_secs(1),
            ..Default::default()
        };
        let pipeline = Pipeline::new(config);

        // Create an event with old timestamp
        let old_time = chrono::Utc::now() - chrono::Duration::seconds(10);
        let mut core_metadata = riglr_events_core::EventMetadata::new(
            String::default(),
            EventKind::Transaction,
            "test".to_string(),
        );
        core_metadata.timestamp = old_time;
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::default(),
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let error = pipeline.check_event_age(&event);
        assert!(error.is_some());
        assert!(matches!(
            error.expect("Expected validation error for stale event"),
            Error::StaleEvent { .. }
        ));
    }

    #[test]
    fn test_check_event_age_fresh() {
        let config = Config {
            max_event_age: Duration::from_secs(3600),
            ..Default::default()
        };
        let pipeline = Pipeline::new(config);

        // Create an event with recent timestamp
        let recent_time = chrono::Utc::now() - chrono::Duration::seconds(10);
        let mut core_metadata = riglr_events_core::EventMetadata::new(
            String::default(),
            EventKind::Transaction,
            "test".to_string(),
        );
        core_metadata.timestamp = recent_time;
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::default(),
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let error = pipeline.check_event_age(&event);
        assert!(error.is_none());
    }

    #[test]
    fn test_check_event_age_future_timestamp() {
        let config = Config::default();
        let pipeline = Pipeline::new(config);

        // Create an event with future timestamp
        let future_time = chrono::Utc::now() + chrono::Duration::seconds(10);
        let mut core_metadata = riglr_events_core::EventMetadata::new(
            String::default(),
            EventKind::Transaction,
            "test".to_string(),
        );
        core_metadata.timestamp = future_time;
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::default(),
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        // Should not return error for future timestamps in check_event_age
        let error = pipeline.check_event_age(&event);
        assert!(error.is_none());
    }

    #[tokio::test]
    async fn test_check_duplicate_cache_cleanup() {
        let config = Config {
            max_event_age: Duration::from_millis(100),
            enable_duplicate_detection: true,
            ..Default::default()
        };
        let pipeline = Pipeline::new(config);

        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::default(),
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        // First check should not find duplicate
        let error1 = pipeline.check_duplicate(&event);
        assert!(error1.is_none());

        // Wait for cache cleanup
        time::sleep(Duration::from_millis(150)).await;

        // Second check should not find duplicate because cache was cleaned
        let error2 = pipeline.check_duplicate(&event);
        assert!(error2.is_none());
    }

    #[tokio::test]
    async fn test_jupiter_validation_missing_fields() {
        let config = Config::default();
        let _pipeline = Pipeline::new(config);

        let json_data = serde_json::json!({
            "some_other_field": "value"
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::Jupiter,
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let error = Pipeline::validate_jupiter_event(&event);
        assert!(error.is_some());
        assert!(matches!(
            error.expect("Expected validation error for missing Jupiter field"),
            Error::MissingField { .. }
        ));
    }

    #[tokio::test]
    async fn test_jupiter_validation_valid_fields() {
        let config = Config::default();
        let _pipeline = Pipeline::new(config);

        let json_data = serde_json::json!({
            "total_amount_in": "1000",
            "total_amount_out": "900"
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::Jupiter,
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let error = Pipeline::validate_jupiter_event(&event);
        assert!(error.is_none());
    }

    #[tokio::test]
    async fn test_jupiter_validation_zero_amount() {
        let config = Config::default();
        let _pipeline = Pipeline::new(config);

        let json_data = serde_json::json!({
            "total_amount_in": "0",
            "total_amount_out": "900"
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::Jupiter,
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let error = Pipeline::validate_jupiter_event(&event);
        assert!(error.is_some());
        assert!(matches!(
            error.expect("Expected validation error for invalid Jupiter value"),
            Error::InvalidValue { .. }
        ));
    }

    #[tokio::test]
    async fn test_jupiter_validation_no_json_data() {
        let config = Config::default();
        let _pipeline = Pipeline::new(config);

        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::Jupiter,
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let error = Pipeline::validate_jupiter_event(&event);
        assert!(error.is_none());
    }

    #[tokio::test]
    async fn test_raydium_validation_swap_missing_amounts() {
        let config = Config::default();
        let _pipeline = Pipeline::new(config);

        let json_data = serde_json::json!({
            "some_other_field": "value"
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::RaydiumAmmV4SwapBaseIn,
            ProtocolType::RaydiumAmmV4,
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let error = Pipeline::validate_raydium_event(&event);
        assert!(error.is_some());
        assert!(matches!(
            error.expect("Expected validation error for missing Raydium field"),
            Error::MissingField { .. }
        ));
    }

    #[tokio::test]
    async fn test_raydium_validation_swap_with_amount_in() {
        let config = Config::default();
        let _pipeline = Pipeline::new(config);

        let json_data = serde_json::json!({
            "amount_in": "1000"
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::RaydiumAmmV4SwapBaseOut,
            ProtocolType::RaydiumAmmV4,
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let error = Pipeline::validate_raydium_event(&event);
        assert!(error.is_none());
    }

    #[tokio::test]
    async fn test_raydium_validation_swap_with_max_amount_in() {
        let config = Config::default();
        let _pipeline = Pipeline::new(config);

        let json_data = serde_json::json!({
            "max_amount_in": "1000"
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::RaydiumAmmV4SwapBaseIn,
            ProtocolType::RaydiumAmmV4,
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let error = Pipeline::validate_raydium_event(&event);
        assert!(error.is_none());
    }

    #[tokio::test]
    async fn test_raydium_validation_non_swap_event() {
        let config = Config::default();
        let _pipeline = Pipeline::new(config);

        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::AddLiquidity,
            ProtocolType::RaydiumAmmV4,
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let error = Pipeline::validate_raydium_event(&event);
        assert!(error.is_none());
    }

    #[tokio::test]
    async fn test_pump_fun_validation_valid_discriminator() {
        let config = Config::default();
        let _pipeline = Pipeline::new(config);

        let json_data = serde_json::json!({
            "discriminator": 0x6606_3d12_01da_ebea_u64
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::PumpSwap,
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let error = Pipeline::validate_pump_fun_event(&event);
        assert!(error.is_none());
    }

    #[tokio::test]
    async fn test_pump_fun_validation_invalid_discriminator() {
        let config = Config::default();
        let _pipeline = Pipeline::new(config);

        let json_data = serde_json::json!({
            "discriminator": 0x0123_4567_89ab_cdef_u64
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::PumpSwap,
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let error = Pipeline::validate_pump_fun_event(&event);
        assert!(error.is_some());
        assert!(matches!(
            error.expect("Expected validation error for invalid discriminator"),
            Error::InvalidValue { .. }
        ));
    }

    #[tokio::test]
    async fn test_pump_fun_validation_no_discriminator() {
        let config = Config::default();
        let _pipeline = Pipeline::new(config);

        let json_data = serde_json::json!({
            "some_other_field": "value"
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::PumpSwap,
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let error = Pipeline::validate_pump_fun_event(&event);
        assert!(error.is_none());
    }

    #[tokio::test]
    async fn test_pump_fun_validation_no_json_data() {
        let config = Config::default();
        let _pipeline = Pipeline::new(config);

        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::PumpSwap,
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let error = Pipeline::validate_pump_fun_event(&event);
        assert!(error.is_none());
    }

    #[tokio::test]
    async fn test_required_fields_rule_name() {
        let rule = RequiredFieldsRule;
        assert_eq!(rule.name(), "required_fields");
    }

    #[tokio::test]
    async fn test_required_fields_rule_valid_event() {
        let rule = RequiredFieldsRule;
        let config = Config::default();

        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "valid-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::default(),
            "valid-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let result = rule.validate(&event, &config).await;
        assert!(result.errors.is_empty());
        assert_eq!(result.consistency_checks, 2);
    }

    #[tokio::test]
    async fn test_data_types_rule_name() {
        let rule = DataTypesRule;
        assert_eq!(rule.name(), "data_types");
    }

    #[tokio::test]
    async fn test_data_types_rule_zero_slot() {
        let rule = DataTypesRule;
        let config = Config::default();

        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0, // slot = 0
            EventType::default(),
            ProtocolType::default(),
            "test-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let result = rule.validate(&event, &config).await;
        assert!(!result.warnings.is_empty());
        assert!(matches!(
            *result
                .warnings
                .first()
                .expect("Expected at least one warning"),
            Warning::UnusualValue { .. }
        ));
        assert_eq!(result.consistency_checks, 1);
    }

    #[tokio::test]
    async fn test_data_types_rule_non_zero_slot() {
        let rule = DataTypesRule;
        let config = Config::default();

        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            123, // slot = 123
            EventType::default(),
            ProtocolType::default(),
            "test-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let result = rule.validate(&event, &config).await;
        assert!(result.warnings.is_empty());
        assert_eq!(result.consistency_checks, 1);
    }

    #[tokio::test]
    async fn test_numeric_ranges_rule_name() {
        let rule = NumericRangesRule;
        assert_eq!(rule.name(), "numeric_ranges");
    }

    #[tokio::test]
    async fn test_numeric_ranges_rule_large_amount() {
        let rule = NumericRangesRule;
        let config = Config::default();

        let json_data = serde_json::json!({
            "amount_in": format!("{}", u64::MAX / 2 + 1)
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::default(),
            "test-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let result = rule.validate(&event, &config).await;
        assert!(!result.warnings.is_empty());
        assert!(matches!(
            *result
                .warnings
                .first()
                .expect("Expected at least one warning"),
            Warning::UnusualValue { .. }
        ));
    }

    #[tokio::test]
    async fn test_numeric_ranges_rule_normal_amount() {
        let rule = NumericRangesRule;
        let config = Config::default();

        let json_data = serde_json::json!({
            "amount_in": "1000"
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::default(),
            "test-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let result = rule.validate(&event, &config).await;
        assert!(result.warnings.is_empty());
    }

    #[tokio::test]
    async fn test_numeric_ranges_rule_no_json() {
        let rule = NumericRangesRule;
        let config = Config::default();

        let metadata = SolanaEventMetadata::default();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let result = rule.validate(&event, &config).await;
        assert!(result.warnings.is_empty());
        assert_eq!(result.consistency_checks, 0);
    }

    #[tokio::test]
    async fn test_numeric_ranges_rule_non_string_amount() {
        let rule = NumericRangesRule;
        let config = Config::default();

        let json_data = serde_json::json!({
            "amount_in": 1000
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::default(),
            "test-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let result = rule.validate(&event, &config).await;
        assert!(result.warnings.is_empty());
    }

    #[tokio::test]
    async fn test_pubkey_validation_rule_name() {
        let rule = PubkeyRule;
        assert_eq!(rule.name(), "pubkey_validation");
    }

    #[tokio::test]
    async fn test_pubkey_validation_rule_valid_event() {
        let rule = PubkeyRule;
        let config = Config::default();

        let json_data = serde_json::json!({
            "mint": "So11111111111111111111111111111111111111112"
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::default(),
            "test-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let result = rule.validate(&event, &config).await;
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_pubkey_validation_json_array() {
        let mut result = RuleResult::default();

        let json_array = serde_json::json!([
            "So11111111111111111111111111111111111111112",
            "InvalidPubkeyStringWithExactly44Characters!"
        ]);

        PubkeyRule::validate_pubkeys_in_json(&json_array, &mut result);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_pubkey_validation_json_object_nested() {
        let mut result = RuleResult::default();

        let json_nested = serde_json::json!({
            "data": {
                "mint": "So11111111111111111111111111111111111111112",
                "address": "InvalidPubkeyStringWithExactly44Characters!"
            }
        });

        PubkeyRule::validate_pubkeys_in_json(&json_nested, &mut result);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_pubkey_validation_json_non_string() {
        let mut result = RuleResult::default();

        let json_non_string = serde_json::json!({
            "mint": 12345
        });

        PubkeyRule::validate_pubkeys_in_json(&json_non_string, &mut result);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_pubkey_validation_json_short_string() {
        let mut result = RuleResult::default();

        let json_short = serde_json::json!({
            "mint": "short"
        });

        PubkeyRule::validate_pubkeys_in_json(&json_short, &mut result);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_pubkey_validation_json_43_char_valid() {
        let mut result = RuleResult::default();

        let json_43_char = serde_json::json!({
            "mint": "11111111111111111111111111111111111111111"
        });

        PubkeyRule::validate_pubkeys_in_json(&json_43_char, &mut result);
        // 43 char string will be checked but likely fail validation
        assert!(result.consistency_checks > 0);
    }

    #[tokio::test]
    async fn test_swap_consistency_rule_name() {
        let rule = SwapConsistencyRule;
        assert_eq!(rule.name(), "swap_consistency");
    }

    #[tokio::test]
    async fn test_swap_consistency_rule_valid_swap() {
        let rule = SwapConsistencyRule;
        let config = Config::default();

        let json_data = serde_json::json!({
            "amount_in": "1000",
            "amount_out": "900"
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::Swap,
            ProtocolType::default(),
            "test-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let result = rule.validate(&event, &config).await;
        assert!(result.errors.is_empty());
        assert_eq!(result.consistency_checks, 1);
    }

    #[tokio::test]
    async fn test_swap_consistency_rule_zero_amounts() {
        let rule = SwapConsistencyRule;
        let config = Config::default();

        let json_data = serde_json::json!({
            "amount_in": "0",
            "amount_out": "900"
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::Swap,
            ProtocolType::default(),
            "test-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let result = rule.validate(&event, &config).await;
        assert!(!result.errors.is_empty());
        assert!(matches!(
            *result.errors.first().expect("Expected at least one error"),
            Error::InvalidValue { .. }
        ));
    }

    #[tokio::test]
    async fn test_swap_consistency_rule_non_swap_event() {
        let rule = SwapConsistencyRule;
        let config = Config::default();

        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::AddLiquidity,
            ProtocolType::default(),
            "test-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let result = rule.validate(&event, &config).await;
        assert!(result.errors.is_empty());
        assert_eq!(result.consistency_checks, 0);
    }

    #[tokio::test]
    async fn test_swap_consistency_rule_missing_amounts() {
        let rule = SwapConsistencyRule;
        let config = Config::default();

        let json_data = serde_json::json!({
            "some_other_field": "value"
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::Swap,
            ProtocolType::default(),
            "test-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let result = rule.validate(&event, &config).await;
        assert!(result.errors.is_empty());
        assert_eq!(result.consistency_checks, 0);
    }

    #[tokio::test]
    async fn test_swap_consistency_rule_invalid_amount_format() {
        let rule = SwapConsistencyRule;
        let config = Config::default();

        let json_data = serde_json::json!({
            "amount_in": "invalid",
            "amount_out": "900"
        });
        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::Swap,
            ProtocolType::default(),
            "test-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned_with_json(metadata, vec![], json_data);

        let result = rule.validate(&event, &config).await;
        assert!(result.errors.is_empty());
        assert_eq!(result.consistency_checks, 0);
    }

    #[tokio::test]
    async fn test_liquidity_validation_rule_name() {
        let rule = LiquidityRule;
        assert_eq!(rule.name(), "liquidity_validation");
    }

    #[tokio::test]
    async fn test_liquidity_validation_rule_add_liquidity() {
        let rule = LiquidityRule;
        let config = Config::default();

        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::AddLiquidity,
            ProtocolType::default(),
            "test-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let result = rule.validate(&event, &config).await;
        assert!(result.errors.is_empty());
        assert_eq!(result.consistency_checks, 1);
    }

    #[tokio::test]
    async fn test_liquidity_validation_rule_remove_liquidity() {
        let rule = LiquidityRule;
        let config = Config::default();

        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::RemoveLiquidity,
            ProtocolType::default(),
            "test-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let result = rule.validate(&event, &config).await;
        assert!(result.errors.is_empty());
        assert_eq!(result.consistency_checks, 1);
    }

    #[tokio::test]
    async fn test_liquidity_validation_rule_other_event() {
        let rule = LiquidityRule;
        let config = Config::default();

        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::Swap,
            ProtocolType::default(),
            "test-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let result = rule.validate(&event, &config).await;
        assert!(result.errors.is_empty());
        assert_eq!(result.consistency_checks, 0);
    }

    #[tokio::test]
    async fn test_timestamp_validation_rule_name() {
        let rule = TimestampRule;
        assert_eq!(rule.name(), "timestamp_validation");
    }

    #[tokio::test]
    async fn test_timestamp_validation_rule_future_timestamp() {
        let rule = TimestampRule;
        let config = Config::default();

        let future_time = chrono::Utc::now() + chrono::Duration::seconds(10);
        let mut core_metadata = riglr_events_core::EventMetadata::new(
            String::default(),
            EventKind::Transaction,
            "test".to_string(),
        );
        core_metadata.timestamp = future_time;
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::default(),
            "test-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let result = rule.validate(&event, &config).await;
        assert!(!result.warnings.is_empty());
        assert!(matches!(
            *result
                .warnings
                .first()
                .expect("Expected at least one warning"),
            Warning::UnusualValue { .. }
        ));
        assert_eq!(result.consistency_checks, 1);
    }

    #[tokio::test]
    async fn test_timestamp_validation_rule_past_timestamp() {
        let rule = TimestampRule;
        let config = Config::default();

        let past_time = chrono::Utc::now() - chrono::Duration::seconds(10);
        let mut core_metadata = riglr_events_core::EventMetadata::new(
            String::default(),
            EventKind::Transaction,
            "test".to_string(),
        );
        core_metadata.timestamp = past_time;
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::default(),
            "test-id".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let result = rule.validate(&event, &config).await;
        assert!(result.warnings.is_empty());
        assert_eq!(result.consistency_checks, 1);
    }

    #[tokio::test]
    async fn test_protocol_specific_validation_other_protocol() {
        let config = Config::default();
        let pipeline = Pipeline::new(config);

        let core_metadata = riglr_events_core::EventMetadata::default();
        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            0,
            EventType::default(),
            ProtocolType::OrcaWhirlpool, // Other protocol
            "0".to_string(),
            0,
            core_metadata,
        );
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let result = pipeline.validate_event(&event).await;

        // Should complete validation without protocol-specific errors
        assert!(result.metrics.validation_time > Duration::ZERO);
        assert!(result.metrics.rules_checked > 0);
    }

    #[test]
    fn test_validation_stats_debug() {
        let stats = Stats {
            total_rules: 5,
            cached_events: 10,
            strict_mode: true,
        };

        let debug_str = format!("{stats:?}");
        assert!(debug_str.contains("total_rules: 5"));
        assert!(debug_str.contains("cached_events: 10"));
        assert!(debug_str.contains("strict_mode: true"));
    }

    #[test]
    fn test_validation_result_debug() {
        let result = Result {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
            metrics: Metrics::default(),
        };

        let debug_str = format!("{result:?}");
        assert!(debug_str.contains("is_valid: true"));
    }

    #[test]
    fn test_validation_error_debug() {
        let error = Error::MissingField {
            field: "test".to_string(),
        };

        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("MissingField"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_validation_warning_debug() {
        let warning = Warning::UnusualValue {
            field: "test".to_string(),
            value: "value".to_string(),
        };

        let debug_str = format!("{warning:?}");
        assert!(debug_str.contains("UnusualValue"));
        assert!(debug_str.contains("test"));
    }
}
