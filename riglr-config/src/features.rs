//! Feature flags configuration

use crate::ConfigResult;
use serde::{Deserialize, Serialize};

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

    /// Validate the features configuration for consistency and warnings
    pub fn validate_config(&self) -> ConfigResult<()> {
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
    /// Enable trading functionality
    Trading,
    /// Enable cross-chain bridging
    Bridging,
    /// Enable social media monitoring
    SocialMonitoring,
    /// Enable graph-based memory
    GraphMemory,
    /// Enable real-time streaming
    Streaming,
    /// Enable webhook notifications
    Webhooks,
    /// Enable analytics collection
    Analytics,
    /// Enable debug mode
    Debug,
    /// Enable experimental features
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_features_config_default() {
        let config = FeaturesConfig::default();

        // Test default values
        assert!(config.enable_trading);
        assert!(config.enable_bridging);
        assert!(!config.enable_social_monitoring);
        assert!(!config.enable_graph_memory);
        assert!(config.enable_streaming);
        assert!(!config.enable_webhooks);
        assert!(!config.enable_analytics);
        assert!(!config.debug_mode);
        assert!(!config.experimental);
        assert!(config.custom.is_empty());
    }

    #[test]
    fn test_is_enabled_when_trading_should_return_correct_value() {
        let mut config = FeaturesConfig::default();
        assert!(config.is_enabled(Feature::Trading));

        config.enable_trading = false;
        assert!(!config.is_enabled(Feature::Trading));
    }

    #[test]
    fn test_is_enabled_when_bridging_should_return_correct_value() {
        let mut config = FeaturesConfig::default();
        assert!(config.is_enabled(Feature::Bridging));

        config.enable_bridging = false;
        assert!(!config.is_enabled(Feature::Bridging));
    }

    #[test]
    fn test_is_enabled_when_social_monitoring_should_return_correct_value() {
        let mut config = FeaturesConfig::default();
        assert!(!config.is_enabled(Feature::SocialMonitoring));

        config.enable_social_monitoring = true;
        assert!(config.is_enabled(Feature::SocialMonitoring));
    }

    #[test]
    fn test_is_enabled_when_graph_memory_should_return_correct_value() {
        let mut config = FeaturesConfig::default();
        assert!(!config.is_enabled(Feature::GraphMemory));

        config.enable_graph_memory = true;
        assert!(config.is_enabled(Feature::GraphMemory));
    }

    #[test]
    fn test_is_enabled_when_streaming_should_return_correct_value() {
        let mut config = FeaturesConfig::default();
        assert!(config.is_enabled(Feature::Streaming));

        config.enable_streaming = false;
        assert!(!config.is_enabled(Feature::Streaming));
    }

    #[test]
    fn test_is_enabled_when_webhooks_should_return_correct_value() {
        let mut config = FeaturesConfig::default();
        assert!(!config.is_enabled(Feature::Webhooks));

        config.enable_webhooks = true;
        assert!(config.is_enabled(Feature::Webhooks));
    }

    #[test]
    fn test_is_enabled_when_analytics_should_return_correct_value() {
        let mut config = FeaturesConfig::default();
        assert!(!config.is_enabled(Feature::Analytics));

        config.enable_analytics = true;
        assert!(config.is_enabled(Feature::Analytics));
    }

    #[test]
    fn test_is_enabled_when_debug_should_return_correct_value() {
        let mut config = FeaturesConfig::default();
        assert!(!config.is_enabled(Feature::Debug));

        config.debug_mode = true;
        assert!(config.is_enabled(Feature::Debug));
    }

    #[test]
    fn test_is_enabled_when_experimental_should_return_correct_value() {
        let mut config = FeaturesConfig::default();
        assert!(!config.is_enabled(Feature::Experimental));

        config.experimental = true;
        assert!(config.is_enabled(Feature::Experimental));
    }

    #[test]
    fn test_is_custom_enabled_when_feature_exists_should_return_true() {
        let mut config = FeaturesConfig::default();
        config.custom.insert("custom_feature".to_string(), true);

        assert!(config.is_custom_enabled("custom_feature"));
    }

    #[test]
    fn test_is_custom_enabled_when_feature_exists_false_should_return_false() {
        let mut config = FeaturesConfig::default();
        config.custom.insert("custom_feature".to_string(), false);

        assert!(!config.is_custom_enabled("custom_feature"));
    }

    #[test]
    fn test_is_custom_enabled_when_feature_not_exists_should_return_false() {
        let config = FeaturesConfig::default();

        assert!(!config.is_custom_enabled("nonexistent_feature"));
    }

    #[test]
    fn test_enable_trading_should_set_true() {
        let mut config = FeaturesConfig::default();
        config.enable_trading = false;

        config.enable(Feature::Trading);
        assert!(config.enable_trading);
    }

    #[test]
    fn test_enable_bridging_should_set_true() {
        let mut config = FeaturesConfig::default();
        config.enable_bridging = false;

        config.enable(Feature::Bridging);
        assert!(config.enable_bridging);
    }

    #[test]
    fn test_enable_social_monitoring_should_set_true() {
        let mut config = FeaturesConfig::default();

        config.enable(Feature::SocialMonitoring);
        assert!(config.enable_social_monitoring);
    }

    #[test]
    fn test_enable_graph_memory_should_set_true() {
        let mut config = FeaturesConfig::default();

        config.enable(Feature::GraphMemory);
        assert!(config.enable_graph_memory);
    }

    #[test]
    fn test_enable_streaming_should_set_true() {
        let mut config = FeaturesConfig::default();
        config.enable_streaming = false;

        config.enable(Feature::Streaming);
        assert!(config.enable_streaming);
    }

    #[test]
    fn test_enable_webhooks_should_set_true() {
        let mut config = FeaturesConfig::default();

        config.enable(Feature::Webhooks);
        assert!(config.enable_webhooks);
    }

    #[test]
    fn test_enable_analytics_should_set_true() {
        let mut config = FeaturesConfig::default();

        config.enable(Feature::Analytics);
        assert!(config.enable_analytics);
    }

    #[test]
    fn test_enable_debug_should_set_true() {
        let mut config = FeaturesConfig::default();

        config.enable(Feature::Debug);
        assert!(config.debug_mode);
    }

    #[test]
    fn test_enable_experimental_should_set_true() {
        let mut config = FeaturesConfig::default();

        config.enable(Feature::Experimental);
        assert!(config.experimental);
    }

    #[test]
    fn test_disable_trading_should_set_false() {
        let mut config = FeaturesConfig::default();

        config.disable(Feature::Trading);
        assert!(!config.enable_trading);
    }

    #[test]
    fn test_disable_bridging_should_set_false() {
        let mut config = FeaturesConfig::default();

        config.disable(Feature::Bridging);
        assert!(!config.enable_bridging);
    }

    #[test]
    fn test_disable_social_monitoring_should_set_false() {
        let mut config = FeaturesConfig::default();
        config.enable_social_monitoring = true;

        config.disable(Feature::SocialMonitoring);
        assert!(!config.enable_social_monitoring);
    }

    #[test]
    fn test_disable_graph_memory_should_set_false() {
        let mut config = FeaturesConfig::default();
        config.enable_graph_memory = true;

        config.disable(Feature::GraphMemory);
        assert!(!config.enable_graph_memory);
    }

    #[test]
    fn test_disable_streaming_should_set_false() {
        let mut config = FeaturesConfig::default();

        config.disable(Feature::Streaming);
        assert!(!config.enable_streaming);
    }

    #[test]
    fn test_disable_webhooks_should_set_false() {
        let mut config = FeaturesConfig::default();
        config.enable_webhooks = true;

        config.disable(Feature::Webhooks);
        assert!(!config.enable_webhooks);
    }

    #[test]
    fn test_disable_analytics_should_set_false() {
        let mut config = FeaturesConfig::default();
        config.enable_analytics = true;

        config.disable(Feature::Analytics);
        assert!(!config.enable_analytics);
    }

    #[test]
    fn test_disable_debug_should_set_false() {
        let mut config = FeaturesConfig::default();
        config.debug_mode = true;

        config.disable(Feature::Debug);
        assert!(!config.debug_mode);
    }

    #[test]
    fn test_disable_experimental_should_set_false() {
        let mut config = FeaturesConfig::default();
        config.experimental = true;

        config.disable(Feature::Experimental);
        assert!(!config.experimental);
    }

    #[test]
    fn test_validate_when_experimental_without_debug_should_warn_and_return_ok() {
        let mut config = FeaturesConfig::default();
        config.experimental = true;
        config.debug_mode = false;

        let result = config.validate_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_when_experimental_with_debug_should_return_ok() {
        let mut config = FeaturesConfig::default();
        config.experimental = true;
        config.debug_mode = true;

        let result = config.validate_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_when_no_experimental_should_return_ok() {
        let config = FeaturesConfig::default();

        let result = config.validate_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_feature_display_trading() {
        assert_eq!(Feature::Trading.to_string(), "trading");
    }

    #[test]
    fn test_feature_display_bridging() {
        assert_eq!(Feature::Bridging.to_string(), "bridging");
    }

    #[test]
    fn test_feature_display_social_monitoring() {
        assert_eq!(Feature::SocialMonitoring.to_string(), "social_monitoring");
    }

    #[test]
    fn test_feature_display_graph_memory() {
        assert_eq!(Feature::GraphMemory.to_string(), "graph_memory");
    }

    #[test]
    fn test_feature_display_streaming() {
        assert_eq!(Feature::Streaming.to_string(), "streaming");
    }

    #[test]
    fn test_feature_display_webhooks() {
        assert_eq!(Feature::Webhooks.to_string(), "webhooks");
    }

    #[test]
    fn test_feature_display_analytics() {
        assert_eq!(Feature::Analytics.to_string(), "analytics");
    }

    #[test]
    fn test_feature_display_debug() {
        assert_eq!(Feature::Debug.to_string(), "debug");
    }

    #[test]
    fn test_feature_display_experimental() {
        assert_eq!(Feature::Experimental.to_string(), "experimental");
    }

    #[test]
    fn test_default_true_function() {
        assert!(default_true());
    }

    #[test]
    fn test_feature_debug_format() {
        let feature = Feature::Trading;
        assert_eq!(format!("{:?}", feature), "Trading");
    }

    #[test]
    fn test_feature_clone() {
        let feature = Feature::Trading;
        let cloned = feature.clone();
        assert_eq!(feature, cloned);
    }

    #[test]
    fn test_feature_copy() {
        let feature = Feature::Trading;
        let copied = feature;
        assert_eq!(feature, copied);
    }

    #[test]
    fn test_feature_partial_eq() {
        assert_eq!(Feature::Trading, Feature::Trading);
        assert_ne!(Feature::Trading, Feature::Bridging);
    }

    #[test]
    fn test_features_config_debug_format() {
        let config = FeaturesConfig::default();
        let debug_string = format!("{:?}", config);
        assert!(debug_string.contains("FeaturesConfig"));
    }

    #[test]
    fn test_features_config_clone() {
        let config = FeaturesConfig::default();
        let cloned = config.clone();
        assert_eq!(config.enable_trading, cloned.enable_trading);
        assert_eq!(config.enable_bridging, cloned.enable_bridging);
    }

    #[test]
    fn test_serde_serialization() {
        let config = FeaturesConfig::default();
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_serde_deserialization() {
        let json = r#"{"enable_trading":true,"enable_bridging":false,"enable_social_monitoring":true,"enable_graph_memory":false,"enable_streaming":true,"enable_webhooks":false,"enable_analytics":false,"debug_mode":false,"experimental":false,"custom":{}}"#;
        let config: Result<FeaturesConfig, _> = serde_json::from_str(json);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert!(config.enable_trading);
        assert!(!config.enable_bridging);
        assert!(config.enable_social_monitoring);
    }

    #[test]
    fn test_serde_deserialization_with_defaults() {
        let json = r#"{}"#;
        let config: Result<FeaturesConfig, _> = serde_json::from_str(json);
        assert!(config.is_ok());
        let config = config.unwrap();
        // Should use default values
        assert!(config.enable_trading);
        assert!(config.enable_bridging);
        assert!(!config.enable_social_monitoring);
        assert!(config.enable_streaming);
    }
}
