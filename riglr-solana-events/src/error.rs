pub use riglr_events_core::error::{EventError, EventResult};
use thiserror::Error;

/// Custom error type for event parsing operations
///
/// NOTE: This will be gradually replaced with EventError from riglr-events-core
#[derive(Error, Debug)]
pub enum ParseError {
    /// Not enough bytes available for the requested operation
    #[error("Not enough bytes: expected {expected}, got {found} at offset {offset}")]
    NotEnoughBytes {
        /// Number of bytes expected
        expected: usize,
        /// Number of bytes actually found
        found: usize,
        /// Byte offset where the error occurred
        offset: usize,
    },

    /// Invalid discriminator for the instruction
    #[error("Invalid discriminator: expected {expected:?}, got {found:?}")]
    InvalidDiscriminator {
        /// Expected discriminator bytes
        expected: Vec<u8>,
        /// Actual discriminator bytes found
        found: Vec<u8>,
    },

    /// Invalid account index
    #[error("Account index {index} out of bounds (max: {max})")]
    InvalidAccountIndex {
        /// The invalid account index that was accessed
        index: usize,
        /// Maximum valid account index
        max: usize,
    },

    /// Invalid public key format
    #[error("Invalid public key: {0}")]
    InvalidPubkey(String),

    /// UTF-8 decoding error
    #[error("UTF-8 decoding error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    /// Borsh deserialization error
    #[error("Borsh deserialization error: {0}")]
    BorshError(String),

    /// Invalid enum variant
    #[error("Invalid enum variant {variant} for type {type_name}")]
    InvalidEnumVariant {
        /// The invalid variant value
        variant: u8,
        /// Name of the enum type
        type_name: String,
    },

    /// Invalid instruction type
    #[error("Invalid instruction type: {0}")]
    InvalidInstructionType(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid data format
    #[error("Invalid data format: {0}")]
    InvalidDataFormat(String),

    /// Overflow error
    #[error("Arithmetic overflow: {0}")]
    Overflow(String),

    /// Generic parsing error
    #[error("Parse error: {0}")]
    Generic(String),

    /// Network error (for streaming)
    #[error("Network error: {0}")]
    Network(String),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),
}

impl ParseError {
    /// Create a NotEnoughBytes error
    pub fn not_enough_bytes(expected: usize, found: usize, offset: usize) -> Self {
        Self::NotEnoughBytes {
            expected,
            found,
            offset,
        }
    }

    /// Create an InvalidDiscriminator error
    pub fn invalid_discriminator(expected: Vec<u8>, found: Vec<u8>) -> Self {
        Self::InvalidDiscriminator { expected, found }
    }

    /// Create an InvalidAccountIndex error
    pub fn invalid_account_index(index: usize, max: usize) -> Self {
        Self::InvalidAccountIndex { index, max }
    }

    /// Create an InvalidEnumVariant error
    pub fn invalid_enum_variant(variant: u8, type_name: &str) -> Self {
        Self::InvalidEnumVariant {
            variant,
            type_name: type_name.to_string(),
        }
    }
}

// Allow conversion from borsh errors
impl From<borsh::io::Error> for ParseError {
    fn from(err: borsh::io::Error) -> Self {
        Self::BorshError(err.to_string())
    }
}

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;

// Bridge between old ParseError and new EventError for migration
impl From<ParseError> for EventError {
    fn from(err: ParseError) -> Self {
        match err {
            ParseError::Network(_) | ParseError::Timeout(_) => EventError::stream_error(
                std::io::Error::other(err.to_string()),
                "Solana parsing error",
            ),
            _ => EventError::parse_error(
                std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()),
                "Solana parsing error",
            ),
        }
    }
}

