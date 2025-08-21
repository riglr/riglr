//! Notification routing patterns for tool outputs
//!
//! This module demonstrates how to route tool outputs to different notification
//! channels like Discord, Telegram, Slack, email, and custom webhooks.

use super::{
    NotificationPriority, OutputFormat, OutputProcessor, ProcessedOutput, RoutingInfo, ToolOutput,
};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;

/// Notification router that sends outputs to various channels
#[derive(Default)]
pub struct NotificationRouter {
    /// Map of channel names to notification channel implementations
    channels: HashMap<String, Box<dyn NotificationChannel>>,
    /// List of routing rules that determine which channels to use
    routing_rules: Vec<RoutingRule>,
    /// Default channel to use when no routing rules match
    default_channel: Option<String>,
}

impl NotificationRouter {
    /// Creates a new notification router with no channels or rules
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            routing_rules: Vec::new(),
            default_channel: None,
        }
    }

    /// Adds a notification channel with the given name
    pub fn add_channel<C: NotificationChannel + 'static>(mut self, name: &str, channel: C) -> Self {
        self.channels.insert(name.to_string(), Box::new(channel));
        self
    }

    /// Sets the default channel to use when no routing rules match
    pub fn set_default_channel(mut self, name: &str) -> Self {
        self.default_channel = Some(name.to_string());
        self
    }

    /// Adds a routing rule for determining which channels to use
    pub fn add_routing_rule(mut self, rule: RoutingRule) -> Self {
        self.routing_rules.push(rule);
        self
    }

    /// Determine which channels to route to based on output and rules
    fn determine_routes(&self, output: &ToolOutput) -> Vec<String> {
        let mut routes = Vec::new();

        // Apply routing rules
        for rule in &self.routing_rules {
            if rule.matches(output) {
                routes.extend(rule.channels.clone());
            }
        }

        // If no rules matched, use default channel
        if routes.is_empty() {
            if let Some(default) = &self.default_channel {
                routes.push(default.clone());
            }
        }

        // Remove duplicates
        routes.sort();
        routes.dedup();

        routes
    }

    /// Send notifications to all matching channels
    async fn send_notifications(
        &self,
        output: &ToolOutput,
        routes: &[String],
    ) -> Result<Vec<NotificationResult>> {
        let mut results = Vec::new();

        for channel_name in routes {
            if let Some(channel) = self.channels.get(channel_name) {
                let result = match channel.send_notification(output).await {
                    Ok(id) => NotificationResult {
                        channel: channel_name.clone(),
                        success: true,
                        message_id: Some(id),
                        error: None,
                    },
                    Err(e) => NotificationResult {
                        channel: channel_name.clone(),
                        success: false,
                        message_id: None,
                        error: Some(e.to_string()),
                    },
                };
                results.push(result);
            } else {
                results.push(NotificationResult {
                    channel: channel_name.clone(),
                    success: false,
                    message_id: None,
                    error: Some(format!("Channel '{}' not found", channel_name)),
                });
            }
        }

        Ok(results)
    }
}

#[async_trait]
impl OutputProcessor for NotificationRouter {
    async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput> {
        let routes = self.determine_routes(&input);
        let notification_results = self.send_notifications(&input, &routes).await?;

        let routing_info = RoutingInfo {
            channel: routes.join(", "),
            recipients: routes.clone(),
            priority: self.determine_priority(&input),
        };

        Ok(ProcessedOutput {
            original: input.clone(),
            processed_result: json!({
                "notification_results": notification_results,
                "routes_used": routes,
                "total_notifications": notification_results.len()
            }),
            format: OutputFormat::Json,
            summary: Some(format!(
                "Sent notifications to {} channel(s): {}",
                routes.len(),
                routes.join(", ")
            )),
            routing_info: Some(routing_info),
        })
    }

    fn name(&self) -> &str {
        "NotificationRouter"
    }

    fn config(&self) -> serde_json::Value {
        json!({
            "name": self.name(),
            "type": "notifier",
            "channels": self.channels.keys().collect::<Vec<_>>(),
            "routing_rules": self.routing_rules.len(),
            "default_channel": self.default_channel
        })
    }

    fn can_process(&self, _output: &ToolOutput) -> bool {
        !self.channels.is_empty()
    }
}

impl NotificationRouter {
    fn determine_priority(&self, output: &ToolOutput) -> NotificationPriority {
        if !output.success {
            NotificationPriority::High
        } else if output.tool_name.contains("trading") || output.tool_name.contains("swap") {
            NotificationPriority::Normal
        } else {
            NotificationPriority::Low
        }
    }
}

/// Routing rule for determining which channels to use
#[derive(Clone)]
pub struct RoutingRule {
    /// Human-readable name for this rule
    pub name: String,
    /// Condition that determines when this rule applies
    pub condition: RoutingCondition,
    /// List of channel names to route to when condition matches
    pub channels: Vec<String>,
}

impl RoutingRule {
    /// Creates a new routing rule with the given name, condition, and target channels
    pub fn new(name: &str, condition: RoutingCondition, channels: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            condition,
            channels,
        }
    }

    /// Checks if this routing rule matches the given tool output
    pub fn matches(&self, output: &ToolOutput) -> bool {
        self.condition.matches(output)
    }
}

