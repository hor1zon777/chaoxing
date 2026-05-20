//! 任务调度引擎
//!
//! 对应 Python JobProcessor
//! 支持暂停/恢复/取消控制。
//! 章节处理顺序：
//! - chapters_per_course == 1 → 串行，保证平台的顺序解锁依赖
//! - chapters_per_course >  1 → 并发，由用户负责承担解锁风险

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinSet;

use crate::api::client::HttpClient;
use crate::error::AppError;
use crate::models::chapter::ChapterPoint;
use crate::models::course::CoursePointSelection;
use crate::models::events::TaskEvent;
use crate::task::chapter::{process_chapter, ChapterResult};
use crate::tiku::TikuManager;

const MAX_RETRIES: u32 = 5;

pub struct TaskScheduler {
    pub is_running: Arc<AtomicBool>,
    pub is_paused: Arc<AtomicBool>,
}

impl TaskScheduler {
    pub fn new(is_running: Arc<AtomicBool>, is_paused: Arc<AtomicBool>) -> Self {
        Self {
            is_running,
            is_paused,
        }
    }

    /// 运行整个课程的任务
    #[allow(clippy::too_many_arguments)]
    pub async fn run_course(
        &self,
        client: &HttpClient,
        course_id: &str,
        clazz_id: &str,
        cpi: &str,
        points: &[ChapterPoint],
        point_selections: &[CoursePointSelection],
        speed: f64,
        notopen_action: &str,
        tiku: Option<Arc<TikuManager>>,
        tasks_per_chapter: u32,
        chapters_per_course: u32,
        event_tx: &mpsc::UnboundedSender<TaskEvent>,
    ) -> Result<(), AppError> {
        let chapter_concurrency = chapters_per_course.max(1).min(8) as usize;

        if chapter_concurrency == 1 || points.len() <= 1 {
            self.run_course_serial(
                client,
                course_id,
                clazz_id,
                cpi,
                points,
                point_selections,
                speed,
                notopen_action,
                tiku,
                tasks_per_chapter,
                event_tx,
            )
            .await
        } else {
            self.run_course_parallel(
                client,
                course_id,
                clazz_id,
                cpi,
                points,
                point_selections,
                speed,
                notopen_action,
                tiku,
                tasks_per_chapter,
                chapter_concurrency,
                event_tx,
            )
            .await
        }
    }

    pub fn pause(&self) {
        self.is_paused.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.is_paused.store(false, Ordering::SeqCst);
    }

    pub fn cancel(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    /// 串行处理章节（保留顺序解锁依赖）
    #[allow(clippy::too_many_arguments)]
    async fn run_course_serial(
        &self,
        client: &HttpClient,
        course_id: &str,
        clazz_id: &str,
        cpi: &str,
        points: &[ChapterPoint],
        point_selections: &[CoursePointSelection],
        speed: f64,
        notopen_action: &str,
        tiku: Option<Arc<TikuManager>>,
        tasks_per_chapter: u32,
        event_tx: &mpsc::UnboundedSender<TaskEvent>,
    ) -> Result<(), AppError> {
        for point in points {
            if !self.is_running.load(Ordering::SeqCst) {
                tracing::info!("任务已取消");
                return Ok(());
            }

            while self.is_paused.load(Ordering::SeqCst) {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                if !self.is_running.load(Ordering::SeqCst) {
                    return Ok(());
                }
            }

            let cancelled = process_one_chapter(
                client,
                course_id,
                clazz_id,
                cpi,
                point,
                point_selections,
                speed,
                notopen_action,
                tiku.as_ref(),
                tasks_per_chapter,
                event_tx,
                &self.is_running,
                &self.is_paused,
            )
            .await;

            if cancelled {
                return Ok(());
            }
        }

        Ok(())
    }

    /// 并发处理章节（用户负责承担解锁风险）
    #[allow(clippy::too_many_arguments)]
    async fn run_course_parallel(
        &self,
        client: &HttpClient,
        course_id: &str,
        clazz_id: &str,
        cpi: &str,
        points: &[ChapterPoint],
        point_selections: &[CoursePointSelection],
        speed: f64,
        notopen_action: &str,
        tiku: Option<Arc<TikuManager>>,
        tasks_per_chapter: u32,
        chapter_concurrency: usize,
        event_tx: &mpsc::UnboundedSender<TaskEvent>,
    ) -> Result<(), AppError> {
        tracing::info!(
            "课程 {} 跨章节并发执行（{} 个章节，并发度 {}）",
            course_id,
            points.len(),
            chapter_concurrency
        );

        let semaphore = Arc::new(Semaphore::new(chapter_concurrency));
        let mut set: JoinSet<()> = JoinSet::new();

        let course_id_owned = course_id.to_string();
        let clazz_id_owned = clazz_id.to_string();
        let cpi_owned = cpi.to_string();
        let notopen_action_owned = notopen_action.to_string();
        let point_selections_owned: Vec<CoursePointSelection> = point_selections.to_vec();

        for point in points {
            if !self.is_running.load(Ordering::SeqCst) {
                break;
            }

            let point = point.clone();
            let client = client.clone();
            let course_id_t = course_id_owned.clone();
            let clazz_id_t = clazz_id_owned.clone();
            let cpi_t = cpi_owned.clone();
            let notopen_action_t = notopen_action_owned.clone();
            let point_selections_t = point_selections_owned.clone();
            let tiku_t = tiku.clone();
            let is_running_t = self.is_running.clone();
            let is_paused_t = self.is_paused.clone();
            let event_tx_t = event_tx.clone();
            let semaphore_t = semaphore.clone();

            set.spawn(async move {
                let _permit = match semaphore_t.acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => return,
                };

                while is_paused_t.load(Ordering::SeqCst) {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    if !is_running_t.load(Ordering::SeqCst) {
                        return;
                    }
                }

                if !is_running_t.load(Ordering::SeqCst) {
                    return;
                }

                process_one_chapter(
                    &client,
                    &course_id_t,
                    &clazz_id_t,
                    &cpi_t,
                    &point,
                    &point_selections_t,
                    speed,
                    &notopen_action_t,
                    tiku_t.as_ref(),
                    tasks_per_chapter,
                    &event_tx_t,
                    &is_running_t,
                    &is_paused_t,
                )
                .await;
            });
        }

        while let Some(joined) = set.join_next().await {
            if let Err(e) = joined {
                tracing::error!("章节任务 join 失败: {}", e);
            }
        }

        Ok(())
    }
}

