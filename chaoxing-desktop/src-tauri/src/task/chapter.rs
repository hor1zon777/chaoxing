//! 章节处理
//!
//! 对应 Python process_chapter()
//! 获取章节任务列表并按配置的并发度处理任务点。
//! 当 tasks_per_chapter > 1 时使用 JoinSet 并发执行；
//! 错误不立即中断，等所有任务完成后再汇总（对应 Python ThreadPoolExecutor.map 语义）。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use rand::Rng;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::api::client::HttpClient;
use crate::api::{course_card, empty_page};
use crate::error::AppError;
use crate::models::chapter::ChapterPoint;
use crate::models::course::CoursePointSelection;
use crate::models::events::TaskEvent;
use crate::models::job::{Job, JobInfo};
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

enum JobOutcome {
    Ok,
    Cancelled,
    Failed(String),
}

/// 处理单个章节
#[allow(clippy::too_many_arguments)]
pub async fn process_chapter(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    cpi: &str,
    point: &ChapterPoint,
    point_selection: Option<&CoursePointSelection>,
    speed: f64,
    tiku: Option<&Arc<TikuManager>>,
    tasks_per_chapter: u32,
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
    let filtered_jobs: Vec<Job> = if selected_job_ids.is_empty() {
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

    // 并发度：clamp 到 [1, 8]
    let concurrency = tasks_per_chapter.max(1).min(8) as usize;

    if concurrency == 1 || filtered_jobs.len() == 1 {
        run_jobs_serial(
            client,
            course_id,
            clazz_id,
            cpi,
            &point.id,
            &filtered_jobs,
            &job_info,
            speed,
            tiku.map(Arc::as_ref),
            event_tx,
            is_running,
            is_paused,
        )
        .await
    } else {
        run_jobs_parallel(
            client,
            course_id,
            clazz_id,
            cpi,
            &point.id,
            filtered_jobs,
            job_info,
            speed,
            tiku.cloned(),
            concurrency,
            event_tx,
            is_running,
            is_paused,
        )
        .await
    }
}

/// 串行执行（保留原有快速失败语义，单任务或并发=1 时使用）
#[allow(clippy::too_many_arguments)]
async fn run_jobs_serial(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    cpi: &str,
    chapter_id: &str,
    jobs: &[Job],
    job_info: &JobInfo,
    speed: f64,
    tiku: Option<&TikuManager>,
    event_tx: Option<&tokio::sync::mpsc::UnboundedSender<TaskEvent>>,
    is_running: &Arc<AtomicBool>,
    is_paused: &Arc<AtomicBool>,
) -> ChapterResult {
    for job in jobs {
        if !is_running.load(Ordering::SeqCst) {
            tracing::info!("任务已取消，停止处理章节");
            return ChapterResult::Cancelled;
        }

        emit_job_started(event_tx, course_id, chapter_id, job);

        let result = process_job(
            client,
            course_id,
            clazz_id,
            cpi,
            job,
            job_info,
            speed,
            tiku,
            event_tx,
            is_running,
            is_paused,
        )
        .await;

        if !is_running.load(Ordering::SeqCst) {
            return ChapterResult::Cancelled;
        }

        match classify_outcome(&result) {
            JobOutcome::Ok => emit_job_completed(event_tx, course_id, chapter_id, job),
            JobOutcome::Cancelled => return ChapterResult::Cancelled,
            JobOutcome::Failed(msg) => {
                emit_job_failed(event_tx, course_id, chapter_id, job, &msg);
                tracing::warn!("任务失败: {}", job.name);
                return ChapterResult::Error;
            }
        }
    }

    ChapterResult::Success
}

/// 并发执行（语义对齐 Python ThreadPoolExecutor.map：错误不立即中断，等所有任务跑完后汇总）
#[allow(clippy::too_many_arguments)]
async fn run_jobs_parallel(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    cpi: &str,
    chapter_id: &str,
    jobs: Vec<Job>,
    job_info: JobInfo,
    speed: f64,
    tiku: Option<Arc<TikuManager>>,
    concurrency: usize,
    event_tx: Option<&tokio::sync::mpsc::UnboundedSender<TaskEvent>>,
    is_running: &Arc<AtomicBool>,
    is_paused: &Arc<AtomicBool>,
) -> ChapterResult {
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut set: JoinSet<(Job, Result<StudyResult, AppError>)> = JoinSet::new();

    let course_id_owned = course_id.to_string();
    let clazz_id_owned = clazz_id.to_string();
    let cpi_owned = cpi.to_string();
    let event_tx_owned = event_tx.cloned();

    tracing::info!(
        "章节 {} 并发执行 {} 个任务点（并发度 {}）",
        chapter_id,
        jobs.len(),
        concurrency
    );

    for job in jobs {
        if !is_running.load(Ordering::SeqCst) {
            break;
        }

        let semaphore = semaphore.clone();
        let client = client.clone();
        let course_id_t = course_id_owned.clone();
        let clazz_id_t = clazz_id_owned.clone();
        let cpi_t = cpi_owned.clone();
        let job_info = job_info.clone();
        let is_running_t = is_running.clone();
        let is_paused_t = is_paused.clone();
        let tiku_clone = tiku.clone();
        let event_tx_clone = event_tx_owned.clone();
        let chapter_id_t = chapter_id.to_string();

        set.spawn(async move {
            // 必须先拿到信号量许可，再发"开始"事件，避免出现
            // 任务还在排队但前端已显示"已开始"的悬挂条目
            let _permit = match semaphore.acquire_owned().await {
                Ok(p) => p,
                Err(_) => {
                    return (
                        job,
                        Err(AppError::Other("信号量已关闭".to_string())),
                    );
                }
            };

            // 拿到许可后立即响应取消信号
            if !is_running_t.load(Ordering::SeqCst) {
                return (job, Ok(StudyResult::Cancelled));
            }

            // 拿到许可后响应暂停信号；并发模式下也能在新任务启动前生效
            while is_paused_t.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(500)).await;
                if !is_running_t.load(Ordering::SeqCst) {
                    return (job, Ok(StudyResult::Cancelled));
                }
            }

            emit_job_started(event_tx_clone.as_ref(), &course_id_t, &chapter_id_t, &job);

            let tiku_ref = tiku_clone.as_deref();
            let result = process_job(
                &client,
                &course_id_t,
                &clazz_id_t,
                &cpi_t,
                &job,
                &job_info,
                speed,
                tiku_ref,
                event_tx_clone.as_ref(),
                &is_running_t,
                &is_paused_t,
            )
            .await;
            (job, result)
        });
    }

    // 等待所有任务完成（错误不立即中断）
    let mut has_error = false;
    let mut cancelled = false;
    while let Some(joined) = set.join_next().await {
        let (job, result) = match joined {
            Ok(pair) => pair,
            Err(e) => {
                tracing::error!("任务 join 失败: {}", e);
                has_error = true;
                continue;
            }
        };

        match classify_outcome(&result) {
            JobOutcome::Ok => emit_job_completed(event_tx, course_id, chapter_id, &job),
            JobOutcome::Cancelled => {
                cancelled = true;
            }
            JobOutcome::Failed(msg) => {
                emit_job_failed(event_tx, course_id, chapter_id, &job, &msg);
                tracing::warn!("任务失败: {}", job.name);
                has_error = true;
            }
        }
    }

    // 优先返回 Error：即便用户中途取消，有错误的章节仍应被统计为失败，
    // 避免取消信号吞掉真实的失败原因
    if has_error {
        ChapterResult::Error
    } else if !is_running.load(Ordering::SeqCst) || cancelled {
        ChapterResult::Cancelled
    } else {
        ChapterResult::Success
    }
}

