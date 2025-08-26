//! Standard capability constants for the agent system.
//!
//! This module defines constants for all standard agent capabilities,
//! preventing typos and improving maintainability by centralizing
//! capability definitions.

/// Trading capability - for agents that can execute trading operations
pub const TRADING: &str = "trading";

/// Research capability - for agents that can perform research and analysis
pub const RESEARCH: &str = "research";

/// Risk analysis capability - for agents that can assess and analyze risk
pub const RISK_ANALYSIS: &str = "risk_analysis";

/// Portfolio management capability - for agents that can manage portfolios
pub const PORTFOLIO: &str = "portfolio";

/// Monitoring capability - for agents that can monitor systems and markets
pub const MONITORING: &str = "monitoring";

/// Tool calling capability - for agents that can invoke tools and functions
pub const TOOL_CALLING: &str = "tool_calling";
