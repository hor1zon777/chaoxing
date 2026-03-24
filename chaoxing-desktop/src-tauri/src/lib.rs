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

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::auth::login,
            commands::auth::login_with_cookies,
            commands::auth::logout,
            commands::auth::validate_session,
            commands::course::get_courses,
            commands::course::get_chapter_tree,
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
        ])
        .run(tauri::generate_context!())
        .expect("启动应用失败");
}
