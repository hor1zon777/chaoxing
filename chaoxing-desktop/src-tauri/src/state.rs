use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::client::HttpClient;
use crate::models::config::AppConfig;

pub struct AppState {
    pub client: Arc<RwLock<Option<HttpClient>>>,
    pub config: Arc<RwLock<AppConfig>>,
    pub is_running: Arc<std::sync::atomic::AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(AppConfig::default())),
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}
