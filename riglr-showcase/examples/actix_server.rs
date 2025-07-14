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
use actix_web::{web, App, HttpServer, middleware::Logger};
#[cfg(feature = "web-server")]
use riglr_web_adapters::actix::{sse_handler, completion_handler, health_handler, info_handler};
use riglr_core::util::must_get_env;
use std::error::Error as StdError;

/// Mock agent implementation for demonstration
/// In production, this would be replaced with a real rig agent
#[derive(Clone)]
struct MockAgent {
    name: String,
}

impl MockAgent {
    fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait::async_trait]
impl riglr_web_adapters::Agent for MockAgent {
    type Error = Box<dyn StdError + Send + Sync>;

    async fn prompt(&self, prompt: &str) -> Result<String, Self::Error> {
        // Mock implementation for demonstration
        Ok(format!(
            "Mock Agent '{}' received: '{}'. This is a demonstration response showing the library-first approach. \
             In production, this would use real Claude API integration.",
            self.name, prompt
        ))
    }

    async fn prompt_stream(&self, prompt: &str) -> Result<futures_util::stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error> {
        // Mock streaming implementation
        let response_chunks = vec![
            format!("Mock Agent '{}' received: '{}'", self.name, prompt),
            " This is a demonstration of streaming responses.".to_string(),
            " The library-first approach allows easy adaptation to any web framework.".to_string(),
            " Real implementation would stream from Claude API.".to_string(),
        ];
        
        let stream = futures_util::stream::iter(response_chunks)
            .map(|chunk| Ok(chunk));
        
        Ok(Box::pin(stream))
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

    // Create mock agent for demonstration
    // In production, this would be:
    // let client = Client::new(&anthropic_key);
    // let agent = client.agent(CLAUDE_3_5_SONNET)
    //     .preamble("You are a helpful blockchain assistant with access to riglr tools.")
    //     .build();
    let agent = MockAgent::new("riglr-demo".to_string());

    tracing::info!("Mock agent created for demonstration");

    // Start HTTP server using riglr-web-adapters
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(agent.clone()))
            .wrap(Logger::default())
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .allowed_headers(vec!["Content-Type", "Authorization"])
                    .max_age(3600)
            )
            // riglr-web-adapters endpoints
            .route("/api/v1/sse", web::post().to(sse_handler))
            .route("/api/v1/completion", web::post().to(completion_handler))
            .route("/health", web::get().to(health_handler))
            .route("/", web::get().to(info_handler))
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
    async fn test_mock_agent_prompt() {
        let agent = MockAgent::new("test".to_string());
        let response = agent.prompt("Hello").await.unwrap();
        assert!(response.contains("Mock Agent 'test' received: 'Hello'"));
    }

    #[actix_web::test]
    async fn test_health_endpoint_integration() {
        let agent = MockAgent::new("test".to_string());
        
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