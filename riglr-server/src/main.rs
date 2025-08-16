use std::sync::Arc;

use riglr_server::{start_axum, ServerConfig};
use riglr_web_adapters::{Agent, CompositeSignerFactory, SignerFactory};

#[derive(Clone)]
struct EchoAgent;

#[async_trait::async_trait]
impl Agent for EchoAgent {
    type Error = std::io::Error;

    async fn prompt(&self, prompt: &str) -> Result<String, Self::Error> {
        Ok(format!("echo: {}", prompt))
    }

    async fn prompt_stream(
        &self,
        prompt: &str,
    ) -> Result<futures_util::stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error> {
        let chunks = vec![
            Ok("start:".to_string()),
            Ok(" ".to_string()),
            Ok(prompt.to_string()),
        ];
        Ok(Box::pin(futures_util::stream::iter(chunks)))
    }
}

// A no-op SignerFactory useful for local dev; always returns a local signer when possible.
struct DevSignerFactory;

#[async_trait::async_trait]
impl SignerFactory for DevSignerFactory {
    async fn create_signer(
        &self,
        auth_data: riglr_web_adapters::AuthenticationData,
        config: &riglr_core::config::RpcConfig,
    ) -> Result<
        Box<dyn riglr_core::signer::TransactionSigner>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        // Create a LocalSolanaSigner from a fresh keypair using the requested network (default devnet)
        let kp = solana_sdk::signature::Keypair::new();
        let network = if auth_data.network.is_empty() {
            "devnet".to_string()
        } else {
            auth_data.network
        };
        let net_cfg = config
            .solana_networks
            .get(&network)
            .cloned()
            .unwrap_or_else(|| riglr_core::config::SolanaNetworkConfig {
                name: "Solana Devnet".to_string(),
                rpc_url: "https://api.devnet.solana.com".to_string(),
                explorer_url: Some("https://explorer.solana.com".to_string()),
            });

        let signer = riglr_core::signer::LocalSolanaSigner::from_keypair(kp, net_cfg);
        Ok(Box::new(signer))
    }

    fn supported_auth_types(&self) -> Vec<String> {
        vec!["dev".into(), "privy".into()]
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // init tracing from env RUST_LOG=info
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let mut composite = CompositeSignerFactory::new();
    composite.register_factory("dev".into(), Box::new(DevSignerFactory));
    // Also register as "privy" to match Axum adapter's default auth_type mapping
    composite.register_factory("privy".into(), Box::new(DevSignerFactory));

    let cfg = ServerConfig::default();
    start_axum(cfg, EchoAgent, Arc::new(composite)).await
}
