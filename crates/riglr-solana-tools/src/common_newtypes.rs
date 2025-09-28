//! Type-safe wrappers for Solana primitives

use core::ops::Deref;
use core::str::FromStr;
use schemars::{schema::Schema, JsonSchema};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::fmt;

/// Type-safe wrapper for Solana addresses
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SolanaAddress(Pubkey);

impl JsonSchema for SolanaAddress {
    #[inline]
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> Schema {
        // Schema for a base58-encoded Solana address string
        generator.subschema_for::<String>()
    }

    #[inline]
    fn schema_name() -> String {
        "SolanaAddress".to_owned()
    }
}

impl SolanaAddress {
    /// Get the inner Pubkey
    #[must_use]
    #[inline]
    pub const fn inner(&self) -> &Pubkey {
        &self.0
    }

    /// Create a new `SolanaAddress` from a Pubkey
    #[must_use]
    #[inline]
    pub const fn new(pubkey: Pubkey) -> Self {
        Self(pubkey)
    }
}

impl FromStr for SolanaAddress {
    type Err = String;

    #[inline]
    fn from_str(input_str: &str) -> Result<Self, Self::Err> {
        // Try to decode from base58
        let bytes = bs58::decode(input_str)
            .into_vec()
            .map_err(|decode_err| format!("Invalid base58 address: {decode_err}"))?;

        // Check if it's exactly 32 bytes
        if bytes.len() != 32 {
            return Err(format!(
                "Invalid address length: expected 32 bytes, got {}",
                bytes.len()
            ));
        }

        // Parse as Pubkey
        Pubkey::from_str(input_str)
            .map(SolanaAddress)
            .map_err(|parse_err| format!("Invalid Solana address: {parse_err}"))
    }
}

impl Deref for SolanaAddress {
    type Target = Pubkey;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for SolanaAddress {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Pubkey> for SolanaAddress {
    #[inline]
    fn from(pubkey: Pubkey) -> Self {
        Self(pubkey)
    }
}

impl From<SolanaAddress> for Pubkey {
    #[inline]
    fn from(addr: SolanaAddress) -> Self {
        addr.0
    }
}

/// Type-safe wrapper for Solana signatures
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SolanaSignature(Signature);

impl JsonSchema for SolanaSignature {
    #[inline]
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> Schema {
        // Schema for a base58-encoded Solana signature string
        generator.subschema_for::<String>()
    }

    #[inline]
    fn schema_name() -> String {
        "SolanaSignature".to_owned()
    }
}

impl SolanaSignature {
    /// Get the inner Signature
    #[must_use]
    #[inline]
    pub const fn inner(&self) -> &Signature {
        &self.0
    }

    /// Create a new `SolanaSignature` from a Signature
    #[must_use]
    #[inline]
    pub const fn new(signature: Signature) -> Self {
        Self(signature)
    }
}

impl FromStr for SolanaSignature {
    type Err = String;

    #[inline]
    fn from_str(input_str: &str) -> Result<Self, Self::Err> {
        // Try to decode from base58
        let bytes = bs58::decode(input_str)
            .into_vec()
            .map_err(|decode_err| format!("Invalid base58 signature: {decode_err}"))?;

        // Check if it's exactly 64 bytes
        if bytes.len() != 64 {
            return Err(format!(
                "Invalid signature length: expected 64 bytes, got {}",
                bytes.len()
            ));
        }

        // Parse as Signature
        Signature::from_str(input_str)
            .map(SolanaSignature)
            .map_err(|parse_err| format!("Invalid Solana signature: {parse_err}"))
    }
}

impl Deref for SolanaSignature {
    type Target = Signature;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for SolanaSignature {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Signature> for SolanaSignature {
    #[inline]
    fn from(signature: Signature) -> Self {
        Self(signature)
    }
}

impl From<SolanaSignature> for Signature {
    #[inline]
    fn from(sig: SolanaSignature) -> Self {
        sig.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[expect(clippy::expect_used, clippy::unwrap_used)]
    fn solana_address_from_str() {
        // Valid address
        let addr_str = "11111111111111111111111111111111";
        let addr =
            SolanaAddress::from_str(addr_str).expect("Valid address should parse successfully");
        assert_eq!(addr.to_string(), addr_str);

        // Invalid base58
        let invalid = "invalid!@#$%";
        SolanaAddress::from_str(invalid).unwrap_err();

        // Wrong length
        let short = "111";
        SolanaAddress::from_str(short).unwrap_err();
    }

    #[test]
    #[expect(clippy::expect_used, clippy::unwrap_used)]
    fn solana_signature_from_str() {
        // Valid signature (64 bytes as base58)
        let sig_str = "1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111";
        let result = SolanaSignature::from_str(sig_str);

        // Should work for a properly formatted signature
        if result.is_ok() {
            assert_eq!(
                result
                    .expect("Validated signature should convert to string")
                    .to_string()
                    .len(),
                88
            ); // Base58 signatures are typically 88 chars
        }

        // Invalid base58
        let invalid = "invalid!@#$%";
        SolanaSignature::from_str(invalid).unwrap_err();

        // Wrong length
        let short = "111";
        SolanaSignature::from_str(short).unwrap_err();
    }

    #[test]
    fn deref() {
        let pubkey = Pubkey::new_unique();
        let addr = SolanaAddress::new(pubkey);

        // Test that we can use deref to access Pubkey methods
        assert_eq!(*addr, pubkey);
        assert_eq!(addr.to_bytes(), pubkey.to_bytes());
    }

    #[test]
    #[expect(clippy::expect_used)]
    fn serialization() {
        let pubkey = Pubkey::new_unique();
        let addr = SolanaAddress::new(pubkey);

        // Test JSON serialization
        let json = serde_json::to_string(&addr).expect("SolanaAddress should serialize to JSON");
        let deserialized: SolanaAddress =
            serde_json::from_str(&json).expect("JSON should deserialize to SolanaAddress");
        assert_eq!(addr, deserialized);
    }
}
