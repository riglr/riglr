use riglr_core::error::ToolError;
use thiserror::Error;

/// Errors that can occur during cross-chain operations
#[derive(Error, Debug)]
pub enum CrossChainError {
    /// Core tool error
    #[error("Core tool error: {0}")]
    ToolError(#[from] ToolError),

    /// Li.fi API error
    #[error("Li.fi API error: {0}")]
    LifiApiError(String),

    /// Quote fetch failed
    #[error("Quote fetch failed: {0}")]
    QuoteFetchError(String),

    /// Invalid route configuration
    #[error("Invalid route configuration: {0}")]
    InvalidRoute(String),

    /// Bridge operation failed
    #[error("Bridge operation failed: {0}")]
    BridgeExecutionError(String),

    /// Unsupported chain pair
    #[error("Unsupported chain pair: {from_chain} -> {to_chain}")]
    UnsupportedChainPair {
        /// Source chain identifier
        from_chain: String,
        /// Destination chain identifier
        to_chain: String,
    },

    /// Insufficient liquidity for amount
    #[error("Insufficient liquidity for amount: {amount}")]
    InsufficientLiquidity {
        /// Amount that was requested but unavailable
        amount: String,
    },
}

impl From<CrossChainError> for ToolError {
    fn from(err: CrossChainError) -> Self {
        match err {
            CrossChainError::ToolError(tool_err) => tool_err,
            CrossChainError::LifiApiError(_) => ToolError::retriable_string(err.to_string()),
            CrossChainError::QuoteFetchError(_) => ToolError::retriable_string(err.to_string()),
            CrossChainError::InvalidRoute(_) => ToolError::permanent_string(err.to_string()),
            CrossChainError::UnsupportedChainPair { .. } => {
                ToolError::permanent_string(err.to_string())
            }
            CrossChainError::InsufficientLiquidity { .. } => {
                ToolError::retriable_string(err.to_string())
            }
            CrossChainError::BridgeExecutionError(_) => {
                ToolError::retriable_string(err.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_core::error::ToolError;
    use std::error::Error;

    #[test]
    fn test_tool_error_variant_creation_and_display() {
        let tool_error = ToolError::permanent_string("Original tool error".to_string());
        let error = CrossChainError::ToolError(tool_error);

        assert_eq!(error.to_string(), "Core tool error: Original tool error");
    }

    #[test]
    fn test_lifi_api_error_variant_creation_and_display() {
        let error = CrossChainError::LifiApiError("API timeout".to_string());

        assert_eq!(error.to_string(), "Li.fi API error: API timeout");
    }

    #[test]
    fn test_quote_fetch_error_variant_creation_and_display() {
        let error = CrossChainError::QuoteFetchError("Network unreachable".to_string());

        assert_eq!(error.to_string(), "Quote fetch failed: Network unreachable");
    }

    #[test]
    fn test_invalid_route_variant_creation_and_display() {
        let error = CrossChainError::InvalidRoute("Invalid token address".to_string());

        assert_eq!(
            error.to_string(),
            "Invalid route configuration: Invalid token address"
        );
    }

    #[test]
    fn test_bridge_execution_error_variant_creation_and_display() {
        let error = CrossChainError::BridgeExecutionError("Transaction failed".to_string());

        assert_eq!(
            error.to_string(),
            "Bridge operation failed: Transaction failed"
        );
    }

    #[test]
    fn test_unsupported_chain_pair_variant_creation_and_display() {
        let error = CrossChainError::UnsupportedChainPair {
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
        let error = CrossChainError::InsufficientLiquidity {
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
        let cross_chain_error = CrossChainError::ToolError(original_tool_error);
        let converted_tool_error: ToolError = cross_chain_error.into();

        assert_eq!(converted_tool_error.to_string(), error_message);
    }

    #[test]
    fn test_from_lifi_api_error_conversion() {
        let error = CrossChainError::LifiApiError("API error".to_string());
        let tool_error: ToolError = error.into();

        assert_eq!(tool_error.to_string(), "Li.fi API error: API error");
        assert!(tool_error.is_retriable());
    }

    #[test]
    fn test_from_quote_fetch_error_conversion() {
        let error = CrossChainError::QuoteFetchError("Fetch failed".to_string());
        let tool_error: ToolError = error.into();

        assert_eq!(tool_error.to_string(), "Quote fetch failed: Fetch failed");
        assert!(tool_error.is_retriable());
    }

    #[test]
    fn test_from_invalid_route_conversion() {
        let error = CrossChainError::InvalidRoute("Bad route".to_string());
        let tool_error: ToolError = error.into();

        assert_eq!(
            tool_error.to_string(),
            "Invalid route configuration: Bad route"
        );
        assert!(!tool_error.is_retriable());
    }

    #[test]
    fn test_from_unsupported_chain_pair_conversion() {
        let error = CrossChainError::UnsupportedChainPair {
            from_chain: "chain1".to_string(),
            to_chain: "chain2".to_string(),
        };
        let tool_error: ToolError = error.into();

        assert_eq!(
            tool_error.to_string(),
            "Unsupported chain pair: chain1 -> chain2"
        );
        assert!(!tool_error.is_retriable());
    }

    #[test]
    fn test_from_insufficient_liquidity_conversion() {
        let error = CrossChainError::InsufficientLiquidity {
            amount: "500".to_string(),
        };
        let tool_error: ToolError = error.into();

        assert_eq!(
            tool_error.to_string(),
            "Insufficient liquidity for amount: 500"
        );
        assert!(tool_error.is_retriable());
    }

    #[test]
    fn test_from_bridge_execution_error_conversion() {
        let error = CrossChainError::BridgeExecutionError("Execution failed".to_string());
        let tool_error: ToolError = error.into();

        assert_eq!(
            tool_error.to_string(),
            "Bridge operation failed: Execution failed"
        );
        assert!(tool_error.is_retriable());
    }

    #[test]
    fn test_error_trait_source_method() {
        let original_tool_error = ToolError::permanent_string("Source error".to_string());
        let error = CrossChainError::ToolError(original_tool_error);

        // Test that the error implements the Error trait
        let error_trait: &dyn Error = &error;
        assert!(error_trait.source().is_some());
    }

    #[test]
    fn test_error_trait_source_method_for_simple_variants() {
        let error = CrossChainError::LifiApiError("Simple error".to_string());

        // Test that simple variants have no source
        let error_trait: &dyn Error = &error;
        assert!(error_trait.source().is_none());
    }

    #[test]
    fn test_debug_trait_implementation() {
        let error = CrossChainError::LifiApiError("Debug test".to_string());
        let debug_output = format!("{:?}", error);

        assert!(debug_output.contains("LifiApiError"));
        assert!(debug_output.contains("Debug test"));
    }

    #[test]
    fn test_unsupported_chain_pair_with_empty_strings() {
        let error = CrossChainError::UnsupportedChainPair {
            from_chain: "".to_string(),
            to_chain: "".to_string(),
        };

        assert_eq!(error.to_string(), "Unsupported chain pair:  -> ");
    }

    #[test]
    fn test_insufficient_liquidity_with_empty_amount() {
        let error = CrossChainError::InsufficientLiquidity {
            amount: "".to_string(),
        };

        assert_eq!(error.to_string(), "Insufficient liquidity for amount: ");
    }

    #[test]
    fn test_all_string_variants_with_empty_strings() {
        let errors = vec![
            CrossChainError::LifiApiError("".to_string()),
            CrossChainError::QuoteFetchError("".to_string()),
            CrossChainError::InvalidRoute("".to_string()),
            CrossChainError::BridgeExecutionError("".to_string()),
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
            CrossChainError::LifiApiError(special_string.to_string()),
            CrossChainError::QuoteFetchError(special_string.to_string()),
            CrossChainError::InvalidRoute(special_string.to_string()),
            CrossChainError::BridgeExecutionError(special_string.to_string()),
        ];

        for error in errors {
            let display_output = error.to_string();
            assert!(display_output.contains(special_string));
        }
    }
}
