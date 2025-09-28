//! Event parsing utilities and base implementations.

use crate::error::{EventError, EventResult};
use crate::traits::{Event, EventParser, ParserInfo};
use crate::types::{EventKind, EventMetadata, GenericEvent};
use async_trait::async_trait;
use core::fmt::{Debug, Formatter, Result as FmtResult};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{
    io::{Error as IoError, ErrorKind},
    sync::Arc,
};

/// Configuration for parsing events from different data sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ParsingConfig {
    /// Default values for missing fields
    pub defaults: HashMap<String, serde_json::Value>,
    /// Field mappings for extracting event data
    pub field_mappings: HashMap<String, String>,
    /// Data format (e.g., "json", "binary", "xml")
    pub format: String,
    /// Parser name
    pub name: String,
    /// Validation rules
    pub validation: ValidationConfig,
    /// Parser version
    pub version: String,
}

impl ParsingConfig {
    /// Create a new parsing configuration
    #[must_use]
    #[inline]
    pub fn new(name: String, version: String, format: String) -> Self {
        Self {
            defaults: HashMap::new(),
            field_mappings: HashMap::new(),
            format,
            name,
            validation: ValidationConfig::default(),
            version,
        }
    }

    /// Add a default value
    #[must_use]
    #[inline]
    pub fn with_default<T: Serialize>(mut self, field: String, value: T) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.defaults.insert(field, json_value);
        }
        self
    }

    /// Add a field mapping
    #[must_use]
    #[inline]
    pub fn with_mapping(mut self, field: String, source_path: String) -> Self {
        self.field_mappings.insert(field, source_path);
        self
    }

    /// Set validation configuration
    #[must_use]
    #[inline]
    pub fn with_validation(mut self, validation: ValidationConfig) -> Self {
        self.validation = validation;
        self
    }
}

/// Validation configuration for parsed events.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct ValidationConfig {
    /// Field type constraints
    pub field_types: HashMap<String, String>,
    /// Maximum values for numeric fields
    pub max_values: HashMap<String, f64>,
    /// Minimum values for numeric fields
    pub min_values: HashMap<String, f64>,
    /// Regex patterns for string fields
    pub patterns: HashMap<String, String>,
    /// Required fields that must be present
    pub required_fields: Vec<String>,
}

impl ValidationConfig {
    /// Add a regex pattern constraint
    #[must_use]
    #[inline]
    pub fn with_pattern(mut self, field: String, pattern: String) -> Self {
        self.patterns.insert(field, pattern);
        self
    }

    /// Add a numeric range constraint
    #[must_use]
    #[inline]
    pub fn with_range(mut self, field: String, min: f64, max: f64) -> Self {
        self.min_values.insert(field.clone(), min);
        self.max_values.insert(field, max);
        self
    }

    /// Add a required field
    #[must_use]
    #[inline]
    pub fn with_required(mut self, field: String) -> Self {
        self.required_fields.push(field);
        self
    }

    /// Add a field type constraint
    #[must_use]
    #[inline]
    pub fn with_type(mut self, field: String, type_name: String) -> Self {
        self.field_types.insert(field, type_name);
        self
    }
}

/// JSON-based event parser implementation.
#[derive(Debug)]
#[expect(clippy::module_name_repetitions)]
pub struct JsonEventParser {
    /// Parser configuration
    config: ParsingConfig,
    /// Event kind for generated events
    event_kind: EventKind,
    /// Parser information
    info: ParserInfo,
    /// Source name for events
    source_name: String,
}

impl JsonEventParser {
    /// Apply field mappings to raw data
    #[must_use]
    #[inline]
    fn apply_mappings(&self, data: &serde_json::Value) -> serde_json::Value {
        let mut result = serde_json::Map::new();

        // Apply field mappings
        for (field, source_path) in &self.config.field_mappings {
            if let Some(value) = Self::extract_field(data, source_path) {
                result.insert(field.clone(), value);
            }
        }

        // Apply defaults for missing fields
        for (field, default_value) in &self.config.defaults {
            if !result.contains_key(field.as_str()) {
                result.insert(field.clone(), default_value.clone());
            }
        }

        serde_json::Value::Object(result)
    }

    /// Extract a field from JSON using JSONPath-like syntax
    #[must_use]
    #[inline]
    fn extract_field(data: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = data;

        for part in parts {
            match *current {
                serde_json::Value::Object(ref obj) => {
                    current = obj.get(part)?;
                }
                serde_json::Value::Array(ref arr) => {
                    if let Ok(index) = part.parse::<usize>() {
                        current = arr.get(index)?;
                    } else {
                        return None;
                    }
                }
                serde_json::Value::Null
                | serde_json::Value::Bool(_)
                | serde_json::Value::Number(_)
                | serde_json::Value::String(_) => return None,
            }
        }

        Some(current.clone())
    }