/// Conditions for routing decisions
#[derive(Clone)]
pub enum RoutingCondition {
    /// Always matches any output
    Always,
    /// Matches only successful outputs
    OnSuccess,
    /// Matches only failed outputs
    OnError,
    /// Matches outputs from a specific tool by exact name
    ToolName(String),
    /// Matches outputs from tools whose name contains the given substring
    ToolNameContains(String),
    /// Matches outputs that took longer than the specified number of milliseconds
    ExecutionTimeOver(u64), // milliseconds
    /// Matches outputs that have metadata with the specified key
    HasMetadata(String),
    /// Matches when all nested conditions match
    And(Vec<RoutingCondition>),
    /// Matches when any nested condition matches
    Or(Vec<RoutingCondition>),
    /// Matches when the nested condition does not match
    Not(Box<RoutingCondition>),
}

impl RoutingCondition {
    /// Evaluates whether this condition matches the given tool output
    pub fn matches(&self, output: &ToolOutput) -> bool {
        match self {
            RoutingCondition::Always => true,
            RoutingCondition::OnSuccess => output.success,
            RoutingCondition::OnError => !output.success,
            RoutingCondition::ToolName(name) => output.tool_name == *name,
            RoutingCondition::ToolNameContains(substr) => output.tool_name.contains(substr),
            RoutingCondition::ExecutionTimeOver(threshold) => output.execution_time_ms > *threshold,
            RoutingCondition::HasMetadata(key) => output.metadata.contains_key(key),
            RoutingCondition::And(conditions) => conditions.iter().all(|c| c.matches(output)),
            RoutingCondition::Or(conditions) => conditions.iter().any(|c| c.matches(output)),
            RoutingCondition::Not(condition) => !condition.matches(output),
        }
    }
}

/// Result of a notification attempt
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct NotificationResult {
    /// Name of the channel that was used for the notification
    pub channel: String,
    /// Whether the notification was sent successfully
    pub success: bool,
    /// Optional message ID returned by the notification service
    pub message_id: Option<String>,
    /// Optional error message if the notification failed
    pub error: Option<String>,
}

/// Trait for notification channels
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Sends a notification for the given tool output and returns a message ID
    async fn send_notification(&self, output: &ToolOutput) -> Result<String>;
    /// Returns the human-readable name of this channel
    fn name(&self) -> &str;
    /// Returns whether this channel supports rich formatting (default: true)
    fn supports_formatting(&self) -> bool {
        true
    }
}

/// Discord webhook notification channel
pub struct DiscordChannel {
    /// Discord webhook URL for sending messages
    #[allow(dead_code)]
    webhook_url: String,
    /// Optional username to display for the bot
    username: Option<String>,
    /// Optional avatar URL for the bot
    avatar_url: Option<String>,
}

impl DiscordChannel {
    /// Creates a new Discord channel with the given webhook URL
    pub fn new(webhook_url: &str) -> Self {
        Self {
            webhook_url: webhook_url.to_string(),
            username: None,
            avatar_url: None,
        }
    }

    /// Sets the bot username and optional avatar URL for Discord messages
    pub fn with_identity(mut self, username: &str, avatar_url: Option<&str>) -> Self {
        self.username = Some(username.to_string());
        self.avatar_url = avatar_url.map(|s| s.to_string());
        self
    }

    fn format_for_discord(&self, output: &ToolOutput) -> serde_json::Value {
        let color = if output.success { 0x00ff00 } else { 0xff0000 }; // Green or red
        let status_emoji = if output.success { "✅" } else { "❌" };

        let mut embed = json!({
            "title": format!("{} {} Results", status_emoji, self.title_case(&output.tool_name)),
            "color": color,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "fields": []
        });

        // Add result fields
        if !output.result.is_null() {
            if let serde_json::Value::Object(obj) = &output.result {
                for (key, value) in obj {
                    embed["fields"].as_array_mut().unwrap().push(json!({
                        "name": self.title_case(key),
                        "value": format!("`{}`", value),
                        "inline": true
                    }));
                }
            }
        }

        // Add error field if present
        if let Some(error) = &output.error {
            embed["fields"].as_array_mut().unwrap().push(json!({
                "name": "Error",
                "value": format!("```\n{}\n```", error),
                "inline": false
            }));
        }

        // Add timing info
        if output.execution_time_ms > 0 {
            embed["footer"] = json!({
                "text": format!("Executed in {}ms", output.execution_time_ms)
            });
        }

        let mut payload = json!({
            "embeds": [embed]
        });

        if let Some(username) = &self.username {
            payload["username"] = json!(username);
        }

        if let Some(avatar_url) = &self.avatar_url {
            payload["avatar_url"] = json!(avatar_url);
        }

        payload
    }

