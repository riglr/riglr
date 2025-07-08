//! Notification routing patterns for tool outputs
//!
//! This module demonstrates how to route tool outputs to different notification
//! channels like Discord, Telegram, Slack, email, and custom webhooks.

use super::{
    OutputProcessor, ToolOutput, ProcessedOutput, OutputFormat, 
    NotificationPriority, RoutingInfo
};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;

/// Notification router that sends outputs to various channels
pub struct NotificationRouter {
    channels: HashMap<String, Box<dyn NotificationChannel>>,
    routing_rules: Vec<RoutingRule>,
    default_channel: Option<String>,
}

impl NotificationRouter {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            routing_rules: Vec::new(),
            default_channel: None,
        }
    }
    
    pub fn add_channel<C: NotificationChannel + 'static>(
        mut self, 
        name: &str, 
        channel: C
    ) -> Self {
        self.channels.insert(name.to_string(), Box::new(channel));
        self
    }
    
    pub fn set_default_channel(mut self, name: &str) -> Self {
        self.default_channel = Some(name.to_string());
        self
    }
    
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
    async fn send_notifications(&self, output: &ToolOutput, routes: &[String]) -> Result<Vec<NotificationResult>> {
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
                    }
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
    pub name: String,
    pub condition: RoutingCondition,
    pub channels: Vec<String>,
}

impl RoutingRule {
    pub fn new(name: &str, condition: RoutingCondition, channels: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            condition,
            channels,
        }
    }
    
    pub fn matches(&self, output: &ToolOutput) -> bool {
        self.condition.matches(output)
    }
}

/// Conditions for routing decisions
#[derive(Clone)]
pub enum RoutingCondition {
    Always,
    OnSuccess,
    OnError,
    ToolName(String),
    ToolNameContains(String),
    ExecutionTimeOver(u64), // milliseconds
    HasMetadata(String),
    And(Vec<RoutingCondition>),
    Or(Vec<RoutingCondition>),
    Not(Box<RoutingCondition>),
}

impl RoutingCondition {
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
    pub channel: String,
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
}

/// Trait for notification channels
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    async fn send_notification(&self, output: &ToolOutput) -> Result<String>;
    fn name(&self) -> &str;
    fn supports_formatting(&self) -> bool { true }
}

/// Discord webhook notification channel
pub struct DiscordChannel {
    webhook_url: String,
    username: Option<String>,
    avatar_url: Option<String>,
}

impl DiscordChannel {
    pub fn new(webhook_url: &str) -> Self {
        Self {
            webhook_url: webhook_url.to_string(),
            username: None,
            avatar_url: None,
        }
    }
    
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
                    None => String::new(),
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
        let _payload = self.format_for_discord(output);
        
        // In a real implementation, you would use reqwest to send the webhook
        /*
        let client = reqwest::Client::new();
        let response = client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(format!("discord_msg_{}", chrono::Utc::now().timestamp()))
        } else {
            Err(anyhow!("Discord webhook failed: {}", response.status()))
        }
        */
        
        // For demo purposes, return a mock message ID
        Ok(format!("discord_msg_{}", chrono::Utc::now().timestamp()))
    }
    
    fn name(&self) -> &str {
        "Discord"
    }
}

/// Telegram bot notification channel
pub struct TelegramChannel {
    bot_token: String,
    chat_id: String,
    parse_mode: String,
}

impl TelegramChannel {
    pub fn new(bot_token: &str, chat_id: &str) -> Self {
        Self {
            bot_token: bot_token.to_string(),
            chat_id: chat_id.to_string(),
            parse_mode: "Markdown".to_string(),
        }
    }
    
    pub fn with_html_mode(mut self) -> Self {
        self.parse_mode = "HTML".to_string();
        self
    }
    
    fn format_for_telegram(&self, output: &ToolOutput) -> String {
        let status_emoji = if output.success { "✅" } else { "❌" };
        let mut message = format!("{} *{}*\n", status_emoji, self.escape_markdown(&output.tool_name));
        
        if !output.result.is_null() {
            message.push_str("\n📊 *Result:*\n");
            message.push_str(&format!("```json\n{}\n```\n", 
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
        let _message = self.format_for_telegram(output);
        
        // In a real implementation:
        /*
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        let payload = json!({
            "chat_id": self.chat_id,
            "text": message,
            "parse_mode": self.parse_mode
        });
        
        let client = reqwest::Client::new();
        let response = client.post(&url).json(&payload).send().await?;
        
        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["result"]["message_id"].to_string())
        } else {
            Err(anyhow!("Telegram API failed: {}", response.status()))
        }
        */
        
        // For demo purposes, return a mock message ID
        Ok(format!("tg_msg_{}", chrono::Utc::now().timestamp()))
    }
    
    fn name(&self) -> &str {
        "Telegram"
    }
}

/// Generic webhook notification channel
pub struct WebhookChannel {
    name: String,
    url: String,
    headers: HashMap<String, String>,
    format_template: String,
}

impl WebhookChannel {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            headers: HashMap::new(),
            format_template: "{\"message\": \"{{message}}\"}".to_string(),
        }
    }
    
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
    
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
        let summary = format!(
            "Tool '{}' {}: {}",
            output.tool_name,
            status,
            error_msg
        );
        
        // Simple template replacement
        self.format_template
            .replace("{{message}}", &summary)
            .replace("{{tool_name}}", &output.tool_name)
            .replace("{{status}}", status)
            .replace("{{result}}", &serde_json::to_string(&output.result).unwrap_or_default())
    }
}

