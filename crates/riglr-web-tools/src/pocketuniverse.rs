//! `PocketUniverse` integration for Solana token rug pull detection
//!
//! This module provides tools for accessing `PocketUniverse` API to analyze Solana tokens
//! and pools for potential rug pull risks based on wallet history and trading patterns.

use crate::{client::WebClient, error::WebToolError};
use riglr_core::provider::ApplicationContext;
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::info;

const POCKET_UNIVERSE_API_KEY_ENV: &str = "POCKET_UNIVERSE_API_KEY";

/// Configuration for `PocketUniverse` API access
#[derive(Debug, Clone)]
pub struct PocketUniverseConfig {
    /// API key for authentication
    pub api_key: Option<String>,
    /// API base URL (default: <https://api.pocketuniverse.app>)
    pub base_url: String,
    /// Rate limit requests per minute (default: 60)
    pub rate_limit_per_minute: u32,
    /// Timeout for API requests in seconds (default: 30)
    pub request_timeout: u64,
}

impl Default for PocketUniverseConfig {
    fn default() -> Self {
        Self {
            api_key: env::var(POCKET_UNIVERSE_API_KEY_ENV).ok(),
            base_url: "https://api.pocketuniverse.app".to_string(),
            rate_limit_per_minute: 60,
            request_timeout: 30,
        }
    }
}

/// Main rug check API response from `PocketUniverse`
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

/// Helper function to get `PocketUniverse` API key from `ApplicationContext`
fn get_api_key_from_context(context: &ApplicationContext) -> Result<String, WebToolError> {
    context.config.providers.pocket_universe_api_key
        .clone()
        .ok_or_else(|| WebToolError::Config(
            "PocketUniverse API key not configured. Set POCKET_UNIVERSE_API_KEY in your environment.".to_string()
        ))
}

/// Simplified rug check result for easier consumption
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RugCheckResult {
    /// Token/pool address that was checked
    pub address: String,
    /// Percentage of volume from fresh wallets (0-100, None if not processed)
    pub fresh_percentage: Option<f64>,
    /// Whether the analysis is available
    pub is_processed: bool,
    /// Whether the token is identified as a scam (None if not processed)
    pub is_scam: Option<bool>,
    /// Human-readable message
    pub message: String,
    /// Summary recommendation
    pub recommendation: String,
    /// Risk level classification
    pub risk_level: RiskLevel,
    /// Percentage of volume from rug pullers (0-100, None if not processed)
    pub rug_percentage: Option<f64>,
}

/// Risk level classification for tokens
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RiskLevel {
    /// Extreme risk - Confirmed scam or very high rug puller percentage
    #[serde(rename = "extreme")]
    Extreme,
    /// High risk - Significant rug puller activity
    #[serde(rename = "high")]
    High,
    /// Low risk - Minimal rug puller activity
    #[serde(rename = "low")]
    Low,
    /// Medium risk - Some concerning patterns
    #[serde(rename = "medium")]
    Medium,
    /// Not enough data to assess risk
    #[serde(rename = "unknown")]
    Unknown,
}

/// Detailed analysis result with additional insights
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DetailedRugAnalysis {
    /// Token/pool address
    pub address: String,
    /// Actionable recommendation
    pub recommendation: String,
    /// Risk assessment
    pub risk_assessment: RiskAssessment,
    /// Scam detection result
    pub scam_detection: Option<ScamDetection>,
    /// Processing status
    pub status: ProcessingStatus,
    /// Volume analysis
    pub volume_analysis: Option<VolumeAnalysis>,
    /// Key warnings
    pub warnings: Vec<String>,
}

/// Processing status of the token
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ProcessingStatus {
    /// Error occurred during processing
    #[serde(rename = "error")]
    Error(String),
    /// Token not yet processed (insufficient data)
    #[serde(rename = "not_processed")]
    NotProcessed,
    /// Token has been fully processed
    #[serde(rename = "processed")]
    Processed,
}

/// Scam detection results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScamDetection {
    /// Confidence level (0-100)
    pub confidence: f64,
    /// Whether identified as scam
    pub is_scam: bool,
    /// Reason for classification
    pub reason: String,
}

/// Volume analysis breakdown
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VolumeAnalysis {
    /// Volume concentration assessment
    pub concentration: VolumeConcentration,
    /// Percentage from fresh wallets (0-100)
    pub fresh_wallet_percentage: f64,
    /// Percentage from regular traders (0-100)
    pub regular_trader_percentage: f64,
    /// Percentage from rug pullers (0-100)
    pub rug_puller_percentage: f64,
}

