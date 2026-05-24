//! 通知相关命令

use tauri::State;

use crate::error::AppError;
use crate::notification::NotificationProvider;
use crate::state::AppState;

/// 发送测试通知
#[tauri::command]
pub async fn test_notification(
    state: State<'_, AppState>,
    provider: String,
    url: String,
    tg_chat_id: String,
) -> Result<bool, AppError> {
    // 基于当前持久配置克隆出测试副本，只覆盖通知三字段；
    // 避免用 AppConfig::default() 把 speed / jobs / 题库等其他字段清零，
    // 防止某些 provider 校验出口依赖其他默认值差异
    let mut config = {
        let snapshot = state.config.read().await;
        snapshot.clone()
    };
    config.notification_provider = provider;
    config.notification_url = url;
    config.notification_tg_chat_id = tg_chat_id;

    let notifier = NotificationProvider::from_config(&config);
    match notifier.send("超星学习通助手 - 测试通知").await {
        Ok(_) => Ok(true),
        Err(e) => {
            tracing::error!("测试通知失败: {}", e);
            Ok(false)
        }
    }
}

/// 发送通知
#[tauri::command]
pub async fn send_notification(
    state: State<'_, AppState>,
    message: String,
) -> Result<(), AppError> {
    let config = state.config.read().await;
    let notifier = NotificationProvider::from_config(&config);
    notifier.send(&message).await
}
