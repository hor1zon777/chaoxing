use tauri::State;

use crate::api::{course, course_card, course_point};
use crate::error::AppError;
use crate::models::chapter::ChapterTree;
use crate::models::course::{ChapterSelectionPoint, Course, CourseJob, CourseSelectionTree};
use crate::models::job::Job;
use crate::state::AppState;

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
#[tauri::command]
pub async fn get_course_selection_tree(
    state: State<'_, AppState>,
    course_id: String,
    clazz_id: String,
    cpi: String,
) -> Result<CourseSelectionTree, AppError> {
    let lock = state.client.read().await;
    let client = lock.as_ref().ok_or(AppError::Unauthorized)?;

    let chapter_tree = course_point::get_course_point(client, &course_id, &clazz_id, &cpi).await?;
    let mut points = Vec::with_capacity(chapter_tree.points.len());

    for point in chapter_tree.points {
        let (jobs, _) = course_card::get_job_list(client, &course_id, &clazz_id, &point.id, &cpi).await?;
        points.push(ChapterSelectionPoint {
            id: point.id,
            title: point.title,
            job_count: jobs.len() as u32,
            has_finished: point.has_finished,
            need_unlock: point.need_unlock,
            jobs: convert_jobs(jobs),
        });
    }

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
