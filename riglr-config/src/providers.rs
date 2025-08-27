//! External API provider configuration

use crate::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};

/// External API providers configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProvidersConfig {
    // AI Providers
    /// API key for Anthropic Claude
    #[serde(default)]
    pub anthropic_api_key: Option<String>,

    /// API key for OpenAI
    #[serde(default)]
    pub openai_api_key: Option<String>,

    /// API key for Groq
    #[serde(default)]
    pub groq_api_key: Option<String>,

    /// API key for Perplexity AI
    #[serde(default)]
    pub perplexity_api_key: Option<String>,

    // Blockchain Data Providers
    /// API key for Alchemy
    #[serde(default)]
    pub alchemy_api_key: Option<String>,

    /// API key for Infura
    #[serde(default)]
    pub infura_api_key: Option<String>,

    /// API key for QuickNode
    #[serde(default)]
    pub quicknode_api_key: Option<String>,

    /// API key for Moralis
    #[serde(default)]
    pub moralis_api_key: Option<String>,

    // Cross-chain and DeFi
    /// API key for LI.FI
    #[serde(default)]
    pub lifi_api_key: Option<String>,

    /// API key for 1inch
    #[serde(default)]
    pub one_inch_api_key: Option<String>,

    /// API key for 0x Protocol
    #[serde(default)]
    pub zerox_api_key: Option<String>,

    // Market Data
    /// API key for DexScreener
    #[serde(default)]
    pub dexscreener_api_key: Option<String>,

    /// API key for CoinGecko
    #[serde(default)]
    pub coingecko_api_key: Option<String>,

    /// API key for CoinMarketCap
    #[serde(default)]
    pub coinmarketcap_api_key: Option<String>,

    /// API key for Pump.fun
    #[serde(default)]
    pub pump_api_key: Option<String>,

    /// API URL for Pump.fun
    #[serde(default)]
    pub pump_api_url: Option<String>,

    /// API URL for Jupiter aggregator
    #[serde(default)]
    pub jupiter_api_url: Option<String>,

    // Social and Web Data
    /// Bearer token for Twitter API
    #[serde(default)]
    pub twitter_bearer_token: Option<String>,

    /// API key for Exa
    #[serde(default)]
    pub exa_api_key: Option<String>,

    /// API key for Serper
    #[serde(default)]
    pub serper_api_key: Option<String>,

    // News and Analytics
    /// API key for LunarCrush
    #[serde(default)]
    pub lunarcrush_api_key: Option<String>,

    /// API key for NewsAPI
    #[serde(default)]
    pub newsapi_key: Option<String>,
}

impl ProvidersConfig {
    /// Check if a specific AI provider is configured
    pub fn has_ai_provider(&self, provider: AiProvider) -> bool {
        match provider {
            AiProvider::Anthropic => self.anthropic_api_key.is_some(),
            AiProvider::OpenAI => self.openai_api_key.is_some(),
            AiProvider::Groq => self.groq_api_key.is_some(),
            AiProvider::Perplexity => self.perplexity_api_key.is_some(),
        }
    }

    /// Get the API key for an AI provider
    pub fn get_ai_key(&self, provider: AiProvider) -> Option<&str> {
        match provider {
            AiProvider::Anthropic => self.anthropic_api_key.as_deref(),
            AiProvider::OpenAI => self.openai_api_key.as_deref(),
            AiProvider::Groq => self.groq_api_key.as_deref(),
            AiProvider::Perplexity => self.perplexity_api_key.as_deref(),
        }
    }

    /// Check if a blockchain provider is configured
    pub fn has_blockchain_provider(&self, provider: BlockchainProvider) -> bool {
        match provider {
            BlockchainProvider::Alchemy => self.alchemy_api_key.is_some(),
            BlockchainProvider::Infura => self.infura_api_key.is_some(),
            BlockchainProvider::QuickNode => self.quicknode_api_key.is_some(),
            BlockchainProvider::Moralis => self.moralis_api_key.is_some(),
        }
    }

    /// Get the API key for a blockchain provider
    pub fn get_blockchain_key(&self, provider: BlockchainProvider) -> Option<&str> {
        match provider {
            BlockchainProvider::Alchemy => self.alchemy_api_key.as_deref(),
            BlockchainProvider::Infura => self.infura_api_key.as_deref(),
            BlockchainProvider::QuickNode => self.quicknode_api_key.as_deref(),
            BlockchainProvider::Moralis => self.moralis_api_key.as_deref(),
        }
    }

