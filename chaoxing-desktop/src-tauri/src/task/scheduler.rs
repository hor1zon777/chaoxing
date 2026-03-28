//! 任务调度引擎
//!
//! 对应 Python JobProcessor
//! 支持暂停/恢复/取消控制

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::mpsc;

use crate::api::client::HttpClient;
use crate::error::AppError;
use crate::models::chapter::ChapterPoint;
use crate::models::course::CoursePointSelection;
use crate::models::events::TaskEvent;
use crate::task::chapter::{process_chapter, ChapterResult};
use crate::tiku::TikuManager;

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
    pub async fn run_course(
        &self,
        client: &HttpClient,
        course_id: &str,
        clazz_id: &str,
        cpi: &str,
        points: &[ChapterPoint],
        point_selections: &[CoursePointSelection],
        speed: f64,
        _jobs: u32,
        notopen_action: &str,
        tiku: Option<&TikuManager>,
        event_tx: &mpsc::UnboundedSender<TaskEvent>,
    ) -> Result<(), AppError> {
        let mut i = 0;
        while i < points.len() {
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

            let point = &points[i];
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

            let result = process_chapter(
                client,
                course_id,
                clazz_id,
                cpi,
                point,
                point_selection,
                speed,
                tiku,
                Some(event_tx),
                &self.is_running,
                &self.is_paused,
            )
            .await;

            match result {
                ChapterResult::Success => {
                    let _ = event_tx.send(TaskEvent::ChapterCompleted {
                        course_id: course_id.to_string(),
                        chapter_id: point.id.clone(),
                        chapter_title: point.title.clone(),
                    });
                    i += 1;
                }
                ChapterResult::Cancelled => {
                    tracing::info!("章节任务已取消: {}", point.title);
                    return Ok(());
                }
                ChapterResult::NotOpen => match notopen_action {
                    "continue" => {
                        let _ = event_tx.send(TaskEvent::ChapterSkipped {
                            course_id: course_id.to_string(),
                            chapter_id: point.id.clone(),
                            chapter_title: point.title.clone(),
                            reason: "not_open".to_string(),
                        });
                        i += 1;
                    }
                    _ => {
                        let _ = event_tx.send(TaskEvent::ChapterRetrying {
                            course_id: course_id.to_string(),
                            chapter_id: point.id.clone(),
                            chapter_title: point.title.clone(),
                            attempt: 1,
                            max_attempts: 5,
                        });
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        i += 1;
                    }
                },
                ChapterResult::Error => {
                    let _ = event_tx.send(TaskEvent::ChapterSkipped {
                        course_id: course_id.to_string(),
                        chapter_id: point.id.clone(),
                        chapter_title: point.title.clone(),
                        reason: "error".to_string(),
                    });
                    i += 1;
                }
            }
        }

        Ok(())
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
}
