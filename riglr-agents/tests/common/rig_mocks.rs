//! Mock LLM providers and rig-core integrations for testing.
//!
//! This module provides mock implementations of rig-core components
//! for testing riglr-agents without requiring actual LLM API calls.
//!
//! NOTE: This is currently a stub implementation. Full rig-core integration
//! mocks will be implemented in Phase 4 of the test suite development.

use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

/// Mock completion model for testing rig-core integration.
///
/// TODO: Phase 4 - Implement actual rig-core trait implementations:
/// - CompletionModel trait implementation
/// - Mock responses based on prompt content analysis
/// - Configurable response patterns
/// - Error simulation capabilities
#[derive(Debug, Clone)]
pub struct MockCompletionModel {
    responses: HashMap<String, String>,
    default_response: String,
    should_fail: bool,
    response_delay: Duration,
}

impl MockCompletionModel {
    /// Create a new mock completion model with default responses.
    pub fn new() -> Self {
        let mut responses = HashMap::with_capacity(4);

        // Default responses for common trading scenarios
        responses.insert(
            "transfer".to_string(),
            json!({
                "action": "transfer_sol",
                "reasoning": "User wants to transfer SOL",
                "confidence": 0.9
            })
            .to_string(),
        );

        responses.insert(
            "balance".to_string(),
            json!({
                "action": "get_balance",
                "reasoning": "User wants to check balance",
                "confidence": 0.95
            })
            .to_string(),
        );

        responses.insert(
            "trade".to_string(),
            json!({
                "action": "execute_trade",
                "reasoning": "User wants to execute a trade",
                "confidence": 0.85
            })
            .to_string(),
        );

        responses.insert(
            "analyze".to_string(),
            json!({
                "action": "market_analysis",
                "reasoning": "User wants market analysis",
                "confidence": 0.8
            })
            .to_string(),
        );

        Self {
            responses,
            default_response: json!({
                "action": "unknown",
                "reasoning": "Could not determine user intent",
                "confidence": 0.1
            })
            .to_string(),
            should_fail: false,
            response_delay: Duration::from_millis(100),
        }
    }

    /// Configure the model to fail requests.
    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    /// Set response delay to simulate LLM latency.
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.response_delay = delay;
        self
    }

    /// Add a custom response pattern.
    pub fn with_response(
        mut self,
        pattern: impl Into<String>,
        response: impl Into<String>,
    ) -> Self {
        self.responses.insert(pattern.into(), response.into());
        self
    }

    /// Generate a response based on the input prompt.
    ///
    /// TODO: Phase 4 - Implement actual CompletionModel trait
    pub async fn complete(&self, prompt: &str) -> Result<String, MockLLMError> {
        // Simulate processing delay
        if !self.response_delay.is_zero() {
            tokio::time::sleep(self.response_delay).await;
        }

        // Simulate failure if configured
        if self.should_fail {
            return Err(MockLLMError::RequestFailed("Mock LLM failure".to_string()));
        }

        // Find matching response pattern
        let prompt_lower = prompt.to_lowercase();
        for (pattern, response) in &self.responses {
            if prompt_lower.contains(pattern) {
                return Ok(response.clone());
            }
        }

        // Return default response
        Ok(self.default_response.clone())
    }
}

impl Default for MockCompletionModel {
    fn default() -> Self {
        let mut responses = HashMap::with_capacity(4);

        // Default responses for common trading scenarios
        responses.insert(
            "transfer".to_string(),
            json!({
                "action": "transfer_sol",
                "reasoning": "User wants to transfer SOL",
                "confidence": 0.9
            })
            .to_string(),
        );

        responses.insert(
            "balance".to_string(),
            json!({
                "action": "get_balance",
                "reasoning": "User wants to check balance",
                "confidence": 0.95
            })
            .to_string(),
        );

        responses.insert(
            "trade".to_string(),
            json!({
                "action": "execute_trade",
                "reasoning": "User wants to execute a trade",
                "confidence": 0.85
            })
            .to_string(),
        );

        responses.insert(
            "analyze".to_string(),
            json!({
                "action": "market_analysis",
                "reasoning": "User wants market analysis",
                "confidence": 0.8
            })
            .to_string(),
        );

        Self {
            responses,
            default_response: json!({
                "action": "unknown",
                "reasoning": "Could not determine user intent",
                "confidence": 0.1
            })
            .to_string(),
            should_fail: false,
            response_delay: Duration::from_millis(100),
        }
    }
}

