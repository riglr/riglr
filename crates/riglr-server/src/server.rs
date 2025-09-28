use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{sse::Event, Sse},
    routing::{get, post},
    Error as AxumError, Json, Router,
};
use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use core::sync::atomic::{AtomicU64, Ordering};
use futures_util::Stream;
use std::sync::Arc;
use tokio::net::TcpListener;

use riglr_web_adapters::{Agent, PromptRequest, SignerFactory};

/// Configuration for riglr server instances
#[derive(Clone, Debug)]
pub struct Config {
    /// Socket address for the server to bind to
    pub addr: SocketAddr,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 8080)),
        }
    }
}

/// Start an Axum server exposing riglr endpoints. Enabled with the `axum` feature.
///
/// # Errors
/// Returns an error if the server fails to bind to the specified address or encounters an error during startup.
#[cfg(feature = "axum")]
pub async fn start_axum<A: Agent + Clone + Send + Sync + 'static>(
    config: Config,
    agent: A,
    signer_factory: Arc<dyn SignerFactory>,
) -> anyhow::Result<()> {
    use riglr_web_adapters::axum as axum_adapters;

    #[derive(Clone)]
    struct AppState<A: Agent + Clone + Send + Sync + 'static> {
        adapter: axum_adapters::AxumRiglrAdapter,
        agent: A,
        completion_count: Arc<AtomicU64>,
        sse_count: Arc<AtomicU64>,
    }

    // Handlers using unified state
    async fn sse_handler<A: Agent + Clone + Send + Sync + 'static>(
        State(state): State<AppState<A>>,
        headers: HeaderMap,
        Json(prompt): Json<PromptRequest>,
    ) -> Result<Sse<impl Stream<Item = Result<Event, AxumError>>>, StatusCode> {
        state.sse_count.fetch_add(1, Ordering::Relaxed);
        return state
            .adapter
            .sse_handler(headers, state.agent.clone(), prompt)
            .await;
    }
    async fn completion_handler<A: Agent + Clone + Send + Sync + 'static>(
        State(state): State<AppState<A>>,
        headers: HeaderMap,
        Json(prompt): Json<PromptRequest>,
    ) -> Result<Json<riglr_web_adapters::CompletionResponse>, StatusCode> {
        state.completion_count.fetch_add(1, Ordering::Relaxed);
        return state
            .adapter
            .completion_handler(headers, state.agent.clone(), prompt)
            .await;
    }
    async fn health() -> Json<serde_json::Value> {
        axum_adapters::health_handler().await
    }
    async fn info() -> Json<serde_json::Value> {
        axum_adapters::info_handler().await
    }
    async fn metrics<A: Agent + Clone + Send + Sync + 'static>(
        State(state): State<AppState<A>>,
    ) -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "sse_requests": state.sse_count.load(Ordering::Relaxed),
            "completion_requests": state.completion_count.load(Ordering::Relaxed),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }

    // Wire adapter with provided factory
    let adapter = axum_adapters::AxumRiglrAdapter::new(signer_factory);

    let app = Router::new()
        .route("/v1/stream", post(sse_handler::<A>))
        .route("/v1/completion", post(completion_handler::<A>))
        .route("/health", get(health))
        .route("/metrics", get(metrics::<A>))
        .route("/", get(info))
        .with_state(AppState {
            adapter,
            agent,
            completion_count: Arc::new(AtomicU64::new(0)),
            sse_count: Arc::new(AtomicU64::new(0)),
        });

    tracing::info!(addr = %config.addr, "Starting riglr Axum server");
    let listener = TcpListener::bind(config.addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

/// Start an Actix server exposing riglr endpoints. Enable with the `actix` feature.
///
/// # Errors
/// Returns an error if the server fails to bind to the specified address or encounters an error during startup.
#[cfg(feature = "actix")]
#[allow(clippy::future_not_send)]
pub async fn start_actix<A: Agent + Clone + Send + Sync + 'static>(
    config: Config,
    agent: A,
    signer_factory: Arc<dyn SignerFactory>,
) -> anyhow::Result<()> {
    use actix_web::{web, App, HttpServer};
    use riglr_web_adapters::actix as actix_adapters;

    let adapter = actix_adapters::ActixRiglrAdapter::new(signer_factory);
    let adapter_data = web::Data::new(adapter);
    let agent_data = web::Data::new(agent);

    tracing::info!(addr = %config.addr, "Starting riglr Actix server");
    HttpServer::new(move || {
        App::new()
            .app_data(adapter_data.clone())
            .app_data(agent_data.clone())
            .route(
                "/v1/stream",
                web::post().to(
                    |adapter: web::Data<actix_adapters::ActixRiglrAdapter>,
                     req: actix_web::HttpRequest,
                     agent: web::Data<A>,
                     prompt: web::Json<PromptRequest>| async move {
                        adapter
                            .sse_handler(&req, agent.as_ref(), prompt.into_inner())
                            .await
                    },
                ),
            )
            .route(
                "/v1/completion",
                web::post().to(
                    |adapter: web::Data<actix_adapters::ActixRiglrAdapter>,
                     req: actix_web::HttpRequest,
                     agent: web::Data<A>,
                     prompt: web::Json<PromptRequest>| async move {
                        adapter
                            .completion_handler(&req, agent.as_ref(), prompt.into_inner())
                            .await
                    },
                ),
            )
            .route("/health", web::get().to(actix_adapters::health_handler))
            .route("/", web::get().to(actix_adapters::info_handler))
    })
    .bind(config.addr)?
    .run()
    .await?;

    Ok(())
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;
    use core::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_server_config_clone() {
        let config = Config::default();
        let cloned = config.clone();
        assert_eq!(config.addr, cloned.addr);
    }

    #[test]
    fn test_server_config_debug() {
        let config = Config::default();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("addr"));
        // Config only contains addr field, no rpc
    }

    #[test]
    fn test_server_config_default() {
        let config = Config::default();
        assert_eq!(config.addr.ip(), IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        assert_eq!(config.addr.port(), 8080);
        // RpcConfig::default() is tested in its own module
    }

    #[test]
    fn test_server_config_custom_addr() {
        let custom_addr: SocketAddr = "127.0.0.1:3000"
            .parse()
            .expect("custom address should parse");
        let config = Config { addr: custom_addr };
        assert_eq!(config.addr, custom_addr);
        assert_eq!(config.addr.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(config.addr.port(), 3000);
    }

    #[test]
    fn test_server_config_ipv6_addr() {
        let ipv6_addr: SocketAddr = "[::1]:8080".parse().expect("ipv6 address should parse");
        let config = Config { addr: ipv6_addr };
        assert_eq!(config.addr, ipv6_addr);
        assert_eq!(config.addr.port(), 8080);
        assert!(config.addr.is_ipv6());
    }

    #[test]
    fn test_server_config_zero_port() {
        let zero_port_addr: SocketAddr =
            "0.0.0.0:0".parse().expect("zero port address should parse");
        let config = Config {
            addr: zero_port_addr,
        };
        assert_eq!(config.addr.port(), 0);
    }

    #[test]
    fn test_server_config_max_port() {
        let max_port_addr: SocketAddr = "0.0.0.0:65535"
            .parse()
            .expect("max port address should parse");
        let config = Config {
            addr: max_port_addr,
        };
        assert_eq!(config.addr.port(), 65535);
    }

    // Test edge cases for address parsing that would be used in Config construction
    #[test]
    fn test_server_config_localhost() {
        let localhost_addr: SocketAddr = "127.0.0.1:8080"
            .parse()
            .expect("localhost IP address should parse");
        let config = Config {
            addr: localhost_addr,
        };
        assert_eq!(config.addr.port(), 8080);
    }

    #[test]
    fn test_server_config_equality() {
        let config1 = Config::default();
        let config2 = Config::default();

        // Test that two default configs have the same address
        assert_eq!(config1.addr, config2.addr);
    }

    #[test]
    fn test_server_config_different_addresses() {
        let config1 = Config {
            addr: "127.0.0.1:8080"
                .parse()
                .expect("default address should parse"),
        };
        let config2 = Config {
            addr: "127.0.0.1:8081"
                .parse()
                .expect("different address should parse"),
        };

        assert_ne!(config1.addr, config2.addr);
    }

    // Test that Config can be moved and used after move
    #[test]
    fn test_server_config_move() {
        let config = Config::default();
        let addr = config.addr;

        // Move config into a function that uses it
        let consume_config = |c: Config| c.addr;
        let returned_addr = consume_config(config);

        assert_eq!(addr, returned_addr);
    }

    // Test Config with various IP address types
    #[test]
    fn test_server_config_broadcast_addr() {
        let broadcast_addr: SocketAddr = "255.255.255.255:8080"
            .parse()
            .expect("broadcast address should parse");
        let config = Config {
            addr: broadcast_addr,
        };
        assert_eq!(config.addr.ip(), IpAddr::V4(Ipv4Addr::BROADCAST));
    }

    #[test]
    fn test_server_config_loopback() {
        let loopback_addr: SocketAddr = "127.0.0.1:8080"
            .parse()
            .expect("loopback address should parse");
        let config = Config {
            addr: loopback_addr,
        };
        assert!(config.addr.ip().is_loopback());
    }

    #[test]
    fn test_server_config_private_network() {
        let private_addr: SocketAddr = "192.168.1.1:8080"
            .parse()
            .expect("private address should parse");
        let config = Config { addr: private_addr };
        match config.addr.ip() {
            IpAddr::V4(ipv4) => {
                assert_eq!(ipv4.octets(), [192, 168, 1, 1]);
            }
            IpAddr::V6(_) => unreachable!("Expected IPv4 address"),
        }
    }

    // Note: The start_axum and start_actix functions are async functions that require
    // actual network binding and external dependencies (Agent, SignerFactory).
    // These would typically be tested with integration tests rather than unit tests,
    // as they require mocking complex external dependencies and actual network operations.
    // Unit testing these functions would require extensive mocking infrastructure
    // that goes beyond the scope of basic unit tests within the same file.
}
