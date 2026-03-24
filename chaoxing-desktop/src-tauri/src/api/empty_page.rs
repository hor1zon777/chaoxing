//! 空页面任务 API
//!
//! 对应 Python base.py study_emptypage()

use crate::api::client::HttpClient;
use crate::error::AppError;
use crate::models::video::StudyResult;

/// 空页面任务学习
///
/// 对于没有任务点的章节，发送一个 studentstudyAjax 请求标记已访问
pub async fn study_emptypage(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    chapter_id: &str,
    cpi: &str,
) -> Result<StudyResult, AppError> {
    let params = [
        ("courseId", course_id),
        ("clazzid", clazz_id),
        ("chapterId", chapter_id),
        ("cpi", cpi),
        ("verificationcode", ""),
        ("mooc2", "1"),
        ("microTopicId", "0"),
        ("editorPreview", "0"),
    ];

    let resp = client
        .client
        .get("https://mooc1.chaoxing.com/mooc-ans/mycourse/studentstudyAjax")
        .query(&params)
        .send()
        .await?;

    if resp.status() == 200 {
        tracing::info!("空页面任务完成: chapter_id={}", chapter_id);
        Ok(StudyResult::Success)
    } else {
        tracing::error!("空页面任务失败: {}", resp.status());
        Ok(StudyResult::Error)
    }
}