/// Mock LLM errors for testing error handling.
#[derive(Debug, thiserror::Error)]
pub enum MockLLMError {
    /// Request to the mock LLM failed
    #[error("Request failed: {0}")]
    RequestFailed(String),

    /// Rate limit was exceeded for the mock LLM
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Invalid API key was provided
    #[error("Invalid API key")]
    InvalidApiKey,

    /// Requested model was not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Request timed out after the specified duration
    #[error("Timeout after {0:?}")]
    Timeout(Duration),
}

/// Mock OpenAI provider for testing.
///
/// TODO: Phase 4 - Implement rig-core provider trait
#[derive(Debug)]
pub struct MockOpenAIProvider {
    model: MockCompletionModel,
    #[allow(dead_code)]
    api_key: String,
}

impl MockOpenAIProvider {
    /// Create a new mock OpenAI provider with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            model: MockCompletionModel::default(),
            api_key: api_key.into(),
        }
    }

    /// Set a custom completion model for this provider.
    pub fn with_model(mut self, model: MockCompletionModel) -> Self {
        self.model = model;
        self
    }

    /// Simulate creating an agent with this provider.
    ///
    /// TODO: Phase 4 - Return actual rig-core Agent instance
    pub fn agent(&self, model_name: &str) -> MockAgentBuilder {
        MockAgentBuilder::new(model_name, self.model.clone())
    }
}

/// Mock Claude provider for testing.
///
/// TODO: Phase 4 - Implement rig-core provider trait
#[derive(Debug)]
pub struct MockClaudeProvider {
    model: MockCompletionModel,
    #[allow(dead_code)]
    api_key: String,
}

impl MockClaudeProvider {
    /// Create a new mock Claude provider with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            model: MockCompletionModel::default(),
            api_key: api_key.into(),
        }
    }

    /// Set a custom completion model for this provider.
    pub fn with_model(mut self, model: MockCompletionModel) -> Self {
        self.model = model;
        self
    }

    /// Create a mock agent builder using this provider.
    pub fn agent(&self, model_name: &str) -> MockAgentBuilder {
        MockAgentBuilder::new(model_name, self.model.clone())
    }
}

/// Builder for creating mock rig-core agents.
///
/// TODO: Phase 4 - Replace with actual rig-core agent builder
#[derive(Debug)]
pub struct MockAgentBuilder {
    model_name: String,
    completion_model: MockCompletionModel,
    preamble: Option<String>,
    tools: Vec<String>,
}

impl MockAgentBuilder {
    /// Create a new mock agent builder with the specified model name and completion model.
    pub fn new(model_name: impl Into<String>, completion_model: MockCompletionModel) -> Self {
        Self {
            model_name: model_name.into(),
            completion_model,
            preamble: None,
            tools: Vec::with_capacity(0),
        }
    }

    /// Set a preamble for the agent to provide context before user prompts.
    pub fn preamble(mut self, preamble: impl Into<String>) -> Self {
        self.preamble = Some(preamble.into());
        self
    }

    /// Add a tool that the agent can use.
    pub fn tool(mut self, tool_name: impl Into<String>) -> Self {
        self.tools.push(tool_name.into());
        self
    }

    /// Build the mock rig agent with the configured settings.
    pub fn build(self) -> MockRigAgent {
        MockRigAgent {
            model_name: self.model_name,
            completion_model: self.completion_model,
            preamble: self.preamble,
            tools: self.tools,
        }
    }
}

/// Mock rig-core agent for testing.
///
/// TODO: Phase 4 - Implement actual rig-core Agent trait
#[derive(Debug)]
pub struct MockRigAgent {
    model_name: String,
    completion_model: MockCompletionModel,
    preamble: Option<String>,
    tools: Vec<String>,
}

