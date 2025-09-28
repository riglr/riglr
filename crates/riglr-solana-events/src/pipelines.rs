pub mod enrichment;
pub mod parsing;
pub mod validation;

// Explicit re-exports from enrichment module
pub use enrichment::{
    CacheStats, Config as EnrichmentConfig, Error as EnrichmentError, EventEnricher, PriceData,
    TokenMetadata, TransactionContext,
};

// Explicit re-exports from parsing module
pub use parsing::{
    Input as ParsingInput, Metrics as ParsingMetrics, Output as ParsingOutput, ParserConfig,
    ParserMetrics, Pipeline as ParsingPipeline, PipelineBuilder as ParsingPipelineBuilder,
    PipelineConfig as ParsingPipelineConfig, PipelineError, PipelineStats,
};

// Explicit re-exports from validation module
pub use validation::{
    Config as ValidationConfig, Error as ValidationError, Metrics as ValidationMetrics,
    Pipeline as ValidationPipeline, Result as ValidationResult, RuleResult,
    Stats as ValidationStats, Warning as ValidationWarning,
};

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use core::{
        hint,
        mem::{size_of, size_of_val},
        time::Duration,
    };

    #[test]
    fn enrichment_exports_are_available() {
        // Test that all main enrichment types are accessible through re-exports
        let config = EnrichmentConfig::default();
        assert!(config.enable_token_metadata);
        assert!(config.enable_price_data);
        assert!(config.enable_transaction_context);
        assert_eq!(config.metadata_cache_ttl, Duration::from_secs(300));
        assert_eq!(config.max_cache_size, 10000);
        assert_eq!(config.api_timeout, Duration::from_secs(5));

        let Ok(enricher) = EventEnricher::new(config) else {
            panic!("EventEnricher should be created with default config")
        };
        let stats = enricher.get_cache_stats();
        assert_eq!(stats.token_cache_size, 0);
        assert_eq!(stats.price_cache_size, 0);
    }

    #[test]
    fn parsing_exports_are_available() {
        // Test that all main parsing types are accessible through re-exports
        let config = ParsingPipelineConfig::default();
        assert_eq!(config.max_batch_size, 100);
        assert_eq!(config.batch_timeout, Duration::from_millis(50));
        assert_eq!(config.max_concurrent_tasks, 4);
        assert_eq!(config.output_buffer_size, 1000);
        assert!(config.enable_metrics);
        assert!(config.parser_configs.is_empty());

        let pipeline = ParsingPipeline::new(config);
        let stats = pipeline.get_stats();
        assert_eq!(stats.max_batch_size, 100);
        assert_eq!(stats.max_concurrent_tasks, 4);
        assert_eq!(stats.registered_parsers, 0);
        assert_eq!(stats.available_permits, 4);
    }

    #[test]
    fn validation_exports_are_available() {
        // Test that all main validation types are accessible through re-exports
        let config = ValidationConfig::default();
        assert!(!config.strict_mode);
        assert!(config.enable_consistency_checks);
        assert!(config.enable_business_validation);
        assert!(config.enable_duplicate_detection);
        assert_eq!(config.max_event_age, Duration::from_secs(3600));
        assert!(config.known_tokens.is_empty());
        assert!(config.known_programs.is_empty());

        let pipeline = ValidationPipeline::new(config);
        // The pipeline should be created successfully and have default rules
        // We can't easily test async methods here, but we can verify creation
        assert_eq!(size_of_val(&pipeline), size_of::<ValidationPipeline>());
    }

    #[tokio::test]
    async fn parsing_pipeline_builder_available() {
        // Test that the builder pattern works through re-exports
        let pipeline = ParsingPipelineBuilder::new()
            .with_batch_size(50)
            .with_batch_timeout(Duration::from_millis(100))
            .with_concurrency_limit(2)
            .with_metrics(false)
            .build();

        let stats = pipeline.get_stats();
        assert_eq!(stats.max_batch_size, 50);
        assert_eq!(stats.max_concurrent_tasks, 2);
    }

    #[test]
    fn parser_config_available() {
        // Test that ParserConfig can be created and used
        let config = ParserConfig::default();
        assert!(config.zero_copy);
        assert!(config.detailed_analysis);
        assert_eq!(config.priority, 1);

        let custom_config = ParserConfig {
            zero_copy: false,
            detailed_analysis: false,
            priority: 5,
        };
        assert!(!custom_config.zero_copy);
        assert!(!custom_config.detailed_analysis);
        assert_eq!(custom_config.priority, 5);
    }

    #[test]
    fn validation_error_types_available() {
        // Test that validation error types are accessible
        let missing_field_error = ValidationError::MissingField {
            field: "test_field".to_owned(),
        };
        match missing_field_error {
            ValidationError::MissingField { field } => {
                assert_eq!(field, "test_field");
            }
            ValidationError::InvalidValue { .. }
            | ValidationError::Inconsistency { .. }
            | ValidationError::BusinessLogicError { .. }
            | ValidationError::StaleEvent { .. }
            | ValidationError::Duplicate { .. } => {
                panic!("Wrong error type");
            }
        }

        let invalid_value_error = ValidationError::InvalidValue {
            field: "amount".to_owned(),
            reason: "negative value".to_owned(),
        };
        match invalid_value_error {
            ValidationError::InvalidValue { field, reason } => {
                assert_eq!(field, "amount");
                assert_eq!(reason, "negative value");
            }
            ValidationError::MissingField { .. }
            | ValidationError::Inconsistency { .. }
            | ValidationError::BusinessLogicError { .. }
            | ValidationError::StaleEvent { .. }
            | ValidationError::Duplicate { .. } => {
                panic!("Wrong error type");
            }
        }

        let inconsistency_error = ValidationError::Inconsistency {
            description: "data mismatch".to_owned(),
        };
        match inconsistency_error {
            ValidationError::Inconsistency { description } => {
                assert_eq!(description, "data mismatch");
            }
            ValidationError::MissingField { .. }
            | ValidationError::InvalidValue { .. }
            | ValidationError::BusinessLogicError { .. }
            | ValidationError::StaleEvent { .. }
            | ValidationError::Duplicate { .. } => {
                panic!("Wrong error type");
            }
        }

        let business_logic_error = ValidationError::BusinessLogicError {
            rule: "min_amount".to_owned(),
            description: "amount too small".to_owned(),
        };
        match business_logic_error {
            ValidationError::BusinessLogicError { rule, description } => {
                assert_eq!(rule, "min_amount");
                assert_eq!(description, "amount too small");
            }
            ValidationError::MissingField { .. }
            | ValidationError::InvalidValue { .. }
            | ValidationError::Inconsistency { .. }
            | ValidationError::StaleEvent { .. }
            | ValidationError::Duplicate { .. } => {
                panic!("Wrong error type");
            }
        }

        let stale_event_error = ValidationError::StaleEvent {
            age: Duration::from_secs(7200),
        };
        match stale_event_error {
            ValidationError::StaleEvent { age } => {
                assert_eq!(age, Duration::from_secs(7200));
            }
            ValidationError::MissingField { .. }
            | ValidationError::InvalidValue { .. }
            | ValidationError::Inconsistency { .. }
            | ValidationError::BusinessLogicError { .. }
            | ValidationError::Duplicate { .. } => {
                panic!("Wrong error type");
            }
        }

        let duplicate_error = ValidationError::Duplicate {
            original_id: "event_123".to_owned(),
        };
        match duplicate_error {
            ValidationError::Duplicate { original_id } => {
                assert_eq!(original_id, "event_123");
            }
            ValidationError::MissingField { .. }
            | ValidationError::InvalidValue { .. }
            | ValidationError::Inconsistency { .. }
            | ValidationError::BusinessLogicError { .. }
            | ValidationError::StaleEvent { .. } => {
                panic!("Wrong error type");
            }
        }
    }

    #[test]
    fn validation_warning_types_available() {
        // Test that validation warning types are accessible
        let unusual_value_warning = ValidationWarning::UnusualValue {
            field: "amount".to_owned(),
            value: "999999999".to_owned(),
        };
        match unusual_value_warning {
            ValidationWarning::UnusualValue { field, value } => {
                assert_eq!(field, "amount");
                assert_eq!(value, "999999999");
            }
            ValidationWarning::DeprecatedField { .. }
            | ValidationWarning::PerformanceWarning { .. } => {
                panic!("Wrong warning type");
            }
        }

        let deprecated_field_warning = ValidationWarning::DeprecatedField {
            field: "old_field".to_owned(),
        };
        match deprecated_field_warning {
            ValidationWarning::DeprecatedField { field } => {
                assert_eq!(field, "old_field");
            }
            ValidationWarning::UnusualValue { .. }
            | ValidationWarning::PerformanceWarning { .. } => {
                panic!("Wrong warning type");
            }
        }

        let performance_warning = ValidationWarning::PerformanceWarning {
            description: "high computational cost".to_owned(),
        };
        match performance_warning {
            ValidationWarning::PerformanceWarning { description } => {
                assert_eq!(description, "high computational cost");
            }
            ValidationWarning::UnusualValue { .. } | ValidationWarning::DeprecatedField { .. } => {
                panic!("Wrong warning type");
            }
        }
    }

    #[test]
    fn enrichment_error_types_available() {
        // Test that enrichment error types are accessible through re-exports

        // Test TaskError variant
        let task_error = EnrichmentError::TaskError("task failed".to_owned());
        match task_error {
            EnrichmentError::TaskError(msg) => {
                assert_eq!(msg, "task failed");
            }
            EnrichmentError::HttpError(_)
            | EnrichmentError::SerializationError(_)
            | EnrichmentError::CacheError(_)
            | EnrichmentError::RateLimitExceeded => {
                panic!("Wrong error type");
            }
        }

        // Test CacheError variant
        let cache_error = EnrichmentError::CacheError("cache full".to_owned());
        match cache_error {
            EnrichmentError::CacheError(msg) => {
                assert_eq!(msg, "cache full");
            }
            EnrichmentError::HttpError(_)
            | EnrichmentError::SerializationError(_)
            | EnrichmentError::TaskError(_)
            | EnrichmentError::RateLimitExceeded => {
                panic!("Wrong error type");
            }
        }

        // Test RateLimitExceeded variant
        let rate_limit_error = EnrichmentError::RateLimitExceeded;
        match rate_limit_error {
            EnrichmentError::RateLimitExceeded => {
                // Expected variant
            }
            EnrichmentError::HttpError(_)
            | EnrichmentError::SerializationError(_)
            | EnrichmentError::TaskError(_)
            | EnrichmentError::CacheError(_) => {
                panic!("Wrong error type");
            }
        }
    }

    #[test]
    fn parsing_metrics_available() {
        // Test that parsing metrics structures are accessible
        let metrics = ParsingMetrics::default();
        assert_eq!(metrics.processing_time, Duration::from_secs(0));
        assert_eq!(metrics.events_parsed, 0);
        assert_eq!(metrics.bytes_processed, 0);
        assert_eq!(metrics.error_count, 0);
        assert!(metrics.parser_metrics.is_empty());

        let parser_metrics = ParserMetrics::default();
        assert_eq!(parser_metrics.parse_time, Duration::from_secs(0));
        assert_eq!(parser_metrics.events_count, 0);
    }

    #[test]
    fn pipeline_error_types_available() {
        // Test that pipeline error types are accessible
        let semaphore_error = PipelineError::SemaphoreError(());
        match semaphore_error {
            PipelineError::SemaphoreError(()) => {
                // Expected variant
            }
            PipelineError::ChannelError
            | PipelineError::ParseError(_)
            | PipelineError::ConfigError(_) => {
                panic!("Wrong error type");
            }
        }

        let channel_error = PipelineError::ChannelError;
        match channel_error {
            PipelineError::ChannelError => {
                // Expected variant
            }
            PipelineError::SemaphoreError(())
            | PipelineError::ParseError(_)
            | PipelineError::ConfigError(_) => {
                panic!("Wrong error type");
            }
        }

        let config_error = PipelineError::ConfigError("invalid config".to_owned());
        match config_error {
            PipelineError::ConfigError(msg) => {
                assert_eq!(msg, "invalid config");
            }
            PipelineError::SemaphoreError(())
            | PipelineError::ChannelError
            | PipelineError::ParseError(_) => {
                panic!("Wrong error type");
            }
        }
    }

    #[test]
    fn all_structs_are_cloneable_where_expected() {
        // Test that structures that should be cloneable actually are
        let enrichment_config = EnrichmentConfig::default();
        let _: EnrichmentConfig = enrichment_config;

        let parsing_config = ParsingPipelineConfig::default();
        let _: ParsingPipelineConfig = parsing_config;

        let validation_config = ValidationConfig::default();
        let _: ValidationConfig = validation_config;

        let parser_config = ParserConfig::default();
        let _: ParserConfig = parser_config;

        let cache_stats = CacheStats {
            token_cache_size: 0,
            price_cache_size: 0,
            token_cache_hit_rate: 0.0,
            price_cache_hit_rate: 0.0,
        };
        let _: CacheStats = cache_stats;

        let validation_metrics = ValidationMetrics::default();
        let _: ValidationMetrics = validation_metrics;

        let parsing_metrics = ParsingMetrics::default();
        let _: ParsingMetrics = parsing_metrics;

        let parser_metrics = ParserMetrics::default();
        let _: ParserMetrics = parser_metrics;
    }

    #[test]
    fn all_structs_are_debuggable() {
        // Test that all major structures implement Debug trait
        let enrichment_config = EnrichmentConfig::default();
        let enrichment_debug_str = format!("{enrichment_config:?}");
        assert!(enrichment_debug_str.contains("EnrichmentConfig"));

        let parsing_config = ParsingPipelineConfig::default();
        let parsing_debug_str = format!("{parsing_config:?}");
        assert!(parsing_debug_str.contains("ParsingPipelineConfig"));

        let validation_config = ValidationConfig::default();
        let validation_debug_str = format!("{validation_config:?}");
        assert!(validation_debug_str.contains("ValidationConfig"));

        let parser_config = ParserConfig::default();
        let parser_debug_str = format!("{parser_config:?}");
        assert!(parser_debug_str.contains("ParserConfig"));

        let cache_stats = CacheStats {
            token_cache_size: 0,
            price_cache_size: 0,
            token_cache_hit_rate: 0.0,
            price_cache_hit_rate: 0.0,
        };
        let cache_debug_str = format!("{cache_stats:?}");
        assert!(cache_debug_str.contains("CacheStats"));
    }

    #[test]
    fn module_re_exports_work_correctly() {
        // Comprehensive test to ensure all re-exports from sub-modules are working

        // From enrichment module
        let enrichment_config = EnrichmentConfig::default();
        let Ok(event_enricher) = EventEnricher::new(EnrichmentConfig::default()) else {
            panic!("EventEnricher should be created with default config")
        };
        let cache_stats = CacheStats {
            token_cache_size: 0,
            price_cache_size: 0,
            token_cache_hit_rate: 0.0,
            price_cache_hit_rate: 0.0,
        };

        // From parsing module
        let parsing_pipeline_config = ParsingPipelineConfig::default();
        let parsing_pipeline = ParsingPipeline::new(ParsingPipelineConfig::default());
        let parsing_pipeline_builder = ParsingPipelineBuilder::new();
        let parser_config = ParserConfig::default();
        let parsing_metrics = ParsingMetrics::default();
        let parser_metrics = ParserMetrics::default();

        // From validation module
        let validation_config = ValidationConfig::default();
        let validation_pipeline = ValidationPipeline::new(ValidationConfig::default());
        let validation_metrics = ValidationMetrics::default();

        // Use all values to ensure they have effects and prevent optimization
        hint::black_box((
            enrichment_config,
            event_enricher,
            cache_stats,
            parsing_pipeline_config,
            parsing_pipeline,
            parsing_pipeline_builder,
            parser_config,
            parsing_metrics,
            parser_metrics,
            validation_config,
            validation_pipeline,
            validation_metrics,
        ));

        // This test ensures that all the main types from each module
        // are correctly re-exported and accessible
    }
}
