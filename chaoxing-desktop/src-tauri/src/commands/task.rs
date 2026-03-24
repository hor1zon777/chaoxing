//! 任务相关 Tauri 命令
//!
//! 提供前端调用的任务控制接口：
//! - start_course_tasks: 启动课程任务
//! - pause_tasks: 暂停任务
//! - resume_tasks: 恢复任务
//! - cancel_tasks: 取消任务

use tauri::ipc::Channel;
use tauri::State;
use tokio::sync::mpsc;

use crate::api::course_point::get_course_point;
use crate::error::AppError;
use crate::models::events::TaskEvent;
use crate::state::AppState;
use crate::task::scheduler::TaskScheduler;
use crate::tiku::TikuManager;

/// 课程信息参数
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseParam {
    pub course_id: String,
    pub clazz_id: String,
    pub cpi: String,
    pub title: String,
}

/// 启动课程任务
///
/// 使用 Tauri Channel 向前端推送实时事件（进度、日志等）
#[tauri::command]
pub async fn start_course_tasks(
    state: State<'_, AppState>,
    channel: Channel<TaskEvent>,
    courses: Vec<CourseParam>,
    speed: f64,
    jobs: u32,
    notopen_action: String,
) -> Result<(), AppError> {
    let client_lock = state.client.read().await;
    let client = client_lock.as_ref().ok_or(AppError::Unauthorized)?;

    let (tx, mut rx) = mpsc::unbounded_channel::<TaskEvent>();

    // 将 mpsc 事件转发到 Tauri Channel
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = channel.send(event);
        }
    });

    let scheduler = TaskScheduler::new();

    let speed = speed.clamp(1.0, 2.0);
    let jobs = jobs.clamp(1, 8);

    // 从配置创建题库管理器
    let config = state.config.read().await;
    let tiku = TikuManager::from_config(&config);
    drop(config); // 释放读锁

    // 初始化题库（LIKE 知识库需要获取余额）
    tiku.init().await;

    // 如果需要检查 LLM 连接
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
        let _ = tx.send(TaskEvent::Log {
            level: "info".to_string(),
            message: format!("开始学习课程: {}", course.title),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });

        // 获取章节树
        let chapter_tree =
            get_course_point(client, &course.course_id, &course.clazz_id, &course.cpi).await?;

        let _ = tx.send(TaskEvent::CourseStarted {
            course_id: course.course_id.clone(),
            course_title: course.title.clone(),
            total_chapters: chapter_tree.points.len() as u32,
        });

        // 运行课程任务
        scheduler
            .run_course(
                client,
                &course.course_id,
                &course.clazz_id,
                &course.cpi,
                &chapter_tree.points,
                speed,
                jobs,
                &notopen_action,
                tiku_ref,
                &tx,
            )
            .await?;

        let _ = tx.send(TaskEvent::CourseCompleted {
            course_id: course.course_id.clone(),
            course_title: course.title.clone(),
        });
    }

    let _ = tx.send(TaskEvent::AllTasksCompleted);
    Ok(())
}

/// 暂停任务 (Phase 5 完善)
#[tauri::command]
pub async fn pause_tasks(_state: State<'_, AppState>) -> Result<(), AppError> {
    Ok(())
}

/// 恢复任务 (Phase 5 完善)
#[tauri::command]
pub async fn resume_tasks(_state: State<'_, AppState>) -> Result<(), AppError> {
    Ok(())
}

/// 取消任务 (Phase 5 完善)
#[tauri::command]
pub async fn cancel_tasks(_state: State<'_, AppState>) -> Result<(), AppError> {
    Ok(())
}