    fn title_case(&self, s: &str) -> String {
        s.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::default(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[async_trait]
impl NotificationChannel for DiscordChannel {
    async fn send_notification(&self, output: &ToolOutput) -> Result<String> {
        let payload = self.format_for_discord(output);

        // Real implementation using reqwest
        let client = reqwest::Client::new();
        let response = client
            .post(&self.webhook_url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send Discord webhook: {}", e))?;

        if response.status().is_success() {
            Ok(format!("discord_msg_{}", chrono::Utc::now().timestamp()))
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!(
                "Discord webhook failed with status {}: {}",
                status,
                error_text
            ))
        }
    }

    fn name(&self) -> &str {
        "Discord"
    }
}

/// Telegram bot notification channel
pub struct TelegramChannel {
    /// Telegram bot token for API authentication
    #[allow(dead_code)]
    bot_token: String,
    /// Chat ID where messages will be sent
    #[allow(dead_code)]
    chat_id: String,
    /// Parse mode for message formatting (Markdown or HTML)
    parse_mode: String,
}

impl TelegramChannel {
    /// Creates a new Telegram channel with the given bot token and chat ID
    pub fn new(bot_token: &str, chat_id: &str) -> Self {
        Self {
            bot_token: bot_token.to_string(),
            chat_id: chat_id.to_string(),
            parse_mode: "Markdown".to_string(),
        }
    }

    /// Switches message formatting from Markdown to HTML mode
    pub fn with_html_mode(mut self) -> Self {
        self.parse_mode = "HTML".to_string();
        self
    }

    fn format_for_telegram(&self, output: &ToolOutput) -> String {
        let status_emoji = if output.success { "✅" } else { "❌" };
        let mut message = format!(
            "{} *{}*\n",
            status_emoji,
            self.escape_markdown(&output.tool_name)
        );

        if !output.result.is_null() {
            message.push_str("\n📊 *Result:*\n");
            message.push_str(&format!(
                "```json\n{}\n```\n",
                serde_json::to_string_pretty(&output.result).unwrap_or_default()
            ));
        }

        if let Some(error) = &output.error {
            message.push_str("\n❌ *Error:*\n");
            message.push_str(&format!("```\n{}\n```\n", error));
        }

        if output.execution_time_ms > 0 {
            message.push_str(&format!("\n⏱️ Executed in {}ms", output.execution_time_ms));
        }

        message
    }

    fn escape_markdown(&self, text: &str) -> String {
        text.replace('*', r"\*")
            .replace('_', r"\_")
            .replace('[', r"\[")
            .replace(']', r"\]")
            .replace('(', r"\(")
            .replace(')', r"\)")
            .replace('~', r"\~")
            .replace('`', r"\`")
            .replace('>', r"\>")
            .replace('#', r"\#")
            .replace('+', r"\+")
            .replace('-', r"\-")
            .replace('=', r"\=")
            .replace('|', r"\|")
            .replace('{', r"\{")
            .replace('}', r"\}")
            .replace('.', r"\.")
            .replace('!', r"\!")
    }
}

#[async_trait]
impl NotificationChannel for TelegramChannel {
    async fn send_notification(&self, output: &ToolOutput) -> Result<String> {
        let message = self.format_for_telegram(output);

        // Real implementation with Telegram Bot API
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        let payload = json!({
            "chat_id": self.chat_id,
            "text": message,
            "parse_mode": self.parse_mode
        });

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send Telegram message: {}", e))?;

        if response.status().is_success() {
            let result: serde_json::Value = response
                .json()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to parse Telegram response: {}", e))?;

            if let Some(message_id) = result["result"]["message_id"].as_i64() {
                Ok(format!("tg_msg_{}", message_id))
            } else {
                Err(anyhow::anyhow!("Invalid Telegram response format"))
            }
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!(
                "Telegram API failed with status {}: {}",
                status,
                error_text
            ))
        }
    }

    fn name(&self) -> &str {
        "Telegram"
    }
}

/// Generic webhook notification channel
pub struct WebhookChannel {
    /// Human-readable name for this webhook
    name: String,
    /// Webhook URL endpoint
    #[allow(dead_code)]
    url: String,
    /// HTTP headers to include with webhook requests
    headers: HashMap<String, String>,
    /// Template string for formatting notification messages
    format_template: String,
}

impl WebhookChannel {
    /// Creates a new webhook channel with the given name and URL
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            headers: HashMap::new(),
            format_template: "{\"message\": \"{{message}}\"}".to_string(),
        }
    }

    /// Adds an HTTP header to include with webhook requests
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Sets a custom format template for webhook message bodies
    pub fn with_format_template(mut self, template: &str) -> Self {
        self.format_template = template.to_string();
        self
    }

    fn format_message(&self, output: &ToolOutput) -> String {
        let status = if output.success { "SUCCESS" } else { "FAILED" };
        let default_error = "Unknown error".to_string();
        let error_msg = if output.success {
            "Operation completed successfully"
        } else {
            output.error.as_ref().unwrap_or(&default_error).as_str()
        };
        let summary = format!("Tool '{}' {}: {}", output.tool_name, status, error_msg);

        // Simple template replacement
        self.format_template
            .replace("{{message}}", &summary)
            .replace("{{tool_name}}", &output.tool_name)
            .replace("{{status}}", status)
            .replace(
                "{{result}}",
                &serde_json::to_string(&output.result).unwrap_or_default(),
            )
    }
}

