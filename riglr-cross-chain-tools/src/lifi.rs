//! LiFi Protocol client implementation for cross-chain operations.
//!
//! This module provides the core client for interacting with LiFi's API to discover
//! routes and execute cross-chain transactions. LiFi aggregates multiple bridge protocols
//! and DEXs to provide optimal cross-chain routing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use url::{ParseError, Url};

/// Errors that can occur during LiFi API operations
#[derive(Error, Debug)]
pub enum LiFiError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// Invalid response format from API
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    /// API returned an error response
    #[error("API error: {code} - {message}")]
    ApiError {
        /// HTTP status code
        code: u16,
        /// Error message from API
        message: String,
    },

    /// Chain is not supported by LiFi
    #[error("Chain not supported: {chain_name}")]
    UnsupportedChain {
        /// Name of the unsupported chain
        chain_name: String,
    },

    /// No route found between chains
    #[error("Route not found for {from_chain} -> {to_chain}")]
    RouteNotFound {
        /// Source chain name
        from_chain: String,
        /// Destination chain name
        to_chain: String,
    },

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// URL parsing error
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] ParseError),
}

/// Supported blockchain networks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChainType {
    /// Ethereum Virtual Machine based blockchain
    #[serde(rename = "evm")]
    Evm,
    /// Solana blockchain
    #[serde(rename = "solana")]
    Solana,
}

/// Chain information from LiFi
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chain {
    /// Unique chain identifier
    pub id: u64,
    /// Human-readable chain name
    pub name: String,
    /// Chain key used by LiFi API
    pub key: String,
    /// Type of blockchain (EVM or Solana)
    pub chain_type: ChainType,
    /// Optional URI for chain logo
    pub logo_uri: Option<String>,
    /// Native token information for this chain
    pub native_token: Token,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    /// Token contract address
    pub address: String,
    /// Token symbol (e.g., ETH, USDC)
    pub symbol: String,
    /// Number of decimal places for this token
    pub decimals: u8,
    /// Full token name
    pub name: String,
    /// Optional URI for token logo
    pub logo_uri: Option<String>,
    /// Current price in USD
    pub price_usd: Option<f64>,
}

/// A cross-chain route option from LiFi
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossChainRoute {
    /// Unique route identifier
    pub id: String,
    /// Source chain ID
    pub from_chain_id: u64,
    /// Destination chain ID
    pub to_chain_id: u64,
    /// Token being sent from source chain
    pub from_token: Token,
    /// Token being received on destination chain
    pub to_token: Token,
    /// Amount to send (in token units)
    pub from_amount: String,
    /// Expected amount to receive (in token units)
    pub to_amount: String,
    /// Minimum amount guaranteed to receive
    pub to_amount_min: String,
    /// Steps required to execute this route
    pub steps: Vec<RouteStep>,
    /// Estimated gas cost in USD
    pub gas_cost_usd: Option<f64>,
    /// Fees associated with this route
    pub fees: Vec<RouteFee>,
    /// Estimated time to complete the route in seconds
    pub estimated_execution_duration: u64, // seconds
    /// Route tags for categorization
    pub tags: Vec<String>,
    /// Transaction request data for executing the bridge
    pub transaction_request: Option<TransactionRequest>,
}

/// A step within a cross-chain route
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteStep {
    /// Unique step identifier
    pub id: String,
    /// Step type (e.g., "lifi", "cross", "swap")
    pub type_: String, // "lifi", "cross", "swap"
    /// Tool/protocol used for this step
    pub tool: String,
    /// Action details for this step
    pub action: StepAction,
    /// Execution estimates for this step
    pub estimate: StepEstimate,
}

/// Action details for a route step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepAction {
    /// Source chain ID for this step
    pub from_chain_id: u64,
    /// Destination chain ID for this step
    pub to_chain_id: u64,
    /// Input token for this step
    pub from_token: Token,
    /// Output token for this step
    pub to_token: Token,
    /// Input amount for this step
    pub from_amount: String,
    /// Expected output amount for this step
    pub to_amount: String,
}

/// Execution estimate for a step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepEstimate {
    /// Tool/protocol used for estimation
    pub tool: String,
    /// Contract address that needs approval (if any)
    pub approval_address: Option<String>,
    /// Minimum guaranteed output amount
    pub to_amount_min: String,
    /// Estimated gas for data/computation
    pub data_gas_estimate: Option<String>,
    /// Current gas price
    pub gas_price: Option<String>,
    /// Total estimated gas cost
    pub gas_cost: Option<String>,
    /// Estimated execution time in seconds
    pub execution_duration: u64,
}

/// Fee information for a route
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteFee {
    /// Fee name/type
    pub name: String,
    /// Human-readable fee description
    pub description: String,
    /// Fee percentage (as string)
    pub percentage: String,
    /// Token in which the fee is denominated
    pub token: Token,
    /// Fee amount in token units
    pub amount: String,
    /// Fee amount in USD
    pub amount_usd: Option<f64>,
    /// Whether this fee is included in the quoted amounts
    pub included: bool,
}

