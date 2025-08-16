//! Multi-Agent Systems Examples
//!
//! This module demonstrates practical multi-agent coordination patterns
//! using the riglr-agents framework. These examples show how to build
//! sophisticated systems where multiple specialized agents work together
//! to achieve complex blockchain automation tasks.
//!
//! ## Examples
//!
//! - [`trading_coordination`]: Real-world multi-agent trading workflow
//! - Integration with existing riglr tools and patterns
//! - SignerContext usage in multi-agent scenarios
//! - Error handling and recovery strategies

pub mod trading_coordination;

pub use trading_coordination::*;
