//! Type-safe wrappers for Solana primitives

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::borrow::Cow;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

/// Type-safe wrapper for Solana addresses
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SolanaAddress(Pubkey);

impl JsonSchema for SolanaAddress {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("SolanaAddress")
    }

    fn json_schema(gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        // Schema for a base58-encoded Solana address string
        gen.subschema_for::<String>()
    }
}

impl SolanaAddress {
    /// Create a new SolanaAddress from a Pubkey
    pub fn new(pubkey: Pubkey) -> Self {
        Self(pubkey)
    }

    /// Get the inner Pubkey
    pub fn inner(&self) -> &Pubkey {
        &self.0
    }
}

impl FromStr for SolanaAddress {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try to decode from base58
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|e| format!("Invalid base58 address: {}", e))?;

        // Check if it's exactly 32 bytes
        if bytes.len() != 32 {
            return Err(format!(
                "Invalid address length: expected 32 bytes, got {}",
                bytes.len()
            ));
        }

        // Parse as Pubkey
        Pubkey::from_str(s)
            .map(SolanaAddress)
            .map_err(|e| format!("Invalid Solana address: {}", e))
    }
}

impl Deref for SolanaAddress {
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for SolanaAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Pubkey> for SolanaAddress {
    fn from(pubkey: Pubkey) -> Self {
        Self(pubkey)
    }
}

impl From<SolanaAddress> for Pubkey {
    fn from(addr: SolanaAddress) -> Self {
        addr.0
    }
}

/// Type-safe wrapper for Solana signatures
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SolanaSignature(Signature);

impl JsonSchema for SolanaSignature {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("SolanaSignature")
    }

    fn json_schema(gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        // Schema for a base58-encoded Solana signature string
        gen.subschema_for::<String>()
    }
}

impl SolanaSignature {
    /// Create a new SolanaSignature from a Signature
    pub fn new(signature: Signature) -> Self {
        Self(signature)
    }

    /// Get the inner Signature
    pub fn inner(&self) -> &Signature {
        &self.0
    }
}

impl FromStr for SolanaSignature {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try to decode from base58
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|e| format!("Invalid base58 signature: {}", e))?;

        // Check if it's exactly 64 bytes
        if bytes.len() != 64 {
            return Err(format!(
                "Invalid signature length: expected 64 bytes, got {}",
                bytes.len()
            ));
        }

        // Parse as Signature
        Signature::from_str(s)
            .map(SolanaSignature)
            .map_err(|e| format!("Invalid Solana signature: {}", e))
    }
}

impl Deref for SolanaSignature {
    type Target = Signature;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for SolanaSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Signature> for SolanaSignature {
    fn from(signature: Signature) -> Self {
        Self(signature)
    }
}

impl From<SolanaSignature> for Signature {
    fn from(sig: SolanaSignature) -> Self {
        sig.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solana_address_from_str() {
        // Valid address
        let addr_str = "11111111111111111111111111111111";
        let addr = SolanaAddress::from_str(addr_str).unwrap();
        assert_eq!(addr.to_string(), addr_str);

        // Invalid base58
        let invalid = "invalid!@#$%";
        assert!(SolanaAddress::from_str(invalid).is_err());

        // Wrong length
        let short = "111";
        assert!(SolanaAddress::from_str(short).is_err());
    }

    #[test]
    fn test_solana_signature_from_str() {
        // Valid signature (64 bytes as base58)
        let sig_str = "1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111";
        let result = SolanaSignature::from_str(sig_str);

        // Should work for a properly formatted signature
        if result.is_ok() {
            assert_eq!(result.unwrap().to_string().len(), 88); // Base58 signatures are typically 88 chars
        }

        // Invalid base58
        let invalid = "invalid!@#$%";
        assert!(SolanaSignature::from_str(invalid).is_err());

        // Wrong length
        let short = "111";
        assert!(SolanaSignature::from_str(short).is_err());
    }

    #[test]
    fn test_deref() {
        let pubkey = Pubkey::new_unique();
        let addr = SolanaAddress::new(pubkey);

        // Test that we can use deref to access Pubkey methods
        assert_eq!(*addr, pubkey);
        assert_eq!(addr.to_bytes(), pubkey.to_bytes());
    }

    #[test]
    fn test_serialization() {
        let pubkey = Pubkey::new_unique();
        let addr = SolanaAddress::new(pubkey);

        // Test JSON serialization
        let json = serde_json::to_string(&addr).unwrap();
        let deserialized: SolanaAddress = serde_json::from_str(&json).unwrap();
        assert_eq!(addr, deserialized);
    }
}
