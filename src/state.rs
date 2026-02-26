use crate::config::AppConfig;
use crate::http_client;
use reqwest_middleware::ClientWithMiddleware;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub client: ClientWithMiddleware,
}

impl AppState {
    pub fn new(config: Arc<AppConfig>) -> Self {
        let client = http_client::build_http_client();

        Self { config, client }
    }
}
