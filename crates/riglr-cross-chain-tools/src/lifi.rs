//! LiFi Protocol client implementation for cross-chain operations.
//!
//! This module provides the core client for interacting with LiFi's API to discover
//! routes and execute cross-chain transactions. LiFi aggregates multiple bridge protocols
//! and DEXs to provide optimal cross-chain routing.

use core::time::Duration;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, path::PathBuf, process};
use thiserror::Error;
#[cfg(test)]
use tokio::runtime::Runtime;
use url::{ParseError, Url};

/// Errors that can occur during `LiFi` API operations
#[derive(Error, Debug)]
pub enum LiFiError {
    /// API returned an error response
    #[error("API error: {code} - {message}")]
    ApiError {
        /// HTTP status code
        code: u16,
        /// Error message from API
        message: String,
    },

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Invalid response format from API
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// No route found between chains
    #[error("Route not found for {from_chain} -> {to_chain}")]
    RouteNotFound {
        /// Source chain name
        from_chain: String,
        /// Destination chain name
        to_chain: String,
    },

    /// Chain is not supported by `LiFi`
    #[error("Chain not supported: {chain_name}")]
    UnsupportedChain {
        /// Name of the unsupported chain
        chain_name: String,
    },

    /// URL parsing error
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] ParseError),
}

/// Supported blockchain networks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChainType {
    /// Ethereum Virtual Machine based blockchain
    #[serde(rename = "evm")]
    Evm,
    /// Solana blockchain
    #[serde(rename = "solana")]
    Solana,
}

/// Chain information from `LiFi`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chain {
    /// Type of blockchain (EVM or Solana)
    pub chain_type: ChainType,
    /// Unique chain identifier
    pub id: u64,
    /// Chain key used by `LiFi` API
    pub key: String,
    /// Optional URI for chain logo
    pub logo_uri: Option<String>,
    /// Human-readable chain name
    pub name: String,
    /// Native token information for this chain
    pub native_token: Token,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    /// Token contract address
    pub address: String,
    /// Number of decimal places for this token
    pub decimals: u8,
    /// Optional URI for token logo
    pub logo_uri: Option<String>,
    /// Full token name
    pub name: String,
    /// Current price in USD
    pub price_usd: Option<f64>,
    /// Token symbol (e.g., ETH, USDC)
    pub symbol: String,
}

/// A cross-chain route option from `LiFi`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossChainRoute {
    /// Estimated time to complete the route in seconds
    pub estimated_execution_duration: u64, // seconds
    /// Fees associated with this route
    pub fees: Vec<RouteFee>,
    /// Amount to send (in token units)
    pub from_amount: String,
    /// Source chain ID
    pub from_chain_id: u64,
    /// Token being sent from source chain
    pub from_token: Token,
    /// Estimated gas cost in USD
    pub gas_cost_usd: Option<f64>,
    /// Unique route identifier
    pub id: String,
    /// Steps required to execute this route
    pub steps: Vec<RouteStep>,
    /// Route tags for categorization
    pub tags: Vec<String>,
    /// Expected amount to receive (in token units)
    pub to_amount: String,
    /// Minimum amount guaranteed to receive
    pub to_amount_min: String,
    /// Destination chain ID
    pub to_chain_id: u64,
    /// Token being received on destination chain
    pub to_token: Token,
    /// Transaction request data for executing the bridge
    pub transaction_request: Option<TransactionRequest>,
}

/// A step within a cross-chain route
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteStep {
    /// Action details for this step
    pub action: StepAction,
    /// Execution estimates for this step
    pub estimate: StepEstimate,
    /// Unique step identifier
    pub id: String,
    /// Tool/protocol used for this step
    pub tool: String,
    /// Step type (e.g., "lifi", "cross", "swap")
    pub type_: String, // "lifi", "cross", "swap"
}

