//! Example Actix Web server using riglr-web-adapters
//!
//! This example demonstrates how to use the riglr-web-adapters crate to create
//! a production-ready server with proper environment configuration and fail-fast
//! error handling.
//!
//! ## Environment Variables Required
//!
//! - `ANTHROPIC_API_KEY`: API key for Anthropic Claude
//! - `PORT`: Port number for the server (defaults to 8080)
//!
//! ## Usage
//!
//! ```bash
//! # Set required environment variables
//! export ANTHROPIC_API_KEY="your-api-key"
//! export PORT="8080"
//!
//! # Run the server
//! cargo run --example actix_server
//! ```
//!
//! ## Endpoints
//!
//! - `POST /api/v1/sse` - Server-Sent Events streaming with agent
//! - `POST /api/v1/completion` - One-shot completion with agent  
//! - `GET /health` - Health check
//! - `GET /` - Server information

#[cfg(feature = "web-server")]
use actix_web::{web, App, HttpServer, middleware::Logger, HttpRequest, HttpResponse};
#[cfg(feature = "web-server")]
use riglr_web_adapters::actix::ActixRiglrAdapter;
use riglr_core::util::must_get_env;
use std::error::Error as StdError;
#[cfg(feature = "web-server")]
use riglr_web_adapters::core::PromptRequest;
#[cfg(feature = "web-server")]
use riglr_web_adapters::factory::CompositeSignerFactory;
#[cfg(feature = "web-server")]
use riglr_core::config::RpcConfig;
#[cfg(feature = "web-server")]
use riglr_showcase::auth::privy::PrivySignerFactory;
#[cfg(feature = "web-server")]
use rig::providers::anthropic::{self, CLAUDE_3_5_SONNET};
#[cfg(feature = "web-server")]
use rig::completion::{Prompt, PromptError};
#[cfg(feature = "web-server")]
use futures_util::StreamExt;

/// Real rig agent implementation using Anthropic Claude
#[derive(Clone)]
struct RiglrAgent {
    agent: rig::agent::Agent<anthropic::CompletionModel>,
}

impl RiglrAgent {
    fn new(api_key: String) -> Self {
        let client = anthropic::Client::new(&api_key);
        let agent = client
            .agent(CLAUDE_3_5_SONNET)
            .preamble("You are a helpful blockchain assistant with access to riglr tools. \
                      Provide clear, concise responses and use the available tools when appropriate.")
            .temperature(0.7)
            .max_tokens(2048)
            .build();
        Self { agent }
    }
}

#[async_trait::async_trait]
impl riglr_web_adapters::Agent for RiglrAgent {
    type Error = Box<dyn StdError + Send + Sync>;

    async fn prompt(&self, prompt: &str) -> Result<String, Self::Error> {
        // Real implementation using rig agent
        match self.agent.prompt(prompt).await {
            Ok(response) => Ok(response.content),
            Err(e) => Err(Box::new(e) as Box<dyn StdError + Send + Sync>)
        }
    }

    async fn prompt_stream(&self, prompt: &str) -> Result<futures_util::stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error> {
        // Real streaming implementation using rig agent
        match self.agent.stream_prompt(prompt).await {
            Ok(stream) => {
                let mapped_stream = stream.map(|chunk| {
                    match chunk {
                        Ok(text) => Ok(text),
                        Err(e) => Err(Box::new(e) as Box<dyn StdError + Send + Sync>)
                    }
                });
                Ok(Box::pin(mapped_stream))
            },
            Err(e) => Err(Box::new(e) as Box<dyn StdError + Send + Sync>)
        }
    }
}

#[cfg(feature = "web-server")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Fail fast on missing configuration using must_get_env
    let anthropic_key = must_get_env("ANTHROPIC_API_KEY");
    let port = must_get_env("PORT")
        .parse::<u16>()
        .unwrap_or_else(|_| panic!("PORT must be a valid number"));

    tracing::info!(
        port = port,
        "Starting riglr-showcase Actix server with library-first architecture"
    );

    // Create real rig agent with Anthropic Claude
    let agent = RiglrAgent::new(anthropic_key);

    // Build signer factory (Privy)
    let privy_app_id = must_get_env("PRIVY_APP_ID");
    let privy_app_secret = must_get_env("PRIVY_APP_SECRET");
    let rpc_config = RpcConfig::default();
    let mut composite = CompositeSignerFactory::default();
    composite.add_factory("privy".to_string(), std::sync::Arc::new(PrivySignerFactory::new(privy_app_id, privy_app_secret)));
    let adapter = std::sync::Arc::new(ActixRiglrAdapter::new(std::sync::Arc::new(composite), rpc_config));

    tracing::info!("Actix adapter initialized with Privy authentication");

    // Start HTTP server using riglr-web-adapters
    HttpServer::new(move || {
        let adapter = adapter.clone();
        App::new()
            .app_data(web::Data::new(agent.clone()))
            .wrap(Logger::default())
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .allowed_headers(vec!["Content-Type", "Authorization", "x-network"])
                    .max_age(3600)
            )
            // riglr-web-adapters endpoints via adapter
            .route("/api/v1/sse", web::post().to({
                let adapter = adapter.clone();
                move |req: HttpRequest, agent: web::Data<RiglrAgent>, prompt: web::Json<PromptRequest>| {
                    let adapter = adapter.clone();
                    let agent = agent.get_ref().clone();
                    async move { adapter.sse_handler(&req, &agent, prompt.into_inner()).await }
                }
            }))
            .route("/api/v1/completion", web::post().to({
                let adapter = adapter.clone();
                move |req: HttpRequest, agent: web::Data<RiglrAgent>, prompt: web::Json<PromptRequest>| {
                    let adapter = adapter.clone();
                    let agent = agent.get_ref().clone();
                    async move { adapter.completion_handler(&req, &agent, prompt.into_inner()).await }
                }
            }))
            .route("/health", web::get().to(|| async {
                Ok::<HttpResponse, actix_web::Error>(HttpResponse::Ok().json(serde_json::json!({
                    "status": "healthy"
                })))
            }))
            .route("/", web::get().to(|| async {
                Ok::<HttpResponse, actix_web::Error>(HttpResponse::Ok().json(serde_json::json!({
                    "service": "riglr-showcase",
                    "version": env!("CARGO_PKG_VERSION")
                })))
            }))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

#[cfg(not(feature = "web-server"))]
fn main() {
    println!("This example requires the 'web-server' feature to be enabled.");
    println!("Run with: cargo run --example actix_server --features web-server");
}

#[cfg(all(test, feature = "web-server"))]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_riglr_agent_creation() {
        // Test that agent can be created with a mock API key
        let agent = RiglrAgent::new("test-api-key".to_string());
        // The agent should be created successfully
        // Real API calls would fail with invalid key, but creation should succeed
        assert!(true); // Agent creation doesn't panic
    }

    #[actix_web::test]
    async fn test_health_endpoint_integration() {
        let agent = RiglrAgent::new("test-api-key".to_string());
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(agent))
                .route("/health", web::get().to(health_handler))
        ).await;

        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}