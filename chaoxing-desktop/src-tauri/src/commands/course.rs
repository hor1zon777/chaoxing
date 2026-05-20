use std::sync::Arc;

use tauri::State;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::api::{course, course_card, course_point};
use crate::error::AppError;
use crate::models::chapter::ChapterTree;
use crate::models::course::{ChapterSelectionPoint, Course, CourseJob, CourseSelectionTree};
use crate::models::job::Job;
use crate::state::AppState;

/// 章节并发预加载上限（兼顾速度与服务端压力）
const CHAPTER_PRELOAD_CONCURRENCY: usize = 8;

/// 获取课程列表
#[tauri::command]
pub async fn get_courses(state: State<'_, AppState>) -> Result<Vec<Course>, AppError> {
    let lock = state.client.read().await;
    let client = lock.as_ref().ok_or(AppError::Unauthorized)?;
    course::get_course_list(client).await
}

/// 获取章节树
#[tauri::command]
pub async fn get_chapter_tree(
    state: State<'_, AppState>,
    course_id: String,
    clazz_id: String,
    cpi: String,
) -> Result<ChapterTree, AppError> {
    let lock = state.client.read().await;
    let client = lock.as_ref().ok_or(AppError::Unauthorized)?;
    course_point::get_course_point(client, &course_id, &clazz_id, &cpi).await
}

/// 获取课程选择树（章节 + 任务点）
///
/// 性能优化：章节维度用 `Semaphore` 控制并发拉取（默认 8），
/// 每个章节内 7 个 num 也并发（`get_job_list_fast`），
/// 总请求数不变但并发执行，30 章节通常从 100s+ 降到几秒。
#[tauri::command]
pub async fn get_course_selection_tree(
    state: State<'_, AppState>,
    course_id: String,
    clazz_id: String,
    cpi: String,
) -> Result<CourseSelectionTree, AppError> {
    let client = {
        let lock = state.client.read().await;
        lock.as_ref().ok_or(AppError::Unauthorized)?.clone()
    };

    let chapter_tree =
        course_point::get_course_point(&client, &course_id, &clazz_id, &cpi).await?;

    let semaphore = Arc::new(Semaphore::new(CHAPTER_PRELOAD_CONCURRENCY));
    let course_id = Arc::new(course_id);
    let clazz_id = Arc::new(clazz_id);
    let cpi = Arc::new(cpi);

    let mut set: JoinSet<Result<(usize, ChapterSelectionPoint), AppError>> = JoinSet::new();

    for (idx, point) in chapter_tree.points.into_iter().enumerate() {
        let semaphore = semaphore.clone();
        let client = client.clone();
        let course_id = course_id.clone();
        let clazz_id = clazz_id.clone();
        let cpi = cpi.clone();

        set.spawn(async move {
            let _permit = semaphore
                .acquire_owned()
                .await
                .map_err(|e| AppError::Other(format!("信号量获取失败: {}", e)))?;
            let (jobs, _) = course_card::get_job_list_fast(
                &client,
                &course_id,
                &clazz_id,
                &point.id,
                &cpi,
            )
            .await?;
            Ok((
                idx,
                ChapterSelectionPoint {
                    id: point.id,
                    title: point.title,
                    job_count: jobs.len() as u32,
                    has_finished: point.has_finished,
                    need_unlock: point.need_unlock,
                    jobs: convert_jobs(jobs),
                },
            ))
        });
    }

    // 收集并按原章节顺序排序
    let mut collected: Vec<(usize, ChapterSelectionPoint)> = Vec::new();
    while let Some(joined) = set.join_next().await {
        let pair = joined.map_err(|e| AppError::Other(format!("join 失败: {}", e)))??;
        collected.push(pair);
    }
    collected.sort_by_key(|(idx, _)| *idx);
    let points = collected.into_iter().map(|(_, p)| p).collect();

    Ok(CourseSelectionTree {
        has_locked: chapter_tree.has_locked,
        points,
    })
}

fn convert_jobs(jobs: Vec<Job>) -> Vec<CourseJob> {
    jobs.into_iter()
        .map(|job| CourseJob {
            id: job.jobid,
            name: if job.name.is_empty() {
                format!("{}任务", job.job_type.label())
            } else {
                job.name
            },
            type_label: job.job_type.label().to_string(),
            job_type: job.job_type,
            is_completed: job.is_completed,
        })
        .collect()
}
