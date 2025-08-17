//! Output distillation patterns using LLMs
//!
//! This module demonstrates how to use separate LLM calls to summarize and distill
//! complex tool outputs into concise, user-friendly summaries.

use super::{OutputFormat, OutputProcessor, ProcessedOutput, ToolOutput};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use rig::providers::{anthropic, gemini, openai};
use serde_json::json;

/// LLM-based output distiller
///
/// Uses a separate LLM call to summarize complex tool outputs.
/// This pattern is useful for making technical outputs more accessible to users.
pub struct DistillationProcessor {
    model: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    #[allow(dead_code)]
    system_prompt: String,
}

impl DistillationProcessor {
    /// Create a new distillation processor with a specific model
    pub fn new(model: &str) -> Self {
        Self {
            model: model.to_string(),
            max_tokens: Some(150),
            temperature: Some(0.3),
            system_prompt: Self::default_system_prompt(),
        }
    }

    /// Create a processor with custom settings
    pub fn with_config(
        model: &str,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
        system_prompt: Option<String>,
    ) -> Self {
        Self {
            model: model.to_string(),
            max_tokens,
            temperature,
            system_prompt: system_prompt.unwrap_or_else(Self::default_system_prompt),
        }
    }

    /// Default system prompt for distillation
    fn default_system_prompt() -> String {
        r#"You are an expert at summarizing technical tool outputs into clear, concise summaries.

Your job is to:
1. Extract the key information from tool outputs
2. Present it in a user-friendly way
3. Highlight any important warnings or errors
4. Keep the summary under 2-3 sentences

Focus on what the user needs to know, not technical implementation details."#
            .to_string()
    }

    /// Call the LLM to distill the output
    async fn call_llm(&self, tool_output: &ToolOutput) -> Result<String> {
        // Create a prompt that includes the tool output
        let user_prompt = format!(
            r#"Tool: {}
Success: {}
Result: {}
{}

Please provide a concise summary of this tool output:"#,
            tool_output.tool_name,
            tool_output.success,
            serde_json::to_string_pretty(&tool_output.result)?,
            if let Some(error) = &tool_output.error {
                format!("Error: {}", error)
            } else {
                String::default()
            }
        );

        // Here you would call your preferred LLM API
        // For this example, we'll show the pattern with different providers
        match self.model.as_str() {
            m if m.starts_with("gpt-") => self.call_openai(&user_prompt).await,
            m if m.starts_with("claude-") => self.call_anthropic(&user_prompt).await,
            m if m.starts_with("gemini-") => self.call_google(&user_prompt).await,
            _ => Err(anyhow!("Unsupported model: {}", self.model)),
        }
    }

    /// Call OpenAI API (real implementation)
    async fn call_openai(&self, prompt: &str) -> Result<String> {
        // Get OpenAI client from environment variable OPENAI_API_KEY
        let client = openai::Client::from_env();

        // Build the agent with configuration
        let mut builder = client.agent(&self.model).preamble(&self.system_prompt);

        if let Some(max_tokens) = self.max_tokens {
            builder = builder.max_tokens(max_tokens.into());
        }

        if let Some(temperature) = self.temperature {
            builder = builder.temperature(temperature.into());
        }

        let agent = builder.build();

        // Make the completion request
        match agent.prompt(prompt).await {
            Ok(response) => Ok(response),
            Err(e) => Err(anyhow!("OpenAI API error: {}", e)),
        }
    }

    /// Call Anthropic Claude API (real implementation)
    async fn call_anthropic(&self, prompt: &str) -> Result<String> {
        // Get Anthropic client from environment variable ANTHROPIC_API_KEY
        let client = anthropic::Client::from_env();

        // Map model names to Anthropic's model constants
        let model = match self.model.as_str() {
            "claude-3-5-sonnet" => anthropic::CLAUDE_3_5_SONNET,
            "claude-3-5-haiku" => anthropic::CLAUDE_3_HAIKU, // Using regular haiku as 3.5 might not be available
            "claude-3-haiku" => anthropic::CLAUDE_3_HAIKU,
            "claude-3-opus" => anthropic::CLAUDE_3_OPUS,
            model => model, // Pass through other model names
        };

        // Build the agent with configuration
        let mut builder = client.agent(model).preamble(&self.system_prompt);

        if let Some(max_tokens) = self.max_tokens {
            builder = builder.max_tokens(max_tokens.into());
        }

        if let Some(temperature) = self.temperature {
            builder = builder.temperature(temperature.into());
        }

        let agent = builder.build();

        // Make the completion request
        match agent.prompt(prompt).await {
            Ok(response) => Ok(response),
            Err(e) => Err(anyhow!("Anthropic API error: {}", e)),
        }
    }