impl MockRigAgent {
    /// Simulate prompting the agent.
    ///
    /// TODO: Phase 4 - Implement actual rig-core prompt/completion pattern
    pub async fn prompt(&self, prompt: &str) -> Result<String, MockLLMError> {
        // Combine preamble with prompt if available
        let full_prompt = match &self.preamble {
            Some(preamble) => format!("{}\n\nUser: {}", preamble, prompt),
            None => prompt.to_string(),
        };

        self.completion_model.complete(&full_prompt).await
    }

    /// Get available tools.
    pub fn tools(&self) -> &[String] {
        &self.tools
    }

    /// Get model name.
    pub fn model_name(&self) -> &str {
        &self.model_name
    }
}

/// Test scenario generators for different AI response patterns.
#[derive(Debug)]
pub struct AIResponseScenarios;

impl AIResponseScenarios {
    /// Create responses for trading scenarios.
    pub fn trading_responses() -> Vec<(String, String)> {
        vec![
            (
                "transfer 1 SOL to wallet".to_string(),
                json!({
                    "action": "transfer_sol",
                    "amount": 1.0,
                    "reasoning": "User wants to transfer 1 SOL"
                })
                .to_string(),
            ),
            (
                "check my balance".to_string(),
                json!({
                    "action": "get_balance",
                    "reasoning": "User wants to check balance"
                })
                .to_string(),
            ),
            (
                "buy BONK tokens".to_string(),
                json!({
                    "action": "token_swap",
                    "token": "BONK",
                    "action_type": "buy",
                    "reasoning": "User wants to purchase BONK tokens"
                })
                .to_string(),
            ),
            (
                "analyze BTC market".to_string(),
                json!({
                    "action": "market_analysis",
                    "symbol": "BTC",
                    "reasoning": "User wants BTC market analysis"
                })
                .to_string(),
            ),
        ]
    }

    /// Create responses for risk assessment scenarios.
    pub fn risk_assessment_responses() -> Vec<(String, String)> {
        vec![
            (
                "high risk trade".to_string(),
                json!({
                    "risk_level": "high",
                    "recommendation": "reject",
                    "reasoning": "Trade involves high risk"
                })
                .to_string(),
            ),
            (
                "low risk trade".to_string(),
                json!({
                    "risk_level": "low",
                    "recommendation": "approve",
                    "reasoning": "Trade has acceptable risk"
                })
                .to_string(),
            ),
        ]
    }

    /// Create error responses for testing error handling.
    pub fn error_responses() -> Vec<(String, MockLLMError)> {
        vec![
            (
                "rate limit test".to_string(),
                MockLLMError::RateLimitExceeded,
            ),
            (
                "timeout test".to_string(),
                MockLLMError::Timeout(Duration::from_secs(30)),
            ),
            (
                "invalid model test".to_string(),
                MockLLMError::ModelNotFound("gpt-nonexistent".to_string()),
            ),
        ]
    }
}

/// Utility functions for testing with mock LLMs.
pub mod test_utils {
    use super::*;

    /// Create a mock completion model with trading-specific responses.
    pub fn trading_completion_model() -> MockCompletionModel {
        let mut model = MockCompletionModel::default();

        for (prompt, response) in AIResponseScenarios::trading_responses() {
            model = model.with_response(prompt, response);
        }

        model
    }

    /// Create a mock completion model that always fails.
    pub fn failing_completion_model() -> MockCompletionModel {
        MockCompletionModel::default().with_failure()
    }

    /// Create a mock completion model with slow responses.
    pub fn slow_completion_model() -> MockCompletionModel {
        MockCompletionModel::default().with_delay(Duration::from_secs(2))
    }

    /// Extract action from a JSON response string.
    pub fn extract_action(response: &str) -> Result<String, serde_json::Error> {
        let parsed: serde_json::Value = serde_json::from_str(response)?;
        Ok(parsed["action"].as_str().unwrap_or("unknown").to_string())
    }

