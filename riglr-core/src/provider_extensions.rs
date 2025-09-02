//! Extension traits for ApplicationContext to provide ergonomic access to chain-specific clients
//!
//! These traits should be implemented in the respective chain-specific crates
//! (riglr-solana-tools, riglr-evm-tools, etc.) to provide type-safe, convenient
//! access methods for their clients.

use crate::ToolError;
use std::sync::Arc;

/// Example trait that should be implemented in riglr-solana-tools
///
/// # Example Implementation (in riglr-solana-tools/src/lib.rs):
/// ```rust
/// use riglr_core::provider::ApplicationContext;
/// use riglr_core::provider_extensions::SolanaAppContextProvider;
/// use riglr_core::ToolError;
/// use solana_client::rpc_client::RpcClient as SolanaRpcClient;
/// use std::sync::Arc;
///
/// impl SolanaAppContextProvider for ApplicationContext {
///     fn solana_client(&self) -> Result<Arc<SolanaRpcClient>, ToolError> {
///         self.get_extension::<Arc<SolanaRpcClient>>()
///             .ok_or_else(|| ToolError::permanent_string(
///                 "Solana RPC client not configured in ApplicationContext"
///             ))
///     }
/// }
/// ```
pub trait SolanaAppContextProvider {
    /// Get the Solana RPC client from the context
    fn solana_client(&self) -> Result<Arc<dyn std::any::Any>, ToolError>;
}

/// Example trait that should be implemented in riglr-evm-tools
///
/// # Example Implementation (in riglr-evm-tools/src/lib.rs):
/// ```rust
/// use riglr_core::provider::ApplicationContext;
/// use riglr_core::provider_extensions::EvmAppContextProvider;
/// use riglr_core::ToolError;
/// use ethers::providers::Provider;
/// use std::sync::Arc;
///
/// impl EvmAppContextProvider for ApplicationContext {
///     fn evm_client(&self) -> Result<Arc<dyn Provider>, ToolError> {
///         self.get_extension::<Arc<dyn Provider>>()
///             .ok_or_else(|| ToolError::permanent_string(
///                 "EVM provider not configured in ApplicationContext"
///             ))
///     }
/// }
/// ```
pub trait EvmAppContextProvider {
    /// Get the EVM provider from the context
    fn evm_client(&self) -> Result<Arc<dyn std::any::Any>, ToolError>;
}

/// Generic extension trait for custom providers
///
/// This can be used for any custom client type that needs to be accessed
/// from the ApplicationContext.
///
/// # Example:
/// ```rust
/// use riglr_core::provider::ApplicationContext;
/// use riglr_core::provider_extensions::AppContextExtension;
/// use riglr_core::ToolError;
/// use std::sync::Arc;
///
/// struct MyCustomClient;
///
/// impl AppContextExtension<MyCustomClient> for ApplicationContext {
///     fn get_client(&self) -> Result<Arc<MyCustomClient>, ToolError> {
///         self.get_extension::<Arc<MyCustomClient>>()
///             .ok_or_else(|| ToolError::permanent_string(
///                 "MyCustomClient not configured in ApplicationContext"
///             ))
///     }
/// }
/// ```
pub trait AppContextExtension<T> {
    /// Get a client of type T from the context
    fn get_client(&self) -> Result<Arc<T>, ToolError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::ApplicationContext;
    
    // Mock client for testing
    struct MockClient;
    
    // Implementation of the extension trait for testing
    impl AppContextExtension<MockClient> for ApplicationContext {
        fn get_client(&self) -> Result<Arc<MockClient>, ToolError> {
            self.get_extension::<MockClient>()
                .ok_or_else(|| ToolError::permanent_string(
                    "MockClient not configured in ApplicationContext"
                ))
        }
    }
    
    #[test]
    fn test_extension_trait_pattern() {
        // This test demonstrates the pattern but won't run without actual implementation
        // It serves as documentation for how to use the extension traits
        
        // Example of how it would be used:
        // let context = ApplicationContext::new(...);
        // context.add_extension(Arc::new(MockClient));
        // let client = context.get_client::<MockClient>()?;
    }
}