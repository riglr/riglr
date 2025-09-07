//! PocketUniverse integration for Solana token rug pull detection
//!
//! This module provides tools for accessing PocketUniverse API to analyze Solana tokens
//! and pools for potential rug pull risks based on wallet history and trading patterns.

use crate::{client::WebClient, error::WebToolError};
use riglr_core::provider::ApplicationContext;
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{debug, info};

const POCKET_UNIVERSE_API_KEY_ENV: &str = "POCKET_UNIVERSE_API_KEY";

/// Configuration for PocketUniverse API access
#[derive(Debug, Clone)]
pub struct PocketUniverseConfig {
    /// API base URL (default: https://api.pocketuniverse.app)
    pub base_url: String,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Rate limit requests per minute (default: 60)
    pub rate_limit_per_minute: u32,
    /// Timeout for API requests in seconds (default: 30)
    pub request_timeout: u64,
}

impl Default for PocketUniverseConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.pocketuniverse.app".to_string(),
            api_key: env::var(POCKET_UNIVERSE_API_KEY_ENV).ok(),
            rate_limit_per_minute: 60,
            request_timeout: 30,
        }
    }
}

/// Helper function to get PocketUniverse API key from ApplicationContext
fn get_api_key_from_context(context: &ApplicationContext) -> Result<String, WebToolError> {
    context.config.providers.pocket_universe_api_key
        .clone()
        .ok_or_else(|| WebToolError::Config(
            "PocketUniverse API key not configured. Set POCKET_UNIVERSE_API_KEY in your environment.".to_string()
        ))
}

/// Main rug check API response from PocketUniverse
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RugApiResponse {
    /// Token has not been processed yet
    #[serde(rename = "not_processed")]
    NotProcessed {
        /// Explanation message
        message: String,
    },
    /// Token has been processed and analyzed
    #[serde(rename = "processed")]
    Processed {
        /// Human-readable analysis summary
        message: String,
        /// Whether the token is identified as a scam
        is_scam: bool,
        /// Percentage of volume from past rug pullers (0.0 to 1.0)
        rug_percent: f64,
        /// Percentage of volume from fresh wallets (0.0 to 1.0)
        fresh_percent: f64,
    },
}

/// Error detail structure
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ErrorDetail {
    /// Error type
    #[serde(rename = "type")]
    pub error_type: String,
    /// Error message
    pub message: String,
}

/// Error response structure
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ErrorResponse {
    /// Error details
    pub error: ErrorDetail,
}

/// Simplified rug check result for easier consumption
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RugCheckResult {
    /// Token/pool address that was checked
    pub address: String,
    /// Whether the analysis is available
    pub is_processed: bool,
    /// Whether the token is identified as a scam (None if not processed)
    pub is_scam: Option<bool>,
    /// Percentage of volume from rug pullers (0-100, None if not processed)
    pub rug_percentage: Option<f64>,
    /// Percentage of volume from fresh wallets (0-100, None if not processed)
    pub fresh_percentage: Option<f64>,
    /// Risk level classification
    pub risk_level: RiskLevel,
    /// Human-readable message
    pub message: String,
    /// Summary recommendation
    pub recommendation: String,
}

/// Risk level classification for tokens
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RiskLevel {
    /// Not enough data to assess risk
    #[serde(rename = "unknown")]
    Unknown,
    /// Low risk - Minimal rug puller activity
    #[serde(rename = "low")]
    Low,
    /// Medium risk - Some concerning patterns
    #[serde(rename = "medium")]
    Medium,
    /// High risk - Significant rug puller activity
    #[serde(rename = "high")]
    High,
    /// Extreme risk - Confirmed scam or very high rug puller percentage
    #[serde(rename = "extreme")]
    Extreme,
}

