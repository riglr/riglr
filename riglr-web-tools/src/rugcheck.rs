//! RugCheck integration for Solana token security analysis and risk assessment
//!
//! This module provides tools for accessing RugCheck API to analyze Solana tokens
//! for potential rug pull risks, security issues, and insider trading patterns.

use crate::{client::WebClient, error::WebToolError};
use chrono::{DateTime, Utc};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Configuration for RugCheck API access
#[derive(Debug, Clone)]
pub struct RugCheckConfig {
    /// API base URL (default: https://api.rugcheck.xyz/v1)
    pub base_url: String,
    /// Rate limit requests per minute (default: 60)
    pub rate_limit_per_minute: u32,
    /// Timeout for API requests in seconds (default: 30)
    pub request_timeout: u64,
}

impl Default for RugCheckConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.rugcheck.xyz/v1".to_string(),
            rate_limit_per_minute: 60,
            request_timeout: 30,
        }
    }
}

/// Main token check report from RugCheck
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenCheck {
    /// Token mint address
    pub mint: String,
    /// Creator wallet address
    pub creator: Option<String>,
    /// When the token was first detected
    pub detected_at: Option<DateTime<Utc>>,
    /// Token events history
    pub events: Option<Vec<TokenEvent>>,
    /// File metadata from the token
    pub file_meta: Option<FileMetadata>,
    /// Freeze authority address
    pub freeze_authority: Option<String>,
    /// Graph-based insider detection report
    pub graph_insider_report: Option<GraphDetectedData>,
    /// Number of graph insiders detected
    pub graph_insiders_detected: Option<i32>,
    /// Insider networks information
    pub insider_networks: Option<Vec<InsiderNetwork>>,
    /// Known accounts mapping
    pub known_accounts: Option<HashMap<String, KnownAccount>>,
    /// Locker owners mapping
    pub locker_owners: Option<HashMap<String, bool>>,
    /// Token lockers information
    pub lockers: Option<HashMap<String, Locker>>,
    /// LP token lockers information
    pub lp_lockers: Option<HashMap<String, Locker>>,
    /// Market trading pairs
    pub markets: Option<Vec<Market>>,
    /// Mint authority address
    pub mint_authority: Option<String>,
    /// Current price in USD
    pub price: Option<f64>,
    /// Risk factors identified
    pub risks: Option<Vec<Risk>>,
    /// Whether the token has been rugged
    pub rugged: Option<bool>,
    /// Overall risk score (0-100, lower is better)
    pub score: Option<i32>,
    /// Normalized risk score
    pub score_normalised: Option<i32>,
    /// Token information
    pub token: Option<TokenInfo>,
    /// Token metadata
    pub token_meta: Option<TokenMetadata>,
    /// Token program ID
    pub token_program: Option<String>,
    /// Token type (e.g., "SPL", "Token22")
    pub token_type: Option<String>,
    /// Token extensions if any
    pub token_extensions: Option<String>,
    /// Creator's token balance
    pub creator_tokens: Option<String>,
    /// Creator's balance
    pub creator_balance: Option<i64>,
    /// Top token holders
    pub top_holders: Option<Vec<TokenHolder>>,
    /// Total number of holders
    pub total_holders: Option<i32>,
    /// Total LP providers
    pub total_lp_providers: Option<i32>,
    /// Total market liquidity in USD
    pub total_market_liquidity: Option<f64>,
    /// Transfer fee configuration
    pub transfer_fee: Option<TransferFee>,
    /// Token verification status
    pub verification: Option<VerifiedToken>,
}

/// Token event in history
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenEvent {
    /// When the event occurred
    pub created_at: DateTime<Utc>,
    /// Event type identifier
    pub event: i32,
    /// New value after the event
    pub new_value: Option<String>,
    /// Old value before the event
    pub old_value: Option<String>,
}

/// File metadata associated with the token
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileMetadata {
    /// Token description
    pub description: Option<String>,
    /// Token image URL
    pub image: Option<String>,
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
}

/// Graph-based insider detection data
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GraphDetectedData {
    /// Whether the token is blacklisted
    pub blacklisted: Option<bool>,
    /// Raw graph analysis data
    pub raw_graph_data: Option<Vec<Account>>,
    /// Detected receivers
    pub receivers: Option<Vec<InsiderDetectedData>>,
    /// Detected senders
    pub senders: Option<Vec<InsiderDetectedData>>,
    /// Total amount sent
    pub total_sent: Option<i64>,
}

/// Account information in graph analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Account {
    /// Account address
    pub address: String,
    /// Sent transfers
    pub sent: Option<Vec<Transfer>>,
}

