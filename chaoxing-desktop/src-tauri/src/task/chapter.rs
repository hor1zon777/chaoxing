//! 章节处理
//!
//! 对应 Python process_chapter()
//! 获取章节任务列表并顺序处理每个任务点

use std::time::Duration;

use rand::Rng;

use crate::api::client::HttpClient;
use crate::api::{course_card, empty_page};
use crate::models::chapter::ChapterPoint;
use crate::models::events::TaskEvent;
use crate::task::job::process_job;
use crate::tiku::TikuManager;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChapterResult {
    Success,
    Error,
    NotOpen,
}

/// 处理单个章节
///
/// 流程：
/// 1. 检查是否已完成
/// 2. 获取任务列表
/// 3. 空任务列表时发送空页面请求
/// 4. 顺序处理每个任务点
pub async fn process_chapter(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    cpi: &str,
    point: &ChapterPoint,
    speed: f64,
    tiku: Option<&TikuManager>,
    event_tx: Option<&tokio::sync::mpsc::UnboundedSender<TaskEvent>>,
) -> ChapterResult {
    tracing::info!("当前章节: {}", point.title);

    if point.has_finished {
        tracing::info!("章节已完成: {}", point.title);
        return ChapterResult::Success;
    }

    // 随机延迟，模拟人类操作
    let delay = rand::thread_rng().gen_range(0.0..0.2);
    tokio::time::sleep(Duration::from_secs_f64(delay)).await;

    // 获取任务列表
    let (jobs, job_info) =
        match course_card::get_job_list(client, course_id, clazz_id, &point.id, cpi).await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("获取任务列表失败: {}", e);
                return ChapterResult::Error;
            }
        };

    if job_info.not_open {
        return ChapterResult::NotOpen;
    }

    if jobs.is_empty() {
        // 空页面任务
        let _ = empty_page::study_emptypage(client, course_id, clazz_id, &point.id, cpi).await;
    }

    // 顺序处理所有任务点
    // 注意：对于少量任务点（通常 1-3 个），顺序执行足够
    for job in &jobs {
        let result = process_job(
            client, course_id, clazz_id, cpi, job, &job_info, speed, tiku, event_tx,
        )
        .await;

        match result {
            Ok(r) if r.is_failure() => {
                tracing::warn!("任务失败: {}", job.name);
                return ChapterResult::Error;
            }
            Err(e) => {
                tracing::error!("任务异常: {}", e);
                return ChapterResult::Error;
            }
            _ => {}
        }
    }

    ChapterResult::Success
}
