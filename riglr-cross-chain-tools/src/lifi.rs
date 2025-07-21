//! LiFi Protocol client implementation for cross-chain operations.
//!
//! This module provides the core client for interacting with LiFi's API to discover
//! routes and execute cross-chain transactions. LiFi aggregates multiple bridge protocols
//! and DEXs to provide optimal cross-chain routing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use url::{Url, ParseError};

/// Errors that can occur during LiFi API operations
#[derive(Error, Debug)]
pub enum LiFiError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
    
    #[error("API error: {code} - {message}")]
    ApiError { code: u16, message: String },
    
    #[error("Chain not supported: {chain_name}")]
    UnsupportedChain { chain_name: String },
    
    #[error("Route not found for {from_chain} -> {to_chain}")]
    RouteNotFound { from_chain: String, to_chain: String },
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] ParseError),
}

/// Supported blockchain networks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChainType {
    #[serde(rename = "evm")]
    Evm,
    #[serde(rename = "solana")]
    Solana,
}

/// Chain information from LiFi
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chain {
    pub id: u64,
    pub name: String,
    pub key: String,
    pub chain_type: ChainType,
    pub logo_uri: Option<String>,
    pub native_token: Token,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub address: String,
    pub symbol: String,
    pub decimals: u8,
    pub name: String,
    pub logo_uri: Option<String>,
    pub price_usd: Option<f64>,
}

/// A cross-chain route option from LiFi
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossChainRoute {
    pub id: String,
    pub from_chain_id: u64,
    pub to_chain_id: u64,
    pub from_token: Token,
    pub to_token: Token,
    pub from_amount: String,
    pub to_amount: String,
    pub to_amount_min: String,
    pub steps: Vec<RouteStep>,
    pub gas_cost_usd: Option<f64>,
    pub fees: Vec<RouteFee>,
    pub estimated_execution_duration: u64, // seconds
    pub tags: Vec<String>,
    /// Transaction request data for executing the bridge
    pub transaction_request: Option<TransactionRequest>,
}

/// A step within a cross-chain route
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteStep {
    pub id: String,
    pub type_: String, // "lifi", "cross", "swap"
    pub tool: String,
    pub action: StepAction,
    pub estimate: StepEstimate,
}

/// Action details for a route step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepAction {
    pub from_chain_id: u64,
    pub to_chain_id: u64,
    pub from_token: Token,
    pub to_token: Token,
    pub from_amount: String,
    pub to_amount: String,
}

/// Execution estimate for a step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepEstimate {
    pub tool: String,
    pub approval_address: Option<String>,
    pub to_amount_min: String,
    pub data_gas_estimate: Option<String>,
    pub gas_price: Option<String>,
    pub gas_cost: Option<String>,
    pub execution_duration: u64,
}

/// Fee information for a route
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteFee {
    pub name: String,
    pub description: String,
    pub percentage: String,
    pub token: Token,
    pub amount: String,
    pub amount_usd: Option<f64>,
    pub included: bool,
}

/// Request parameters for getting cross-chain routes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteRequest {
    pub from_chain: u64,
    pub to_chain: u64,
    pub from_token: String,
    pub to_token: String,
    pub from_amount: String,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub slippage: Option<f64>, // 0.005 = 0.5%
}

/// Response from the routes API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteResponse {
    pub routes: Vec<CrossChainRoute>,
}

/// Bridge transaction status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BridgeStatus {
    NotFound,
    Pending,
    Done,
    Failed,
}

/// Bridge transaction status response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeStatusResponse {
    pub status: BridgeStatus,
    pub from_chain_id: Option<u64>,
    pub to_chain_id: Option<u64>,
    pub tool: Option<String>,
    pub sending_tx_hash: Option<String>,
    pub receiving_tx_hash: Option<String>,
    pub amount_sent: Option<String>,
    pub amount_received: Option<String>,
}

/// Transaction request data for executing cross-chain bridges
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    /// Target contract address
    pub to: String,
    /// Transaction data (hex encoded)
    pub data: String,
    /// Value to send (in wei for EVM chains)
    pub value: String,
    /// Gas limit for the transaction
    pub gas_limit: String,
    /// Gas price (in wei for EVM chains)
    pub gas_price: String,
    /// Chain ID for the transaction
    pub chain_id: u64,
    /// Solana specific account metas if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solana_accounts: Option<Vec<SolanaAccountMeta>>,
}

/// Solana account metadata for building instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaAccountMeta {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

/// LiFi Protocol API client
#[derive(Debug, Clone)]
pub struct LiFiClient {
    client: reqwest::Client,
    base_url: Url,
    api_key: Option<String>,
}

impl LiFiClient {
    const DEFAULT_BASE_URL: &'static str = "https://li.quest/v1/";
    
