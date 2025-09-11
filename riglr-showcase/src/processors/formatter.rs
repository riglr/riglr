//! Output formatting patterns
//!
//! This module demonstrates how to transform tool outputs into different formats
//! like Markdown, HTML, JSON, and custom formats for different presentation needs.

use super::{OutputFormat, OutputProcessor, ProcessedOutput, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Markdown formatter for tool outputs
///
/// Converts tool outputs into clean, readable Markdown format.
/// Useful for documentation, reports, or display in Markdown-aware interfaces.
#[derive(Debug)]
pub struct MarkdownFormatter {
    include_metadata: bool,
    include_timing: bool,
    custom_templates: HashMap<String, String>,
}

impl MarkdownFormatter {
    /// Create a new MarkdownFormatter with default settings
    ///
    /// This is provided for API consistency. You can also use `MarkdownFormatter::default()`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a MarkdownFormatter with custom metadata and timing options
    pub fn with_options(include_metadata: bool, include_timing: bool) -> Self {
        Self {
            include_metadata,
            include_timing,
            custom_templates: HashMap::default(),
        }
    }

    /// Add a custom template for specific tool types
    pub fn with_template(mut self, tool_name: &str, template: &str) -> Self {
        self.custom_templates
            .insert(tool_name.to_string(), template.to_string());
        self
    }

    /// Format the output as Markdown
    fn format_as_markdown(&self, output: &ToolOutput) -> String {
        // Check for custom template first
        if let Some(template) = self.custom_templates.get(&output.tool_name) {
            return self.apply_template(template, output);
        }

        let mut markdown = String::default();

        // Title
        markdown.push_str(&format!(
            "## {} Results\n\n",
            self.title_case(&output.tool_name)
        ));

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
            .replace(
                "{status}",
                if output.success {
                    "✅ Success"
                } else {
                    "❌ Failed"
                },
            )
            .replace(
                "{result}",
                &format!(
                    "```json\n{}\n```",
                    serde_json::to_string_pretty(&output.result).unwrap_or_default()
                ),
            )
            .replace("{error}", output.error.as_deref().unwrap_or_default())
            .replace("{execution_time}", &output.execution_time_ms.to_string())
    }

    /// Format JSON as readable Markdown
    fn format_json_as_markdown(&self, value: &Value) -> String {
        match value {
            Value::Object(obj) => {
                let mut result = String::default();
                for (key, val) in obj {
                    match val {
                        Value::String(s) => {
                            result.push_str(&format!("- **{}:** {}\n", self.title_case(key), s))
                        }
                        Value::Number(n) => {
                            result.push_str(&format!("- **{}:** `{}`\n", self.title_case(key), n))
                        }
                        Value::Bool(b) => result.push_str(&format!(
                            "- **{}:** {}\n",
                            self.title_case(key),
                            if *b { "✅ Yes" } else { "❌ No" }
                        )),
                        _ => {
                            result.push_str(&format!("- **{}:** `{}`\n", self.title_case(key), val))
                        }
                    }
                }
                result
            }
            _ => format!(
                "```json\n{}\n```\n",
                serde_json::to_string_pretty(value).unwrap_or_default()
            ),
        }
    }

    /// Convert snake_case to Title Case
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
impl OutputProcessor for MarkdownFormatter {
    async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput> {
        let markdown_content = self.format_as_markdown(&input);

        Ok(ProcessedOutput {
            original: input,
            processed_result: json!({"markdown": markdown_content}),
            format: OutputFormat::Markdown,
            summary: None, // Formatters typically don't generate summaries
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
#[derive(Debug)]
pub struct HtmlFormatter {
    css_classes: HashMap<String, String>,
    include_styles: bool,
}

impl HtmlFormatter {
    /// Create a new HtmlFormatter with default settings
    ///
    /// This is provided for API consistency. You can also use `HtmlFormatter::default()`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add custom CSS classes, preserving existing defaults
    pub fn with_css_classes(mut self, classes: HashMap<String, String>) -> Self {
        for (key, value) in classes {
            self.css_classes.insert(key, value);
        }
        self
    }

    /// Disable inline CSS styles in output
    pub fn without_styles(mut self) -> Self {
        self.include_styles = false;
        self
    }

    /// Format the output as HTML
    fn format_as_html(&self, output: &ToolOutput) -> String {
        let mut html = String::default();

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
        let container_class = self
            .css_classes
            .get("container")
            .map_or("tool-output", String::as_str);
        html.push_str(&format!(r#"<div class="{}">"#, container_class));
        html.push('\n');

        // Title
        html.push_str(&format!(
            "<h2>{} Results</h2>\n",
            self.title_case(&output.tool_name)
        ));

        // Status
        let default_success = "status-success".to_string();
        let default_error = "status-error".to_string();
        let status_class = if output.success {
            self.css_classes.get("success").unwrap_or(&default_success)
        } else {
            self.css_classes.get("error").unwrap_or(&default_error)
        };
        let status_text = if output.success {
            "✅ Success"
        } else {
            "❌ Failed"
        };
        html.push_str(&format!(
            r#"<p class="{}"><strong>Status:</strong> {}</p>"#,
            status_class, status_text
        ));
        html.push('\n');

        // Error details
        if let Some(error) = &output.error {
            html.push_str("<h3>Error Details</h3>\n");
            html.push_str(&format!("<pre>{}</pre>\n", html_escape(error)));
        }

        // Result
        if !output.result.is_null() {
            html.push_str("<h3>Result</h3>\n");
            let result_class = self
                .css_classes
                .get("result")
                .map_or("result-content", String::as_str);
            html.push_str(&format!(
                r#"<div class="{}"><pre>{}</pre></div>"#,
                result_class,
                html_escape(&serde_json::to_string_pretty(&output.result).unwrap_or_default())
            ));
            html.push('\n');
        }

        // Metadata
        if !output.metadata.is_empty() {
            html.push_str("<h3>Metadata</h3>\n<ul>\n");
            for (key, value) in &output.metadata {
                html.push_str(&format!(
                    "<li><strong>{}:</strong> {}</li>\n",
                    html_escape(&self.title_case(key)),
                    html_escape(value)
                ));
            }
            html.push_str("</ul>\n");
        }

        // Timing
        if output.execution_time_ms > 0 {
            html.push_str(&format!(
                "<hr><em>Executed in {}ms</em>\n",
                output.execution_time_ms
            ));
        }

        html.push_str("</div>\n");
        html
    }

    /// Convert snake_case to Title Case
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
impl OutputProcessor for HtmlFormatter {
    async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput> {
        let html_content = self.format_as_html(&input);

        Ok(ProcessedOutput {
            original: input,
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
#[derive(Debug)]
pub struct JsonFormatter {
    pretty_print: bool,
    include_metadata: bool,
    field_mappings: HashMap<String, String>,
}

impl JsonFormatter {
    /// Create a new JsonFormatter with default settings
    ///
    /// This is provided for API consistency. You can also use `JsonFormatter::default()`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure formatter to output compact JSON
    pub fn compact(mut self) -> Self {
        self.pretty_print = false;
        self
    }

    /// Configure formatter to exclude metadata from output
    pub fn without_metadata(mut self) -> Self {
        self.include_metadata = false;
        self
    }

    /// Add a field name mapping for JSON transformation
    pub fn with_field_mapping(mut self, from: &str, to: &str) -> Self {
        self.field_mappings.insert(from.to_string(), to.to_string());
        self
    }

    /// Format the output as structured JSON
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

    /// Recursively remap field names according to configured mappings
    fn remap_fields(&self, value: &Value) -> Value {
        match value {
            Value::Object(obj) => {
                let mut new_obj = serde_json::Map::new();
                for (key, val) in obj {
                    let new_key = self.field_mappings.get(key).unwrap_or(key);
                    new_obj.insert(new_key.clone(), self.remap_fields(val));
                }
                Value::Object(new_obj)
            }
            Value::Array(arr) => Value::Array(arr.iter().map(|v| self.remap_fields(v)).collect()),
            _ => value.clone(),
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
            original: input,
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
#[derive(Default)]
pub struct MultiFormatProcessor {
    formats: Vec<Box<dyn OutputProcessor>>,
}

impl std::fmt::Debug for MultiFormatProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiFormatProcessor")
            .field("formats", &format!("{} formatters", self.formats.len()))
            .finish()
    }
}

impl MultiFormatProcessor {
    /// Create a new MultiFormatProcessor with no formats
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a formatter to the processor
    pub fn add_format<F: OutputProcessor + 'static>(mut self, formatter: F) -> Self {
        self.formats.push(Box::new(formatter));
        self
    }

    /// Create a MultiFormatProcessor with standard formatters (Markdown, HTML, JSON)
    pub fn standard_formats() -> Self {
        Self::new()
            .add_format(MarkdownFormatter::default())
            .add_format(HtmlFormatter::default())
            .add_format(JsonFormatter::default())
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
                    combined_result[key].clone_from(value);
                }
            }

            // Track what formats we generated
            formats.push(format!("{:?}", formatted.format));
        }

        Ok(ProcessedOutput {
            original: input,
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
        Self {
            include_metadata: true,
            include_timing: true,
            custom_templates: HashMap::default(),
        }
    }
}

impl Default for HtmlFormatter {
    fn default() -> Self {
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
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self {
            pretty_print: true,
            include_metadata: true,
            field_mappings: HashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::utils;

    #[tokio::test]
    async fn test_markdown_formatter() {
        let formatter = MarkdownFormatter::default();
        let output = utils::success_output(
            "get_balance",
            json!({"balance_sol": 1.5, "address": "11111111111111111111111111111112"}),
        );

        let processed = formatter.process(output).await.unwrap();

        assert!(matches!(processed.format, OutputFormat::Markdown));
        if let Some(markdown) = processed.processed_result.get("markdown") {
            let content = markdown.as_str().unwrap();
            assert!(content.contains("## Get Balance Results"));
            assert!(content.contains("✅ Success"));
            assert!(content.contains("Balance Sol"));
        } else {
            panic!("Expected markdown content in processed result");
        }
    }

    #[tokio::test]
    async fn test_html_formatter() {
        let formatter = HtmlFormatter::default();
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
        let formatter =
            JsonFormatter::default().with_field_mapping("balance_sol", "balance_solana");

        let output = utils::success_output("get_balance", json!({"balance_sol": 1.5}));

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
        let processor = MultiFormatProcessor::default()
            .add_format(MarkdownFormatter::default())
            .add_format(JsonFormatter::default());

        let output = utils::success_output("test", json!({"key": "value"}));
        let processed = processor.process(output).await.unwrap();

        assert!(matches!(processed.format, OutputFormat::Custom(_)));
        assert!(processed.processed_result.get("markdown").is_some());
        assert!(processed.processed_result.get("json").is_some());
        assert!(processed.summary.is_some());
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(
            html_escape("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
        assert_eq!(html_escape("Safe text"), "Safe text");
        assert_eq!(
            html_escape("Quotes \"test\" & ampersand"),
            "Quotes &quot;test&quot; &amp; ampersand"
        );
    }

    // Comprehensive tests for MarkdownFormatter
    #[test]
    fn test_markdown_formatter_new() {
        let formatter = MarkdownFormatter::default();
        assert!(formatter.include_metadata);
        assert!(formatter.include_timing);
        assert!(formatter.custom_templates.is_empty());
    }

    #[test]
    fn test_markdown_formatter_with_options() {
        let formatter = MarkdownFormatter::with_options(false, false);
        assert!(!formatter.include_metadata);
        assert!(!formatter.include_timing);
        assert!(formatter.custom_templates.is_empty());
    }

    #[test]
    fn test_markdown_formatter_with_template() {
        let formatter = MarkdownFormatter::default()
            .with_template("test_tool", "Custom template for {tool_name}");

        assert!(formatter.custom_templates.contains_key("test_tool"));
        assert_eq!(
            formatter.custom_templates.get("test_tool").unwrap(),
            "Custom template for {tool_name}"
        );
    }

    #[tokio::test]
    async fn test_markdown_formatter_with_custom_template() {
        let formatter = MarkdownFormatter::default()
            .with_template("test_tool", "## {tool_name}\nStatus: {status}\nResult: {result}\nError: {error}\nTime: {execution_time}ms");

        let output = utils::success_output("test_tool", json!({"key": "value"}));
        let processed = formatter.process(output).await.unwrap();

        if let Some(markdown) = processed.processed_result.get("markdown") {
            let content = markdown.as_str().unwrap();
            assert!(content.contains("## test_tool"));
            assert!(content.contains("Status: ✅ Success"));
            assert!(content.contains("Time: 0ms"));
        }
    }

    #[tokio::test]
    async fn test_markdown_formatter_with_error() {
        let formatter = MarkdownFormatter::default();
        let output = utils::error_output("test_tool", "Test error message");

        let processed = formatter.process(output).await.unwrap();

        if let Some(markdown) = processed.processed_result.get("markdown") {
            let content = markdown.as_str().unwrap();
            assert!(content.contains("❌ Failed"));
            assert!(content.contains("### Error Details"));
            assert!(content.contains("Test error message"));
        }
    }

    #[tokio::test]
    async fn test_markdown_formatter_without_metadata() {
        let formatter = MarkdownFormatter::with_options(false, true);
        let mut output = utils::success_output("test_tool", json!({"key": "value"}));
        output
            .metadata
            .insert("test_key".to_string(), "test_value".to_string());

        let processed = formatter.process(output).await.unwrap();

        if let Some(markdown) = processed.processed_result.get("markdown") {
            let content = markdown.as_str().unwrap();
            assert!(!content.contains("### Metadata"));
            assert!(!content.contains("test_key"));
        }
    }

    #[tokio::test]
    async fn test_markdown_formatter_without_timing() {
        let formatter = MarkdownFormatter::with_options(true, false);
        let mut output = utils::success_output("test_tool", json!({"key": "value"}));
        output.execution_time_ms = 100;

        let processed = formatter.process(output).await.unwrap();

        if let Some(markdown) = processed.processed_result.get("markdown") {
            let content = markdown.as_str().unwrap();
            assert!(!content.contains("Executed in"));
            assert!(!content.contains("100ms"));
        }
    }

    #[tokio::test]
    async fn test_markdown_formatter_with_null_result() {
        let formatter = MarkdownFormatter::default();
        let mut output = utils::success_output("test_tool", json!(null));
        output.result = serde_json::Value::Null;

        let processed = formatter.process(output).await.unwrap();

        if let Some(markdown) = processed.processed_result.get("markdown") {
            let content = markdown.as_str().unwrap();
            assert!(!content.contains("### Result"));
        }
    }

    #[test]
    fn test_markdown_formatter_title_case() {
        let formatter = MarkdownFormatter::default();
        assert_eq!(formatter.title_case("hello_world"), "Hello World");
        assert_eq!(formatter.title_case("single"), "Single");
        assert_eq!(formatter.title_case(""), "");
        assert_eq!(
            formatter.title_case("already_formatted_string"),
            "Already Formatted String"
        );
        assert_eq!(formatter.title_case("a"), "A");
    }

    #[test]
    fn test_markdown_formatter_format_json_as_markdown_object() {
        let formatter = MarkdownFormatter::default();
        let json_obj = json!({
            "string_field": "test_value",
            "number_field": 42,
            "bool_true": true,
            "bool_false": false,
            "complex_field": {"nested": "value"}
        });

        let result = formatter.format_json_as_markdown(&json_obj);
        assert!(result.contains("- **String Field:** test_value"));
        assert!(result.contains("- **Number Field:** `42`"));
        assert!(result.contains("- **Bool True:** ✅ Yes"));
        assert!(result.contains("- **Bool False:** ❌ No"));
        assert!(result.contains("- **Complex Field:** `{\"nested\":\"value\"}`"));
    }

    #[test]
    fn test_markdown_formatter_format_json_as_markdown_non_object() {
        let formatter = MarkdownFormatter::default();
        let json_array = json!(["item1", "item2"]);

        let result = formatter.format_json_as_markdown(&json_array);
        assert!(result.contains("```json"));
        assert!(result.contains("\"item1\""));
        assert!(result.contains("\"item2\""));
    }

    #[test]
    fn test_markdown_formatter_apply_template() {
        let formatter = MarkdownFormatter::default();
        let template = "Tool: {tool_name}, Status: {status}, Result: {result}, Error: {error}, Time: {execution_time}";
        let output = ToolOutput {
            tool_name: "test_tool".to_string(),
            success: false,
            result: json!({"key": "value"}),
            error: Some("Test error".to_string()),
            execution_time_ms: 150,
            metadata: HashMap::new(),
        };

        let result = formatter.apply_template(template, &output);
        assert!(result.contains("Tool: test_tool"));
        assert!(result.contains("Status: ❌ Failed"));
        assert!(result.contains("Error: Test error"));
        assert!(result.contains("Time: 150"));
        assert!(result.contains("```json"));
    }

    #[test]
    fn test_markdown_formatter_apply_template_no_error() {
        let formatter = MarkdownFormatter::default();
        let template = "Error: {error}";
        let output = ToolOutput {
            tool_name: "test_tool".to_string(),
            success: true,
            result: json!({}),
            error: None,
            execution_time_ms: 0,
            metadata: HashMap::new(),
        };

        let result = formatter.apply_template(template, &output);
        assert_eq!(result, "Error: ");
    }

    #[test]
    fn test_markdown_formatter_config() {
        let formatter = MarkdownFormatter::with_options(false, true)
            .with_template("tool1", "template1")
            .with_template("tool2", "template2");

        let config = formatter.config();
        assert_eq!(config["name"], "MarkdownFormatter");
        assert_eq!(config["type"], "formatter");
        assert_eq!(config["format"], "markdown");
        assert!(!config["include_metadata"].as_bool().unwrap());
        assert!(config["include_timing"].as_bool().unwrap());
        assert_eq!(config["custom_templates"], 2);
    }

    // Comprehensive tests for HtmlFormatter
    #[test]
    fn test_html_formatter_new() {
        let formatter = HtmlFormatter::default();
        assert!(!formatter.css_classes.is_empty());
        assert!(formatter.include_styles);
    }

    #[test]
    fn test_html_formatter_with_css_classes() {
        let mut custom_classes = HashMap::new();
        custom_classes.insert("custom".to_string(), "custom-class".to_string());
        custom_classes.insert("container".to_string(), "override-container".to_string());

        let formatter = HtmlFormatter::default().with_css_classes(custom_classes);

        assert_eq!(formatter.css_classes.get("custom").unwrap(), "custom-class");
        assert_eq!(
            formatter.css_classes.get("container").unwrap(),
            "override-container"
        );
    }

    #[test]
    fn test_html_formatter_without_styles() {
        let formatter = HtmlFormatter::default().without_styles();
        assert!(!formatter.include_styles);
    }

    #[tokio::test]
    async fn test_html_formatter_with_styles() {
        let formatter = HtmlFormatter::default();
        let output = utils::success_output("test_tool", json!({"key": "value"}));

        let processed = formatter.process(output).await.unwrap();

        if let Some(html) = processed.processed_result.get("html") {
            let content = html.as_str().unwrap();
            assert!(content.contains("<style>"));
            assert!(content.contains(".tool-output"));
            assert!(content.contains(".status-success"));
        }
    }

    #[tokio::test]
    async fn test_html_formatter_without_styles_content() {
        let formatter = HtmlFormatter::default().without_styles();
        let output = utils::success_output("test_tool", json!({"key": "value"}));

        let processed = formatter.process(output).await.unwrap();

        if let Some(html) = processed.processed_result.get("html") {
            let content = html.as_str().unwrap();
            assert!(!content.contains("<style>"));
        }
    }

    #[tokio::test]
    async fn test_html_formatter_with_metadata() {
        let formatter = HtmlFormatter::default();
        let mut output = utils::success_output("test_tool", json!({"key": "value"}));
        output
            .metadata
            .insert("test_key".to_string(), "test_value".to_string());
        output
            .metadata
            .insert("another_key".to_string(), "another_value".to_string());

        let processed = formatter.process(output).await.unwrap();

        if let Some(html) = processed.processed_result.get("html") {
            let content = html.as_str().unwrap();
            assert!(content.contains("<h3>Metadata</h3>"));
            assert!(content.contains("<ul>"));
            assert!(content.contains("Test Key"));
            assert!(content.contains("test_value"));
        }
    }

    #[tokio::test]
    async fn test_html_formatter_with_timing() {
        let formatter = HtmlFormatter::default();
        let mut output = utils::success_output("test_tool", json!({"key": "value"}));
        output.execution_time_ms = 250;

        let processed = formatter.process(output).await.unwrap();

        if let Some(html) = processed.processed_result.get("html") {
            let content = html.as_str().unwrap();
            assert!(content.contains("<hr>"));
            assert!(content.contains("Executed in 250ms"));
        }
    }

    #[tokio::test]
    async fn test_html_formatter_with_null_result() {
        let formatter = HtmlFormatter::default();
        let mut output = utils::success_output("test_tool", json!(null));
        output.result = serde_json::Value::Null;

        let processed = formatter.process(output).await.unwrap();

        if let Some(html) = processed.processed_result.get("html") {
            let content = html.as_str().unwrap();
            assert!(!content.contains("<h3>Result</h3>"));
        }
    }

    #[test]
    fn test_html_formatter_title_case() {
        let formatter = HtmlFormatter::default();
        assert_eq!(formatter.title_case("hello_world"), "Hello World");
        assert_eq!(formatter.title_case("single"), "Single");
        assert_eq!(formatter.title_case(""), "");
        assert_eq!(formatter.title_case("test_case"), "Test Case");
    }

    #[test]
    fn test_html_formatter_config() {
        let formatter = HtmlFormatter::default().without_styles();
        let config = formatter.config();

        assert_eq!(config["name"], "HtmlFormatter");
        assert_eq!(config["type"], "formatter");
        assert_eq!(config["format"], "html");
        assert!(!config["include_styles"].as_bool().unwrap());
        assert!(config["css_classes"].as_u64().unwrap() > 0);
    }

    // Comprehensive tests for JsonFormatter
    #[test]
    fn test_json_formatter_new() {
        let formatter = JsonFormatter::default();
        assert!(formatter.pretty_print);
        assert!(formatter.include_metadata);
        assert!(formatter.field_mappings.is_empty());
    }

    #[test]
    fn test_json_formatter_compact() {
        let formatter = JsonFormatter::default().compact();
        assert!(!formatter.pretty_print);
    }

    #[test]
    fn test_json_formatter_without_metadata() {
        let formatter = JsonFormatter::default().without_metadata();
        assert!(!formatter.include_metadata);
    }

    #[test]
    fn test_json_formatter_with_field_mapping() {
        let formatter = JsonFormatter::default()
            .with_field_mapping("old_field", "new_field")
            .with_field_mapping("another_old", "another_new");

        assert_eq!(
            formatter.field_mappings.get("old_field").unwrap(),
            "new_field"
        );
        assert_eq!(
            formatter.field_mappings.get("another_old").unwrap(),
            "another_new"
        );
    }

    #[tokio::test]
    async fn test_json_formatter_compact_output() {
        let formatter = JsonFormatter::default().compact();
        let output = utils::success_output("test_tool", json!({"key": "value"}));

        let processed = formatter.process(output).await.unwrap();

        if let Some(json_str) = processed.processed_result.get("json") {
            let content = json_str.as_str().unwrap();
            // Compact JSON should not have pretty formatting
            assert!(!content.contains("  "));
            assert!(!content.contains("\n"));
        }
    }

    #[tokio::test]
    async fn test_json_formatter_without_metadata_content() {
        let formatter = JsonFormatter::default().without_metadata();
        let mut output = utils::success_output("test_tool", json!({"key": "value"}));
        output
            .metadata
            .insert("test_key".to_string(), "test_value".to_string());

        let processed = formatter.process(output).await.unwrap();

        if let Some(structured) = processed.processed_result.get("structured") {
            assert!(structured.get("metadata").is_none());
        }
    }

    #[tokio::test]
    async fn test_json_formatter_with_metadata_content() {
        let formatter = JsonFormatter::default();
        let mut output = utils::success_output("test_tool", json!({"key": "value"}));
        output
            .metadata
            .insert("test_key".to_string(), "test_value".to_string());

        let processed = formatter.process(output).await.unwrap();

        if let Some(structured) = processed.processed_result.get("structured") {
            assert!(structured.get("metadata").is_some());
            assert_eq!(structured["metadata"]["test_key"], "test_value");
        }
    }

    #[tokio::test]
    async fn test_json_formatter_with_null_result() {
        let formatter = JsonFormatter::default();
        let mut output = utils::success_output("test_tool", json!(null));
        output.result = serde_json::Value::Null;

        let processed = formatter.process(output).await.unwrap();

        if let Some(structured) = processed.processed_result.get("structured") {
            assert!(structured.get("data").is_none());
        }
    }

    #[tokio::test]
    async fn test_json_formatter_with_execution_time() {
        let formatter = JsonFormatter::default();
        let mut output = utils::success_output("test_tool", json!({"key": "value"}));
        output.execution_time_ms = 300;

        let processed = formatter.process(output).await.unwrap();

        if let Some(structured) = processed.processed_result.get("structured") {
            assert_eq!(structured["execution_time_ms"], 300);
        }
    }

    #[tokio::test]
    async fn test_json_formatter_without_execution_time() {
        let formatter = JsonFormatter::default();
        let mut output = utils::success_output("test_tool", json!({"key": "value"}));
        output.execution_time_ms = 0;

        let processed = formatter.process(output).await.unwrap();

        if let Some(structured) = processed.processed_result.get("structured") {
            assert!(structured.get("execution_time_ms").is_none());
        }
    }

    #[test]
    fn test_json_formatter_remap_fields_object() {
        let formatter = JsonFormatter::default()
            .with_field_mapping("old_key", "new_key")
            .with_field_mapping("another_old", "another_new");

        let input = json!({
            "old_key": "value1",
            "another_old": "value2",
            "unchanged": "value3"
        });

        let result = formatter.remap_fields(&input);

        assert_eq!(result["new_key"], "value1");
        assert_eq!(result["another_new"], "value2");
        assert_eq!(result["unchanged"], "value3");
        assert!(result.get("old_key").is_none());
    }

    #[test]
    fn test_json_formatter_remap_fields_array() {
        let formatter = JsonFormatter::default().with_field_mapping("old_key", "new_key");

        let input = json!([
            {"old_key": "value1"},
            {"old_key": "value2"}
        ]);

        let result = formatter.remap_fields(&input);

        if let Value::Array(arr) = result {
            assert_eq!(arr[0]["new_key"], "value1");
            assert_eq!(arr[1]["new_key"], "value2");
            assert!(arr[0].get("old_key").is_none());
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_json_formatter_remap_fields_primitive() {
        let formatter = JsonFormatter::default();
        let input = json!("simple_string");

        let result = formatter.remap_fields(&input);
        assert_eq!(result, "simple_string");
    }

    #[test]
    fn test_json_formatter_remap_fields_nested() {
        let formatter = JsonFormatter::default().with_field_mapping("nested_old", "nested_new");

        let input = json!({
            "outer": {
                "nested_old": "value"
            }
        });

        let result = formatter.remap_fields(&input);
        assert_eq!(result["outer"]["nested_new"], "value");
        assert!(result["outer"].get("nested_old").is_none());
    }

    #[test]
    fn test_json_formatter_config() {
        let formatter = JsonFormatter::default()
            .compact()
            .without_metadata()
            .with_field_mapping("old", "new");

        let config = formatter.config();
        assert_eq!(config["name"], "JsonFormatter");
        assert_eq!(config["type"], "formatter");
        assert_eq!(config["format"], "json");
        assert!(!config["pretty_print"].as_bool().unwrap());
        assert!(!config["include_metadata"].as_bool().unwrap());
        assert_eq!(config["field_mappings"], 1);
    }

    // Comprehensive tests for MultiFormatProcessor
    #[test]
    fn test_multi_format_processor_new() {
        let processor = MultiFormatProcessor::default();
        assert!(processor.formats.is_empty());
    }

    #[test]
    fn test_multi_format_processor_add_format() {
        let processor = MultiFormatProcessor::default()
            .add_format(MarkdownFormatter::default())
            .add_format(JsonFormatter::default());

        assert_eq!(processor.formats.len(), 2);
    }

    #[test]
    fn test_multi_format_processor_standard_formats() {
        let processor = MultiFormatProcessor::standard_formats();
        assert_eq!(processor.formats.len(), 3); // Markdown, HTML, JSON
    }

    #[tokio::test]
    async fn test_multi_format_processor_empty_formats() {
        let processor = MultiFormatProcessor::default();
        let output = utils::success_output("test", json!({"key": "value"}));

        let processed = processor.process(output).await.unwrap();

        assert!(matches!(processed.format, OutputFormat::Custom(_)));
        assert_eq!(processed.processed_result, json!({}));
        assert_eq!(processed.summary.unwrap(), "Generated formats: ");
    }

    #[tokio::test]
    async fn test_multi_format_processor_single_format() {
        let processor = MultiFormatProcessor::default().add_format(JsonFormatter::default());

        let output = utils::success_output("test", json!({"key": "value"}));
        let processed = processor.process(output).await.unwrap();

        assert!(processed.processed_result.get("json").is_some());
        assert!(processed.summary.unwrap().contains("Json"));
    }

    #[test]
    fn test_multi_format_processor_config() {
        let processor = MultiFormatProcessor::default()
            .add_format(MarkdownFormatter::default())
            .add_format(JsonFormatter::default());

        let config = processor.config();
        assert_eq!(config["name"], "MultiFormatProcessor");
        assert_eq!(config["type"], "multi_formatter");

        if let Value::Array(formatters) = &config["formatters"] {
            assert_eq!(formatters.len(), 2);
        } else {
            panic!("Expected formatters array in config");
        }
    }

    // Tests for Default trait implementations
    #[test]
    fn test_markdown_formatter_default() {
        let formatter = MarkdownFormatter::default();
        assert!(formatter.include_metadata);
        assert!(formatter.include_timing);
        assert!(formatter.custom_templates.is_empty());
    }

    #[test]
    fn test_html_formatter_default() {
        let formatter = HtmlFormatter::default();
        assert!(formatter.include_styles);
        assert!(!formatter.css_classes.is_empty());
        assert_eq!(
            formatter.css_classes.get("container").unwrap(),
            "tool-output"
        );
        assert_eq!(
            formatter.css_classes.get("success").unwrap(),
            "status-success"
        );
        assert_eq!(formatter.css_classes.get("error").unwrap(), "status-error");
        assert_eq!(
            formatter.css_classes.get("result").unwrap(),
            "result-content"
        );
    }

    #[test]
    fn test_json_formatter_default() {
        let formatter = JsonFormatter::default();
        assert!(formatter.pretty_print);
        assert!(formatter.include_metadata);
        assert!(formatter.field_mappings.is_empty());
    }

    #[test]
    fn test_multi_format_processor_default() {
        let processor = MultiFormatProcessor::default();
        assert!(processor.formats.is_empty());
    }

    // Edge case tests
    #[test]
    fn test_html_escape_empty_string() {
        assert_eq!(html_escape(""), "");
    }

    #[test]
    fn test_html_escape_all_special_chars() {
        assert_eq!(html_escape("&<>\"'"), "&amp;&lt;&gt;&quot;&#x27;");
    }

    #[tokio::test]
    async fn test_markdown_formatter_empty_metadata() {
        let formatter = MarkdownFormatter::default();
        let output = utils::success_output("test_tool", json!({"key": "value"}));

        let processed = formatter.process(output).await.unwrap();

        if let Some(markdown) = processed.processed_result.get("markdown") {
            let content = markdown.as_str().unwrap();
            assert!(!content.contains("### Metadata"));
        }
    }

    #[tokio::test]
    async fn test_html_formatter_empty_metadata() {
        let formatter = HtmlFormatter::default();
        let output = utils::success_output("test_tool", json!({"key": "value"}));

        let processed = formatter.process(output).await.unwrap();

        if let Some(html) = processed.processed_result.get("html") {
            let content = html.as_str().unwrap();
            assert!(!content.contains("<h3>Metadata</h3>"));
        }
    }

    #[test]
    fn test_title_case_edge_cases() {
        let formatter = MarkdownFormatter::default();

        // Test empty string
        assert_eq!(formatter.title_case(""), "");

        // Test single character
        assert_eq!(formatter.title_case("a"), "A");

        // Test multiple underscores
        assert_eq!(formatter.title_case("a__b"), "A  B");

        // Test trailing underscore
        assert_eq!(formatter.title_case("test_"), "Test ");

        // Test leading underscore
        assert_eq!(formatter.title_case("_test"), " Test");
    }

    #[tokio::test]
    async fn test_formatters_name_method() {
        let md_formatter = MarkdownFormatter::default();
        let html_formatter = HtmlFormatter::default();
        let json_formatter = JsonFormatter::default();
        let multi_formatter = MultiFormatProcessor::default();

        assert_eq!(md_formatter.name(), "MarkdownFormatter");
        assert_eq!(html_formatter.name(), "HtmlFormatter");
        assert_eq!(json_formatter.name(), "JsonFormatter");
        assert_eq!(multi_formatter.name(), "MultiFormatProcessor");
    }

    #[tokio::test]
    async fn test_json_serialization_error_handling() {
        // Test that the formatter handles serialization gracefully
        let formatter = JsonFormatter::default();
        let output = utils::success_output("test_tool", json!({"key": "value"}));

        // This should not panic and should return a valid result
        let result = formatter.process(output).await;
        assert!(result.is_ok());
    }
}
