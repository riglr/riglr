//! Privy-specific types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Privy user data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivyUserData {
    /// User ID
    pub id: String,

    /// Linked accounts
    pub linked_accounts: Vec<LinkedAccount>,

    /// Whether user is verified
    #[serde(default)]
    pub verified: bool,

    /// User metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl PrivyUserData {
    /// Get Solana wallet if available
    pub fn solana_wallet(&self) -> Option<&PrivyWallet> {
        self.linked_accounts
            .iter()
            .find_map(|account| match account {
                LinkedAccount::Wallet(w) if w.chain_type == "solana" && w.delegated => Some(w),
                _ => None,
            })
    }

    /// Get EVM wallet if available
    pub fn evm_wallet(&self) -> Option<&PrivyWallet> {
        self.linked_accounts
            .iter()
            .find_map(|account| match account {
                LinkedAccount::Wallet(w) if w.chain_type == "ethereum" && w.delegated => Some(w),
                _ => None,
            })
    }

    /// Get email if available
    pub fn email(&self) -> Option<String> {
        self.linked_accounts
            .iter()
            .find_map(|account| match account {
                LinkedAccount::Email { address, .. } => Some(address.clone()),
                _ => None,
            })
    }
}

/// Linked account types in Privy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LinkedAccount {
    /// Wallet account
    Wallet(PrivyWallet),

    /// Email account
    Email {
        /// Email address
        address: String,
        /// Whether the email is verified
        #[serde(default)]
        verified: bool,
    },

    /// Phone account
    Phone {
        /// Phone number
        number: String,
        /// Whether the phone number is verified
        #[serde(default)]
        verified: bool,
    },

    /// Social account
    Social {
        /// Social media provider (e.g., "google", "discord")
        provider: String,
        /// Username on the social platform
        username: Option<String>,
        /// Whether the social account is verified
        #[serde(default)]
        verified: bool,
    },

    /// Other account types
    #[serde(other)]
    Other,
}

/// Privy wallet information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivyWallet {
    /// Wallet ID
    #[serde(default)]
    pub id: Option<String>,

    /// Wallet address
    pub address: String,

    /// Chain type (solana, ethereum, etc.)
    pub chain_type: String,

    /// Wallet client type
    #[serde(default)]
    pub wallet_client: String,

    /// Whether this is a delegated wallet
    #[serde(default)]
    pub delegated: bool,

    /// Whether wallet is imported
    #[serde(default)]
    pub imported: bool,
}

/// JWT claims for Privy tokens
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PrivyClaims {
    /// Subject (user ID)
    pub sub: String,

    /// Audience (app ID)
    pub aud: String,

    /// Issuer
    pub iss: String,

    /// Session ID
    #[serde(default)]
    pub sid: String,

    /// Expiration time
    pub exp: i64,

    /// Issued at
    pub iat: i64,
}

/// Privy RPC request structure
#[derive(Debug, Serialize)]
pub struct PrivyRpcRequest {
    /// Wallet address
    pub address: String,

    /// Chain type
    pub chain_type: String,

    /// RPC method
    pub method: String,

    /// CAIP-2 chain identifier
    pub caip2: String,

    /// Method parameters
    pub params: serde_json::Value,
}

/// Privy RPC response structure
#[derive(Debug, Deserialize)]
pub struct PrivyRpcResponse {
    /// Response data
    pub data: serde_json::Value,
}

/// Privy transaction parameters for Solana
#[derive(Debug, Serialize)]
pub struct PrivySolanaTransactionParams {
    /// Base64-encoded transaction
    pub transaction: String,

    /// Encoding type
    pub encoding: String,
}

/// Privy transaction parameters for EVM
#[derive(Debug, Serialize)]
pub struct PrivyEvmTransactionParams {
    /// From address
    pub from: String,

    /// To address
    pub to: String,

    /// Value in hex
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Data in hex
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,

    /// Gas limit
    #[serde(skip_serializing_if = "Option::is_none", rename = "gasLimit")]
    pub gas_limit: Option<String>,