    /// Create a new LiFi client with default settings
    pub fn new() -> Result<Self, LiFiError> {
        Self::with_base_url(Self::DEFAULT_BASE_URL)
    }
    
    /// Create a new LiFi client with custom base URL
    pub fn with_base_url(base_url: &str) -> Result<Self, LiFiError> {
        let base_url = Url::parse(base_url)
            .map_err(|e| LiFiError::Configuration(format!("Invalid base URL: {}", e)))?;
            
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(format!("riglr-cross-chain-tools/{}", crate::VERSION))
            .build()?;
            
        Ok(Self {
            client,
            base_url,
            api_key: None,
        })
    }
    
    /// Set an API key for authenticated requests (optional)
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }
    
    /// Get available chains from LiFi
    pub async fn get_chains(&self) -> Result<Vec<Chain>, LiFiError> {
        let url = self.base_url.join("chains")?;
        
        let mut request = self.client.get(url);
        if let Some(ref api_key) = self.api_key {
            request = request.header("x-lifi-api-key", api_key);
        }
        
        let response = request.send().await?;
        
        if !response.status().is_success() {
            return Err(LiFiError::ApiError {
                code: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }
        
        let chains: HashMap<String, Chain> = response.json().await?;
        Ok(chains.into_values().collect())
    }
    
    /// Get cross-chain routes for a given request
    pub async fn get_routes(&self, request: &RouteRequest) -> Result<Vec<CrossChainRoute>, LiFiError> {
        let url = self.base_url.join("advanced/routes")?;
        
        let mut http_request = self.client.get(url);
        if let Some(ref api_key) = self.api_key {
            http_request = http_request.header("x-lifi-api-key", api_key);
        }
        
        // Convert request to query parameters
        let mut params = vec![
            ("fromChain", request.from_chain.to_string()),
            ("toChain", request.to_chain.to_string()),
            ("fromToken", request.from_token.clone()),
            ("toToken", request.to_token.clone()),
            ("fromAmount", request.from_amount.clone()),
        ];
        
        if let Some(ref from_address) = request.from_address {
            params.push(("fromAddress", from_address.clone()));
        }
        if let Some(ref to_address) = request.to_address {
            params.push(("toAddress", to_address.clone()));
        }
        if let Some(slippage) = request.slippage {
            params.push(("slippage", slippage.to_string()));
        }
        
        http_request = http_request.query(&params);
        
        let response = http_request.send().await?;
        
        if !response.status().is_success() {
            return Err(LiFiError::ApiError {
                code: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }
        
        let route_response: RouteResponse = response.json().await
            .map_err(|e| LiFiError::InvalidResponse(format!("Failed to parse routes: {}", e)))?;
            
        Ok(route_response.routes)
    }
    
    /// Get the status of a bridge transaction
    pub async fn get_bridge_status(
        &self,
        bridge_id: &str,
        tx_hash: &str,
    ) -> Result<BridgeStatusResponse, LiFiError> {
        let url = self.base_url.join(&format!("status?bridge={}&txHash={}", bridge_id, tx_hash))?;
        
        let mut request = self.client.get(url);
        if let Some(ref api_key) = self.api_key {
            request = request.header("x-lifi-api-key", api_key);
        }
        
        let response = request.send().await?;
        
        if !response.status().is_success() {
            return Err(LiFiError::ApiError {
                code: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }
        
        let status: BridgeStatusResponse = response.json().await
            .map_err(|e| LiFiError::InvalidResponse(format!("Failed to parse status: {}", e)))?;
            
        Ok(status)
    }

    /// Get a route with transaction request for bridge execution
    /// This method gets routes and includes the transaction data needed for execution
    pub async fn get_route_with_transaction(
        &self,
        request: &RouteRequest,
    ) -> Result<Vec<CrossChainRoute>, LiFiError> {
        let mut routes = self.get_routes(request).await?;
        
        // For each route, fetch the transaction request data
        for route in &mut routes {
            match self.get_transaction_request_for_route(&route.id).await {
                Ok(tx_request) => {
                    route.transaction_request = Some(tx_request);
                }
                Err(e) => {
                    // Log error but don't fail the entire request
                    eprintln!("Failed to get transaction request for route {}: {}", route.id, e);
                    route.transaction_request = None;
                }
            }
        }
        
        Ok(routes)
    }

    /// Get transaction request data for a specific route
    pub async fn get_transaction_request_for_route(
        &self,
        route_id: &str,
    ) -> Result<TransactionRequest, LiFiError> {
        let url = self.base_url.join(&format!("advanced/stepTransaction?route={}", route_id))?;
        
        let mut request = self.client.get(url);
        if let Some(ref api_key) = self.api_key {
            request = request.header("x-lifi-api-key", api_key);
        }
        
        let response = request.send().await?;
        
        if !response.status().is_success() {
            return Err(LiFiError::ApiError {
                code: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }
        
        let tx_data: serde_json::Value = response.json().await
            .map_err(|e| LiFiError::InvalidResponse(format!("Failed to parse transaction data: {}", e)))?;
        
        // Parse the transaction request from LiFi API response
        // Note: This is a simplified implementation - actual LiFi API response format may vary
        let to = tx_data["to"].as_str()
            .or_else(|| tx_data["programId"].as_str())
            .unwrap_or_default()
            .to_string();
        let data = tx_data["data"].as_str().unwrap_or_default().to_string();
        let value = tx_data["value"].as_str().unwrap_or("0").to_string();
        let gas_limit = tx_data["gasLimit"].as_str().unwrap_or("200000").to_string();
        let gas_price = tx_data["gasPrice"].as_str().unwrap_or("20000000000").to_string();
        let chain_id = tx_data["chainId"].as_u64().unwrap_or(1);

        // Attempt to parse Solana accounts if present
        let solana_accounts = if let Some(accounts) = tx_data.get("accounts").and_then(|a| a.as_array()) {
            let mut metas: Vec<SolanaAccountMeta> = Vec::with_capacity(accounts.len());
            for acc in accounts {
                let pubkey = acc["pubkey"].as_str().unwrap_or_default().to_string();
                let is_signer = acc["isSigner"].as_bool().unwrap_or(false);
                let is_writable = acc["isWritable"].as_bool().unwrap_or(false);
                if !pubkey.is_empty() {
                    metas.push(SolanaAccountMeta { pubkey, is_signer, is_writable });
                }
            }
            if metas.is_empty() { None } else { Some(metas) }
        } else {
            None
        };

        let tx_request = TransactionRequest { to, data, value, gas_limit, gas_price, chain_id, solana_accounts };
        
        Ok(tx_request)
    }

    /// Execute a cross-chain bridge transaction (requires integration with wallet/signer)
    /// This method prepares the transaction data but requires external signing
    pub async fn prepare_bridge_execution(
        &self,
        route: &CrossChainRoute,
    ) -> Result<TransactionRequest, LiFiError> {
        match &route.transaction_request {
            Some(tx_request) => {
                // Validate the transaction request
                if tx_request.to.is_empty() || tx_request.data.is_empty() {
                    return Err(LiFiError::Configuration(
                        "Invalid transaction request: missing to address or data".to_string()
                    ));
                }
                
                // Return the transaction request for external signing and execution
                Ok(tx_request.clone())
            }
            None => Err(LiFiError::Configuration(
                "Route does not contain transaction request data. Use get_route_with_transaction() first.".to_string()
            )),
        }
    }
}

impl Default for LiFiClient {
    fn default() -> Self {
        // Create a minimal client configuration that shouldn't fail
        let base_url = Url::parse(Self::DEFAULT_BASE_URL)
            .expect("Default LiFi URL should be valid");
            
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Default HTTP client should build successfully");
            
        Self {
            client,
            base_url,
            api_key: None,
        }
    }
}

/// Helper function to convert chain name to chain ID
pub fn chain_name_to_id(name: &str) -> Result<u64, LiFiError> {
    match name.to_lowercase().as_str() {
        "solana" | "sol" => Ok(1151111081099710), // Solana chain ID in LiFi
        _ => {
            // Use riglr-evm-common for EVM chain mapping
            riglr_evm_common::chain_name_to_id(name)
                .map_err(|_| LiFiError::UnsupportedChain {
                    chain_name: name.to_string(),
                })
        }
    }
}

/// Helper function to convert chain ID to chain name
pub fn chain_id_to_name(id: u64) -> Result<String, LiFiError> {
    match id {
        1151111081099710 => Ok("solana".to_string()),
        _ => {
            // Use riglr-evm-common for EVM chain mapping
            riglr_evm_common::chain_id_to_name(id)
                .map_err(|_| LiFiError::UnsupportedChain {
                    chain_name: format!("Chain ID {}", id),
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_lifi_client_creation() {
        let client = LiFiClient::new().unwrap();
        assert!(client.api_key.is_none());
    }
    
    #[tokio::test]
    async fn test_chain_name_conversion() {
        assert_eq!(chain_name_to_id("ethereum").unwrap(), 1);
        assert_eq!(chain_name_to_id("polygon").unwrap(), 137);
        assert_eq!(chain_name_to_id("solana").unwrap(), 1151111081099710);
        
        assert!(chain_name_to_id("unknown").is_err());
    }
    
    #[tokio::test]
    async fn test_chain_id_to_name() {
        assert_eq!(chain_id_to_name(1).unwrap(), "ethereum");
        assert_eq!(chain_id_to_name(137).unwrap(), "polygon");
        assert_eq!(chain_id_to_name(1151111081099710).unwrap(), "solana");
        
        assert!(chain_id_to_name(999999).is_err());
    }
}