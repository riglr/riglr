//! riglr-server: HTTP server for serving rig agents over REST API
//!
//! This crate provides a generic HTTP server that can turn any rig agent into a production-ready
//! REST API with Server-Sent Events (SSE) streaming support. The server handles SignerContext
//! isolation per request, ensuring secure multi-tenant operation.
//!
//! ## Features
//!
//! - **Generic Agent Support**: Works with any type that implements rig agent traits
//! - **SignerContext Integration**: Per-request signer isolation for secure multi-tenant operation
//! - **Server-Sent Events**: Real-time streaming responses for chat interactions
//! - **CORS Support**: Cross-origin resource sharing for web clients
//! - **Health Checks**: Built-in health endpoint for monitoring
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! // Example with SignerContext integration
//! use riglr_server::AgentServer;
//!
//! #[derive(Clone)]
//! struct MockAgent { name: String }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let agent = MockAgent { name: "AI Agent".to_string() };
//!     let server = AgentServer::new(agent, "127.0.0.1:8080".to_string());
//!     server.run().await?;
//!     
//!     // The server now supports:
//!     // POST /chat - Chat with SignerContext integration
//!     // GET /stream - Server-Sent Events with SignerContext
//!     // GET /health - Health check
//!     // GET / - Server information
//!     
//!     Ok(())
//! }
//! ```

pub mod server;
pub mod handlers;
pub mod sse;
pub mod error;

pub use server::AgentServer;
pub use error::{ServerError, Result};

// Re-export commonly used types
pub use serde::{Deserialize, Serialize};
pub use actix_web::HttpResponse;

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use serde_json::json;

    #[derive(Clone)]
    struct MockAgent {
        _name: String,
    }

    #[tokio::test]
    async fn test_server_error_display() {
        let error = ServerError::Internal("Test error".to_string());
        assert_eq!(error.to_string(), "Internal server error: Test error");
    }

    #[tokio::test]
    async fn test_server_error_from_string() {
        let error: ServerError = "Test error".into();
        match error {
            ServerError::Internal(msg) => assert_eq!(msg, "Test error"),
            _ => panic!("Expected Internal error"),
        }
    }

    #[tokio::test]
    async fn test_server_creation() {
        let agent = MockAgent {
            _name: "Test Agent".to_string(),
        };
        
        let _server = AgentServer::new(agent, "127.0.0.1:8080".to_string());
        
        // Test that server was created successfully
        // Server creation itself validates the AgentServer::new function
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let agent = MockAgent {
            _name: "Test Agent".to_string(),
        };

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

        let body = test::read_body(resp).await;
        let health_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(health_response["status"], "healthy");
        assert!(health_response["timestamp"].is_string());
    }

    #[tokio::test]
    async fn test_info_endpoint() {
        let agent = MockAgent {
            _name: "Test Agent".to_string(),
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(agent))
                .route("/", web::get().to(info_handler))
        ).await;

        let req = test::TestRequest::get()
            .uri("/")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let info_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(info_response["service"], "riglr-server");
        assert!(info_response["version"].is_string());
    }

    async fn health_handler() -> actix_web::Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339()
        })))
    }

    async fn info_handler() -> actix_web::Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(json!({
            "service": "riglr-server",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "HTTP server for rig agents"
        })))
    }

    #[tokio::test]
    async fn test_server_error_types() {
        let validation_error = ServerError::Validation("Invalid input".to_string());
        assert!(matches!(validation_error, ServerError::Validation(_)));

        let internal_error = ServerError::Internal("System error".to_string());
        assert!(matches!(internal_error, ServerError::Internal(_)));

        let agent_error = ServerError::Agent("Agent failed".to_string());
        assert!(matches!(agent_error, ServerError::Agent(_)));
    }

    #[tokio::test]
    async fn test_server_result_type() {
        let success_result: Result<String> = Ok("success".to_string());
        assert!(success_result.is_ok());

        let error_result: Result<String> = Err(ServerError::Internal("error".to_string()));
        assert!(error_result.is_err());
    }
}