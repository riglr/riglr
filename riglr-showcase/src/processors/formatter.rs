//! Output formatting patterns
//!
//! This module demonstrates how to transform tool outputs into different formats
//! like Markdown, HTML, JSON, and custom formats for different presentation needs.

use super::{OutputProcessor, ToolOutput, ProcessedOutput, OutputFormat};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Markdown formatter for tool outputs
///
/// Converts tool outputs into clean, readable Markdown format.
/// Useful for documentation, reports, or display in Markdown-aware interfaces.
pub struct MarkdownFormatter {
    include_metadata: bool,
    include_timing: bool,
    custom_templates: HashMap<String, String>,
}

impl MarkdownFormatter {
    pub fn new() -> Self {
        Self {
            include_metadata: true,
            include_timing: true,
            custom_templates: HashMap::new(),
        }
    }
    
    pub fn with_options(include_metadata: bool, include_timing: bool) -> Self {
        Self {
            include_metadata,
            include_timing,
            custom_templates: HashMap::new(),
        }
    }
    
    /// Add a custom template for specific tool types
    pub fn with_template(mut self, tool_name: &str, template: &str) -> Self {
        self.custom_templates.insert(tool_name.to_string(), template.to_string());
        self
    }
    
    /// Format the output as Markdown
    fn format_as_markdown(&self, output: &ToolOutput) -> String {
        // Check for custom template first
        if let Some(template) = self.custom_templates.get(&output.tool_name) {
            return self.apply_template(template, output);
        }
        
        let mut markdown = String::new();
        
        // Title
        markdown.push_str(&format!("## {} Results\n\n", self.title_case(&output.tool_name)));
        
        // Status indicator
        let status_emoji = if output.success { "✅" } else { "❌" };
        let status_text = if output.success { "Success" } else { "Failed" };
        markdown.push_str(&format!("**Status:** {} {}\n\n", status_emoji, status_text));
        
        // Error message if present
        if let Some(error) = &output.error {
            markdown.push_str("### Error Details\n\n");
            markdown.push_str(&format!("```\n{}\n```\n\n", error));
        }
        
        // Main result
        if !output.result.is_null() {
            markdown.push_str("### Result\n\n");
            markdown.push_str(&self.format_json_as_markdown(&output.result));
            markdown.push('\n');
        }
        
        // Metadata section
        if self.include_metadata && !output.metadata.is_empty() {
            markdown.push_str("### Metadata\n\n");
            for (key, value) in &output.metadata {
                markdown.push_str(&format!("- **{}:** {}\n", self.title_case(key), value));
            }
            markdown.push('\n');
        }
        
        // Timing information
        if self.include_timing && output.execution_time_ms > 0 {
            markdown.push_str(&format!(
                "---\n*Executed in {}ms*\n",
                output.execution_time_ms
            ));
        }
        
        markdown
    }
    
    /// Apply a custom template to the output
    fn apply_template(&self, template: &str, output: &ToolOutput) -> String {
        template
            .replace("{tool_name}", &output.tool_name)
            .replace("{status}", if output.success { "✅ Success" } else { "❌ Failed" })
            .replace("{result}", &format!("```json\n{}\n```", serde_json::to_string_pretty(&output.result).unwrap_or_default()))
            .replace("{error}", &output.error.clone().unwrap_or_default())
            .replace("{execution_time}", &output.execution_time_ms.to_string())
    }
    
    /// Format JSON as readable Markdown
    fn format_json_as_markdown(&self, value: &Value) -> String {
        match value {
            Value::Object(obj) => {
                let mut result = String::new();
                for (key, val) in obj {
                    match val {
                        Value::String(s) => result.push_str(&format!("- **{}:** {}\n", self.title_case(key), s)),
                        Value::Number(n) => result.push_str(&format!("- **{}:** `{}`\n", self.title_case(key), n)),
                        Value::Bool(b) => result.push_str(&format!("- **{}:** {}\n", self.title_case(key), if *b { "✅ Yes" } else { "❌ No" })),
                        _ => result.push_str(&format!("- **{}:** `{}`\n", self.title_case(key), val)),
                    }
                }
                result
            },
            _ => format!("```json\n{}\n```\n", serde_json::to_string_pretty(value).unwrap_or_default())
        }
    }
    