    /// Call Google Gemini API (real implementation)
    async fn call_google(&self, prompt: &str) -> Result<String> {
        // Get Gemini client from environment variable GEMINI_API_KEY
        let client = gemini::Client::from_env();

        // Build the agent with configuration
        let mut builder = client.agent(&self.model).preamble(&self.system_prompt);

        if let Some(max_tokens) = self.max_tokens {
            builder = builder.max_tokens(max_tokens.into());
        }

        if let Some(temperature) = self.temperature {
            builder = builder.temperature(temperature.into());
        }

        let agent = builder.build();

        // Make the completion request
        match agent.prompt(prompt).await {
            Ok(response) => Ok(response),
            Err(e) => Err(anyhow!("Gemini API error: {}", e)),
        }
    }
}

#[async_trait]
impl OutputProcessor for DistillationProcessor {
    async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput> {
        let summary = if input.success {
            // Try to distill successful outputs
            match self.call_llm(&input).await {
                Ok(summary) => Some(summary),
                Err(e) => {
                    // If distillation fails, log the error but don't fail the entire process
                    eprintln!("Failed to distill output: {}", e);
                    None
                }
            }
        } else {
            // For errors, create a user-friendly summary without LLM call
            Some(format!(
                "The {} tool encountered an error: {}",
                input.tool_name,
                super::utils::user_friendly_error(&input)
            ))
        };

        Ok(ProcessedOutput {
            original: input.clone(),
            processed_result: input.result,
            format: OutputFormat::PlainText,
            summary,
            routing_info: None,
        })
    }

    fn name(&self) -> &str {
        "DistillationProcessor"
    }

    fn can_process(&self, output: &ToolOutput) -> bool {
        // Can process any output, but more valuable for complex results
        !output.result.is_null() || output.error.is_some()
    }

    fn config(&self) -> serde_json::Value {
        json!({
            "name": self.name(),
            "type": "distiller",
            "model": self.model,
            "max_tokens": self.max_tokens,
            "temperature": self.temperature
        })
    }
}

/// Smart distiller that chooses different strategies based on output type
pub struct SmartDistiller {
    processors: Vec<DistillationProcessor>,
}

impl SmartDistiller {
    /// Create a new SmartDistiller with default processors
    pub fn new() -> Self {
        Self::default()
    }

    /// Choose the best processor for the given output
    fn choose_processor(&self, output: &ToolOutput) -> &DistillationProcessor {
        // Simple heuristic - in practice, you might have more sophisticated logic
        match output.tool_name.as_str() {
            name if name.contains("trading") || name.contains("swap") => &self.processors[1], // Claude for finance
            name if name.contains("balance") || name.contains("transaction") => &self.processors[2], // Gemini for simple queries
            _ => &self.processors[0], // GPT-4 for general use
        }
    }
}

#[async_trait]
impl OutputProcessor for SmartDistiller {
    async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput> {
        let processor = self.choose_processor(&input);
        processor.process(input).await
    }

    fn name(&self) -> &str {
        "SmartDistiller"
    }

    fn config(&self) -> serde_json::Value {
        json!({
            "name": self.name(),
            "type": "smart_distiller",
            "available_models": self.processors.iter().map(|p| &p.model).collect::<Vec<_>>()
        })
    }
}

/// Mock distiller for testing without API calls
pub struct MockDistiller {
    responses: std::collections::HashMap<String, String>,
}

