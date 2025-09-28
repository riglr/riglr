//! Utility modules and functions for riglr-core
//!
//! This module provides various utility functions and types for riglr-core operations.
//! Contains rate limiting, secure key management, and token bucket implementations.

/// Rate limiting strategy types and traits
///
/// This module defines the core interfaces for rate limiting strategies.
pub mod rate_limit_strategy {
    use crate::ToolError;
    use core::time::Duration;
    use dashmap::DashMap;
    use std::time::Instant;

    /// Trait defining the interface for rate limiting strategies
    ///
    /// Different strategies can implement this trait to provide various
    /// rate limiting algorithms such as token bucket, fixed window, sliding window, etc.
    pub trait RateLimitStrategy: Send + Sync {
        /// Check if a request should be allowed for the given client
        ///
        /// # Arguments
        /// * `client_id` - Unique identifier for the client
        ///
        /// # Returns
        /// * `Ok(())` if the request is allowed
        /// * `Err(ToolError::RateLimited)` if the rate limit is exceeded
        ///
        /// # Errors
        /// Returns `ToolError::RateLimited` when the rate limit is exceeded for the given client.
        fn check_rate_limit(&self, client_id: &str) -> Result<(), ToolError>;

        /// Clear all rate limit data
        fn clear_all(&self);

        /// Get current request count for a client
        fn get_request_count(&self, client_id: &str) -> usize;

        /// Reset rate limit state for a specific client
        fn reset_client(&self, client_id: &str);

        /// Get strategy name for debugging/logging
        fn strategy_name(&self) -> &str;
    }

    /// Information about a client's rate limit status
    #[derive(Debug, Clone)]
    pub struct ClientRateInfo {
        /// Number of tokens available for burst (stored as f64 for fractional replenishment)
        pub burst_tokens: f64,
        /// Last time tokens were refilled
        pub last_refill: Instant,
        /// Timestamps of recent requests
        pub request_times: Vec<Instant>,
    }

    impl ClientRateInfo {
        /// Create new client rate info with initial values
        #[must_use]
        pub fn new(initial_tokens: f64) -> Self {
            Self {
                burst_tokens: initial_tokens,
                last_refill: Instant::now(),
                request_times: Vec::new(),
            }
        }
    }

    /// Fixed window rate limiting strategy
    ///
    /// This strategy divides time into fixed windows and allows a fixed number
    /// of requests per window. When a window expires, the count resets.
    #[derive(Debug)]
    pub struct FixedWindowStrategy {
        /// Client tracking
        pub clients: DashMap<String, FixedWindowClientInfo>,
        /// Maximum requests per window
        pub max_requests: usize,
        /// Duration of each window
        pub window_duration: Duration,
    }

    /// Information about a client's fixed window rate limit state
    #[derive(Debug, Clone)]
    pub struct FixedWindowClientInfo {
        /// Number of requests in current window
        pub request_count: usize,
        /// Start of current window
        pub window_start: Instant,
    }

    impl RateLimitStrategy for FixedWindowStrategy {
        fn check_rate_limit(&self, client_id: &str) -> Result<(), ToolError> {
            let now = Instant::now();
            let mut entry = self
                .clients
                .entry(client_id.to_string())
                .or_insert_with(|| FixedWindowClientInfo {
                    request_count: 0,
                    window_start: now,
                });

            // Check if we're in a new window
            if now.duration_since(entry.window_start) >= self.window_duration {
                // Reset for new window
                entry.window_start = now;
                entry.request_count = 0;
            }

            // Check if limit exceeded
            if entry.request_count >= self.max_requests {
                let time_until_reset = self
                    .window_duration
                    .saturating_sub(now.duration_since(entry.window_start));

                return Err(ToolError::RateLimited {
                    source: None,
                    source_message: format!(
                        "Fixed window rate limit: {} requests per {:?}",
                        self.max_requests, self.window_duration
                    ),
                    context: format!("Exceeded {} requests in current window", self.max_requests),
                    retry_after: Some(time_until_reset),
                });
            }

            entry.request_count = entry.request_count.saturating_add(1);
            drop(entry);
            Ok(())
        }