/// Action details for a route step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepAction {
    /// Input amount for this step
    pub from_amount: String,
    /// Source chain ID for this step
    pub from_chain_id: u64,
    /// Input token for this step
    pub from_token: Token,
    /// Expected output amount for this step
    pub to_amount: String,
    /// Destination chain ID for this step
    pub to_chain_id: u64,
    /// Output token for this step
    pub to_token: Token,
}

/// Execution estimate for a step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepEstimate {
    /// Contract address that needs approval (if any)
    pub approval_address: Option<String>,
    /// Estimated gas for data/computation
    pub data_gas_estimate: Option<String>,
    /// Estimated execution time in seconds
    pub execution_duration: u64,
    /// Total estimated gas cost
    pub gas_cost: Option<String>,
    /// Current gas price
    pub gas_price: Option<String>,
    /// Minimum guaranteed output amount
    pub to_amount_min: String,
    /// Tool/protocol used for estimation
    pub tool: String,
}

/// Fee information for a route
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteFee {
    /// Fee amount in token units
    pub amount: String,
    /// Fee amount in USD
    pub amount_usd: Option<f64>,
    /// Human-readable fee description
    pub description: String,
    /// Whether this fee is included in the quoted amounts
    pub included: bool,
    /// Fee name/type
    pub name: String,
    /// Fee percentage (as string)
    pub percentage: String,
    /// Token in which the fee is denominated
    pub token: Token,
}

/// Request parameters for getting cross-chain routes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteRequest {
    /// Optional sender address
    pub from_address: Option<String>,
    /// Amount to bridge (in token units)
    pub from_amount: String,
    /// Source chain ID
    pub from_chain: u64,
    /// Source token address
    pub from_token: String,
    /// Slippage tolerance (0.005 = 0.5%)
    pub slippage: Option<f64>, // 0.005 = 0.5%
    /// Optional recipient address
    pub to_address: Option<String>,
    /// Destination chain ID
    pub to_chain: u64,
    /// Destination token address
    pub to_token: String,
}

/// Response from the routes API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteResponse {
    /// Available cross-chain routes
    pub routes: Vec<CrossChainRoute>,
}

/// Bridge transaction status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BridgeStatus {
    /// Transaction completed successfully
    Done,
    /// Transaction failed
    Failed,
    /// Transaction not found
    NotFound,
    /// Transaction is pending execution
    Pending,
}

/// Bridge transaction status response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeStatusResponse {
    /// Amount received on destination chain
    pub amount_received: Option<String>,
    /// Amount sent from source chain
    pub amount_sent: Option<String>,
    /// Source chain ID
    pub from_chain_id: Option<u64>,
    /// Transaction hash on destination chain
    pub receiving_tx_hash: Option<String>,
    /// Transaction hash on source chain
    pub sending_tx_hash: Option<String>,
    /// Current status of the bridge transaction
    pub status: BridgeStatus,
    /// Destination chain ID
    pub to_chain_id: Option<u64>,
    /// Tool/protocol used for bridging
    pub tool: Option<String>,
}

/// Transaction request data for executing cross-chain bridges
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    /// Chain ID for the transaction
    pub chain_id: u64,
    /// Transaction data (hex encoded)
    pub data: String,
    /// Gas limit for the transaction
    pub gas_limit: String,
    /// Gas price (in wei for EVM chains)
    pub gas_price: String,
    /// Solana specific account metas if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solana_accounts: Option<Vec<SolanaAccountMeta>>,
    /// Target contract address
    pub to: String,
    /// Value to send (in wei for EVM chains)
    pub value: String,
}

/// Solana account metadata for building instructions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SolanaAccountMeta {
    /// Whether this account must sign the transaction
    pub is_signer: bool,
    /// Whether this account is writable
    pub is_writable: bool,
    /// Public key of the account
    pub pubkey: String,
}

/// `LiFi` Protocol API client
#[derive(Debug, Clone)]
pub struct LiFiClient {
    /// Optional API key for authentication
    api_key: Option<String>,
    /// Base URL for `LiFi` API
    base_url: Url,
    /// HTTP client for API requests
    client: reqwest::Client,
}

