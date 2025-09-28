//! Output formatting patterns
//!
//! This module demonstrates how to transform tool outputs into different formats
//! like Markdown, HTML, JSON, and custom formats for different presentation needs.

use super::{OutputFormat, OutputProcessor, ProcessedOutput, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use core::fmt::{Debug, Formatter, Result as FmtResult, Write};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Markdown formatter for tool outputs
///
/// Converts tool outputs into clean, readable Markdown format.
/// Useful for documentation, reports, or display in Markdown-aware interfaces.
#[derive(Debug)]
pub struct Markdown {
    custom_templates: HashMap<String, String>,
    include_metadata: bool,
    include_timing: bool,
}

impl Markdown {
    /// Create a new `MarkdownFormatter` with default settings
    ///
    /// This is provided for API consistency. You can also use `MarkdownFormatter::default()`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a `MarkdownFormatter` with custom metadata and timing options
    #[must_use]
    pub fn with_options(include_metadata: bool, include_timing: bool) -> Self {
        Self {
            custom_templates: HashMap::default(),
            include_metadata,
            include_timing,
        }
    }

    /// Add a custom template for specific tool types
    #[must_use]
    pub fn with_template(mut self, tool_name: &str, template: &str) -> Self {
        self.custom_templates
            .insert(tool_name.to_string(), template.to_string());
        self
    }

    /// Apply a custom template to the output
    fn apply_template(template: &str, output: &ToolOutput) -> String {
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

    /// Format the output as Markdown
    fn format_as_markdown(&self, output: &ToolOutput) -> String {
        // Check for custom template first
        if let Some(template) = self.custom_templates.get(&output.tool_name) {
            return Self::apply_template(template, output);
        }

        let mut markdown = String::default();

        // Title
        let _ = write!(
            markdown,
            "## {} Results\n\n",
            Self::title_case(&output.tool_name)
        );

        // Status indicator
        let status_emoji = if output.success { "✅" } else { "❌" };
        let status_text = if output.success { "Success" } else { "Failed" };
        let _ = write!(markdown, "**Status:** {status_emoji} {status_text}\n\n");

        // Error message if present
        if let Some(ref error) = output.error {
            markdown.push_str("### Error Details\n\n");
            let _ = write!(markdown, "```\n{error}\n```\n\n");
        }

        // Main result
        if !output.result.is_null() {
            markdown.push_str("### Result\n\n");
            markdown.push_str(&Self::format_json_as_markdown(&output.result));
            markdown.push('\n');
        }

        // Metadata section
        if self.include_metadata && !output.metadata.is_empty() {
            markdown.push_str("### Metadata\n\n");
            for (key, value) in &output.metadata {
                let _ = writeln!(markdown, "- **{}:** {}", Self::title_case(key), value);
            }
            markdown.push('\n');
        }

        // Timing information
        if self.include_timing && output.execution_time_ms > 0 {
            let _ = write!(
                markdown,
                "---\n*Executed in {}ms*\n",
                output.execution_time_ms
            );
        }

        markdown
    }

    /// Format JSON as readable Markdown
    #[expect(clippy::pattern_type_mismatch, clippy::needless_borrowed_reference)]
    fn format_json_as_markdown(value: &Value) -> String {
        match value {
            &Value::Object(ref obj) => {
                let mut result = String::default();
                for (key, val) in obj {
                    match val {
                        &Value::String(ref s) => {
                            let _ = writeln!(result, "- **{}:** {}", Self::title_case(key), s);
                        }
                        Value::Number(n) => {
                            let _ = writeln!(result, "- **{}:** `{}`", Self::title_case(key), n);
                        }
                        Value::Bool(b) => {
                            let _ = writeln!(
                                result,
                                "- **{}:** {}",
                                Self::title_case(key),
                                if *b { "✅ Yes" } else { "❌ No" }
                            );
                        }
                        val => {
                            let _ = writeln!(result, "- **{}:** `{}`", Self::title_case(key), val);
                        }
                    }
                }
                result
            }
            value => {
                format!(
                    "```json\n{}\n```\n",
                    serde_json::to_string_pretty(&value).unwrap_or_default()
                )
            }
        }
    }

    /// Convert `snake_case` to Title Case
    fn title_case(s: &str) -> String {
        s.split('_')
            .map(|word| {
                let mut chars = word.chars();
                chars.next().map_or_else(String::default, |first| {
                    first.to_uppercase().collect::<String>() + chars.as_str()
                })
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[async_trait]
impl OutputProcessor for Markdown {
    async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput> {
        let markdown_content = self.format_as_markdown(&input);

        return Ok(ProcessedOutput {
            original: input,
            processed_result: json!({"markdown": markdown_content}),
            format: OutputFormat::Markdown,
            summary: None, // Formatters typically don't generate summaries
            routing_info: None,
        });
    }
    fn name(&self) -> &'static str {
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
pub struct Html {
    css_classes: HashMap<String, String>,
    include_styles: bool,
}

impl Html {
    /// Create a new `HtmlFormatter` with default settings
    ///
    /// This is provided for API consistency. You can also use `Html::default()`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add custom CSS classes, preserving existing defaults
    #[must_use]
    pub fn with_css_classes(mut self, classes: HashMap<String, String>) -> Self {
        for (key, value) in classes {
            self.css_classes.insert(key, value);
        }
        self
    }

    /// Disable inline CSS styles in output
    #[must_use]
    pub const fn without_styles(mut self) -> Self {
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
        let _ = write!(html, r#"<div class="{container_class}">"#);
        html.push('\n');

        // Title
        let _ = writeln!(
            html,
            "<h2>{} Results</h2>",
            Self::title_case(&output.tool_name)
        );

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
        let _ = write!(
            html,
            r#"<p class="{status_class}"><strong>Status:</strong> {status_text}</p>"#
        );
        html.push('\n');

        // Error details
        if let Some(ref error) = output.error {
            html.push_str("<h3>Error Details</h3>\n");
            let _ = writeln!(html, "<pre>{}</pre>", html_escape(error));
        }

        // Result
        if !output.result.is_null() {
            html.push_str("<h3>Result</h3>\n");
            let result_class = self
                .css_classes
                .get("result")
                .map_or("result-content", String::as_str);
            let _ = write!(
                html,
                r#"<div class="{}"><pre>{}</pre></div>"#,
                result_class,
                html_escape(&serde_json::to_string_pretty(&output.result).unwrap_or_default())
            );
            html.push('\n');
        }

        // Metadata
        if !output.metadata.is_empty() {
            html.push_str("<h3>Metadata</h3>\n<ul>\n");
            for (key, value) in &output.metadata {
                let _ = writeln!(
                    html,
                    "<li><strong>{}:</strong> {}</li>",
                    html_escape(&Self::title_case(key)),
                    html_escape(value)
                );
            }
            html.push_str("</ul>\n");
        }

        // Timing
        if output.execution_time_ms > 0 {
            let _ = writeln!(
                html,
                "<hr><em>Executed in {}ms</em>",
                output.execution_time_ms
            );
        }

        html.push_str("</div>\n");
        html
    }

    /// Convert `snake_case` to Title Case
    fn title_case(s: &str) -> String {
        s.split('_')
            .map(|word| {
                let mut chars = word.chars();
                chars.next().map_or_else(String::default, |first| {
                    first.to_uppercase().collect::<String>() + chars.as_str()
                })
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[async_trait]
impl OutputProcessor for Html {
    async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput> {
        let html_content = self.format_as_html(&input);

        return Ok(ProcessedOutput {
            original: input,
            processed_result: json!({"html": html_content}),
            format: OutputFormat::Html,
            summary: None,
            routing_info: None,
        });
    }
    fn name(&self) -> &'static str {
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
pub struct Json {
    field_mappings: HashMap<String, String>,
    include_metadata: bool,
    pretty_print: bool,
}

impl Json {
    /// Create a new `JsonFormatter` with default settings
    ///
    /// This is provided for API consistency. You can also use `Json::default()`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure formatter to output compact JSON
    #[must_use]
    pub const fn compact(mut self) -> Self {
        self.pretty_print = false;
        self
    }

    /// Add a field name mapping for JSON transformation
    #[must_use]
    pub fn with_field_mapping(mut self, from: &str, to: &str) -> Self {
        self.field_mappings.insert(from.to_string(), to.to_string());
        self
    }

    /// Configure formatter to exclude metadata from output
    #[must_use]
    pub const fn without_metadata(mut self) -> Self {
        self.include_metadata = false;
        self
    }

    /// Format the output as structured JSON
    fn format_as_json(&self, output: &ToolOutput) -> Value {
        let mut result = json!({
            "tool": output.tool_name,
            "success": output.success,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        if let Some(ref error) = output.error {
            if let Value::Object(ref mut obj) = result {
                obj.insert("error".to_string(), json!(error));
            }
        }

        if !output.result.is_null() {
            if let Value::Object(ref mut obj) = result {
                obj.insert("data".to_string(), self.remap_fields(&output.result));
            }
        }

        if self.include_metadata && !output.metadata.is_empty() {
            if let Value::Object(ref mut obj) = result {
                obj.insert("metadata".to_string(), json!(output.metadata));
            }
        }

        if output.execution_time_ms > 0 {
            if let Value::Object(ref mut obj) = result {
                obj.insert(
                    "execution_time_ms".to_string(),
                    json!(output.execution_time_ms),
                );
            }
        }

        result
    }

    /// Recursively remap field names according to configured mappings
    #[expect(clippy::pattern_type_mismatch, clippy::needless_borrowed_reference)]
    fn remap_fields(&self, value: &Value) -> Value {
        match value {
            &Value::Object(ref obj) => {
                let mut new_obj = serde_json::Map::new();
                for (key, val) in obj {
                    let new_key = self.field_mappings.get(key).unwrap_or(key);
                    new_obj.insert(new_key.clone(), self.remap_fields(val));
                }
                Value::Object(new_obj)
            }
            Value::Array(arr) => Value::Array(arr.iter().map(|v| self.remap_fields(v)).collect()),
            value => value.clone(),
        }
    }
}

#[async_trait]
impl OutputProcessor for Json {
    async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput> {
        let formatted_json = self.format_as_json(&input);

        let json_string = if self.pretty_print {
            serde_json::to_string_pretty(&formatted_json)?
        } else {
            serde_json::to_string(&formatted_json)?
        };

        return Ok(ProcessedOutput {
            original: input,
            processed_result: json!({"json": json_string, "structured": formatted_json}),
            format: OutputFormat::Json,
            summary: None,
            routing_info: None,
        });
    }
    fn name(&self) -> &'static str {
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

impl Debug for MultiFormatProcessor {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("MultiFormatProcessor")
            .field("formats", &format!("{} formatters", self.formats.len()))
            .finish()
    }
}

impl MultiFormatProcessor {
    /// Add a formatter to the processor
    #[must_use]
    pub fn add_format<F: OutputProcessor + 'static>(mut self, formatter: F) -> Self {
        self.formats.push(Box::new(formatter));
        self
    }

    /// Create a new `MultiFormatProcessor` with no formats
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a `MultiFormatProcessor` with standard formatters (Markdown, HTML, JSON)
    #[must_use]
    pub fn standard_formats() -> Self {
        Self::new()
            .add_format(Markdown::default())
            .add_format(Html::default())
            .add_format(Json::default())
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
            if let Value::Object(ref obj) = formatted.processed_result {
                if let Value::Object(ref mut combined_obj) = combined_result {
                    for (key, value) in obj {
                        combined_obj.insert(key.clone(), value.clone());
                    }
                }
            }

            // Track what formats we generated
            formats.push(format!("{:?}", formatted.format));
        }

        return Ok(ProcessedOutput {
            original: input,
            processed_result: combined_result,
            format: OutputFormat::Custom("multi".to_string()),
            summary: Some(format!("Generated formats: {}", formats.join(", "))),
            routing_info: None,
        });
    }
    fn name(&self) -> &'static str {
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

impl Default for Markdown {
    fn default() -> Self {
        Self {
            custom_templates: HashMap::default(),
            include_metadata: true,
            include_timing: true,
        }
    }
}

impl Default for Html {
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

impl Default for Json {
    fn default() -> Self {
        Self {
            field_mappings: HashMap::default(),
            include_metadata: true,
            pretty_print: true,
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::processors::utils;

    #[tokio::test]
    async fn test_markdown_formatter() {
        let formatter = Markdown::default();
        let output = utils::success_output(
            "get_balance",
            json!({"balance_sol": 1.5, "address": "11111111111111111111111111111112"}),
        );

        let processed = formatter
            .process(output)
            .await
            .expect("Markdown formatter should process valid output");

        assert!(matches!(processed.format, OutputFormat::Markdown));
        if let Some(markdown) = processed.processed_result.get("markdown") {
            let content = markdown
                .as_str()
                .expect("Processed markdown should be a string");
            assert!(content.contains("## Get Balance Results"));
            assert!(content.contains("✅ Success"));
            assert!(content.contains("Balance Sol"));
        } else {
            panic!("Expected markdown content in processed result");
        }
    }

    #[tokio::test]
    async fn test_html_formatter() {
        let formatter = Html::default();
        let output = utils::error_output("test_tool", "Connection failed");

        let processed = formatter
            .process(output)
            .await
            .expect("HTML formatter should process valid output");

        assert!(matches!(processed.format, OutputFormat::Html));
        if let Some(html) = processed.processed_result.get("html") {
            let content = html.as_str().expect("Processed HTML should be a string");
            assert!(content.contains("<div"));
            assert!(content.contains("❌ Failed"));
            assert!(content.contains("Connection failed"));
        } else {
            panic!("Expected HTML content in processed result");
        }
    }

    #[tokio::test]
    async fn test_json_formatter() {
        let formatter = Json::default().with_field_mapping("balance_sol", "balance_solana");

        let output = utils::success_output("get_balance", json!({"balance_sol": 1.5}));

        let processed = formatter
            .process(output)
            .await
            .expect("JSON formatter should process valid output");

        assert!(matches!(processed.format, OutputFormat::Json));
        if let Some(structured) = processed.processed_result.get("structured") {
            assert!(structured
                .get("data")
                .unwrap()
                .get("balance_solana")
                .unwrap()
                .is_number());
            assert_eq!(
                structured
                    .get("data")
                    .unwrap()
                    .get("balance_solana")
                    .unwrap(),
                1.5
            );
            assert!(structured["success"]
                .as_bool()
                .expect("Success field should be a boolean"));
        } else {
            panic!("Expected structured JSON in processed result");
        }
    }

    #[tokio::test]
    async fn test_multi_format_processor() {
        let processor = MultiFormatProcessor::default()
            .add_format(Markdown::default())
            .add_format(Json::default());

        let output = utils::success_output("test", json!({"key": "value"}));
        let processed = processor
            .process(output)
            .await
            .expect("Multi-format processor should process valid output");

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
        let formatter = Markdown::default();
        assert!(formatter.include_metadata);
        assert!(formatter.include_timing);
        assert!(formatter.custom_templates.is_empty());
    }

    #[test]
    fn test_markdown_formatter_with_options() {
        let formatter = Markdown::with_options(false, false);
        assert!(!formatter.include_metadata);
        assert!(!formatter.include_timing);
        assert!(formatter.custom_templates.is_empty());
    }

    #[test]
    fn test_markdown_formatter_with_template() {
        let formatter =
            Markdown::default().with_template("test_tool", "Custom template for {tool_name}");

        assert!(formatter.custom_templates.contains_key("test_tool"));
        assert_eq!(
            formatter
                .custom_templates
                .get("test_tool")
                .expect("Custom template should exist"),
            "Custom template for {tool_name}"
        );
    }

    #[tokio::test]
    async fn test_markdown_formatter_with_custom_template() {
        let formatter = Markdown::default()
            .with_template("test_tool", "## {tool_name}\nStatus: {status}\nResult: {result}\nError: {error}\nTime: {execution_time}ms");

        let output = utils::success_output("test_tool", json!({"key": "value"}));
        let processed = formatter
            .process(output)
            .await
            .expect("Markdown formatter should process with custom template");

        if let Some(markdown) = processed.processed_result.get("markdown") {
            let content = markdown
                .as_str()
                .expect("Custom templated markdown should be a string");
            assert!(content.contains("## test_tool"));
            assert!(content.contains("Status: ✅ Success"));
            assert!(content.contains("Time: 0ms"));
        }
    }

    #[tokio::test]
    async fn test_markdown_formatter_with_error() {
        let formatter = Markdown::default();
        let output = utils::error_output("test_tool", "Test error message");

        let processed = formatter
            .process(output)
            .await
            .expect("Markdown formatter should process error output");

        if let Some(markdown) = processed.processed_result.get("markdown") {
            let content = markdown
                .as_str()
                .expect("Error markdown should be a string");
            assert!(content.contains("❌ Failed"));
            assert!(content.contains("### Error Details"));
            assert!(content.contains("Test error message"));
        }
    }

    #[tokio::test]
    async fn test_markdown_formatter_without_metadata() {
        let formatter = Markdown::with_options(false, true);
        let mut output = utils::success_output("test_tool", json!({"key": "value"}));
        output
            .metadata
            .insert("test_key".to_string(), "test_value".to_string());

        let processed = formatter
            .process(output)
            .await
            .expect("Markdown formatter should process output without metadata");

        if let Some(markdown) = processed.processed_result.get("markdown") {
            let content = markdown
                .as_str()
                .expect("No-metadata markdown should be a string");
            assert!(!content.contains("### Metadata"));
            assert!(!content.contains("test_key"));
        }
    }

    #[tokio::test]
    async fn test_markdown_formatter_without_timing() {
        let formatter = Markdown::with_options(true, false);
        let mut output = utils::success_output("test_tool", json!({"key": "value"}));
        output.execution_time_ms = 100;

        let processed = formatter
            .process(output)
            .await
            .expect("Markdown formatter should process output without timing");

        if let Some(markdown) = processed.processed_result.get("markdown") {
            let content = markdown
                .as_str()
                .expect("No-timing markdown should be a string");
            assert!(!content.contains("Executed in"));
            assert!(!content.contains("100ms"));
        }
    }

    #[tokio::test]
    async fn test_markdown_formatter_with_null_result() {
        let formatter = Markdown::default();
        let mut output = utils::success_output("test_tool", json!(null));
        output.result = serde_json::Value::Null;

        let processed = formatter
            .process(output)
            .await
            .expect("Markdown formatter should process null result");

        if let Some(markdown) = processed.processed_result.get("markdown") {
            let content = markdown
                .as_str()
                .expect("Null result markdown should be a string");
            assert!(!content.contains("### Result"));
        }
    }

    #[test]
    fn test_markdown_formatter_title_case() {
        let _formatter = Markdown::default();
        assert_eq!(Markdown::title_case("hello_world"), "Hello World");
        assert_eq!(Markdown::title_case("single"), "Single");
        assert_eq!(Markdown::title_case(""), "");
        assert_eq!(
            Markdown::title_case("already_formatted_string"),
            "Already Formatted String"
        );
        assert_eq!(Markdown::title_case("a"), "A");
    }

    #[test]
    fn test_markdown_formatter_format_json_as_markdown_object() {
        let _formatter = Markdown::default();
        let json_obj = json!({
            "string_field": "test_value",
            "number_field": 42,
            "bool_true": true,
            "bool_false": false,
            "complex_field": {"nested": "value"}
        });

        let result = Markdown::format_json_as_markdown(&json_obj);
        assert!(result.contains("- **String Field:** test_value"));
        assert!(result.contains("- **Number Field:** `42`"));
        assert!(result.contains("- **Bool True:** ✅ Yes"));
        assert!(result.contains("- **Bool False:** ❌ No"));
        assert!(result.contains("- **Complex Field:** `{\"nested\":\"value\"}`"));
    }

    #[test]
    fn test_markdown_formatter_format_json_as_markdown_non_object() {
        let _formatter = Markdown::default();
        let json_array = json!(["item1", "item2"]);

        let result = Markdown::format_json_as_markdown(&json_array);
        assert!(result.contains("```json"));
        assert!(result.contains("\"item1\""));
        assert!(result.contains("\"item2\""));
    }

    #[test]
    fn test_markdown_formatter_apply_template() {
        let _formatter = Markdown::default();
        #[expect(clippy::literal_string_with_formatting_args)]
        let template = "Tool: {tool_name}, Status: {status}, Result: {result}, Error: {error}, Time: {execution_time}";
        let output = ToolOutput {
            tool_name: "test_tool".to_string(),
            success: false,
            result: json!({"key": "value"}),
            error: Some("Test error".to_string()),
            execution_time_ms: 150,
            metadata: HashMap::new(),
        };

        let result = Markdown::apply_template(template, &output);
        assert!(result.contains("Tool: test_tool"));
        assert!(result.contains("Status: ❌ Failed"));
        assert!(result.contains("Error: Test error"));
        assert!(result.contains("Time: 150"));
        assert!(result.contains("```json"));
    }

    #[test]
    fn test_markdown_formatter_apply_template_no_error() {
        let _formatter = Markdown::default();
        let template = "Error: {error}";
        let output = ToolOutput {
            tool_name: "test_tool".to_string(),
            success: true,
            result: json!({}),
            error: None,
            execution_time_ms: 0,
            metadata: HashMap::new(),
        };

        let result = Markdown::apply_template(template, &output);
        assert_eq!(result, "Error: ");
    }

    #[test]
    fn test_markdown_formatter_config() {
        let formatter = Markdown::with_options(false, true)
            .with_template("tool1", "template1")
            .with_template("tool2", "template2");

        let config = formatter.config();
        assert_eq!(config.get("name").unwrap(), "MarkdownFormatter");
        assert_eq!(config.get("type").unwrap(), "formatter");
        assert_eq!(config.get("format").unwrap(), "markdown");
        assert!(!config
            .get("include_metadata")
            .unwrap()
            .as_bool()
            .expect("include_metadata should be a boolean"));
        assert!(config
            .get("include_timing")
            .unwrap()
            .as_bool()
            .expect("include_timing should be a boolean"));
        assert_eq!(config.get("custom_templates").unwrap(), 2);
    }

    // Comprehensive tests for HtmlFormatter
    #[test]
    fn test_html_formatter_new() {
        let formatter = Html::default();
        assert!(!formatter.css_classes.is_empty());
        assert!(formatter.include_styles);
    }

    #[test]
    fn test_html_formatter_with_css_classes() {
        let mut custom_classes = HashMap::new();
        custom_classes.insert("custom".to_string(), "custom-class".to_string());
        custom_classes.insert("container".to_string(), "override-container".to_string());

        let formatter = Html::default().with_css_classes(custom_classes);

        assert_eq!(
            formatter
                .css_classes
                .get("custom")
                .expect("Custom CSS class should exist"),
            "custom-class"
        );
        assert_eq!(
            formatter
                .css_classes
                .get("container")
                .expect("Container CSS class should exist"),
            "override-container"
        );
    }

    #[test]
    fn test_html_formatter_without_styles() {
        let formatter = Html::default().without_styles();
        assert!(!formatter.include_styles);
    }

    #[tokio::test]
    async fn test_html_formatter_with_styles() {
        let formatter = Html::default();
        let output = utils::success_output("test_tool", json!({"key": "value"}));

        let processed = formatter
            .process(output)
            .await
            .expect("HTML formatter should process output with styles");

        if let Some(html) = processed.processed_result.get("html") {
            let content = html.as_str().expect("HTML with styles should be a string");
            assert!(content.contains("<style>"));
            assert!(content.contains(".tool-output"));
            assert!(content.contains(".status-success"));
        }
    }

    #[tokio::test]
    async fn test_html_formatter_without_styles_content() {
        let formatter = Html::default().without_styles();
        let output = utils::success_output("test_tool", json!({"key": "value"}));

        let processed = formatter
            .process(output)
            .await
            .expect("HTML formatter should process output without styles");

        if let Some(html) = processed.processed_result.get("html") {
            let content = html
                .as_str()
                .expect("HTML without styles should be a string");
            assert!(!content.contains("<style>"));
        }
    }

    #[tokio::test]
    async fn test_html_formatter_with_metadata() {
        let formatter = Html::default();
        let mut output = utils::success_output("test_tool", json!({"key": "value"}));
        output
            .metadata
            .insert("test_key".to_string(), "test_value".to_string());
        output
            .metadata
            .insert("another_key".to_string(), "another_value".to_string());

        let processed = formatter
            .process(output)
            .await
            .expect("HTML formatter should process output with metadata");

        if let Some(html) = processed.processed_result.get("html") {
            let content = html
                .as_str()
                .expect("HTML with metadata should be a string");
            assert!(content.contains("<h3>Metadata</h3>"));
            assert!(content.contains("<ul>"));
            assert!(content.contains("Test Key"));
            assert!(content.contains("test_value"));
        }
    }

    #[tokio::test]
    async fn test_html_formatter_with_timing() {
        let formatter = Html::default();
        let mut output = utils::success_output("test_tool", json!({"key": "value"}));
        output.execution_time_ms = 250;

        let processed = formatter
            .process(output)
            .await
            .expect("HTML formatter should process output with timing");

        if let Some(html) = processed.processed_result.get("html") {
            let content = html.as_str().expect("HTML with timing should be a string");
            assert!(content.contains("<hr>"));
            assert!(content.contains("Executed in 250ms"));
        }
    }

    #[tokio::test]
    async fn test_html_formatter_with_null_result() {
        let formatter = Html::default();
        let mut output = utils::success_output("test_tool", json!(null));
        output.result = serde_json::Value::Null;

        let processed = formatter
            .process(output)
            .await
            .expect("HTML formatter should process null result");

        if let Some(html) = processed.processed_result.get("html") {
            let content = html
                .as_str()
                .expect("HTML with null result should be a string");
            assert!(!content.contains("<h3>Result</h3>"));
        }
    }

    #[test]
    fn test_html_formatter_title_case() {
        let _formatter = Html::default();
        assert_eq!(Html::title_case("hello_world"), "Hello World");
        assert_eq!(Html::title_case("single"), "Single");
        assert_eq!(Html::title_case(""), "");
        assert_eq!(Html::title_case("test_case"), "Test Case");
    }

    #[test]
    fn test_html_formatter_config() {
        let formatter = Html::default().without_styles();
        let config = formatter.config();

        assert_eq!(config.get("name").unwrap(), "HtmlFormatter");
        assert_eq!(config.get("type").unwrap(), "formatter");
        assert_eq!(config.get("format").unwrap(), "html");
        assert!(!config
            .get("include_styles")
            .unwrap()
            .as_bool()
            .expect("include_styles should be a boolean"));
        assert!(
            config
                .get("css_classes")
                .unwrap()
                .as_u64()
                .expect("css_classes should be a number")
                > 0
        );
    }

    // Comprehensive tests for JsonFormatter
    #[test]
    fn test_json_formatter_new() {
        let formatter = Json::default();
        assert!(formatter.pretty_print);
        assert!(formatter.include_metadata);
        assert!(formatter.field_mappings.is_empty());
    }

    #[test]
    fn test_json_formatter_compact() {
        let formatter = Json::default().compact();
        assert!(!formatter.pretty_print);
    }

    #[test]
    fn test_json_formatter_without_metadata() {
        let formatter = Json::default().without_metadata();
        assert!(!formatter.include_metadata);
    }

    #[test]
    fn test_json_formatter_with_field_mapping() {
        let formatter = Json::default()
            .with_field_mapping("old_field", "new_field")
            .with_field_mapping("another_old", "another_new");

        assert_eq!(
            formatter
                .field_mappings
                .get("old_field")
                .expect("old_field mapping should exist"),
            "new_field"
        );
        assert_eq!(
            formatter
                .field_mappings
                .get("another_old")
                .expect("another_old mapping should exist"),
            "another_new"
        );
    }

    #[tokio::test]
    async fn test_json_formatter_compact_output() {
        let formatter = Json::default().compact();
        let output = utils::success_output("test_tool", json!({"key": "value"}));

        let processed = formatter
            .process(output)
            .await
            .expect("JSON formatter should process compact output");

        if let Some(json_str) = processed.processed_result.get("json") {
            let content = json_str.as_str().expect("Compact JSON should be a string");
            // Compact JSON should not have pretty formatting
            assert!(!content.contains("  "));
            assert!(!content.contains('\n'));
        }
    }

    #[tokio::test]
    async fn test_json_formatter_without_metadata_content() {
        let formatter = Json::default().without_metadata();
        let mut output = utils::success_output("test_tool", json!({"key": "value"}));
        output
            .metadata
            .insert("test_key".to_string(), "test_value".to_string());

        let processed = formatter
            .process(output)
            .await
            .expect("JSON formatter should process output without metadata");

        if let Some(structured) = processed.processed_result.get("structured") {
            assert!(structured.get("metadata").is_none());
        }
    }

    #[tokio::test]
    async fn test_json_formatter_with_metadata_content() {
        let formatter = Json::default();
        let mut output = utils::success_output("test_tool", json!({"key": "value"}));
        output
            .metadata
            .insert("test_key".to_string(), "test_value".to_string());

        let processed = formatter
            .process(output)
            .await
            .expect("JSON formatter should process output with metadata");

        if let Some(structured) = processed.processed_result.get("structured") {
            assert!(structured.get("metadata").is_some());
            assert_eq!(
                structured.get("metadata").unwrap().get("test_key").unwrap(),
                "test_value"
            );
        }
    }

    #[tokio::test]
    async fn test_json_formatter_with_null_result() {
        let formatter = Json::default();
        let mut output = utils::success_output("test_tool", json!(null));
        output.result = serde_json::Value::Null;

        let processed = formatter
            .process(output)
            .await
            .expect("JSON formatter should process null result");

        if let Some(structured) = processed.processed_result.get("structured") {
            assert!(structured.get("data").is_none());
        }
    }

    #[tokio::test]
    async fn test_json_formatter_with_execution_time() {
        let formatter = Json::default();
        let mut output = utils::success_output("test_tool", json!({"key": "value"}));
        output.execution_time_ms = 300;

        let processed = formatter
            .process(output)
            .await
            .expect("JSON formatter should process output with execution time");

        if let Some(structured) = processed.processed_result.get("structured") {
            assert_eq!(structured["execution_time_ms"], 300);
        }
    }

    #[tokio::test]
    async fn test_json_formatter_without_execution_time() {
        let formatter = Json::default();
        let mut output = utils::success_output("test_tool", json!({"key": "value"}));
        output.execution_time_ms = 0;

        let processed = formatter
            .process(output)
            .await
            .expect("JSON formatter should process output without execution time");

        if let Some(structured) = processed.processed_result.get("structured") {
            assert!(structured.get("execution_time_ms").is_none());
        }
    }

    #[test]
    fn test_json_formatter_remap_fields_object() {
        let formatter = Json::default()
            .with_field_mapping("old_key", "new_key")
            .with_field_mapping("another_old", "another_new");

        let input = json!({
            "old_key": "value1",
            "another_old": "value2",
            "unchanged": "value3"
        });

        let result = formatter.remap_fields(&input);

        assert_eq!(result.get("new_key").unwrap(), "value1");
        assert_eq!(result.get("another_new").unwrap(), "value2");
        assert_eq!(result.get("unchanged").unwrap(), "value3");
        assert!(result.get("old_key").is_none());
    }

    #[test]
    fn test_json_formatter_remap_fields_array() {
        let formatter = Json::default().with_field_mapping("old_key", "new_key");

        let input = json!([
            {"old_key": "value1"},
            {"old_key": "value2"}
        ]);

        let result = formatter.remap_fields(&input);

        if let Value::Array(arr) = result {
            assert_eq!(
                arr.first().expect("First element should exist")["new_key"],
                "value1"
            );
            assert_eq!(
                arr.get(1).expect("Second element should exist")["new_key"],
                "value2"
            );
            assert!(arr
                .first()
                .expect("First element should exist")
                .get("old_key")
                .is_none());
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_json_formatter_remap_fields_primitive() {
        let formatter = Json::default();
        let input = json!("simple_string");

        let result = formatter.remap_fields(&input);
        assert_eq!(result, "simple_string");
    }

    #[test]
    fn test_json_formatter_remap_fields_nested() {
        let formatter = Json::default().with_field_mapping("nested_old", "nested_new");

        let input = json!({
            "outer": {
                "nested_old": "value"
            }
        });

        let result = formatter.remap_fields(&input);
        assert_eq!(
            result.get("outer").unwrap().get("nested_new").unwrap(),
            "value"
        );
        assert!(result.get("outer").unwrap().get("nested_old").is_none());
    }

    #[test]
    fn test_json_formatter_config() {
        let formatter = Json::default()
            .compact()
            .without_metadata()
            .with_field_mapping("old", "new");

        let config = formatter.config();
        assert_eq!(config.get("name").unwrap(), "JsonFormatter");
        assert_eq!(config.get("type").unwrap(), "formatter");
        assert_eq!(config.get("format").unwrap(), "json");
        assert!(!config
            .get("pretty_print")
            .unwrap()
            .as_bool()
            .expect("pretty_print should be a boolean"));
        assert!(!config
            .get("include_metadata")
            .unwrap()
            .as_bool()
            .expect("include_metadata should be a boolean"));
        assert_eq!(config.get("field_mappings").unwrap(), 1);
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
            .add_format(Markdown::default())
            .add_format(Json::default());

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

        let processed = processor
            .process(output)
            .await
            .expect("Empty multi-format processor should succeed");

        assert!(matches!(processed.format, OutputFormat::Custom(_)));
        assert_eq!(processed.processed_result, json!({}));
        assert_eq!(
            processed.summary.expect("Summary should exist"),
            "Generated formats: "
        );
    }

    #[tokio::test]
    async fn test_multi_format_processor_single_format() {
        let processor = MultiFormatProcessor::default().add_format(Json::default());

        let output = utils::success_output("test", json!({"key": "value"}));
        let processed = processor
            .process(output)
            .await
            .expect("Single format processor should succeed");

        assert!(processed.processed_result.get("json").is_some());
        assert!(processed
            .summary
            .expect("Summary should exist")
            .contains("Json"));
    }

    #[test]
    fn test_multi_format_processor_config() {
        let processor = MultiFormatProcessor::default()
            .add_format(Markdown::default())
            .add_format(Json::default());

        let config = processor.config();
        assert_eq!(config.get("name").unwrap(), "MultiFormatProcessor");
        assert_eq!(config.get("type").unwrap(), "multi_formatter");

        if let Value::Array(ref formatters) = *config.get("formatters").unwrap() {
            assert_eq!(formatters.len(), 2);
        } else {
            panic!("Expected formatters array in config");
        }
    }

    // Tests for Default trait implementations
    #[test]
    fn test_markdown_formatter_default() {
        let formatter = Markdown::default();
        assert!(formatter.include_metadata);
        assert!(formatter.include_timing);
        assert!(formatter.custom_templates.is_empty());
    }

    #[test]
    fn test_html_formatter_default() {
        let formatter = Html::default();
        assert!(formatter.include_styles);
        assert!(!formatter.css_classes.is_empty());
        assert_eq!(
            formatter
                .css_classes
                .get("container")
                .expect("Container CSS class should exist"),
            "tool-output"
        );
        assert_eq!(
            formatter
                .css_classes
                .get("success")
                .expect("Success CSS class should exist"),
            "status-success"
        );
        assert_eq!(
            formatter
                .css_classes
                .get("error")
                .expect("Error CSS class should exist"),
            "status-error"
        );
        assert_eq!(
            formatter
                .css_classes
                .get("result")
                .expect("Result CSS class should exist"),
            "result-content"
        );
    }

    #[test]
    fn test_json_formatter_default() {
        let formatter = Json::default();
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
        let formatter = Markdown::default();
        let output = utils::success_output("test_tool", json!({"key": "value"}));

        let processed = formatter
            .process(output)
            .await
            .expect("Markdown formatter should process output with empty metadata");

        if let Some(markdown) = processed.processed_result.get("markdown") {
            let content = markdown
                .as_str()
                .expect("Empty metadata markdown should be a string");
            assert!(!content.contains("### Metadata"));
        }
    }

    #[tokio::test]
    async fn test_html_formatter_empty_metadata() {
        let formatter = Html::default();
        let output = utils::success_output("test_tool", json!({"key": "value"}));

        let processed = formatter
            .process(output)
            .await
            .expect("HTML formatter should process output with empty metadata");

        if let Some(html) = processed.processed_result.get("html") {
            let content = html
                .as_str()
                .expect("Empty metadata HTML should be a string");
            assert!(!content.contains("<h3>Metadata</h3>"));
        }
    }

    #[test]
    fn test_title_case_edge_cases() {
        let _formatter = Markdown::default();

        // Test empty string
        assert_eq!(Markdown::title_case(""), "");

        // Test single character
        assert_eq!(Markdown::title_case("a"), "A");

        // Test multiple underscores
        assert_eq!(Markdown::title_case("a__b"), "A  B");

        // Test trailing underscore
        assert_eq!(Markdown::title_case("test_"), "Test ");

        // Test leading underscore
        assert_eq!(Markdown::title_case("_test"), " Test");
    }

    #[tokio::test]
    async fn test_formatters_name_method() {
        let md_formatter = Markdown::default();
        let html_formatter = Html::default();
        let json_formatter = Json::default();
        let multi_formatter = MultiFormatProcessor::default();

        assert_eq!(md_formatter.name(), "MarkdownFormatter");
        assert_eq!(html_formatter.name(), "HtmlFormatter");
        assert_eq!(json_formatter.name(), "JsonFormatter");
        assert_eq!(multi_formatter.name(), "MultiFormatProcessor");
    }

    #[tokio::test]
    async fn test_json_serialization_error_handling() {
        // Test that the formatter handles serialization gracefully
        let formatter = Json::default();
        let output = utils::success_output("test_tool", json!({"key": "value"}));

        // This should not panic and should return a valid result
        let result = formatter.process(output).await;
        assert!(result.is_ok());
    }
}