/// Volume concentration level
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum VolumeConcentration {
    /// High concentration in few wallets
    #[serde(rename = "concentrated")]
    Concentrated,
    /// Volume well distributed
    #[serde(rename = "distributed")]
    Distributed,
    /// Extreme concentration (potential manipulation)
    #[serde(rename = "extreme")]
    Extreme,
    /// Moderate concentration
    #[serde(rename = "moderate")]
    Moderate,
}

/// Risk assessment summary
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RiskAssessment {
    /// Suggested action
    pub action: String,
    /// Main risk factors
    pub factors: Vec<String>,
    /// Overall risk level
    pub level: RiskLevel,
    /// Risk score (0-100)
    pub score: f64,
}

/// Helper function to create the `PocketUniverse` API URL
fn create_api_url(base_url: &str, api_key: &str, address: &str) -> String {
    format!("{base_url}/rug_check/{api_key}?address={address}")
}

/// Helper function to fetch and parse the rug check response
async fn fetch_rug_check_response(
    client: &WebClient,
    url: &str,
) -> Result<RugApiResponse, WebToolError> {
    let response_text = client
        .get(url)
        .await
        .map_err(|e| WebToolError::Network(format!("Failed to fetch rug check: {e}")))?;

    serde_json::from_str(&response_text)
        .map_err(|e| WebToolError::Parsing(format!("Failed to parse PocketUniverse response: {e}")))
}

/// Helper function to log the rug check response
fn log_rug_check_response(address: &str, response: &RugApiResponse) {
    match *response {
        RugApiResponse::NotProcessed { ref message } => {
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
}

/// Check a Solana token or pool for rug pull risk using `PocketUniverse`.
/// This is the raw API call that returns the direct response.
///
/// # Errors
///
/// Returns `WebToolError` if:
/// - API key is not configured
/// - Network request fails
/// - Response parsing fails
#[tool]
pub async fn check_rug_pull_raw(
    context: &ApplicationContext,
    address: String,
) -> Result<RugApiResponse, WebToolError> {
    let config = PocketUniverseConfig::default();
    let client = WebClient::default();

    // Get API key from ApplicationContext
    let api_key = get_api_key_from_context(context)?;

    let url = create_api_url(&config.base_url, &api_key, &address);

    info!("Requesting rug check from PocketUniverse for: {}", address);

    let response = fetch_rug_check_response(&client, &url).await?;

    log_rug_check_response(&address, &response);

    Ok(response)
}

/// Check a Solana token or pool for rug pull risk with simplified results.
/// Provides an easy-to-use risk assessment based on `PocketUniverse` data.
///
/// # Errors
///
/// Returns `WebToolError` if:
/// - API key is not configured
/// - Network request fails
/// - Response parsing fails
#[tool]
pub async fn check_rug_pull(
    context: &ApplicationContext,
    address: String,
) -> Result<RugCheckResult, WebToolError> {
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
        fresh_percentage,
        is_processed,
        is_scam,
        message,
        recommendation,
        risk_level,
        rug_percentage,
    })
}

/// Helper function to build scam detection from processed data
fn build_scam_detection(is_scam: bool, rug_percent: f64, message: String) -> ScamDetection {
    ScamDetection {
        confidence: if is_scam {
            rug_percent * 100.0
        } else {
            (1.0 - rug_percent) * 100.0
        },
        is_scam,
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
        concentration: determine_volume_concentration(rug_percent),
        fresh_wallet_percentage: fresh_percent * 100.0,
        regular_trader_percentage: regular_percentage,
        rug_puller_percentage: rug_percent * 100.0,
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
            "Low regular trader participation: {regular_percentage:.1}%"
        ));
    }
}

/// Helper function to analyze scam detection and update risk factors and score
fn analyze_scam_detection(
    scam_detection: Option<&ScamDetection>,
    risk_factors: &mut Vec<String>,
    risk_score: &mut f64,
) {
    if let Some(scam) = scam_detection {
        if scam.is_scam {
            risk_factors.push("Identified as scam".to_string());
            *risk_score = 100.0;
        }
    }
}

/// Helper function to analyze volume patterns and update risk factors and score
fn analyze_volume_patterns(
    volume_analysis: Option<&VolumeAnalysis>,
    risk_factors: &mut Vec<String>,
    risk_score: &mut f64,
) {
    if let Some(vol) = volume_analysis {
        if vol.rug_puller_percentage > 50.0 {
            risk_factors.push("Majority volume from rug pullers".to_string());
            *risk_score = risk_score.max((vol.rug_puller_percentage - 50.0).mul_add(0.4, 80.0));
        } else if vol.rug_puller_percentage > 25.0 {
            risk_factors.push("Significant rug puller presence".to_string());
            *risk_score = risk_score.max((vol.rug_puller_percentage - 25.0).mul_add(1.6, 40.0));
        }

        if vol.fresh_wallet_percentage > 40.0 {
            risk_factors.push("High fresh wallet activity".to_string());
            *risk_score = risk_score.max(*risk_score + 10.0);
        }

        if matches!(
            vol.concentration,
            VolumeConcentration::Extreme | VolumeConcentration::Concentrated
        ) {
            risk_factors.push("Volume highly concentrated".to_string());
            *risk_score = risk_score.max(*risk_score + 15.0);
        }
    }
}

