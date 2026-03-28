use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::RwLock;

use crate::api::client::HttpClient;
use crate::models::config::AppConfig;

pub struct AppState {
    pub client: Arc<RwLock<Option<HttpClient>>>,
    pub active_account_id: Arc<RwLock<Option<String>>>,
    pub config: Arc<RwLock<AppConfig>>,
    pub is_running: Arc<AtomicBool>,
    pub is_paused: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            active_account_id: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(AppConfig::default())),
            is_running: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
        }
    }
}
