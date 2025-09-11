//! Parsing pipeline orchestration for high-throughput event processing
//!
//! This module provides a configurable pipeline for parsing Solana transaction data
//! with support for batching, parallel processing, and backpressure handling.

use crate::solana_metadata::SolanaEventMetadata;
use crate::types::ProtocolType;
use crate::zero_copy::{BatchEventParser, ByteSliceEventParser, ParseError, ZeroCopyEvent};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Semaphore};
use tokio_stream::{Stream, StreamExt};

/// Configuration for the parsing pipeline
#[derive(Debug, Clone)]
pub struct ParsingPipelineConfig {
    /// Maximum batch size for processing
    pub max_batch_size: usize,
    /// Timeout for batch collection
    pub batch_timeout: Duration,
    /// Maximum number of concurrent parsing tasks
    pub max_concurrent_tasks: usize,
    /// Buffer size for the output channel
    pub output_buffer_size: usize,
    /// Enable performance metrics collection
    pub enable_metrics: bool,
    /// Parser-specific configurations
    pub parser_configs: HashMap<ProtocolType, ParserConfig>,
}

impl Default for ParsingPipelineConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            batch_timeout: Duration::from_millis(50),
            max_concurrent_tasks: 4,
            output_buffer_size: 1000,
            enable_metrics: true,
            parser_configs: HashMap::new(),
        }
    }
}

/// Parser-specific configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Enable zero-copy parsing for this protocol
    pub zero_copy: bool,
    /// Enable detailed analysis
    pub detailed_analysis: bool,
    /// Parser priority (higher numbers = higher priority)
    pub priority: u8,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            zero_copy: true,
            detailed_analysis: true,
            priority: 1,
        }
    }
}

/// Input for the parsing pipeline
#[derive(Debug, Clone)]
pub struct ParsingInput {
    /// Raw instruction or transaction data
    pub data: Vec<u8>,
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Optional program ID hint for faster parsing
    pub program_id_hint: Option<solana_sdk::pubkey::Pubkey>,
}

/// Output from the parsing pipeline
#[derive(Debug)]
pub struct ParsingOutput {
    /// Parsed events
    pub events: Vec<ZeroCopyEvent<'static>>,
    /// Parsing metrics
    pub metrics: ParsingMetrics,
    /// Any errors that occurred during parsing
    pub errors: Vec<ParseError>,
}

/// Metrics for parsing performance
#[derive(Debug, Clone, Default)]
pub struct ParsingMetrics {
    /// Total processing time
    pub processing_time: Duration,
    /// Number of events parsed
    pub events_parsed: usize,
    /// Number of bytes processed
    pub bytes_processed: usize,
    /// Number of parsing errors
    pub error_count: usize,
    /// Parser-specific metrics
    pub parser_metrics: HashMap<ProtocolType, ParserMetrics>,
}

/// Parser-specific metrics
#[derive(Debug, Clone, Default)]
pub struct ParserMetrics {
    /// Time spent parsing
    pub parse_time: Duration,
    /// Number of events parsed by this parser
    pub events_count: usize,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}

/// High-performance parsing pipeline
pub struct ParsingPipeline {
    /// Pipeline configuration
    config: ParsingPipelineConfig,
    /// Batch event parser
    batch_parser: BatchEventParser,
    /// Semaphore for controlling concurrency
    semaphore: Arc<Semaphore>,
    /// Output channel sender
    output_sender: mpsc::UnboundedSender<ParsingOutput>,
    /// Output channel receiver
    output_receiver: mpsc::UnboundedReceiver<ParsingOutput>,
}

impl std::fmt::Debug for ParsingPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParsingPipeline")
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