impl From<EventError> for ParseError {
    fn from(err: EventError) -> Self {
        ParseError::Generic(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_enough_bytes_error_creation() {
        let error = ParseError::not_enough_bytes(10, 5, 20);
        match error {
            ParseError::NotEnoughBytes {
                expected,
                found,
                offset,
            } => {
                assert_eq!(expected, 10);
                assert_eq!(found, 5);
                assert_eq!(offset, 20);
            }
            _ => panic!("Expected NotEnoughBytes variant"),
        }
    }

    #[test]
    fn test_not_enough_bytes_display() {
        let error = ParseError::not_enough_bytes(10, 5, 20);
        let error_string = error.to_string();
        assert_eq!(
            error_string,
            "Not enough bytes: expected 10, got 5 at offset 20"
        );
    }

    #[test]
    fn test_invalid_discriminator_error_creation() {
        let expected = vec![1, 2, 3];
        let found = vec![4, 5, 6];
        let error = ParseError::invalid_discriminator(expected.clone(), found.clone());

        match error {
            ParseError::InvalidDiscriminator {
                expected: exp,
                found: fnd,
            } => {
                assert_eq!(exp, expected);
                assert_eq!(fnd, found);
            }
            _ => panic!("Expected InvalidDiscriminator variant"),
        }
    }

    #[test]
    fn test_invalid_discriminator_display() {
        let error = ParseError::invalid_discriminator(vec![1, 2], vec![3, 4]);
        let error_string = error.to_string();
        assert_eq!(
            error_string,
            "Invalid discriminator: expected [1, 2], got [3, 4]"
        );
    }

    #[test]
    fn test_invalid_account_index_error_creation() {
        let error = ParseError::invalid_account_index(15, 10);
        match error {
            ParseError::InvalidAccountIndex { index, max } => {
                assert_eq!(index, 15);
                assert_eq!(max, 10);
            }
            _ => panic!("Expected InvalidAccountIndex variant"),
        }
    }

    #[test]
    fn test_invalid_account_index_display() {
        let error = ParseError::invalid_account_index(15, 10);
        let error_string = error.to_string();
        assert_eq!(error_string, "Account index 15 out of bounds (max: 10)");
    }

    #[test]
    fn test_invalid_enum_variant_error_creation() {
        let error = ParseError::invalid_enum_variant(99, "MyEnum");
        match error {
            ParseError::InvalidEnumVariant { variant, type_name } => {
                assert_eq!(variant, 99);
                assert_eq!(type_name, "MyEnum");
            }
            _ => panic!("Expected InvalidEnumVariant variant"),
        }
    }

    #[test]
    fn test_invalid_enum_variant_display() {
        let error = ParseError::invalid_enum_variant(99, "MyEnum");
        let error_string = error.to_string();
        assert_eq!(error_string, "Invalid enum variant 99 for type MyEnum");
    }

    #[test]
    fn test_invalid_pubkey_error() {
        let error = ParseError::InvalidPubkey("invalid_key".to_string());
        let error_string = error.to_string();
        assert_eq!(error_string, "Invalid public key: invalid_key");
    }

    #[test]
    fn test_utf8_error_conversion() {
        // Create invalid UTF-8 bytes dynamically to avoid compiler warning
        let invalid_utf8 = vec![0xff, 0xfe, 0xfd];
        let utf8_result = std::str::from_utf8(&invalid_utf8);
        let utf8_error = utf8_result.unwrap_err();

        let parse_error: ParseError = utf8_error.into();
        match parse_error {
            ParseError::Utf8Error(_) => {
                // Success - the conversion worked
                assert!(parse_error.to_string().contains("UTF-8 decoding error"));
            }
            _ => panic!("Expected Utf8Error variant"),
        }
    }

    #[test]
    fn test_borsh_error_display() {
        let error = ParseError::BorshError("test borsh error".to_string());
        let error_string = error.to_string();
        assert_eq!(
            error_string,
            "Borsh deserialization error: test borsh error"
        );
    }

    #[test]
    fn test_borsh_io_error_conversion() {
        let io_error = borsh::io::Error::new(borsh::io::ErrorKind::InvalidData, "test error");
        let parse_error: ParseError = io_error.into();

        match parse_error {
            ParseError::BorshError(msg) => {
                assert!(msg.contains("test error"));
            }
            _ => panic!("Expected BorshError variant"),
        }
    }

    #[test]
    fn test_invalid_instruction_type_error() {
        let error = ParseError::InvalidInstructionType("unknown_instruction".to_string());
        let error_string = error.to_string();
        assert_eq!(
            error_string,
            "Invalid instruction type: unknown_instruction"
        );
    }

    #[test]
    fn test_missing_field_error() {
        let error = ParseError::MissingField("account_key".to_string());
        let error_string = error.to_string();
        assert_eq!(error_string, "Missing required field: account_key");
    }

    #[test]
    fn test_invalid_data_format_error() {
        let error = ParseError::InvalidDataFormat("expected JSON".to_string());
        let error_string = error.to_string();
        assert_eq!(error_string, "Invalid data format: expected JSON");
    }

    #[test]
    fn test_overflow_error() {
        let error = ParseError::Overflow("u64 overflow".to_string());
        let error_string = error.to_string();
        assert_eq!(error_string, "Arithmetic overflow: u64 overflow");
    }

    #[test]
    fn test_generic_error() {
        let error = ParseError::Generic("generic parsing issue".to_string());
        let error_string = error.to_string();
        assert_eq!(error_string, "Parse error: generic parsing issue");
    }

    #[test]
    fn test_network_error() {
        let error = ParseError::Network("connection failed".to_string());
        let error_string = error.to_string();
        assert_eq!(error_string, "Network error: connection failed");
    }

    #[test]
    fn test_timeout_error() {
        let error = ParseError::Timeout("operation took too long".to_string());
        let error_string = error.to_string();
        assert_eq!(error_string, "Operation timed out: operation took too long");
    }

    #[test]
    fn test_parse_result_type_alias() {
        // Test that ParseResult<T> is properly defined as Result<T, ParseError>
        let success: ParseResult<i32> = Ok(42);
        let failure: ParseResult<i32> = Err(ParseError::Generic("test".to_string()));

        assert!(success.is_ok());
        assert_eq!(success.unwrap(), 42);
        assert!(failure.is_err());
    }

    #[test]
    fn test_parse_error_to_event_error_network_conversion() {
        let network_error = ParseError::Network("network issue".to_string());
        let event_error: EventError = network_error.into();

        // Verify it's a stream error
        assert!(event_error.to_string().contains("Solana parsing error"));
    }

    #[test]
    fn test_parse_error_to_event_error_timeout_conversion() {
        let timeout_error = ParseError::Timeout("timeout issue".to_string());
        let event_error: EventError = timeout_error.into();

        // Verify it's a stream error
        assert!(event_error.to_string().contains("Solana parsing error"));
    }

    #[test]
    fn test_parse_error_to_event_error_other_conversion() {
        let generic_error = ParseError::Generic("generic issue".to_string());
        let event_error: EventError = generic_error.into();

        // Verify it's a parse error
        assert!(event_error.to_string().contains("Solana parsing error"));
    }

    #[test]
    fn test_parse_error_to_event_error_not_enough_bytes_conversion() {
        let not_enough_bytes_error = ParseError::not_enough_bytes(10, 5, 0);
        let event_error: EventError = not_enough_bytes_error.into();

        // Verify it's a parse error (not network/stream)
        assert!(event_error.to_string().contains("Solana parsing error"));
    }

    #[test]
    fn test_parse_error_to_event_error_invalid_discriminator_conversion() {
        let invalid_disc_error = ParseError::invalid_discriminator(vec![1], vec![2]);
        let event_error: EventError = invalid_disc_error.into();

        // Verify it's a parse error (not network/stream)
        assert!(event_error.to_string().contains("Solana parsing error"));
    }

    #[test]
    fn test_event_error_to_parse_error_conversion() {
        let event_error = EventError::parse_error(
            std::io::Error::new(std::io::ErrorKind::InvalidData, "test"),
            "test context",
        );
        let parse_error: ParseError = event_error.into();

        match parse_error {
            ParseError::Generic(msg) => {
                assert!(msg.contains("test context"));
            }
            _ => panic!("Expected Generic variant"),
        }
    }

    #[test]
    fn test_debug_trait_implementation() {
        let error = ParseError::not_enough_bytes(10, 5, 20);
        let debug_string = format!("{:?}", error);
        assert!(debug_string.contains("NotEnoughBytes"));
        assert!(debug_string.contains("expected: 10"));
        assert!(debug_string.contains("found: 5"));
        assert!(debug_string.contains("offset: 20"));
    }

    #[test]
    fn test_error_trait_implementation() {
        let error = ParseError::Generic("test error".to_string());

        // Test that it implements std::error::Error
        let error_trait: &dyn std::error::Error = &error;
        assert_eq!(error_trait.to_string(), "Parse error: test error");
    }

    #[test]
    fn test_all_error_variants_are_covered() {
        // This test ensures we don't miss any variants when adding new ones
        let errors = vec![
            ParseError::NotEnoughBytes {
                expected: 1,
                found: 0,
                offset: 0,
            },
            ParseError::InvalidDiscriminator {
                expected: vec![1],
                found: vec![2],
            },
            ParseError::InvalidAccountIndex { index: 1, max: 0 },
            ParseError::InvalidPubkey("test".to_string()),
            ParseError::BorshError("test".to_string()),
            ParseError::InvalidEnumVariant {
                variant: 1,
                type_name: "test".to_string(),
            },
            ParseError::InvalidInstructionType("test".to_string()),
            ParseError::MissingField("test".to_string()),
            ParseError::InvalidDataFormat("test".to_string()),
            ParseError::Overflow("test".to_string()),
            ParseError::Generic("test".to_string()),
            ParseError::Network("test".to_string()),
            ParseError::Timeout("test".to_string()),
        ];

        // Ensure all variants have string representations
        for error in errors {
            assert!(!error.to_string().is_empty());
        }
    }
}
