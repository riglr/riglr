//! Privy authentication provider implementation
//!
//! Provides embedded wallets with social login capabilities through Privy.

mod config;
mod provider;
mod signer;
mod types;

pub use config::PrivyConfig;
pub use provider::PrivyProvider;
pub use types::{PrivyUserData, PrivyWallet};

// Re-export for convenience
pub use provider::create_privy_provider;