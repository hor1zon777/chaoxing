//! 章节知识点列表 API
//!
//! 对应 Python base.py get_course_point()

use crate::api::client::HttpClient;
use crate::error::AppError;
use crate::models::chapter::ChapterTree;
use crate::parser::course_point::parse_course_point;

/// 获取课程章节知识点列表
pub async fn get_course_point(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    cpi: &str,
) -> Result<ChapterTree, AppError> {
    let url = format!(
        "https://mooc2-ans.chaoxing.com/mooc2-ans/mycourse/studentcourse?courseid={}&clazzid={}&cpi={}&ut=s",
        course_id, clazz_id, cpi
    );
    let resp = client.client.get(&url).send().await?;
    let html = resp.text().await?;
    parse_course_point(&html)
}