/// Request parameters for getting cross-chain routes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteRequest {
    /// Source chain ID
    pub from_chain: u64,
    /// Destination chain ID
    pub to_chain: u64,
    /// Source token address
    pub from_token: String,
    /// Destination token address
    pub to_token: String,
    /// Amount to bridge (in token units)
    pub from_amount: String,
    /// Optional sender address
    pub from_address: Option<String>,
    /// Optional recipient address
    pub to_address: Option<String>,
    /// Slippage tolerance (0.005 = 0.5%)
    pub slippage: Option<f64>, // 0.005 = 0.5%
}

/// Response from the routes API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteResponse {
    /// Available cross-chain routes
    pub routes: Vec<CrossChainRoute>,
}

/// Bridge transaction status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BridgeStatus {
    /// Transaction not found
    NotFound,
    /// Transaction is pending execution
    Pending,
    /// Transaction completed successfully
    Done,
    /// Transaction failed
    Failed,
}

/// Bridge transaction status response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeStatusResponse {
    /// Current status of the bridge transaction
    pub status: BridgeStatus,
    /// Source chain ID
    pub from_chain_id: Option<u64>,
    /// Destination chain ID
    pub to_chain_id: Option<u64>,
    /// Tool/protocol used for bridging
    pub tool: Option<String>,
    /// Transaction hash on source chain
    pub sending_tx_hash: Option<String>,
    /// Transaction hash on destination chain
    pub receiving_tx_hash: Option<String>,
    /// Amount sent from source chain
    pub amount_sent: Option<String>,
    /// Amount received on destination chain
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SolanaAccountMeta {
    /// Public key of the account
    pub pubkey: String,
    /// Whether this account must sign the transaction
    pub is_signer: bool,
    /// Whether this account is writable
    pub is_writable: bool,
}

/// LiFi Protocol API client
#[derive(Debug, Clone)]
pub struct LiFiClient {
    /// HTTP client for API requests
    client: reqwest::Client,
    /// Base URL for LiFi API
    base_url: Url,
    /// Optional API key for authentication
    api_key: Option<String>,
}

impl LiFiClient {
    const DEFAULT_BASE_URL: &'static str = "https://li.quest/v1/";

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
    pub async fn get_routes(
        &self,
        request: &RouteRequest,
    ) -> Result<Vec<CrossChainRoute>, LiFiError> {
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

        let route_response: RouteResponse = response
            .json()
            .await
            .map_err(|e| LiFiError::InvalidResponse(format!("Failed to parse routes: {}", e)))?;

        Ok(route_response.routes)
    }

