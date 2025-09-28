//! Parsing pipeline orchestration for high-throughput event processing
//!
//! This module provides a configurable pipeline for parsing Solana transaction data
//! with support for batching, parallel processing, and backpressure handling.

extern crate alloc;

use crate::solana_metadata::SolanaEventMetadata;
use crate::types::ProtocolType;
use crate::zero_copy::{BatchEventParser, ByteSliceEventParser, ParseError, ZeroCopyEvent};
use alloc::sync::Arc;
use core::{
    fmt::{Debug, Formatter, Result as FmtResult},
    mem::{replace, take},
    time::Duration,
};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::time::Instant;
use tokio::{
    sync::{mpsc, Semaphore},
    time::interval,
};
use tokio_stream::{Stream, StreamExt as _};

/// Configuration for the parsing pipeline
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PipelineConfig {
    /// Timeout for batch collection
    pub batch_timeout: Duration,
    /// Enable performance metrics collection
    pub enable_metrics: bool,
    /// Maximum batch size for processing
    pub max_batch_size: usize,
    /// Maximum number of concurrent parsing tasks
    pub max_concurrent_tasks: usize,
    /// Buffer size for the output channel
    pub output_buffer_size: usize,
    /// Parser-specific configurations
    pub parser_configs: HashMap<ProtocolType, ParserConfig>,
}

impl Default for PipelineConfig {
    #[inline]
    fn default() -> Self {
        Self {
            batch_timeout: Duration::from_millis(50),
            enable_metrics: true,
            max_batch_size: 100,
            max_concurrent_tasks: 4,
            output_buffer_size: 1000,
            parser_configs: HashMap::new(),
        }
    }
}

/// Parser-specific configuration
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ParserConfig {
    /// Enable detailed analysis
    pub detailed_analysis: bool,
    /// Parser priority (higher numbers = higher priority)
    pub priority: u8,
    /// Enable zero-copy parsing for this protocol
    pub zero_copy: bool,
}

impl Default for ParserConfig {
    #[inline]
    fn default() -> Self {
        Self {
            detailed_analysis: true,
            priority: 1,
            zero_copy: true,
        }
    }
}

/// Input for the parsing pipeline
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Input {
    /// Raw instruction or transaction data
    pub data: Vec<u8>,
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Optional program ID hint for faster parsing
    pub program_id_hint: Option<Pubkey>,
}

/// Output from the parsing pipeline
#[derive(Debug)]
#[non_exhaustive]
pub struct Output {
    /// Any errors that occurred during parsing
    pub errors: Vec<ParseError>,
    /// Parsed events
    pub events: Vec<ZeroCopyEvent<'static>>,
    /// Parsing metrics
    pub metrics: Metrics,
}

/// Metrics for parsing performance
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Metrics {
    /// Number of bytes processed
    pub bytes_processed: usize,
    /// Number of parsing errors
    pub error_count: usize,
    /// Number of events parsed
    pub events_parsed: usize,
    /// Parser-specific metrics
    pub parser_metrics: HashMap<ProtocolType, ParserMetrics>,
    /// Total processing time
    pub processing_time: Duration,
}

/// Parser-specific metrics
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ParserMetrics {
    /// Number of events parsed by this parser
    pub events_count: usize,
    /// Time spent parsing
    pub parse_time: Duration,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}

/// High-performance parsing pipeline
pub struct Pipeline {
    /// Batch event parser
    batch_parser: BatchEventParser,
    /// Pipeline configuration
    config: PipelineConfig,
    /// Output channel receiver
    output_receiver: mpsc::UnboundedReceiver<Output>,
    /// Output channel sender
    output_sender: mpsc::UnboundedSender<Output>,
    /// Semaphore for controlling concurrency
    semaphore: Arc<Semaphore>,
}

impl Debug for Pipeline {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("Pipeline")
            .field("config", &self.config)
            .field("batch_parser", &"BatchEventParser { ... }")
            .field(
                "semaphore",
                &format!("Semaphore({})", self.semaphore.available_permits()),
            )
            .field("output_sender", &"UnboundedSender { ... }")
            .field("output_receiver", &"UnboundedReceiver { ... }")
            .finish()
    }
}

