//! Chain-agnostic application context for dependency injection
//!
//! This module provides the `ApplicationContext` pattern for managing shared resources
//! and dependencies across the application without circular dependencies.
//!
//! # Architecture
//!
//! The ApplicationContext enables riglr-core to remain chain-agnostic by using
//! type erasure and dependency injection. Concrete blockchain implementations
//! are injected at runtime by the application layer.
//!
//! # Dependency Flow
//!
//! ```text
//! Application Layer (creates clients)
//!         ↓
//! ApplicationContext (stores as Arc<dyn Any>)
//!         ↓
//! Tools/Workers (retrieve by type)
//! ```

use dashmap::DashMap;
use riglr_config::Config;
use std::any::{Any, TypeId};
use std::sync::Arc;
use std::time::Duration;

use crate::util::RateLimiter;

/// Chain-agnostic context for dependency injection and resource management.
///
/// The ApplicationContext serves as a dependency injection container for
/// the riglr ecosystem, enabling tools and workers to access shared resources
/// (RPC clients, signers, database connections) without creating circular dependencies.
///
/// # Chain-Agnostic Design
///
/// The context uses type erasure (`Arc<dyn Any>`) to store blockchain clients
/// without depending on their concrete types. This allows riglr-core to remain
/// independent of blockchain SDKs while still providing access to them at runtime.
///
/// # Extension System
///
/// Resources are stored as "extensions" - type-erased objects that can be
/// retrieved by their original type. This enables clean separation between
/// riglr-core (which defines the interfaces) and chain-specific crates
/// (which provide the implementations).
///
/// # Examples
///
/// ```rust,no_run
/// use riglr_core::provider::ApplicationContext;
/// use riglr_config::Config;
/// use std::sync::Arc;
///
/// // Application layer creates context and injects dependencies
/// let config = Config::default();
/// let context = ApplicationContext::from_config(&config);
///
/// // Inject Solana RPC client (in real code, from riglr-solana-tools)
/// // let solana_client = Arc::new(solana_client::rpc_client::RpcClient::new(...));
/// // context.set_extension(solana_client.clone());
///
/// // Inject EVM provider (in real code, from riglr-evm-tools)  
/// // let evm_provider = Arc::new(alloy::providers::Provider::new(...));
/// // context.set_extension(evm_provider.clone());
///
/// // Tools retrieve clients by type
/// // let client: Option<Arc<RpcClient>> = context.get_extension();
/// ```
#[derive(Clone, Debug)]
pub struct ApplicationContext {
    /// Configuration for the application
    pub config: Config,
    /// Rate limiter for controlling request rates per client/user
    pub rate_limiter: Arc<RateLimiter>,
    /// Type-safe extensions for storing arbitrary shared resources
    extensions: Arc<DashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl ApplicationContext {
    /// Create a new ApplicationContext from configuration
    pub fn from_config(config: &Config) -> Self {
        // Initialize rate limiter with default values
        // Default: 100 requests per minute per client
        let rate_limiter = Arc::new(RateLimiter::new(100, Duration::from_secs(60)));

        Self {
            config: config.clone(),
            rate_limiter,
            extensions: Arc::new(DashMap::new()),
        }
    }

    /// Create a new ApplicationContext from environment variables
    ///
    /// **DEPRECATED**: Configuration should be loaded in the application binary using
    /// `riglr_config::Config::from_env()` and passed to `ApplicationContext::from_config()`.
    /// This ensures proper separation of concerns where `riglr-core` consumes configuration
    /// but does not load it, reinforcing the unidirectional dependency flow.
    ///
    /// # Migration Guide
    ///
    /// Instead of:
    /// ```rust,no_run
    /// use riglr_core::provider::ApplicationContext;
    /// let context = ApplicationContext::from_env();
    /// ```
    ///
    /// Use:
    /// ```rust,no_run
    /// use riglr_core::provider::ApplicationContext;
    /// use riglr_config::Config;
    ///
    /// let config = Config::from_env();
    /// let context = ApplicationContext::from_config(&config);
    /// ```
    #[deprecated(
        since = "0.3.0",
        note = "Use Config::from_env() followed by ApplicationContext::from_config() instead. This ensures proper separation of concerns."
    )]
    pub fn from_env() -> Self {
        let config = Config::from_env();
        Self::from_config(&config)
    }

    /// Add an extension to the context
    ///
    /// Extensions are stored by their type, allowing type-safe retrieval later.
    /// This is the recommended pattern for injecting RPC clients and other resources.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use riglr_core::provider::ApplicationContext;
    /// use riglr_config::Config;
    /// use std::sync::Arc;
    ///
    /// let context = ApplicationContext::from_config(&Config::default());
    ///
    /// // Add blockchain RPC clients as extensions
    /// // Example: Add Solana RPC client
    /// // let solana_client = Arc::new(solana_client::rpc_client::RpcClient::new(...));
    /// // context.set_extension(solana_client);
    ///
    /// // Example: Add EVM provider
    /// // let evm_provider = Arc::new(alloy::Provider::new(...));
    /// // context.set_extension(evm_provider);
    /// ```
    pub fn set_extension<T: Send + Sync + 'static>(&self, extension: Arc<T>) {
        self.extensions.insert(TypeId::of::<T>(), extension);
    }

    /// Get an extension by type
    ///
    /// Returns None if no extension of the given type has been set.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use riglr_core::provider::ApplicationContext;
    /// use riglr_config::Config;
    /// use std::sync::Arc;
    ///
    /// let context = ApplicationContext::from_config(&Config::default());
    /// // Add a typed extension (e.g., an RPC client)
    /// // let client = Arc::new(MyRpcClient::new(...));
    /// // context.set_extension(client.clone());
    ///
    /// // Retrieve the client later by type
    /// // let retrieved: Arc<MyRpcClient> = context.get_extension()
    ///     // .expect("RPC client not found");
    /// ```
    pub fn get_extension<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.extensions
            .get(&TypeId::of::<T>())
            .and_then(|ext| ext.clone().downcast::<T>().ok())
    }

    /// Check if an extension of the given type exists
    pub fn has_extension<T: Send + Sync + 'static>(&self) -> bool {
        self.extensions.contains_key(&TypeId::of::<T>())
    }

    /// Remove an extension by type
    ///
    /// Returns the removed extension if it existed.
    pub fn remove_extension<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.extensions
            .remove(&TypeId::of::<T>())
            .and_then(|(_, ext)| ext.downcast::<T>().ok())
    }

    /// Clear all extensions
    pub fn clear_extensions(&self) {
        self.extensions.clear();
    }

    /// Get the number of extensions
    pub fn extension_count(&self) -> usize {
        self.extensions.len()
    }
}