    /// Convert snake_case to Title Case
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
impl OutputProcessor for MarkdownFormatter {
    async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput> {
        let markdown_content = self.format_as_markdown(&input);
        
        Ok(ProcessedOutput {
            original: input.clone(),
            processed_result: json!({"markdown": markdown_content}),
            format: OutputFormat::Markdown,
            summary: None,
            routing_info: None,
        })
    }
    
    fn name(&self) -> &str {
        "MarkdownFormatter"
    }
    
    fn config(&self) -> serde_json::Value {
        json!({
            "name": self.name(),
            "type": "formatter",
            "format": "markdown",
            "include_metadata": self.include_metadata,
            "include_timing": self.include_timing,
            "custom_templates": self.custom_templates.len()
        })
    }
}

/// HTML formatter for tool outputs
pub struct HtmlFormatter {
    css_classes: HashMap<String, String>,
    include_styles: bool,
}

impl HtmlFormatter {
    pub fn new() -> Self {
        let mut css_classes = HashMap::new();
        css_classes.insert("container".to_string(), "tool-output".to_string());
        css_classes.insert("success".to_string(), "status-success".to_string());
        css_classes.insert("error".to_string(), "status-error".to_string());
        css_classes.insert("result".to_string(), "result-content".to_string());
        
        Self {
            css_classes,
            include_styles: true,
        }
    }
    
    pub fn with_css_classes(mut self, classes: HashMap<String, String>) -> Self {
        self.css_classes.extend(classes);
        self
    }
    
    pub fn without_styles(mut self) -> Self {
        self.include_styles = false;
        self
    }
    