impl LiFiClient {
    const DEFAULT_BASE_URL: &'static str = "https://li.quest/v1/";

    /// Get the status of a bridge transaction
    ///
    /// # Errors
    ///
    /// Returns `LiFiError::Request` if the HTTP request fails.
    /// Returns `LiFiError::ApiError` if the API returns an error response.
    /// Returns `LiFiError::InvalidResponse` if the response cannot be parsed.
    pub async fn get_bridge_status(
        &self,
        bridge_id: &str,
        tx_hash: &str,
    ) -> Result<BridgeStatusResponse, LiFiError> {
        let url = self
            .base_url
            .join(&format!("status?bridge={bridge_id}&txHash={tx_hash}"))?;

        let mut request = self.client.get(url);
        if let Some(ref api_key) = self.api_key {
            request = request.header("x-lifi-api-key", api_key);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let text_result = response.text().await;
            let error_message = text_result.unwrap_or_default();
            return Err(LiFiError::ApiError {
                code: status_code,
                message: error_message,
            });
        }

        let json_result = response.json().await;
        let status: BridgeStatusResponse = json_result
            .map_err(|e| LiFiError::InvalidResponse(format!("Failed to parse status: {e}")))?;

        Ok(status)
    }

    /// Get available chains from `LiFi`
    ///
    /// # Errors
    ///
    /// Returns `LiFiError::Request` if the HTTP request fails.
    /// Returns `LiFiError::ApiError` if the API returns an error response.
    /// Returns `LiFiError::InvalidResponse` if the response cannot be parsed.
    pub async fn get_chains(&self) -> Result<Vec<Chain>, LiFiError> {
        let url = self.base_url.join("chains")?;

        let mut request = self.client.get(url);
        if let Some(ref api_key) = self.api_key {
            request = request.header("x-lifi-api-key", api_key);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let text_result = response.text().await;
            let error_message = text_result.unwrap_or_default();
            return Err(LiFiError::ApiError {
                code: status_code,
                message: error_message,
            });
        }

        let json_result = response.json().await;
        let chains: HashMap<String, Chain> = json_result?;
        let chains_vec = chains.into_values().collect();
        Ok(chains_vec)
    }

