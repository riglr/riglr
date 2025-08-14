use thiserror::Error;
pub use riglr_events_core::error::{EventError, EventResult};

/// Custom error type for event parsing operations
/// 
/// NOTE: This will be gradually replaced with EventError from riglr-events-core
#[derive(Error, Debug)]
pub enum ParseError {
    /// Not enough bytes available for the requested operation
    #[error("Not enough bytes: expected {expected}, got {found} at offset {offset}")]
    NotEnoughBytes {
        expected: usize,
        found: usize,
        offset: usize,
    },

    /// Invalid discriminator for the instruction
    #[error("Invalid discriminator: expected {expected:?}, got {found:?}")]
    InvalidDiscriminator {
        expected: Vec<u8>,
        found: Vec<u8>,
    },

    /// Invalid account index
    #[error("Account index {index} out of bounds (max: {max})")]
    InvalidAccountIndex {
        index: usize,
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
        variant: u8,
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
        Self::NotEnoughBytes { expected, found, offset }
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
            ParseError::Network(_) | ParseError::Timeout(_) => {
                EventError::stream_error(std::io::Error::new(std::io::ErrorKind::Other, err.to_string()), "Solana parsing error")
            }
            _ => {
                EventError::parse_error(std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()), "Solana parsing error")
            }
        }
    }
}

impl From<EventError> for ParseError {
    fn from(err: EventError) -> Self {
        ParseError::Generic(err.to_string())
    }
}