/// Transfer information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Transfer {
    /// Transfer amount
    pub amount: i64,
    /// Token mint
    pub mint: String,
    /// Transfer receiver
    pub receiver: Option<Receiver>,
}

/// Transfer receiver
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Receiver {
    /// Receiver address
    pub address: String,
}

/// Insider detection data
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InsiderDetectedData {
    /// Insider address
    pub address: String,
    /// Amount held or transferred
    pub amount: i64,
}

/// Insider network information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InsiderNetwork {
    /// Network identifier
    pub id: String,
    /// Network size
    pub size: i32,
    /// Network type
    #[serde(rename = "type")]
    pub network_type: String,
    /// Token amount in network
    pub token_amount: i64,
    /// Active accounts in network
    pub active_accounts: i32,
}

/// Known account information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnownAccount {
    /// Account name
    pub name: String,
    /// Account type
    #[serde(rename = "type")]
    pub account_type: String,
}

/// Token locker information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Locker {
    /// Locker owner
    pub owner: Option<String>,
    /// Program ID
    pub program_id: Option<String>,
    /// Token account
    pub token_account: Option<String>,
    /// Locker type
    #[serde(rename = "type")]
    pub locker_type: Option<String>,
    /// Unlock date timestamp
    pub unlock_date: Option<i64>,
    /// Locker URI
    pub uri: Option<String>,
    /// USDC value locked
    pub usdc_locked: Option<f64>,
}

/// Market trading pair information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Market {
    /// Market address
    pub pubkey: String,
    /// Market type (e.g., "Raydium", "Orca")
    pub market_type: String,
    /// Mint A address
    pub mint_a: Option<String>,
    /// Mint A account
    pub mint_a_account: Option<String>,
    /// Mint B address
    pub mint_b: Option<String>,
    /// Mint B account
    pub mint_b_account: Option<String>,
    /// Liquidity A amount
    pub liquidity_a: Option<String>,
    /// Liquidity A account
    pub liquidity_a_account: Option<String>,
    /// Liquidity B amount
    pub liquidity_b: Option<String>,
    /// Liquidity B account
    pub liquidity_b_account: Option<String>,
    /// LP mint address
    pub mint_lp: Option<String>,
    /// LP mint account
    pub mint_lp_account: Option<String>,
    /// LP token information
    pub lp: Option<MarketLP>,
}

/// Market LP token information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MarketLP {
    /// Base token amount
    pub base: Option<f64>,
    /// Base token mint
    pub base_mint: Option<String>,
    /// Base token price
    pub base_price: Option<f64>,
    /// Base token USD value
    pub base_usd: Option<f64>,
    /// Current supply
    pub current_supply: Option<i64>,
    /// LP token holders
    pub holders: Option<Vec<TokenHolder>>,
    /// LP current supply
    pub lp_current_supply: Option<i64>,
    /// LP locked amount
    pub lp_locked: Option<i64>,
    /// LP locked percentage
    pub lp_locked_pct: Option<f64>,
    /// LP locked USD value
    pub lp_locked_usd: Option<f64>,
    /// LP max supply
    pub lp_max_supply: Option<i64>,
    /// LP mint address
    pub lp_mint: Option<String>,
    /// LP total supply
    pub lp_total_supply: Option<i64>,
    /// LP unlocked amount
    pub lp_unlocked: Option<i64>,
    /// Percentage of reserve
    pub pct_reserve: Option<f64>,
    /// Percentage of supply
    pub pct_supply: Option<f64>,
    /// Quote token amount
    pub quote: Option<f64>,
    /// Quote token mint
    pub quote_mint: Option<String>,
    /// Quote token price
    pub quote_price: Option<f64>,
    /// Quote token USD value
    pub quote_usd: Option<f64>,
    /// Reserve supply
    pub reserve_supply: Option<i64>,
    /// Token supply
    pub token_supply: Option<i64>,
    /// Total tokens unlocked
    pub total_tokens_unlocked: Option<i64>,
}

/// Risk factor information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Risk {
    /// Risk name
    pub name: String,
    /// Risk description
    pub description: String,
    /// Risk level (e.g., "High", "Medium", "Low")
    pub level: String,
    /// Risk score contribution
    pub score: i32,
    /// Risk value or details
    pub value: Option<String>,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenInfo {
    /// Mint authority
    pub mint_authority: Option<String>,
    /// Token supply
    pub supply: Option<i64>,
    /// Token decimals
    pub decimals: Option<i32>,
    /// Whether token is initialized
    pub is_initialized: Option<bool>,
    /// Freeze authority
    pub freeze_authority: Option<String>,
}

