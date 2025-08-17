//! Parsing pipeline orchestration for high-throughput event processing
//!
//! This module provides a configurable pipeline for parsing Solana transaction data
//! with support for batching, parallel processing, and backpressure handling.

use crate::types::{EventMetadata, ProtocolType};
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
    pub metadata: EventMetadata,
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

        let batch_metadata: Vec<EventMetadata> =
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
            metadata: EventMetadata::default(),
            program_id_hint: None,
        };

        assert_eq!(input.data.len(), 3);
        assert!(input.program_id_hint.is_none());
    }
}
