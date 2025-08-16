use std::net::SocketAddr;
use std::sync::Arc;

use riglr_core::config::RpcConfig;
use riglr_web_adapters::{Agent, PromptRequest, SignerFactory};

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub addr: SocketAddr,
    pub rpc: RpcConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            addr: "0.0.0.0:8080".parse().unwrap(),
            rpc: RpcConfig::default(),
        }
    }
}

/// Start an Axum server exposing riglr endpoints. Enabled with the `axum` feature.
#[cfg(feature = "axum")]
pub async fn start_axum<A: Agent + Clone + Send + Sync + 'static>(
    config: ServerConfig,
    agent: A,
    signer_factory: Arc<dyn SignerFactory>,
) -> anyhow::Result<()> {
    use axum::{
        extract::State,
        http::HeaderMap,
        routing::{get, post},
        Json, Router,
    };
    use riglr_web_adapters::axum as axum_adapters;

    // Wire adapter with provided factory and rpc config
    let adapter = axum_adapters::AxumRiglrAdapter::new(signer_factory, config.rpc.clone());

    use std::sync::atomic::{AtomicU64, Ordering};

    #[derive(Clone)]
    struct AppState<A: Agent + Clone + Send + Sync + 'static> {
        adapter: axum_adapters::AxumRiglrAdapter,
        agent: A,
        sse_count: Arc<AtomicU64>,
        completion_count: Arc<AtomicU64>,
    }

    // Handlers using unified state
    async fn sse_handler<A: Agent + Clone + Send + Sync + 'static>(
        State(state): State<AppState<A>>,
        headers: HeaderMap,
        Json(prompt): Json<PromptRequest>,
    ) -> Result<
        axum::response::Sse<
            impl futures_util::Stream<Item = Result<axum::response::sse::Event, axum::Error>>,
        >,
        axum::http::StatusCode,
    > {
        state.sse_count.fetch_add(1, Ordering::Relaxed);
        state
            .adapter
            .sse_handler(headers, state.agent.clone(), prompt)
            .await
    }

    async fn completion_handler<A: Agent + Clone + Send + Sync + 'static>(
        State(state): State<AppState<A>>,
        headers: HeaderMap,
        Json(prompt): Json<PromptRequest>,
    ) -> Result<Json<riglr_web_adapters::CompletionResponse>, axum::http::StatusCode> {
        state.completion_count.fetch_add(1, Ordering::Relaxed);
        state
            .adapter
            .completion_handler(headers, state.agent.clone(), prompt)
            .await
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

    let app = Router::new()
        .route("/v1/stream", post(sse_handler::<A>))
        .route("/v1/completion", post(completion_handler::<A>))
        .route("/health", get(health))
        .route("/metrics", get(metrics::<A>))
        .route("/", get(info))
        .with_state(AppState {
            adapter,
            agent,
            sse_count: Arc::new(AtomicU64::new(0)),
            completion_count: Arc::new(AtomicU64::new(0)),
        });

    tracing::info!(addr = %config.addr, "Starting riglr Axum server");
    let listener = tokio::net::TcpListener::bind(config.addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

/// Start an Actix server exposing riglr endpoints. Enable with the `actix` feature.
#[cfg(feature = "actix")]
pub async fn start_actix<A: Agent + Clone + Send + Sync + 'static>(
    config: ServerConfig,
    agent: A,
    signer_factory: Arc<dyn SignerFactory>,
) -> anyhow::Result<()> {
    use actix_web::{web, App, HttpServer};
    use riglr_web_adapters::actix as actix_adapters;

    let adapter = actix_adapters::ActixRiglrAdapter::new(signer_factory, config.rpc.clone());
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
