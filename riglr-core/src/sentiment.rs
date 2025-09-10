//! Generic sentiment analysis abstraction
//!
//! This module provides a marker type for sentiment analyzers that can be
//! stored in the ApplicationContext's extension system. The actual trait
//! definition remains in the implementing crates to avoid circular dependencies.

/// Marker trait for sentiment analyzers
///
/// This trait serves as a type marker for sentiment analysis implementations
/// that can be stored in ApplicationContext's extension system.
///
/// The actual sentiment analysis logic is defined in implementing crates
/// (e.g., riglr-web-tools) to avoid circular dependencies.
///
/// # Example
///
/// ```rust,no_run
/// use riglr_core::provider::ApplicationContext;
/// use std::sync::Arc;
///
/// // In application setup:
/// let context = ApplicationContext::default();
///
/// // Store a sentiment analyzer (from riglr-web-tools or custom implementation)
/// // let analyzer = Arc::new(MyConcreteAnalyzer::new());
/// // context.set_extension(analyzer);
///
/// // Later, retrieve it:
/// // let analyzer = context.get_extension::<Arc<dyn SentimentAnalyzerMarker>>()
/// //     .expect("Sentiment analyzer not configured");
/// ```
pub trait SentimentAnalyzerMarker: Send + Sync + 'static {}