        fn clear_all(&self) {
            self.clients.clear();
        }

        fn reset_client(&self, client_id: &str) {
            self.clients.remove(client_id);
        }

        fn get_request_count(&self, client_id: &str) -> usize {
            self.clients
                .get(client_id)
                .map_or(0, |entry| entry.request_count)
        }

        fn strategy_name(&self) -> &'static str {
            "FixedWindow"
        }
    }

    impl FixedWindowStrategy {
        /// Create a new fixed window rate limiter
        #[must_use]
        pub fn new(max_requests: usize, window_duration: Duration) -> Self {
            Self {
                max_requests,
                window_duration,
                clients: DashMap::new(),
            }
        }
    }
}

/// Token bucket rate limiting strategy
///
/// This module implements a time-based token bucket algorithm where tokens
/// are replenished continuously based on elapsed time.
pub mod token_bucket {
    use super::rate_limit_strategy::{ClientRateInfo, RateLimitStrategy};
    use crate::ToolError;
    use alloc::sync::Arc;
    use core::time::Duration;
    use dashmap::DashMap;
    use std::time::Instant;

    /// Token bucket rate limiting strategy
    ///
    /// This strategy implements a time-based token bucket algorithm where:
    /// - Tokens are replenished continuously based on elapsed time
    /// - The replenishment rate is `max_requests` per `time_window`
    /// - Burst capacity allows temporary spikes up to `burst_size` tokens
    #[derive(Debug, Clone)]
    pub struct BucketStrategy {
        /// Optional burst size for allowing temporary spikes
        burst_size: Option<usize>,
        /// Map of client ID to their request history
        clients: Arc<DashMap<String, ClientRateInfo>>,
        /// Maximum number of requests allowed in the time window
        max_requests: usize,
        /// Time window for rate limiting
        time_window: Duration,
    }

    impl BucketStrategy {
        /// Create a new token bucket strategy
        #[must_use]
        pub fn new(max_requests: usize, time_window: Duration) -> Self {
            Self {
                burst_size: None,
                clients: Arc::new(DashMap::new()),
                max_requests,
                time_window,
            }
        }

        /// Create with burst capacity
        #[must_use]
        pub fn with_burst(max_requests: usize, time_window: Duration, burst_size: usize) -> Self {
            Self {
                burst_size: Some(burst_size),
                clients: Arc::new(DashMap::new()),
                max_requests,
                time_window,
            }
        }
    }

    impl RateLimitStrategy for BucketStrategy {
        fn check_rate_limit(&self, client_id: &str) -> Result<(), ToolError> {
            let now = Instant::now();
            let mut entry = self
                .clients
                .entry(client_id.to_string())
                .or_insert_with(|| {
                    #[expect(clippy::cast_precision_loss)]
                    let initial_tokens = self.burst_size.unwrap_or(self.max_requests) as f64;
                    ClientRateInfo::new(initial_tokens)
                });

            // Time-based token replenishment
            let elapsed = now.duration_since(entry.last_refill);
            let elapsed_seconds = elapsed.as_secs_f64();

            // Calculate the refill rate: tokens per second
            #[expect(clippy::cast_precision_loss)]
            let refill_rate = self.max_requests as f64 / self.time_window.as_secs_f64();

            // Calculate tokens to add based on elapsed time
            let tokens_to_add = elapsed_seconds * refill_rate;

            // Add tokens up to the burst size limit (or max_requests if no burst size)
            #[expect(clippy::cast_precision_loss)]
            let max_tokens = self.burst_size.unwrap_or(self.max_requests) as f64;
            entry.burst_tokens = (entry.burst_tokens + tokens_to_add).min(max_tokens);
            entry.last_refill = now;

            // Remove old requests outside the time window (for sustained rate tracking)
            entry
                .request_times
                .retain(|&time| now.duration_since(time) < self.time_window);

            // Check if we have tokens available
            let has_burst_token = entry.burst_tokens >= 1.0f64;

            if !has_burst_token {
                // No tokens available, calculate retry time
                let tokens_needed = 1.0f64 - entry.burst_tokens;
                let seconds_until_token = tokens_needed / refill_rate;
                let retry_after = Duration::from_secs_f64(seconds_until_token);

                return Err(ToolError::RateLimited {
                    source: None,
                    source_message: format!(
                        "Token bucket rate limit: {} requests per {:?}",
                        self.max_requests, self.time_window
                    ),
                    context: format!("User exceeded rate limit of {} requests", self.max_requests),
                    retry_after: Some(retry_after),
                });
            }

            // We have a token, consume it and allow the request
            entry.burst_tokens -= 1.0f64;
            entry.request_times.push(now);
            drop(entry);
            Ok(())
        }

