pub mod enrichment;
pub mod parsing;
pub mod validation;

pub use enrichment::*;
pub use parsing::*;
pub use validation::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_enrichment_exports_are_available() {
        // Test that all main enrichment types are accessible through re-exports
        let config = EnrichmentConfig::default();
        assert!(config.enable_token_metadata);
        assert!(config.enable_price_data);
        assert!(config.enable_transaction_context);
        assert_eq!(config.metadata_cache_ttl, Duration::from_secs(300));
        assert_eq!(config.max_cache_size, 10000);
        assert_eq!(config.api_timeout, Duration::from_secs(5));

        let enricher = EventEnricher::new(config);
        let stats = enricher.get_cache_stats();
        assert_eq!(stats.token_cache_size, 0);
        assert_eq!(stats.price_cache_size, 0);
    }

    #[test]
    fn test_parsing_exports_are_available() {
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
    fn test_validation_exports_are_available() {
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
        assert_eq!(
            std::mem::size_of_val(&pipeline),
            std::mem::size_of::<ValidationPipeline>()
        );
    }

    #[tokio::test]
    async fn test_parsing_pipeline_builder_available() {
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
    fn test_parser_config_available() {
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
    fn test_validation_error_types_available() {
        // Test that validation error types are accessible
        let error = ValidationError::MissingField {
            field: "test_field".to_string(),
        };
        match error {
            ValidationError::MissingField { field } => {
                assert_eq!(field, "test_field");
            }
            _ => panic!("Wrong error type"),
        }

        let error = ValidationError::InvalidValue {
            field: "amount".to_string(),
            reason: "negative value".to_string(),
        };
        match error {
            ValidationError::InvalidValue { field, reason } => {
                assert_eq!(field, "amount");
                assert_eq!(reason, "negative value");
            }
            _ => panic!("Wrong error type"),
        }

        let error = ValidationError::Inconsistency {
            description: "data mismatch".to_string(),
        };
        match error {
            ValidationError::Inconsistency { description } => {
                assert_eq!(description, "data mismatch");
            }
            _ => panic!("Wrong error type"),
        }

        let error = ValidationError::BusinessLogicError {
            rule: "min_amount".to_string(),
            description: "amount too small".to_string(),
        };
        match error {
            ValidationError::BusinessLogicError { rule, description } => {
                assert_eq!(rule, "min_amount");
                assert_eq!(description, "amount too small");
            }
            _ => panic!("Wrong error type"),
        }

        let error = ValidationError::StaleEvent {
            age: Duration::from_secs(7200),
        };
        match error {
            ValidationError::StaleEvent { age } => {
                assert_eq!(age, Duration::from_secs(7200));
            }
            _ => panic!("Wrong error type"),
        }

        let error = ValidationError::Duplicate {
            original_id: "event_123".to_string(),
        };
        match error {
            ValidationError::Duplicate { original_id } => {
                assert_eq!(original_id, "event_123");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_validation_warning_types_available() {
        // Test that validation warning types are accessible
        let warning = ValidationWarning::UnusualValue {
            field: "amount".to_string(),
            value: "999999999".to_string(),
        };
        match warning {
            ValidationWarning::UnusualValue { field, value } => {
                assert_eq!(field, "amount");
                assert_eq!(value, "999999999");
            }
            _ => panic!("Wrong warning type"),
        }

        let warning = ValidationWarning::DeprecatedField {
            field: "old_field".to_string(),
        };
        match warning {
            ValidationWarning::DeprecatedField { field } => {
                assert_eq!(field, "old_field");
            }
            _ => panic!("Wrong warning type"),
        }

        let warning = ValidationWarning::PerformanceWarning {
            description: "high computational cost".to_string(),
        };
        match warning {
            ValidationWarning::PerformanceWarning { description } => {
                assert_eq!(description, "high computational cost");
            }
            _ => panic!("Wrong warning type"),
        }
    }

    #[test]
    fn test_enrichment_error_types_available() {
        // Test that enrichment error types are accessible through re-exports

        // Test TaskError variant
        let error = EnrichmentError::TaskError("task failed".to_string());
        match error {
            EnrichmentError::TaskError(msg) => {
                assert_eq!(msg, "task failed");
            }
            _ => panic!("Wrong error type"),
        }

        // Test CacheError variant
        let error = EnrichmentError::CacheError("cache full".to_string());
        match error {
            EnrichmentError::CacheError(msg) => {
                assert_eq!(msg, "cache full");
            }
            _ => panic!("Wrong error type"),
        }

        // Test RateLimitExceeded variant
        let error = EnrichmentError::RateLimitExceeded;
        match error {
            EnrichmentError::RateLimitExceeded => {
                // Expected variant
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_parsing_metrics_available() {
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
        assert_eq!(parser_metrics.success_rate, 0.0);
    }

    #[test]
    fn test_validation_metrics_available() {
        // Test that validation metrics structures are accessible
        let metrics = ValidationMetrics::default();
        assert_eq!(metrics.validation_time, Duration::from_secs(0));
        assert_eq!(metrics.rules_checked, 0);
        assert_eq!(metrics.consistency_checks, 0);
    }

    #[test]
    fn test_cache_stats_available() {
        // Test that cache statistics structure is accessible
        let stats = CacheStats {
            token_cache_size: 10,
            price_cache_size: 5,
            token_cache_hit_rate: 0.85,
            price_cache_hit_rate: 0.92,
        };
        assert_eq!(stats.token_cache_size, 10);
        assert_eq!(stats.price_cache_size, 5);
        assert_eq!(stats.token_cache_hit_rate, 0.85);
        assert_eq!(stats.price_cache_hit_rate, 0.92);
    }

    #[test]
    fn test_pipeline_error_types_available() {
        // Test that pipeline error types are accessible
        let error = PipelineError::SemaphoreError(());
        match error {
            PipelineError::SemaphoreError(_) => {
                // Expected variant
            }
            _ => panic!("Wrong error type"),
        }

        let error = PipelineError::ChannelError;
        match error {
            PipelineError::ChannelError => {
                // Expected variant
            }
            _ => panic!("Wrong error type"),
        }

        let error = PipelineError::ConfigError("invalid config".to_string());
        match error {
            PipelineError::ConfigError(msg) => {
                assert_eq!(msg, "invalid config");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_all_structs_are_cloneable_where_expected() {
        // Test that structures that should be cloneable actually are
        let enrichment_config = EnrichmentConfig::default();
        let _cloned_enrichment_config = enrichment_config.clone();

        let parsing_config = ParsingPipelineConfig::default();
        let _cloned_parsing_config = parsing_config.clone();

        let validation_config = ValidationConfig::default();
        let _cloned_validation_config = validation_config.clone();

        let parser_config = ParserConfig::default();
        let _cloned_parser_config = parser_config.clone();

        let cache_stats = CacheStats {
            token_cache_size: 0,
            price_cache_size: 0,
            token_cache_hit_rate: 0.0,
            price_cache_hit_rate: 0.0,
        };
        let _cloned_cache_stats = cache_stats.clone();

        let validation_metrics = ValidationMetrics::default();
        let _cloned_validation_metrics = validation_metrics.clone();

        let parsing_metrics = ParsingMetrics::default();
        let _cloned_parsing_metrics = parsing_metrics.clone();

        let parser_metrics = ParserMetrics::default();
        let _cloned_parser_metrics = parser_metrics.clone();
    }

    #[test]
    fn test_all_structs_are_debuggable() {
        // Test that all major structures implement Debug trait
        let enrichment_config = EnrichmentConfig::default();
        let debug_str = format!("{:?}", enrichment_config);
        assert!(debug_str.contains("EnrichmentConfig"));

        let parsing_config = ParsingPipelineConfig::default();
        let debug_str = format!("{:?}", parsing_config);
        assert!(debug_str.contains("ParsingPipelineConfig"));

        let validation_config = ValidationConfig::default();
        let debug_str = format!("{:?}", validation_config);
        assert!(debug_str.contains("ValidationConfig"));

        let parser_config = ParserConfig::default();
        let debug_str = format!("{:?}", parser_config);
        assert!(debug_str.contains("ParserConfig"));

        let cache_stats = CacheStats {
            token_cache_size: 0,
            price_cache_size: 0,
            token_cache_hit_rate: 0.0,
            price_cache_hit_rate: 0.0,
        };
        let debug_str = format!("{:?}", cache_stats);
        assert!(debug_str.contains("CacheStats"));
    }

    #[test]
    fn test_module_re_exports_work_correctly() {
        // Comprehensive test to ensure all re-exports from sub-modules are working

        // From enrichment module
        let _: EnrichmentConfig = EnrichmentConfig::default();
        let _: EventEnricher = EventEnricher::new(EnrichmentConfig::default());
        let _: CacheStats = CacheStats {
            token_cache_size: 0,
            price_cache_size: 0,
            token_cache_hit_rate: 0.0,
            price_cache_hit_rate: 0.0,
        };

        // From parsing module
        let _: ParsingPipelineConfig = ParsingPipelineConfig::default();
        let _: ParsingPipeline = ParsingPipeline::new(ParsingPipelineConfig::default());
        let _: ParsingPipelineBuilder = ParsingPipelineBuilder::new();
        let _: ParserConfig = ParserConfig::default();
        let _: ParsingMetrics = ParsingMetrics::default();
        let _: ParserMetrics = ParserMetrics::default();

        // From validation module
        let _: ValidationConfig = ValidationConfig::default();
        let _: ValidationPipeline = ValidationPipeline::new(ValidationConfig::default());
        let _: ValidationMetrics = ValidationMetrics::default();

        // This test ensures that all the main types from each module
        // are correctly re-exported and accessible
    }
}