/// Detailed analysis result with additional insights
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DetailedRugAnalysis {
    /// Token/pool address
    pub address: String,
    /// Processing status
    pub status: ProcessingStatus,
    /// Scam detection result
    pub scam_detection: Option<ScamDetection>,
    /// Volume analysis
    pub volume_analysis: Option<VolumeAnalysis>,
    /// Risk assessment
    pub risk_assessment: RiskAssessment,
    /// Key warnings
    pub warnings: Vec<String>,
    /// Actionable recommendation
    pub recommendation: String,
}

/// Processing status of the token
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ProcessingStatus {
    /// Token has been fully processed
    #[serde(rename = "processed")]
    Processed,
    /// Token not yet processed (insufficient data)
    #[serde(rename = "not_processed")]
    NotProcessed,
    /// Error occurred during processing
    #[serde(rename = "error")]
    Error(String),
}

/// Scam detection results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScamDetection {
    /// Whether identified as scam
    pub is_scam: bool,
    /// Confidence level (0-100)
    pub confidence: f64,
    /// Reason for classification
    pub reason: String,
}

/// Volume analysis breakdown
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VolumeAnalysis {
    /// Percentage from rug pullers (0-100)
    pub rug_puller_percentage: f64,
    /// Percentage from fresh wallets (0-100)
    pub fresh_wallet_percentage: f64,
    /// Percentage from regular traders (0-100)
    pub regular_trader_percentage: f64,
    /// Volume concentration assessment
    pub concentration: VolumeConcentration,
}

/// Volume concentration level
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum VolumeConcentration {
    /// Volume well distributed
    #[serde(rename = "distributed")]
    Distributed,
    /// Moderate concentration
    #[serde(rename = "moderate")]
    Moderate,
    /// High concentration in few wallets
    #[serde(rename = "concentrated")]
    Concentrated,
    /// Extreme concentration (potential manipulation)
    #[serde(rename = "extreme")]
    Extreme,
}

/// Risk assessment summary
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RiskAssessment {
    /// Overall risk level
    pub level: RiskLevel,
    /// Risk score (0-100)
    pub score: f64,
    /// Main risk factors
    pub factors: Vec<String>,
    /// Suggested action
    pub action: String,
}

/// Check a Solana token or pool for rug pull risk using PocketUniverse.
/// This is the raw API call that returns the direct response.
#[tool]
pub async fn check_rug_pull_raw(
    context: &ApplicationContext,
    address: String,
) -> crate::error::Result<RugApiResponse> {
    debug!("Checking rug pull risk for address: {}", address);

    let config = PocketUniverseConfig::default();
    let client = WebClient::default();

    // Get API key from ApplicationContext
    let api_key = get_api_key_from_context(context)?;

    let url = format!(
        "{}/rug_check/{}?address={}",
        config.base_url, api_key, address
    );

    info!("Requesting rug check from PocketUniverse for: {}", address);

    let response_text = client
        .get(&url)
        .await
        .map_err(|e| WebToolError::Network(format!("Failed to fetch rug check: {}", e)))?;

    let response: RugApiResponse = serde_json::from_str(&response_text).map_err(|e| {
        WebToolError::Parsing(format!("Failed to parse PocketUniverse response: {}", e))
    })?;

    match &response {
        RugApiResponse::NotProcessed { message } => {
            info!("Token {} not processed: {}", address, message);
        }
        RugApiResponse::Processed {
            is_scam,
            rug_percent,
            ..
        } => {
            info!(
                "Token {} analyzed - Scam: {}, Rug percentage: {:.1}%",
                address,
                is_scam,
                rug_percent * 100.0
            );
        }
    }

    Ok(response)
}

