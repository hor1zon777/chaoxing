//! 章节处理
//!
//! 对应 Python process_chapter()
//! 获取章节任务列表并顺序处理每个任务点

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use rand::Rng;

use crate::api::client::HttpClient;
use crate::api::{course_card, empty_page};
use crate::models::chapter::ChapterPoint;
use crate::models::course::CoursePointSelection;
use crate::models::events::TaskEvent;
use crate::models::video::StudyResult;
use crate::task::job::process_job;
use crate::tiku::TikuManager;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChapterResult {
    Success,
    Error,
    NotOpen,
    Cancelled,
}

/// 处理单个章节
pub async fn process_chapter(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    cpi: &str,
    point: &ChapterPoint,
    point_selection: Option<&CoursePointSelection>,
    speed: f64,
    tiku: Option<&TikuManager>,
    event_tx: Option<&tokio::sync::mpsc::UnboundedSender<TaskEvent>>,
    is_running: &Arc<AtomicBool>,
    is_paused: &Arc<AtomicBool>,
) -> ChapterResult {
    tracing::info!("当前章节: {}", point.title);

    if point.has_finished {
        tracing::info!("章节已完成: {}", point.title);
        return ChapterResult::Success;
    }

    let delay = rand::thread_rng().gen_range(0.0..0.2);
    tokio::time::sleep(Duration::from_secs_f64(delay)).await;

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

    let selected_job_ids = point_selection
        .map(|selection| selection.selected_job_ids.as_slice())
        .unwrap_or(&[]);
    let filtered_jobs = if selected_job_ids.is_empty() {
        jobs.into_iter().filter(|job| !job.is_completed).collect()
    } else {
        jobs.into_iter()
            .filter(|job| {
                !job.is_completed && selected_job_ids.iter().any(|job_id| job_id == &job.jobid)
            })
            .collect::<Vec<_>>()
    };

    if filtered_jobs.is_empty() {
        let _ = empty_page::study_emptypage(client, course_id, clazz_id, &point.id, cpi).await;
        return ChapterResult::Success;
    }

    for job in &filtered_jobs {
        if !is_running.load(Ordering::SeqCst) {
            tracing::info!("任务已取消，停止处理章节: {}", point.title);
            return ChapterResult::Cancelled;
        }

        // 发送 JobStarted 事件
        if let Some(tx) = event_tx {
            let _ = tx.send(TaskEvent::JobStarted {
                course_id: course_id.to_string(),
                chapter_id: point.id.clone(),
                job_id: job.jobid.clone(),
                job_name: job.name.clone(),
                job_type: format!("{:?}", job.job_type),
            });
        }

        let result = process_job(
            client,
            course_id,
            clazz_id,
            cpi,
            job,
            &job_info,
            speed,
            tiku,
            event_tx,
            is_running,
            is_paused,
        )
        .await;

        if !is_running.load(Ordering::SeqCst) {
            tracing::info!("任务已取消，停止处理章节: {}", point.title);
            return ChapterResult::Cancelled;
        }

        match result {
            Ok(StudyResult::Cancelled) => {
                return ChapterResult::Cancelled;
            }
            Ok(r) if r.is_failure() => {
                // 发送 JobFailed 事件
                if let Some(tx) = event_tx {
                    let _ = tx.send(TaskEvent::JobFailed {
                        course_id: course_id.to_string(),
                        chapter_id: point.id.clone(),
                        job_id: job.jobid.clone(),
                        job_name: job.name.clone(),
                        error: format!("{:?}", r),
                    });
                }
                tracing::warn!("任务失败: {}", job.name);
                return ChapterResult::Error;
            }
            Err(e) => {
                // 发送 JobFailed 事件
                if let Some(tx) = event_tx {
                    let _ = tx.send(TaskEvent::JobFailed {
                        course_id: course_id.to_string(),
                        chapter_id: point.id.clone(),
                        job_id: job.jobid.clone(),
                        job_name: job.name.clone(),
                        error: e.to_string(),
                    });
                }
                tracing::error!("任务异常: {}", e);
                return ChapterResult::Error;
            }
            Ok(_) => {
                // 发送 JobCompleted 事件
                if let Some(tx) = event_tx {
                    let _ = tx.send(TaskEvent::JobCompleted {
                        course_id: course_id.to_string(),
                        chapter_id: point.id.clone(),
                        job_id: job.jobid.clone(),
                        job_name: job.name.clone(),
                        job_type: format!("{:?}", job.job_type),
                    });
                }
            }
        }
    }

    ChapterResult::Success
}
