//! 任务分发
//!
//! 对应 Python process_job()
//! 根据 JobType 分发到对应的处理函数

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::api::client::HttpClient;
use crate::api::{document, read, video, work};
use crate::error::AppError;
use crate::models::events::TaskEvent;
use crate::models::job::{Job, JobInfo, JobType};
use crate::models::video::StudyResult;
use crate::task::live_processor;
use crate::tiku::TikuManager;

/// 处理单个任务点
///
/// 根据任务类型分发到对应的 API 调用：
/// - Video: 视频任务（失败时回退尝试音频模式）
/// - Document: 文档任务
/// - Read: 阅读任务
/// - Work: 章节检测（答题）
/// - Live: 直播任务
pub async fn process_job(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    cpi: &str,
    job: &Job,
    job_info: &JobInfo,
    speed: f64,
    tiku: Option<&TikuManager>,
    event_tx: Option<&tokio::sync::mpsc::UnboundedSender<TaskEvent>>,
    is_running: &Arc<AtomicBool>,
    is_paused: &Arc<AtomicBool>,
) -> Result<StudyResult, AppError> {
    match job.job_type {
        JobType::Video => {
            tracing::info!("识别到视频任务: {}", job.name);
            let result = video::study_video(
                client,
                course_id,
                clazz_id,
                cpi,
                job,
                job_info,
                speed,
                "Video",
                event_tx,
                is_running,
                is_paused,
            )
            .await?;

            if result.is_failure() {
                tracing::warn!("视频模式失败，尝试音频模式: {}", job.name);
                return video::study_video(
                    client,
                    course_id,
                    clazz_id,
                    cpi,
                    job,
                    job_info,
                    speed,
                    "Audio",
                    event_tx,
                    is_running,
                    is_paused,
                )
                .await;
            }
            Ok(result)
        }
        JobType::Document => {
            tracing::info!("识别到文档任务: {}", job.name);
            document::study_document(client, course_id, clazz_id, job).await
        }
        JobType::Read => {
            tracing::info!("识别到阅读任务: {}", job.name);
            read::study_read(client, course_id, clazz_id, job, job_info).await
        }
        JobType::Work => {
            tracing::info!("识别到章节检测: {}", job.name);
            match tiku {
                Some(tiku_mgr) => {
                    work::study_work(client, course_id, clazz_id, cpi, job, job_info, tiku_mgr)
                        .await
                }
                None => {
                    tracing::info!("题库未配置，跳过章节检测");
                    Ok(StudyResult::Success)
                }
            }
        }
        JobType::Live => {
            tracing::info!("识别到直播任务: {}", job.name);
            live_processor::run_live(
                client,
                job,
                course_id,
                clazz_id,
                &job_info.knowledgeid,
                speed,
                event_tx,
                is_running,
                is_paused,
            )
            .await
        }
    }
}