/// Check a Solana token or pool for rug pull risk with simplified results.
/// Provides an easy-to-use risk assessment based on PocketUniverse data.
#[tool]
pub async fn check_rug_pull(
    context: &ApplicationContext,
    address: String,
) -> crate::error::Result<RugCheckResult> {
    debug!("Performing simplified rug check for: {}", address);

    let raw_response = check_rug_pull_raw(context, address.clone()).await?;

    let (is_processed, is_scam, rug_percentage, fresh_percentage, message) = match raw_response {
        RugApiResponse::NotProcessed { message } => (false, None, None, None, message),
        RugApiResponse::Processed {
            message,
            is_scam,
            rug_percent,
            fresh_percent,
        } => (
            true,
            Some(is_scam),
            Some(rug_percent * 100.0),
            Some(fresh_percent * 100.0),
            message,
        ),
    };

    // Determine risk level
    let risk_level = if !is_processed {
        RiskLevel::Unknown
    } else if is_scam.unwrap_or(false) {
        RiskLevel::Extreme
    } else if let Some(rug_pct) = rug_percentage {
        if rug_pct > 70.0 {
            RiskLevel::Extreme
        } else if rug_pct > 50.0 {
            RiskLevel::High
        } else if rug_pct > 25.0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    } else {
        RiskLevel::Unknown
    };

    // Generate recommendation
    let recommendation = match risk_level {
        RiskLevel::Unknown => {
            "Unable to assess risk. Token may be too new or have insufficient trading data."
                .to_string()
        }
        RiskLevel::Low => {
            "Low risk detected. Token appears relatively safe but always DYOR.".to_string()
        }
        RiskLevel::Medium => {
            "Moderate risk. Some concerning patterns detected. Proceed with caution.".to_string()
        }
        RiskLevel::High => {
            "HIGH RISK: Significant rug puller activity detected. Strong caution advised."
                .to_string()
        }
        RiskLevel::Extreme => {
            "EXTREME RISK: Token identified as scam or has very high rug puller percentage. AVOID."
                .to_string()
        }
    };

    Ok(RugCheckResult {
        address,
        is_processed,
        is_scam,
        rug_percentage,
        fresh_percentage,
        risk_level,
        message,
        recommendation,
    })
}

/// Helper function to build scam detection from processed data
fn build_scam_detection(is_scam: bool, rug_percent: f64, message: String) -> ScamDetection {
    ScamDetection {
        is_scam,
        confidence: if is_scam {
            rug_percent * 100.0
        } else {
            (1.0 - rug_percent) * 100.0
        },
        reason: message,
    }
}

/// Helper function to determine volume concentration level
fn determine_volume_concentration(rug_percent: f64) -> VolumeConcentration {
    if rug_percent > 0.7 {
        VolumeConcentration::Extreme
    } else if rug_percent > 0.5 {
        VolumeConcentration::Concentrated
    } else if rug_percent > 0.3 {
        VolumeConcentration::Moderate
    } else {
        VolumeConcentration::Distributed
    }
}

/// Helper function to build volume analysis from processed data
fn build_volume_analysis(rug_percent: f64, fresh_percent: f64) -> VolumeAnalysis {
    let regular_percent = 1.0 - rug_percent - fresh_percent;
    let regular_percentage = if regular_percent > 0.0 {
        regular_percent * 100.0
    } else {
        0.0
    };

    VolumeAnalysis {
        rug_puller_percentage: rug_percent * 100.0,
        fresh_wallet_percentage: fresh_percent * 100.0,
        regular_trader_percentage: regular_percentage,
        concentration: determine_volume_concentration(rug_percent),
    }
}

/// Helper function to add warnings based on volume analysis
fn add_volume_warnings(
    warnings: &mut Vec<String>,
    is_scam: bool,
    rug_percent: f64,
    fresh_percent: f64,
    regular_percentage: f64,
) {
    if is_scam {
        warnings.push("Token identified as SCAM by PocketUniverse".to_string());
    }

    if rug_percent > 0.7 {
        warnings.push(format!(
            "{:.1}% of volume from known rug pullers",
            rug_percent * 100.0
        ));
    } else if rug_percent > 0.5 {
        warnings.push(format!(
            "High rug puller activity: {:.1}%",
            rug_percent * 100.0
        ));
    }

    if fresh_percent > 0.5 {
        warnings.push(format!(
            "High fresh wallet activity: {:.1}%",
            fresh_percent * 100.0
        ));
    }

    if regular_percentage < 20.0 {
        warnings.push(format!(
            "Low regular trader participation: {:.1}%",
            regular_percentage
        ));
    }
}

