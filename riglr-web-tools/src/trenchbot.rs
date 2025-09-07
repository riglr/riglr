//! TrenchBot integration for Solana token bundle analysis and risk assessment
//!
//! This module provides tools for accessing TrenchBot API to analyze token bundles,
//! detect potential rug pulls, identify sniper wallets, and assess creator risk.

use crate::{client::WebClient, error::WebToolError};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Configuration for TrenchBot API access
#[derive(Debug, Clone)]
pub struct TrenchBotConfig {
    /// API base URL (default: https://trench.bot/api)
    pub base_url: String,
    /// Rate limit requests per minute (default: 60)
    pub rate_limit_per_minute: u32,
    /// Timeout for API requests in seconds (default: 30)
    pub request_timeout: u64,
}

impl Default for TrenchBotConfig {
    fn default() -> Self {
        Self {
            base_url: "https://trench.bot/api".to_string(),
            rate_limit_per_minute: 60,
            request_timeout: 30,
        }
    }
}

/// Main bundle response from TrenchBot
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BundleResponse {
    /// Indicates if the bundle is bonded
    pub bonded: Option<bool>,
    /// Collection of bundles, keyed by bundle ID
    pub bundles: Option<HashMap<String, BundleDetails>>,
    /// Analysis of the bundle creator
    pub creator_analysis: Option<CreatorAnalysis>,
    /// Total amount of tokens distributed
    pub distributed_amount: Option<i64>,
    /// Percentage of total tokens distributed
    pub distributed_percentage: Option<f64>,
    /// Number of wallets the token has been distributed to
    pub distributed_wallets: Option<i32>,
    /// Ticker symbol of the token
    pub ticker: Option<String>,
    /// Total number of bundles created for this token
    pub total_bundles: Option<i32>,
    /// Total amount of tokens held in bundles
    pub total_holding_amount: Option<i64>,
    /// Percentage of the total token supply held in bundles
    pub total_holding_percentage: Option<f64>,
    /// Total percentage of tokens that are bundled
    pub total_percentage_bundled: Option<f64>,
    /// Total SOL spent on creating the bundles
    pub total_sol_spent: Option<f64>,
    /// Total number of tokens included in bundles
    pub total_tokens_bundled: Option<i64>,
}

/// Details about a specific bundle
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BundleDetails {
    /// Analysis of the bundle
    pub bundle_analysis: Option<BundleAnalysis>,
    /// Amount of tokens held within the bundle
    pub holding_amount: Option<i64>,
    /// Percentage of total tokens held within the bundle
    pub holding_percentage: Option<f64>,
    /// Percentage of the total token supply within the bundle
    pub token_percentage: Option<f64>,
    /// Total SOL value of the bundle
    pub total_sol: Option<f64>,
    /// Total number of tokens within the bundle
    pub total_tokens: Option<i64>,
    /// Number of unique wallets interacting with the bundle
    pub unique_wallets: Option<i32>,
    /// Categories of wallets (e.g., "sniper", "regular"), keyed by wallet address
    pub wallet_categories: Option<HashMap<String, String>>,
    /// Information about individual wallets, keyed by wallet address
    pub wallet_info: Option<HashMap<String, WalletInfo>>,
}

/// Analysis results for a bundle
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BundleAnalysis {
    /// Breakdown of wallet categories within the bundle
    pub category_breakdown: Option<HashMap<String, i32>>,
    /// Indicates if the interaction is likely part of a bundle
    pub is_likely_bundle: Option<bool>,
    /// Primary category of the bundle (e.g., "regular", "sniper")
    pub primary_category: Option<String>,
}

/// Information about a specific wallet's interaction with a bundle
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WalletInfo {
    /// SOL balance of the wallet associated with this bundle
    pub sol: Option<f64>,
    /// Percentage of total SOL in the bundle represented by this wallet
    pub sol_percentage: Option<f64>,
    /// Percentage of total tokens in the bundle held by this wallet
    pub token_percentage: Option<f64>,
    /// Number of tokens held by this wallet within the bundle
    pub tokens: Option<i64>,
}

/// Analysis of the token creator
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatorAnalysis {
    /// Wallet address of the token creator
    pub address: Option<String>,
    /// Creator's current holdings of the token
    pub current_holdings: Option<i64>,
    /// Historical data on the creator's activity
    pub history: Option<CreatorHistory>,
    /// Percentage of the token supply currently held by the creator
    pub holding_percentage: Option<f64>,
    /// Assessed risk level of the creator
    pub risk_level: Option<RiskLevel>,
    /// Warning flags associated with this creator
    pub warning_flags: Option<Vec<String>>,
}

