use riglr_core::error::ToolError;
use thiserror::Error;

/// Errors that can occur during cross-chain operations
#[derive(Error, Debug)]
pub enum Error {
    /// Bridge operation failed
    #[error("Bridge operation failed: {0}")]
    BridgeExecutionError(String),

    /// Insufficient liquidity for amount
    #[error("Insufficient liquidity for amount: {amount}")]
    InsufficientLiquidity {
        /// Amount that was requested but unavailable
        amount: String,
    },

    /// Invalid route configuration
    #[error("Invalid route configuration: {0}")]
    InvalidRoute(String),

    /// Li.fi API error
    #[error("Li.fi API error: {0}")]
    LifiApiError(String),

    /// Quote fetch failed
    #[error("Quote fetch failed: {0}")]
    QuoteFetchError(String),

    /// Core tool error
    #[error("Core tool error: {0}")]
    ToolError(#[from] ToolError),

    /// Unsupported chain pair
    #[error("Unsupported chain pair: {from_chain} -> {to_chain}")]
    UnsupportedChainPair {
        /// Source chain identifier
        from_chain: String,
        /// Destination chain identifier
        to_chain: String,
    },
}

impl From<Error> for ToolError {
    fn from(err: Error) -> Self {
        match err {
            Error::ToolError(tool_err) => tool_err,
            // Permanent errors - configuration/validation issues
            Error::InvalidRoute(_) | Error::UnsupportedChainPair { .. } => {
                Self::permanent_string(err.to_string())
            }
            // Retriable errors - network, API, execution, and liquidity issues
            Error::LifiApiError(_)
            | Error::QuoteFetchError(_)
            | Error::InsufficientLiquidity { .. }
            | Error::BridgeExecutionError(_) => {
                Self::retriable_string(err.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use core::error::Error as StdError;
    use riglr_core::error::ToolError;

    #[test]
    fn test_tool_error_variant_creation_and_display() {
        let tool_error = ToolError::permanent_string("Original tool error".to_string());
        let error = Error::ToolError(tool_error);

        assert_eq!(error.to_string(), "Core tool error: Permanent error: Original tool error - Original tool error");
    }

    #[test]
    fn test_lifi_api_error_variant_creation_and_display() {
        let error = Error::LifiApiError("API timeout".to_string());

        assert_eq!(error.to_string(), "Li.fi API error: API timeout");
    }

    #[test]
    fn test_quote_fetch_error_variant_creation_and_display() {
        let error = Error::QuoteFetchError("Network unreachable".to_string());

        assert_eq!(error.to_string(), "Quote fetch failed: Network unreachable");
    }

    #[test]
    fn test_invalid_route_variant_creation_and_display() {
        let error = Error::InvalidRoute("Invalid token address".to_string());

        assert_eq!(
            error.to_string(),
            "Invalid route configuration: Invalid token address"
        );
    }

    #[test]
    fn test_bridge_execution_error_variant_creation_and_display() {
        let error = Error::BridgeExecutionError("Transaction failed".to_string());

        assert_eq!(
            error.to_string(),
            "Bridge operation failed: Transaction failed"
        );
    }

    #[test]
    fn test_unsupported_chain_pair_variant_creation_and_display() {
        let error = Error::UnsupportedChainPair {
            from_chain: "ethereum".to_string(),
            to_chain: "bitcoin".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Unsupported chain pair: ethereum -> bitcoin"
        );
    }

    #[test]
    fn test_insufficient_liquidity_variant_creation_and_display() {
        let error = Error::InsufficientLiquidity {
            amount: "1000000".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Insufficient liquidity for amount: 1000000"
        );
    }

    #[test]
    fn test_from_tool_error_conversion() {
        let original_tool_error = ToolError::permanent_string("Original error".to_string());
        let error_message = original_tool_error.to_string();
        let cross_chain_error = Error::ToolError(original_tool_error);
        let converted_tool_error: ToolError = cross_chain_error.into();

        assert_eq!(converted_tool_error.to_string(), error_message);
    }

    #[test]
    fn test_from_lifi_api_error_conversion() {
        let error = Error::LifiApiError("API error".to_string());
        let tool_error: ToolError = error.into();

        assert_eq!(tool_error.to_string(), "Operation can be retried: Li.fi API error: API error - Li.fi API error: API error");
        assert!(tool_error.is_retriable());
    }

    #[test]
    fn test_from_quote_fetch_error_conversion() {
        let error = Error::QuoteFetchError("Fetch failed".to_string());
        let tool_error: ToolError = error.into();

        assert_eq!(tool_error.to_string(), "Operation can be retried: Quote fetch failed: Fetch failed - Quote fetch failed: Fetch failed");
        assert!(tool_error.is_retriable());
    }

    #[test]
    fn test_from_invalid_route_conversion() {
        let error = Error::InvalidRoute("Bad route".to_string());
        let tool_error: ToolError = error.into();

        assert_eq!(
            tool_error.to_string(),
            "Permanent error: Invalid route configuration: Bad route - Invalid route configuration: Bad route"
        );
        assert!(!tool_error.is_retriable());
    }

    #[test]
    fn test_from_unsupported_chain_pair_conversion() {
        let error = Error::UnsupportedChainPair {
            from_chain: "chain1".to_string(),
            to_chain: "chain2".to_string(),
        };
        let tool_error: ToolError = error.into();

        assert_eq!(
            tool_error.to_string(),
            "Permanent error: Unsupported chain pair: chain1 -> chain2 - Unsupported chain pair: chain1 -> chain2"
        );
        assert!(!tool_error.is_retriable());
    }

    #[test]
    fn test_from_insufficient_liquidity_conversion() {
        let error = Error::InsufficientLiquidity {
            amount: "500".to_string(),
        };
        let tool_error: ToolError = error.into();

        assert_eq!(
            tool_error.to_string(),
            "Operation can be retried: Insufficient liquidity for amount: 500 - Insufficient liquidity for amount: 500"
        );
        assert!(tool_error.is_retriable());
    }

    #[test]
    fn test_from_bridge_execution_error_conversion() {
        let error = Error::BridgeExecutionError("Execution failed".to_string());
        let tool_error: ToolError = error.into();

        assert_eq!(
            tool_error.to_string(),
            "Operation can be retried: Bridge operation failed: Execution failed - Bridge operation failed: Execution failed"
        );
        assert!(tool_error.is_retriable());
    }

    #[test]
    fn test_error_trait_source_method() {
        let original_tool_error = ToolError::permanent_string("Source error".to_string());
        let error = Error::ToolError(original_tool_error);

        // Test that the error implements the Error trait
        let error_trait: &dyn StdError = &error;
        assert!(error_trait.source().is_some());
    }

    #[test]
    fn test_error_trait_source_method_for_simple_variants() {
        let error = Error::LifiApiError("Simple error".to_string());

        // Test that simple variants have no source
        let error_trait: &dyn StdError = &error;
        assert!(error_trait.source().is_none());
    }

    #[test]
    fn test_debug_trait_implementation() {
        let error = Error::LifiApiError("Debug test".to_string());
        let debug_output = format!("{error:?}");

        assert!(debug_output.contains("LifiApiError"));
        assert!(debug_output.contains("Debug test"));
    }

    #[test]
    fn test_unsupported_chain_pair_with_empty_strings() {
        let error = Error::UnsupportedChainPair {
            from_chain: String::new(),
            to_chain: String::new(),
        };

        assert_eq!(error.to_string(), "Unsupported chain pair:  -> ");
    }

    #[test]
    fn test_insufficient_liquidity_with_empty_amount() {
        let error = Error::InsufficientLiquidity {
            amount: String::new(),
        };

        assert_eq!(error.to_string(), "Insufficient liquidity for amount: ");
    }

    #[test]
    fn test_all_string_variants_with_empty_strings() {
        let errors = vec![
            Error::LifiApiError(String::new()),
            Error::QuoteFetchError(String::new()),
            Error::InvalidRoute(String::new()),
            Error::BridgeExecutionError(String::new()),
        ];

        for error in errors {
            // Should not panic and should produce valid display output
            let display_output = error.to_string();
            assert!(!display_output.is_empty());
        }
    }

    #[test]
    fn test_all_string_variants_with_special_characters() {
        let special_string = "Special chars: \n\t\r\"'\\";
        let errors = vec![
            Error::LifiApiError(special_string.to_string()),
            Error::QuoteFetchError(special_string.to_string()),
            Error::InvalidRoute(special_string.to_string()),
            Error::BridgeExecutionError(special_string.to_string()),
        ];

        for error in errors {
            let display_output = error.to_string();
            assert!(display_output.contains(special_string));
        }
    }
}