/// Helper function to build risk assessment from scam detection and volume analysis
fn build_risk_assessment(
    scam_detection: &Option<ScamDetection>,
    volume_analysis: &Option<VolumeAnalysis>,
    status: &ProcessingStatus,
) -> RiskAssessment {
    let mut risk_factors = Vec::new();
    let mut risk_score: f64 = 0.0;

    // Check scam detection
    if let Some(scam) = scam_detection {
        if scam.is_scam {
            risk_factors.push("Identified as scam".to_string());
            risk_score = 100.0;
        }
    }

    // Analyze volume patterns
    if let Some(vol) = volume_analysis {
        if vol.rug_puller_percentage > 50.0 {
            risk_factors.push("Majority volume from rug pullers".to_string());
            risk_score = risk_score.max(80.0 + (vol.rug_puller_percentage - 50.0) * 0.4);
        } else if vol.rug_puller_percentage > 25.0 {
            risk_factors.push("Significant rug puller presence".to_string());
            risk_score = risk_score.max(40.0 + (vol.rug_puller_percentage - 25.0) * 1.6);
        }

        if vol.fresh_wallet_percentage > 40.0 {
            risk_factors.push("High fresh wallet activity".to_string());
            risk_score = risk_score.max(risk_score + 10.0);
        }

        if matches!(
            vol.concentration,
            VolumeConcentration::Extreme | VolumeConcentration::Concentrated
        ) {
            risk_factors.push("Volume highly concentrated".to_string());
            risk_score = risk_score.max(risk_score + 15.0);
        }
    }

    if risk_factors.is_empty() && matches!(status, ProcessingStatus::Processed) {
        risk_factors.push("No major risk factors identified".to_string());
    }

    // Determine risk level
    let risk_level = if risk_score >= 80.0 {
        RiskLevel::Extreme
    } else if risk_score >= 60.0 {
        RiskLevel::High
    } else if risk_score >= 30.0 {
        RiskLevel::Medium
    } else if matches!(status, ProcessingStatus::Processed) {
        RiskLevel::Low
    } else {
        RiskLevel::Unknown
    };

    let action = match risk_level {
        RiskLevel::Unknown => "Wait for more trading data before investing".to_string(),
        RiskLevel::Low => "Can consider investment with standard precautions".to_string(),
        RiskLevel::Medium => {
            "Exercise caution, invest only what you can afford to lose".to_string()
        }
        RiskLevel::High => "Avoid investment, high risk of loss".to_string(),
        RiskLevel::Extreme => "DO NOT INVEST - Extreme risk or confirmed scam".to_string(),
    };

    RiskAssessment {
        level: risk_level,
        score: risk_score,
        factors: risk_factors,
        action,
    }
}

/// Helper function to generate recommendation based on status and risk level
fn generate_recommendation(status: &ProcessingStatus, risk_level: &RiskLevel) -> String {
    match (status, risk_level) {
        (ProcessingStatus::NotProcessed, _) => {
            "Token has insufficient data for analysis. Wait for more trading activity before making investment decisions.".to_string()
        }
        (_, RiskLevel::Extreme) => {
            "EXTREME DANGER: This token shows clear signs of being a scam or rug pull. Do not invest under any circumstances.".to_string()
        }
        (_, RiskLevel::High) => {
            "HIGH RISK: Significant red flags detected. This token has high probability of being a rug pull. Strongly recommend avoiding.".to_string()
        }
        (_, RiskLevel::Medium) => {
            "MODERATE RISK: Some concerning patterns detected. If you choose to invest, use extreme caution and only risk what you can afford to lose.".to_string()
        }
        (_, RiskLevel::Low) => {
            "LOW RISK: Token appears relatively safe based on wallet analysis, but always do your own research and invest responsibly.".to_string()
        }
        (_, RiskLevel::Unknown) => {
            "UNKNOWN RISK: Unable to determine risk level. More data needed for proper assessment.".to_string()
        }
    }
}

