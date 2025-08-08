//! Configuration management for riglr-showcase.

use anyhow::{Context, Result};
use std::env;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Solana RPC URL
    pub solana_rpc_url: String,

    /// Ethereum RPC URL  
    pub ethereum_rpc_url: String,

    /// Twitter Bearer Token
    pub twitter_bearer_token: Option<String>,

    /// Exa API Key
    pub exa_api_key: Option<String>,

    /// Neo4j connection string
    pub neo4j_url: String,

    /// Redis connection string
    pub redis_url: String,

    /// OpenAI API key for LLM
    pub openai_api_key: String,
}

impl Config {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            solana_rpc_url: env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
            ethereum_rpc_url: env::var("ETHEREUM_RPC_URL")
                .unwrap_or_else(|_| "https://eth-mainnet.alchemyapi.io/v2/demo".to_string()),
            twitter_bearer_token: env::var("TWITTER_BEARER_TOKEN").ok(),
            exa_api_key: env::var("EXA_API_KEY").ok(),
            neo4j_url: env::var("NEO4J_URL")
                .unwrap_or_else(|_| "neo4j://localhost:7687".to_string()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            openai_api_key: env::var("OPENAI_API_KEY")
                .context("OPENAI_API_KEY environment variable is required")?,
        })
    }
}