/// Historical data on the creator's activity
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatorHistory {
    /// Average market cap of tokens created by this creator
    pub average_market_cap: Option<i64>,
    /// Indicates if the creator is considered high risk
    pub high_risk: Option<bool>,
    /// List of previously created coins
    pub previous_coins: Option<Vec<PreviousCoin>>,
    /// Number of recent rug pulls by the creator
    pub recent_rugs: Option<i64>,
    /// Total number of rug pulls by the creator
    pub rug_count: Option<i64>,
    /// Percentage of the creator's tokens that were rug pulls
    pub rug_percentage: Option<f64>,
    /// Total number of coins created by this creator
    pub total_coins_created: Option<i64>,
}

/// Information about a previously created coin
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PreviousCoin {
    /// Timestamp of when the coin was created
    pub created_at: Option<i64>,
    /// Indicates if the coin was a rug pull
    pub is_rug: Option<bool>,
    /// Market cap of the coin
    pub market_cap: Option<i64>,
    /// Mint address of the coin
    pub mint: Option<String>,
    /// Symbol of the coin
    pub symbol: Option<String>,
}

/// Risk level enumeration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RiskLevel {
    /// Low risk - Creator has good track record
    #[serde(rename = "LOW")]
    Low,
    /// Medium risk - Some concerning patterns
    #[serde(rename = "MEDIUM")]
    Medium,
    /// High risk - Multiple red flags detected
    #[serde(rename = "HIGH")]
    High,
}

/// Simplified bundle analysis result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BundleAnalysisResult {
    /// Token address/mint
    pub token: String,
    /// Token ticker symbol
    pub ticker: Option<String>,
    /// Whether the token is likely bundled
    pub is_bundled: bool,
    /// Total percentage of supply bundled
    pub bundle_percentage: f64,
    /// Number of bundles detected
    pub bundle_count: i32,
    /// Total SOL spent on bundles
    pub total_sol_spent: f64,
    /// Number of wallets holding the token
    pub holder_count: i32,
    /// Creator risk assessment
    pub creator_risk: CreatorRiskAssessment,
    /// Wallet category breakdown
    pub wallet_categories: WalletCategoryBreakdown,
    /// Key warning flags
    pub warnings: Vec<String>,
    /// Summary recommendation
    pub recommendation: String,
}

/// Creator risk assessment
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatorRiskAssessment {
    /// Creator wallet address
    pub address: Option<String>,
    /// Risk level
    pub risk_level: Option<RiskLevel>,
    /// Percentage of tokens that were rugs
    pub rug_percentage: f64,
    /// Total coins created
    pub total_coins: i64,
    /// Number of rug pulls
    pub rug_count: i64,
    /// Creator's current holding percentage
    pub holding_percentage: f64,
}

/// Wallet category breakdown
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WalletCategoryBreakdown {
    /// Number of sniper wallets
    pub snipers: i32,
    /// Number of regular wallets
    pub regular: i32,
    /// Percentage of tokens held by snipers
    pub sniper_percentage: f64,
    /// Most common wallet category
    pub primary_category: String,
}

/// Get comprehensive bundle information for a Solana token from TrenchBot.
/// This tool analyzes token distribution patterns to detect bundling and assess risk.
#[tool]
pub async fn get_bundle_info(
    _context: &riglr_core::provider::ApplicationContext,
    token: String,
) -> crate::error::Result<BundleResponse> {
    debug!("Fetching bundle info for token: {}", token);

    let config = TrenchBotConfig::default();
    let client = WebClient::default();

    let url = format!("{}/bundle/bundle_advanced/{}", config.base_url, token);

    info!("Requesting bundle info from: {}", url);

    let response_text = client
        .get(&url)
        .await
        .map_err(|e| WebToolError::Network(format!("Failed to fetch bundle info: {}", e)))?;

    let bundle_response: BundleResponse = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Failed to parse TrenchBot response: {}", e)))?;

    info!(
        "Successfully fetched bundle info for {} - Bundles: {:?}, Bundled: {:?}%",
        token, bundle_response.total_bundles, bundle_response.total_percentage_bundled
    );

    Ok(bundle_response)
}

