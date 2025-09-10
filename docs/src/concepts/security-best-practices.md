# Security Best Practices for riglr

This document outlines security best practices for using riglr in production environments.

## Private Key Management

### ❌ Never Do This

```rust
// NEVER hardcode private keys
let private_key = "5JuE4z..."; // DON'T DO THIS!

// AVOID using environment variables for production keys
let key = std::env::var("PRIVATE_KEY").unwrap(); // Not recommended for production
```

### ✅ Recommended Approach

riglr provides secure key loading utilities in `riglr_core::util`:

```rust
use riglr_core::util::{load_private_key_from_file, load_private_key_with_fallback};

// Load from secure file location
let key = load_private_key_from_file("~/.riglr/keys/solana.key")?;

// Load with fallback (file first, then env var)
let key = load_private_key_with_fallback(
    "~/.riglr/keys/solana.key",
    "SOLANA_PRIVATE_KEY"  // Only used if file doesn't exist
)?;
```

### File Permissions

On Unix-like systems, ensure your key files have restrictive permissions:

```bash
# Create secure key directory
mkdir -p ~/.riglr/keys
chmod 700 ~/.riglr/keys

# Set restrictive permissions on key files
chmod 600 ~/.riglr/keys/solana.key
chmod 600 ~/.riglr/keys/evm.key
```

The `load_private_key_from_file` function will warn if file permissions are too permissive.

### Key File Formats

Supported formats:
- **Base58** (Solana): Plain text base58-encoded private key
- **Hex** (EVM): Plain text hex-encoded private key (with or without 0x prefix)
- **Comma-separated bytes**: Legacy format `1,2,3,...,64`

### Production Recommendations

For production environments, consider:

1. **Hardware Security Modules (HSMs)**: Store keys in dedicated hardware
2. **Key Management Services**: Use AWS KMS, Azure Key Vault, or similar
3. **Encrypted Storage**: Encrypt keys at rest using system keyrings
4. **Multi-signature Wallets**: Require multiple signatures for critical operations
5. **Key Rotation**: Regularly rotate keys and maintain audit logs

## Rate Limiting

riglr includes built-in rate limiting to prevent abuse:

```rust
use riglr_core::util::RateLimiter;
use std::time::Duration;

// Configure rate limiter
let rate_limiter = RateLimiter::builder()
    .max_requests(100)
    .time_window(Duration::from_secs(60))
    .burst_size(20)  // Allow temporary spikes
    .build();

// Add to ApplicationContext for dependency injection
context.set_extension(Arc::new(rate_limiter));
```

## SignerContext Security

The `SignerContext` pattern ensures signing operations are properly isolated:

```rust
use riglr_core::SignerContext;

// Signer is only available within this context
SignerContext::with_signer(signer, async {
    // Signing operations here
    transfer_sol(recipient, amount).await
}).await?;
// Signer is automatically cleaned up
```

## Environment Variables

### Development vs Production

**Development** (`.env.test`):
```env
# OK for development
GEMINI_API_KEY=test-key
SOLANA_RPC_URL=https://api.devnet.solana.com
```

**Production**:
- Use proper secret management (AWS Secrets Manager, Kubernetes Secrets, etc.)
- Never commit `.env` files with real credentials
- Rotate API keys regularly
- Use read-only keys where possible

### Configuration Validation

riglr validates configuration at startup:

```rust
let config = Config::from_env(); // Will panic if required vars are missing
```

## Network Security

### RPC Endpoints

1. **Use HTTPS**: Always use HTTPS endpoints for RPC connections
2. **Private RPCs**: Consider using private RPC nodes for production
3. **Rate Limiting**: Implement rate limiting on your RPC requests
4. **Fallback Nodes**: Configure multiple RPC endpoints for redundancy

### Transaction Security

1. **Simulation**: Always simulate transactions before execution
2. **Slippage Protection**: Configure appropriate slippage tolerances
3. **Gas Limits**: Set reasonable gas limits to prevent excessive fees
4. **Nonce Management**: Properly manage transaction nonces to prevent replay attacks

## Audit Logging

Enable comprehensive logging for security events:

```rust
use tracing::{info, warn};

// Log all signing operations
info!(address = %signer.address(), "Initiating transaction");

// Log rate limit violations
warn!(client = %client_id, "Rate limit exceeded");
```

## Error Handling

Never expose sensitive information in error messages:

```rust
// ❌ Bad: Exposes internal details
return Err(format!("Failed to sign with key: {}", private_key));

// ✅ Good: Generic error message
return Err(ToolError::permanent_string("Signing operation failed"));
```

## Testing Security

1. **Use Testnets**: Always test on testnets before mainnet
2. **Separate Keys**: Use different keys for development, testing, and production
3. **Security Scans**: Regularly run security audits with tools like `cargo-audit`
4. **Penetration Testing**: Consider professional security audits for production systems

## Compliance

Ensure your implementation complies with:
- **GDPR**: For handling European user data
- **SOC 2**: For service organizations
- **PCI DSS**: If handling payment card data
- **Local Regulations**: Comply with local cryptocurrency regulations

## Incident Response

Have a plan for security incidents:
1. **Key Compromise**: Immediately rotate affected keys
2. **Rate Limit Bypass**: Block offending IPs/accounts
3. **Vulnerability Discovery**: Follow responsible disclosure practices
4. **Post-Mortem**: Document and learn from incidents

## Additional Resources

- [OWASP Cryptocurrency Security](https://owasp.org/www-community/vulnerabilities/Cryptocurrency_Security)
- [Ethereum Smart Contract Security Best Practices](https://consensys.github.io/smart-contract-best-practices/)
- [Solana Security Best Practices](https://docs.solana.com/security-best-practices)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)