impl Default for ApplicationContext {
    fn default() -> Self {
        // Create with an empty/default configuration using builder
        // Users should use from_config() for production use
        let config = riglr_config::ConfigBuilder::new()
            .build()
            .expect("Default config should be valid");
        let rate_limiter = Arc::new(RateLimiter::new(100, Duration::from_secs(60)));

        Self {
            config,
            rate_limiter,
            extensions: Arc::new(DashMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestResource {
        value: String,
    }

    #[test]
    fn test_application_context_extensions() {
        let context = ApplicationContext::default();

        // Test adding and retrieving an extension
        let resource = Arc::new(TestResource {
            value: "test".to_string(),
        });
        context.set_extension(resource.clone());

        let retrieved: Arc<TestResource> = context.get_extension().expect("Resource not found");
        assert_eq!(retrieved.value, "test");
    }

    #[test]
    fn test_application_context_multiple_extensions() {
        let context = ApplicationContext::default();

        // Add multiple different types
        let resource1 = Arc::new(TestResource {
            value: "test1".to_string(),
        });
        let resource2 = Arc::new(42u32);

        context.set_extension(resource1.clone());
        context.set_extension(resource2.clone());

        // Retrieve both
        let retrieved1: Arc<TestResource> = context.get_extension().expect("Resource not found");
        let retrieved2: Arc<u32> = context.get_extension().expect("u32 not found");

        assert_eq!(retrieved1.value, "test1");
        assert_eq!(*retrieved2, 42);
    }

    #[test]
    fn test_application_context_has_extension() {
        let context = ApplicationContext::default();

        assert!(!context.has_extension::<TestResource>());

        let resource = Arc::new(TestResource {
            value: "test".to_string(),
        });
        context.set_extension(resource);

        assert!(context.has_extension::<TestResource>());
    }

    #[test]
    fn test_application_context_remove_extension() {
        let context = ApplicationContext::default();

        let resource = Arc::new(TestResource {
            value: "test".to_string(),
        });
        context.set_extension(resource);

        assert!(context.has_extension::<TestResource>());

        let removed: Option<Arc<TestResource>> = context.remove_extension();
        assert!(removed.is_some());
        assert!(!context.has_extension::<TestResource>());
    }
}
