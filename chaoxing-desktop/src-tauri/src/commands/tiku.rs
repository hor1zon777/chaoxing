//! 题库相关 Tauri 命令

use tauri::State;

use crate::error::AppError;
use crate::state::AppState;
use crate::tiku::TikuManager;

/// 测试题库连接
///
/// 前端可调用此命令检查当前配置的题库是否可用
#[tauri::command]
pub async fn test_tiku_connection(state: State<'_, AppState>) -> Result<bool, AppError> {
    let config = state.config.read().await;
    let tiku = TikuManager::from_config(&config);
    drop(config);

    if tiku.disabled {
        return Ok(false);
    }

    // 初始化
    tiku.init().await;

    // 检查连接
    let result = tiku.check_connection().await;
    Ok(result)
}