/// Perform detailed analysis of a token's rug pull risk with comprehensive insights.
/// Provides volume breakdown, risk factors, and actionable recommendations.
#[tool]
pub async fn analyze_rug_risk(
    context: &ApplicationContext,
    address: String,
) -> crate::error::Result<DetailedRugAnalysis> {
    debug!("Performing detailed rug analysis for: {}", address);

    let raw_response = check_rug_pull_raw(context, address.clone()).await?;
    let mut warnings = Vec::new();

    let (status, scam_detection, volume_analysis) = match raw_response {
        RugApiResponse::NotProcessed { message } => {
            warnings.push(message);
            (ProcessingStatus::NotProcessed, None, None)
        }
        RugApiResponse::Processed {
            message,
            is_scam,
            rug_percent,
            fresh_percent,
        } => {
            let scam_detection = Some(build_scam_detection(is_scam, rug_percent, message));
            let volume_analysis = Some(build_volume_analysis(rug_percent, fresh_percent));

            // Add warnings based on thresholds
            let regular_percentage = (1.0 - rug_percent - fresh_percent).max(0.0) * 100.0;
            add_volume_warnings(
                &mut warnings,
                is_scam,
                rug_percent,
                fresh_percent,
                regular_percentage,
            );

            (ProcessingStatus::Processed, scam_detection, volume_analysis)
        }
    };

    let risk_assessment = build_risk_assessment(&scam_detection, &volume_analysis, &status);
    let recommendation = generate_recommendation(&status, &risk_assessment.level);

    Ok(DetailedRugAnalysis {
        address,
        status,
        scam_detection,
        volume_analysis,
        risk_assessment,
        warnings,
        recommendation,
    })
}

/// Quick safety check for a Solana token - returns a simple safe/unsafe verdict.
/// Best for quick filtering of tokens before deeper analysis.
#[tool]
pub async fn is_token_safe(
    context: &ApplicationContext,
    address: String,
    risk_tolerance: Option<RiskTolerance>,
) -> crate::error::Result<SafetyCheck> {
    debug!("Performing quick safety check for: {}", address);

    let risk_tolerance = risk_tolerance.unwrap_or(RiskTolerance::Low);
    let result = check_rug_pull(context, address.clone()).await?;

    let is_safe = match (&result.risk_level, &risk_tolerance) {
        (RiskLevel::Unknown, _) => false, // Unknown is unsafe by default
        (RiskLevel::Low, _) => true,
        (RiskLevel::Medium, RiskTolerance::Medium | RiskTolerance::High) => true,
        (RiskLevel::Medium, RiskTolerance::Low) => false,
        (RiskLevel::High, RiskTolerance::High) => true,
        (RiskLevel::High, _) => false,
        (RiskLevel::Extreme, _) => false, // Extreme is never safe
    };

    let safety_score = match result.risk_level {
        RiskLevel::Unknown => 0.0,
        RiskLevel::Low => 80.0 - result.rug_percentage.unwrap_or(0.0) * 0.8,
        RiskLevel::Medium => 60.0 - result.rug_percentage.unwrap_or(25.0) * 0.6,
        RiskLevel::High => 30.0 - result.rug_percentage.unwrap_or(50.0) * 0.3,
        RiskLevel::Extreme => 0.0,
    };

    let verdict = if !result.is_processed {
        "UNVERIFIED: Insufficient data"
    } else if result.is_scam.unwrap_or(false) {
        "UNSAFE: Confirmed scam"
    } else if is_safe {
        "SAFE: Acceptable risk level"
    } else {
        "UNSAFE: Risk exceeds tolerance"
    };

    Ok(SafetyCheck {
        address,
        is_safe,
        risk_level: result.risk_level,
        safety_score,
        verdict: verdict.to_string(),
        details: result.message,
    })
}