    /// Check if a data provider is configured
    pub fn has_data_provider(&self, provider: DataProvider) -> bool {
        match provider {
            DataProvider::DexScreener => self.dexscreener_api_key.is_some(),
            DataProvider::CoinGecko => self.coingecko_api_key.is_some(),
            DataProvider::CoinMarketCap => self.coinmarketcap_api_key.is_some(),
            DataProvider::Twitter => self.twitter_bearer_token.is_some(),
            DataProvider::LunarCrush => self.lunarcrush_api_key.is_some(),
        }
    }

    /// Validate API key formats and configurations
    pub fn validate_config(&self) -> ConfigResult<()> {
        // Validate API key formats
        if let Some(ref key) = self.anthropic_api_key {
            if key.is_empty() {
                return Err(ConfigError::validation("ANTHROPIC_API_KEY cannot be empty"));
            }
        }

        if let Some(ref token) = self.twitter_bearer_token {
            if !token.is_empty() && !token.starts_with("Bearer ") {
                return Err(ConfigError::validation(
                    "TWITTER_BEARER_TOKEN must start with 'Bearer ' if it is set",
                ));
            }
        }

        // More validations can be added as needed
        Ok(())
    }
}

/// AI provider enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiProvider {
    /// Anthropic Claude AI provider
    Anthropic,
    /// OpenAI provider
    OpenAI,
    /// Groq provider
    Groq,
    /// Perplexity AI provider
    Perplexity,
}

/// Blockchain data provider enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockchainProvider {
    /// Alchemy blockchain data provider
    Alchemy,
    /// Infura blockchain infrastructure provider
    Infura,
    /// QuickNode blockchain infrastructure provider
    QuickNode,
    /// Moralis Web3 development platform
    Moralis,
}

