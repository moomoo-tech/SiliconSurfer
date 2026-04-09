use axum::{extract::State, http::StatusCode, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use agent_browser_core::{Engine, FetchMode};
use agent_browser_core::distiller_fast::FastDistiller;
use agent_browser_core::probe::{Probe, ProbeRequest, ProbeResult};

struct AppState {
    engine: Engine,
    probe: Probe,
}

#[derive(Deserialize)]
struct FetchRequest {
    url: String,
    #[serde(default = "default_output")]
    output: String,
    /// "t0", "t1", or "auto"
    #[serde(default)]
    mode: FetchMode,
    #[serde(default = "default_timeout")]
    timeout_secs: u64,
    /// Use fast (lol_html) distiller instead of default (scraper)
    #[serde(default)]
    fast: bool,
    /// Distill mode: "llm_friendly", "reader", "operator", "spider", "developer", "data"
    #[serde(default)]
    distill: agent_browser_core::distiller_fast::DistillMode,
}

fn default_output() -> String {
    "markdown".to_string()
}

fn default_timeout() -> u64 {
    30
}

#[derive(Serialize)]
struct FetchResponse {
    url: String,
    title: Option<String>,
    content: String,
    content_length: usize,
    mode_used: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    browser_ready: bool,
}

async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        browser_ready: true, // TODO: actual check
    })
}

async fn fetch_page(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FetchRequest>,
) -> Result<Json<FetchResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Use fetch_full to pass distill mode
    match state.engine.fetch_full(&req.url, &req.output, req.mode, req.fast, req.distill).await {
        Ok(result) => Ok(Json(FetchResponse {
            url: result.url,
            title: result.title,
            content: result.content,
            content_length: result.content_length,
            mode_used: result.mode_used,
        })),
        Err(e) => Err((
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

/// Distill raw HTML directly (no HTTP fetch). Used when Agent already has the page
/// via Playwright/CDP and just needs our distiller to process it.
#[derive(Deserialize)]
struct DistillRequest {
    html: String,
    url: Option<String>,
    #[serde(default)]
    distill: agent_browser_core::distiller_fast::DistillMode,
}

async fn distill_html(
    Json(req): Json<DistillRequest>,
) -> Json<FetchResponse> {
    let base_url = req.url.as_deref();
    let title = agent_browser_core::distiller_fast::FastDistiller::extract_title(&req.html);
    let content = agent_browser_core::distiller_fast::FastDistiller::distill(&req.html, req.distill, base_url);
    let content_length = content.len();

    Json(FetchResponse {
        url: req.url.unwrap_or_default(),
        title,
        content,
        content_length,
        mode_used: format!("{:?}", req.distill).to_lowercase(),
    })
}

async fn probe_page(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ProbeRequest>,
) -> Result<Json<ProbeResult>, (StatusCode, Json<ErrorResponse>)> {
    match state.probe.check(req).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let engine = Engine::new();

    // Start T1 browser daemon
    if let Err(e) = engine.start_browser().await {
        eprintln!("Warning: T1 browser failed to start: {e}");
        eprintln!("T1 mode will be unavailable. T0 (reqwest) still works.");
    } else {
        println!("T1 headless Chrome daemon started");
    }

    let probe = Probe::new().with_browser(engine.browser_pool());

    let state = Arc::new(AppState {
        engine,
        probe,
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/probe", post(probe_page))
        .route("/fetch", post(fetch_page))
        .route("/distill", post(distill_html))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("Agent Browser listening on http://{}", addr);
    println!("  POST /fetch  {{\"url\": \"...\", \"mode\": \"t0|t1|auto\"}}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
