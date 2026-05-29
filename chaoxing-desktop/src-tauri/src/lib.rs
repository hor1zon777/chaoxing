#![allow(dead_code)]

mod error;
mod state;
mod models;
mod crypto;
mod utils;
mod api;
mod parser;
mod tiku;
mod notification;
mod task;
mod commands;
mod font;

use state::AppState;
use tauri::Manager;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use crate::utils::log_bridge::FrontendLogLayer;

pub fn run() {
    // 初始化 tracing 订阅器
    // - RUST_LOG 环境变量控制终端 fmt 层级别（dev 默认 info，release 默认 info）
    // - FrontendLogLayer 始终全级别转发（不受 RUST_LOG 影响）
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // 前端日志桥接：始终 info 级别（前端仅显示业务日志，不显示库的 DEBUG/TRACE）
    let frontend_filter = EnvFilter::new("info");

    tracing_subscriber::registry()
        .with(fmt::layer().with_filter(env_filter))
        .with(FrontendLogLayer.with_filter(frontend_filter))
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 启动时自动加载配置（确保持久化配置在任务开始前已就绪）
            // blocking_write 在 tokio 异步 runtime 中调用会 panic，
            // 这里用 async_runtime::block_on 在同步 setup 中安全获取异步锁
            let state = app.state::<AppState>();
            let config_dir = app.path().app_config_dir().unwrap_or_default();
            // 题库缓存与 config.json 同根，避免落到 LOCALAPPDATA / HOME 不同位置
            crate::tiku::set_cache_base_dir(config_dir.clone());
            let config_path = config_dir.join("config.json");
            if config_path.exists() {
                match std::fs::read_to_string(&config_path) {
                    Ok(content) => match serde_json::from_str::<crate::models::config::AppConfig>(&content) {
                        Ok(config) => {
                            tauri::async_runtime::block_on(async {
                                let mut current = state.config.write().await;
                                *current = config;
                            });
                            tracing::info!("启动时加载配置: {:?}", config_path);
                        }
                        Err(e) => {
                            tracing::error!(
                                "启动时配置解析失败，将保留默认配置（路径: {:?}, 错误: {}）",
                                config_path,
                                e
                            );
                        }
                    },
                    Err(e) => {
                        tracing::error!(
                            "启动时配置读取失败（路径: {:?}, 错误: {}）",
                            config_path,
                            e
                        );
                    }
                }
            }
            Ok(())
        })
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::auth::login,
            commands::auth::login_with_cookies,
            commands::auth::list_saved_accounts,
            commands::auth::login_with_saved_account,
            commands::auth::delete_saved_account,
            commands::auth::logout,
            commands::auth::validate_session,
            commands::course::get_courses,
            commands::course::get_chapter_tree,
            commands::course::get_course_selection_tree,
            commands::task::start_course_tasks,
            commands::task::pause_tasks,
            commands::task::resume_tasks,
            commands::task::cancel_tasks,
            commands::notification::test_notification,
            commands::notification::send_notification,
            commands::config::get_config,
            commands::config::save_config,
            commands::config::load_config,
            commands::config::import_ini,
            commands::tiku::test_tiku_connection,
            commands::topic::send_topic,
            commands::topic::list_course_topics,
            commands::topic::list_my_topics,
        ])
        .run(tauri::generate_context!())
        .expect("启动应用失败");
}
