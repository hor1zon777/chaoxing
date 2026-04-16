//! 直播任务处理器
//!
//! 对应 Python: api/live_process.py (LiveProcessor)

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tokio::sync::mpsc;

use crate::api::client::HttpClient;
use crate::api::live;
use crate::error::AppError;
use crate::models::events::TaskEvent;
use crate::models::job::Job;
use crate::models::video::StudyResult;

/// 循环提交直播时长
/// 对应 Python: LiveProcessor.run_live()
pub async fn run_live(
    client: &HttpClient,
    job: &Job,
    course_id: &str,
    clazz_id: &str,
    knowledge_id: &str,
    speed: f64,
    event_tx: Option<&mpsc::UnboundedSender<TaskEvent>>,
    is_running: &Arc<AtomicBool>,
    is_paused: &Arc<AtomicBool>,
) -> Result<StudyResult, AppError> {
    let user_id = client.get_uid().unwrap_or_default();
    let live_name = job
        .property
        .get("title")
        .and_then(|v| v.as_str())
        .or_else(|| job.property.get("name").and_then(|v| v.as_str()))
        .unwrap_or("未知直播");

    // 获取直播状态
    let status =
        live::get_live_status(client, job, course_id, clazz_id, knowledge_id, &user_id).await?;

    let duration_secs = status
        .as_ref()
        .and_then(|s| s.get("temp"))
        .and_then(|t| t.get("data"))
        .and_then(|d| d.get("duration"))
        .and_then(|d| d.as_u64())
        .unwrap_or(30 * 60); // 默认 30 分钟

    // 根据倍速调整时长，向上取整为分钟
    let speed = if speed <= 0.0 { 1.0 } else { speed };
    let adjusted = duration_secs as f64 / speed;
    let total_minutes = ((adjusted as u64) + 59) / 60;

    tracing::info!(
        "开始刷取直播 '{}'，总时长 {} 分钟（已根据倍速调整）",
        live_name,
        total_minutes
    );

    // 循环提交（每分钟一次）
    for i in 0..total_minutes {
        if !is_running.load(Ordering::SeqCst) {
            tracing::info!("直播任务已取消: {}", live_name);
            return Ok(StudyResult::Cancelled);
        }

        while is_paused.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if !is_running.load(Ordering::SeqCst) {
                tracing::info!("直播任务已取消: {}", live_name);
                return Ok(StudyResult::Cancelled);
            }
        }

        tracing::info!(
            "直播 '{}' 已观看 {}/{} 分钟",
            live_name,
            i + 1,
            total_minutes
        );

        // 推送进度事件
        if let Some(tx) = event_tx {
            let _ = tx.send(TaskEvent::LiveProgress {
                course_id: course_id.to_string(),
                job_id: job.jobid.clone(),
                job_name: live_name.to_string(),
                current_minute: (i + 1) as u32,
                total_minutes: total_minutes as u32,
            });
        }

        let success = live::submit_live_time(client, job, course_id, &user_id).await?;
        if !success {
            tracing::warn!("第 {} 分钟时长提交失败，将重试", i + 1);
            tokio::time::sleep(Duration::from_secs(5)).await;
            let _ = live::submit_live_time(client, job, course_id, &user_id).await;
        }

        // 根据倍速调整间隔 (59 / speed 秒)
        let sleep_time = 59.0 / speed.max(0.1);
        tokio::time::sleep(Duration::from_secs_f64(sleep_time)).await;
    }

    tracing::info!("直播 '{}' 时长刷取完成", live_name);
    Ok(StudyResult::Success)
}