#[async_trait]
impl NotificationChannel for WebhookChannel {
    async fn send_notification(&self, output: &ToolOutput) -> Result<String> {
        let _message = self.format_message(output);
        
        // In a real implementation:
        /*
        let client = reqwest::Client::new();
        let mut request = client.post(&self.url);
        
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }
        
        let response = request.body(message).send().await?;
        
        if response.status().is_success() {
            Ok(format!("webhook_{}", chrono::Utc::now().timestamp()))
        } else {
            Err(anyhow!("Webhook failed: {}", response.status()))
        }
        */
        
        // For demo purposes, return a mock ID
        Ok(format!("webhook_{}_{}", self.name, chrono::Utc::now().timestamp()))
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Console/log notification channel for debugging
pub struct ConsoleChannel {
    use_colors: bool,
}

impl ConsoleChannel {
    pub fn new() -> Self {
        Self { use_colors: true }
    }
    
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
                println!("\x1b[32m✅ [{}] {} completed successfully\x1b[0m", timestamp, output.tool_name);
                if !output.result.is_null() {
                    println!("\x1b[36mResult: {}\x1b[0m", serde_json::to_string_pretty(&output.result)?);
                }
            } else {
                println!("\x1b[31m❌ [{}] {} failed\x1b[0m", timestamp, output.tool_name);
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

impl Default for NotificationRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ConsoleChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::utils;
    
    #[tokio::test]
    async fn test_notification_router() {
        let router = NotificationRouter::new()
            .add_channel("console", ConsoleChannel::new())
            .set_default_channel("console")
            .add_routing_rule(
                RoutingRule::new(
                    "error_routing",
                    RoutingCondition::OnError,
                    vec!["console".to_string()]
                )
            );
        
        let output = utils::error_output("test_tool", "Something went wrong");
        let processed = router.process(output).await.unwrap();
        
        assert!(processed.routing_info.is_some());
        assert!(processed.summary.is_some());
        
        let results = processed.processed_result["notification_results"].as_array().unwrap();
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
            RoutingCondition::ToolNameContains("swap".to_string())
        ]);
        
        assert!(condition.matches(&output));
        
        let condition = RoutingCondition::Or(vec![
            RoutingCondition::OnError,
            RoutingCondition::ToolNameContains("balance".to_string())
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
            json!({"balance": "1.5 SOL", "address": "11111111111111111111111111111112"})
        );
        
        let formatted = channel.format_for_discord(&output);
        
        assert!(formatted["embeds"].is_array());
        assert!(formatted["username"] == "RiglrBot");
        assert!(formatted["avatar_url"] == "https://example.com/avatar.png");
        
        let embed = &formatted["embeds"][0];
        assert!(embed["title"].as_str().unwrap().contains("Get Balance Results"));
        assert!(embed["color"] == 0x00ff00); // Green for success
    }
    
    #[tokio::test]
    async fn test_telegram_formatting() {
        let channel = TelegramChannel::new("bot_token", "chat_id");
        let output = utils::error_output("trading_bot", "Insufficient balance");
        
        let formatted = channel.format_for_telegram(&output);
        
        assert!(formatted.contains("❌"));
        assert!(formatted.contains("trading_bot"));
        assert!(formatted.contains("Insufficient balance"));
        assert!(formatted.contains("```"));
    }
    
    #[tokio::test]
    async fn test_webhook_channel() {
        let channel = WebhookChannel::new("custom", "https://example.com/webhook")
            .with_header("Authorization", "Bearer token")
            .with_format_template(r#"{"text": "{{message}}", "tool": "{{tool_name}}"}"#);
        
        let output = utils::success_output("test", json!({}));
        let result = channel.send_notification(&output).await.unwrap();
        
        assert!(result.starts_with("webhook_custom_"));
    }
}