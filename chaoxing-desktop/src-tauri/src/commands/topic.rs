//! 讨论区命令：发帖 / 列话题
//!
//! `puid` 取自当前登录会话的 uid（cookie），无需前端传入。

use tauri::State;

use crate::api::topic;
use crate::error::AppError;
use crate::models::topic::{AddTopicResult, TopicListItem};
use crate::state::AppState;

/// 发布讨论区话题（写操作：会在课程讨论区真实发帖）
#[tauri::command]
pub async fn send_topic(
    state: State<'_, AppState>,
    course_id: String,
    clazz_id: String,
    title: String,
    content: String,
) -> Result<AddTopicResult, AppError> {
    let client = {
        let lock = state.client.read().await;
        lock.as_ref().ok_or(AppError::Unauthorized)?.clone()
    };
    let puid = client.get_uid().ok_or(AppError::Unauthorized)?;

    if title.trim().is_empty() && content.trim().is_empty() {
        return Err(AppError::Other("标题与正文不能同时为空".to_string()));
    }

    topic::add_topic(&client, &course_id, &clazz_id, &puid, &title, &content).await
}

/// 列出课程讨论区话题（用于确认发帖结果 / 校验 bbsid）
#[tauri::command]
pub async fn list_course_topics(
    state: State<'_, AppState>,
    course_id: String,
    clazz_id: String,
) -> Result<Vec<TopicListItem>, AppError> {
    let client = {
        let lock = state.client.read().await;
        lock.as_ref().ok_or(AppError::Unauthorized)?.clone()
    };
    let puid = client.get_uid().ok_or(AppError::Unauthorized)?;

    topic::list_topics(&client, &course_id, &clazz_id, &puid).await
}

/// 列出“我发布的”话题（groupweb getMyTopicList，服务端按登录用户过滤，无需 puid）。
#[tauri::command]
pub async fn list_my_topics(
    state: State<'_, AppState>,
    course_id: String,
) -> Result<Vec<TopicListItem>, AppError> {
    let client = {
        let lock = state.client.read().await;
        lock.as_ref().ok_or(AppError::Unauthorized)?.clone()
    };

    topic::list_my_topics(&client, &course_id).await
}
