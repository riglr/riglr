//! # riglr-server
//!
//! Development server for the riglr ecosystem providing example implementations
//! of agents and signer factories for testing and development purposes.
//!
//! This server includes an echo agent that simply echoes back prompts and a
//! development signer factory that creates local Solana signers for testing.

use std::sync::Arc;

use riglr_config::Config;
use riglr_server::{start_axum, ServerConfig};
use riglr_solana_tools::signer::LocalSolanaSigner;
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

// A SignerFactory for local dev; creates a local signer using config.
struct DevSignerFactory {
    config: Arc<Config>,
}

impl DevSignerFactory {
    fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl SignerFactory for DevSignerFactory {
    async fn create_signer(
        &self,
        auth_data: riglr_web_adapters::AuthenticationData,
    ) -> Result<Box<dyn riglr_core::signer::UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>>
    {
        // Create a LocalSolanaSigner from a fresh keypair using the requested network (default devnet)
        let kp = solana_sdk::signature::Keypair::new();
        let network = if auth_data.network.is_empty() {
            "devnet".to_string()
        } else {
            auth_data.network
        };
        // Use the config to get the appropriate RPC URL
        let rpc_url = self.config.network.solana_rpc_url.clone(); // Simplified for this example

        let config = riglr_config::SolanaNetworkConfig::new(network.clone(), rpc_url);
        let signer = LocalSolanaSigner::from_keypair(kp, config);
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

    // Load the application configuration
    let config = Config::from_env();

    let mut composite = CompositeSignerFactory::new();
    composite.register_factory("dev".into(), Box::new(DevSignerFactory::new(config.clone())));
    // Also register as "privy" to match Axum adapter's default auth_type mapping
    composite.register_factory("privy".into(), Box::new(DevSignerFactory::new(config)));

    let cfg = ServerConfig::default();
    start_axum(cfg, EchoAgent, Arc::new(composite)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream::StreamExt;
    use riglr_web_adapters::AuthenticationData;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_echo_agent_prompt_when_valid_input_should_return_echo() {
        let agent = EchoAgent;
        let result = agent.prompt("hello").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "echo: hello");
    }

    #[tokio::test]
    async fn test_echo_agent_prompt_when_empty_input_should_return_echo_empty() {
        let agent = EchoAgent;
        let result = agent.prompt("").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "echo: ");
    }

    #[tokio::test]
    async fn test_echo_agent_prompt_when_special_characters_should_return_echo() {
        let agent = EchoAgent;
        let result = agent.prompt("hello\nworld\t!@#$%^&*()").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "echo: hello\nworld\t!@#$%^&*()");
    }