/// Token metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenMetadata {
    /// Whether metadata is mutable
    pub mutable: Option<bool>,
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Update authority
    pub update_authority: Option<String>,
    /// Metadata URI
    pub uri: Option<String>,
}

/// Token holder information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenHolder {
    /// Holder's wallet address
    pub address: String,
    /// Token amount held
    pub amount: i64,
    /// Token decimals
    pub decimals: Option<i32>,
    /// Whether holder is an insider
    pub insider: Option<bool>,
    /// Account owner
    pub owner: Option<String>,
    /// Percentage of total supply
    pub pct: Option<f64>,
    /// UI formatted amount
    pub ui_amount: Option<f64>,
    /// UI amount as string
    pub ui_amount_string: Option<String>,
}

/// Transfer fee configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransferFee {
    /// Fee authority
    pub authority: Option<String>,
    /// Maximum fee amount
    pub max_amount: Option<f64>,
    /// Fee percentage
    pub pct: Option<f64>,
}

/// Verified token information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerifiedToken {
    /// Token description
    pub description: Option<String>,
    /// Whether verified by Jupiter
    pub jup_verified: Option<bool>,
    /// Social and other links
    pub links: Option<Vec<VerifiedTokenLinks>>,
    /// Token mint
    pub mint: String,
    /// Token name
    pub name: String,
    /// Payer address
    pub payer: Option<String>,
    /// Token symbol
    pub symbol: String,
}

/// Verified token links
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerifiedTokenLinks {
    /// Link provider (e.g., "twitter", "website", "telegram")
    pub provider: String,
    /// Link value/URL
    pub value: String,
}

/// Risk analysis summary
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RiskAnalysis {
    /// Token being analyzed
    pub token_mint: String,
    /// Overall risk score (0-100)
    pub risk_score: i32,
    /// Risk level classification
    pub risk_level: RiskLevel,
    /// Whether token has been rugged
    pub is_rugged: bool,
    /// Critical risks found
    pub critical_risks: Vec<Risk>,
    /// High risks found
    pub high_risks: Vec<Risk>,
    /// Medium risks found
    pub medium_risks: Vec<Risk>,
    /// Low risks found
    pub low_risks: Vec<Risk>,
    /// Insider trading analysis
    pub insider_analysis: Option<InsiderAnalysis>,
    /// Liquidity analysis
    pub liquidity_analysis: Option<LiquidityAnalysis>,
    /// Holder concentration analysis
    pub concentration_analysis: Option<ConcentrationAnalysis>,
    /// Summary recommendation
    pub recommendation: String,
    /// Analysis timestamp
    pub analyzed_at: DateTime<Utc>,
}

/// Risk level classification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RiskLevel {
    /// Critical risk - Extreme danger, avoid at all costs
    #[serde(rename = "critical")]
    Critical,
    /// High risk - Significant concerns present
    #[serde(rename = "high")]
    High,
    /// Medium risk - Some concerns, proceed with caution
    #[serde(rename = "medium")]
    Medium,
    /// Low risk - Minor concerns only
    #[serde(rename = "low")]
    Low,
    /// Safe - Minimal to no risk detected
    #[serde(rename = "safe")]
    Safe,
}

/// Insider trading analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InsiderAnalysis {
    /// Number of insider networks detected
    pub networks_detected: i32,
    /// Total insider accounts
    pub insider_accounts: i32,
    /// Percentage of supply held by insiders
    pub insider_supply_pct: f64,
    /// Suspicious transfer patterns detected
    pub suspicious_patterns: Vec<String>,
}

/// Liquidity analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LiquidityAnalysis {
    /// Total liquidity in USD
    pub total_liquidity_usd: f64,
    /// Percentage of LP tokens locked
    pub lp_locked_pct: f64,
    /// Number of liquidity providers
    pub provider_count: i32,
    /// Liquidity concentration
    pub concentration: String,
}

/// Holder concentration analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConcentrationAnalysis {
    /// Top 10 holders percentage
    pub top_10_pct: f64,
    /// Top 25 holders percentage
    pub top_25_pct: f64,
    /// Number of holders
    pub holder_count: i32,
    /// Whale dominance level
    pub whale_dominance: String,
}

