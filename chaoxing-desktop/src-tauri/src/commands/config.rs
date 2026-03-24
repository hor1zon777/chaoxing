//! 配置管理命令

use std::path::PathBuf;

use tauri::{Manager, State};

use crate::error::AppError;
use crate::models::config::AppConfig;
use crate::state::AppState;

/// 获取配置文件路径
fn config_file_path(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    std::fs::create_dir_all(&dir).ok();
    dir.join("config.json")
}

/// 获取当前配置
#[tauri::command]
pub async fn get_config(
    state: State<'_, AppState>,
) -> Result<AppConfig, AppError> {
    let config = state.config.read().await;
    Ok(config.clone())
}

/// 保存配置
#[tauri::command]
pub async fn save_config(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    config: AppConfig,
) -> Result<(), AppError> {
    let path = config_file_path(&app);
    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write(&path, json)?;
    let mut current = state.config.write().await;
    *current = config;
    tracing::info!("配置已保存到 {:?}", path);
    Ok(())
}

/// 加载配置（启动时调用）
#[tauri::command]
pub async fn load_config(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<AppConfig, AppError> {
    let path = config_file_path(&app);
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let config: AppConfig = serde_json::from_str(&content).unwrap_or_default();
        let mut current = state.config.write().await;
        *current = config.clone();
        Ok(config)
    } else {
        Ok(AppConfig::default())
    }
}

/// 导入 INI 配置文件（兼容 Python 版）
#[tauri::command]
pub async fn import_ini(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    path: String,
) -> Result<AppConfig, AppError> {
    let mut ini = configparser::ini::Ini::new();
    ini.load(&path)
        .map_err(|e| AppError::Config(format!("读取 INI 文件失败: {}", e)))?;

    let mut config = AppConfig::default();

    // [common] section
    if let Some(v) = ini.get("common", "speed") {
        config.speed = v.parse().unwrap_or(1.0);
    }
    if let Some(v) = ini.get("common", "jobs") {
        config.jobs = v.parse().unwrap_or(4);
    }
    if let Some(v) = ini.get("common", "notopen_action") {
        config.notopen_action = v;
    }

    // [tiku] section
    if let Some(v) = ini.get("tiku", "provider") {
        config.tiku_provider = v;
    }
    if let Some(v) = ini.get("tiku", "tokens") {
        config.tiku_tokens = v;
    }
    if let Some(v) = ini.get("tiku", "submit") {
        config.tiku_submit = v.to_lowercase() == "true";
    }
    if let Some(v) = ini.get("tiku", "cover_rate") {
        config.tiku_cover_rate = v.parse().unwrap_or(0.9);
    }
    if let Some(v) = ini.get("tiku", "delay") {
        config.tiku_delay = v.parse().unwrap_or(1.0);
    }
    if let Some(v) = ini.get("tiku", "endpoint") {
        config.ai_endpoint = v;
    }
    if let Some(v) = ini.get("tiku", "key") {
        config.ai_key = v;
    }
    if let Some(v) = ini.get("tiku", "model") {
        config.ai_model = v;
    }
    if let Some(v) = ini.get("tiku", "http_proxy") {
        config.ai_proxy = v;
    }
    if let Some(v) = ini.get("tiku", "min_interval_seconds") {
        config.ai_min_interval = v.parse().unwrap_or(3);
    }
    if let Some(v) = ini.get("tiku", "siliconflow_key") {
        config.siliconflow_key = v;
    }
    if let Some(v) = ini.get("tiku", "siliconflow_model") {
        config.siliconflow_model = v;
    }
    if let Some(v) = ini.get("tiku", "siliconflow_endpoint") {
        config.siliconflow_endpoint = v;
    }
    if let Some(v) = ini.get("tiku", "url") {
        config.tiku_adapter_url = v;
    }
    if let Some(v) = ini.get("tiku", "true_list") {
        config.true_list = v;
    }
    if let Some(v) = ini.get("tiku", "false_list") {
        config.false_list = v;
    }
    if let Some(v) = ini.get("tiku", "likeapi_search") {
        config.like_search = v.to_lowercase() == "true";
    }
    if let Some(v) = ini.get("tiku", "likeapi_vision") {
        config.like_vision = v.to_lowercase() == "true";
    }
    if let Some(v) = ini.get("tiku", "likeapi_model") {
        config.like_model = v;
    }
    if let Some(v) = ini.get("tiku", "likeapi_retry") {
        config.like_retry = v.to_lowercase() == "true";
    }
    if let Some(v) = ini.get("tiku", "likeapi_retry_times") {
        config.like_retry_times = v.parse().unwrap_or(3);
    }
    if let Some(v) = ini.get("tiku", "check_llm_connection") {
        config.check_llm_connection = v.to_lowercase() == "true";
    }

    // [notification] section
    if let Some(v) = ini.get("notification", "provider") {
        config.notification_provider = v;
    }
    if let Some(v) = ini.get("notification", "url") {
        config.notification_url = v;
    }
    if let Some(v) = ini.get("notification", "tg_chat_id") {
        config.notification_tg_chat_id = v;
    }

    // 保存
    let json_path = config_file_path(&app);
    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write(&json_path, json)?;
    let mut current = state.config.write().await;
    *current = config.clone();

    tracing::info!("INI 配置已导入并保存");
    Ok(config)
}
