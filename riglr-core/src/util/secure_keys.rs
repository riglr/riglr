//! Secure key loading utilities for riglr-core
//!
//! This module provides utilities for loading private keys from files
//! instead of environment variables, following security best practices.
//!
//! ## ⚠️ IMPORTANT PRODUCTION SECURITY NOTICE ⚠️
//!
//! **The file-based key loading provided by this module is intended for DEVELOPMENT ONLY.**
//! In production environments, you MUST use more secure key management solutions.
//!
//! ### Recommended Production Key Management Solutions:
//!
//! #### 1. Operating System Keychains
//! Use OS-native secure storage via the [`keyring`](https://crates.io/crates/keyring) crate:
//!
//! ```toml
//! [dependencies]
//! keyring = "2.0"
//! ```
//!
//! ```rust,no_run
//! use keyring::Entry;
//!
//! // Store key (one-time setup)
//! let entry = Entry::new("riglr", "solana_private_key")?;
//! entry.set_password("your_base58_private_key_here")?;
//!
//! // Retrieve key in your application
//! let private_key = entry.get_password()?;
//! ```
//!
//! #### 2. Hardware Security Modules (HSMs)
//! - **AWS CloudHSM**: Enterprise-grade hardware security
//! - **YubiHSM**: Compact, affordable HSM for smaller deployments
//! - **Ledger/Trezor**: For development and testing with hardware wallets
//!
//! #### 3. Cloud-Based Secret Managers
//! - **AWS Secrets Manager**: Fully managed secrets with automatic rotation
//! - **Google Cloud Secret Manager**: Secure, convenient secret storage
//! - **Azure Key Vault**: Enterprise identity and access management
//! - **HashiCorp Vault**: Open-source secret management with audit trails
//!
//! Example with AWS Secrets Manager:
//! ```rust,no_run
//! use aws_sdk_secretsmanager::Client;
//!
//! let config = aws_config::load_from_env().await;
//! let client = Client::new(&config);
//! let response = client
//!     .get_secret_value()
//!     .secret_id("riglr/solana/private-key")
//!     .send()
//!     .await?;
//! let private_key = response.secret_string().unwrap();
//! ```
//!
//! ### Security Best Practices:
//! - **Principle of Least Privilege**: Only allow necessary access to keys
//! - **Audit Logging**: Log all key access for security monitoring  
//! - **Key Rotation**: Regularly rotate private keys and update storage
//! - **Multi-Factor Authentication**: Require MFA for key access
//! - **Encryption at Rest**: Ensure keys are encrypted when stored
//! - **Network Security**: Use TLS for all key transmission
//! - **Environment Isolation**: Separate development and production key storage
//!
//! ### File-Based Storage Risks:
//! - Keys stored in plaintext on disk
//! - Vulnerable to file system access
//! - No audit trail of key usage
//! - Risk of accidental commits to version control
//! - No automatic key rotation capabilities
//!
//! **Use the file-based utilities in this module only for:**
//! - Local development and testing
//! - Proof-of-concept applications
//! - Educational purposes
//!
//! **Never use file-based storage for:**
//! - Production applications
//! - Applications handling real funds
//! - Multi-user or hosted environments

use crate::ToolError;
use std::fs;
use std::path::{Path, PathBuf};

/// Test environment variable name for private key
#[cfg(test)]
const TEST_PRIVATE_KEY_ENV: &str = "TEST_PRIVATE_KEY";

/// Load a private key from a file with appropriate security checks
///
/// # Security Notes
/// - The key file should have restricted permissions (e.g., 0600 on Unix)
/// - Never commit key files to version control
/// - Consider using system keyrings or HSMs in production
///
/// # Example
/// ```no_run
/// use riglr_core::util::load_private_key_from_file;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Load from default location
/// let key = load_private_key_from_file("~/.riglr/keys/solana.key")?;
///
/// // Load from custom location
/// let key = load_private_key_from_file("/secure/keys/my-key.pem")?;
/// # Ok(())
/// # }
/// ```
pub fn load_private_key_from_file<P: AsRef<Path>>(path: P) -> Result<String, ToolError> {
    let path = path.as_ref();

    // Expand home directory if path starts with ~
    let expanded_path = expand_home_dir(path)?;

    // Check if file exists
    if !expanded_path.exists() {
        return Err(ToolError::permanent_string(format!(
            "Key file not found: {}",
            expanded_path.display()
        )));
    }

    // Check file permissions on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&expanded_path).map_err(|e| {
            ToolError::permanent_string(format!("Failed to read file metadata: {}", e))
        })?;

        let permissions = metadata.permissions();
        let mode = permissions.mode();

        // Check if file is readable only by owner (0600 or 0400)
        if mode & 0o077 != 0 {
            tracing::warn!(
                "Key file {} has insecure permissions: {:o}. Consider using chmod 600",
                expanded_path.display(),
                mode & 0o777
            );
        }
    }

    // Read the key file
    let key_content = fs::read_to_string(&expanded_path)
        .map_err(|e| ToolError::permanent_string(format!("Failed to read key file: {}", e)))?;

    // Trim whitespace
    Ok(key_content.trim().to_string())
}