        fn clear_all(&self) {
            self.clients.clear();
        }

        fn get_request_count(&self, client_id: &str) -> usize {
            let now = Instant::now();
            self.clients.get(client_id).map_or(0, |entry| {
                entry
                    .request_times
                    .iter()
                    .filter(|&&time| now.duration_since(time) < self.time_window)
                    .count()
            })
        }

        fn reset_client(&self, client_id: &str) {
            self.clients.remove(client_id);
        }

        fn strategy_name(&self) -> &'static str {
            "TokenBucket"
        }
    }
}

/// Rate limiting utilities for riglr-core
///
/// This module provides flexible, strategy-based rate limiting that supports
/// multiple algorithms including token bucket, fixed window, and custom strategies.
pub mod rate_limiter {
    use core::fmt::{Debug, Formatter, Result as FmtResult};
    use core::time::Duration;
    use std::sync::Arc;

    use super::rate_limit_strategy::{FixedWindowStrategy, RateLimitStrategy};
    use super::token_bucket::BucketStrategy;
    use crate::ToolError;

    /// Rate limiting strategy type
    #[derive(Debug, Clone, Copy)]
    pub enum RateLimitStrategyType {
        /// Fixed window algorithm
        FixedWindow,
        /// Token bucket algorithm (default)
        TokenBucket,
    }

    /// A configurable rate limiter for controlling request rates.
    ///
    /// This rate limiter supports multiple strategies:
    /// - Token bucket: Continuous token replenishment with burst capacity
    /// - Fixed window: Fixed request count per time window
    /// - Custom strategies via the `RateLimitStrategy` trait
    ///
    /// # Example
    ///
    /// ```rust
    /// use riglr_core::util::{RateLimiter, RateLimitStrategyType};
    /// use std::time::Duration;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Default token bucket strategy
    /// let rate_limiter = RateLimiter::new(10, Duration::from_secs(60));
    ///
    /// // Check rate limit for a user
    /// rate_limiter.check_rate_limit("user123")?;
    ///
    /// // Use fixed window strategy
    /// let fixed_limiter = RateLimiter::builder()
    ///     .strategy(RateLimitStrategyType::FixedWindow)
    ///     .max_requests(100)
    ///     .time_window(Duration::from_secs(60))
    ///     .build();
    /// # Ok(())
    /// # }
    /// ```
    #[derive(Clone)]
    pub struct RateLimiter {
        /// The underlying rate limiting strategy
        strategy: Arc<dyn RateLimitStrategy>,
    }