/// Get a comprehensive security report for a Solana token from RugCheck.
/// This tool analyzes tokens for rug pull risks, insider trading, and other security concerns.
#[tool]
pub async fn get_token_report(
    _context: &riglr_core::provider::ApplicationContext,
    mint: String,
) -> crate::error::Result<TokenCheck> {
    debug!("Fetching RugCheck report for token: {}", mint);

    let config = RugCheckConfig::default();
    let client = WebClient::default();

    let url = format!("{}/tokens/{}/report", config.base_url, mint);

    info!("Requesting RugCheck report from: {}", url);

    let response_text = client
        .get(&url)
        .await
        .map_err(|e| WebToolError::Network(format!("Failed to fetch RugCheck report: {}", e)))?;

    let report: TokenCheck = serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Failed to parse RugCheck response: {}", e)))?;

    info!(
        "Successfully fetched RugCheck report for {} - Score: {:?}, Rugged: {:?}",
        mint, report.score, report.rugged
    );

    Ok(report)
}

/// Categorize risks by severity level
fn categorize_risks(risks: &[Risk]) -> (Vec<Risk>, Vec<Risk>, Vec<Risk>, Vec<Risk>) {
    let mut critical_risks = Vec::new();
    let mut high_risks = Vec::new();
    let mut medium_risks = Vec::new();
    let mut low_risks = Vec::new();

    for risk in risks {
        match risk.level.to_lowercase().as_str() {
            "critical" | "danger" => critical_risks.push(risk.clone()),
            "high" | "warning" => high_risks.push(risk.clone()),
            "medium" | "caution" => medium_risks.push(risk.clone()),
            "low" | "info" => low_risks.push(risk.clone()),
            _ => medium_risks.push(risk.clone()),
        }
    }

    (critical_risks, high_risks, medium_risks, low_risks)
}

/// Determine risk level from score
fn calculate_risk_level(risk_score: i32) -> RiskLevel {
    match risk_score {
        0..=20 => RiskLevel::Safe,
        21..=40 => RiskLevel::Low,
        41..=60 => RiskLevel::Medium,
        61..=80 => RiskLevel::High,
        _ => RiskLevel::Critical,
    }
}

/// Analyze insider trading patterns
fn analyze_insider_activity(report: &TokenCheck) -> Option<InsiderAnalysis> {
    let networks = report.insider_networks.as_ref()?;
    let total_insiders = report.graph_insiders_detected.unwrap_or(0);
    let mut suspicious_patterns = Vec::new();

    if total_insiders > 10 {
        suspicious_patterns.push("High number of insider accounts detected".to_string());
    }

    if networks.len() > 3 {
        suspicious_patterns.push("Multiple insider networks identified".to_string());
    }

    Some(InsiderAnalysis {
        networks_detected: networks.len() as i32,
        insider_accounts: total_insiders,
        insider_supply_pct: 0.0, // Would need to calculate from holders
        suspicious_patterns,
    })
}

/// Analyze market liquidity characteristics
fn analyze_market_liquidity(report: &TokenCheck) -> Option<LiquidityAnalysis> {
    let liquidity = report.total_market_liquidity?;
    let lp_providers = report.total_lp_providers.unwrap_or(0);

    let concentration = if lp_providers < 10 {
        "High concentration"
    } else if lp_providers < 50 {
        "Moderate concentration"
    } else {
        "Well distributed"
    };

    Some(LiquidityAnalysis {
        total_liquidity_usd: liquidity,
        lp_locked_pct: 0.0, // Would need to calculate from markets
        provider_count: lp_providers,
        concentration: concentration.to_string(),
    })
}

/// Analyze token holder concentration patterns
fn analyze_holder_concentration(report: &TokenCheck) -> Option<ConcentrationAnalysis> {
    let holders = report.top_holders.as_ref()?;
    let total_holders = report.total_holders.unwrap_or(0);

    let top_10_supply: f64 = holders.iter().take(10).filter_map(|h| h.pct).sum();
    let top_25_supply: f64 = holders.iter().take(25).filter_map(|h| h.pct).sum();

    let whale_dominance = if top_10_supply > 50.0 {
        "Very High"
    } else if top_10_supply > 30.0 {
        "High"
    } else if top_10_supply > 15.0 {
        "Moderate"
    } else {
        "Low"
    };

    Some(ConcentrationAnalysis {
        top_10_pct: top_10_supply,
        top_25_pct: top_25_supply,
        holder_count: total_holders,
        whale_dominance: whale_dominance.to_string(),
    })
}

