//! API server for the indexer service

use axum::Router;
use std::sync::Arc;

use crate::config::ApiConfig;
use crate::core::ServiceContext;
use crate::error::{IndexerError, IndexerResult};

pub mod rest;
pub mod websocket;

pub use rest::RestHandler;
pub use websocket::WebSocketStreamer;

/// Main API server
pub struct ApiServer {
    /// Service context
    context: Arc<ServiceContext>,
    /// Configuration
    config: ApiConfig,
    /// REST handler
    rest_handler: RestHandler,
    /// WebSocket streamer
    websocket_streamer: Option<WebSocketStreamer>,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(context: Arc<ServiceContext>) -> IndexerResult<Self> {
        let config = context.config.api.clone();
        let rest_handler = RestHandler::new(context.clone())?;

        let websocket_streamer = if config.websocket.enabled {
            Some(WebSocketStreamer::new(context.clone())?)
        } else {
            None
        };

        Ok(Self {
            context,
            config,
            rest_handler,
            websocket_streamer,
        })
    }

    /// Create the router for the API server
    pub fn create_router(&self) -> Router<Arc<ServiceContext>> {
        let mut router = self.rest_handler.create_router();

        if let Some(ws_streamer) = &self.websocket_streamer {
            router = router.merge(ws_streamer.create_router());
        }

        router
    }

    /// Start the API server
    pub async fn start(&self) -> IndexerResult<()> {
        let addr = format!("{}:{}", self.config.http.bind, self.config.http.port);
        let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|_e| {
            IndexerError::Network(crate::error::NetworkError::HttpFailed {
                status: 0,
                url: addr.clone(),
            })
        })?;

        tracing::info!("API server listening on {}", addr);

        let app = self.create_router();

        axum::serve(listener, app.with_state(self.context.clone()))
            .await
            .map_err(|e| IndexerError::internal_with_source("API server failed", e))?;

        Ok(())
    }
}
