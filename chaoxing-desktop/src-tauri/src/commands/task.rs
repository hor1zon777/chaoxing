//! 任务相关 Tauri 命令
//!
//! 提供前端调用的任务控制接口：
//! - start_course_tasks: 启动课程任务
//! - pause_tasks: 暂停任务
//! - resume_tasks: 恢复任务
//! - cancel_tasks: 取消任务

use std::sync::atomic::Ordering;

use tauri::ipc::Channel;
use tauri::State;
use tokio::sync::mpsc;

use crate::api::course_point::get_course_point;
use crate::error::AppError;
use crate::models::events::TaskEvent;
use crate::models::course::CourseTaskSelection;
use crate::state::AppState;
use crate::task::scheduler::TaskScheduler;
use crate::tiku::TikuManager;

/// 启动课程任务
///
/// 使用 Tauri Channel 向前端推送实时事件（进度、日志等）
#[tauri::command]
pub async fn start_course_tasks(
    state: State<'_, AppState>,
    channel: Channel<TaskEvent>,
    courses: Vec<CourseTaskSelection>,
    speed: f64,
    jobs: u32,
    notopen_action: String,
) -> Result<(), AppError> {
    let client_lock = state.client.read().await;
    let client = client_lock.as_ref().ok_or(AppError::Unauthorized)?;

    let (tx, mut rx) = mpsc::unbounded_channel::<TaskEvent>();

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = channel.send(event);
        }
    });

    state.is_running.store(true, Ordering::SeqCst);
    state.is_paused.store(false, Ordering::SeqCst);
    let scheduler = TaskScheduler::new(state.is_running.clone(), state.is_paused.clone());

    let speed = speed.clamp(1.0, 2.0);
    let jobs = jobs.clamp(1, 8);

    let config = state.config.read().await;
    let tiku = TikuManager::from_config(&config);
    drop(config);

    tiku.init().await;

    let config_check = state.config.read().await;
    if config_check.check_llm_connection && !tiku.disabled {
        drop(config_check);
        if !tiku.check_connection().await {
            let _ = tx.send(TaskEvent::Log {
                level: "error".to_string(),
                message: "题库/LLM 连接检查失败，题库功能将被禁用".to_string(),
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            });
        }
    } else {
        drop(config_check);
    }

    let tiku_ref = if tiku.disabled { None } else { Some(&tiku) };

    for course in &courses {
        if !state.is_running.load(Ordering::SeqCst) {
            break;
        }

        let selected_points = course.selected_points.len();
        let selected_jobs = course
            .selected_points
            .iter()
            .map(|point| point.selected_job_ids.len())
            .sum::<usize>();

        let _ = tx.send(TaskEvent::Log {
            level: "info".to_string(),
            message: format!(
                "开始学习课程: {} ({} 个章节，{} 个任务)",
                course.title, selected_points, selected_jobs
            ),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });

        let chapter_tree =
            get_course_point(client, &course.course_id, &course.clazz_id, &course.cpi).await?;
        let selected_chapters = chapter_tree
            .points
            .iter()
            .filter(|point| {
                course
                    .selected_points
                    .iter()
                    .any(|selected| selected.point_id == point.id)
            })
            .cloned()
            .collect::<Vec<_>>();

        let _ = tx.send(TaskEvent::CourseStarted {
            course_id: course.course_id.clone(),
            course_title: course.title.clone(),
            total_chapters: selected_chapters.len() as u32,
        });

        scheduler
            .run_course(
                client,
                &course.course_id,
                &course.clazz_id,
                &course.cpi,
                &selected_chapters,
                &course.selected_points,
                speed,
                jobs,
                &notopen_action,
                tiku_ref,
                &tx,
            )
            .await?;

        if !state.is_running.load(Ordering::SeqCst) {
            break;
        }

        let _ = tx.send(TaskEvent::CourseCompleted {
            course_id: course.course_id.clone(),
            course_title: course.title.clone(),
        });
    }

    let was_cancelled = !state.is_running.load(Ordering::SeqCst);
    state.is_running.store(false, Ordering::SeqCst);
    state.is_paused.store(false, Ordering::SeqCst);
    if !was_cancelled {
        let _ = tx.send(TaskEvent::AllTasksCompleted);
    }
    Ok(())
}

#[tauri::command]
pub async fn pause_tasks(state: State<'_, AppState>) -> Result<(), AppError> {
    state.is_paused.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn resume_tasks(state: State<'_, AppState>) -> Result<(), AppError> {
    state.is_paused.store(false, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn cancel_tasks(state: State<'_, AppState>) -> Result<(), AppError> {
    state.is_running.store(false, Ordering::SeqCst);
    state.is_paused.store(false, Ordering::SeqCst);
    Ok(())
}