    /// Gas price
    #[serde(skip_serializing_if = "Option::is_none", rename = "gasPrice")]
    pub gas_price: Option<String>,

    /// Transaction type
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub tx_type: Option<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_privy_user_data_solana_wallet_when_delegated_solana_wallet_exists_should_return_some() {
        let wallet = PrivyWallet {
            id: Some("wallet1".to_string()),
            address: "SolanaAddress123".to_string(),
            chain_type: "solana".to_string(),
            wallet_client: "phantom".to_string(),
            delegated: true,
            imported: false,
        };

        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![LinkedAccount::Wallet(wallet.clone())],
            verified: true,
            metadata: HashMap::default(),
        };

        let result = user_data.solana_wallet();
        assert!(result.is_some());
        assert_eq!(result.unwrap().address, "SolanaAddress123");
        assert_eq!(result.unwrap().chain_type, "solana");
        assert!(result.unwrap().delegated);
    }

    #[test]
    fn test_privy_user_data_solana_wallet_when_non_delegated_solana_wallet_exists_should_return_none(
    ) {
        let wallet = PrivyWallet {
            id: Some("wallet1".to_string()),
            address: "SolanaAddress123".to_string(),
            chain_type: "solana".to_string(),
            wallet_client: "phantom".to_string(),
            delegated: false,
            imported: false,
        };

        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![LinkedAccount::Wallet(wallet)],
            verified: true,
            metadata: HashMap::default(),
        };

        let result = user_data.solana_wallet();
        assert!(result.is_none());
    }

    #[test]
    fn test_privy_user_data_solana_wallet_when_ethereum_wallet_exists_should_return_none() {
        let wallet = PrivyWallet {
            id: Some("wallet1".to_string()),
            address: "0x123".to_string(),
            chain_type: "ethereum".to_string(),
            wallet_client: "metamask".to_string(),
            delegated: true,
            imported: false,
        };

        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![LinkedAccount::Wallet(wallet)],
            verified: true,
            metadata: HashMap::default(),
        };

        let result = user_data.solana_wallet();
        assert!(result.is_none());
    }

    #[test]
    fn test_privy_user_data_solana_wallet_when_no_wallets_exist_should_return_none() {
        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![LinkedAccount::Email {
                address: "test@example.com".to_string(),
                verified: true,
            }],
            verified: true,
            metadata: HashMap::default(),
        };

        let result = user_data.solana_wallet();
        assert!(result.is_none());
    }

    #[test]
    fn test_privy_user_data_solana_wallet_when_empty_linked_accounts_should_return_none() {
        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![],
            verified: true,
            metadata: HashMap::default(),
        };

        let result = user_data.solana_wallet();
        assert!(result.is_none());
    }

    #[test]
    fn test_privy_user_data_evm_wallet_when_delegated_ethereum_wallet_exists_should_return_some() {
        let wallet = PrivyWallet {
            id: Some("wallet1".to_string()),
            address: "0x123456789".to_string(),
            chain_type: "ethereum".to_string(),
            wallet_client: "metamask".to_string(),
            delegated: true,
            imported: false,
        };

        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![LinkedAccount::Wallet(wallet.clone())],
            verified: true,
            metadata: HashMap::default(),
        };

        let result = user_data.evm_wallet();
        assert!(result.is_some());
        assert_eq!(result.unwrap().address, "0x123456789");
        assert_eq!(result.unwrap().chain_type, "ethereum");
        assert!(result.unwrap().delegated);
    }

    #[test]
    fn test_privy_user_data_evm_wallet_when_non_delegated_ethereum_wallet_exists_should_return_none(
    ) {
        let wallet = PrivyWallet {
            id: Some("wallet1".to_string()),
            address: "0x123456789".to_string(),
            chain_type: "ethereum".to_string(),
            wallet_client: "metamask".to_string(),
            delegated: false,
            imported: false,
        };

        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![LinkedAccount::Wallet(wallet)],
            verified: true,
            metadata: HashMap::default(),
        };

        let result = user_data.evm_wallet();
        assert!(result.is_none());
    }

    #[test]
    fn test_privy_user_data_evm_wallet_when_solana_wallet_exists_should_return_none() {
        let wallet = PrivyWallet {
            id: Some("wallet1".to_string()),
            address: "SolanaAddress123".to_string(),
            chain_type: "solana".to_string(),
            wallet_client: "phantom".to_string(),
            delegated: true,
            imported: false,
        };

        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![LinkedAccount::Wallet(wallet)],
            verified: true,
            metadata: HashMap::default(),
        };

        let result = user_data.evm_wallet();
        assert!(result.is_none());
    }

    #[test]
    fn test_privy_user_data_evm_wallet_when_no_wallets_exist_should_return_none() {
        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![LinkedAccount::Phone {
                number: "+1234567890".to_string(),
                verified: true,
            }],
            verified: true,
            metadata: HashMap::default(),
        };

        let result = user_data.evm_wallet();
        assert!(result.is_none());
    }

    #[test]
    fn test_privy_user_data_email_when_email_account_exists_should_return_some() {
        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![LinkedAccount::Email {
                address: "test@example.com".to_string(),
                verified: true,
            }],
            verified: true,
            metadata: HashMap::default(),
        };

        let result = user_data.email();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "test@example.com");
    }

    #[test]
    fn test_privy_user_data_email_when_multiple_accounts_with_email_should_return_first_email() {
        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![
                LinkedAccount::Phone {
                    number: "+1234567890".to_string(),
                    verified: true,
                },
                LinkedAccount::Email {
                    address: "first@example.com".to_string(),
                    verified: true,
                },
                LinkedAccount::Email {
                    address: "second@example.com".to_string(),
                    verified: false,
                },
            ],
            verified: true,
            metadata: HashMap::default(),
        };

        let result = user_data.email();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "first@example.com");
    }

    #[test]
    fn test_privy_user_data_email_when_no_email_account_exists_should_return_none() {
        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![LinkedAccount::Phone {
                number: "+1234567890".to_string(),
                verified: true,
            }],
            verified: true,
            metadata: HashMap::default(),
        };

        let result = user_data.email();
        assert!(result.is_none());
    }

    #[test]
    fn test_privy_user_data_email_when_empty_linked_accounts_should_return_none() {
        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![],
            verified: true,
            metadata: HashMap::default(),
        };

        let result = user_data.email();
        assert!(result.is_none());
    }

    #[test]
    fn test_privy_user_data_creation_with_defaults() {
        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![],
            verified: false,
            metadata: HashMap::default(),
        };

        assert_eq!(user_data.id, "user123");
        assert!(user_data.linked_accounts.is_empty());
        assert!(!user_data.verified);
        assert!(user_data.metadata.is_empty());
    }

    #[test]
    fn test_privy_user_data_creation_with_metadata() {
        let mut metadata = HashMap::default();
        metadata.insert("role".to_string(), json!("admin"));
        metadata.insert("preferences".to_string(), json!({"theme": "dark"}));

        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![],
            verified: true,
            metadata,
        };

        assert_eq!(user_data.metadata.len(), 2);
        assert_eq!(user_data.metadata.get("role").unwrap(), &json!("admin"));
    }

    #[test]
    fn test_linked_account_email_serialization() {
        let email_account = LinkedAccount::Email {
            address: "test@example.com".to_string(),
            verified: true,
        };

        let serialized = serde_json::to_string(&email_account).unwrap();
        assert!(serialized.contains("\"type\":\"email\""));
        assert!(serialized.contains("\"address\":\"test@example.com\""));
        assert!(serialized.contains("\"verified\":true"));
    }

    #[test]
    fn test_linked_account_phone_serialization() {
        let phone_account = LinkedAccount::Phone {
            number: "+1234567890".to_string(),
            verified: false,
        };

        let serialized = serde_json::to_string(&phone_account).unwrap();
        assert!(serialized.contains("\"type\":\"phone\""));
        assert!(serialized.contains("\"number\":\"+1234567890\""));
        assert!(serialized.contains("\"verified\":false"));
    }

    #[test]
    fn test_linked_account_social_serialization() {
        let social_account = LinkedAccount::Social {
            provider: "google".to_string(),
            username: Some("testuser".to_string()),
            verified: true,
        };

        let serialized = serde_json::to_string(&social_account).unwrap();
        assert!(serialized.contains("\"type\":\"social\""));
        assert!(serialized.contains("\"provider\":\"google\""));
        assert!(serialized.contains("\"username\":\"testuser\""));
        assert!(serialized.contains("\"verified\":true"));
    }

    #[test]
    fn test_linked_account_social_without_username() {
        let social_account = LinkedAccount::Social {
            provider: "discord".to_string(),
            username: None,
            verified: false,
        };

        let serialized = serde_json::to_string(&social_account).unwrap();
        assert!(serialized.contains("\"type\":\"social\""));
        assert!(serialized.contains("\"provider\":\"discord\""));
        assert!(serialized.contains("\"username\":null"));
        assert!(serialized.contains("\"verified\":false"));
    }

    #[test]
    fn test_linked_account_wallet_serialization() {
        let wallet = PrivyWallet {
            id: Some("wallet123".to_string()),
            address: "0xabc123".to_string(),
            chain_type: "ethereum".to_string(),
            wallet_client: "metamask".to_string(),
            delegated: true,
            imported: false,
        };

        let wallet_account = LinkedAccount::Wallet(wallet);
        let serialized = serde_json::to_string(&wallet_account).unwrap();
        assert!(serialized.contains("\"type\":\"wallet\""));
        assert!(serialized.contains("\"address\":\"0xabc123\""));
        assert!(serialized.contains("\"chain_type\":\"ethereum\""));
    }

    #[test]
    fn test_linked_account_other_deserialization() {
        let json_str = r#"{"type":"unknown_type","data":"some_data"}"#;
        let account: LinkedAccount = serde_json::from_str(json_str).unwrap();

        match account {
            LinkedAccount::Other => {}
            _ => panic!("Expected Other variant"),
        }
    }

    #[test]
    fn test_privy_wallet_with_all_fields() {
        let wallet = PrivyWallet {
            id: Some("wallet123".to_string()),
            address: "SolanaAddress456".to_string(),
            chain_type: "solana".to_string(),
            wallet_client: "phantom".to_string(),
            delegated: true,
            imported: true,
        };

        assert_eq!(wallet.id.as_ref().unwrap(), "wallet123");
        assert_eq!(wallet.address, "SolanaAddress456");
        assert_eq!(wallet.chain_type, "solana");
        assert_eq!(wallet.wallet_client, "phantom");
        assert!(wallet.delegated);
        assert!(wallet.imported);
    }

    #[test]
    fn test_privy_wallet_with_defaults() {
        let wallet = PrivyWallet {
            id: None,
            address: "0x123".to_string(),
            chain_type: "ethereum".to_string(),
            wallet_client: String::default(),
            delegated: false,
            imported: false,
        };

        assert!(wallet.id.is_none());
        assert_eq!(wallet.address, "0x123");
        assert_eq!(wallet.chain_type, "ethereum");
        assert_eq!(wallet.wallet_client, "");
        assert!(!wallet.delegated);
        assert!(!wallet.imported);
    }

    #[test]
    fn test_privy_claims_deserialization() {
        let json_str = r#"{
            "sub": "user123",
            "aud": "app456",
            "iss": "privy.io",
            "sid": "session789",
            "exp": 1672531200,
            "iat": 1672444800
        }"#;

        let claims: PrivyClaims = serde_json::from_str(json_str).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.aud, "app456");
        assert_eq!(claims.iss, "privy.io");
        assert_eq!(claims.sid, "session789");
        assert_eq!(claims.exp, 1672531200);
        assert_eq!(claims.iat, 1672444800);
    }

    #[test]
    fn test_privy_claims_deserialization_with_default_sid() {
        let json_str = r#"{
            "sub": "user123",
            "aud": "app456",
            "iss": "privy.io",
            "exp": 1672531200,
            "iat": 1672444800
        }"#;

        let claims: PrivyClaims = serde_json::from_str(json_str).unwrap();
        assert_eq!(claims.sid, "");
    }

    #[test]
    fn test_privy_rpc_request_serialization() {
        let request = PrivyRpcRequest {
            address: "0x123".to_string(),
            chain_type: "ethereum".to_string(),
            method: "eth_sendTransaction".to_string(),
            caip2: "eip155:1".to_string(),
            params: json!({"value": "0x1"}),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("\"address\":\"0x123\""));
        assert!(serialized.contains("\"chain_type\":\"ethereum\""));
        assert!(serialized.contains("\"method\":\"eth_sendTransaction\""));
        assert!(serialized.contains("\"caip2\":\"eip155:1\""));
        assert!(serialized.contains("\"params\":{\"value\":\"0x1\"}"));
    }

    #[test]
    fn test_privy_rpc_response_deserialization() {
        let json_str = r#"{"data": {"result": "0xabc123"}}"#;

        let response: PrivyRpcResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(response.data["result"], "0xabc123");
    }

    #[test]
    fn test_privy_solana_transaction_params_serialization() {
        let params = PrivySolanaTransactionParams {
            transaction: "base64encodedtransaction".to_string(),
            encoding: "base64".to_string(),
        };

        let serialized = serde_json::to_string(&params).unwrap();
        assert!(serialized.contains("\"transaction\":\"base64encodedtransaction\""));
        assert!(serialized.contains("\"encoding\":\"base64\""));
    }

    #[test]
    fn test_privy_evm_transaction_params_serialization_with_all_fields() {
        let params = PrivyEvmTransactionParams {
            from: "0x123".to_string(),
            to: "0x456".to_string(),
            value: Some("0x1".to_string()),
            data: Some("0xabcd".to_string()),
            gas_limit: Some("21000".to_string()),
            gas_price: Some("20000000000".to_string()),
            tx_type: Some(2),
        };

        let serialized = serde_json::to_string(&params).unwrap();
        assert!(serialized.contains("\"from\":\"0x123\""));
        assert!(serialized.contains("\"to\":\"0x456\""));
        assert!(serialized.contains("\"value\":\"0x1\""));
        assert!(serialized.contains("\"data\":\"0xabcd\""));
        assert!(serialized.contains("\"gasLimit\":\"21000\""));
        assert!(serialized.contains("\"gasPrice\":\"20000000000\""));
        assert!(serialized.contains("\"type\":2"));
    }

    #[test]
    fn test_privy_evm_transaction_params_serialization_with_minimal_fields() {
        let params = PrivyEvmTransactionParams {
            from: "0x123".to_string(),
            to: "0x456".to_string(),
            value: None,
            data: None,
            gas_limit: None,
            gas_price: None,
            tx_type: None,
        };

        let serialized = serde_json::to_string(&params).unwrap();
        assert!(serialized.contains("\"from\":\"0x123\""));
        assert!(serialized.contains("\"to\":\"0x456\""));
        // Optional fields should be skipped
        assert!(!serialized.contains("\"value\""));
        assert!(!serialized.contains("\"data\""));
        assert!(!serialized.contains("\"gasLimit\""));
        assert!(!serialized.contains("\"gasPrice\""));
        assert!(!serialized.contains("\"type\""));
    }

    #[test]
    fn test_privy_user_data_with_all_account_types() {
        let wallet = PrivyWallet {
            id: Some("wallet1".to_string()),
            address: "0x123".to_string(),
            chain_type: "ethereum".to_string(),
            wallet_client: "metamask".to_string(),
            delegated: true,
            imported: false,
        };

        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![
                LinkedAccount::Wallet(wallet),
                LinkedAccount::Email {
                    address: "test@example.com".to_string(),
                    verified: true,
                },
                LinkedAccount::Phone {
                    number: "+1234567890".to_string(),
                    verified: false,
                },
                LinkedAccount::Social {
                    provider: "google".to_string(),
                    username: Some("testuser".to_string()),
                    verified: true,
                },
                LinkedAccount::Other,
            ],
            verified: true,
            metadata: HashMap::default(),
        };

        assert_eq!(user_data.linked_accounts.len(), 5);
        assert!(user_data.evm_wallet().is_some());
        assert!(user_data.email().is_some());
    }

    #[test]
    fn test_privy_user_data_complex_metadata() {
        let mut metadata = HashMap::default();
        metadata.insert(
            "nested".to_string(),
            json!({
                "level1": {
                    "level2": ["array", "values"]
                }
            }),
        );
        metadata.insert("null_value".to_string(), json!(null));
        metadata.insert("number".to_string(), json!(42));
        metadata.insert("boolean".to_string(), json!(true));

        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![],
            verified: false,
            metadata,
        };

        assert_eq!(user_data.metadata.len(), 4);
        assert_eq!(user_data.metadata.get("number").unwrap(), &json!(42));
        assert_eq!(user_data.metadata.get("boolean").unwrap(), &json!(true));
        assert!(user_data.metadata.get("null_value").unwrap().is_null());
    }

    #[test]
    fn test_debug_implementations() {
        let wallet = PrivyWallet {
            id: Some("wallet1".to_string()),
            address: "0x123".to_string(),
            chain_type: "ethereum".to_string(),
            wallet_client: "metamask".to_string(),
            delegated: true,
            imported: false,
        };

        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![LinkedAccount::Wallet(wallet.clone())],
            verified: true,
            metadata: HashMap::default(),
        };

        let claims = PrivyClaims {
            sub: "user123".to_string(),
            aud: "app456".to_string(),
            iss: "privy.io".to_string(),
            sid: "session789".to_string(),
            exp: 1672531200,
            iat: 1672444800,
        };

        let rpc_request = PrivyRpcRequest {
            address: "0x123".to_string(),
            chain_type: "ethereum".to_string(),
            method: "eth_sendTransaction".to_string(),
            caip2: "eip155:1".to_string(),
            params: json!({}),
        };

        let rpc_response = PrivyRpcResponse {
            data: json!({"result": "success"}),
        };

        let solana_params = PrivySolanaTransactionParams {
            transaction: "base64".to_string(),
            encoding: "base64".to_string(),
        };

        let evm_params = PrivyEvmTransactionParams {
            from: "0x123".to_string(),
            to: "0x456".to_string(),
            value: None,
            data: None,
            gas_limit: None,
            gas_price: None,
            tx_type: None,
        };

        // Test that debug formatting works without panicking
        let _ = format!("{:?}", user_data);
        let _ = format!("{:?}", wallet);
        let _ = format!("{:?}", LinkedAccount::Other);
        let _ = format!("{:?}", claims);
        let _ = format!("{:?}", rpc_request);
        let _ = format!("{:?}", rpc_response);
        let _ = format!("{:?}", solana_params);
        let _ = format!("{:?}", evm_params);
    }

    #[test]
    fn test_clone_implementations() {
        let wallet = PrivyWallet {
            id: Some("wallet1".to_string()),
            address: "0x123".to_string(),
            chain_type: "ethereum".to_string(),
            wallet_client: "metamask".to_string(),
            delegated: true,
            imported: false,
        };

        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![LinkedAccount::Wallet(wallet.clone())],
            verified: true,
            metadata: HashMap::default(),
        };

        let cloned_wallet = wallet.clone();
        let cloned_user_data = user_data.clone();
        let cloned_account = LinkedAccount::Other.clone();

        assert_eq!(wallet.address, cloned_wallet.address);
        assert_eq!(user_data.id, cloned_user_data.id);

        match cloned_account {
            LinkedAccount::Other => {}
            _ => panic!("Clone should preserve variant"),
        }
    }
}
