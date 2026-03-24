use tauri::State;

use crate::api::course;
use crate::error::AppError;
use crate::models::chapter::ChapterTree;
use crate::models::course::Course;
use crate::parser::course_point::parse_course_point;
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
    let url = format!(
        "https://mooc2-ans.chaoxing.com/mooc2-ans/mycourse/studentcourse?courseid={}&clazzid={}&cpi={}&ut=s",
        course_id, clazz_id, cpi
    );
    let resp = client.client.get(&url).send().await?;
    let html = resp.text().await?;
    parse_course_point(&html)
}