/// Extract wallet category statistics from bundle data
fn calculate_wallet_categories(
    bundles: &HashMap<String, BundleDetails>,
) -> (i32, i32, f64, String) {
    let mut sniper_count = 0;
    let mut regular_count = 0;
    let mut sniper_tokens = 0i64;
    let mut total_tokens = 0i64;

    for bundle in bundles.values() {
        if let Some(categories) = &bundle.wallet_categories {
            for category in categories.values() {
                match category.as_str() {
                    "sniper" => sniper_count += 1,
                    "regular" => regular_count += 1,
                    _ => {}
                }
            }
        }

        if let Some(wallet_info) = &bundle.wallet_info {
            for (addr, info) in wallet_info {
                if let Some(tokens) = info.tokens {
                    total_tokens += tokens;
                    if let Some(categories) = &bundle.wallet_categories {
                        if categories.get(addr).map_or(false, |c| c == "sniper") {
                            sniper_tokens += tokens;
                        }
                    }
                }
            }
        }
    }

    let sniper_percentage = if total_tokens > 0 {
        (sniper_tokens as f64 / total_tokens as f64) * 100.0
    } else {
        0.0
    };

    let primary_category = if sniper_count > regular_count {
        "sniper".to_string()
    } else {
        "regular".to_string()
    };

    (
        sniper_count,
        regular_count,
        sniper_percentage,
        primary_category,
    )
}

/// Build creator risk assessment from creator analysis data
fn build_creator_risk_assessment(
    creator_analysis: Option<&CreatorAnalysis>,
) -> CreatorRiskAssessment {
    if let Some(creator) = creator_analysis {
        CreatorRiskAssessment {
            address: creator.address.clone(),
            risk_level: creator.risk_level.clone(),
            rug_percentage: creator
                .history
                .as_ref()
                .and_then(|h| h.rug_percentage)
                .unwrap_or(0.0),
            total_coins: creator
                .history
                .as_ref()
                .and_then(|h| h.total_coins_created)
                .unwrap_or(0),
            rug_count: creator
                .history
                .as_ref()
                .and_then(|h| h.rug_count)
                .unwrap_or(0),
            holding_percentage: creator.holding_percentage.unwrap_or(0.0),
        }
    } else {
        CreatorRiskAssessment {
            address: None,
            risk_level: None,
            rug_percentage: 0.0,
            total_coins: 0,
            rug_count: 0,
            holding_percentage: 0.0,
        }
    }
}

/// Collect warning flags based on bundle data and analysis
fn collect_warnings(bundle_data: &BundleResponse, sniper_percentage: f64) -> Vec<String> {
    let mut warnings = Vec::new();

    if bundle_data.total_percentage_bundled.unwrap_or(0.0) > 50.0 {
        warnings.push("High bundle concentration (>50% bundled)".to_string());
    }

    if sniper_percentage > 30.0 {
        warnings.push("High sniper wallet presence".to_string());
    }

    if let Some(creator) = &bundle_data.creator_analysis {
        if let Some(flags) = &creator.warning_flags {
            warnings.extend(flags.clone());
        }

        if matches!(creator.risk_level, Some(RiskLevel::High)) {
            warnings.push("Creator has high risk profile".to_string());
        }

        if creator.holding_percentage.unwrap_or(0.0) > 20.0 {
            warnings.push("Creator holds >20% of supply".to_string());
        }
    }

    warnings
}

/// Generate recommendation based on bundle percentage
fn generate_recommendation(bundle_percentage: f64) -> String {
    if bundle_percentage > 70.0 {
        "EXTREME CAUTION: Very high bundle concentration detected. High risk of coordinated dump."
    } else if bundle_percentage > 50.0 {
        "HIGH RISK: Significant bundling detected. Potential for price manipulation."
    } else if bundle_percentage > 30.0 {
        "MODERATE RISK: Notable bundling present. Monitor closely for unusual activity."
    } else if bundle_percentage > 10.0 {
        "LOW-MODERATE RISK: Some bundling detected but within acceptable range."
    } else {
        "LOW RISK: Minimal bundling detected. Distribution appears organic."
    }
    .to_string()
}

