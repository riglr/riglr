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