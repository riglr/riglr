//! Feature flags configuration

use serde::{Deserialize, Serialize};
use crate::ConfigResult;

/// Feature flags configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeaturesConfig {
    /// Enable trading functionality
    #[serde(default = "default_true")]
    pub enable_trading: bool,
    
    /// Enable cross-chain bridging
    #[serde(default = "default_true")]
    pub enable_bridging: bool,
    
    /// Enable social media monitoring
    #[serde(default)]
    pub enable_social_monitoring: bool,
    
    /// Enable graph-based memory
    #[serde(default)]
    pub enable_graph_memory: bool,
    
    /// Enable real-time streaming
    #[serde(default = "default_true")]
    pub enable_streaming: bool,
    
    /// Enable webhook notifications
    #[serde(default)]
    pub enable_webhooks: bool,
    
    /// Enable analytics collection
    #[serde(default)]
    pub enable_analytics: bool,
    
    /// Enable debug mode
    #[serde(default)]
    pub debug_mode: bool,
    
    /// Enable experimental features
    #[serde(default)]
    pub experimental: bool,
    
    /// Custom feature flags
    #[serde(default)]
    pub custom: std::collections::HashMap<String, bool>,
}

impl FeaturesConfig {
    /// Check if a feature is enabled
    pub fn is_enabled(&self, feature: Feature) -> bool {
        match feature {
            Feature::Trading => self.enable_trading,
            Feature::Bridging => self.enable_bridging,
            Feature::SocialMonitoring => self.enable_social_monitoring,
            Feature::GraphMemory => self.enable_graph_memory,
            Feature::Streaming => self.enable_streaming,
            Feature::Webhooks => self.enable_webhooks,
            Feature::Analytics => self.enable_analytics,
            Feature::Debug => self.debug_mode,
            Feature::Experimental => self.experimental,
        }
    }
    
    /// Check if a custom feature is enabled
    pub fn is_custom_enabled(&self, name: &str) -> bool {
        self.custom.get(name).copied().unwrap_or(false)
    }
    
    /// Enable a feature
    pub fn enable(&mut self, feature: Feature) {
        match feature {
            Feature::Trading => self.enable_trading = true,
            Feature::Bridging => self.enable_bridging = true,
            Feature::SocialMonitoring => self.enable_social_monitoring = true,
            Feature::GraphMemory => self.enable_graph_memory = true,
            Feature::Streaming => self.enable_streaming = true,
            Feature::Webhooks => self.enable_webhooks = true,
            Feature::Analytics => self.enable_analytics = true,
            Feature::Debug => self.debug_mode = true,
            Feature::Experimental => self.experimental = true,
        }
    }
    
    /// Disable a feature
    pub fn disable(&mut self, feature: Feature) {
        match feature {
            Feature::Trading => self.enable_trading = false,
            Feature::Bridging => self.enable_bridging = false,
            Feature::SocialMonitoring => self.enable_social_monitoring = false,
            Feature::GraphMemory => self.enable_graph_memory = false,
            Feature::Streaming => self.enable_streaming = false,
            Feature::Webhooks => self.enable_webhooks = false,
            Feature::Analytics => self.enable_analytics = false,
            Feature::Debug => self.debug_mode = false,
            Feature::Experimental => self.experimental = false,
        }
    }
    
    pub fn validate(&self) -> ConfigResult<()> {
        // Add any feature-specific validation here
        if self.experimental && !self.debug_mode {
            tracing::warn!("Experimental features enabled without debug mode");
        }
        
        Ok(())
    }
}

/// Feature enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    Trading,
    Bridging,
    SocialMonitoring,
    GraphMemory,
    Streaming,
    Webhooks,
    Analytics,
    Debug,
    Experimental,
}

impl std::fmt::Display for Feature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Feature::Trading => write!(f, "trading"),
            Feature::Bridging => write!(f, "bridging"),
            Feature::SocialMonitoring => write!(f, "social_monitoring"),
            Feature::GraphMemory => write!(f, "graph_memory"),
            Feature::Streaming => write!(f, "streaming"),
            Feature::Webhooks => write!(f, "webhooks"),
            Feature::Analytics => write!(f, "analytics"),
            Feature::Debug => write!(f, "debug"),
            Feature::Experimental => write!(f, "experimental"),
        }
    }
}

fn default_true() -> bool {
    true
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            enable_trading: true,
            enable_bridging: true,
            enable_social_monitoring: false,
            enable_graph_memory: false,
            enable_streaming: true,
            enable_webhooks: false,
            enable_analytics: false,
            debug_mode: false,
            experimental: false,
            custom: std::collections::HashMap::new(),
        }
    }
}