    /// Get the string representation of a JSON value's type
    #[inline]
    const fn get_json_type(value: &serde_json::Value) -> &'static str {
        match *value {
            serde_json::Value::String(_) => "string",
            serde_json::Value::Number(_) => "number",
            serde_json::Value::Bool(_) => "boolean",
            serde_json::Value::Array(_) => "array",
            serde_json::Value::Object(_) => "object",
            serde_json::Value::Null => "null",
        }
    }

    /// Create a new JSON event parser
    #[must_use]
    #[inline]
    pub fn new(config: ParsingConfig, event_kind: EventKind, source_name: String) -> Self {
        let info = ParserInfo::new(config.name.clone(), config.version.clone())
            .with_kind(event_kind.clone())
            .with_format(config.format.clone());
        Self {
            config,
            event_kind,
            info,
            source_name,
        }
    }

    /// Validate extracted data against configuration rules
    #[inline]
    fn validate(&self, data: &serde_json::Value) -> EventResult<()> {
        let obj = data.as_object().ok_or_else(|| {
            EventError::parse_error(
                IoError::new(ErrorKind::InvalidData, "Expected object"),
                "Data is not a JSON object",
            )
        })?;

        self.validate_required_fields(obj)?;
        self.validate_field_types(obj)?;
        self.validate_numeric_ranges(obj)?;
        self.validate_string_patterns(obj)?;

        Ok(())
    }

    /// Validate field types match expected types
    #[inline]
    fn validate_field_types(
        &self,
        obj: &serde_json::Map<String, serde_json::Value>,
    ) -> EventResult<()> {
        for (field, expected_type) in &self.config.validation.field_types {
            if let Some(value) = obj.get(field) {
                let actual_type = Self::get_json_type(value);
                if actual_type != expected_type {
                    return Err(EventError::parse_error(
                        IoError::new(
                            ErrorKind::InvalidData,
                            format!(
                                "Type mismatch for field '{field}': expected {expected_type}, got {actual_type}"
                            ),
                        ),
                        format!("Field '{field}' has wrong type"),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Validate maximum values for numeric fields
    #[inline]
    fn validate_maximum_values(
        &self,
        obj: &serde_json::Map<String, serde_json::Value>,
    ) -> EventResult<()> {
        for (field, max_value) in &self.config.validation.max_values {
            if let Some(value) = obj.get(field) {
                if let Some(num) = value.as_f64() {
                    if num > *max_value {
                        return Err(EventError::parse_error(
                            IoError::new(
                                ErrorKind::InvalidData,
                                format!(
                                    "Value {num} for field '{field}' is above maximum {max_value}"
                                ),
                            ),
                            format!("Field '{field}' value too large"),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Validate minimum values for numeric fields
    #[inline]
    fn validate_minimum_values(
        &self,
        obj: &serde_json::Map<String, serde_json::Value>,
    ) -> EventResult<()> {
        for (field, min_value) in &self.config.validation.min_values {
            if let Some(value) = obj.get(field) {
                if let Some(num) = value.as_f64() {
                    if num < *min_value {
                        return Err(EventError::parse_error(
                            IoError::new(
                                ErrorKind::InvalidData,
                                format!(
                                    "Value {num} for field '{field}' is below minimum {min_value}"
                                ),
                            ),
                            format!("Field '{field}' value too small"),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Validate numeric fields are within specified ranges
    #[inline]
    fn validate_numeric_ranges(
        &self,
        obj: &serde_json::Map<String, serde_json::Value>,
    ) -> EventResult<()> {
        self.validate_minimum_values(obj)?;
        self.validate_maximum_values(obj)?;
        Ok(())
    }

    /// Validate a single string against a regex pattern
    #[inline]
    fn validate_regex_pattern(field: &str, string_value: &str, pattern: &str) -> EventResult<()> {
        match Regex::new(pattern) {
            Ok(regex) => {
                if !regex.is_match(string_value) {
                    return Err(EventError::parse_error(
                        IoError::new(
                            ErrorKind::InvalidData,
                            format!(
                                "String '{string_value}' for field '{field}' doesn't match pattern '{pattern}'"
                            ),
                        ),
                        format!("Field '{field}' doesn't match pattern"),
                    ));
                }
            }
            Err(regex_error) => {
                return Err(EventError::parse_error(
                    IoError::new(
                        ErrorKind::InvalidData,
                        format!(
                            "Invalid regex pattern '{pattern}' for field '{field}': {regex_error}"
                        ),
                    ),
                    format!("Invalid regex pattern for field '{field}'"),
                ));
            }
        }
        Ok(())
    }

    /// Validate that all required fields are present
    #[inline]
    fn validate_required_fields(
        &self,
        obj: &serde_json::Map<String, serde_json::Value>,
    ) -> EventResult<()> {
        for required_field in &self.config.validation.required_fields {
            if !obj.contains_key(required_field) {
                return Err(EventError::parse_error(
                    IoError::new(
                        ErrorKind::InvalidData,
                        format!("Missing required field: {required_field}"),
                    ),
                    format!("Required field '{required_field}' is missing"),
                ));
            }
        }
        Ok(())
    }

    /// Validate string fields match specified regex patterns
    #[inline]
    fn validate_string_patterns(
        &self,
        obj: &serde_json::Map<String, serde_json::Value>,
    ) -> EventResult<()> {
        for (field, pattern) in &self.config.validation.patterns {
            if let Some(value) = obj.get(field) {
                if let Some(string_value) = value.as_str() {
                    Self::validate_regex_pattern(field, string_value, pattern)?;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl EventParser for JsonEventParser {
    type Input = serde_json::Value;

    #[inline]
    fn can_parse(&self, input: &Self::Input) -> bool {
        // Check if input has expected structure based on field mappings
        for source_path in self.config.field_mappings.values() {
            if Self::extract_field(input, source_path).is_none() {
                return false;
            }
        }
        true
    }

    #[inline]
    fn info(&self) -> &ParserInfo {
        &self.info
    }

    #[inline]
    async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>> {
        // Apply field mappings and defaults
        let mapped_data = self.apply_mappings(&input);

        // Validate the mapped data
        self.validate(&mapped_data)?;

        // Create event metadata
        let event_id = mapped_data
            .get("id")
            .and_then(|v| v.as_str())
            .map_or_else(|| uuid::Uuid::new_v4().to_string(), ToString::to_string);

        let metadata =
            EventMetadata::new(event_id, self.event_kind.clone(), self.source_name.clone());

        // Create the event
        let event = GenericEvent::with_metadata(metadata, mapped_data);

        return Ok(vec![Box::new(event)]);
    }
}

/// Multi-format parser that can handle different data formats.
#[expect(clippy::module_name_repetitions)]
pub struct MultiFormatParser {
    /// Default format to use when detection fails
    default_format: String,
    /// Parser information
    info: ParserInfo,
    /// Parsers for different data formats
    parsers: HashMap<String, Arc<dyn EventParser<Input = Vec<u8>> + Send + Sync>>,
}

impl Debug for MultiFormatParser {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("MultiFormatParser")
            .field("parsers", &format!("{} parsers", self.parsers.len()))
            .field("default_format", &self.default_format)
            .finish_non_exhaustive()
    }
}

impl MultiFormatParser {
    /// Detect format from data content
    #[inline]
    fn detect_format(&self, data: &[u8]) -> String {
        // Simple format detection - can be made more sophisticated
        if data.starts_with(b"{") || data.starts_with(b"[") {
            return "json".to_owned();
        }
        if data.starts_with(b"<") {
            return "xml".to_owned();
        }
        self.default_format.clone()
    }

    /// Create a new multi-format parser
    #[must_use]
    #[inline]
    pub fn new(default_format: String) -> Self {
        let info = ParserInfo::new("multi-format".to_owned(), "1.0.0".to_owned())
            .with_format("json".to_owned())
            .with_format("xml".to_owned())
            .with_format("binary".to_owned());
        Self {
            default_format,
            info,
            parsers: HashMap::new(),
        }
    }

    /// Add a parser for a specific format
    #[must_use]
    #[inline]
    pub fn with_parser<P>(mut self, format: String, parser: P) -> Self
    where
        P: EventParser<Input = Vec<u8>> + Send + Sync + 'static,
    {
        self.parsers.insert(format, Arc::new(parser));
        self
    }
}

#[async_trait]
impl EventParser for MultiFormatParser {
    type Input = Vec<u8>;

    #[inline]
    fn can_parse(&self, input: &Self::Input) -> bool {
        let format = self.detect_format(input);
        {
            self.parsers
                .get(&format)
                .is_some_and(|p| p.can_parse(input))
        }
    }

    #[inline]
    fn info(&self) -> &ParserInfo {
        &self.info
    }

    #[inline]
    async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>> {
        let format = self.detect_format(&input);

        if let Some(parser) = self.parsers.get(&format) {
            return parser.parse(input).await;
        }
        return Err(EventError::parse_error(
            IoError::new(ErrorKind::NotFound, "Parser not found"),
            format!("No parser available for format: {format}"),
        ));
    }
}

/// Binary data parser for handling raw bytes.
#[derive(Debug)]
#[expect(clippy::module_name_repetitions)]
pub struct BinaryEventParser {
    /// Event kind for generated events
    event_kind: EventKind,
    /// Parser information
    info: ParserInfo,
    /// Source name for events
    source_name: String,
}

impl BinaryEventParser {
    /// Create a new binary event parser
    #[must_use]
    #[inline]
    pub fn new(name: String, version: String, event_kind: EventKind, source_name: String) -> Self {
        let info = ParserInfo::new(name, version)
            .with_kind(event_kind.clone())
            .with_format("binary".to_owned());
        Self {
            event_kind,
            info,
            source_name,
        }
    }
}

#[async_trait]
impl EventParser for BinaryEventParser {
    type Input = Vec<u8>;

    #[inline]
    fn can_parse(&self, _input: &Self::Input) -> bool {
        true // Can always parse binary data
    }

    #[inline]
    fn info(&self) -> &ParserInfo {
        &self.info
    }

    #[inline]
    async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>> {
        // Create metadata for binary data
        let event_id = format!("binary_{}", uuid::Uuid::new_v4());
        let metadata =
            EventMetadata::new(event_id, self.event_kind.clone(), self.source_name.clone());

        // Convert binary data to hex representation for storage
        let hex_data = hex::encode(&input);
        let data = serde_json::json!({
            "format": "binary",
            "hex": hex_data,
            "length": input.len()
        });

        let event = GenericEvent::with_metadata(metadata, data);
        return Ok(vec![Box::new(event)]);
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::types::EventKind;
    use serde_json::json;

    #[tokio::test]
    async fn test_json_parser_basic() {
        let config = ParsingConfig::new(
            "test-parser".to_string(),
            "1.0.0".to_string(),
            "json".to_string(),
        )
        .with_mapping("amount".to_string(), "transaction.amount".to_string())
        .with_mapping("token".to_string(), "transaction.token".to_string())
        .with_default("fee".to_string(), 0.01);

        let parser =
            JsonEventParser::new(config, EventKind::Transaction, "test-source".to_string());

        let input = json!({
            "transaction": {
                "amount": 100,
                "token": "USDC"
            }
        });

        let events = parser
            .parse(input)
            .await
            .expect("Failed to parse test input for basic JSON parser test");
        assert_eq!(events.len(), 1);

        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.kind(), &EventKind::Transaction);
        assert_eq!(event.source(), "test-source");
    }

    #[tokio::test]
    async fn test_json_parser_validation() {
        let config = ParsingConfig::new(
            "test-parser".to_string(),
            "1.0.0".to_string(),
            "json".to_string(),
        )
        .with_mapping("amount".to_string(), "amount".to_string())
        .with_validation(
            ValidationConfig::default()
                .with_required("amount".to_string())
                .with_range("amount".to_string(), 0.0, 1000.0),
        );

        let parser =
            JsonEventParser::new(config, EventKind::Transaction, "test-source".to_string());

        // Test missing required field
        let input = json!({"other": "value"});
        let result = parser.parse(input).await;
        assert!(result.is_err());

        // Test valid input
        let input = json!({"amount": 100});
        let result = parser.parse(input).await;
        assert!(result.is_ok());

        // Test out of range value
        let input = json!({"amount": 2000});
        let result = parser.parse(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_binary_parser() {
        let parser = BinaryEventParser::new(
            "binary-parser".to_string(),
            "1.0.0".to_string(),
            EventKind::Contract,
            "binary-source".to_string(),
        );

        let input = vec![0x01, 0x02, 0x03, 0x04];
        let events = parser
            .parse(input.clone())
            .await
            .expect("Failed to parse test input for binary parser test");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.kind(), &EventKind::Contract);
        assert_eq!(event.source(), "binary-source");
    }

    #[test]
    fn test_parsing_config_builder() {
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string())
                .with_mapping("field1".to_string(), "source.field1".to_string())
                .with_default("field2".to_string(), "default_value")
                .with_validation(
                    ValidationConfig::default()
                        .with_required("field1".to_string())
                        .with_type("field1".to_string(), "string".to_string()),
                );

        assert_eq!(config.field_mappings.len(), 1);
        assert_eq!(config.defaults.len(), 1);
        assert_eq!(config.validation.required_fields.len(), 1);
        assert_eq!(config.validation.field_types.len(), 1);
    }

    #[test]
    fn test_validation_config_builder() {
        let validation = ValidationConfig::default()
            .with_required("amount".to_string())
            .with_type("amount".to_string(), "number".to_string())
            .with_range("amount".to_string(), 0.0, 1000.0)
            .with_pattern("token".to_string(), "^[A-Z]+$".to_string());

        assert_eq!(validation.required_fields.len(), 1);
        assert_eq!(validation.field_types.len(), 1);
        assert_eq!(validation.min_values.len(), 1);
        assert_eq!(validation.max_values.len(), 1);
        assert_eq!(validation.patterns.len(), 1);
    }

    // Additional comprehensive tests for 100% coverage

    #[test]
    fn test_parsing_config_with_default_serialization_error() {
        // Test with a value that cannot be serialized to JSON
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string());
        // This should not add the default since serialization would fail for certain types
        // but our current implementation uses serde_json::to_value which handles most types
        let config = config.with_default("field".to_string(), "valid_string");
        assert_eq!(config.defaults.len(), 1);
    }

    #[test]
    fn test_json_parser_extract_field_nested_object() {
        let data = json!({
            "level1": {
                "level2": {
                    "value": "found"
                }
            }
        });

        let result = JsonEventParser::extract_field(&data, "level1.level2.value");
        assert_eq!(result, Some(json!("found")));
    }

    #[test]
    fn test_json_parser_extract_field_array_access() {
        let data = json!({
            "items": [
                {"name": "first"},
                {"name": "second"}
            ]
        });

        let result = JsonEventParser::extract_field(&data, "items.0.name");
        assert_eq!(result, Some(json!("first")));

        let result = JsonEventParser::extract_field(&data, "items.1.name");
        assert_eq!(result, Some(json!("second")));
    }

    #[test]
    fn test_json_parser_extract_field_array_invalid_index() {
        let data = json!({
            "items": [{"name": "first"}]
        });

        // Index out of bounds
        let result = JsonEventParser::extract_field(&data, "items.5.name");
        assert_eq!(result, None);

        // Non-numeric index on array
        let result = JsonEventParser::extract_field(&data, "items.invalid.name");
        assert_eq!(result, None);
    }

    #[test]
    fn test_json_parser_extract_field_invalid_path() {
        let data = json!({
            "value": "string"
        });

        // Trying to access property on string
        let result = JsonEventParser::extract_field(&data, "value.nonexistent");
        assert_eq!(result, None);

        // Non-existent path
        let result = JsonEventParser::extract_field(&data, "nonexistent.path");
        assert_eq!(result, None);
    }

    #[test]
    fn test_json_parser_extract_field_empty_path() {
        let data = json!({"key": "value"});

        // Single part path
        let result = JsonEventParser::extract_field(&data, "key");
        assert_eq!(result, Some(json!("value")));
    }

    #[tokio::test]
    async fn test_json_parser_validation_missing_required() {
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string())
                .with_validation(
                    ValidationConfig::default().with_required("required_field".to_string()),
                );

        let parser = JsonEventParser::new(config, EventKind::Transaction, "test".to_string());
        let input = json!({"other_field": "value"});

        let result = parser.parse(input).await;
        assert!(result.is_err());

        if let Err(EventError::ParseError { context, .. }) = result {
            assert!(context.contains("Required field 'required_field' is missing"));
        } else {
            panic!("Expected ParseError but got different error type");
        }
    }

    #[tokio::test]
    async fn test_json_parser_validation_type_mismatch() {
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string())
                .with_mapping("field".to_string(), "field".to_string())
                .with_validation(
                    ValidationConfig::default()
                        .with_type("field".to_string(), "string".to_string()),
                );

        let parser = JsonEventParser::new(config, EventKind::Transaction, "test".to_string());
        let input = json!({"field": 123}); // number instead of string

        let result = parser.parse(input).await;
        assert!(result.is_err());

        if let Err(EventError::ParseError { context, .. }) = result {
            assert!(context.contains("Field 'field' has wrong type"));
        } else {
            panic!("Expected ParseError but got different error type");
        }
    }

    #[tokio::test]
    async fn test_json_parser_validation_all_types() {
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string())
                .with_mapping("str_field".to_string(), "str_field".to_string())
                .with_mapping("num_field".to_string(), "num_field".to_string())
                .with_mapping("bool_field".to_string(), "bool_field".to_string())
                .with_mapping("arr_field".to_string(), "arr_field".to_string())
                .with_mapping("obj_field".to_string(), "obj_field".to_string())
                .with_mapping("null_field".to_string(), "null_field".to_string())
                .with_validation(
                    ValidationConfig::default()
                        .with_type("str_field".to_string(), "string".to_string())
                        .with_type("num_field".to_string(), "number".to_string())
                        .with_type("bool_field".to_string(), "boolean".to_string())
                        .with_type("arr_field".to_string(), "array".to_string())
                        .with_type("obj_field".to_string(), "object".to_string())
                        .with_type("null_field".to_string(), "null".to_string()),
                );

        let parser = JsonEventParser::new(config, EventKind::Transaction, "test".to_string());
        let input = json!({
            "str_field": "string",
            "num_field": 123,
            "bool_field": true,
            "arr_field": [1, 2, 3],
            "obj_field": {"key": "value"},
            "null_field": null
        });

        let result = parser.parse(input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_json_parser_validation_numeric_range_below_min() {
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string())
                .with_mapping("amount".to_string(), "amount".to_string())
                .with_validation(ValidationConfig::default().with_range(
                    "amount".to_string(),
                    10.0,
                    100.0,
                ));

        let parser = JsonEventParser::new(config, EventKind::Transaction, "test".to_string());
        let input = json!({"amount": 5.0});

        let result = parser.parse(input).await;
        assert!(result.is_err());

        if let Err(EventError::ParseError { context, .. }) = result {
            assert!(context.contains("Field 'amount' value too small"));
        } else {
            panic!("Expected ParseError but got different error type");
        }
    }

    #[tokio::test]
    async fn test_json_parser_validation_numeric_range_above_max() {
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string())
                .with_mapping("amount".to_string(), "amount".to_string())
                .with_validation(ValidationConfig::default().with_range(
                    "amount".to_string(),
                    10.0,
                    100.0,
                ));

        let parser = JsonEventParser::new(config, EventKind::Transaction, "test".to_string());
        let input = json!({"amount": 150.0});

        let result = parser.parse(input).await;
        assert!(result.is_err());

        if let Err(EventError::ParseError { context, .. }) = result {
            assert!(context.contains("Field 'amount' value too large"));
        } else {
            panic!("Expected ParseError but got different error type");
        }
    }

    #[tokio::test]
    async fn test_json_parser_validation_string_pattern_mismatch() {
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string())
                .with_mapping("token".to_string(), "token".to_string())
                .with_validation(
                    ValidationConfig::default()
                        .with_pattern("token".to_string(), "^USD[C|T]$".to_string()),
                );

        let parser = JsonEventParser::new(config, EventKind::Transaction, "test".to_string());
        let input = json!({"token": "BTC"});

        let result = parser.parse(input).await;
        assert!(result.is_err());

        if let Err(EventError::ParseError { context, .. }) = result {
            assert!(context.contains("Field 'token' doesn't match pattern"));
        } else {
            panic!("Expected ParseError but got different error type");
        }
    }

    #[tokio::test]
    async fn test_json_parser_validation_string_pattern_match() {
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string())
                .with_mapping("token".to_string(), "token".to_string())
                .with_validation(
                    ValidationConfig::default()
                        .with_pattern("token".to_string(), "^USD[C|T]$".to_string()),
                );

        let parser = JsonEventParser::new(config, EventKind::Transaction, "test".to_string());
        let input = json!({"token": "USDC"});

        let result = parser.parse(input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_json_parser_validation_invalid_regex_pattern() {
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string())
                .with_mapping("token".to_string(), "token".to_string())
                .with_validation(
                    ValidationConfig::default()
                        .with_pattern("token".to_string(), "[invalid".to_string()),
                );

        let parser = JsonEventParser::new(config, EventKind::Transaction, "test".to_string());
        let input = json!({"token": "USDC"});

        let result = parser.parse(input).await;
        assert!(result.is_err());

        if let Err(EventError::ParseError { context, .. }) = result {
            assert!(context.contains("Invalid regex pattern for field 'token'"));
        } else {
            panic!("Expected ParseError but got different error type");
        }
    }

    #[tokio::test]
    async fn test_json_parser_validation_non_object_input() {
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string())
                .with_validation(
                    ValidationConfig::default().with_required("some_field".to_string()),
                );
        let parser = JsonEventParser::new(config, EventKind::Transaction, "test".to_string());

        let input = json!("not an object");
        let result = parser.parse(input).await;
        assert!(result.is_err());

        // The error should be about missing required field since apply_mappings converts
        // any input to a valid (but potentially empty) JSON object
        if let Err(EventError::ParseError { context, .. }) = result {
            assert!(context.contains("Required field 'some_field' is missing"));
        } else {
            panic!("Expected ParseError but got different error type");
        }
    }

    #[tokio::test]
    async fn test_json_parser_with_custom_id() {
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string())
                .with_mapping("id".to_string(), "custom_id".to_string());

        let parser = JsonEventParser::new(config, EventKind::Transaction, "test".to_string());
        let input = json!({"custom_id": "my-custom-id"});

        let events = parser
            .parse(input)
            .await
            .expect("Failed to parse test input with custom ID");
        assert_eq!(events.len(), 1);
        assert_eq!(
            events.first().expect("Expected at least one event").id(),
            "my-custom-id"
        );
    }

    #[test]
    fn test_json_parser_can_parse_valid() {
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string())
                .with_mapping("field".to_string(), "nested.field".to_string());

        let parser = JsonEventParser::new(config, EventKind::Transaction, "test".to_string());
        let input = json!({"nested": {"field": "value"}});

        assert!(parser.can_parse(&input));
    }

    #[test]
    fn test_json_parser_can_parse_invalid() {
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string())
                .with_mapping("field".to_string(), "nested.field".to_string());

        let parser = JsonEventParser::new(config, EventKind::Transaction, "test".to_string());
        let input = json!({"other": "value"});

        assert!(!parser.can_parse(&input));
    }

    #[test]
    fn test_json_parser_can_parse_empty_mappings() {
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string());
        let parser = JsonEventParser::new(config, EventKind::Transaction, "test".to_string());
        let input = json!({"any": "value"});

        assert!(parser.can_parse(&input)); // No mappings to check
    }

    #[test]
    fn test_json_parser_info() {
        let config = ParsingConfig::new(
            "test-parser".to_string(),
            "2.0.0".to_string(),
            "json".to_string(),
        );
        let parser = JsonEventParser::new(config, EventKind::Contract, "test".to_string());

        let info = parser.info();
        assert_eq!(info.name, "test-parser");
        assert_eq!(info.version, "2.0.0");
        assert!(info.supported_kinds.contains(&EventKind::Contract));
        assert!(info.supported_formats.contains(&"json".to_string()));
    }

    #[test]
    fn test_multi_format_parser_new() {
        let parser = MultiFormatParser::new("binary".to_string());
        assert_eq!(parser.default_format, "binary");
        assert_eq!(parser.parsers.len(), 0);
    }

    #[test]
    fn test_multi_format_parser_detect_format_json() {
        let parser = MultiFormatParser::new("binary".to_string());

        assert_eq!(parser.detect_format(b"{\"key\": \"value\"}"), "json");
        assert_eq!(parser.detect_format(b"[1, 2, 3]"), "json");
    }

    #[test]
    fn test_multi_format_parser_detect_format_xml() {
        let parser = MultiFormatParser::new("binary".to_string());

        assert_eq!(parser.detect_format(b"<xml>content</xml>"), "xml");
    }

    #[test]
    fn test_multi_format_parser_detect_format_default() {
        let parser = MultiFormatParser::new("custom".to_string());

        assert_eq!(parser.detect_format(b"random binary data"), "custom");
    }

    #[tokio::test]
    async fn test_multi_format_parser_parse_no_parser() {
        let parser = MultiFormatParser::new("unknown".to_string());
        let input = b"some data".to_vec();

        let result = parser.parse(input).await;
        assert!(result.is_err());

        if let Err(EventError::ParseError { context, .. }) = result {
            assert!(context.contains("No parser available for format: unknown"));
        } else {
            panic!("Expected ParseError but got different error type");
        }
    }

    #[test]
    fn test_multi_format_parser_can_parse_no_parser() {
        let parser = MultiFormatParser::new("unknown".to_string());
        let input = b"some data".to_vec();

        assert!(!parser.can_parse(&input));
    }

    #[test]
    fn test_multi_format_parser_info() {
        let parser = MultiFormatParser::new("default".to_string());
        let info = parser.info();

        assert_eq!(info.name, "multi-format");
        assert_eq!(info.version, "1.0.0");
        // Should have multiple formats
        assert!(!info.supported_formats.is_empty());
    }

    #[test]
    fn test_binary_parser_new() {
        let parser = BinaryEventParser::new(
            "bin-parser".to_string(),
            "1.5.0".to_string(),
            EventKind::Contract,
            "bin-source".to_string(),
        );

        assert_eq!(parser.info.name, "bin-parser");
        assert_eq!(parser.info.version, "1.5.0");
        assert_eq!(parser.event_kind, EventKind::Contract);
        assert_eq!(parser.source_name, "bin-source");
    }

    #[tokio::test]
    async fn test_binary_parser_parse_empty() {
        let parser = BinaryEventParser::new(
            "test".to_string(),
            "1.0.0".to_string(),
            EventKind::Transaction,
            "test".to_string(),
        );

        let input = vec![];
        let events = parser
            .parse(input)
            .await
            .expect("Failed to parse empty binary input for test");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");
        assert_eq!(event.kind(), &EventKind::Transaction);
        assert_eq!(event.source(), "test");

        // Check data structure by downcasting to GenericEvent
        let generic_event = event
            .as_any()
            .downcast_ref::<GenericEvent>()
            .expect("Failed to downcast event to GenericEvent for test verification");
        assert_eq!(
            generic_event
                .data
                .get("format")
                .expect("Expected format field"),
            &serde_json::json!("binary")
        );
        assert_eq!(
            generic_event.data.get("hex").expect("Expected hex field"),
            &serde_json::json!("")
        );
        assert_eq!(
            generic_event
                .data
                .get("length")
                .expect("Expected length field"),
            &serde_json::json!(0)
        );
    }

    #[tokio::test]
    async fn test_binary_parser_parse_with_data() {
        let parser = BinaryEventParser::new(
            "test".to_string(),
            "1.0.0".to_string(),
            EventKind::Transaction,
            "test".to_string(),
        );

        let input = vec![0xde, 0xad, 0xbe, 0xef];
        let events = parser
            .parse(input.clone())
            .await
            .expect("Failed to parse binary input with data for test");

        assert_eq!(events.len(), 1);
        let event = events.first().expect("Expected at least one event");

        let generic_event = event
            .as_any()
            .downcast_ref::<GenericEvent>()
            .expect("Failed to downcast event to GenericEvent for data verification");
        assert_eq!(
            generic_event
                .data
                .get("format")
                .expect("Expected format field"),
            &serde_json::json!("binary")
        );
        assert_eq!(
            generic_event.data.get("hex").expect("Expected hex field"),
            &serde_json::json!("deadbeef")
        );
        assert_eq!(
            generic_event
                .data
                .get("length")
                .expect("Expected length field"),
            &serde_json::json!(4)
        );
    }

    #[test]
    fn test_binary_parser_can_parse() {
        let parser = BinaryEventParser::new(
            "test".to_string(),
            "1.0.0".to_string(),
            EventKind::Transaction,
            "test".to_string(),
        );

        assert!(parser.can_parse(&vec![]));
        assert!(parser.can_parse(&vec![1, 2, 3]));
        assert!(parser.can_parse(&vec![0; 1000]));
    }

    #[test]
    fn test_binary_parser_info() {
        let parser = BinaryEventParser::new(
            "binary-test".to_string(),
            "2.1.0".to_string(),
            EventKind::External,
            "test".to_string(),
        );

        let info = parser.info();
        assert_eq!(info.name, "binary-test");
        assert_eq!(info.version, "2.1.0");
        assert!(info.supported_kinds.contains(&EventKind::External));
        assert!(info.supported_formats.contains(&"binary".to_string()));
    }

    #[test]
    fn test_validation_config_default() {
        let validation = ValidationConfig::default();

        assert!(validation.required_fields.is_empty());
        assert!(validation.field_types.is_empty());
        assert!(validation.min_values.is_empty());
        assert!(validation.max_values.is_empty());
        assert!(validation.patterns.is_empty());
    }

    #[test]
    fn test_validation_config_multiple_required() {
        let validation = ValidationConfig::default()
            .with_required("field1".to_string())
            .with_required("field2".to_string());

        assert_eq!(validation.required_fields.len(), 2);
        assert!(validation.required_fields.contains(&"field1".to_string()));
        assert!(validation.required_fields.contains(&"field2".to_string()));
    }

    #[test]
    fn test_validation_config_multiple_types() {
        let validation = ValidationConfig::default()
            .with_type("field1".to_string(), "string".to_string())
            .with_type("field2".to_string(), "number".to_string());

        assert_eq!(validation.field_types.len(), 2);
        assert_eq!(
            validation.field_types.get("field1"),
            Some(&"string".to_string())
        );
        assert_eq!(
            validation.field_types.get("field2"),
            Some(&"number".to_string())
        );
    }

    #[test]
    fn test_validation_config_range_sets_both_min_max() {
        let validation = ValidationConfig::default().with_range("amount".to_string(), 5.0, 10.0);

        assert_eq!(validation.min_values.get("amount"), Some(&5.0));
        assert_eq!(validation.max_values.get("amount"), Some(&10.0));
    }

    #[tokio::test]
    async fn test_json_parser_validation_non_numeric_range_field() {
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string())
                .with_mapping("field".to_string(), "field".to_string())
                .with_validation(ValidationConfig::default().with_range(
                    "field".to_string(),
                    0.0,
                    100.0,
                ));

        let parser = JsonEventParser::new(config, EventKind::Transaction, "test".to_string());
        let input = json!({"field": "not a number"});

        // Should not error for non-numeric fields when range validation is specified
        let result = parser.parse(input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_json_parser_validation_non_string_pattern_field() {
        let config =
            ParsingConfig::new("test".to_string(), "1.0.0".to_string(), "json".to_string())
                .with_mapping("field".to_string(), "field".to_string())
                .with_validation(
                    ValidationConfig::default()
                        .with_pattern("field".to_string(), "pattern".to_string()),
                );

        let parser = JsonEventParser::new(config, EventKind::Transaction, "test".to_string());
        let input = json!({"field": 123});

        // Should not error for non-string fields when pattern validation is specified
        let result = parser.parse(input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multi_format_parser_with_parser() {
        let binary_parser = BinaryEventParser::new(
            "test-binary".to_string(),
            "1.0.0".to_string(),
            EventKind::Transaction,
            "test".to_string(),
        );

        let multi_parser = MultiFormatParser::new("default".to_string())
            .with_parser("binary".to_string(), binary_parser);

        // Test parsing with the added parser
        let input = b"random binary data".to_vec(); // Will be detected as default format
        let result = multi_parser.parse(input).await;

        // Should fail because "default" format parser is not registered, only "binary" is
        assert!(result.is_err());

        // Test with JSON format detection and parsing
        let json_input = b"{\"test\": \"data\"}".to_vec();
        let result = multi_parser.parse(json_input).await;
        // Should fail because no JSON parser is registered
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multi_format_parser_with_json_format() {
        let binary_parser = BinaryEventParser::new(
            "test-binary".to_string(),
            "1.0.0".to_string(),
            EventKind::Transaction,
            "test".to_string(),
        );

        let multi_parser = MultiFormatParser::new("binary".to_string())
            .with_parser("binary".to_string(), binary_parser);

        // Test with data that will be detected as JSON but no JSON parser is registered
        let json_input = b"{\"test\": \"data\"}".to_vec();
        let result = multi_parser.parse(json_input).await;
        assert!(result.is_err());

        // Test with data that will use default format (binary)
        let binary_input = b"some binary data".to_vec();
        let result = multi_parser.parse(binary_input).await;
        assert!(result.is_ok());

        let events = result.expect("Failed to parse multi-format input for test");
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_multi_format_parser_can_parse_with_registered_parser() {
        let binary_parser = BinaryEventParser::new(
            "test-binary".to_string(),
            "1.0.0".to_string(),
            EventKind::Transaction,
            "test".to_string(),
        );

        let multi_parser = MultiFormatParser::new("binary".to_string())
            .with_parser("binary".to_string(), binary_parser);

        // Test data that defaults to binary format
        let input = b"random data".to_vec();
        assert!(multi_parser.can_parse(&input));

        // Test data that would be detected as JSON but no JSON parser
        let json_input = b"{\"test\": \"data\"}".to_vec();
        assert!(!multi_parser.can_parse(&json_input));
    }
}