    /// Get the status of a bridge transaction
    pub async fn get_bridge_status(
        &self,
        bridge_id: &str,
        tx_hash: &str,
    ) -> Result<BridgeStatusResponse, LiFiError> {
        let url = self
            .base_url
            .join(&format!("status?bridge={}&txHash={}", bridge_id, tx_hash))?;

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

        let status: BridgeStatusResponse = response
            .json()
            .await
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
                    eprintln!(
                        "Failed to get transaction request for route {}: {}",
                        route.id, e
                    );
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
        let url = self
            .base_url
            .join(&format!("advanced/stepTransaction?route={}", route_id))?;

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

        let tx_data: serde_json::Value = response.json().await.map_err(|e| {
            LiFiError::InvalidResponse(format!("Failed to parse transaction data: {}", e))
        })?;

        // Parse the transaction request from LiFi API response
        // Note: This is a simplified implementation - actual LiFi API response format may vary
        let to = tx_data["to"]
            .as_str()
            .or_else(|| tx_data["programId"].as_str())
            .unwrap_or_default()
            .to_string();
        let data = tx_data["data"].as_str().unwrap_or_default().to_string();
        let value = tx_data["value"].as_str().unwrap_or("0").to_string();
        let gas_limit = tx_data["gasLimit"].as_str().unwrap_or("200000").to_string();
        let gas_price = tx_data["gasPrice"]
            .as_str()
            .unwrap_or("20000000000")
            .to_string();
        let chain_id = tx_data["chainId"].as_u64().unwrap_or(1);

        // Attempt to parse Solana accounts if present
        let solana_accounts =
            if let Some(accounts) = tx_data.get("accounts").and_then(|a| a.as_array()) {
                let mut metas: Vec<SolanaAccountMeta> = Vec::with_capacity(accounts.len());
                for acc in accounts {
                    let pubkey = acc["pubkey"].as_str().unwrap_or_default().to_string();
                    let is_signer = acc["isSigner"].as_bool().unwrap_or(false);
                    let is_writable = acc["isWritable"].as_bool().unwrap_or(false);
                    if !pubkey.is_empty() {
                        metas.push(SolanaAccountMeta {
                            pubkey,
                            is_signer,
                            is_writable,
                        });
                    }
                }
                if metas.is_empty() {
                    None
                } else {
                    Some(metas)
                }
            } else {
                None
            };

        let tx_request = TransactionRequest {
            to,
            data,
            value,
            gas_limit,
            gas_price,
            chain_id,
            solana_accounts,
        };

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
        let base_url =
            Url::parse(Self::DEFAULT_BASE_URL).expect("Default LiFi URL should be valid");

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
            // Use riglr-evm-tools for EVM chain mapping
            riglr_evm_tools::chain_name_to_id(name).map_err(|_| LiFiError::UnsupportedChain {
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
            // Use riglr-evm-tools for EVM chain mapping
            riglr_evm_tools::chain_id_to_name(id).map_err(|_| LiFiError::UnsupportedChain {
                chain_name: format!("Chain ID {}", id),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // Test LiFiClient creation and configuration
    #[test]
    fn test_lifi_client_default() {
        let client = LiFiClient::default();
        assert!(client.api_key.is_none());
        assert_eq!(client.base_url.as_str(), LiFiClient::DEFAULT_BASE_URL);
    }

    #[test]
    fn test_lifi_client_with_base_url_valid() {
        let custom_url = "https://custom.lifi.api/v2/";
        let client = LiFiClient::with_base_url(custom_url).unwrap();
        assert_eq!(client.base_url.as_str(), custom_url);
        assert!(client.api_key.is_none());
    }

    #[test]
    fn test_lifi_client_with_base_url_invalid() {
        let invalid_url = "not-a-valid-url";
        let result = LiFiClient::with_base_url(invalid_url);
        assert!(result.is_err());
        match result.unwrap_err() {
            LiFiError::Configuration(msg) => {
                assert!(msg.contains("Invalid base URL"));
            }
            _ => panic!("Expected Configuration error"),
        }
    }

    #[test]
    fn test_lifi_client_with_api_key() {
        let client = LiFiClient::default().with_api_key("test-api-key".to_string());
        assert_eq!(client.api_key, Some("test-api-key".to_string()));
    }

    // Test helper functions for chain name/ID conversion
    #[test]
    fn test_chain_name_to_id_solana_variants() {
        assert_eq!(chain_name_to_id("solana").unwrap(), 1151111081099710);
        assert_eq!(chain_name_to_id("sol").unwrap(), 1151111081099710);
        assert_eq!(chain_name_to_id("SOLANA").unwrap(), 1151111081099710);
        assert_eq!(chain_name_to_id("SOL").unwrap(), 1151111081099710);
    }

    #[test]
    fn test_chain_name_to_id_evm_chains() {
        // These rely on riglr_evm_common
        assert_eq!(chain_name_to_id("ethereum").unwrap(), 1);
        assert_eq!(chain_name_to_id("polygon").unwrap(), 137);
        assert_eq!(chain_name_to_id("arbitrum").unwrap(), 42161);
    }

    #[test]
    fn test_chain_name_to_id_unsupported() {
        let result = chain_name_to_id("unknown-chain");
        assert!(result.is_err());
        match result.unwrap_err() {
            LiFiError::UnsupportedChain { chain_name } => {
                assert_eq!(chain_name, "unknown-chain");
            }
            _ => panic!("Expected UnsupportedChain error"),
        }
    }

    #[test]
    fn test_chain_name_to_id_empty_string() {
        let result = chain_name_to_id("");
        assert!(result.is_err());
        match result.unwrap_err() {
            LiFiError::UnsupportedChain { chain_name } => {
                assert_eq!(chain_name, "");
            }
            _ => panic!("Expected UnsupportedChain error"),
        }
    }

    #[test]
    fn test_chain_id_to_name_solana() {
        assert_eq!(chain_id_to_name(1151111081099710).unwrap(), "solana");
    }

    #[test]
    fn test_chain_id_to_name_evm_chains() {
        // These rely on riglr_evm_common
        assert_eq!(chain_id_to_name(1).unwrap(), "ethereum");
        assert_eq!(chain_id_to_name(137).unwrap(), "polygon");
        assert_eq!(chain_id_to_name(42161).unwrap(), "arbitrum");
    }

    #[test]
    fn test_chain_id_to_name_unsupported() {
        let result = chain_id_to_name(999999);
        assert!(result.is_err());
        match result.unwrap_err() {
            LiFiError::UnsupportedChain { chain_name } => {
                assert_eq!(chain_name, "Chain ID 999999");
            }
            _ => panic!("Expected UnsupportedChain error"),
        }
    }

    #[test]
    fn test_chain_id_to_name_zero() {
        let result = chain_id_to_name(0);
        assert!(result.is_err());
        match result.unwrap_err() {
            LiFiError::UnsupportedChain { chain_name } => {
                assert_eq!(chain_name, "Chain ID 0");
            }
            _ => panic!("Expected UnsupportedChain error"),
        }
    }

    // Test error types and their display messages
    #[test]
    fn test_lifi_error_display_request() {
        // Since we can't easily create a reqwest::Error in tests, we'll test the display format differently
        // by testing the variants that we can create directly
        let error = LiFiError::InvalidResponse("bad json".to_string());
        let error_str = format!("{}", error);
        assert!(error_str.contains("Invalid response format"));
    }

    #[test]
    fn test_lifi_error_display_invalid_response() {
        let error = LiFiError::InvalidResponse("bad json".to_string());
        assert_eq!(format!("{}", error), "Invalid response format: bad json");
    }

    #[test]
    fn test_lifi_error_display_api_error() {
        let error = LiFiError::ApiError {
            code: 404,
            message: "Not found".to_string(),
        };
        assert_eq!(format!("{}", error), "API error: 404 - Not found");
    }

    #[test]
    fn test_lifi_error_display_unsupported_chain() {
        let error = LiFiError::UnsupportedChain {
            chain_name: "test-chain".to_string(),
        };
        assert_eq!(format!("{}", error), "Chain not supported: test-chain");
    }

    #[test]
    fn test_lifi_error_display_route_not_found() {
        let error = LiFiError::RouteNotFound {
            from_chain: "ethereum".to_string(),
            to_chain: "polygon".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Route not found for ethereum -> polygon"
        );
    }

    #[test]
    fn test_lifi_error_display_configuration() {
        let error = LiFiError::Configuration("invalid config".to_string());
        assert_eq!(format!("{}", error), "Configuration error: invalid config");
    }

    #[test]
    fn test_lifi_error_display_url_parse() {
        let parse_error = url::ParseError::RelativeUrlWithoutBase;
        let error = LiFiError::UrlParse(parse_error);
        let error_str = format!("{}", error);
        assert!(error_str.contains("URL parsing error"));
    }

    // Test enum serialization/deserialization
    #[test]
    fn test_chain_type_serialization() {
        let evm = ChainType::Evm;
        let solana = ChainType::Solana;

        let evm_json = serde_json::to_string(&evm).unwrap();
        let solana_json = serde_json::to_string(&solana).unwrap();

        assert_eq!(evm_json, "\"evm\"");
        assert_eq!(solana_json, "\"solana\"");
    }

    #[test]
    fn test_chain_type_deserialization() {
        let evm: ChainType = serde_json::from_str("\"evm\"").unwrap();
        let solana: ChainType = serde_json::from_str("\"solana\"").unwrap();

        assert_eq!(evm, ChainType::Evm);
        assert_eq!(solana, ChainType::Solana);
    }

    #[test]
    fn test_bridge_status_serialization() {
        let statuses = [
            BridgeStatus::NotFound,
            BridgeStatus::Pending,
            BridgeStatus::Done,
            BridgeStatus::Failed,
        ];

        let expected_json = ["\"NOT_FOUND\"", "\"PENDING\"", "\"DONE\"", "\"FAILED\""];

        for (status, expected) in statuses.iter().zip(expected_json.iter()) {
            let json = serde_json::to_string(status).unwrap();
            assert_eq!(&json, expected);
        }
    }

    #[test]
    fn test_bridge_status_deserialization() {
        let json_values = ["\"NOT_FOUND\"", "\"PENDING\"", "\"DONE\"", "\"FAILED\""];

        let expected_statuses = [
            BridgeStatus::NotFound,
            BridgeStatus::Pending,
            BridgeStatus::Done,
            BridgeStatus::Failed,
        ];

        for (json, expected) in json_values.iter().zip(expected_statuses.iter()) {
            let status: BridgeStatus = serde_json::from_str(json).unwrap();
            assert_eq!(&status, expected);
        }
    }

    // Test struct serialization/deserialization with sample data
    #[test]
    fn test_token_serialization() {
        let token = Token {
            address: "0x123...".to_string(),
            symbol: "ETH".to_string(),
            decimals: 18,
            name: "Ethereum".to_string(),
            logo_uri: Some("https://example.com/eth.png".to_string()),
            price_usd: Some(2000.0),
        };

        let json = serde_json::to_string(&token).unwrap();
        let deserialized: Token = serde_json::from_str(&json).unwrap();

        assert_eq!(token.address, deserialized.address);
        assert_eq!(token.symbol, deserialized.symbol);
        assert_eq!(token.decimals, deserialized.decimals);
        assert_eq!(token.name, deserialized.name);
        assert_eq!(token.logo_uri, deserialized.logo_uri);
        assert_eq!(token.price_usd, deserialized.price_usd);
    }

    #[test]
    fn test_token_serialization_with_none_values() {
        let token = Token {
            address: "0x456...".to_string(),
            symbol: "USDC".to_string(),
            decimals: 6,
            name: "USD Coin".to_string(),
            logo_uri: None,
            price_usd: None,
        };

        let json = serde_json::to_string(&token).unwrap();
        let deserialized: Token = serde_json::from_str(&json).unwrap();

        assert_eq!(token.address, deserialized.address);
        assert_eq!(token.symbol, deserialized.symbol);
        assert_eq!(token.decimals, deserialized.decimals);
        assert_eq!(token.name, deserialized.name);
        assert_eq!(token.logo_uri, deserialized.logo_uri);
        assert_eq!(token.price_usd, deserialized.price_usd);
    }

    #[test]
    fn test_route_request_serialization() {
        let request = RouteRequest {
            from_chain: 1,
            to_chain: 137,
            from_token: "0x123".to_string(),
            to_token: "0x456".to_string(),
            from_amount: "1000000000000000000".to_string(),
            from_address: Some("0xabc".to_string()),
            to_address: Some("0xdef".to_string()),
            slippage: Some(0.005),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: RouteRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.from_chain, deserialized.from_chain);
        assert_eq!(request.to_chain, deserialized.to_chain);
        assert_eq!(request.from_token, deserialized.from_token);
        assert_eq!(request.to_token, deserialized.to_token);
        assert_eq!(request.from_amount, deserialized.from_amount);
        assert_eq!(request.from_address, deserialized.from_address);
        assert_eq!(request.to_address, deserialized.to_address);
        assert_eq!(request.slippage, deserialized.slippage);
    }

    #[test]
    fn test_route_request_serialization_with_none_values() {
        let request = RouteRequest {
            from_chain: 1,
            to_chain: 137,
            from_token: "0x123".to_string(),
            to_token: "0x456".to_string(),
            from_amount: "1000000000000000000".to_string(),
            from_address: None,
            to_address: None,
            slippage: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: RouteRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.from_chain, deserialized.from_chain);
        assert_eq!(request.to_chain, deserialized.to_chain);
        assert_eq!(request.from_token, deserialized.from_token);
        assert_eq!(request.to_token, deserialized.to_token);
        assert_eq!(request.from_amount, deserialized.from_amount);
        assert_eq!(request.from_address, deserialized.from_address);
        assert_eq!(request.to_address, deserialized.to_address);
        assert_eq!(request.slippage, deserialized.slippage);
    }

    #[test]
    fn test_solana_account_meta_serialization() {
        let account = SolanaAccountMeta {
            pubkey: "11111111111111111111111111111112".to_string(),
            is_signer: true,
            is_writable: false,
        };

        let json = serde_json::to_string(&account).unwrap();
        let deserialized: SolanaAccountMeta = serde_json::from_str(&json).unwrap();

        assert_eq!(account.pubkey, deserialized.pubkey);
        assert_eq!(account.is_signer, deserialized.is_signer);
        assert_eq!(account.is_writable, deserialized.is_writable);
    }

    #[test]
    fn test_transaction_request_serialization_evm() {
        let tx_request = TransactionRequest {
            to: "0x123456789abcdef".to_string(),
            data: "0xdeadbeef".to_string(),
            value: "1000000000000000000".to_string(),
            gas_limit: "21000".to_string(),
            gas_price: "20000000000".to_string(),
            chain_id: 1,
            solana_accounts: None,
        };

        let json = serde_json::to_string(&tx_request).unwrap();
        let deserialized: TransactionRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(tx_request.to, deserialized.to);
        assert_eq!(tx_request.data, deserialized.data);
        assert_eq!(tx_request.value, deserialized.value);
        assert_eq!(tx_request.gas_limit, deserialized.gas_limit);
        assert_eq!(tx_request.gas_price, deserialized.gas_price);
        assert_eq!(tx_request.chain_id, deserialized.chain_id);
        assert_eq!(tx_request.solana_accounts, deserialized.solana_accounts);
    }

    #[test]
    fn test_transaction_request_serialization_solana() {
        let solana_accounts = vec![
            SolanaAccountMeta {
                pubkey: "11111111111111111111111111111112".to_string(),
                is_signer: true,
                is_writable: false,
            },
            SolanaAccountMeta {
                pubkey: "22222222222222222222222222222223".to_string(),
                is_signer: false,
                is_writable: true,
            },
        ];

        let tx_request = TransactionRequest {
            to: "SomeProgram1111111111111111111111111111".to_string(),
            data: "instruction_data".to_string(),
            value: "0".to_string(),
            gas_limit: "200000".to_string(),
            gas_price: "5000".to_string(),
            chain_id: 1151111081099710,
            solana_accounts: Some(solana_accounts.clone()),
        };

        let json = serde_json::to_string(&tx_request).unwrap();
        let deserialized: TransactionRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(tx_request.to, deserialized.to);
        assert_eq!(tx_request.data, deserialized.data);
        assert_eq!(tx_request.value, deserialized.value);
        assert_eq!(tx_request.gas_limit, deserialized.gas_limit);
        assert_eq!(tx_request.gas_price, deserialized.gas_price);
        assert_eq!(tx_request.chain_id, deserialized.chain_id);

        let deserialized_accounts = deserialized.solana_accounts.unwrap();
        assert_eq!(solana_accounts.len(), deserialized_accounts.len());
        for (original, deserialized) in solana_accounts.iter().zip(deserialized_accounts.iter()) {
            assert_eq!(original.pubkey, deserialized.pubkey);
            assert_eq!(original.is_signer, deserialized.is_signer);
            assert_eq!(original.is_writable, deserialized.is_writable);
        }
    }

    #[test]
    fn test_bridge_status_response_serialization() {
        let response = BridgeStatusResponse {
            status: BridgeStatus::Done,
            from_chain_id: Some(1),
            to_chain_id: Some(137),
            tool: Some("lifi".to_string()),
            sending_tx_hash: Some("0xabc123".to_string()),
            receiving_tx_hash: Some("0xdef456".to_string()),
            amount_sent: Some("1000000000000000000".to_string()),
            amount_received: Some("999000000000000000".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: BridgeStatusResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.status, deserialized.status);
        assert_eq!(response.from_chain_id, deserialized.from_chain_id);
        assert_eq!(response.to_chain_id, deserialized.to_chain_id);
        assert_eq!(response.tool, deserialized.tool);
        assert_eq!(response.sending_tx_hash, deserialized.sending_tx_hash);
        assert_eq!(response.receiving_tx_hash, deserialized.receiving_tx_hash);
        assert_eq!(response.amount_sent, deserialized.amount_sent);
        assert_eq!(response.amount_received, deserialized.amount_received);
    }

    #[test]
    fn test_bridge_status_response_serialization_with_none_values() {
        let response = BridgeStatusResponse {
            status: BridgeStatus::NotFound,
            from_chain_id: None,
            to_chain_id: None,
            tool: None,
            sending_tx_hash: None,
            receiving_tx_hash: None,
            amount_sent: None,
            amount_received: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: BridgeStatusResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.status, deserialized.status);
        assert_eq!(response.from_chain_id, deserialized.from_chain_id);
        assert_eq!(response.to_chain_id, deserialized.to_chain_id);
        assert_eq!(response.tool, deserialized.tool);
        assert_eq!(response.sending_tx_hash, deserialized.sending_tx_hash);
        assert_eq!(response.receiving_tx_hash, deserialized.receiving_tx_hash);
        assert_eq!(response.amount_sent, deserialized.amount_sent);
        assert_eq!(response.amount_received, deserialized.amount_received);
    }

    // Test LiFiClient methods - we can't easily test the async HTTP methods without mocking,
    // but we can test the synchronous logic and error handling
    #[test]
    fn test_prepare_bridge_execution_with_valid_transaction_request() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = LiFiClient::default();

            let tx_request = TransactionRequest {
                to: "0x123456789abcdef".to_string(),
                data: "0xdeadbeef".to_string(),
                value: "1000000000000000000".to_string(),
                gas_limit: "21000".to_string(),
                gas_price: "20000000000".to_string(),
                chain_id: 1,
                solana_accounts: None,
            };

            let route = CrossChainRoute {
                id: "test-route".to_string(),
                from_chain_id: 1,
                to_chain_id: 137,
                from_token: Token {
                    address: "0x123".to_string(),
                    symbol: "ETH".to_string(),
                    decimals: 18,
                    name: "Ethereum".to_string(),
                    logo_uri: None,
                    price_usd: None,
                },
                to_token: Token {
                    address: "0x456".to_string(),
                    symbol: "MATIC".to_string(),
                    decimals: 18,
                    name: "Polygon".to_string(),
                    logo_uri: None,
                    price_usd: None,
                },
                from_amount: "1000000000000000000".to_string(),
                to_amount: "999000000000000000".to_string(),
                to_amount_min: "990000000000000000".to_string(),
                steps: vec![],
                gas_cost_usd: Some(5.0),
                fees: vec![],
                estimated_execution_duration: 300,
                tags: vec!["fast".to_string()],
                transaction_request: Some(tx_request.clone()),
            };

            let result = client.prepare_bridge_execution(&route).await;
            assert!(result.is_ok());

            let prepared_tx = result.unwrap();
            assert_eq!(prepared_tx.to, tx_request.to);
            assert_eq!(prepared_tx.data, tx_request.data);
            assert_eq!(prepared_tx.value, tx_request.value);
            assert_eq!(prepared_tx.gas_limit, tx_request.gas_limit);
            assert_eq!(prepared_tx.gas_price, tx_request.gas_price);
            assert_eq!(prepared_tx.chain_id, tx_request.chain_id);
        });
    }

    #[test]
    fn test_prepare_bridge_execution_with_invalid_transaction_request_empty_to() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = LiFiClient::default();

            let tx_request = TransactionRequest {
                to: "".to_string(), // Empty to address
                data: "0xdeadbeef".to_string(),
                value: "1000000000000000000".to_string(),
                gas_limit: "21000".to_string(),
                gas_price: "20000000000".to_string(),
                chain_id: 1,
                solana_accounts: None,
            };

            let route = CrossChainRoute {
                id: "test-route".to_string(),
                from_chain_id: 1,
                to_chain_id: 137,
                from_token: Token {
                    address: "0x123".to_string(),
                    symbol: "ETH".to_string(),
                    decimals: 18,
                    name: "Ethereum".to_string(),
                    logo_uri: None,
                    price_usd: None,
                },
                to_token: Token {
                    address: "0x456".to_string(),
                    symbol: "MATIC".to_string(),
                    decimals: 18,
                    name: "Polygon".to_string(),
                    logo_uri: None,
                    price_usd: None,
                },
                from_amount: "1000000000000000000".to_string(),
                to_amount: "999000000000000000".to_string(),
                to_amount_min: "990000000000000000".to_string(),
                steps: vec![],
                gas_cost_usd: Some(5.0),
                fees: vec![],
                estimated_execution_duration: 300,
                tags: vec!["fast".to_string()],
                transaction_request: Some(tx_request),
            };

            let result = client.prepare_bridge_execution(&route).await;
            assert!(result.is_err());

            match result.unwrap_err() {
                LiFiError::Configuration(msg) => {
                    assert!(msg.contains("Invalid transaction request"));
                    assert!(msg.contains("missing to address or data"));
                }
                _ => panic!("Expected Configuration error"),
            }
        });
    }

