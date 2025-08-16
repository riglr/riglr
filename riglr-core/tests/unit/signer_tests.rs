use std::sync::Arc;
use riglr_core::signer::{SignerError, SignerContext, TransactionSigner, LocalEvmSigner, LocalSolanaSigner};
use riglr_core::config::{RpcConfig, EvmNetworkConfig, SolanaNetworkConfig};
use alloy::rpc::types::TransactionRequest;
use solana_sdk::signature::Keypair;

#[tokio::test]
async fn signer_context_availability() {
    // Outside context
    assert!(!SignerContext::is_available().await);

    // Minimal mock signer implementing TransactionSigner
    #[derive(Debug)]
    struct Dummy;
    #[async_trait::async_trait]
    impl TransactionSigner for Dummy {
        async fn sign_and_send_solana_transaction(
            &self,
            _tx: &mut solana_sdk::transaction::Transaction,
        ) -> Result<String, SignerError> { Ok("sig".into()) }
        async fn sign_and_send_evm_transaction(
            &self,
            _tx: TransactionRequest,
        ) -> Result<String, SignerError> { Ok("0xhash".into()) }
        fn solana_client(&self) -> Option<Arc<solana_client::rpc_client::RpcClient>> { None }
        fn evm_client(&self) -> Result<Arc<dyn riglr_core::signer::EvmClient>, SignerError> {
            Err(SignerError::UnsupportedOperation("no evm".into()))
        }
    }

    let signer = Arc::new(Dummy);
    let inside = SignerContext::with_signer(signer, async {
        assert!(SignerContext::is_available().await);
        let _ = SignerContext::current().await.unwrap();
        Ok(())
    }).await;

    assert!(inside.is_ok());
    assert!(!SignerContext::is_available().await);
}

#[test]
fn signer_error_display_variants() {
    let e = SignerError::UnsupportedOperation("x".into());
    assert!(e.to_string().contains("Unsupported operation"));
    let ee: SignerError = solana_sdk::signer::SignerError::Custom("oops".into()).into();
    assert!(ee.to_string().contains("Signing error"));
}

#[test]
fn evm_signer_from_config_unknown_network() {
    let cfg = RpcConfig::default();
    let res = LocalEvmSigner::from_config("0x12".into(), &cfg, "does-not-exist");
    assert!(matches!(res, Err(SignerError::UnsupportedNetwork(_))));
}

#[test]
fn solana_signer_from_config_unknown_network() {
    let cfg = RpcConfig::default();
    let res = LocalSolanaSigner::from_config("priv".into(), &cfg, "does-not-exist");
    assert!(matches!(res, Err(SignerError::UnsupportedNetwork(_))));
}

#[test]
fn evm_signer_invalid_private_key() {
    // clearly invalid key
    let net = EvmNetworkConfig { name: "x".into(), chain_id: 1, rpc_url: "http://localhost".into(), explorer_url: None };
    let res = LocalEvmSigner::new("not_a_key".into(), net);
    assert!(matches!(res, Err(SignerError::InvalidPrivateKey(_))));
}

#[test]
fn solana_signer_new_from_keypair() {
    let kp = Keypair::new();
    let net = SolanaNetworkConfig { name: "devnet".into(), rpc_url: "https://api.devnet.solana.com".into(), explorer_url: None };
    let signer = riglr_core::signer::solana::LocalSolanaSigner::from_keypair(kp, net);
    // basic sanity
    assert!(signer.pubkey().is_some());
}