/// 将一次任务调用结果归类
fn classify_outcome(result: &Result<StudyResult, AppError>) -> JobOutcome {
    match result {
        Ok(StudyResult::Cancelled) => JobOutcome::Cancelled,
        Ok(r) if r.is_failure() => JobOutcome::Failed(format!("{:?}", r)),
        Ok(_) => JobOutcome::Ok,
        Err(e) => JobOutcome::Failed(e.to_string()),
    }
}

fn emit_job_started(
    event_tx: Option<&tokio::sync::mpsc::UnboundedSender<TaskEvent>>,
    course_id: &str,
    chapter_id: &str,
    job: &Job,
) {
    if let Some(tx) = event_tx {
        let _ = tx.send(TaskEvent::JobStarted {
            course_id: course_id.to_string(),
            chapter_id: chapter_id.to_string(),
            job_id: job.jobid.clone(),
            job_name: job.name.clone(),
            job_type: format!("{:?}", job.job_type),
        });
    }
}

fn emit_job_completed(
    event_tx: Option<&tokio::sync::mpsc::UnboundedSender<TaskEvent>>,
    course_id: &str,
    chapter_id: &str,
    job: &Job,
) {
    if let Some(tx) = event_tx {
        let _ = tx.send(TaskEvent::JobCompleted {
            course_id: course_id.to_string(),
            chapter_id: chapter_id.to_string(),
            job_id: job.jobid.clone(),
            job_name: job.name.clone(),
            job_type: format!("{:?}", job.job_type),
        });
    }
}

fn emit_job_failed(
    event_tx: Option<&tokio::sync::mpsc::UnboundedSender<TaskEvent>>,
    course_id: &str,
    chapter_id: &str,
    job: &Job,
    msg: &str,
) {
    if let Some(tx) = event_tx {
        let _ = tx.send(TaskEvent::JobFailed {
            course_id: course_id.to_string(),
            chapter_id: chapter_id.to_string(),
            job_id: job.jobid.clone(),
            job_name: job.name.clone(),
            error: msg.to_string(),
        });
    }
}