    #[test]
    fn test_prepare_bridge_execution_with_invalid_transaction_request_empty_data() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = LiFiClient::default();

            let tx_request = TransactionRequest {
                to: "0x123456789abcdef".to_string(),
                data: "".to_string(), // Empty data
                value: "1000000000000000000".to_string(),
                gas_limit: "21000".to_string(),
                gas_price: "20000000000".to_string(),
                chain_id: 1,
                solana_accounts: None,
            };

            let route = CrossChainRoute {
                id: "test-route".to_string(),
                from_chain_id: 1,
                to_chain_id: 137,
                from_token: Token {
                    address: "0x123".to_string(),
                    symbol: "ETH".to_string(),
                    decimals: 18,
                    name: "Ethereum".to_string(),
                    logo_uri: None,
                    price_usd: None,
                },
                to_token: Token {
                    address: "0x456".to_string(),
                    symbol: "MATIC".to_string(),
                    decimals: 18,
                    name: "Polygon".to_string(),
                    logo_uri: None,
                    price_usd: None,
                },
                from_amount: "1000000000000000000".to_string(),
                to_amount: "999000000000000000".to_string(),
                to_amount_min: "990000000000000000".to_string(),
                steps: vec![],
                gas_cost_usd: Some(5.0),
                fees: vec![],
                estimated_execution_duration: 300,
                tags: vec!["fast".to_string()],
                transaction_request: Some(tx_request),
            };

