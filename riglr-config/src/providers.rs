//! External API provider configuration

use crate::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};

/// External API providers configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProvidersConfig {
    // AI Providers
    #[serde(default)]
    pub anthropic_api_key: Option<String>,

    #[serde(default)]
    pub openai_api_key: Option<String>,

    #[serde(default)]
    pub groq_api_key: Option<String>,

    #[serde(default)]
    pub perplexity_api_key: Option<String>,

    // Blockchain Data Providers
    #[serde(default)]
    pub alchemy_api_key: Option<String>,

    #[serde(default)]
    pub infura_api_key: Option<String>,

    #[serde(default)]
    pub quicknode_api_key: Option<String>,

    #[serde(default)]
    pub moralis_api_key: Option<String>,

    // Cross-chain and DeFi
    #[serde(default)]
    pub lifi_api_key: Option<String>,

    #[serde(default)]
    pub one_inch_api_key: Option<String>,

    #[serde(default)]
    pub zerox_api_key: Option<String>,

    // Market Data
    #[serde(default)]
    pub dexscreener_api_key: Option<String>,

    #[serde(default)]
    pub coingecko_api_key: Option<String>,

    #[serde(default)]
    pub coinmarketcap_api_key: Option<String>,

    #[serde(default)]
    pub pump_api_key: Option<String>,

    // Social and Web Data
    #[serde(default)]
    pub twitter_bearer_token: Option<String>,

    #[serde(default)]
    pub exa_api_key: Option<String>,

    #[serde(default)]
    pub serper_api_key: Option<String>,

    // News and Analytics
    #[serde(default)]
    pub lunarcrush_api_key: Option<String>,

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

    pub fn validate(&self) -> ConfigResult<()> {
        // Validate API key formats
        if let Some(ref key) = self.anthropic_api_key {
            if key.is_empty() {
                return Err(ConfigError::validation("ANTHROPIC_API_KEY cannot be empty"));
            }
        }

        if let Some(ref token) = self.twitter_bearer_token {
            if !token.starts_with("Bearer ") && !token.is_empty() {
                tracing::warn!("Twitter bearer token should start with 'Bearer '");
            }
        }

        // More validations can be added as needed
        Ok(())
    }
}

/// AI provider enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiProvider {
    Anthropic,
    OpenAI,
    Groq,
    Perplexity,
}

/// Blockchain data provider enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockchainProvider {
    Alchemy,
    Infura,
    QuickNode,
    Moralis,
}

/// Data provider enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataProvider {
    DexScreener,
    CoinGecko,
    CoinMarketCap,
    Twitter,
    LunarCrush,
}
