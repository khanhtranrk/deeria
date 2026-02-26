mod config;
mod http_client;
mod proxy;
mod state;

use axum::{Router, routing::get};
use clap::Parser;
use state::AppState;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    host: Option<String>,

    #[arg(long)]
    port: Option<u16>,

    #[arg(long)]
    config: Option<String>,
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
        .route("/{id}/{*path}", get(proxy::handler))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    tracing::info!("Server running on http://{}", addr);

    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.unwrap();
}