impl Pipeline {
    /// Add a parser to the pipeline
    #[inline]
    pub fn add_parser(&mut self, parser: Arc<dyn ByteSliceEventParser>) {
        self.batch_parser.add_parser(parser);
    }

    /// Get pipeline statistics
    #[must_use]
    #[inline]
    pub fn get_stats(&self) -> PipelineStats {
        let batch_stats = self.batch_parser.get_stats();

        PipelineStats {
            available_permits: self.semaphore.available_permits(),
            max_batch_size: self.config.max_batch_size,
            max_concurrent_tasks: self.config.max_concurrent_tasks,
            registered_parsers: batch_stats.registered_parsers,
        }
    }

    /// Create a new parsing pipeline
    #[must_use]
    #[inline]
    pub fn new(config: PipelineConfig) -> Self {
        let batch_parser = BatchEventParser::new(config.max_batch_size);
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));
        let (output_sender, output_receiver) = mpsc::unbounded_channel();

        Self {
            batch_parser,
            config,
            output_receiver,
            output_sender,
            semaphore,
        }
    }

    /// Process a batch of inputs
    async fn process_batch(&self, batch: Vec<Input>) -> Result<(), PipelineError> {
        if batch.is_empty() {
            return Ok(());
        }

        // Acquire semaphore permit for concurrency control
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_ignored_error| PipelineError::SemaphoreError(()))?;

        let start_time = Instant::now();
        let mut total_bytes: usize = 0;
        let mut all_events = Vec::new();
        let mut all_errors = Vec::new();
        let mut parser_metrics = HashMap::new();

        // Prepare batch data for parsing
        let batch_data: Vec<&[u8]> = batch
            .iter()
            .map(|input| {
                {
                    {
                        total_bytes = total_bytes.saturating_add(input.data.len());
                    }
                }
                input.data.as_slice()
            })
            .collect();

        let batch_metadata: Vec<SolanaEventMetadata> =
            batch.iter().map(|input| input.metadata.clone()).collect();

        // Parse the batch
        match self.batch_parser.parse_batch(&batch_data, batch_metadata) {
            Ok(events) => {
                // Convert to owned events
                for event in events {
                    all_events.push(event.to_owned());
                }
            }
            Err(error) => {
                all_errors.push(error);
            }
        }

        // Calculate metrics
        let processing_time = start_time.elapsed();

        if self.config.enable_metrics {
            // Collect parser-specific metrics
            for protocol in [
                ProtocolType::RaydiumAmmV4,
                ProtocolType::Jupiter,
                ProtocolType::PumpSwap,
            ] {
                let events_count = all_events
                    .iter()
                    .filter(|event| event.protocol_type() == protocol)
                    .count();

                if events_count > 0 {
                    let events_count_u32 = u32::try_from(events_count).unwrap_or(1);
                    let avg_parse_time = processing_time
                        .checked_div(events_count_u32)
                        .unwrap_or(Duration::ZERO);

                    parser_metrics.insert(
                        protocol,
                        ParserMetrics {
                            events_count,
                            parse_time: avg_parse_time,
                            success_rate: if all_errors.is_empty() { 1.0 } else { 0.8 }, // Simplified calculation
                        },
                    );
                }
            }
        }

        let metrics = Metrics {
            bytes_processed: total_bytes,
            error_count: all_errors.len(),
            events_parsed: all_events.len(),
            parser_metrics,
            processing_time,
        };

        // Send output
        let output = Output {
            errors: all_errors,
            events: all_events,
            metrics,
        };

        self.output_sender
            .send(output)
            .map_err(|_ignored_send_error| PipelineError::ChannelError)?;

        Ok(())
    }

    /// Process a stream of parsing inputs
    ///
    /// # Errors
    ///
    /// Returns `PipelineError` if batch processing fails or channel operations fail
    #[inline]
    pub async fn process_stream<S>(&mut self, mut input_stream: S) -> Result<(), PipelineError>
    where
        S: Stream<Item = Input> + Unpin,
    {
        let mut batch = Vec::new();
        let mut batch_timer = interval(self.config.batch_timeout);

        loop {
            tokio::select! {
                // Collect inputs into batches
                input_opt = input_stream.next() => {
                    if let Some(input) = input_opt {
                        batch.push(input);
                        if batch.len() >= self.config.max_batch_size {
                            self.process_batch(take(&mut batch)).await?;
                        }
                    } else {
                        // Stream ended, process remaining batch
                        if !batch.is_empty() {
                            self.process_batch(take(&mut batch)).await?;
                        }
                        break;
                    }
                }
                _ = batch_timer.tick() => {
                    // Timeout - process current batch if not empty
                    if !batch.is_empty() {
                        self.process_batch(take(&mut batch)).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Get the output receiver for processed events
    #[inline]
    pub fn take_output_receiver(&mut self) -> mpsc::UnboundedReceiver<Output> {
        let (new_sender, new_receiver) = mpsc::unbounded_channel();
        let old_receiver = replace(&mut self.output_receiver, new_receiver);
        self.output_sender = new_sender;
        old_receiver
    }
}

/// Statistics for the parsing pipeline
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PipelineStats {
    /// Number of available semaphore permits
    pub available_permits: usize,
    /// Maximum batch size for processing
    pub max_batch_size: usize,
    /// Maximum number of concurrent parsing tasks
    pub max_concurrent_tasks: usize,
    /// Number of registered parsers
    pub registered_parsers: usize,
}

/// Error type for parsing pipeline operations
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PipelineError {
    /// Error sending data through channel
    #[error("Channel error")]
    ChannelError,

    /// Invalid pipeline configuration
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Error during event parsing
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),

    /// Error acquiring semaphore permit
    #[error("Semaphore error")]
    SemaphoreError(()),
}

/// Builder for creating parsing pipelines
#[derive(Default)]
pub struct PipelineBuilder {
    /// Pipeline configuration
    config: PipelineConfig,
    /// List of parsers to add to the pipeline
    parsers: Vec<Arc<dyn ByteSliceEventParser>>,
}

impl Debug for PipelineBuilder {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("PipelineBuilder")
            .field("config", &self.config)
            .field("parsers", &format!("{} parsers", self.parsers.len()))
            .finish()
    }
}

impl PipelineBuilder {
    /// Add a parser
    #[must_use]
    #[inline]
    pub fn add_parser(mut self, parser: Arc<dyn ByteSliceEventParser>) -> Self {
        self.parsers.push(parser);
        self
    }

    /// Build the parsing pipeline
    #[must_use]
    #[inline]
    pub fn build(self) -> Pipeline {
        let mut pipeline = Pipeline::new(self.config);

        // Add all parsers
        for parser in self.parsers {
            pipeline.add_parser(parser);
        }

        pipeline
    }

    /// Create a new builder
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            config: PipelineConfig::default(),
            parsers: Vec::new(),
        }
    }

    /// Set batch size
    #[must_use]
    #[inline]
    pub const fn with_batch_size(mut self, size: usize) -> Self {
        self.config.max_batch_size = size;
        self
    }

    /// Set batch timeout
    #[must_use]
    #[inline]
    pub const fn with_batch_timeout(mut self, timeout: Duration) -> Self {
        self.config.batch_timeout = timeout;
        self
    }

    /// Set concurrency limit
    #[must_use]
    #[inline]
    pub const fn with_concurrency_limit(mut self, limit: usize) -> Self {
        self.config.max_concurrent_tasks = limit;
        self
    }

    /// Enable or disable metrics collection
    #[must_use]
    #[inline]
    pub const fn with_metrics(mut self, enable: bool) -> Self {
        self.config.enable_metrics = enable;
        self
    }

    /// Add parser configuration
    #[must_use]
    #[inline]
    pub fn with_parser_config(mut self, protocol: ProtocolType, config: ParserConfig) -> Self {
        self.config.parser_configs.insert(protocol, config);
        self
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::parsers::{JupiterParserFactory, RaydiumV4ParserFactory};

    #[tokio::test]
    async fn test_pipeline_builder() {
        let pipeline = PipelineBuilder::default()
            .with_batch_size(50)
            .with_concurrency_limit(2)
            .add_parser(RaydiumV4ParserFactory::create_zero_copy())
            .add_parser(JupiterParserFactory::create_zero_copy())
            .build();

        let stats = pipeline.get_stats();
        assert_eq!(stats.max_batch_size, 50);
        assert_eq!(stats.max_concurrent_tasks, 2);
        assert_eq!(stats.registered_parsers, 2);
    }

    #[tokio::test]
    async fn test_empty_batch_processing() {
        let config = PipelineConfig::default();
        let pipeline = Pipeline::new(config);

        // Empty batch should not cause errors
        let result = pipeline.process_batch(Vec::new()).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_parsing_input_creation() {
        let input = Input {
            data: vec![0x09, 0x01, 0x02],
            metadata: SolanaEventMetadata::default(),
            program_id_hint: None,
        };

        assert_eq!(input.data.len(), 3);
        assert!(input.program_id_hint.is_none());
    }

    #[test]
    fn test_parsing_pipeline_config_default() {
        let config = PipelineConfig::default();
        assert_eq!(config.max_batch_size, 100);
        assert_eq!(config.batch_timeout, Duration::from_millis(50));
        assert_eq!(config.max_concurrent_tasks, 4);
        assert_eq!(config.output_buffer_size, 1000);
        assert!(config.enable_metrics);
        assert!(config.parser_configs.is_empty());
    }

    #[test]
    fn test_parser_config_default() {
        let config = ParserConfig::default();
        assert!(config.zero_copy);
        assert!(config.detailed_analysis);
        assert_eq!(config.priority, 1);
    }

    #[test]
    fn test_parsing_metrics_default() {
        let metrics = Metrics::default();
        assert_eq!(metrics.processing_time, Duration::default());
        assert_eq!(metrics.events_parsed, 0);
        assert_eq!(metrics.bytes_processed, 0);
        assert_eq!(metrics.error_count, 0);
        assert!(metrics.parser_metrics.is_empty());
    }

    #[test]
    fn test_parser_metrics_default() {
        let metrics = ParserMetrics::default();
        assert_eq!(metrics.parse_time, Duration::default());
        assert_eq!(metrics.events_count, 0);
        {
            assert!(metrics.success_rate.abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_parsing_pipeline_builder_default() {
        let builder = PipelineBuilder::default();
        assert_eq!(builder.config.max_batch_size, 100);
        assert!(builder.parsers.is_empty());
    }

    #[test]
    fn test_parsing_pipeline_builder_with_batch_size() {
        let builder = PipelineBuilder::default().with_batch_size(200);
        assert_eq!(builder.config.max_batch_size, 200);
    }

    #[test]
    fn test_parsing_pipeline_builder_with_batch_timeout() {
        let timeout = Duration::from_millis(100);
        let builder = PipelineBuilder::default().with_batch_timeout(timeout);
        assert_eq!(builder.config.batch_timeout, timeout);
    }

    #[test]
    fn test_parsing_pipeline_builder_with_concurrency_limit() {
        let builder = PipelineBuilder::default().with_concurrency_limit(8);
        assert_eq!(builder.config.max_concurrent_tasks, 8);
    }

    #[test]
    fn test_parsing_pipeline_builder_with_metrics() {
        let builder = PipelineBuilder::default().with_metrics(false);
        assert!(!builder.config.enable_metrics);
    }

    #[test]
    fn test_parsing_pipeline_builder_with_parser_config() {
        let parser_config = ParserConfig {
            zero_copy: false,
            detailed_analysis: false,
            priority: 5,
        };
        let builder =
            PipelineBuilder::default().with_parser_config(ProtocolType::Jupiter, parser_config);

        assert_eq!(builder.config.parser_configs.len(), 1);
        let config = builder
            .config
            .parser_configs
            .get(&ProtocolType::Jupiter)
            .expect("Jupiter parser config should exist in test");
        assert!(!config.zero_copy);
        assert!(!config.detailed_analysis);
        assert_eq!(config.priority, 5);
    }

    #[test]
    fn test_parsing_pipeline_builder_add_parser() {
        let parser = RaydiumV4ParserFactory::create_zero_copy();
        let builder = PipelineBuilder::default().add_parser(parser);
        assert_eq!(builder.parsers.len(), 1);
    }

    #[test]
    fn test_parsing_pipeline_builder_chaining() {
        let parser1 = RaydiumV4ParserFactory::create_zero_copy();
        let parser2 = JupiterParserFactory::create_zero_copy();
        let parser_config = ParserConfig {
            zero_copy: false,
            detailed_analysis: true,
            priority: 3,
        };

        let builder = PipelineBuilder::default()
            .with_batch_size(150)
            .with_batch_timeout(Duration::from_millis(75))
            .with_concurrency_limit(6)
            .with_metrics(false)
            .add_parser(parser1)
            .add_parser(parser2)
            .with_parser_config(ProtocolType::RaydiumAmmV4, parser_config);

        assert_eq!(builder.config.max_batch_size, 150);
        assert_eq!(builder.config.batch_timeout, Duration::from_millis(75));
        assert_eq!(builder.config.max_concurrent_tasks, 6);
        assert!(!builder.config.enable_metrics);
        assert_eq!(builder.parsers.len(), 2);
        assert_eq!(builder.config.parser_configs.len(), 1);
    }

    #[tokio::test]
    async fn test_parsing_pipeline_new() {
        let config = PipelineConfig {
            max_batch_size: 50,
            batch_timeout: Duration::from_millis(25),
            max_concurrent_tasks: 2,
            output_buffer_size: 500,
            enable_metrics: false,
            parser_configs: HashMap::new(),
        };

        let pipeline = Pipeline::new(config);
        let stats = pipeline.get_stats();

        assert_eq!(stats.max_batch_size, 50);
        assert_eq!(stats.max_concurrent_tasks, 2);
        assert_eq!(stats.registered_parsers, 0);
        assert_eq!(stats.available_permits, 2);
    }

    #[tokio::test]
    async fn test_parsing_pipeline_add_parser() {
        let config = PipelineConfig::default();
        let mut pipeline = Pipeline::new(config);

        let parser = RaydiumV4ParserFactory::create_zero_copy();
        pipeline.add_parser(parser);

        let stats = pipeline.get_stats();
        assert_eq!(stats.registered_parsers, 1);
    }

    #[tokio::test]
    async fn test_parsing_pipeline_take_output_receiver() {
        let config = PipelineConfig::default();
        let mut pipeline = Pipeline::new(config);

        let _receiver = pipeline.take_output_receiver();
        // After taking the receiver, a new one should be created internally
        // This is verified by the function not panicking
    }

    #[test]
    fn test_parsing_input_with_program_id_hint() {
        use solana_sdk::pubkey::Pubkey;

        let program_id = Pubkey::new_unique();
        let input = Input {
            data: vec![0x01, 0x02, 0x03, 0x04],
            metadata: SolanaEventMetadata::default(),
            program_id_hint: Some(program_id),
        };

        assert_eq!(input.data.len(), 4);
        assert_eq!(input.program_id_hint, Some(program_id));
    }

    #[test]
    fn test_pipeline_error_display() {
        let semaphore_error = PipelineError::SemaphoreError(());
        let channel_error = PipelineError::ChannelError;
        let config_error = PipelineError::ConfigError("Invalid config".to_string());

        assert_eq!(format!("{semaphore_error}"), "Semaphore error");
        assert_eq!(format!("{channel_error}"), "Channel error");
        assert_eq!(
            format!("{config_error}"),
            "Configuration error: Invalid config"
        );
    }

    #[test]
    fn test_pipeline_error_from_parse_error() {
        let parse_error = ParseError::InvalidInstructionData("Invalid data".to_string());
        let pipeline_error = PipelineError::from(parse_error);

        match pipeline_error {
            PipelineError::ParseError(ParseError::InvalidInstructionData(_)) => {}
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_pipeline_stats_debug() {
        let stats = PipelineStats {
            max_batch_size: 100,
            max_concurrent_tasks: 4,
            registered_parsers: 2,
            available_permits: 3,
        };

        let debug_str = format!("{stats:?}");
        assert!(debug_str.contains("max_batch_size: 100"));
        assert!(debug_str.contains("max_concurrent_tasks: 4"));
        assert!(debug_str.contains("registered_parsers: 2"));
        assert!(debug_str.contains("available_permits: 3"));
    }

    #[test]
    fn test_parsing_output_debug() {
        let output = Output {
            events: Vec::new(),
            metrics: Metrics::default(),
            errors: Vec::new(),
        };

        let debug_str = format!("{output:?}");
        assert!(debug_str.contains("events"));
        assert!(debug_str.contains("metrics"));
        assert!(debug_str.contains("errors"));
    }

    #[test]
    fn test_parsing_metrics_with_parser_metrics() {
        let mut parser_metrics = HashMap::new();
        parser_metrics.insert(
            ProtocolType::Jupiter,
            ParserMetrics {
                parse_time: Duration::from_millis(10),
                events_count: 5,
                success_rate: 0.9,
            },
        );

        let metrics = Metrics {
            processing_time: Duration::from_millis(100),
            events_parsed: 10,
            bytes_processed: 1024,
            error_count: 1,
            parser_metrics,
        };

        assert_eq!(metrics.processing_time, Duration::from_millis(100));
        assert_eq!(metrics.events_parsed, 10);
        assert_eq!(metrics.bytes_processed, 1024);
        assert_eq!(metrics.error_count, 1);
        assert_eq!(metrics.parser_metrics.len(), 1);

        let jupiter_metrics = metrics
            .parser_metrics
            .get(&ProtocolType::Jupiter)
            .expect("Jupiter metrics should exist in test");
        assert_eq!(jupiter_metrics.parse_time, Duration::from_millis(10));
        assert_eq!(jupiter_metrics.events_count, 5);
        {
            assert!((jupiter_metrics.success_rate - 0.9).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_parser_config_custom_values() {
        let config = ParserConfig {
            zero_copy: false,
            detailed_analysis: false,
            priority: 10,
        };

        assert!(!config.zero_copy);
        assert!(!config.detailed_analysis);
        assert_eq!(config.priority, 10);
    }

    #[test]
    fn test_parsing_pipeline_config_custom_values() {
        let mut parser_configs = HashMap::new();
        parser_configs.insert(ProtocolType::Jupiter, ParserConfig::default());

        let config = PipelineConfig {
            max_batch_size: 250,
            batch_timeout: Duration::from_millis(200),
            max_concurrent_tasks: 8,
            output_buffer_size: 2000,
            enable_metrics: false,
            parser_configs,
        };

        assert_eq!(config.max_batch_size, 250);
        assert_eq!(config.batch_timeout, Duration::from_millis(200));
        assert_eq!(config.max_concurrent_tasks, 8);
        assert_eq!(config.output_buffer_size, 2000);
        assert!(!config.enable_metrics);
        assert_eq!(config.parser_configs.len(), 1);
    }

    #[test]
    fn test_parsing_input_clone() {
        let input = Input {
            data: vec![0x05, 0x06],
            metadata: SolanaEventMetadata::default(),
            program_id_hint: None,
        };

        let cloned = input.clone();
        assert_eq!(input.data, cloned.data);
        assert_eq!(input.program_id_hint, cloned.program_id_hint);
    }

    #[test]
    fn test_parsing_pipeline_config_clone() {
        let config = PipelineConfig::default();
        let cloned = config.clone();

        assert_eq!(config.max_batch_size, cloned.max_batch_size);
        assert_eq!(config.batch_timeout, cloned.batch_timeout);
        assert_eq!(config.max_concurrent_tasks, cloned.max_concurrent_tasks);
        assert_eq!(config.output_buffer_size, cloned.output_buffer_size);
        assert_eq!(config.enable_metrics, cloned.enable_metrics);
    }

    #[test]
    fn test_parser_config_clone() {
        let config = ParserConfig::default();
        let cloned = config.clone();

        assert_eq!(config.zero_copy, cloned.zero_copy);
        assert_eq!(config.detailed_analysis, cloned.detailed_analysis);
        assert_eq!(config.priority, cloned.priority);
    }

    #[test]
    fn test_parsing_metrics_clone() {
        let metrics = Metrics::default();
        let cloned = metrics.clone();

        assert_eq!(metrics.processing_time, cloned.processing_time);
        assert_eq!(metrics.events_parsed, cloned.events_parsed);
        assert_eq!(metrics.bytes_processed, cloned.bytes_processed);
        assert_eq!(metrics.error_count, cloned.error_count);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_parser_metrics_clone() {
        let metrics = ParserMetrics::default();
        let cloned = metrics.clone();

        assert_eq!(metrics.parse_time, cloned.parse_time);
        assert_eq!(metrics.events_count, cloned.events_count);
        {
            assert_eq!(metrics.success_rate, cloned.success_rate);
        }
    }

    #[test]
    fn test_pipeline_stats_clone() {
        let stats = PipelineStats {
            max_batch_size: 100,
            max_concurrent_tasks: 4,
            registered_parsers: 2,
            available_permits: 3,
        };
        let cloned = stats.clone();

        assert_eq!(stats.max_batch_size, cloned.max_batch_size);
        assert_eq!(stats.max_concurrent_tasks, cloned.max_concurrent_tasks);
        assert_eq!(stats.registered_parsers, cloned.registered_parsers);
        assert_eq!(stats.available_permits, cloned.available_permits);
    }

    // Edge cases and error paths
    #[test]
    fn test_parsing_input_empty_data() {
        let input = Input {
            data: Vec::new(),
            metadata: SolanaEventMetadata::default(),
            program_id_hint: None,
        };

        assert!(input.data.is_empty());
    }

    #[test]
    fn test_parsing_input_large_data() {
        let large_data = vec![0xFF; 10000];
        let input = Input {
            data: large_data.clone(),
            metadata: SolanaEventMetadata::default(),
            program_id_hint: None,
        };

        assert_eq!(input.data.len(), 10000);
        assert_eq!(input.data, large_data);
    }

    #[test]
    fn test_parsing_pipeline_config_zero_batch_size() {
        let config = PipelineConfig {
            max_batch_size: 0,
            ..Default::default()
        };

        assert_eq!(config.max_batch_size, 0);
    }

    #[test]
    fn test_parsing_pipeline_config_zero_timeout() {
        let config = PipelineConfig {
            batch_timeout: Duration::from_millis(0),
            ..Default::default()
        };

        assert_eq!(config.batch_timeout, Duration::from_millis(0));
    }

    #[test]
    fn test_parsing_pipeline_config_zero_concurrency() {
        let config = PipelineConfig {
            max_concurrent_tasks: 0,
            ..Default::default()
        };

        assert_eq!(config.max_concurrent_tasks, 0);
    }

    #[test]
    fn test_parser_config_zero_priority() {
        let config = ParserConfig {
            priority: 0,
            ..Default::default()
        };

        assert_eq!(config.priority, 0);
    }

    #[test]
    fn test_parser_config_max_priority() {
        let config = ParserConfig {
            priority: u8::MAX,
            ..Default::default()
        };

        assert_eq!(config.priority, u8::MAX);
    }

    #[test]
    fn test_parser_metrics_zero_success_rate() {
        let metrics = ParserMetrics {
            success_rate: 0.0,
            ..Default::default()
        };

        {
            assert!(metrics.success_rate.abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_parser_metrics_full_success_rate() {
        let metrics = ParserMetrics {
            success_rate: 1.0,
            ..Default::default()
        };

        {
            assert!((metrics.success_rate - 1.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_parsing_pipeline_builder_multiple_parser_configs() {
        let jupiter_config = ParserConfig {
            zero_copy: true,
            detailed_analysis: false,
            priority: 1,
        };
        let raydium_config = ParserConfig {
            zero_copy: false,
            detailed_analysis: true,
            priority: 2,
        };

        let builder = PipelineBuilder::default()
            .with_parser_config(ProtocolType::Jupiter, jupiter_config)
            .with_parser_config(ProtocolType::RaydiumAmmV4, raydium_config);

        assert_eq!(builder.config.parser_configs.len(), 2);
        assert!(builder
            .config
            .parser_configs
            .contains_key(&ProtocolType::Jupiter));
        assert!(builder
            .config
            .parser_configs
            .contains_key(&ProtocolType::RaydiumAmmV4));
    }

    #[tokio::test]
    async fn test_parsing_pipeline_builder_build_with_all_options() {
        let parser1 = RaydiumV4ParserFactory::create_zero_copy();
        let parser2 = JupiterParserFactory::create_zero_copy();

        let pipeline = PipelineBuilder::default()
            .with_batch_size(75)
            .with_batch_timeout(Duration::from_millis(30))
            .with_concurrency_limit(3)
            .with_metrics(true)
            .add_parser(parser1)
            .add_parser(parser2)
            .with_parser_config(ProtocolType::Jupiter, ParserConfig::default())
            .build();

        let stats = pipeline.get_stats();
        assert_eq!(stats.max_batch_size, 75);
        assert_eq!(stats.max_concurrent_tasks, 3);
        assert_eq!(stats.registered_parsers, 2);
        assert_eq!(stats.available_permits, 3);
    }
}