/// Data provider enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataProvider {
    /// DexScreener DEX analytics provider
    DexScreener,
    /// CoinGecko cryptocurrency data provider
    CoinGecko,
    /// CoinMarketCap cryptocurrency market data provider
    CoinMarketCap,
    /// Twitter social media data provider
    Twitter,
    /// LunarCrush social analytics provider
    LunarCrush,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_providers_config_default() {
        let config = ProvidersConfig::default();

        // Test all fields are None by default
        assert!(config.anthropic_api_key.is_none());
        assert!(config.openai_api_key.is_none());
        assert!(config.groq_api_key.is_none());
        assert!(config.perplexity_api_key.is_none());
        assert!(config.alchemy_api_key.is_none());
        assert!(config.infura_api_key.is_none());
        assert!(config.quicknode_api_key.is_none());
        assert!(config.moralis_api_key.is_none());
        assert!(config.lifi_api_key.is_none());
        assert!(config.one_inch_api_key.is_none());
        assert!(config.zerox_api_key.is_none());
        assert!(config.dexscreener_api_key.is_none());
        assert!(config.coingecko_api_key.is_none());
        assert!(config.coinmarketcap_api_key.is_none());
        assert!(config.pump_api_key.is_none());
        assert!(config.twitter_bearer_token.is_none());
        assert!(config.exa_api_key.is_none());
        assert!(config.serper_api_key.is_none());
        assert!(config.lunarcrush_api_key.is_none());
        assert!(config.newsapi_key.is_none());
    }

    #[test]
    fn test_has_ai_provider_when_keys_are_none_should_return_false() {
        let config = ProvidersConfig::default();

        assert!(!config.has_ai_provider(AiProvider::Anthropic));
        assert!(!config.has_ai_provider(AiProvider::OpenAI));
        assert!(!config.has_ai_provider(AiProvider::Groq));
        assert!(!config.has_ai_provider(AiProvider::Perplexity));
    }

    #[test]
    fn test_has_ai_provider_when_keys_are_some_should_return_true() {
        let config = ProvidersConfig {
            anthropic_api_key: Some("test_anthropic_key".to_string()),
            openai_api_key: Some("test_openai_key".to_string()),
            groq_api_key: Some("test_groq_key".to_string()),
            perplexity_api_key: Some("test_perplexity_key".to_string()),
            ..Default::default()
        };

        assert!(config.has_ai_provider(AiProvider::Anthropic));
        assert!(config.has_ai_provider(AiProvider::OpenAI));
        assert!(config.has_ai_provider(AiProvider::Groq));
        assert!(config.has_ai_provider(AiProvider::Perplexity));
    }

    #[test]
    fn test_get_ai_key_when_keys_are_none_should_return_none() {
        let config = ProvidersConfig::default();

        assert!(config.get_ai_key(AiProvider::Anthropic).is_none());
        assert!(config.get_ai_key(AiProvider::OpenAI).is_none());
        assert!(config.get_ai_key(AiProvider::Groq).is_none());
        assert!(config.get_ai_key(AiProvider::Perplexity).is_none());
    }

    #[test]
    fn test_get_ai_key_when_keys_are_some_should_return_key() {
        let config = ProvidersConfig {
            anthropic_api_key: Some("test_anthropic_key".to_string()),
            openai_api_key: Some("test_openai_key".to_string()),
            groq_api_key: Some("test_groq_key".to_string()),
            perplexity_api_key: Some("test_perplexity_key".to_string()),
            ..Default::default()
        };

        assert_eq!(
            config.get_ai_key(AiProvider::Anthropic),
            Some("test_anthropic_key")
        );
        assert_eq!(
            config.get_ai_key(AiProvider::OpenAI),
            Some("test_openai_key")
        );
        assert_eq!(config.get_ai_key(AiProvider::Groq), Some("test_groq_key"));
        assert_eq!(
            config.get_ai_key(AiProvider::Perplexity),
            Some("test_perplexity_key")
        );
    }

    #[test]
    fn test_has_blockchain_provider_when_keys_are_none_should_return_false() {
        let config = ProvidersConfig::default();

        assert!(!config.has_blockchain_provider(BlockchainProvider::Alchemy));
        assert!(!config.has_blockchain_provider(BlockchainProvider::Infura));
        assert!(!config.has_blockchain_provider(BlockchainProvider::QuickNode));
        assert!(!config.has_blockchain_provider(BlockchainProvider::Moralis));
    }

    #[test]
    fn test_has_blockchain_provider_when_keys_are_some_should_return_true() {
        let config = ProvidersConfig {
            alchemy_api_key: Some("test_alchemy_key".to_string()),
            infura_api_key: Some("test_infura_key".to_string()),
            quicknode_api_key: Some("test_quicknode_key".to_string()),
            moralis_api_key: Some("test_moralis_key".to_string()),
            ..Default::default()
        };

        assert!(config.has_blockchain_provider(BlockchainProvider::Alchemy));
        assert!(config.has_blockchain_provider(BlockchainProvider::Infura));
        assert!(config.has_blockchain_provider(BlockchainProvider::QuickNode));
        assert!(config.has_blockchain_provider(BlockchainProvider::Moralis));
    }

    #[test]
    fn test_get_blockchain_key_when_keys_are_none_should_return_none() {
        let config = ProvidersConfig::default();

        assert!(config
            .get_blockchain_key(BlockchainProvider::Alchemy)
            .is_none());
        assert!(config
            .get_blockchain_key(BlockchainProvider::Infura)
            .is_none());
        assert!(config
            .get_blockchain_key(BlockchainProvider::QuickNode)
            .is_none());
        assert!(config
            .get_blockchain_key(BlockchainProvider::Moralis)
            .is_none());
    }

    #[test]
    fn test_get_blockchain_key_when_keys_are_some_should_return_key() {
        let config = ProvidersConfig {
            alchemy_api_key: Some("test_alchemy_key".to_string()),
            infura_api_key: Some("test_infura_key".to_string()),
            quicknode_api_key: Some("test_quicknode_key".to_string()),
            moralis_api_key: Some("test_moralis_key".to_string()),
            ..Default::default()
        };

        assert_eq!(
            config.get_blockchain_key(BlockchainProvider::Alchemy),
            Some("test_alchemy_key")
        );
        assert_eq!(
            config.get_blockchain_key(BlockchainProvider::Infura),
            Some("test_infura_key")
        );
        assert_eq!(
            config.get_blockchain_key(BlockchainProvider::QuickNode),
            Some("test_quicknode_key")
        );
        assert_eq!(
            config.get_blockchain_key(BlockchainProvider::Moralis),
            Some("test_moralis_key")
        );
    }

    #[test]
    fn test_has_data_provider_when_keys_are_none_should_return_false() {
        let config = ProvidersConfig::default();

        assert!(!config.has_data_provider(DataProvider::DexScreener));
        assert!(!config.has_data_provider(DataProvider::CoinGecko));
        assert!(!config.has_data_provider(DataProvider::CoinMarketCap));
        assert!(!config.has_data_provider(DataProvider::Twitter));
        assert!(!config.has_data_provider(DataProvider::LunarCrush));
    }

    #[test]
    fn test_has_data_provider_when_keys_are_some_should_return_true() {
        let config = ProvidersConfig {
            dexscreener_api_key: Some("test_dexscreener_key".to_string()),
            coingecko_api_key: Some("test_coingecko_key".to_string()),
            coinmarketcap_api_key: Some("test_coinmarketcap_key".to_string()),
            twitter_bearer_token: Some("test_twitter_token".to_string()),
            lunarcrush_api_key: Some("test_lunarcrush_key".to_string()),
            ..Default::default()
        };

        assert!(config.has_data_provider(DataProvider::DexScreener));
        assert!(config.has_data_provider(DataProvider::CoinGecko));
        assert!(config.has_data_provider(DataProvider::CoinMarketCap));
        assert!(config.has_data_provider(DataProvider::Twitter));
        assert!(config.has_data_provider(DataProvider::LunarCrush));
    }

    #[test]
    fn test_validate_when_all_keys_are_none_should_return_ok() {
        let config = ProvidersConfig::default();
        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_validate_when_anthropic_key_is_empty_should_return_err() {
        let config = ProvidersConfig {
            anthropic_api_key: Some("".to_string()),
            ..Default::default()
        };

        let result = config.validate_config();
        assert!(result.is_err());
        if let Err(error) = result {
            let error_message = format!("{}", error);
            assert!(error_message.contains("ANTHROPIC_API_KEY cannot be empty"));
        }
    }

    #[test]
    fn test_validate_when_anthropic_key_is_valid_should_return_ok() {
        let config = ProvidersConfig {
            anthropic_api_key: Some("valid_key".to_string()),
            ..Default::default()
        };
        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_validate_when_twitter_token_has_bearer_prefix_should_return_ok() {
        let config = ProvidersConfig {
            twitter_bearer_token: Some("Bearer valid_token".to_string()),
            ..Default::default()
        };
        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_validate_when_twitter_token_missing_bearer_prefix_should_return_err() {
        let config = ProvidersConfig {
            twitter_bearer_token: Some("valid_token_without_bearer".to_string()),
            ..Default::default()
        };
        // This should now return an error
        let result = config.validate_config();
        assert!(result.is_err());
        if let Err(error) = result {
            let error_message = format!("{}", error);
            assert!(error_message.contains("TWITTER_BEARER_TOKEN must start with 'Bearer '"));
        }
    }

    #[test]
    fn test_validate_when_twitter_token_is_empty_should_return_ok() {
        let config = ProvidersConfig {
            twitter_bearer_token: Some("".to_string()),
            ..Default::default()
        };
        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_ai_provider_debug_display() {
        // Test Debug implementation
        assert_eq!(format!("{:?}", AiProvider::Anthropic), "Anthropic");
        assert_eq!(format!("{:?}", AiProvider::OpenAI), "OpenAI");
        assert_eq!(format!("{:?}", AiProvider::Groq), "Groq");
        assert_eq!(format!("{:?}", AiProvider::Perplexity), "Perplexity");
    }

    #[test]
    fn test_ai_provider_clone_and_copy() {
        let provider = AiProvider::Anthropic;
        let cloned = provider.clone();
        let copied = provider;

        assert_eq!(provider, cloned);
        assert_eq!(provider, copied);
    }

    #[test]
    fn test_ai_provider_equality() {
        assert_eq!(AiProvider::Anthropic, AiProvider::Anthropic);
        assert_ne!(AiProvider::Anthropic, AiProvider::OpenAI);
        assert_ne!(AiProvider::OpenAI, AiProvider::Groq);
        assert_ne!(AiProvider::Groq, AiProvider::Perplexity);
    }

    #[test]
    fn test_blockchain_provider_debug_display() {
        assert_eq!(format!("{:?}", BlockchainProvider::Alchemy), "Alchemy");
        assert_eq!(format!("{:?}", BlockchainProvider::Infura), "Infura");
        assert_eq!(format!("{:?}", BlockchainProvider::QuickNode), "QuickNode");
        assert_eq!(format!("{:?}", BlockchainProvider::Moralis), "Moralis");
    }

    #[test]
    fn test_blockchain_provider_clone_and_copy() {
        let provider = BlockchainProvider::Alchemy;
        let cloned = provider.clone();
        let copied = provider;

        assert_eq!(provider, cloned);
        assert_eq!(provider, copied);
    }

    #[test]
    fn test_blockchain_provider_equality() {
        assert_eq!(BlockchainProvider::Alchemy, BlockchainProvider::Alchemy);
        assert_ne!(BlockchainProvider::Alchemy, BlockchainProvider::Infura);
        assert_ne!(BlockchainProvider::Infura, BlockchainProvider::QuickNode);
        assert_ne!(BlockchainProvider::QuickNode, BlockchainProvider::Moralis);
    }

    #[test]
    fn test_data_provider_debug_display() {
        assert_eq!(format!("{:?}", DataProvider::DexScreener), "DexScreener");
        assert_eq!(format!("{:?}", DataProvider::CoinGecko), "CoinGecko");
        assert_eq!(
            format!("{:?}", DataProvider::CoinMarketCap),
            "CoinMarketCap"
        );
        assert_eq!(format!("{:?}", DataProvider::Twitter), "Twitter");
        assert_eq!(format!("{:?}", DataProvider::LunarCrush), "LunarCrush");
    }

    #[test]
    fn test_data_provider_clone_and_copy() {
        let provider = DataProvider::DexScreener;
        let cloned = provider.clone();
        let copied = provider;

        assert_eq!(provider, cloned);
        assert_eq!(provider, copied);
    }

    #[test]
    fn test_data_provider_equality() {
        assert_eq!(DataProvider::DexScreener, DataProvider::DexScreener);
        assert_ne!(DataProvider::DexScreener, DataProvider::CoinGecko);
        assert_ne!(DataProvider::CoinGecko, DataProvider::CoinMarketCap);
        assert_ne!(DataProvider::Twitter, DataProvider::LunarCrush);
    }

    #[test]
    fn test_providers_config_debug_display() {
        let config = ProvidersConfig {
            anthropic_api_key: Some("test_key".to_string()),
            ..Default::default()
        };
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ProvidersConfig"));
        assert!(debug_str.contains("anthropic_api_key"));
    }

    #[test]
    fn test_providers_config_clone() {
        let config = ProvidersConfig {
            anthropic_api_key: Some("test_key".to_string()),
            openai_api_key: Some("openai_key".to_string()),
            ..Default::default()
        };
        let cloned = config.clone();

        assert_eq!(config.anthropic_api_key, cloned.anthropic_api_key);
        assert_eq!(config.openai_api_key, cloned.openai_api_key);
        assert_eq!(config.groq_api_key, cloned.groq_api_key);
    }

    #[test]
    fn test_serde_serialization_deserialization() {
        let config = ProvidersConfig {
            anthropic_api_key: Some("anthropic_test".to_string()),
            openai_api_key: Some("openai_test".to_string()),
            twitter_bearer_token: Some("Bearer twitter_test".to_string()),
            ..Default::default()
        };

        // Test serialization
        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("anthropic_test"));
        assert!(serialized.contains("openai_test"));
        assert!(serialized.contains("Bearer twitter_test"));

        // Test deserialization
        let deserialized: ProvidersConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config.anthropic_api_key, deserialized.anthropic_api_key);
        assert_eq!(config.openai_api_key, deserialized.openai_api_key);
        assert_eq!(
            config.twitter_bearer_token,
            deserialized.twitter_bearer_token
        );
    }

    #[test]
    fn test_serde_default_fields() {
        // Test that fields are properly defaulted when not present in JSON
        let json = "{}";
        let config: ProvidersConfig = serde_json::from_str(json).unwrap();

        assert!(config.anthropic_api_key.is_none());
        assert!(config.openai_api_key.is_none());
        assert!(config.groq_api_key.is_none());
        assert!(config.perplexity_api_key.is_none());
    }

    #[test]
    fn test_mixed_provider_configuration() {
        let config = ProvidersConfig {
            anthropic_api_key: Some("anthropic_key".to_string()),
            alchemy_api_key: Some("alchemy_key".to_string()),
            dexscreener_api_key: Some("dex_key".to_string()),
            // Leave others as None
            ..Default::default()
        };

        // Test AI providers
        assert!(config.has_ai_provider(AiProvider::Anthropic));
        assert!(!config.has_ai_provider(AiProvider::OpenAI));

        // Test blockchain providers
        assert!(config.has_blockchain_provider(BlockchainProvider::Alchemy));
        assert!(!config.has_blockchain_provider(BlockchainProvider::Infura));

        // Test data providers
        assert!(config.has_data_provider(DataProvider::DexScreener));
        assert!(!config.has_data_provider(DataProvider::CoinGecko));
    }
}
