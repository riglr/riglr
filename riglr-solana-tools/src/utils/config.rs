//! Configuration utilities for Solana operations
//!
//! This module provides utilities for retrieving RPC URLs and other configuration
//! values needed for Solana operations. All configuration follows environment-first
//! patterns with sensible defaults.

use crate::error::{Result, SolanaToolError};

/// Get RPC URL from environment or use default
/// 
/// Retrieves the Solana RPC URL from the `SOLANA_RPC_URL` environment variable.
/// If not set or empty, defaults to mainnet-beta. Validates URL format to ensure
/// it's a proper HTTP/HTTPS/WebSocket URL.
/// 
/// # Returns
/// 
/// Returns the RPC URL string on success, or a `SolanaToolError::Generic` if
/// the URL format is invalid.
/// 
/// # Environment Variables
/// 
/// * `SOLANA_RPC_URL` - The RPC endpoint URL (optional)
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_solana_tools::utils::config::get_rpc_url;
/// 
/// // With environment variable set
/// std::env::set_var("SOLANA_RPC_URL", "https://api.devnet.solana.com");
/// let url = get_rpc_url()?;
/// assert_eq!(url, "https://api.devnet.solana.com");
/// 
/// // Without environment variable (uses default)
/// std::env::remove_var("SOLANA_RPC_URL");
/// let url = get_rpc_url()?;
/// assert_eq!(url, "https://api.mainnet-beta.solana.com");
/// ```
/// 
/// # Security Notes
/// 
/// - Always validates URL format to prevent injection attacks
/// - Logs only the first 50 characters of custom URLs for privacy
/// - Defaults to mainnet for production safety
pub fn get_rpc_url() -> Result<String> {
    match std::env::var("SOLANA_RPC_URL") {
        Ok(url) if !url.trim().is_empty() => {
            // Validate URL format
            if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("wss://") {
                return Err(SolanaToolError::Generic(format!(
                    "Invalid RPC URL format: {}. Must start with http://, https://, or wss://",
                    url
                )));
            }
            tracing::debug!("Found Solana RPC URL: {}", &url[..std::cmp::min(50, url.len())]);
            Ok(url)
        }
        _ => {
            // Default to mainnet-beta
            Ok("https://api.mainnet-beta.solana.com".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_rpc_url() {
        // Test with valid env var
        std::env::set_var("SOLANA_RPC_URL", "https://custom.solana.com");
        let result = get_rpc_url();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://custom.solana.com");
        
        // Test with invalid URL format
        std::env::set_var("SOLANA_RPC_URL", "invalid-url");
        let result = get_rpc_url();
        assert!(result.is_err());
        
        // Test without env var (default)
        std::env::remove_var("SOLANA_RPC_URL");
        let result = get_rpc_url();
        assert!(result.is_ok());
        assert!(result.unwrap().contains("mainnet"));
        
        // Test with empty env var
        std::env::set_var("SOLANA_RPC_URL", "");
        let result = get_rpc_url();
        assert!(result.is_ok());
        assert!(result.unwrap().contains("mainnet"));
        
        // Clean up
        std::env::remove_var("SOLANA_RPC_URL");
    }
    
    #[test]
    fn test_valid_url_formats() {
        let valid_urls = vec![
            "https://api.mainnet-beta.solana.com",
            "http://localhost:8899",
            "wss://api.mainnet-beta.solana.com",
        ];
        
        for url in valid_urls {
            std::env::set_var("SOLANA_RPC_URL", url);
            let result = get_rpc_url();
            assert!(result.is_ok(), "URL should be valid: {}", url);
            assert_eq!(result.unwrap(), url);
        }
        
        std::env::remove_var("SOLANA_RPC_URL");
    }
}