impl MockDistiller {
    /// Create a new MockDistiller with default responses
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a custom response for a specific tool name
    pub fn with_response(mut self, tool_name: &str, response: &str) -> Self {
        self.responses
            .insert(tool_name.to_string(), response.to_string());
        self
    }
}

#[async_trait]
impl OutputProcessor for MockDistiller {
    async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput> {
        let summary = if input.success {
            self.responses
                .get(&input.tool_name)
                .or(self.responses.get("default"))
                .cloned()
        } else {
            self.responses.get("error").cloned()
        };

        Ok(ProcessedOutput {
            original: input.clone(),
            processed_result: input.result,
            format: OutputFormat::PlainText,
            summary,
            routing_info: None,
        })
    }

    fn name(&self) -> &str {
        "MockDistiller"
    }

    fn config(&self) -> serde_json::Value {
        json!({
            "name": self.name(),
            "type": "mock_distiller",
            "responses": self.responses.len()
        })
    }
}

impl Default for SmartDistiller {
    fn default() -> Self {
        Self {
            processors: vec![
                DistillationProcessor::new("gpt-4o-mini"), // Fast for simple summaries
                DistillationProcessor::new("claude-3-5-haiku"), // Good for technical content
                DistillationProcessor::new("gemini-1.5-flash"), // Cost-effective option
            ],
        }
    }
}

impl Default for MockDistiller {
    fn default() -> Self {
        let mut responses = std::collections::HashMap::default();
        responses.insert(
            "get_sol_balance".to_string(),
            "Successfully retrieved SOL balance for the specified address.".to_string(),
        );
        responses.insert(
            "swap_tokens".to_string(),
            "Token swap completed successfully.".to_string(),
        );
        responses.insert(
            "error".to_string(),
            "An error occurred while processing the request.".to_string(),
        );
        responses.insert(
            "test_tool".to_string(),
            "Test tool executed successfully.".to_string(),
        );
        Self { responses }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::utils;

    #[tokio::test]
    async fn test_distillation_processor() {
        // Skip test if OPENAI_API_KEY is not set
        if std::env::var("OPENAI_API_KEY").is_err() {
            eprintln!("Skipping test: OPENAI_API_KEY not set");
            return;
        }

        let processor = DistillationProcessor::new("gpt-4o-mini");
        let output = utils::success_output(
            "get_sol_balance",
            json!({"balance_sol": 1.5, "address": "11111111111111111111111111111112"}),
        );

        let processed = processor.process(output).await.unwrap();
        assert!(processed.summary.is_some());
    }

    #[tokio::test]
    async fn test_mock_distiller() {
        let processor =
            MockDistiller::default().with_response("test_tool", "This is a test summary");

        let output = utils::success_output("test_tool", json!({"result": "success"}));
        let processed = processor.process(output).await.unwrap();

        assert_eq!(
            processed.summary,
            Some("This is a test summary".to_string())
        );
    }

    #[tokio::test]
    async fn test_smart_distiller() {
        // Skip test if ANTHROPIC_API_KEY is not set
        if std::env::var("ANTHROPIC_API_KEY").is_err() {
            eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
            return;
        }

        let processor = SmartDistiller::default();
        let output = utils::success_output("trading_tool", json!({"profit": 100}));

        let processed = processor.process(output).await.unwrap();
        assert!(processed.summary.is_some());
    }

    #[test]
    fn test_processor_selection() {
        let distiller = SmartDistiller::default();

        let trading_output = utils::success_output("swap_tokens", json!({}));
        let balance_output = utils::success_output("get_balance", json!({}));
        let general_output = utils::success_output("general_tool", json!({}));

        // Test that different tools get routed to different processors
        let trading_processor = distiller.choose_processor(&trading_output);
        let balance_processor = distiller.choose_processor(&balance_output);
        let general_processor = distiller.choose_processor(&general_output);

        // They should potentially be different (though our mock implementation returns the same)
        assert_eq!(trading_processor.model, "claude-3-5-haiku");
        assert_eq!(balance_processor.model, "gemini-1.5-flash");
        assert_eq!(general_processor.model, "gpt-4o-mini");
    }
}
