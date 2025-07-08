//! Core server infrastructure for serving rig agents over HTTP

use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use std::sync::Arc;
use crate::{handlers, Result};

/// Configuration for creating a signer from request data
/// This will be extended once SignerContext is implemented
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct SignerConfig {
    /// Solana RPC URL
    pub solana_rpc_url: Option<String>,
    /// EVM RPC URL  
    pub evm_rpc_url: Option<String>,
    /// User identifier for multi-tenant scenarios
    pub user_id: Option<String>,
    /// Locale for localized responses
    pub locale: Option<String>,
}

impl Default for SignerConfig {
    fn default() -> Self {
        Self {
            solana_rpc_url: Some("https://api.devnet.solana.com".to_string()),
            evm_rpc_url: Some("https://eth.public-rpc.com".to_string()),
            user_id: None,
            locale: Some("en".to_string()),
        }
    }
}

/// Generic HTTP server that can serve any rig agent
pub struct AgentServer<A> {
    agent: Arc<A>,
    bind_address: String,
    cors_origins: Vec<String>,
    enable_logging: bool,
}

impl<A> AgentServer<A>
where
    A: Clone + Send + Sync + 'static,
{
    /// Create a new agent server
    pub fn new(agent: A, bind_address: String) -> Self {
        Self {
            agent: Arc::new(agent),
            bind_address,
            cors_origins: vec!["*".to_string()],
            enable_logging: true,
        }
    }
    
    /// Configure CORS origins (default: ["*"])
    pub fn cors_origins(mut self, origins: Vec<String>) -> Self {
        self.cors_origins = origins;
        self
    }
    
    /// Enable or disable request logging (default: true)
    pub fn enable_logging(mut self, enable: bool) -> Self {
        self.enable_logging = enable;
        self
    }
    
    /// Start the HTTP server
    pub async fn run(self) -> Result<()> {
        tracing::info!("Starting riglr-server on {}", self.bind_address);
        
        let agent_data = web::Data::new(self.agent);
        
        let mut server = HttpServer::new(move || {
            let mut cors = Cors::default();
            
            // Configure CORS
            if self.cors_origins.contains(&"*".to_string()) {
                cors = cors.allow_any_origin();
            } else {
                for origin in &self.cors_origins {
                    cors = cors.allowed_origin(origin);
                }
            }
            
            cors = cors
                .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                .allowed_headers(vec!["Content-Type", "Authorization"])
                .max_age(3600);
            
            App::new()
                .app_data(agent_data.clone())
                .wrap(cors)
                .wrap(Logger::default())
                .route("/chat", web::post().to(handlers::chat::<A>))
                .route("/stream", web::get().to(handlers::stream::<A>))
                .route("/health", web::get().to(handlers::health))
                .route("/", web::get().to(handlers::index))
        });
        
        server = server.bind(&self.bind_address)
            .map_err(|e| crate::error::ServerError::Bind(e))?;
        
        tracing::info!("Server running on http://{}", self.bind_address);
        tracing::info!("Endpoints:");
        tracing::info!("  GET  /         - Server information");  
        tracing::info!("  POST /chat     - Chat with agent");
        tracing::info!("  GET  /stream   - Server-Sent Events streaming");
        tracing::info!("  GET  /health   - Health check");
        
        server.run().await.map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock agent for testing
    #[derive(Clone)]
    struct MockAgent {
        name: String,
    }
    
    impl MockAgent {
        fn new(name: String) -> Self {
            Self { name }
        }
    }
    
    #[tokio::test]
    async fn test_agent_server_creation() {
        let agent = MockAgent::new("test-agent".to_string());
        let server = AgentServer::new(agent, "127.0.0.1:0".to_string());
        
        // Test that server can be created and configured
        let server = server
            .cors_origins(vec!["http://localhost:3000".to_string()])
            .enable_logging(false);
        
        // Verify configuration was applied
        assert_eq!(server.cors_origins, vec!["http://localhost:3000"]);
        assert!(!server.enable_logging);
    }
    
    #[tokio::test]
    async fn test_signer_config_default() {
        let config = SignerConfig::default();
        assert!(config.solana_rpc_url.is_some());
        assert!(config.evm_rpc_url.is_some());
        assert_eq!(config.locale, Some("en".to_string()));
        assert!(config.user_id.is_none());
    }
}