#[async_trait]
impl NotificationChannel for WebhookChannel {
    async fn send_notification(&self, output: &ToolOutput) -> Result<String> {
        let message = self.format_message(output);

        // Real implementation with configurable webhook
        let client = reqwest::Client::new();
        let mut request = client
            .post(&self.url)
            .timeout(std::time::Duration::from_secs(30));

        // Add custom headers
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        // Set content-type if not already specified
        if !self.headers.contains_key("Content-Type") {
            request = request.header("Content-Type", "application/json");
        }

        // Send the request with body
        let response = request
            .body(message)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send webhook to {}: {}", self.url, e))?;

        let status = response.status();

        if status.is_success() {
            Ok(format!(
                "webhook_{}_{}",
                self.name,
                chrono::Utc::now().timestamp()
            ))
        } else {
            // Try to get error details from response
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!(
                "Webhook '{}' failed with status {}: {}",
                self.name,
                status,
                error_text
            ))
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Console/log notification channel for debugging
pub struct ConsoleChannel {
    /// Whether to use ANSI color codes in console output
    use_colors: bool,
}

impl ConsoleChannel {
    /// Creates a new console channel with color output enabled
    pub fn new() -> Self {
        Self { use_colors: true }
    }

    /// Disables color output for plain text logging
    pub fn without_colors(mut self) -> Self {
        self.use_colors = false;
        self
    }
}

#[async_trait]
impl NotificationChannel for ConsoleChannel {
    async fn send_notification(&self, output: &ToolOutput) -> Result<String> {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

        if self.use_colors {
            if output.success {
                println!(
                    "\x1b[32m✅ [{}] {} completed successfully\x1b[0m",
                    timestamp, output.tool_name
                );
                if !output.result.is_null() {
                    println!(
                        "\x1b[36mResult: {}\x1b[0m",
                        serde_json::to_string_pretty(&output.result)?
                    );
                }
            } else {
                println!(
                    "\x1b[31m❌ [{}] {} failed\x1b[0m",
                    timestamp, output.tool_name
                );
                if let Some(error) = &output.error {
                    println!("\x1b[31mError: {}\x1b[0m", error);
                }
            }
        } else {
            let status = if output.success { "SUCCESS" } else { "FAILED" };
            println!("[{}] {} {}", timestamp, output.tool_name, status);

            if output.success && !output.result.is_null() {
                println!("Result: {}", serde_json::to_string_pretty(&output.result)?);
            } else if let Some(error) = &output.error {
                println!("Error: {}", error);
            }
        }

        Ok(format!("console_{}", chrono::Utc::now().timestamp()))
    }

    fn name(&self) -> &str {
        "Console"
    }
}

impl Default for ConsoleChannel {
    fn default() -> Self {
        Self { use_colors: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::utils;

    #[tokio::test]
    async fn test_notification_router() {
        let router = NotificationRouter::default()
            .add_channel("console", ConsoleChannel::default())
            .set_default_channel("console")
            .add_routing_rule(RoutingRule::new(
                "error_routing",
                RoutingCondition::OnError,
                vec!["console".to_string()],
            ));

        let output = utils::error_output("test_tool", "Something went wrong");
        let processed = router.process(output).await.unwrap();

        assert!(processed.routing_info.is_some());
        assert!(processed.summary.is_some());

        let results = processed.processed_result["notification_results"]
            .as_array()
            .unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_routing_conditions() {
        let success_output = utils::success_output("test", json!({}));
        let error_output = utils::error_output("test", "error");

        assert!(RoutingCondition::Always.matches(&success_output));
        assert!(RoutingCondition::Always.matches(&error_output));

        assert!(RoutingCondition::OnSuccess.matches(&success_output));
        assert!(!RoutingCondition::OnSuccess.matches(&error_output));

        assert!(!RoutingCondition::OnError.matches(&success_output));
        assert!(RoutingCondition::OnError.matches(&error_output));

        assert!(RoutingCondition::ToolName("test".to_string()).matches(&success_output));
        assert!(!RoutingCondition::ToolName("other".to_string()).matches(&success_output));

        assert!(RoutingCondition::ToolNameContains("tes".to_string()).matches(&success_output));
    }

    #[test]
    fn test_complex_routing_conditions() {
        let output = utils::success_output("swap_tokens", json!({}));

        let condition = RoutingCondition::And(vec![
            RoutingCondition::OnSuccess,
            RoutingCondition::ToolNameContains("swap".to_string()),
        ]);

        assert!(condition.matches(&output));

        let condition = RoutingCondition::Or(vec![
            RoutingCondition::OnError,
            RoutingCondition::ToolNameContains("balance".to_string()),
        ]);

        assert!(!condition.matches(&output));

        let condition = RoutingCondition::Not(Box::new(RoutingCondition::OnError));
        assert!(condition.matches(&output));
    }

    #[tokio::test]
    async fn test_discord_formatting() {
        let channel = DiscordChannel::new("https://discord.com/api/webhooks/test")
            .with_identity("RiglrBot", Some("https://example.com/avatar.png"));

        let output = utils::success_output(
            "get_balance",
            json!({"balance": "1.5 SOL", "address": "11111111111111111111111111111112"}),
        );

        let formatted = channel.format_for_discord(&output);

        assert!(formatted["embeds"].is_array());
        assert!(formatted["username"] == "RiglrBot");
        assert!(formatted["avatar_url"] == "https://example.com/avatar.png");

        let embed = &formatted["embeds"][0];
        assert!(embed["title"]
            .as_str()
            .unwrap()
            .contains("Get Balance Results"));
        assert!(embed["color"] == 0x00ff00); // Green for success
    }

    #[tokio::test]
    async fn test_telegram_formatting() {
        let channel = TelegramChannel::new("bot_token", "chat_id");
        let output = utils::error_output("trading_bot", "Insufficient balance");

        let formatted = channel.format_for_telegram(&output);

        assert!(formatted.contains("❌"));
        assert!(formatted.contains("trading\\_bot"));
        assert!(formatted.contains("Insufficient balance"));
        assert!(formatted.contains("```"));
    }

    #[tokio::test]
    async fn test_webhook_channel() {
        let channel = WebhookChannel::new("custom", "https://example.com/webhook")
            .with_header("Authorization", "Bearer token")
            .with_format_template(r#"{"text": "{{message}}", "tool": "{{tool_name}}"}"#);

        let output = utils::success_output("test", json!({}));

        // Test may fail due to network issues or example.com blocking, so we check for expected behavior
        match channel.send_notification(&output).await {
            Ok(result) => {
                assert!(result.starts_with("webhook_custom_"));
            }
            Err(e) => {
                // Expected to fail with 403 or connection issues in test environment
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("403")
                        || error_msg.contains("Failed to send webhook")
                        || error_msg.contains("connection"),
                    "Unexpected error: {}",
                    error_msg
                );
            }
        }
    }

    // Additional comprehensive tests for 100% code coverage

    #[test]
    fn test_notification_router_new() {
        let router = NotificationRouter::default();
        assert!(router.channels.is_empty());
        assert!(router.routing_rules.is_empty());
        assert!(router.default_channel.is_none());
    }

    #[test]
    fn test_notification_router_default() {
        let router = NotificationRouter::default();
        assert!(router.channels.is_empty());
        assert!(router.routing_rules.is_empty());
        assert!(router.default_channel.is_none());
    }

    #[test]
    fn test_notification_router_builder_methods() {
        let console = ConsoleChannel::default();
        let router = NotificationRouter::default()
            .add_channel("test", console)
            .set_default_channel("test")
            .add_routing_rule(RoutingRule::new(
                "test_rule",
                RoutingCondition::Always,
                vec!["test".to_string()],
            ));

        assert_eq!(router.channels.len(), 1);
        assert_eq!(router.default_channel, Some("test".to_string()));
        assert_eq!(router.routing_rules.len(), 1);
    }

    #[test]
    fn test_notification_router_can_process() {
        let router = NotificationRouter::default();
        let output = utils::success_output("test", json!({}));
        assert!(!router.can_process(&output));

        let router = router.add_channel("console", ConsoleChannel::default());
        assert!(router.can_process(&output));
    }

    #[test]
    fn test_notification_router_name_and_config() {
        let router = NotificationRouter::default()
            .add_channel("console", ConsoleChannel::default())
            .set_default_channel("console");

        assert_eq!(router.name(), "NotificationRouter");

        let config = router.config();
        assert_eq!(config["name"], "NotificationRouter");
        assert_eq!(config["type"], "notifier");
        assert_eq!(config["default_channel"], "console");
        assert_eq!(config["routing_rules"], 0);
    }

    #[test]
    fn test_determine_routes_with_default_channel() {
        let router = NotificationRouter::default()
            .add_channel("console", ConsoleChannel::default())
            .set_default_channel("console");

        let output = utils::success_output("test", json!({}));
        let routes = router.determine_routes(&output);

        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0], "console");
    }

    #[test]
    fn test_determine_routes_with_matching_rules() {
        let router = NotificationRouter::default()
            .add_channel("console", ConsoleChannel::default())
            .add_channel("discord", DiscordChannel::new("https://example.com"))
            .add_routing_rule(RoutingRule::new(
                "success_rule",
                RoutingCondition::OnSuccess,
                vec!["console".to_string(), "discord".to_string()],
            ));

        let output = utils::success_output("test", json!({}));
        let routes = router.determine_routes(&output);

        assert_eq!(routes.len(), 2);
        assert!(routes.contains(&"console".to_string()));
        assert!(routes.contains(&"discord".to_string()));
    }

    #[test]
    fn test_determine_routes_removes_duplicates() {
        let router = NotificationRouter::default()
            .add_channel("console", ConsoleChannel::default())
            .add_routing_rule(RoutingRule::new(
                "rule1",
                RoutingCondition::Always,
                vec!["console".to_string()],
            ))
            .add_routing_rule(RoutingRule::new(
                "rule2",
                RoutingCondition::Always,
                vec!["console".to_string()],
            ));

        let output = utils::success_output("test", json!({}));
        let routes = router.determine_routes(&output);

        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0], "console");
    }

    #[test]
    fn test_determine_routes_no_match_no_default() {
        let router = NotificationRouter::default()
            .add_channel("console", ConsoleChannel::default())
            .add_routing_rule(RoutingRule::new(
                "error_only",
                RoutingCondition::OnError,
                vec!["console".to_string()],
            ));

        let output = utils::success_output("test", json!({}));
        let routes = router.determine_routes(&output);

        assert!(routes.is_empty());
    }

    #[tokio::test]
    async fn test_send_notifications_channel_not_found() {
        let router = NotificationRouter::default();
        let output = utils::success_output("test", json!({}));
        let routes = vec!["nonexistent".to_string()];

        let results = router.send_notifications(&output, &routes).await.unwrap();

        assert_eq!(results.len(), 1);
        assert!(!results[0].success);
        assert_eq!(results[0].channel, "nonexistent");
        assert!(results[0]
            .error
            .as_ref()
            .unwrap()
            .contains("Channel 'nonexistent' not found"));
    }

    #[test]
    fn test_determine_priority() {
        let router = NotificationRouter::default();

        // Test error case
        let error_output = utils::error_output("test", "error");
        assert_eq!(
            router.determine_priority(&error_output),
            NotificationPriority::High
        );

        // Test trading tool
        let trading_output = utils::success_output("trading_bot", json!({}));
        assert_eq!(
            router.determine_priority(&trading_output),
            NotificationPriority::Normal
        );

        // Test swap tool
        let swap_output = utils::success_output("swap_tokens", json!({}));
        assert_eq!(
            router.determine_priority(&swap_output),
            NotificationPriority::Normal
        );

        // Test other tool
        let other_output = utils::success_output("get_balance", json!({}));
        assert_eq!(
            router.determine_priority(&other_output),
            NotificationPriority::Low
        );
    }

    #[test]
    fn test_routing_rule_new_and_matches() {
        let rule = RoutingRule::new(
            "test_rule",
            RoutingCondition::OnSuccess,
            vec!["console".to_string()],
        );

        assert_eq!(rule.name, "test_rule");
        assert_eq!(rule.channels, vec!["console".to_string()]);

        let success_output = utils::success_output("test", json!({}));
        let error_output = utils::error_output("test", "error");

        assert!(rule.matches(&success_output));
        assert!(!rule.matches(&error_output));
    }

    #[test]
    fn test_routing_condition_execution_time_over() {
        let mut output = utils::success_output("test", json!({}));
        output.execution_time_ms = 1000;

        let condition = RoutingCondition::ExecutionTimeOver(500);
        assert!(condition.matches(&output));

        let condition = RoutingCondition::ExecutionTimeOver(1500);
        assert!(!condition.matches(&output));
    }

    #[test]
    fn test_routing_condition_has_metadata() {
        let mut output = utils::success_output("test", json!({}));
        output
            .metadata
            .insert("priority".to_string(), "high".to_string());

        let condition = RoutingCondition::HasMetadata("priority".to_string());
        assert!(condition.matches(&output));

        let condition = RoutingCondition::HasMetadata("level".to_string());
        assert!(!condition.matches(&output));
    }

    #[test]
    fn test_routing_condition_and_empty() {
        let output = utils::success_output("test", json!({}));
        let condition = RoutingCondition::And(vec![]);
        assert!(condition.matches(&output));
    }

    #[test]
    fn test_routing_condition_or_empty() {
        let output = utils::success_output("test", json!({}));
        let condition = RoutingCondition::Or(vec![]);
        assert!(!condition.matches(&output));
    }

    #[test]
    fn test_discord_channel_new() {
        let channel = DiscordChannel::new("https://discord.com/api/webhooks/test");
        assert_eq!(channel.webhook_url, "https://discord.com/api/webhooks/test");
        assert!(channel.username.is_none());
        assert!(channel.avatar_url.is_none());
    }

    #[test]
    fn test_discord_channel_with_identity() {
        let channel = DiscordChannel::new("https://discord.com/api/webhooks/test")
            .with_identity("TestBot", Some("https://example.com/avatar.png"));

        assert_eq!(channel.username, Some("TestBot".to_string()));
        assert_eq!(
            channel.avatar_url,
            Some("https://example.com/avatar.png".to_string())
        );
    }

    #[test]
    fn test_discord_channel_with_identity_no_avatar() {
        let channel = DiscordChannel::new("https://discord.com/api/webhooks/test")
            .with_identity("TestBot", None);

        assert_eq!(channel.username, Some("TestBot".to_string()));
        assert!(channel.avatar_url.is_none());
    }

    #[test]
    fn test_discord_channel_name() {
        let channel = DiscordChannel::new("https://discord.com/api/webhooks/test");
        assert_eq!(channel.name(), "Discord");
    }

    #[test]
    fn test_discord_title_case() {
        let channel = DiscordChannel::new("https://discord.com/api/webhooks/test");

        assert_eq!(channel.title_case("hello_world"), "Hello World");
        assert_eq!(channel.title_case("test"), "Test");
        assert_eq!(channel.title_case(""), "");
        assert_eq!(channel.title_case("a"), "A");
        assert_eq!(channel.title_case("one_two_three"), "One Two Three");
    }

    #[test]
    fn test_discord_format_error_output() {
        let channel = DiscordChannel::new("https://discord.com/api/webhooks/test");
        let output = utils::error_output("test_tool", "Something went wrong");

        let formatted = channel.format_for_discord(&output);

        assert_eq!(formatted["embeds"][0]["color"], 0xff0000); // Red for error
        assert!(formatted["embeds"][0]["title"]
            .as_str()
            .unwrap()
            .contains("❌"));
    }

    #[test]
    fn test_discord_format_with_metadata() {
        let channel = DiscordChannel::new("https://discord.com/api/webhooks/test");
        let mut output = utils::success_output("test_tool", json!({"key": "value", "number": 42}));
        output.execution_time_ms = 1500;

        let formatted = channel.format_for_discord(&output);

        let embed = &formatted["embeds"][0];
        assert!(embed["fields"].is_array());
        assert!(embed["footer"]["text"].as_str().unwrap().contains("1500ms"));
    }

    #[test]
    fn test_discord_format_null_result() {
        let channel = DiscordChannel::new("https://discord.com/api/webhooks/test");
        let output = utils::success_output("test_tool", json!(null));

        let formatted = channel.format_for_discord(&output);

        let fields = formatted["embeds"][0]["fields"].as_array().unwrap();
        assert!(fields.is_empty());
    }

    #[test]
    fn test_telegram_channel_new() {
        let channel = TelegramChannel::new("bot_token", "chat_id");
        assert_eq!(channel.bot_token, "bot_token");
        assert_eq!(channel.chat_id, "chat_id");
        assert_eq!(channel.parse_mode, "Markdown");
    }

    #[test]
    fn test_telegram_channel_with_html_mode() {
        let channel = TelegramChannel::new("bot_token", "chat_id").with_html_mode();
        assert_eq!(channel.parse_mode, "HTML");
    }

    #[test]
    fn test_telegram_channel_name() {
        let channel = TelegramChannel::new("bot_token", "chat_id");
        assert_eq!(channel.name(), "Telegram");
    }

    #[test]
    fn test_telegram_escape_markdown() {
        let channel = TelegramChannel::new("bot_token", "chat_id");

        let text = "*bold* _italic_ [link](url) `code` > quote # header + plus - minus = equal | pipe { } . ! () ~tilde~";
        let escaped = channel.escape_markdown(text);

        assert!(escaped.contains(r"\*bold\*"));
        assert!(escaped.contains(r"\_italic\_"));
        assert!(escaped.contains(r"\[link\]"));
        assert!(escaped.contains(r"\`code\`"));
        assert!(escaped.contains(r"\> quote"));
        assert!(escaped.contains(r"\# header"));
        assert!(escaped.contains(r"\+ plus"));
        assert!(escaped.contains(r"\- minus"));
        assert!(escaped.contains(r"\= equal"));
        assert!(escaped.contains(r"\| pipe"));
        assert!(escaped.contains(r"\{") && escaped.contains(r"\}"));
        assert!(escaped.contains(r"\.") && escaped.contains(r"\!"));
        assert!(escaped.contains(r"\(") && escaped.contains(r"\)"));
        assert!(escaped.contains(r"\~tilde\~"));
    }

    #[test]
    fn test_telegram_format_success() {
        let channel = TelegramChannel::new("bot_token", "chat_id");
        let output = utils::success_output("test_tool", json!({"balance": "1.5 SOL"}));

        let formatted = channel.format_for_telegram(&output);

        assert!(formatted.contains("✅"));
        assert!(formatted.contains("test\\_tool"));
        assert!(formatted.contains("📊 *Result:*"));
        assert!(formatted.contains("```json"));
        assert!(formatted.contains("1.5 SOL"));
    }

    #[test]
    fn test_telegram_format_error() {
        let channel = TelegramChannel::new("bot_token", "chat_id");
        let output = utils::error_output("test_tool", "Connection failed");

        let formatted = channel.format_for_telegram(&output);

        assert!(formatted.contains("❌"));
        assert!(formatted.contains("test\\_tool"));
        assert!(formatted.contains("❌ *Error:*"));
        assert!(formatted.contains("Connection failed"));
    }

    #[test]
    fn test_telegram_format_with_execution_time() {
        let channel = TelegramChannel::new("bot_token", "chat_id");
        let mut output = utils::success_output("test_tool", json!({}));
        output.execution_time_ms = 2500;

        let formatted = channel.format_for_telegram(&output);

        assert!(formatted.contains("⏱️ Executed in 2500ms"));
    }

    #[test]
    fn test_telegram_format_null_result() {
        let channel = TelegramChannel::new("bot_token", "chat_id");
        let output = utils::success_output("test_tool", json!(null));

        let formatted = channel.format_for_telegram(&output);

        assert!(!formatted.contains("📊 *Result:*"));
    }

    #[test]
    fn test_webhook_channel_new() {
        let channel = WebhookChannel::new("test_webhook", "https://example.com/webhook");
        assert_eq!(channel.name, "test_webhook");
        assert_eq!(channel.url, "https://example.com/webhook");
        assert!(channel.headers.is_empty());
        assert_eq!(channel.format_template, "{\"message\": \"{{message}}\"}");
    }

    #[test]
    fn test_webhook_channel_with_header() {
        let channel = WebhookChannel::new("test", "https://example.com")
            .with_header("Authorization", "Bearer token")
            .with_header("X-API-Key", "secret");

        assert_eq!(
            channel.headers.get("Authorization"),
            Some(&"Bearer token".to_string())
        );
        assert_eq!(
            channel.headers.get("X-API-Key"),
            Some(&"secret".to_string())
        );
    }

    #[test]
    fn test_webhook_channel_with_format_template() {
        let template =
            r#"{"text": "{{message}}", "tool": "{{tool_name}}", "status": "{{status}}"}"#;
        let channel =
            WebhookChannel::new("test", "https://example.com").with_format_template(template);

        assert_eq!(channel.format_template, template);
    }

    #[test]
    fn test_webhook_channel_name() {
        let channel = WebhookChannel::new("my_webhook", "https://example.com");
        assert_eq!(channel.name(), "my_webhook");
    }

    #[test]
    fn test_webhook_format_message_success() {
        let channel = WebhookChannel::new("test", "https://example.com")
            .with_format_template(r#"{"message": "{{message}}", "tool": "{{tool_name}}", "status": "{{status}}", "result": "{{result}}"}"#);

        let output = utils::success_output("test_tool", json!({"balance": "1.5"}));
        let formatted = channel.format_message(&output);

        assert!(formatted.contains("\"tool\": \"test_tool\""));
        assert!(formatted.contains("\"status\": \"SUCCESS\""));
        assert!(formatted.contains("Operation completed successfully"));
        assert!(formatted.contains("\"balance\":\"1.5\""));
    }

    #[test]
    fn test_webhook_format_message_error() {
        let channel = WebhookChannel::new("test", "https://example.com");
        let output = utils::error_output("test_tool", "Connection timeout");
        let formatted = channel.format_message(&output);

        assert!(formatted.contains("FAILED"));
        assert!(formatted.contains("Connection timeout"));
    }

    #[test]
    fn test_webhook_format_message_error_without_message() {
        let channel = WebhookChannel::new("test", "https://example.com");
        let mut output = utils::success_output("test_tool", json!({}));
        output.success = false;
        output.error = None;

        let formatted = channel.format_message(&output);

        assert!(formatted.contains("FAILED"));
        assert!(formatted.contains("Unknown error"));
    }

    #[test]
    fn test_console_channel_new() {
        let channel = ConsoleChannel::default();
        assert!(channel.use_colors);
    }

    #[test]
    fn test_console_channel_default() {
        let channel = ConsoleChannel::default();
        assert!(channel.use_colors);
    }

    #[test]
    fn test_console_channel_without_colors() {
        let channel = ConsoleChannel::default().without_colors();
        assert!(!channel.use_colors);
    }

    #[test]
    fn test_console_channel_name() {
        let channel = ConsoleChannel::default();
        assert_eq!(channel.name(), "Console");
    }

    #[tokio::test]
    async fn test_console_channel_send_notification_success_with_colors() {
        let channel = ConsoleChannel::default();
        let output = utils::success_output("test_tool", json!({"result": "success"}));

        let result = channel.send_notification(&output).await.unwrap();
        assert!(result.starts_with("console_"));
    }

    #[tokio::test]
    async fn test_console_channel_send_notification_error_with_colors() {
        let channel = ConsoleChannel::default();
        let output = utils::error_output("test_tool", "Something went wrong");

        let result = channel.send_notification(&output).await.unwrap();
        assert!(result.starts_with("console_"));
    }

    #[tokio::test]
    async fn test_console_channel_send_notification_success_without_colors() {
        let channel = ConsoleChannel::default().without_colors();
        let output = utils::success_output("test_tool", json!({"result": "success"}));

        let result = channel.send_notification(&output).await.unwrap();
        assert!(result.starts_with("console_"));
    }

    #[tokio::test]
    async fn test_console_channel_send_notification_error_without_colors() {
        let channel = ConsoleChannel::default().without_colors();
        let output = utils::error_output("test_tool", "Something went wrong");

        let result = channel.send_notification(&output).await.unwrap();
        assert!(result.starts_with("console_"));
    }

    #[tokio::test]
    async fn test_console_channel_send_notification_null_result() {
        let channel = ConsoleChannel::default();
        let output = utils::success_output("test_tool", json!(null));

        let result = channel.send_notification(&output).await.unwrap();
        assert!(result.starts_with("console_"));
    }

    #[test]
    fn test_notification_channel_supports_formatting_default() {
        let channel = ConsoleChannel::default();
        assert!(channel.supports_formatting());
    }

    #[test]
    fn test_notification_result_serialization() {
        let result = NotificationResult {
            channel: "test".to_string(),
            success: true,
            message_id: Some("msg_123".to_string()),
            error: None,
        };

        let serialized = serde_json::to_string(&result).unwrap();
        let deserialized: NotificationResult = serde_json::from_str(&serialized).unwrap();

        assert_eq!(result.channel, deserialized.channel);
        assert_eq!(result.success, deserialized.success);
        assert_eq!(result.message_id, deserialized.message_id);
        assert_eq!(result.error, deserialized.error);
    }

    #[tokio::test]
    async fn test_notification_router_process_complete_flow() {
        let router = NotificationRouter::default()
            .add_channel("console", ConsoleChannel::default())
            .add_channel(
                "webhook",
                WebhookChannel::new("test", "https://httpbin.org/post"),
            )
            .set_default_channel("console")
            .add_routing_rule(RoutingRule::new(
                "success_to_all",
                RoutingCondition::OnSuccess,
                vec!["console".to_string(), "webhook".to_string()],
            ));

        let output = utils::success_output("test_tool", json!({"result": "test"}));
        let processed = router.process(output).await.unwrap();

        assert_eq!(processed.format, OutputFormat::Json);
        assert!(processed.summary.is_some());
        assert!(processed.routing_info.is_some());
        assert_eq!(
            processed.routing_info.as_ref().unwrap().priority,
            NotificationPriority::Low
        );

        let results = processed.processed_result["notification_results"]
            .as_array()
            .unwrap();
        assert_eq!(results.len(), 2);
    }
}