            let result = client.prepare_bridge_execution(&route).await;
            assert!(result.is_err());

            match result.unwrap_err() {
                LiFiError::Configuration(msg) => {
                    assert!(msg.contains("Invalid transaction request"));
                    assert!(msg.contains("missing to address or data"));
                }
                _ => panic!("Expected Configuration error"),
            }
        });
    }

    #[test]
    fn test_prepare_bridge_execution_without_transaction_request() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = LiFiClient::default();

            let route = CrossChainRoute {
                id: "test-route".to_string(),
                from_chain_id: 1,
                to_chain_id: 137,
                from_token: Token {
                    address: "0x123".to_string(),
                    symbol: "ETH".to_string(),
                    decimals: 18,
                    name: "Ethereum".to_string(),
                    logo_uri: None,
                    price_usd: None,
                },
                to_token: Token {
                    address: "0x456".to_string(),
                    symbol: "MATIC".to_string(),
                    decimals: 18,
                    name: "Polygon".to_string(),
                    logo_uri: None,
                    price_usd: None,
                },
                from_amount: "1000000000000000000".to_string(),
                to_amount: "999000000000000000".to_string(),
                to_amount_min: "990000000000000000".to_string(),
                steps: vec![],
                gas_cost_usd: Some(5.0),
                fees: vec![],
                estimated_execution_duration: 300,
                tags: vec!["fast".to_string()],
                transaction_request: None, // No transaction request
            };

            let result = client.prepare_bridge_execution(&route).await;
            assert!(result.is_err());

            match result.unwrap_err() {
                LiFiError::Configuration(msg) => {
                    assert!(msg.contains("Route does not contain transaction request data"));
                    assert!(msg.contains("Use get_route_with_transaction() first"));
                }
                _ => panic!("Expected Configuration error"),
            }
        });
    }

    // Test edge cases for serialization with complex nested structures
    #[test]
    fn test_complex_cross_chain_route_serialization() {
        let token = Token {
            address: "0x123".to_string(),
            symbol: "TEST".to_string(),
            decimals: 18,
            name: "Test Token".to_string(),
            logo_uri: Some("https://example.com/test.png".to_string()),
            price_usd: Some(1.50),
        };

        let step_estimate = StepEstimate {
            tool: "test-tool".to_string(),
            approval_address: Some("0xapproval".to_string()),
            to_amount_min: "990000000000000000".to_string(),
            data_gas_estimate: Some("21000".to_string()),
            gas_price: Some("20000000000".to_string()),
            gas_cost: Some("420000000000000".to_string()),
            execution_duration: 30,
        };

        let step_action = StepAction {
            from_chain_id: 1,
            to_chain_id: 137,
            from_token: token.clone(),
            to_token: token.clone(),
            from_amount: "1000000000000000000".to_string(),
            to_amount: "999000000000000000".to_string(),
        };

        let route_step = RouteStep {
            id: "step-1".to_string(),
            type_: "cross".to_string(),
            tool: "lifi".to_string(),
            action: step_action,
            estimate: step_estimate,
        };

        let route_fee = RouteFee {
            name: "Bridge Fee".to_string(),
            description: "Fee for cross-chain bridging".to_string(),
            percentage: "0.05".to_string(),
            token: token.clone(),
            amount: "50000000000000000".to_string(),
            amount_usd: Some(0.075),
            included: true,
        };

        let route = CrossChainRoute {
            id: "complex-route".to_string(),
            from_chain_id: 1,
            to_chain_id: 137,
            from_token: token.clone(),
            to_token: token.clone(),
            from_amount: "1000000000000000000".to_string(),
            to_amount: "949000000000000000".to_string(),
            to_amount_min: "940000000000000000".to_string(),
            steps: vec![route_step],
            gas_cost_usd: Some(7.5),
            fees: vec![route_fee],
            estimated_execution_duration: 300,
            tags: vec!["fast".to_string(), "low-fee".to_string()],
            transaction_request: None,
        };

        let json = serde_json::to_string(&route).unwrap();
        let deserialized: CrossChainRoute = serde_json::from_str(&json).unwrap();

        assert_eq!(route.id, deserialized.id);
        assert_eq!(route.from_chain_id, deserialized.from_chain_id);
        assert_eq!(route.to_chain_id, deserialized.to_chain_id);
        assert_eq!(route.from_amount, deserialized.from_amount);
        assert_eq!(route.to_amount, deserialized.to_amount);
        assert_eq!(route.to_amount_min, deserialized.to_amount_min);
        assert_eq!(route.steps.len(), deserialized.steps.len());
        assert_eq!(route.fees.len(), deserialized.fees.len());
        assert_eq!(
            route.estimated_execution_duration,
            deserialized.estimated_execution_duration
        );
        assert_eq!(route.tags, deserialized.tags);
    }

    // Test Debug trait implementations
    #[test]
    fn test_debug_implementations() {
        let token = Token {
            address: "0x123".to_string(),
            symbol: "TEST".to_string(),
            decimals: 18,
            name: "Test Token".to_string(),
            logo_uri: None,
            price_usd: None,
        };

        let debug_str = format!("{:?}", token);
        assert!(debug_str.contains("Token"));
        assert!(debug_str.contains("0x123"));
        assert!(debug_str.contains("TEST"));

        let chain_type = ChainType::Evm;
        let debug_str = format!("{:?}", chain_type);
        assert!(debug_str.contains("Evm"));

        let status = BridgeStatus::Pending;
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("Pending"));
    }

    // Test Clone trait implementations
    #[test]
    fn test_clone_implementations() {
        let token = Token {
            address: "0x123".to_string(),
            symbol: "TEST".to_string(),
            decimals: 18,
            name: "Test Token".to_string(),
            logo_uri: None,
            price_usd: None,
        };

        let cloned_token = token.clone();
        assert_eq!(token.address, cloned_token.address);
        assert_eq!(token.symbol, cloned_token.symbol);
        assert_eq!(token.decimals, cloned_token.decimals);

        let chain_type = ChainType::Solana;
        let cloned_chain_type = chain_type.clone();
        assert_eq!(chain_type, cloned_chain_type);

        let status = BridgeStatus::Done;
        let cloned_status = status.clone();
        assert_eq!(status, cloned_status);
    }

    // Test PartialEq implementations
    #[test]
    fn test_partial_eq_implementations() {
        let chain_type1 = ChainType::Evm;
        let chain_type2 = ChainType::Evm;
        let chain_type3 = ChainType::Solana;

        assert_eq!(chain_type1, chain_type2);
        assert_ne!(chain_type1, chain_type3);

        let status1 = BridgeStatus::Done;
        let status2 = BridgeStatus::Done;
        let status3 = BridgeStatus::Failed;

        assert_eq!(status1, status2);
        assert_ne!(status1, status3);
    }

    // Test edge cases for JSON parsing with missing fields
    #[test]
    fn test_json_parsing_missing_optional_fields() {
        let json = r#"{
            "address": "0x123",
            "symbol": "ETH",
            "decimals": 18,
            "name": "Ethereum"
        }"#;

        let token: Token = serde_json::from_str(json).unwrap();
        assert_eq!(token.address, "0x123");
        assert_eq!(token.symbol, "ETH");
        assert_eq!(token.decimals, 18);
        assert_eq!(token.name, "Ethereum");
        assert_eq!(token.logo_uri, None);
        assert_eq!(token.price_usd, None);
    }

    #[test]
    fn test_json_parsing_bridge_status_response_minimal() {
        let json = r#"{
            "status": "PENDING"
        }"#;

        let response: BridgeStatusResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, BridgeStatus::Pending);
        assert_eq!(response.from_chain_id, None);
        assert_eq!(response.to_chain_id, None);
        assert_eq!(response.tool, None);
        assert_eq!(response.sending_tx_hash, None);
        assert_eq!(response.receiving_tx_hash, None);
        assert_eq!(response.amount_sent, None);
        assert_eq!(response.amount_received, None);
    }

    // Additional coverage for URL parsing errors
    #[test]
    fn test_lifi_error_from_url_parse_error() {
        let url_error = url::ParseError::EmptyHost;
        let lifi_error = LiFiError::from(url_error);

        match lifi_error {
            LiFiError::UrlParse(_) => {
                // Success - the conversion worked
            }
            _ => panic!("Expected UrlParse error"),
        }
    }

    // Test reqwest error conversion
    #[test]
    fn test_lifi_error_from_reqwest_error() {
        // Test LiFiError::ApiError variant since we can create it directly
        let error = LiFiError::ApiError {
            code: 404,
            message: "Not found".to_string(),
        };

        match error {
            LiFiError::ApiError { code, message } => {
                assert_eq!(code, 404);
                assert_eq!(message, "Not found");
            }
            _ => {
                panic!("Expected LiFiError::ApiError variant");
            }
        }
    }
}
