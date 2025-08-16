use riglr_core::signer::SignerError;

#[test]
fn signer_error_variants_coverage() {
    // Exercise Display on less common variants
    let e = SignerError::ClientCreation("x".into());
    assert!(e.to_string().contains("Client creation error"));

    let e = SignerError::ProviderError("x".into());
    assert!(e.to_string().contains("Provider error"));

    let e = SignerError::TransactionFailed("x".into());
    assert!(e.to_string().contains("Transaction failed"));

    let e = SignerError::BlockhashError("x".into());
    assert!(e.to_string().contains("Blockhash error"));

    let e = SignerError::InvalidRpcUrl("x".into());
    assert!(e.to_string().contains("Invalid RPC URL"));
}

#[test]
fn test_all_signer_error_variants() {
    // Test NoSignerContext variant
    let e = SignerError::NoSignerContext;
    assert!(e.to_string().contains("No signer context available"));
    assert!(e
        .to_string()
        .contains("must be called within SignerContext::with_signer"));

    // Test SolanaTransaction variant
    let client_error = Box::new(solana_client::client_error::ClientError {
        request: None,
        kind: solana_client::client_error::ClientErrorKind::Custom("test error".to_string()),
    });
    let e = SignerError::SolanaTransaction(client_error);
    assert!(e.to_string().contains("Solana transaction error"));

    // Test EvmTransaction variant
    let e = SignerError::EvmTransaction("evm error".to_string());
    assert!(e.to_string().contains("EVM transaction error"));
    assert!(e.to_string().contains("evm error"));

    // Test Configuration variant
    let e = SignerError::Configuration("config error".to_string());
    assert!(e.to_string().contains("Invalid configuration"));
    assert!(e.to_string().contains("config error"));

    // Test Signing variant
    let e = SignerError::Signing("signing error".to_string());
    assert!(e.to_string().contains("Signing error"));
    assert!(e.to_string().contains("signing error"));

    // Test ClientCreation variant
    let e = SignerError::ClientCreation("client error".to_string());
    assert!(e.to_string().contains("Client creation error"));
    assert!(e.to_string().contains("client error"));

    // Test InvalidPrivateKey variant
    let e = SignerError::InvalidPrivateKey("key error".to_string());
    assert!(e.to_string().contains("Invalid private key"));
    assert!(e.to_string().contains("key error"));

    // Test ProviderError variant
    let e = SignerError::ProviderError("provider error".to_string());
    assert!(e.to_string().contains("Provider error"));
    assert!(e.to_string().contains("provider error"));

    // Test TransactionFailed variant
    let e = SignerError::TransactionFailed("tx failed".to_string());
    assert!(e.to_string().contains("Transaction failed"));
    assert!(e.to_string().contains("tx failed"));

    // Test UnsupportedOperation variant
    let e = SignerError::UnsupportedOperation("not supported".to_string());
    assert!(e.to_string().contains("Unsupported operation"));
    assert!(e.to_string().contains("not supported"));

    // Test BlockhashError variant
    let e = SignerError::BlockhashError("blockhash error".to_string());
    assert!(e.to_string().contains("Blockhash error"));
    assert!(e.to_string().contains("blockhash error"));

    // Test UnsupportedNetwork variant
    let e = SignerError::UnsupportedNetwork("unknown network".to_string());
    assert!(e.to_string().contains("Unsupported network"));
    assert!(e.to_string().contains("unknown network"));

    // Test InvalidRpcUrl variant
    let e = SignerError::InvalidRpcUrl("bad url".to_string());
    assert!(e.to_string().contains("Invalid RPC URL"));
    assert!(e.to_string().contains("bad url"));
}

#[test]
fn test_signer_error_from_solana_signer_error() {
    // Test From implementation for solana_sdk::signer::SignerError
    let solana_error = solana_sdk::signer::SignerError::Custom("custom solana error".to_string());
    let signer_error: SignerError = solana_error.into();

    assert!(matches!(signer_error, SignerError::Signing(_)));
    assert!(signer_error.to_string().contains("Signing error"));
    assert!(signer_error.to_string().contains("custom solana error"));

    // Test with different solana signer errors
    let solana_error = solana_sdk::signer::SignerError::NotEnoughSigners;
    let signer_error: SignerError = solana_error.into();
    assert!(matches!(signer_error, SignerError::Signing(_)));
    assert!(signer_error.to_string().contains("Signing error"));

    let solana_error = solana_sdk::signer::SignerError::UserCancel("cancelled".to_string());
    let signer_error: SignerError = solana_error.into();
    assert!(matches!(signer_error, SignerError::Signing(_)));
    assert!(signer_error.to_string().contains("Signing error"));
}

#[test]
fn test_signer_error_debug_trait() {
    // Test Debug trait implementation
    let e = SignerError::NoSignerContext;
    let debug_str = format!("{:?}", e);
    assert!(debug_str.contains("NoSignerContext"));

    let e = SignerError::Configuration("test".to_string());
    let debug_str = format!("{:?}", e);
    assert!(debug_str.contains("Configuration"));
    assert!(debug_str.contains("test"));

    let e = SignerError::InvalidPrivateKey("bad key".to_string());
    let debug_str = format!("{:?}", e);
    assert!(debug_str.contains("InvalidPrivateKey"));
    assert!(debug_str.contains("bad key"));
}