/// Analyze a Solana token for bundling patterns and provide risk assessment.
/// This tool provides a simplified analysis based on TrenchBot data.
#[tool]
pub async fn analyze_token_bundles(
    context: &riglr_core::provider::ApplicationContext,
    token: String,
) -> crate::error::Result<BundleAnalysisResult> {
    debug!("Analyzing token bundles for: {}", token);

    let bundle_data = get_bundle_info(context, token.clone()).await?;

    let (sniper_count, regular_count, sniper_percentage, primary_category) =
        if let Some(bundles) = &bundle_data.bundles {
            calculate_wallet_categories(bundles)
        } else {
            (0, 0, 0.0, "regular".to_string())
        };

    let creator_risk = build_creator_risk_assessment(bundle_data.creator_analysis.as_ref());
    let warnings = collect_warnings(&bundle_data, sniper_percentage);
    let bundle_percentage = bundle_data.total_percentage_bundled.unwrap_or(0.0);
    let recommendation = generate_recommendation(bundle_percentage);

    Ok(BundleAnalysisResult {
        token,
        ticker: bundle_data.ticker,
        is_bundled: bundle_data.total_bundles.unwrap_or(0) > 0,
        bundle_percentage,
        bundle_count: bundle_data.total_bundles.unwrap_or(0),
        total_sol_spent: bundle_data.total_sol_spent.unwrap_or(0.0),
        holder_count: bundle_data.distributed_wallets.unwrap_or(0),
        creator_risk,
        wallet_categories: WalletCategoryBreakdown {
            snipers: sniper_count,
            regular: regular_count,
            sniper_percentage,
            primary_category,
        },
        warnings,
        recommendation,
    })
}

/// Check if a Solana token shows signs of bundling manipulation.
/// Returns a simple assessment of bundling risk.
#[tool]
pub async fn check_bundle_risk(
    context: &riglr_core::provider::ApplicationContext,
    token: String,
) -> crate::error::Result<BundleRiskCheck> {
    debug!("Checking bundle risk for token: {}", token);

    let bundle_data = get_bundle_info(context, token.clone()).await?;

    let bundle_percentage = bundle_data.total_percentage_bundled.unwrap_or(0.0);
    let bundle_count = bundle_data.total_bundles.unwrap_or(0);

    let risk_level = if bundle_percentage > 70.0 {
        "EXTREME"
    } else if bundle_percentage > 50.0 {
        "HIGH"
    } else if bundle_percentage > 30.0 {
        "MODERATE"
    } else if bundle_percentage > 10.0 {
        "LOW-MODERATE"
    } else {
        "LOW"
    };

    let is_high_risk = bundle_percentage > 50.0;

    let message = format!(
        "{} risk: {:.1}% of supply is bundled across {} bundles",
        risk_level, bundle_percentage, bundle_count
    );

    Ok(BundleRiskCheck {
        token,
        is_bundled: bundle_count > 0,
        bundle_percentage,
        bundle_count,
        risk_level: risk_level.to_string(),
        is_high_risk,
        message,
    })
}

/// Simple bundle risk check result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BundleRiskCheck {
    /// Token address
    pub token: String,
    /// Whether bundling is detected
    pub is_bundled: bool,
    /// Percentage of supply bundled
    pub bundle_percentage: f64,
    /// Number of bundles
    pub bundle_count: i32,
    /// Risk level assessment
    pub risk_level: String,
    /// Whether this represents high risk
    pub is_high_risk: bool,
    /// Summary message
    pub message: String,
}

/// Analyze the creator of a Solana token for historical rug pull activity.
/// Provides detailed creator risk assessment based on their track record.
#[tool]
pub async fn analyze_creator_risk(
    context: &riglr_core::provider::ApplicationContext,
    token: String,
) -> crate::error::Result<CreatorAnalysisResult> {
    debug!("Analyzing creator risk for token: {}", token);

    let bundle_data = get_bundle_info(context, token.clone()).await?;

    if let Some(creator) = bundle_data.creator_analysis {
        let mut red_flags = Vec::new();

        // Check for red flags
        if let Some(history) = &creator.history {
            if history.rug_count.unwrap_or(0) > 0 {
                red_flags.push(format!(
                    "Creator has {} previous rug pulls",
                    history.rug_count.unwrap_or(0)
                ));
            }

            if history.rug_percentage.unwrap_or(0.0) > 20.0 {
                red_flags.push(format!(
                    "{:.1}% of creator's tokens were rugs",
                    history.rug_percentage.unwrap_or(0.0)
                ));
            }

            if history.high_risk.unwrap_or(false) {
                red_flags.push("Creator flagged as high risk".to_string());
            }

            if history.recent_rugs.unwrap_or(0) > 0 {
                red_flags.push(format!(
                    "{} recent rug pulls detected",
                    history.recent_rugs.unwrap_or(0)
                ));
            }
        }

        if creator.holding_percentage.unwrap_or(0.0) > 30.0 {
            red_flags.push(format!(
                "Creator holds {:.1}% of supply",
                creator.holding_percentage.unwrap_or(0.0)
            ));
        }

        let risk_assessment = match &creator.risk_level {
            Some(RiskLevel::High) => "HIGH RISK: Creator has concerning history",
            Some(RiskLevel::Medium) => "MODERATE RISK: Some red flags in creator history",
            Some(RiskLevel::Low) => "LOW RISK: Creator appears legitimate",
            None => "UNKNOWN: Unable to assess creator risk",
        }
        .to_string();

        Ok(CreatorAnalysisResult {
            token,
            creator_address: creator.address,
            risk_level: creator.risk_level,
            current_holdings: creator.current_holdings,
            holding_percentage: creator.holding_percentage.unwrap_or(0.0),
            total_coins_created: creator
                .history
                .as_ref()
                .and_then(|h| h.total_coins_created)
                .unwrap_or(0),
            rug_count: creator
                .history
                .as_ref()
                .and_then(|h| h.rug_count)
                .unwrap_or(0),
            rug_percentage: creator
                .history
                .as_ref()
                .and_then(|h| h.rug_percentage)
                .unwrap_or(0.0),
            average_market_cap: creator.history.as_ref().and_then(|h| h.average_market_cap),
            previous_coins: creator
                .history
                .and_then(|h| h.previous_coins)
                .unwrap_or_default(),
            red_flags,
            risk_assessment,
        })
    } else {
        Ok(CreatorAnalysisResult {
            token,
            creator_address: None,
            risk_level: None,
            current_holdings: None,
            holding_percentage: 0.0,
            total_coins_created: 0,
            rug_count: 0,
            rug_percentage: 0.0,
            average_market_cap: None,
            previous_coins: Vec::new(),
            red_flags: vec!["No creator analysis available".to_string()],
            risk_assessment: "UNKNOWN: Creator information not available".to_string(),
        })
    }
}