/// Generate risk-based recommendation
fn generate_recommendation(risk_level: &RiskLevel) -> String {
    match risk_level {
        RiskLevel::Critical => {
            "EXTREME CAUTION: This token shows critical risk factors. Avoid trading."
        }
        RiskLevel::High => {
            "HIGH RISK: Significant security concerns detected. Trade with extreme caution."
        }
        RiskLevel::Medium => {
            "MODERATE RISK: Some concerns present. Perform additional due diligence."
        }
        RiskLevel::Low => "LOW RISK: Token appears relatively safe but always DYOR.",
        RiskLevel::Safe => "MINIMAL RISK: Token shows good security characteristics.",
    }
    .to_string()
}

/// Analyze a Solana token's security risks and provide a comprehensive risk assessment.
/// This tool provides a simplified risk analysis based on RugCheck data.
#[tool]
pub async fn analyze_token_risks(
    context: &riglr_core::provider::ApplicationContext,
    mint: String,
) -> crate::error::Result<RiskAnalysis> {
    debug!("Analyzing token risks for: {}", mint);

    // Get the full report first
    let report = get_token_report(context, mint.clone()).await?;

    // Categorize risks by level
    let (critical_risks, high_risks, medium_risks, low_risks) = if let Some(risks) = &report.risks {
        categorize_risks(risks)
    } else {
        (Vec::new(), Vec::new(), Vec::new(), Vec::new())
    };

    // Determine overall risk level
    let risk_score = report.score.unwrap_or(0);
    let risk_level = calculate_risk_level(risk_score);

    // Analyze different aspects
    let insider_analysis = analyze_insider_activity(&report);
    let liquidity_analysis = analyze_market_liquidity(&report);
    let concentration_analysis = analyze_holder_concentration(&report);

    // Generate recommendation
    let recommendation = generate_recommendation(&risk_level);

    Ok(RiskAnalysis {
        token_mint: mint,
        risk_score,
        risk_level,
        is_rugged: report.rugged.unwrap_or(false),
        critical_risks,
        high_risks,
        medium_risks,
        low_risks,
        insider_analysis,
        liquidity_analysis,
        concentration_analysis,
        recommendation,
        analyzed_at: Utc::now(),
    })
}

/// Check if a Solana token has been rugged or shows signs of a rug pull.
/// Returns a simple boolean result with basic risk information.
#[tool]
pub async fn check_if_rugged(
    context: &riglr_core::provider::ApplicationContext,
    mint: String,
) -> crate::error::Result<RugCheckResult> {
    debug!("Checking rug status for token: {}", mint);

    let report = get_token_report(context, mint.clone()).await?;

    let is_rugged = report.rugged.unwrap_or(false);
    let risk_score = report.score.unwrap_or(0);
    let risk_count = report.risks.as_ref().map(|r| r.len()).unwrap_or(0);

    let status = if is_rugged {
        "RUGGED: This token has been identified as a rug pull"
    } else if risk_score > 80 {
        "EXTREME RISK: Very high likelihood of rug pull"
    } else if risk_score > 60 {
        "HIGH RISK: Significant rug pull indicators present"
    } else if risk_score > 40 {
        "MODERATE RISK: Some concerning factors detected"
    } else {
        "LOW RISK: No major rug pull indicators found"
    };

    Ok(RugCheckResult {
        mint,
        is_rugged,
        risk_score,
        risk_factors: risk_count as i32,
        status: status.to_string(),
        check_time: Utc::now(),
    })
}

/// Simple rug check result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RugCheckResult {
    /// Token mint address
    pub mint: String,
    /// Whether token has been rugged
    pub is_rugged: bool,
    /// Risk score (0-100)
    pub risk_score: i32,
    /// Number of risk factors detected
    pub risk_factors: i32,
    /// Status message
    pub status: String,
    /// Check timestamp
    pub check_time: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rugcheck_config_default() {
        let config = RugCheckConfig::default();
        assert_eq!(config.base_url, "https://api.rugcheck.xyz/v1");
        assert_eq!(config.rate_limit_per_minute, 60);
        assert_eq!(config.request_timeout, 30);
    }

    #[test]
    fn test_risk_level_serialization() {
        let risk = RiskLevel::Critical;
        let json = serde_json::to_string(&risk).unwrap();
        assert_eq!(json, "\"critical\"");

        let risk: RiskLevel = serde_json::from_str("\"high\"").unwrap();
        assert!(matches!(risk, RiskLevel::High));
    }

    #[test]
    fn test_token_check_deserialization() {
        let json = r#"{
            "mint": "So11111111111111111111111111111111111111112",
            "score": 25,
            "rugged": false
        }"#;

        let token: TokenCheck = serde_json::from_str(json).unwrap();
        assert_eq!(token.mint, "So11111111111111111111111111111111111111112");
        assert_eq!(token.score, Some(25));
        assert_eq!(token.rugged, Some(false));
    }
}
