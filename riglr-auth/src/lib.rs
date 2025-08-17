//! # RIGLR Authentication
//!
//! First-class authentication and signer factory implementations for RIGLR.
//!
//! This crate provides official, maintained SignerFactory implementations for popular
//! authentication services, making it easy to build consumer-facing, multi-tenant
//! blockchain applications.
//!
//! ## Supported Providers
//!
//! - **Privy**: Embedded wallets with social login
//! - **Web3Auth**: Non-custodial key management with social login
//! - **Magic.link**: Email-based authentication with embedded wallets
//!
//! ## Usage
//!
//! ```rust,no_run
//! use riglr_auth::{AuthProvider, PrivyConfig, CompositeSignerFactoryExt};
//! use riglr_auth::config::ProviderConfig;
//! use riglr_web_adapters::factory::CompositeSignerFactory;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create authentication provider
//!     let privy_config = PrivyConfig::from_env().expect("Failed to load Privy config");
//!     let privy_provider = AuthProvider::privy(privy_config);
//!
//!     // Register with composite factory
//!     let mut factory = CompositeSignerFactory::new();
//!     factory.register_provider(privy_provider);
//!
//!     // Use in your web server
//!     // ... server setup with factory ...
//! }
//! ```

pub mod config;
pub mod error;
pub mod provider;

#[cfg(feature = "privy")]
pub mod privy;

#[cfg(feature = "web3auth")]
pub mod web3auth;

#[cfg(feature = "magic")]
pub mod magic;

// Re-export main types
pub use config::AuthConfig;
pub use error::{AuthError, AuthResult};
pub use provider::{AuthProvider, AuthProviderType};

#[cfg(feature = "privy")]
pub use privy::{PrivyConfig, PrivyProvider};

#[cfg(feature = "web3auth")]
pub use web3auth::{Web3AuthConfig, Web3AuthProvider};

#[cfg(feature = "magic")]
pub use magic::{MagicConfig, MagicProvider};

// Re-export the SignerFactory trait from web-adapters for convenience
pub use riglr_web_adapters::factory::{AuthenticationData, SignerFactory};

/// Extension trait for CompositeSignerFactory to easily register auth providers
pub trait CompositeSignerFactoryExt {
    /// Register an authentication provider with the factory
    fn register_provider(&mut self, provider: AuthProvider);
}

impl CompositeSignerFactoryExt for riglr_web_adapters::factory::CompositeSignerFactory {
    fn register_provider(&mut self, provider: AuthProvider) {
        let auth_type = provider.auth_type();
        self.register_factory(auth_type, Box::new(provider));
    }
}