/// Risk tolerance levels for safety checks
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RiskTolerance {
    /// Only accept low risk tokens
    #[serde(rename = "low")]
    Low,
    /// Accept low and medium risk tokens
    #[serde(rename = "medium")]
    Medium,
    /// Accept all except extreme risk tokens
    #[serde(rename = "high")]
    High,
}

/// Simple safety check result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SafetyCheck {
    /// Token address
    pub address: String,
    /// Whether token is considered safe given risk tolerance
    pub is_safe: bool,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Safety score (0-100, higher is safer)
    pub safety_score: f64,
    /// Simple verdict
    pub verdict: String,
    /// Additional details
    pub details: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pocketuniverse_config_default() {
        let config = PocketUniverseConfig::default();
        assert_eq!(config.base_url, "https://api.pocketuniverse.app");
        assert_eq!(config.rate_limit_per_minute, 60);
        assert_eq!(config.request_timeout, 30);
    }

    #[test]
    fn test_risk_level_serialization() {
        let risk = RiskLevel::High;
        let json = serde_json::to_string(&risk).unwrap();
        assert_eq!(json, "\"high\"");

        let risk: RiskLevel = serde_json::from_str("\"extreme\"").unwrap();
        assert!(matches!(risk, RiskLevel::Extreme));
    }

    #[test]
    fn test_rug_api_response_not_processed() {
        let json = r#"{
            "status": "not_processed",
            "message": "Token has not been processed"
        }"#;

        let response: RugApiResponse = serde_json::from_str(json).unwrap();
        assert!(matches!(response, RugApiResponse::NotProcessed { .. }));

        if let RugApiResponse::NotProcessed { message } = response {
            assert_eq!(message, "Token has not been processed");
        }
    }

    #[test]
    fn test_rug_api_response_processed() {
        let json = r#"{
            "status": "processed",
            "message": "88% of volume is from past rug pullers",
            "is_scam": true,
            "rug_percent": 0.88,
            "fresh_percent": 0.11
        }"#;

        let response: RugApiResponse = serde_json::from_str(json).unwrap();
        assert!(matches!(response, RugApiResponse::Processed { .. }));

        if let RugApiResponse::Processed {
            message,
            is_scam,
            rug_percent,
            fresh_percent,
        } = response
        {
            assert_eq!(message, "88% of volume is from past rug pullers");
            assert!(is_scam);
            assert!((rug_percent - 0.88).abs() < 0.001);
            assert!((fresh_percent - 0.11).abs() < 0.001);
        }
    }

    #[test]
    fn test_risk_tolerance_serialization() {
        let tolerance = RiskTolerance::Medium;
        let json = serde_json::to_string(&tolerance).unwrap();
        assert_eq!(json, "\"medium\"");

        let tolerance: RiskTolerance = serde_json::from_str("\"high\"").unwrap();
        assert!(matches!(tolerance, RiskTolerance::High));
    }

    #[test]
    fn test_volume_concentration_serialization() {
        let concentration = VolumeConcentration::Extreme;
        let json = serde_json::to_string(&concentration).unwrap();
        assert_eq!(json, "\"extreme\"");

        let concentration: VolumeConcentration = serde_json::from_str("\"distributed\"").unwrap();
        assert!(matches!(concentration, VolumeConcentration::Distributed));
    }

    #[test]
    fn test_processing_status_serialization() {
        let status = ProcessingStatus::Processed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"processed\"");

        let status = ProcessingStatus::Error("test error".to_string());
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("test error"));
    }
}