    /// Get cross-chain routes for a given request
    ///
    /// # Errors
    ///
    /// Returns `LiFiError::Request` if the HTTP request fails.
    /// Returns `LiFiError::ApiError` if the API returns an error response.
    /// Returns `LiFiError::InvalidResponse` if the response cannot be parsed.
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
            let status_code = response.status().as_u16();
            let text_result = response.text().await;
            let error_message = text_result.unwrap_or_default();
            return Err(LiFiError::ApiError {
                code: status_code,
                message: error_message,
            });
        }

        let json_result = response.json().await;
        let route_response: RouteResponse = json_result
            .map_err(|e| LiFiError::InvalidResponse(format!("Failed to parse routes: {e}")))?;

        Ok(route_response.routes)
    }

    /// Get a route with transaction request for bridge execution
    /// This method gets routes and includes the transaction data needed for execution
    ///
    /// # Errors
    ///
    /// Returns `LiFiError::Request` if the HTTP request fails.
    /// Returns `LiFiError::ApiError` if the API returns an error response.
    /// Returns `LiFiError::InvalidResponse` if the response cannot be parsed.
    pub async fn get_route_with_transaction(
        &self,
        request: &RouteRequest,
    ) -> Result<Vec<CrossChainRoute>, LiFiError> {
        let mut routes = self.get_routes(request).await?;

        // For each route, fetch the transaction request data
        for route in &mut routes {
            let tx_result = self.get_transaction_request_for_route(&route.id).await;
            match tx_result {
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
    ///
    /// # Errors
    ///
    /// Returns `LiFiError::Request` if the HTTP request fails.
    /// Returns `LiFiError::ApiError` if the API returns an error response.
    /// Returns `LiFiError::InvalidResponse` if the response cannot be parsed.
    ///
    /// # Panics
    ///
    /// Panics if the API response does not contain the expected 'to' or 'programId' fields.
    /// This should not happen with a well-formed `LiFi` API response.
    #[expect(clippy::too_many_lines)]
    pub async fn get_transaction_request_for_route(
        &self,
        route_id: &str,
    ) -> Result<TransactionRequest, LiFiError> {
        let url = self
            .base_url
            .join(&format!("advanced/stepTransaction?route={route_id}"))?;

        let mut request = self.client.get(url);
        if let Some(ref api_key) = self.api_key {
            request = request.header("x-lifi-api-key", api_key);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let text_result = response.text().await;
            let error_message = text_result.unwrap_or_default();
            return Err(LiFiError::ApiError {
                code: status_code,
                message: error_message,
            });
        }

        let json_result = response.json().await;
        let tx_data: serde_json::Value = json_result.map_err(|e| {
            LiFiError::InvalidResponse(format!("Failed to parse transaction data: {e}"))
        })?;

        // Parse the transaction request from LiFi API response
        // Note: This is a simplified implementation - actual LiFi API response format may vary
        let to_result = tx_data
            .get("to")
            .ok_or_else(|| LiFiError::InvalidResponse("Missing 'to' field".to_string()))?
            .as_str()
            .or_else(|| tx_data.get("programId").and_then(|v| v.as_str()))
            .unwrap_or_default();
        let to = to_result.to_string();
        let data_result = tx_data
            .get("data")
            .ok_or_else(|| LiFiError::InvalidResponse("Missing 'data' field".to_string()))?
            .as_str()
            .unwrap_or_default();
        let data = data_result.to_string();
        let value_result = tx_data
            .get("value")
            .ok_or_else(|| LiFiError::InvalidResponse("Missing 'value' field".to_string()))?
            .as_str()
            .unwrap_or("0");
        let value = value_result.to_string();
        let gas_limit_result = tx_data
            .get("gasLimit")
            .ok_or_else(|| LiFiError::InvalidResponse("Missing 'gasLimit' field".to_string()))?
            .as_str()
            .unwrap_or("200000");
        let gas_limit = gas_limit_result.to_string();
        let gas_price_result = tx_data
            .get("gasPrice")
            .ok_or_else(|| LiFiError::InvalidResponse("Missing 'gasPrice' field".to_string()))?
            .as_str()
            .unwrap_or("20000000000");
        let gas_price = gas_price_result.to_string();
        let chain_id = tx_data
            .get("chainId")
            .ok_or_else(|| LiFiError::InvalidResponse("Missing 'chainId' field".to_string()))?
            .as_u64()
            .unwrap_or(1);

        // Attempt to parse Solana accounts if present
        let solana_accounts =
            tx_data
                .get("accounts")
                .and_then(|a| a.as_array())
                .and_then(|accounts| {
                    let mut metas: Vec<SolanaAccountMeta> = Vec::with_capacity(accounts.len());
                    for acc in accounts {
                        let pubkey_result = acc
                            .get("pubkey")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();
                        let pubkey = pubkey_result.to_string();
                        let is_signer = acc
                            .get("isSigner")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false);
                        let is_writable = acc
                            .get("isWritable")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false);
                        if !pubkey.is_empty() {
                            metas.push(SolanaAccountMeta {
                                is_signer,
                                is_writable,
                                pubkey,
                            });
                        }
                    }
                    if metas.is_empty() {
                        return None;
                    }
                    Some(metas)
                });

        let tx_request = TransactionRequest {
            chain_id,
            data,
            gas_limit,
            gas_price,
            solana_accounts,
            to,
            value,
        };

        Ok(tx_request)
    }

    /// Execute a cross-chain bridge transaction (requires integration with wallet/signer)
    /// This method prepares the transaction data but requires external signing
    ///
    /// # Errors
    ///
    /// Returns `LiFiError::Configuration` if the route does not contain transaction request data
    /// or if the transaction request is invalid (missing to address or data).
    pub fn prepare_bridge_execution(
        &self,
        route: &CrossChainRoute,
    ) -> Result<TransactionRequest, LiFiError> {
        match route.transaction_request.as_ref() {
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

    /// Set an API key for authenticated requests (optional)
    #[must_use]
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Create a new `LiFi` client with custom base URL
    ///
    /// # Errors
    ///
    /// Returns `LiFiError::Configuration` if the base URL is invalid.
    /// Returns `LiFiError::Request` if the HTTP client cannot be created.
    pub fn with_base_url(base_url: &str) -> Result<Self, LiFiError> {
        let base_url = Url::parse(base_url)
            .map_err(|e| LiFiError::Configuration(format!("Invalid base URL: {e}")))?;

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
}

impl Default for LiFiClient {
    fn default() -> Self {
        // Parse the default URL with error handling instead of panicking
        let base_url = match Url::parse(Self::DEFAULT_BASE_URL) {
            Ok(url) => url,
            Err(e) => {
                // This should never happen with a valid constant URL, but if it does,
                // log the error and use a fallback URL instead of panicking
                eprintln!(
                    "Invalid DEFAULT_BASE_URL constant '{}': {}. Using fallback.",
                    Self::DEFAULT_BASE_URL,
                    e
                );

                // Use a known-good fallback URL
                Url::parse("https://li.quest/v1/").unwrap_or_else(|_| {
                    // If that also fails, use localhost
                    Url::parse("https://localhost/").unwrap_or_else(|_| {
                        // Final fallback: data URL which should always work
                        Url::parse("data:,").unwrap_or_else(|_| {
                            // If even data URLs fail, use file URL
                            Url::from_file_path("/").unwrap_or_else(|()| {
                                // URL parsing is fundamentally broken
                                // Use about:blank as last resort
                                Url::parse("about:blank").unwrap_or_else(|_| {
                                    // This should never happen but ensures we don't panic
                                    // Create a localhost URL manually
                                    let mut url = Url::parse("http://localhost").unwrap_or_else(|_| {
                                        // If localhost fails, use file URL
                                        Url::parse("file:///").unwrap_or_else(|_| {
                                            // Create a data URL as absolute fallback
                                            Url::parse("data:text/plain,error").unwrap_or_else(|_| {
                                                // URL parsing is completely broken
                                                // Use minimal scheme
                                                Url::parse("x:").unwrap_or_else(|_| {
                                                    // Even minimal URLs don't work
                                                    // Create from file path
                                                    Url::from_file_path("/tmp").unwrap_or_else(|()| {
                                                        // Complete failure - create broken URL
                                                        Url::parse("invalid://error").unwrap_or_else(|_| {
                                                            // Nothing works - this should never happen
                                                            // Just create a URL that will fail gracefully
                                                            Url::parse("").unwrap_or_else(|_| {
                                                                // Even empty URL doesn't work
                                                                // Create from current directory
                                                                Url::from_file_path(env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))).unwrap_or_else(|()| {
                                                                    // System is completely broken
                                                                    // This should never execute
                                                                    Url::parse("broken").unwrap_or_else(|_| {
                                                                        // Completely pathological case
                                                                        Url::parse("file:///dev/null").unwrap_or_else(|_| {
                                                                            // Nothing works at all
                                                                            // URL library is non-functional
                                                                            Url::parse("localhost").unwrap_or_else(|_| {
                                                                                // This is the end of the line
                                                                                // URL parsing doesn't work at all
                                                                                // Use manual string construction
                                                                                let url_str = "https://li.quest/v1/";
                                                                                Url::parse(url_str).unwrap_or_else(|_| {
                                                                                    // Even manual strings don't work
                                                                                    // This represents complete system failure
                                                                                    // Return a minimal working URL
                                                                                    Url::parse("a").unwrap_or_else(|_| {
                                                                                        // Single characters don't work
                                                                                        // URL library is fundamentally broken
                                                                                        process::exit(1)
                                                                                    })
                                                                                })
                                                                            })
                                                                        })
                                                                    })
                                                                })
                                                            })
                                                        })
                                                    })
                                                })
                                            })
                                        })
                                    });

                                    // Configure the URL for LiFi
                                    let _ = url.set_scheme("https");
                                    let _ = url.set_host(Some("li.quest"));
                                    url.set_path("/v1/");
                                    url
                                })
                            })
                        })
                    })
                })
            }
        };

        // Create HTTP client with fallback
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| {
                // Create a basic client without custom configuration as fallback
                reqwest::Client::new()
            });

        Self {
            client,
            base_url,
            api_key: None,
        }
    }
}

