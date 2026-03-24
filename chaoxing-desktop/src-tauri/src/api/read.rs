//! 阅读任务 API
//!
//! 对应 Python base.py study_read()

use crate::api::client::HttpClient;
use crate::error::AppError;
use crate::models::job::{Job, JobInfo};
use crate::models::video::StudyResult;

/// 学习阅读任务
pub async fn study_read(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    job: &Job,
    job_info: &JobInfo,
) -> Result<StudyResult, AppError> {
    let params = [
        ("jobid", job.jobid.as_str()),
        ("knowledgeid", job_info.knowledgeid.as_str()),
        ("jtoken", job.jtoken.as_str()),
        ("courseid", course_id),
        ("clazzid", clazz_id),
    ];

    let resp = client
        .client
        .get("https://mooc1.chaoxing.com/ananas/job/readv2")
        .query(&params)
        .send()
        .await?;

    if resp.status() != 200 {
        tracing::error!("阅读任务失败: {}", resp.status());
        return Ok(StudyResult::Error);
    }

    let json: serde_json::Value = resp.json().await?;
    tracing::info!(
        "阅读任务: {}",
        json["msg"].as_str().unwrap_or("")
    );
    Ok(StudyResult::Success)
}