/// 处理单个章节（带 NotOpen 重试 + 事件推送）。返回 true 表示被取消。
#[allow(clippy::too_many_arguments)]
async fn process_one_chapter(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    cpi: &str,
    point: &ChapterPoint,
    point_selections: &[CoursePointSelection],
    speed: f64,
    notopen_action: &str,
    tiku: Option<&Arc<TikuManager>>,
    tasks_per_chapter: u32,
    event_tx: &mpsc::UnboundedSender<TaskEvent>,
    is_running: &Arc<AtomicBool>,
    is_paused: &Arc<AtomicBool>,
) -> bool {
    let point_selection = point_selections
        .iter()
        .find(|selection| selection.point_id == point.id);

    let selected_job_count = point_selection
        .map(|selection| selection.selected_job_ids.len() as u32)
        .unwrap_or(point.job_count);

    let _ = event_tx.send(TaskEvent::ChapterStarted {
        course_id: course_id.to_string(),
        chapter_id: point.id.clone(),
        chapter_title: point.title.clone(),
        job_count: selected_job_count,
    });

    let mut attempt = 0u32;
    let result = loop {
        attempt += 1;
        let r = process_chapter(
            client,
            course_id,
            clazz_id,
            cpi,
            point,
            point_selection,
            speed,
            tiku,
            tasks_per_chapter,
            Some(event_tx),
            is_running,
            is_paused,
        )
        .await;

        if matches!(&r, ChapterResult::NotOpen)
            && notopen_action != "continue"
            && attempt < MAX_RETRIES
        {
            let _ = event_tx.send(TaskEvent::ChapterRetrying {
                course_id: course_id.to_string(),
                chapter_id: point.id.clone(),
                chapter_title: point.title.clone(),
                attempt,
                max_attempts: MAX_RETRIES,
            });
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            if !is_running.load(Ordering::SeqCst) {
                break ChapterResult::Cancelled;
            }
            continue;
        }
        break r;
    };

    match result {
        ChapterResult::Success => {
            let _ = event_tx.send(TaskEvent::ChapterCompleted {
                course_id: course_id.to_string(),
                chapter_id: point.id.clone(),
                chapter_title: point.title.clone(),
            });
            false
        }
        ChapterResult::Cancelled => {
            tracing::info!("章节任务已取消: {}", point.title);
            true
        }
        ChapterResult::NotOpen => {
            let _ = event_tx.send(TaskEvent::ChapterSkipped {
                course_id: course_id.to_string(),
                chapter_id: point.id.clone(),
                chapter_title: point.title.clone(),
                reason: "not_open".to_string(),
            });
            false
        }
        ChapterResult::Error => {
            let _ = event_tx.send(TaskEvent::ChapterSkipped {
                course_id: course_id.to_string(),
                chapter_id: point.id.clone(),
                chapter_title: point.title.clone(),
                reason: "error".to_string(),
            });
            false
        }
    }
}