    impl Debug for RateLimiter {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            f.debug_struct("RateLimiter")
                .field("strategy", &self.strategy.strategy_name())
                .finish()
        }
    }

    impl RateLimiter {
        /// Create a new rate limiter with the default token bucket strategy
        #[must_use]
        pub fn new(max_requests: usize, time_window: Duration) -> Self {
            Self {
                strategy: Arc::new(BucketStrategy::new(max_requests, time_window)),
            }
        }

        /// Create a new rate limiter builder for advanced configuration
        #[must_use]
        pub fn builder() -> Builder {
            Builder::default()
        }

        /// Create a rate limiter with a custom strategy
        pub fn with_strategy<S: RateLimitStrategy + 'static>(strategy: S) -> Self {
            Self {
                strategy: Arc::new(strategy),
            }
        }

        /// Check if a client has exceeded the rate limit
        ///
        /// # Arguments
        /// * `client_id` - Unique identifier for the client (e.g., IP address, user ID)
        ///
        /// # Returns
        /// * `Ok(())` if the request is allowed
        /// * `Err(ToolError::RateLimited)` if the rate limit is exceeded
        ///
        /// # Errors
        /// Returns `ToolError::RateLimited` when the rate limit is exceeded for the given client.
        pub fn check_rate_limit(&self, client_id: &str) -> Result<(), ToolError> {
            self.strategy.check_rate_limit(client_id)
        }

        /// Clear all rate limit data
        pub fn clear_all(&self) {
            self.strategy.clear_all();
        }

        /// Reset rate limit for a specific client
        pub fn reset_client(&self, client_id: &str) {
            self.strategy.reset_client(client_id);
        }

        /// Get current request count for a client
        #[must_use]
        pub fn get_request_count(&self, client_id: &str) -> usize {
            self.strategy.get_request_count(client_id)
        }

        /// Get the name of the current strategy
        #[must_use]
        pub fn strategy_name(&self) -> &str {
            self.strategy.strategy_name()
        }
    }

    /// Builder for creating customized `RateLimiter` instances
    #[derive(Debug, Default)]
    pub struct Builder {
        burst_size: Option<usize>,
        max_requests: Option<usize>,
        strategy_type: Option<RateLimitStrategyType>,
        time_window: Option<Duration>,
    }

    impl Builder {
        /// Build the `RateLimiter`
        #[must_use]
        pub fn build(self) -> RateLimiter {
            let max_requests = self.max_requests.unwrap_or(10);
            let time_window = self.time_window.unwrap_or_else(|| Duration::from_secs(60));
            let strategy_type = self
                .strategy_type
                .unwrap_or(RateLimitStrategyType::TokenBucket);

            let strategy: Arc<dyn RateLimitStrategy> = match strategy_type {
                RateLimitStrategyType::TokenBucket => self.burst_size.map_or_else(
                    || Arc::new(BucketStrategy::new(max_requests, time_window)),
                    |burst_size| {
                        Arc::new(BucketStrategy::with_burst(
                            max_requests,
                            time_window,
                            burst_size,
                        ))
                    },
                ),
                RateLimitStrategyType::FixedWindow => {
                    Arc::new(FixedWindowStrategy::new(max_requests, time_window))
                }
            };

            RateLimiter { strategy }
        }

        /// Set the burst size for temporary spikes
        #[must_use]
        pub const fn burst_size(mut self, size: usize) -> Self {
            self.burst_size = Some(size);
            self
        }

        /// Set the maximum number of requests allowed in the time window
        #[must_use]
        pub const fn max_requests(mut self, max: usize) -> Self {
            self.max_requests = Some(max);
            self
        }

        /// Set the rate limiting strategy type
        #[must_use]
        pub const fn strategy(mut self, strategy: RateLimitStrategyType) -> Self {
            self.strategy_type = Some(strategy);
            self
        }

        /// Set the time window for rate limiting
        #[must_use]
        pub const fn time_window(mut self, window: Duration) -> Self {
            self.time_window = Some(window);
            self
        }
    }
}