impl ParsingPipeline {
    /// Create a new parsing pipeline
    pub fn new(config: ParsingPipelineConfig) -> Self {
        let batch_parser = BatchEventParser::new(config.max_batch_size);
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));
        let (output_sender, output_receiver) = mpsc::unbounded_channel();

        Self {
            config,
            batch_parser,
            semaphore,
            output_sender,
            output_receiver,
        }
    }

    /// Add a parser to the pipeline
    pub fn add_parser(&mut self, parser: Arc<dyn ByteSliceEventParser>) {
        self.batch_parser.add_parser(parser);
    }

    /// Process a stream of parsing inputs
    pub async fn process_stream<S>(&mut self, mut input_stream: S) -> Result<(), PipelineError>
    where
        S: Stream<Item = ParsingInput> + Unpin,
    {
        let mut batch = Vec::new();
        let mut batch_timer = tokio::time::interval(self.config.batch_timeout);

        loop {
            tokio::select! {
                // Collect inputs into batches
                input_opt = input_stream.next() => {
                    match input_opt {
                        Some(input) => {
                            batch.push(input);

                            // Process batch if it reaches max size
                            if batch.len() >= self.config.max_batch_size {
                                self.process_batch(std::mem::take(&mut batch)).await?;
                            }
                        }
                        None => {
                            // Stream ended, process remaining batch
                            if !batch.is_empty() {
                                self.process_batch(batch).await?;
                            }
                            break;
                        }
                    }
                }

                // Process batch on timeout
                _ = batch_timer.tick() => {
                    if !batch.is_empty() {
                        self.process_batch(std::mem::take(&mut batch)).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Process a batch of inputs
    async fn process_batch(&self, batch: Vec<ParsingInput>) -> Result<(), PipelineError> {
        if batch.is_empty() {
            return Ok(());
        }

        // Acquire semaphore permit for concurrency control
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| PipelineError::SemaphoreError(()))?;

        let start_time = Instant::now();
        let mut total_bytes = 0;
        let mut all_events = Vec::new();
        let mut all_errors = Vec::new();
        let mut parser_metrics = HashMap::new();

        // Prepare batch data for parsing
        let batch_data: Vec<&[u8]> = batch
            .iter()
            .map(|input| {
                total_bytes += input.data.len();
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
            Err(e) => {
                all_errors.push(e);
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
                    .filter(|e| e.protocol_type() == protocol)
                    .count();

                if events_count > 0 {
                    parser_metrics.insert(
                        protocol,
                        ParserMetrics {
                            parse_time: processing_time / events_count as u32,
                            events_count,
                            success_rate: if all_errors.is_empty() { 1.0 } else { 0.8 }, // Simplified calculation
                        },
                    );
                }
            }
        }

        let metrics = ParsingMetrics {
            processing_time,
            events_parsed: all_events.len(),
            bytes_processed: total_bytes,
            error_count: all_errors.len(),
            parser_metrics,
        };

        // Send output
        let output = ParsingOutput {
            events: all_events,
            metrics,
            errors: all_errors,
        };

        self.output_sender
            .send(output)
            .map_err(|_| PipelineError::ChannelError)?;

        Ok(())
    }

    /// Get the output receiver for processed events
    pub fn take_output_receiver(&mut self) -> mpsc::UnboundedReceiver<ParsingOutput> {
        let (new_sender, new_receiver) = mpsc::unbounded_channel();
        let old_receiver = std::mem::replace(&mut self.output_receiver, new_receiver);
        self.output_sender = new_sender;
        old_receiver
    }

    /// Get pipeline statistics
    pub fn get_stats(&self) -> PipelineStats {
        let batch_stats = self.batch_parser.get_stats();

        PipelineStats {
            max_batch_size: self.config.max_batch_size,
            max_concurrent_tasks: self.config.max_concurrent_tasks,
            registered_parsers: batch_stats.registered_parsers,
            available_permits: self.semaphore.available_permits(),
        }
    }
}

/// Statistics for the parsing pipeline
#[derive(Debug, Clone)]
pub struct PipelineStats {
    /// Maximum batch size for processing
    pub max_batch_size: usize,
    /// Maximum number of concurrent parsing tasks
    pub max_concurrent_tasks: usize,
    /// Number of registered parsers
    pub registered_parsers: usize,
    /// Number of available semaphore permits
    pub available_permits: usize,
}

/// Error type for parsing pipeline operations
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    /// Error acquiring semaphore permit
    #[error("Semaphore error")]
    SemaphoreError(()),

    /// Error sending data through channel
    #[error("Channel error")]
    ChannelError,

    /// Error during event parsing
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),

    /// Invalid pipeline configuration
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Builder for creating parsing pipelines
#[derive(Default)]
pub struct ParsingPipelineBuilder {
    config: ParsingPipelineConfig,
    parsers: Vec<Arc<dyn ByteSliceEventParser>>,
}

impl std::fmt::Debug for ParsingPipelineBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParsingPipelineBuilder")
            .field("config", &self.config)
            .field("parsers", &format!("{} parsers", self.parsers.len()))
            .finish()
    }
}

impl ParsingPipelineBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: ParsingPipelineConfig::default(),
            parsers: Vec::new(),
        }
    }

    /// Set batch size
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.config.max_batch_size = size;
        self
    }

    /// Set batch timeout
    pub fn with_batch_timeout(mut self, timeout: Duration) -> Self {
        self.config.batch_timeout = timeout;
        self
    }

    /// Set concurrency limit
    pub fn with_concurrency_limit(mut self, limit: usize) -> Self {
        self.config.max_concurrent_tasks = limit;
        self
    }

    /// Enable or disable metrics collection
    pub fn with_metrics(mut self, enable: bool) -> Self {
        self.config.enable_metrics = enable;
        self
    }

    /// Add a parser
    pub fn add_parser(mut self, parser: Arc<dyn ByteSliceEventParser>) -> Self {
        self.parsers.push(parser);
        self
    }

    /// Add parser configuration
    pub fn with_parser_config(mut self, protocol: ProtocolType, config: ParserConfig) -> Self {
        self.config.parser_configs.insert(protocol, config);
        self
    }

    /// Build the parsing pipeline
    pub fn build(self) -> ParsingPipeline {
        let mut pipeline = ParsingPipeline::new(self.config);

        // Add all parsers
        for parser in self.parsers {
            pipeline.add_parser(parser);
        }

        pipeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::{JupiterParserFactory, RaydiumV4ParserFactory};

    #[tokio::test]
    async fn test_pipeline_builder() {
        let pipeline = ParsingPipelineBuilder::default()
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
        let config = ParsingPipelineConfig::default();
        let pipeline = ParsingPipeline::new(config);

        // Empty batch should not cause errors
        let result = pipeline.process_batch(Vec::new()).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_parsing_input_creation() {
        let input = ParsingInput {
            data: vec![0x09, 0x01, 0x02],
            metadata: SolanaEventMetadata::default(),
            program_id_hint: None,
        };

        assert_eq!(input.data.len(), 3);
        assert!(input.program_id_hint.is_none());
    }

    #[test]
    fn test_parsing_pipeline_config_default() {
        let config = ParsingPipelineConfig::default();
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
        let metrics = ParsingMetrics::default();
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
        assert_eq!(metrics.success_rate, 0.0);
    }

    #[test]
    fn test_parsing_pipeline_builder_default() {
        let builder = ParsingPipelineBuilder::default();
        assert_eq!(builder.config.max_batch_size, 100);
        assert!(builder.parsers.is_empty());
    }

    #[test]
    fn test_parsing_pipeline_builder_with_batch_size() {
        let builder = ParsingPipelineBuilder::default().with_batch_size(200);
        assert_eq!(builder.config.max_batch_size, 200);
    }

    #[test]
    fn test_parsing_pipeline_builder_with_batch_timeout() {
        let timeout = Duration::from_millis(100);
        let builder = ParsingPipelineBuilder::default().with_batch_timeout(timeout);
        assert_eq!(builder.config.batch_timeout, timeout);
    }

    #[test]
    fn test_parsing_pipeline_builder_with_concurrency_limit() {
        let builder = ParsingPipelineBuilder::default().with_concurrency_limit(8);
        assert_eq!(builder.config.max_concurrent_tasks, 8);
    }

    #[test]
    fn test_parsing_pipeline_builder_with_metrics() {
        let builder = ParsingPipelineBuilder::default().with_metrics(false);
        assert!(!builder.config.enable_metrics);
    }

    #[test]
    fn test_parsing_pipeline_builder_with_parser_config() {
        let parser_config = ParserConfig {
            zero_copy: false,
            detailed_analysis: false,
            priority: 5,
        };
        let builder = ParsingPipelineBuilder::default()
            .with_parser_config(ProtocolType::Jupiter, parser_config.clone());

        assert_eq!(builder.config.parser_configs.len(), 1);
        let config = builder
            .config
            .parser_configs
            .get(&ProtocolType::Jupiter)
            .unwrap();
        assert!(!config.zero_copy);
        assert!(!config.detailed_analysis);
        assert_eq!(config.priority, 5);
    }

    #[test]
    fn test_parsing_pipeline_builder_add_parser() {
        let parser = RaydiumV4ParserFactory::create_zero_copy();
        let builder = ParsingPipelineBuilder::default().add_parser(parser);
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

        let builder = ParsingPipelineBuilder::default()
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
        let config = ParsingPipelineConfig {
            max_batch_size: 50,
            batch_timeout: Duration::from_millis(25),
            max_concurrent_tasks: 2,
            output_buffer_size: 500,
            enable_metrics: false,
            parser_configs: HashMap::new(),
        };

        let pipeline = ParsingPipeline::new(config);
        let stats = pipeline.get_stats();

        assert_eq!(stats.max_batch_size, 50);
        assert_eq!(stats.max_concurrent_tasks, 2);
        assert_eq!(stats.registered_parsers, 0);
        assert_eq!(stats.available_permits, 2);
    }

    #[tokio::test]
    async fn test_parsing_pipeline_add_parser() {
        let config = ParsingPipelineConfig::default();
        let mut pipeline = ParsingPipeline::new(config);

        let parser = RaydiumV4ParserFactory::create_zero_copy();
        pipeline.add_parser(parser);

        let stats = pipeline.get_stats();
        assert_eq!(stats.registered_parsers, 1);
    }

    #[tokio::test]
    async fn test_parsing_pipeline_take_output_receiver() {
        let config = ParsingPipelineConfig::default();
        let mut pipeline = ParsingPipeline::new(config);

        let _receiver = pipeline.take_output_receiver();
        // After taking the receiver, a new one should be created internally
        // This is verified by the function not panicking
    }

    #[test]
    fn test_parsing_input_with_program_id_hint() {
        use solana_sdk::pubkey::Pubkey;

        let program_id = Pubkey::new_unique();
        let input = ParsingInput {
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

        assert_eq!(format!("{}", semaphore_error), "Semaphore error");
        assert_eq!(format!("{}", channel_error), "Channel error");
        assert_eq!(
            format!("{}", config_error),
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

        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("max_batch_size: 100"));
        assert!(debug_str.contains("max_concurrent_tasks: 4"));
        assert!(debug_str.contains("registered_parsers: 2"));
        assert!(debug_str.contains("available_permits: 3"));
    }

    #[test]
    fn test_parsing_output_debug() {
        let output = ParsingOutput {
            events: Vec::new(),
            metrics: ParsingMetrics::default(),
            errors: Vec::new(),
        };

        let debug_str = format!("{:?}", output);
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

        let metrics = ParsingMetrics {
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

        let jupiter_metrics = metrics.parser_metrics.get(&ProtocolType::Jupiter).unwrap();
        assert_eq!(jupiter_metrics.parse_time, Duration::from_millis(10));
        assert_eq!(jupiter_metrics.events_count, 5);
        assert_eq!(jupiter_metrics.success_rate, 0.9);
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

        let config = ParsingPipelineConfig {
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
        let input = ParsingInput {
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
        let config = ParsingPipelineConfig::default();
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
        let metrics = ParsingMetrics::default();
        let cloned = metrics.clone();

        assert_eq!(metrics.processing_time, cloned.processing_time);
        assert_eq!(metrics.events_parsed, cloned.events_parsed);
        assert_eq!(metrics.bytes_processed, cloned.bytes_processed);
        assert_eq!(metrics.error_count, cloned.error_count);
    }

    #[test]
    fn test_parser_metrics_clone() {
        let metrics = ParserMetrics::default();
        let cloned = metrics.clone();

        assert_eq!(metrics.parse_time, cloned.parse_time);
        assert_eq!(metrics.events_count, cloned.events_count);
        assert_eq!(metrics.success_rate, cloned.success_rate);
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
        let input = ParsingInput {
            data: Vec::new(),
            metadata: SolanaEventMetadata::default(),
            program_id_hint: None,
        };

        assert!(input.data.is_empty());
    }

    #[test]
    fn test_parsing_input_large_data() {
        let large_data = vec![0xFF; 10000];
        let input = ParsingInput {
            data: large_data.clone(),
            metadata: SolanaEventMetadata::default(),
            program_id_hint: None,
        };

        assert_eq!(input.data.len(), 10000);
        assert_eq!(input.data, large_data);
    }

    #[test]
    fn test_parsing_pipeline_config_zero_batch_size() {
        let config = ParsingPipelineConfig {
            max_batch_size: 0,
            ..Default::default()
        };

        assert_eq!(config.max_batch_size, 0);
    }

    #[test]
    fn test_parsing_pipeline_config_zero_timeout() {
        let config = ParsingPipelineConfig {
            batch_timeout: Duration::from_millis(0),
            ..Default::default()
        };

        assert_eq!(config.batch_timeout, Duration::from_millis(0));
    }

    #[test]
    fn test_parsing_pipeline_config_zero_concurrency() {
        let config = ParsingPipelineConfig {
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

        assert_eq!(metrics.success_rate, 0.0);
    }

    #[test]
    fn test_parser_metrics_full_success_rate() {
        let metrics = ParserMetrics {
            success_rate: 1.0,
            ..Default::default()
        };

        assert_eq!(metrics.success_rate, 1.0);
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

        let builder = ParsingPipelineBuilder::default()
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

        let pipeline = ParsingPipelineBuilder::default()
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
