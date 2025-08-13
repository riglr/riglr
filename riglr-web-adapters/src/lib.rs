//! riglr-web-adapters: Framework-agnostic web adapters for riglr agents
//!
//! This crate provides a library-first approach to serving riglr agents over HTTP,
//! with framework-agnostic core handlers and specific adapters for different web frameworks.
//!
//! ## Design Philosophy
//!
//! - **Library-first**: Core logic is framework-agnostic and reusable
//! - **Framework adapters**: Thin wrappers around core handlers for specific frameworks
//! - **SignerContext integration**: Proper signer isolation per request
//! - **Type safety**: Leverages Rust's type system for correctness
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────┐
//! │   Your App      │
//! └─────────────────┘
//!          │
//! ┌─────────────────┐
//! │ Framework       │ ← actix.rs, axum.rs
//! │ Adapters        │
//! └─────────────────┘
//!          │
//! ┌─────────────────┐
//! │ Core Handlers   │ ← core.rs (framework-agnostic)
//! └─────────────────┘
//!          │
//! ┌─────────────────┐
//! │ riglr-core      │
//! └─────────────────┘
//! ```
//!
//! ## Usage
//!
//! ### With Actix Web
//!
//! ```rust,ignore
//! use riglr_web_adapters::actix::{sse_handler, completion_handler};
//! use actix_web::{web, App, HttpServer};
//! use rig::providers::anthropic::Client;
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     let client = Client::from_env();
//!     let agent = client.agent("claude-3-5-sonnet").build();
//!     
//!     HttpServer::new(move || {
//!         App::new()
//!             .app_data(web::Data::new(agent.clone()))
//!             .route("/api/v1/sse", web::post().to(sse_handler))
//!             .route("/api/v1/completion", web::post().to(completion_handler))
//!     })
//!     .bind("0.0.0.0:8080")?
//!     .run()
//!     .await
//! }
//! ```
//!
//! ### With Axum
//!
//! ```rust,ignore
//! use riglr_web_adapters::axum::{sse_handler, completion_handler};
//! use axum::{routing::post, Router};
//! use rig::providers::anthropic::Client;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Client::from_env();
//!     let agent = client.agent("claude-3-5-sonnet").build();
//!     
//!     let app = Router::new()
//!         .route("/api/v1/sse", post(sse_handler))
//!         .route("/api/v1/completion", post(completion_handler))
//!         .with_state(agent);
//!         
//!     axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
//!         .serve(app.into_make_service())
//!         .await
//!         .unwrap();
//! }
//! ```

pub mod core;
pub mod factory;

#[cfg(feature = "actix")]
pub mod actix;

#[cfg(feature = "axum")]
pub mod axum;

// Re-export commonly used types
pub use core::{Agent, AgentStream, PromptRequest, CompletionResponse};

// Re-export factory traits for easy use
pub use factory::{SignerFactory, AuthenticationData, CompositeSignerFactory};

// Re-export new adapter types for easy use
#[cfg(feature = "actix")]
pub use actix::ActixRiglrAdapter;

#[cfg(feature = "axum")]
pub use axum::AxumRiglrAdapter;

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_library_compiles() {
        // Basic compilation test to ensure the library structure is valid
        assert!(true);
    }
}