/// Helper function to determine risk level based on score and status
fn determine_risk_level(risk_score: f64, status: &ProcessingStatus) -> RiskLevel {
    if risk_score >= 80.0 {
        RiskLevel::Extreme
    } else if risk_score >= 60.0 {
        RiskLevel::High
    } else if risk_score >= 30.0 {
        RiskLevel::Medium
    } else if matches!(*status, ProcessingStatus::Processed) {
        RiskLevel::Low
    } else {
        RiskLevel::Unknown
    }
}

/// Helper function to generate action recommendation based on risk level
fn generate_risk_action(risk_level: &RiskLevel) -> String {
    match *risk_level {
        RiskLevel::Unknown => "Wait for more trading data before investing".to_string(),
        RiskLevel::Low => "Can consider investment with standard precautions".to_string(),
        RiskLevel::Medium => {
            "Exercise caution, invest only what you can afford to lose".to_string()
        }
        RiskLevel::High => "Avoid investment, high risk of loss".to_string(),
        RiskLevel::Extreme => "DO NOT INVEST - Extreme risk or confirmed scam".to_string(),
    }
}

/// Helper function to build risk assessment from scam detection and volume analysis
fn build_risk_assessment(
    scam_detection: Option<&ScamDetection>,
    volume_analysis: Option<&VolumeAnalysis>,
    status: &ProcessingStatus,
) -> RiskAssessment {
    let mut risk_factors = Vec::new();
    let mut risk_score: f64 = 0.0;

    analyze_scam_detection(scam_detection, &mut risk_factors, &mut risk_score);
    analyze_volume_patterns(volume_analysis, &mut risk_factors, &mut risk_score);

    if risk_factors.is_empty() && matches!(*status, ProcessingStatus::Processed) {
        risk_factors.push("No major risk factors identified".to_string());
    }

    let risk_level = determine_risk_level(risk_score, status);
    let action = generate_risk_action(&risk_level);

    RiskAssessment {
        level: risk_level,
        score: risk_score,
        factors: risk_factors,
        action,
    }
}

/// Risk tolerance levels for safety checks
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RiskTolerance {
    /// Accept all except extreme risk tokens
    #[serde(rename = "high")]
    High,
    /// Only accept low risk tokens
    #[serde(rename = "low")]
    Low,
    /// Accept low and medium risk tokens
    #[serde(rename = "medium")]
    Medium,
}

/// Helper function to generate recommendation based on status and risk level
fn generate_recommendation(status: &ProcessingStatus, risk_level: &RiskLevel) -> String {
    match (status, risk_level) {
        (&ProcessingStatus::NotProcessed, _) => {
            "Token has insufficient data for analysis. Wait for more trading activity before making investment decisions.".to_string()
        }
        (_, &RiskLevel::Extreme) => {
            "EXTREME DANGER: This token shows clear signs of being a scam or rug pull. Do not invest under any circumstances.".to_string()
        }
        (_, &RiskLevel::High) => {
            "HIGH RISK: Significant red flags detected. This token has high probability of being a rug pull. Strongly recommend avoiding.".to_string()
        }
        (_, &RiskLevel::Medium) => {
            "MODERATE RISK: Some concerning patterns detected. If you choose to invest, use extreme caution and only risk what you can afford to lose.".to_string()
        }
        (_, &RiskLevel::Low) => {
            "LOW RISK: Token appears relatively safe based on wallet analysis, but always do your own research and invest responsibly.".to_string()
        }
        (_, &RiskLevel::Unknown) => {
            "UNKNOWN RISK: Unable to determine risk level. More data needed for proper assessment.".to_string()
        }
    }
}

/// Simple safety check result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SafetyCheck {
    /// Token address
    pub address: String,
    /// Additional details
    pub details: String,
    /// Whether token is considered safe given risk tolerance
    pub is_safe: bool,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Safety score (0-100, higher is safer)
    pub safety_score: f64,
    /// Simple verdict
    pub verdict: String,
}