/// Secure key loading utilities for riglr-core
///
/// This module provides utilities for loading private keys from files
/// instead of environment variables, following security best practices.
///
/// ## ⚠️ IMPORTANT PRODUCTION SECURITY NOTICE ⚠️
///
/// **The file-based key loading provided by this module is intended for DEVELOPMENT ONLY.**
/// In production environments, you MUST use more secure key management solutions.
///
/// ### Recommended Production Key Management Solutions:
///
/// #### 1. Operating System Keychains
/// Use OS-native secure storage via the [`keyring`](https://crates.io/crates/keyring) crate:
///
/// ```toml
/// [dependencies]
/// keyring = "2.0"
/// ```
///
/// ```rust,ignore
/// use keyring::Entry;
///
/// // Store key (one-time setup)
/// let entry = Entry::new("riglr", "solana_private_key")?;
/// entry.set_password("your_base58_private_key_here")?;
///
/// // Retrieve key in your application
/// let private_key = entry.get_password()?;
/// ```
///
/// #### 2. Hardware Security Modules (HSMs)
/// - **AWS `CloudHSM`**: Enterprise-grade hardware security
/// - **`YubiHSM`**: Compact, affordable HSM for smaller deployments
/// - **Ledger/Trezor**: For development and testing with hardware wallets
///
/// #### 3. Cloud-Based Secret Managers
/// - **AWS Secrets Manager**: Fully managed secrets with automatic rotation
/// - **Google Cloud Secret Manager**: Secure, convenient secret storage
/// - **Azure Key Vault**: Enterprise identity and access management
/// - **`HashiCorp` Vault**: Open-source secret management with audit trails
///
/// Example with AWS Secrets Manager:
/// ```rust,ignore
/// use aws_sdk_secretsmanager::Client;
///
/// let config = aws_config::load_from_env().await;
/// let client = Client::new(&config);
/// let response = client
///     .get_secret_value()
///     .secret_id("riglr/solana/private-key")
///     .send()
///     .await?;
/// let private_key = response.secret_string().unwrap();
/// ```
///
/// ### Security Best Practices:
/// - **Principle of Least Privilege**: Only allow necessary access to keys
/// - **Audit Logging**: Log all key access for security monitoring
/// - **Key Rotation**: Regularly rotate private keys and update storage
/// - **Multi-Factor Authentication**: Require MFA for key access
/// - **Encryption at Rest**: Ensure keys are encrypted when stored
/// - **Network Security**: Use TLS for all key transmission
/// - **Environment Isolation**: Separate development and production key storage
///
/// ### File-Based Storage Risks:
/// - Keys stored in plaintext on disk
/// - Vulnerable to file system access
/// - No audit trail of key usage
/// - Risk of accidental commits to version control
/// - No automatic key rotation capabilities
///
/// **Use the file-based utilities in this module only for:**
/// - Local development and testing
/// - Proof-of-concept applications
/// - Educational purposes
///
/// **Never use file-based storage for:**
/// - Production applications
/// - Applications handling real funds
/// - Multi-user or hosted environments
pub mod secure_keys {
    use crate::ToolError;
    use std::path::{Path, PathBuf};
    use std::{env, fs};

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
    /// use riglr_core::util::secure_keys::load_private_key_from_file;
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
    ///
    /// # Errors
    /// Returns `ToolError` when:
    /// - The key file does not exist
    /// - File permissions cannot be read (Unix systems)
    /// - File content cannot be read due to I/O errors
    /// - Home directory cannot be determined for path expansion
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
            // The ? operator is used here for error conversion as it's a common Rust idiom
            // and provides clearer error handling than manual match expressions
            let metadata = fs::metadata(&expanded_path).map_err(|metadata_err| {
                ToolError::permanent_string(format!("Failed to read file metadata: {metadata_err}"))
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
        // The ? operator is used here for error conversion as it's a common Rust idiom
        // and provides clearer error handling than manual match expressions
        let key_content = fs::read_to_string(&expanded_path).map_err(|read_err| {
            ToolError::permanent_string(format!("Failed to read key file: {read_err}"))
        })?;

        // Trim whitespace and return
        Ok(key_content.trim().to_owned())
    }

    /// Load a private key with fallback to environment variable
    ///
    /// This function first attempts to load from a file, then falls back to
    /// an environment variable if the file doesn't exist.
    ///
    /// # Example
    /// ```no_run
    /// use riglr_core::util::secure_keys::load_private_key_with_fallback;
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
    ///
    /// # Errors
    /// Returns `ToolError` when:
    /// - Both the key file and environment variable are unavailable
    /// - File loading fails and environment variable is not set
    /// - Environment variable exists but cannot be read
    pub fn load_private_key_with_fallback<P: AsRef<Path>>(
        file_path: P,
        env_var: &str,
    ) -> Result<String, ToolError> {
        load_private_key_from_file(&file_path).map_or_else(
            |_env_fallback| {
                // Fall back to environment variable
                tracing::debug!(
                    "Key file not found, trying environment variable: {}",
                    env_var
                );
                env::var(env_var).map_err(|_env_err| {
                    ToolError::permanent_string(format!(
                        "Private key not found in file {} or environment variable {}",
                        file_path.as_ref().display(),
                        env_var
                    ))
                })
            },
            |key| {
                tracing::debug!("Loaded key from file: {}", file_path.as_ref().display());
                Ok(key)
            },
        )
    }

    /// Get the default key directory for riglr
    ///
    /// Returns ~/.riglr/keys on Unix-like systems
    /// Returns %APPDATA%\riglr\keys on Windows
    pub fn get_default_key_directory() -> PathBuf {
        let base_dir = if cfg!(target_os = "windows") {
            env::var("APPDATA").map_or_else(
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
                }
                return Err(ToolError::permanent_string(
                    "Unable to determine home directory".to_string(),
                ));
            }
        }
        Ok(path.to_path_buf())
    }

