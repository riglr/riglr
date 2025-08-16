use thiserror::Error;

#[derive(Error, Debug)]
pub enum SignerError {
    #[error("No signer context available - must be called within SignerContext::with_signer")]
    NoSignerContext,

    #[error("Solana transaction error: {0}")]
    SolanaTransaction(#[from] Box<solana_client::client_error::ClientError>),

    #[error("EVM transaction error: {0}")]
    EvmTransaction(String),

    #[error("Invalid configuration: {0}")]
    Configuration(String),

    #[error("Signing error: {0}")]
    Signing(String),

    #[error("Client creation error: {0}")]
    ClientCreation(String),

    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Blockhash error: {0}")]
    BlockhashError(String),

    #[error("Unsupported network: {0}")]
    UnsupportedNetwork(String),

    #[error("Invalid RPC URL: {0}")]
    InvalidRpcUrl(String),
}

impl From<solana_sdk::signer::SignerError> for SignerError {
    fn from(err: solana_sdk::signer::SignerError) -> Self {
        SignerError::Signing(err.to_string())
    }
}
