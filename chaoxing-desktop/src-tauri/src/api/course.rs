//! 课程列表获取 API
//!
//! 精确复刻 Python api/base.py 中 Chaoxing.get_course_list() 的逻辑，
//! 包括主列表和文件夹内课程的合并获取。

use crate::api::client::HttpClient;
use crate::error::AppError;
use crate::models::course::Course;
use crate::parser::course_folder::parse_course_folder;
use crate::parser::course_list::parse_course_list;

/// 获取所有课程列表（含文件夹内课程）
/// 对应 Python: Chaoxing.get_course_list()
pub async fn get_course_list(client: &HttpClient) -> Result<Vec<Course>, AppError> {
    client.rate_limiter.limit_rate().await;

    let params = [
        ("courseType", "1"),
        ("courseFolderId", "0"),
        ("query", ""),
        ("superstarClass", "0"),
    ];

    // 获取主课程列表
    let resp = client
        .client
        .post("https://mooc2-ans.chaoxing.com/mooc2-ans/visit/courselistdata")
        .form(&params)
        .header(
            "Referer",
            "https://mooc2-ans.chaoxing.com/mooc2-ans/visit/interaction\
             ?moocDomain=https://mooc1-1.chaoxing.com/mooc-ans",
        )
        .send()
        .await?;

    let html = resp.text().await?;
    let mut course_list = parse_course_list(&html)?;

    // 获取课程文件夹列表
    let interaction_resp = client
        .client
        .get("https://mooc2-ans.chaoxing.com/mooc2-ans/visit/interaction")
        .send()
        .await?;
    let interaction_html = interaction_resp.text().await?;
    let folders = parse_course_folder(&interaction_html)?;

    // 获取文件夹内的课程
    for folder in &folders {
        client.rate_limiter.limit_rate().await;

        let folder_params = [
            ("courseType", "1"),
            ("courseFolderId", folder.id.as_str()),
            ("query", ""),
            ("superstarClass", "0"),
        ];

        let folder_resp = client
            .client
            .post("https://mooc2-ans.chaoxing.com/mooc2-ans/visit/courselistdata")
            .form(&folder_params)
            .send()
            .await?;

        let folder_html = folder_resp.text().await?;
        let folder_courses = parse_course_list(&folder_html)?;
        course_list.extend(folder_courses);
    }

    Ok(course_list)
}
