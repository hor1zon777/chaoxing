//! 任务相关 Tauri 命令
//!
//! 提供前端调用的任务控制接口：
//! - start_course_tasks: 启动课程任务
//! - pause_tasks: 暂停任务
//! - resume_tasks: 恢复任务
//! - cancel_tasks: 取消任务

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::ipc::Channel;
use tauri::State;
use tokio::sync::{mpsc, Semaphore};

use crate::api::course_point::get_course_point;
use crate::error::AppError;
use crate::models::events::TaskEvent;
use crate::models::course::CourseTaskSelection;
use crate::state::AppState;
use crate::task::scheduler::TaskScheduler;
use crate::tiku::TikuManager;
use crate::utils::log_bridge;

/// RAII guard：确保 is_running 在任何退出路径（包括 panic）时都被重置
struct RunningGuard {
    is_running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
}

impl Drop for RunningGuard {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
    }
}

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
    let client = {
        let client_lock = state.client.read().await;
        client_lock.as_ref().ok_or(AppError::Unauthorized)?.clone()
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<TaskEvent>();

    // 激活日志桥接：所有 tracing 事件将转发到前端
    log_bridge::set_log_channel(tx.clone());

    // 保留 forwarder 句柄，函数退出前 await，避免多次进入时旧 forwarder 仍在跑、
    // 把后续 session 的事件错位归入新 channel
    let forwarder = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = channel.send(event);
        }
    });

    // 重入保护：原子 CAS 防止并发启动
    if state.is_running.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return Err(AppError::Other("任务已在运行中".to_string()));
    }

    // RAII guard：无论正常返回、提前 return 还是 panic，都会重置 is_running/is_paused
    let _guard = RunningGuard {
        is_running: state.is_running.clone(),
        is_paused: state.is_paused.clone(),
    };

    state.is_paused.store(false, Ordering::SeqCst);

    let original_speed = speed;
    let original_jobs = jobs;
    let speed = speed.clamp(1.0, 2.0);
    let jobs = jobs.clamp(1, 8);

    if (original_speed - speed).abs() > f64::EPSILON {
        let _ = tx.send(TaskEvent::Log {
            level: "warn".to_string(),
            message: format!(
                "播放速度 {} 超出允许范围，已自动调整为 {}",
                original_speed, speed
            ),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
    }

    if original_jobs != jobs {
        let _ = tx.send(TaskEvent::Log {
            level: "warn".to_string(),
            message: format!(
                "并发任务数 {} 超出允许范围，已自动调整为 {}",
                original_jobs, jobs
            ),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
    }

    // 一次性快照配置，避免多次 read 期间被 save_config 改写造成读到的字段不一致
    let (tiku, tasks_per_chapter, chapters_per_course, check_llm_connection) = {
        let config = state.config.read().await;
        (
            TikuManager::from_config(&config),
            config.tasks_per_chapter.clamp(1, 8),
            config.chapters_per_course.clamp(1, 8),
            config.check_llm_connection,
        )
    };

    tiku.init().await;

    let mut tiku_disabled_by_check = false;
    if check_llm_connection && !tiku.disabled {
        if !tiku.check_connection().await {
            tiku_disabled_by_check = true;
            let _ = tx.send(TaskEvent::Log {
                level: "error".to_string(),
                message: "题库/LLM 连接检查失败，题库功能将被禁用".to_string(),
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            });
        }
    }

    let tiku_ref = if tiku.disabled || tiku_disabled_by_check { None } else { Some(Arc::new(tiku)) };

    // 并发控制：使用 Semaphore 限制同时运行的课程数
    let semaphore = Arc::new(Semaphore::new(jobs as usize));
    let is_running = state.is_running.clone();
    let is_paused = state.is_paused.clone();
    let mut handles = Vec::new();

    for course in &courses {
        if !is_running.load(Ordering::SeqCst) {
            break;
        }

        let course = course.clone();
        let client = client.clone();
        let tx = tx.clone();
        let tiku_ref = tiku_ref.clone();
        let notopen_action = notopen_action.clone();
        let semaphore = semaphore.clone();
        let is_running = is_running.clone();
        let is_paused = is_paused.clone();
        let tasks_per_chapter = tasks_per_chapter;
        let chapters_per_course = chapters_per_course;

        let handle = tokio::spawn(async move {
            // semaphore 关闭时直接返回错误，避免许可缺失导致并发上限失效
            let _permit = match semaphore.acquire_owned().await {
                Ok(p) => p,
                Err(e) => {
                    let _ = tx.send(TaskEvent::Log {
                        level: "error".to_string(),
                        message: format!("获取课程并发许可失败: {}", e),
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                    });
                    return Err(AppError::Other(format!("信号量已关闭: {}", e)));
                }
            };

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

            let chapter_tree = match get_course_point(
                &client,
                &course.course_id,
                &course.clazz_id,
                &course.cpi,
            )
            .await
            {
                Ok(tree) => tree,
                Err(e) => {
                    let _ = tx.send(TaskEvent::CourseError {
                        course_id: course.course_id.clone(),
                        course_title: course.title.clone(),
                        error: e.to_string(),
                    });
                    // 单课失败不应连带取消整批任务；仅本课程退出
                    return Err(e);
                }
            };

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

            let scheduler = TaskScheduler::new(is_running.clone(), is_paused.clone());

            if let Err(e) = scheduler
                .run_course(
                    &client,
                    &course.course_id,
                    &course.clazz_id,
                    &course.cpi,
                    &selected_chapters,
                    &course.selected_points,
                    speed,
                    &notopen_action,
                    tiku_ref.clone(),
                    tasks_per_chapter,
                    chapters_per_course,
                    &tx,
                )
                .await
            {
                let _ = tx.send(TaskEvent::CourseError {
                    course_id: course.course_id.clone(),
                    course_title: course.title.clone(),
                    error: e.to_string(),
                });
                // 单课失败不应连带取消整批任务；仅本课程退出
                return Err(e);
            }

            if !is_running.load(Ordering::SeqCst) {
                return Err(AppError::Other("任务已取消".to_string()));
            }

            let _ = tx.send(TaskEvent::CourseCompleted {
                course_id: course.course_id.clone(),
                course_title: course.title.clone(),
            });

            Ok(())
        });

        handles.push(handle);
    }

    // 等待所有课程任务完成
    let mut task_error: Option<AppError> = None;
    for handle in handles {
        match handle.await {
            Ok(Ok(())) => { /* 课程成功完成 */ }
            Ok(Err(e)) => {
                task_error = Some(e);
            }
            Err(join_err) => {
                task_error = Some(AppError::Other(format!("课程任务异常: {}", join_err)));
            }
        }
    }

    // _guard 的 Drop 会自动重置 is_running / is_paused
    let was_cancelled = !is_running.load(Ordering::SeqCst);
    if !was_cancelled && task_error.is_none() {
        let _ = tx.send(TaskEvent::AllTasksCompleted);
    }

    // 清除日志桥接，停止追踪事件转发
    log_bridge::clear_log_channel();

    // drop 最后一个 sender 让 forwarder 的 rx.recv() 返回 None 而退出循环；
    // 然后 await 它结束，避免函数返回后旧 forwarder 仍存活导致事件错位
    drop(tx);
    let _ = forwarder.await;

    match task_error {
        Some(e) => Err(e),
        None => Ok(()),
    }
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