/// Load a private key with fallback to environment variable
///
/// This function first attempts to load from a file, then falls back to
/// an environment variable if the file doesn't exist.
///
/// # Example
/// ```no_run
/// use riglr_core::util::load_private_key_with_fallback;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Try file first, then env var
/// let key = load_private_key_with_fallback(
///     "~/.riglr/keys/solana.key",
///     "SOLANA_PRIVATE_KEY"
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn load_private_key_with_fallback<P: AsRef<Path>>(
    file_path: P,
    env_var: &str,
) -> Result<String, ToolError> {
    // Try loading from file first
    match load_private_key_from_file(&file_path) {
        Ok(key) => {
            tracing::debug!("Loaded key from file: {}", file_path.as_ref().display());
            Ok(key)
        }
        Err(_) => {
            // Fall back to environment variable
            tracing::debug!(
                "Key file not found, trying environment variable: {}",
                env_var
            );
            std::env::var(env_var).map_err(|_| {
                ToolError::permanent_string(format!(
                    "Private key not found in file {} or environment variable {}",
                    file_path.as_ref().display(),
                    env_var
                ))
            })
        }
    }
}

/// Environment variable for Windows APPDATA directory
const APPDATA_ENV: &str = "APPDATA";

/// Get the default key directory for riglr
///
/// Returns ~/.riglr/keys on Unix-like systems
/// Returns %APPDATA%\riglr\keys on Windows
pub fn get_default_key_directory() -> PathBuf {
    let base_dir = if cfg!(target_os = "windows") {
        std::env::var(APPDATA_ENV).map_or_else(
            |_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("AppData")
                    .join("Roaming")
            },
            PathBuf::from,
        )
    } else {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
    };

    base_dir.join(".riglr").join("keys")
}

/// Expand ~ to home directory in a path
fn expand_home_dir(path: &Path) -> Result<PathBuf, ToolError> {
    if let Some(path_str) = path.to_str() {
        if path_str.starts_with("~/") || path_str == "~" {
            if let Some(home) = dirs::home_dir() {
                let relative = path_str.strip_prefix("~/").unwrap_or("");
                return Ok(home.join(relative));
            } else {
                return Err(ToolError::permanent_string(
                    "Unable to determine home directory".to_string(),
                ));
            }
        }
    }
    Ok(path.to_path_buf())
}

/// Create the default key directory with appropriate permissions
pub fn ensure_key_directory() -> Result<PathBuf, ToolError> {
    let key_dir = get_default_key_directory();

    if !key_dir.exists() {
        fs::create_dir_all(&key_dir).map_err(|e| {
            ToolError::permanent_string(format!("Failed to create key directory: {}", e))
        })?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o700);
            fs::set_permissions(&key_dir, permissions).map_err(|e| {
                ToolError::permanent_string(format!("Failed to set directory permissions: {}", e))
            })?;
        }
    }

    Ok(key_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_private_key_from_file() {
        // Create a temporary file with a test key
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test-private-key-content").unwrap();

        // Load the key
        let key = load_private_key_from_file(temp_file.path()).unwrap();
        assert_eq!(key, "test-private-key-content");
    }

    #[test]
    fn test_load_private_key_with_fallback() {
        // Test fallback to environment variable
        std::env::set_var(TEST_PRIVATE_KEY_ENV, "env-key-content");

        let key = load_private_key_with_fallback("/nonexistent/path/to/key", TEST_PRIVATE_KEY_ENV)
            .unwrap();

        assert_eq!(key, "env-key-content");

        std::env::remove_var(TEST_PRIVATE_KEY_ENV);
    }

    #[test]
    fn test_get_default_key_directory() {
        let key_dir = get_default_key_directory();
        assert!(key_dir.ends_with(".riglr/keys") || key_dir.ends_with("riglr\\keys"));
    }

    #[test]
    fn test_expand_home_dir() {
        let expanded = expand_home_dir(Path::new("~/test")).unwrap();
        assert!(!expanded.to_str().unwrap().starts_with("~"));

        let not_expanded = expand_home_dir(Path::new("/absolute/path")).unwrap();
        assert_eq!(not_expanded, Path::new("/absolute/path"));
    }
}