    fn format_as_html(&self, output: &ToolOutput) -> String {
        let mut html = String::new();
        
        // Add basic styles if requested
        if self.include_styles {
            html.push_str("<style>\n");
            html.push_str(".tool-output { font-family: -apple-system, BlinkMacSystemFont, sans-serif; max-width: 800px; margin: 20px auto; padding: 20px; border: 1px solid #e0e0e0; border-radius: 8px; }\n");
            html.push_str(".status-success { color: #4CAF50; }\n");
            html.push_str(".status-error { color: #F44336; }\n");
            html.push_str(".result-content { background: #f5f5f5; padding: 15px; border-radius: 4px; margin: 10px 0; }\n");
            html.push_str("pre { white-space: pre-wrap; }\n");
            html.push_str("</style>\n\n");
        }
        
        // Container
        html.push_str(&format!(r#"<div class="{}">"#, self.css_classes.get("container").unwrap_or(&"tool-output".to_string())));
        html.push('\n');
        
        // Title
        html.push_str(&format!("<h2>{} Results</h2>\n", self.title_case(&output.tool_name)));
        
        // Status
        let default_success = "status-success".to_string();
        let default_error = "status-error".to_string();
        let status_class = if output.success { 
            self.css_classes.get("success").unwrap_or(&default_success) 
        } else { 
            self.css_classes.get("error").unwrap_or(&default_error) 
        };
        let status_text = if output.success { "✅ Success" } else { "❌ Failed" };
        html.push_str(&format!(r#"<p class="{}"><strong>Status:</strong> {}</p>"#, status_class, status_text));
        html.push('\n');
        
        // Error details
        if let Some(error) = &output.error {
            html.push_str("<h3>Error Details</h3>\n");
            html.push_str(&format!("<pre>{}</pre>\n", html_escape(error)));
        }
        
        // Result
        if !output.result.is_null() {
            html.push_str("<h3>Result</h3>\n");
            html.push_str(&format!(
                r#"<div class="{}"><pre>{}</pre></div>"#,
                self.css_classes.get("result").unwrap_or(&"result-content".to_string()),
                html_escape(&serde_json::to_string_pretty(&output.result).unwrap_or_default())
            ));
            html.push('\n');
        }
        
        // Metadata
        if !output.metadata.is_empty() {
            html.push_str("<h3>Metadata</h3>\n<ul>\n");
            for (key, value) in &output.metadata {
                html.push_str(&format!("<li><strong>{}:</strong> {}</li>\n", 
                    html_escape(&self.title_case(key)), 
                    html_escape(value)
                ));
            }
            html.push_str("</ul>\n");
        }
        
        // Timing
        if output.execution_time_ms > 0 {
            html.push_str(&format!("<hr><em>Executed in {}ms</em>\n", output.execution_time_ms));
        }
        
        html.push_str("</div>\n");
        html
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
impl OutputProcessor for HtmlFormatter {
    async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput> {
        let html_content = self.format_as_html(&input);
        
        Ok(ProcessedOutput {
            original: input.clone(),
            processed_result: json!({"html": html_content}),
            format: OutputFormat::Html,
            summary: None,
            routing_info: None,
        })
    }
    
    fn name(&self) -> &str {
        "HtmlFormatter"
    }
    
    fn config(&self) -> serde_json::Value {
        json!({
            "name": self.name(),
            "type": "formatter",
            "format": "html",
            "include_styles": self.include_styles,
            "css_classes": self.css_classes.len()
        })
    }
}

/// JSON formatter that can restructure and clean up outputs
pub struct JsonFormatter {
    pretty_print: bool,
    include_metadata: bool,
    field_mappings: HashMap<String, String>,
}

impl JsonFormatter {
    pub fn new() -> Self {
        Self {
            pretty_print: true,
            include_metadata: true,
            field_mappings: HashMap::new(),
        }
    }
    
    pub fn compact(mut self) -> Self {
        self.pretty_print = false;
        self
    }
    
    pub fn without_metadata(mut self) -> Self {
        self.include_metadata = false;
        self
    }
    
    pub fn with_field_mapping(mut self, from: &str, to: &str) -> Self {
        self.field_mappings.insert(from.to_string(), to.to_string());
        self
    }
    
    fn format_as_json(&self, output: &ToolOutput) -> Value {
        let mut result = json!({
            "tool": output.tool_name,
            "success": output.success,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        if let Some(error) = &output.error {
            result["error"] = json!(error);
        }
        
        if !output.result.is_null() {
            result["data"] = self.remap_fields(&output.result);
        }
        
        if self.include_metadata && !output.metadata.is_empty() {
            result["metadata"] = json!(output.metadata);
        }
        
        if output.execution_time_ms > 0 {
            result["execution_time_ms"] = json!(output.execution_time_ms);
        }
        
        result
    }
    
    fn remap_fields(&self, value: &Value) -> Value {
        match value {
            Value::Object(obj) => {
                let mut new_obj = serde_json::Map::new();
                for (key, val) in obj {
                    let new_key = self.field_mappings.get(key).unwrap_or(key);
                    new_obj.insert(new_key.clone(), self.remap_fields(val));
                }
                Value::Object(new_obj)
            },
            Value::Array(arr) => {
                Value::Array(arr.iter().map(|v| self.remap_fields(v)).collect())
            },
            _ => value.clone()
        }
    }
}

#[async_trait]
impl OutputProcessor for JsonFormatter {
    async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput> {
        let formatted_json = self.format_as_json(&input);
        
        let json_string = if self.pretty_print {
            serde_json::to_string_pretty(&formatted_json)?
        } else {
            serde_json::to_string(&formatted_json)?
        };
        
        Ok(ProcessedOutput {
            original: input.clone(),
            processed_result: json!({"json": json_string, "structured": formatted_json}),
            format: OutputFormat::Json,
            summary: None,
            routing_info: None,
        })
    }
    
    fn name(&self) -> &str {
        "JsonFormatter"
    }
    
    fn config(&self) -> serde_json::Value {
        json!({
            "name": self.name(),
            "type": "formatter",
            "format": "json",
            "pretty_print": self.pretty_print,
            "include_metadata": self.include_metadata,
            "field_mappings": self.field_mappings.len()
        })
    }
}

/// Multi-format processor that can output in multiple formats simultaneously
pub struct MultiFormatProcessor {
    formats: Vec<Box<dyn OutputProcessor>>,
}

impl MultiFormatProcessor {
    pub fn new() -> Self {
        Self {
            formats: Vec::new(),
        }
    }
    
    pub fn add_format<F: OutputProcessor + 'static>(mut self, formatter: F) -> Self {
        self.formats.push(Box::new(formatter));
        self
    }
    
    pub fn standard_formats() -> Self {
        Self::new()
            .add_format(MarkdownFormatter::new())
            .add_format(HtmlFormatter::new())
            .add_format(JsonFormatter::new())
    }
}

#[async_trait]
impl OutputProcessor for MultiFormatProcessor {
    async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput> {
        let mut combined_result = json!({});
        let mut formats = Vec::new();
        
        for formatter in &self.formats {
            let formatted = formatter.process(input.clone()).await?;
            
            // Extract the formatted content and add to combined result
            if let Value::Object(obj) = &formatted.processed_result {
                for (key, value) in obj {
                    combined_result[key] = value.clone();
                }
            }
            
            // Track what formats we generated
            formats.push(format!("{:?}", formatted.format));
        }
        
        Ok(ProcessedOutput {
            original: input.clone(),
            processed_result: combined_result,
            format: OutputFormat::Custom("multi".to_string()),
            summary: Some(format!("Generated formats: {}", formats.join(", "))),
            routing_info: None,
        })
    }
    
    fn name(&self) -> &str {
        "MultiFormatProcessor"
    }
    
    fn config(&self) -> serde_json::Value {
        json!({
            "name": self.name(),
            "type": "multi_formatter",
            "formatters": self.formats.iter().map(|f| f.config()).collect::<Vec<_>>()
        })
    }
}

// Helper function for HTML escaping
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

impl Default for MarkdownFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for HtmlFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MultiFormatProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::utils;
    
    #[tokio::test]
    async fn test_markdown_formatter() {
        let formatter = MarkdownFormatter::new();
        let output = utils::success_output(
            "get_balance",
            json!({"balance_sol": 1.5, "address": "11111111111111111111111111111112"})
        );
        
        let processed = formatter.process(output).await.unwrap();
        
        assert!(matches!(processed.format, OutputFormat::Markdown));
        if let Some(markdown) = processed.processed_result.get("markdown") {
            let content = markdown.as_str().unwrap();
            assert!(content.contains("## Get Balance Results"));
            assert!(content.contains("✅ Success"));
            assert!(content.contains("balance_sol"));
        } else {
            panic!("Expected markdown content in processed result");
        }
    }
    
    #[tokio::test]
    async fn test_html_formatter() {
        let formatter = HtmlFormatter::new();
        let output = utils::error_output("test_tool", "Connection failed");
        
        let processed = formatter.process(output).await.unwrap();
        
        assert!(matches!(processed.format, OutputFormat::Html));
        if let Some(html) = processed.processed_result.get("html") {
            let content = html.as_str().unwrap();
            assert!(content.contains("<div"));
            assert!(content.contains("❌ Failed"));
            assert!(content.contains("Connection failed"));
        } else {
            panic!("Expected HTML content in processed result");
        }
    }
    
    #[tokio::test]
    async fn test_json_formatter() {
        let formatter = JsonFormatter::new()
            .with_field_mapping("balance_sol", "balance_solana");
            
        let output = utils::success_output(
            "get_balance",
            json!({"balance_sol": 1.5})
        );
        
        let processed = formatter.process(output).await.unwrap();
        
        assert!(matches!(processed.format, OutputFormat::Json));
        if let Some(structured) = processed.processed_result.get("structured") {
            assert!(structured["data"]["balance_solana"].is_number());
            assert_eq!(structured["data"]["balance_solana"], 1.5);
            assert!(structured["success"].as_bool().unwrap());
        } else {
            panic!("Expected structured JSON in processed result");
        }
    }
    
    #[tokio::test]
    async fn test_multi_format_processor() {
        let processor = MultiFormatProcessor::new()
            .add_format(MarkdownFormatter::new())
            .add_format(JsonFormatter::new());
            
        let output = utils::success_output("test", json!({"key": "value"}));
        let processed = processor.process(output).await.unwrap();
        
        assert!(matches!(processed.format, OutputFormat::Custom(_)));
        assert!(processed.processed_result.get("markdown").is_some());
        assert!(processed.processed_result.get("json").is_some());
        assert!(processed.summary.is_some());
    }
    
    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>alert('xss')</script>"), "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;");
        assert_eq!(html_escape("Safe text"), "Safe text");
        assert_eq!(html_escape("Quotes \"test\" & ampersand"), "Quotes &quot;test&quot; &amp; ampersand");
    }
}