    /// Create the default key directory with appropriate permissions
    ///
    /// # Errors
    /// Returns `ToolError` when:
    /// - Directory creation fails due to insufficient permissions
    /// - Setting directory permissions fails (Unix systems)
    /// - Filesystem I/O errors occur during directory creation
    pub fn ensure_key_directory() -> Result<PathBuf, ToolError> {
        let key_dir = get_default_key_directory();

        if !key_dir.exists() {
            // The ? operator is used here for error conversion as it's a common Rust idiom
            // and provides clearer error handling than manual match expressions
            fs::create_dir_all(&key_dir).map_err(|create_err| {
                ToolError::permanent_string(format!("Failed to create key directory: {create_err}"))
            })?;

            // Set restrictive permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let permissions = fs::Permissions::from_mode(0o700);
                // The ? operator is used here for error conversion as it's a common Rust idiom
                // and provides clearer error handling than manual match expressions
                fs::set_permissions(&key_dir, permissions).map_err(|perm_err| {
                    ToolError::permanent_string(format!(
                        "Failed to set directory permissions: {perm_err}"
                    ))
                })?;
            }
        }

        Ok(key_dir)
    }

    #[cfg(test)]
    #[expect(clippy::unwrap_used)]
    // Test functions use unsafe std::env operations for Rust 2024 compatibility with proper safety documentation
    mod tests {
        use super::*;
        use std::{env, io::Write};
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
            // SAFETY: This is a test function and we're only setting a test environment variable
            // temporarily for the duration of this test. The variable is cleaned up afterwards.
            #[expect(unsafe_code)]
            unsafe {
                env::set_var(TEST_PRIVATE_KEY_ENV, "env-key-content");
            }

            let key =
                load_private_key_with_fallback("/nonexistent/path/to/key", TEST_PRIVATE_KEY_ENV)
                    .unwrap();

            assert_eq!(key, "env-key-content");

            // SAFETY: This is a test cleanup operation, removing the test environment variable
            // that was set earlier in this same test function.
            #[expect(unsafe_code)]
            unsafe {
                env::remove_var(TEST_PRIVATE_KEY_ENV);
            }
        }

        #[test]
        fn test_get_default_key_directory() {
            let key_dir = get_default_key_directory();
            assert!(key_dir.ends_with(".riglr/keys") || key_dir.ends_with("riglr\\keys"));
        }

        #[test]
        fn test_expand_home_dir() {
            let expanded = expand_home_dir(Path::new("~/test")).unwrap();
            assert!(!expanded.to_str().unwrap().starts_with('~'));

            let not_expanded = expand_home_dir(Path::new("/absolute/path")).unwrap();
            assert_eq!(not_expanded, Path::new("/absolute/path"));
        }
    }
}

// Re-export commonly used items for library API convenience
pub use rate_limit_strategy::*;
pub use rate_limiter::{Builder as RateLimiterBuilder, RateLimitStrategyType, RateLimiter};
pub use secure_keys::*;
pub use token_bucket::BucketStrategy;