/// Perform detailed analysis of a token's rug pull risk with comprehensive insights.
/// Provides volume breakdown, risk factors, and actionable recommendations.
///
/// # Errors
///
/// Returns `WebToolError` if:
/// - API key is not configured
/// - Network request fails
/// - Response parsing fails
#[tool]
pub async fn analyze_rug_risk(
    context: &ApplicationContext,
    address: String,
) -> Result<DetailedRugAnalysis, WebToolError> {
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

    let risk_assessment =
        build_risk_assessment(scam_detection.as_ref(), volume_analysis.as_ref(), &status);
    let recommendation = generate_recommendation(&status, &risk_assessment.level);

    Ok(DetailedRugAnalysis {
        address,
        recommendation,
        risk_assessment,
        scam_detection,
        status,
        volume_analysis,
        warnings,
    })
}

/// Quick safety check for a Solana token - returns a simple safe/unsafe verdict.
/// Best for quick filtering of tokens before deeper analysis.
///
/// # Errors
///
/// Returns `WebToolError` if:
/// - API key is not configured
/// - Network request fails
/// - Response parsing fails
#[tool]
pub async fn is_token_safe(
    context: &ApplicationContext,
    address: String,
    risk_tolerance: Option<RiskTolerance>,
) -> Result<SafetyCheck, WebToolError> {
    let risk_tolerance = risk_tolerance.unwrap_or(RiskTolerance::Low);
    let result = check_rug_pull(context, address.clone()).await?;

    let is_safe = match (&result.risk_level, &risk_tolerance) {
        (&RiskLevel::Low, _)
        | (&RiskLevel::Medium, &RiskTolerance::Medium | &RiskTolerance::High)
        | (&RiskLevel::High, &RiskTolerance::High) => true,
        (&RiskLevel::Medium, &RiskTolerance::Low)
        | (&RiskLevel::High | &RiskLevel::Unknown | &RiskLevel::Extreme, _) => false, // Different risk patterns that are all unsafe
    };

    let safety_score = match result.risk_level {
        RiskLevel::Low => result.rug_percentage.unwrap_or(0.0).mul_add(-0.8, 80.0),
        RiskLevel::Medium => result.rug_percentage.unwrap_or(25.0).mul_add(-0.6, 60.0),
        RiskLevel::High => result.rug_percentage.unwrap_or(50.0).mul_add(-0.3, 30.0),
        RiskLevel::Unknown | RiskLevel::Extreme => 0.0,
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
        details: result.message,
        is_safe,
        risk_level: result.risk_level,
        safety_score,
        verdict: verdict.to_string(),
    })
}

#[cfg(test)]
#[expect(clippy::expect_used)]
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
        let json =
            serde_json::to_string(&risk).expect("Failed to serialize RiskLevel::High to JSON");
        assert_eq!(json, "\"high\"");

        let risk: RiskLevel = serde_json::from_str("\"extreme\"")
            .expect("Failed to deserialize RiskLevel::Extreme from JSON");
        assert!(matches!(risk, RiskLevel::Extreme));
    }

    #[test]
    fn test_rug_api_response_not_processed() {
        let json = r#"{
            "status": "not_processed",
            "message": "Token has not been processed"
        }"#;

        let response: RugApiResponse = serde_json::from_str(json)
            .expect("Failed to deserialize RugApiResponse::NotProcessed from JSON");
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

        let response: RugApiResponse = serde_json::from_str(json)
            .expect("Failed to deserialize RugApiResponse::Processed from JSON");
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
        let json = serde_json::to_string(&tolerance)
            .expect("Failed to serialize RiskTolerance::Medium to JSON");
        assert_eq!(json, "\"medium\"");

        let tolerance: RiskTolerance = serde_json::from_str("\"high\"")
            .expect("Failed to deserialize RiskTolerance::High from JSON");
        assert!(matches!(tolerance, RiskTolerance::High));
    }

    #[test]
    fn test_volume_concentration_serialization() {
        let concentration = VolumeConcentration::Extreme;
        let json = serde_json::to_string(&concentration)
            .expect("Failed to serialize VolumeConcentration::Extreme to JSON");
        assert_eq!(json, "\"extreme\"");

        let concentration: VolumeConcentration = serde_json::from_str("\"distributed\"")
            .expect("Failed to deserialize VolumeConcentration::Distributed from JSON");
        assert!(matches!(concentration, VolumeConcentration::Distributed));
    }

    #[test]
    fn test_processing_status_serialization() {
        let status = ProcessingStatus::Processed;
        let json = serde_json::to_string(&status)
            .expect("Failed to serialize ProcessingStatus::Processed to JSON");
        assert_eq!(json, "\"processed\"");

        let status = ProcessingStatus::Error("test error".to_string());
        let json = serde_json::to_string(&status)
            .expect("Failed to serialize ProcessingStatus::Error to JSON");
        assert!(json.contains("error"));
        assert!(json.contains("test error"));
    }
}