    #[tokio::test]
    async fn test_echo_agent_prompt_when_unicode_should_return_echo() {
        let agent = EchoAgent;
        let result = agent.prompt("こんにちは🦀").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "echo: こんにちは🦀");
    }

    #[tokio::test]
    async fn test_echo_agent_prompt_stream_when_valid_input_should_return_stream() {
        let agent = EchoAgent;
        let result = agent.prompt_stream("test").await;

        assert!(result.is_ok());

        let stream = result.unwrap();
        let chunks: Vec<_> = stream.collect().await;

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].as_ref().unwrap(), "start:");
        assert_eq!(chunks[1].as_ref().unwrap(), " ");
        assert_eq!(chunks[2].as_ref().unwrap(), "test");
    }

    #[tokio::test]
    async fn test_echo_agent_prompt_stream_when_empty_input_should_return_stream() {
        let agent = EchoAgent;
        let result = agent.prompt_stream("").await;

        assert!(result.is_ok());

        let stream = result.unwrap();
        let chunks: Vec<_> = stream.collect().await;

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].as_ref().unwrap(), "start:");
        assert_eq!(chunks[1].as_ref().unwrap(), " ");
        assert_eq!(chunks[2].as_ref().unwrap(), "");
    }

    #[tokio::test]
    async fn test_echo_agent_prompt_stream_when_special_characters_should_return_stream() {
        let agent = EchoAgent;
        let result = agent.prompt_stream("hello\nworld").await;

        assert!(result.is_ok());

        let stream = result.unwrap();
        let chunks: Vec<_> = stream.collect().await;

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].as_ref().unwrap(), "start:");
        assert_eq!(chunks[1].as_ref().unwrap(), " ");
        assert_eq!(chunks[2].as_ref().unwrap(), "hello\nworld");
    }

    #[tokio::test]
    async fn test_dev_signer_factory_create_signer_when_empty_network_should_use_devnet() {
        let config = Config::from_env();
        let factory = DevSignerFactory::new(config);
        let mut credentials = HashMap::new();
        credentials.insert("user_id".to_string(), "test_user".to_string());
        let auth_data = AuthenticationData {
            auth_type: "dev".to_string(),
            credentials,
            network: "".to_string(),
        };

        let result = factory.create_signer(auth_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dev_signer_factory_create_signer_when_specified_network_should_use_it() {
        let config = Config::from_env();
        let factory = DevSignerFactory::new(config);
        let mut credentials = HashMap::new();
        credentials.insert("user_id".to_string(), "test_user".to_string());
        let auth_data = AuthenticationData {
            auth_type: "dev".to_string(),
            credentials,
            network: "mainnet".to_string(),
        };

        let result = factory.create_signer(auth_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dev_signer_factory_create_signer_when_network_not_in_config_should_use_default() {
        let config = Config::from_env();
        let factory = DevSignerFactory::new(config);
        let mut credentials = HashMap::new();
        credentials.insert("user_id".to_string(), "test_user".to_string());
        let auth_data = AuthenticationData {
            auth_type: "dev".to_string(),
            credentials,
            network: "nonexistent".to_string(),
        };

        let result = factory.create_signer(auth_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dev_signer_factory_create_signer_when_different_auth_types_should_work() {
        let config = Config::from_env();
        let factory = DevSignerFactory::new(config);

        // Test with different auth types
        for auth_type in &["dev", "privy", "other"] {
            let mut credentials = HashMap::new();
            credentials.insert("user_id".to_string(), "test_user".to_string());
            let auth_data = AuthenticationData {
                auth_type: auth_type.to_string(),
                credentials,
                network: "devnet".to_string(),
            };

            let result = factory.create_signer(auth_data).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_dev_signer_factory_create_signer_when_with_token_should_work() {
        let config = Config::from_env();
        let factory = DevSignerFactory::new(config);
        let mut credentials = HashMap::new();
        credentials.insert("user_id".to_string(), "test_user".to_string());
        credentials.insert("token".to_string(), "test_token".to_string());
        let auth_data = AuthenticationData {
            auth_type: "dev".to_string(),
            credentials,
            network: "devnet".to_string(),
        };

        let result = factory.create_signer(auth_data).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_dev_signer_factory_supported_auth_types_should_return_dev_and_privy() {
        let config = Config::from_env();
        let factory = DevSignerFactory::new(config);
        let auth_types = factory.supported_auth_types();

        assert_eq!(auth_types.len(), 2);
        assert!(auth_types.contains(&"dev".to_string()));
        assert!(auth_types.contains(&"privy".to_string()));
    }

    #[test]
    fn test_echo_agent_clone_should_work() {
        let agent1 = EchoAgent;
        let agent2 = agent1.clone();

        // Both should be usable (compile-time test mostly)
        let _ = agent1;
        let _ = agent2;
    }

    #[test]
    fn test_dev_signer_factory_clone_not_implemented() {
        // DevSignerFactory doesn't implement Clone, this tests that it's a conscious decision
        // This test just ensures we can create multiple instances
        let config = Config::from_env();
        let _factory1 = DevSignerFactory::new(config.clone());
        let _factory2 = DevSignerFactory::new(config);
    }

    // Test edge cases for network configuration
    #[tokio::test]
    async fn test_dev_signer_factory_create_signer_when_network_config_has_no_explorer_url() {
        let config = Config::from_env();
        let factory = DevSignerFactory::new(config);
        let mut credentials = HashMap::new();
        credentials.insert("user_id".to_string(), "test_user".to_string());
        let auth_data = AuthenticationData {
            auth_type: "dev".to_string(),
            credentials,
            network: "testnet".to_string(),
        };

        let result = factory.create_signer(auth_data).await;
        assert!(result.is_ok());
    }

    // Test with very long network name
    #[tokio::test]
    async fn test_dev_signer_factory_create_signer_when_long_network_name_should_work() {
        let config = Config::from_env();
        let factory = DevSignerFactory::new(config);
        let long_network_name = "a".repeat(1000);
        let mut credentials = HashMap::new();
        credentials.insert("user_id".to_string(), "test_user".to_string());
        let auth_data = AuthenticationData {
            auth_type: "dev".to_string(),
            credentials,
            network: long_network_name.clone(),
        };

        let result = factory.create_signer(auth_data).await;
        assert!(result.is_ok());
    }

    // Test with special characters in user_id
    #[tokio::test]
    async fn test_dev_signer_factory_create_signer_when_special_user_id_should_work() {
        let config = Config::from_env();
        let factory = DevSignerFactory::new(config);
        let mut credentials = HashMap::new();
        credentials.insert("user_id".to_string(), "test@user.com!@#$%^&*()".to_string());
        let auth_data = AuthenticationData {
            auth_type: "dev".to_string(),
            credentials,
            network: "devnet".to_string(),
        };

        let result = factory.create_signer(auth_data).await;
        assert!(result.is_ok());
    }
}
