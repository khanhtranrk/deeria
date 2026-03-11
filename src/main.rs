mod config;
mod http_client;
mod proxy;
mod state;

use axum::{
    Router,
    http::{HeaderMap, header},
    response::{Html, IntoResponse},
    routing::get,
};
use clap::Parser;
use state::AppState;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const HTML: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/index.html"));
const IMAGE: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/deeria.png"));

#[derive(Parser)]
struct Args {
    #[arg(long)]
    host: Option<String>,

    #[arg(long)]
    port: Option<u16>,

    #[arg(long)]
    config: Option<String>,
}

async fn index_page() -> Html<&'static str> {
    Html(HTML)
}

async fn deeria_image() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "image/png".parse().unwrap());

    (headers, IMAGE)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_axum=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting deeria...");

    let args = Args::parse();

    tracing::info!("Loading configuration...");

    let mut cfg = config::load_config(args.config);

    if let Some(host) = args.host {
        cfg.server.host = host;
    }
    if let Some(port) = args.port {
        cfg.server.port = port;
    }

    let state = AppState::new(Arc::new(cfg));

    let addr = format!("{}:{}", state.config.server.host, state.config.server.port);

    let app = Router::new()
        .route("/", get(index_page))
        .route("/deeria.png", get(deeria_image))
        .route("/{id}/{*path}", get(proxy::handler))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    tracing::info!("Server running on http://{}", addr);

    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.unwrap();
}