/// Creator analysis result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatorAnalysisResult {
    /// Token being analyzed
    pub token: String,
    /// Creator wallet address
    pub creator_address: Option<String>,
    /// Risk level
    pub risk_level: Option<RiskLevel>,
    /// Creator's current token holdings
    pub current_holdings: Option<i64>,
    /// Percentage of supply held by creator
    pub holding_percentage: f64,
    /// Total number of coins created
    pub total_coins_created: i64,
    /// Number of rug pulls
    pub rug_count: i64,
    /// Percentage of tokens that were rugs
    pub rug_percentage: f64,
    /// Average market cap of created tokens
    pub average_market_cap: Option<i64>,
    /// List of previous coins
    pub previous_coins: Vec<PreviousCoin>,
    /// Red flags identified
    pub red_flags: Vec<String>,
    /// Risk assessment summary
    pub risk_assessment: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trenchbot_config_default() {
        let config = TrenchBotConfig::default();
        assert_eq!(config.base_url, "https://trench.bot/api");
        assert_eq!(config.rate_limit_per_minute, 60);
        assert_eq!(config.request_timeout, 30);
    }

    #[test]
    fn test_risk_level_serialization() {
        let risk = RiskLevel::High;
        let json = serde_json::to_string(&risk).unwrap();
        assert_eq!(json, "\"HIGH\"");

        let risk: RiskLevel = serde_json::from_str("\"MEDIUM\"").unwrap();
        assert!(matches!(risk, RiskLevel::Medium));
    }

    #[test]
    fn test_bundle_response_deserialization() {
        let json = r#"{
            "bonded": true,
            "ticker": "TEST",
            "total_bundles": 5,
            "total_percentage_bundled": 25.5
        }"#;

        let response: BundleResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.bonded, Some(true));
        assert_eq!(response.ticker, Some("TEST".to_string()));
        assert_eq!(response.total_bundles, Some(5));
        assert_eq!(response.total_percentage_bundled, Some(25.5));
    }

    #[test]
    fn test_creator_analysis_deserialization() {
        let json = r#"{
            "address": "SomeWalletAddress",
            "risk_level": "HIGH",
            "holding_percentage": 15.5,
            "history": {
                "rug_count": 3,
                "total_coins_created": 10,
                "rug_percentage": 30.0
            }
        }"#;

        let creator: CreatorAnalysis = serde_json::from_str(json).unwrap();
        assert_eq!(creator.address, Some("SomeWalletAddress".to_string()));
        assert!(matches!(creator.risk_level, Some(RiskLevel::High)));
        assert_eq!(creator.holding_percentage, Some(15.5));
        assert!(creator.history.is_some());

        let history = creator.history.unwrap();
        assert_eq!(history.rug_count, Some(3));
        assert_eq!(history.total_coins_created, Some(10));
        assert_eq!(history.rug_percentage, Some(30.0));
    }
}