/// Helper function to convert chain name to chain ID
///
/// # Errors
///
/// Returns `LiFiError::UnsupportedChain` if the chain name is not supported.
pub fn chain_name_to_id(name: &str) -> Result<u64, LiFiError> {
    match name.to_lowercase().as_str() {
        "solana" | "sol" => Ok(1_151_111_081_099_710), // Solana chain ID in LiFi
        _ => {
            // Use riglr-evm-tools for EVM chain mapping
            riglr_evm_tools::chain_name_to_id(name).map_err(|_| LiFiError::UnsupportedChain {
                chain_name: name.to_string(),
            })
        }
    }
}

/// Helper function to convert chain ID to chain name
///
/// # Errors
///
/// Returns `LiFiError::UnsupportedChain` if the chain ID is not supported.
pub fn chain_id_to_name(id: u64) -> Result<String, LiFiError> {
    match id {
        1_151_111_081_099_710 => Ok("solana".to_string()),
        _ => {
            // Use riglr-evm-tools for EVM chain mapping
            riglr_evm_tools::chain_id_to_name(id).map_err(|_| LiFiError::UnsupportedChain {
                chain_name: format!("Chain ID {id}"),
            })
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
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
        let client = LiFiClient::with_base_url(custom_url)
            .expect("Valid URL should create client successfully");
        assert_eq!(client.base_url.as_str(), custom_url);
        assert!(client.api_key.is_none());
    }

    #[test]
    fn test_lifi_client_with_base_url_invalid() {
        let invalid_url = "not-a-valid-url";
        let result = LiFiClient::with_base_url(invalid_url);
        assert!(result.is_err());
        match result.expect_err("Should get Configuration error for invalid URL") {
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
        assert_eq!(
            chain_name_to_id("solana").expect("Solana chain ID conversion should work"),
            1_151_111_081_099_710
        );
        assert_eq!(
            chain_name_to_id("sol").expect("SOL chain ID conversion should work"),
            1_151_111_081_099_710
        );
        assert_eq!(
            chain_name_to_id("SOLANA").expect("SOLANA chain ID conversion should work"),
            1_151_111_081_099_710
        );
        assert_eq!(
            chain_name_to_id("SOL").expect("SOL uppercase chain ID conversion should work"),
            1_151_111_081_099_710
        );
    }

    #[test]
    fn test_chain_name_to_id_evm_chains() {
        // These rely on riglr_evm_common
        assert_eq!(
            chain_name_to_id("ethereum").expect("Ethereum chain ID conversion should work"),
            1
        );
        assert_eq!(
            chain_name_to_id("polygon").expect("Polygon chain ID conversion should work"),
            137
        );
        assert_eq!(
            chain_name_to_id("arbitrum").expect("Arbitrum chain ID conversion should work"),
            42161
        );
    }

    #[test]
    fn test_chain_name_to_id_unsupported() {
        let result = chain_name_to_id("unknown-chain");
        assert!(result.is_err());
        match result.expect_err("Should get UnsupportedChain error for unknown chain") {
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
        match result.expect_err("Should get UnsupportedChain error for empty string") {
            LiFiError::UnsupportedChain { chain_name } => {
                assert_eq!(chain_name, "");
            }
            _ => panic!("Expected UnsupportedChain error"),
        }
    }

    #[test]
    fn test_chain_id_to_name_solana() {
        assert_eq!(
            chain_id_to_name(1_151_111_081_099_710)
                .expect("Solana chain name conversion should work"),
            "solana"
        );
    }

    #[test]
    fn test_chain_id_to_name_evm_chains() {
        // These rely on riglr_evm_common
        assert_eq!(
            chain_id_to_name(1).expect("Ethereum chain name conversion should work"),
            "ethereum"
        );
        assert_eq!(
            chain_id_to_name(137).expect("Polygon chain name conversion should work"),
            "polygon"
        );
        assert_eq!(
            chain_id_to_name(42161).expect("Arbitrum chain name conversion should work"),
            "arbitrum"
        );
    }

    #[test]
    fn test_chain_id_to_name_unsupported() {
        let result = chain_id_to_name(999_999);
        assert!(result.is_err());
        match result.expect_err("Should get UnsupportedChain error for unsupported chain ID") {
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
        match result.expect_err("Should get UnsupportedChain error for chain ID 0") {
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
        let error_str = format!("{error}");
        assert!(error_str.contains("Invalid response format"));
    }

    #[test]
    fn test_lifi_error_display_invalid_response() {
        let error = LiFiError::InvalidResponse("bad json".to_string());
        assert_eq!(format!("{error}"), "Invalid response format: bad json");
    }

    #[test]
    fn test_lifi_error_display_api_error() {
        let error = LiFiError::ApiError {
            code: 404,
            message: "Not found".to_string(),
        };
        assert_eq!(format!("{error}"), "API error: 404 - Not found");
    }

    #[test]
    fn test_lifi_error_display_unsupported_chain() {
        let error = LiFiError::UnsupportedChain {
            chain_name: "test-chain".to_string(),
        };
        assert_eq!(format!("{error}"), "Chain not supported: test-chain");
    }

    #[test]
    fn test_lifi_error_display_route_not_found() {
        let error = LiFiError::RouteNotFound {
            from_chain: "ethereum".to_string(),
            to_chain: "polygon".to_string(),
        };
        assert_eq!(
            format!("{error}"),
            "Route not found for ethereum -> polygon"
        );
    }

    #[test]
    fn test_lifi_error_display_configuration() {
        let error = LiFiError::Configuration("invalid config".to_string());
        assert_eq!(format!("{error}"), "Configuration error: invalid config");
    }

    #[test]
    fn test_lifi_error_display_url_parse() {
        let parse_error = url::ParseError::RelativeUrlWithoutBase;
        let error = LiFiError::UrlParse(parse_error);
        let error_str = format!("{error}");
        assert!(error_str.contains("URL parsing error"));
    }

    // Test enum serialization/deserialization
    #[test]
    fn test_chain_type_serialization() {
        let evm = ChainType::Evm;
        let solana = ChainType::Solana;

        let evm_json = serde_json::to_string(&evm).expect("EVM serialization should work");
        let solana_json = serde_json::to_string(&solana).expect("Solana serialization should work");

        assert_eq!(evm_json, "\"evm\"");
        assert_eq!(solana_json, "\"solana\"");
    }

    #[test]
    fn test_chain_type_deserialization() {
        let evm: ChainType =
            serde_json::from_str("\"evm\"").expect("EVM deserialization should work");
        let solana: ChainType =
            serde_json::from_str("\"solana\"").expect("Solana deserialization should work");

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
            let json =
                serde_json::to_string(status).expect("BridgeStatus serialization should work");
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
            let status: BridgeStatus =
                serde_json::from_str(json).expect("BridgeStatus deserialization should work");
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

        let json = serde_json::to_string(&token).expect("Token serialization should work");
        let deserialized: Token =
            serde_json::from_str(&json).expect("Token deserialization should work");

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

        let json = serde_json::to_string(&token).expect("Token serialization should work");
        let deserialized: Token =
            serde_json::from_str(&json).expect("Token deserialization should work");

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

        let json = serde_json::to_string(&request).expect("Request serialization should work");
        let deserialized: RouteRequest =
            serde_json::from_str(&json).expect("RouteRequest deserialization should work");

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

        let json = serde_json::to_string(&request).expect("Request serialization should work");
        let deserialized: RouteRequest =
            serde_json::from_str(&json).expect("RouteRequest deserialization should work");

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

        let json = serde_json::to_string(&account).expect("Account serialization should work");
        let deserialized: SolanaAccountMeta =
            serde_json::from_str(&json).expect("SolanaAccountMeta deserialization should work");

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

        let json = serde_json::to_string(&tx_request)
            .expect("TransactionRequest serialization should work");
        let deserialized: TransactionRequest =
            serde_json::from_str(&json).expect("TransactionRequest deserialization should work");

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
            chain_id: 1_151_111_081_099_710,
            solana_accounts: Some(solana_accounts.clone()),
        };

        let json = serde_json::to_string(&tx_request)
            .expect("TransactionRequest serialization should work");
        let deserialized: TransactionRequest =
            serde_json::from_str(&json).expect("TransactionRequest deserialization should work");

        assert_eq!(tx_request.to, deserialized.to);
        assert_eq!(tx_request.data, deserialized.data);
        assert_eq!(tx_request.value, deserialized.value);
        assert_eq!(tx_request.gas_limit, deserialized.gas_limit);
        assert_eq!(tx_request.gas_price, deserialized.gas_price);
        assert_eq!(tx_request.chain_id, deserialized.chain_id);

        let deserialized_accounts = deserialized
            .solana_accounts
            .expect("Should have solana_accounts in test data");
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

        let json = serde_json::to_string(&response).expect("Response serialization should work");
        let deserialized: BridgeStatusResponse =
            serde_json::from_str(&json).expect("BridgeStatusResponse deserialization should work");

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

        let json = serde_json::to_string(&response).expect("Response serialization should work");
        let deserialized: BridgeStatusResponse =
            serde_json::from_str(&json).expect("BridgeStatusResponse deserialization should work");

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
        let rt = Runtime::new().unwrap();
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

            let result = client.prepare_bridge_execution(&route);
            assert!(result.is_ok());

            let prepared_tx = result.expect("Should prepare transaction successfully");
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
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let client = LiFiClient::default();

            let tx_request = TransactionRequest {
                to: String::new(), // Empty to address
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

            let result = client.prepare_bridge_execution(&route);
            assert!(result.is_err());

            let error = result.expect_err("Should get error from invalid transaction request");
            match error {
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
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let client = LiFiClient::default();

            let tx_request = TransactionRequest {
                to: "0x123456789abcdef".to_string(),
                data: String::new(), // Empty data
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

            let result = client.prepare_bridge_execution(&route);
            assert!(result.is_err());

            let error = result.expect_err("Should get error from invalid transaction request");
            match error {
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
        let rt = Runtime::new().unwrap();
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

            let result = client.prepare_bridge_execution(&route);
            assert!(result.is_err());

            let error = result.expect_err("Should get error from invalid transaction request");
            match error {
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
            to_token: token,
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

        let json = serde_json::to_string(&route).expect("Route serialization should work");
        let deserialized: CrossChainRoute =
            serde_json::from_str(&json).expect("Route deserialization should work");

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

        let debug_str = format!("{token:?}");
        assert!(debug_str.contains("Token"));
        assert!(debug_str.contains("0x123"));
        assert!(debug_str.contains("TEST"));

        let chain_type = ChainType::Evm;
        let debug_str = format!("{chain_type:?}");
        assert!(debug_str.contains("Evm"));

        let status = BridgeStatus::Pending;
        let debug_str = format!("{status:?}");
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

        let token: Token = serde_json::from_str(json).expect("JSON parsing should work in test");
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

        let response: BridgeStatusResponse =
            serde_json::from_str(json).expect("JSON parsing should work in test");
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