    /// Extract reasoning from a JSON response string.
    pub fn extract_reasoning(response: &str) -> Result<String, serde_json::Error> {
        let parsed: serde_json::Value = serde_json::from_str(response)?;
        Ok(parsed["reasoning"]
            .as_str()
            .unwrap_or("No reasoning provided")
            .to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::*;
    use super::*;

    #[tokio::test]
    async fn test_mock_completion_model_basic() {
        let model = MockCompletionModel::default();

        let response = model.complete("transfer some SOL").await.unwrap();
        let action = extract_action(&response).unwrap();
        assert_eq!(action, "transfer_sol");
    }

    #[tokio::test]
    async fn test_mock_completion_model_balance_query() {
        let model = MockCompletionModel::default();

        let response = model.complete("what is my balance?").await.unwrap();
        let action = extract_action(&response).unwrap();
        assert_eq!(action, "get_balance");
    }

    #[tokio::test]
    async fn test_mock_completion_model_unknown_intent() {
        let model = MockCompletionModel::default();

        let response = model.complete("random gibberish xyz").await.unwrap();
        let action = extract_action(&response).unwrap();
        assert_eq!(action, "unknown");
    }

    #[tokio::test]
    async fn test_mock_completion_model_failure() {
        let model = MockCompletionModel::default().with_failure();

        let result = model.complete("test prompt").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MockLLMError::RequestFailed(_)
        ));
    }

    #[tokio::test]
    async fn test_mock_completion_model_custom_response() {
        let model = MockCompletionModel::default().with_response(
            "custom_pattern",
            json!({"action": "custom_action"}).to_string(),
        );

        let response = model
            .complete("this contains custom_pattern")
            .await
            .unwrap();
        let action = extract_action(&response).unwrap();
        assert_eq!(action, "custom_action");
    }

    #[tokio::test]
    async fn test_mock_completion_model_delay() {
        use std::time::Instant;

        let delay = Duration::from_millis(200);
        let model = MockCompletionModel::default().with_delay(delay);

        let start = Instant::now();
        let _response = model.complete("test").await.unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed >= delay);
    }

    #[test]
    fn test_mock_openai_provider() {
        let provider = MockOpenAIProvider::new("test-api-key");
        let agent = provider.agent("gpt-4").build();

        assert_eq!(agent.model_name(), "gpt-4");
        assert_eq!(agent.tools().len(), 0);
    }

    #[test]
    fn test_mock_claude_provider() {
        let provider = MockClaudeProvider::new("test-api-key");
        let agent = provider
            .agent("claude-3")
            .preamble("You are a helpful assistant")
            .tool("calculator")
            .tool("web_search")
            .build();

        assert_eq!(agent.model_name(), "claude-3");
        assert_eq!(agent.tools().len(), 2);
        assert!(agent.tools().contains(&"calculator".to_string()));
    }

    #[tokio::test]
    async fn test_mock_rig_agent_with_preamble() {
        let model = MockCompletionModel::default();
        let agent = MockAgentBuilder::new("gpt-4", model)
            .preamble("You are a trading agent.")
            .build();

        let response = agent.prompt("transfer money").await.unwrap();
        let action = extract_action(&response).unwrap();
        assert_eq!(action, "transfer_sol");
    }

    #[test]
    fn test_ai_response_scenarios() {
        let trading_responses = AIResponseScenarios::trading_responses();
        assert!(!trading_responses.is_empty());

        let risk_responses = AIResponseScenarios::risk_assessment_responses();
        assert!(!risk_responses.is_empty());

        let error_responses = AIResponseScenarios::error_responses();
        assert!(!error_responses.is_empty());
    }

    #[test]
    fn test_trading_completion_model() {
        let model = trading_completion_model();
        // Model should have default responses plus trading-specific ones
        assert!(!model.responses.is_empty());
    }

    #[test]
    fn test_failing_completion_model() {
        let model = failing_completion_model();
        assert!(model.should_fail);
    }

    #[test]
    fn test_response_extraction() {
        let response = json!({
            "action": "test_action",
            "reasoning": "test reasoning"
        })
        .to_string();

        assert_eq!(extract_action(&response).unwrap(), "test_action");
        assert_eq!(extract_reasoning(&response).unwrap(), "test reasoning");
    }

    #[test]
    fn test_response_extraction_invalid_json() {
        let invalid_response = "not json";
        assert!(extract_action(invalid_response).is_err());
    }

    #[test]
    fn test_response_extraction_missing_fields() {
        let response = json!({"other_field": "value"}).to_string();
        assert_eq!(extract_action(&response).unwrap(), "unknown");
        assert_eq!(
            extract_reasoning(&response).unwrap(),
            "No reasoning provided"
        );
    }
}
