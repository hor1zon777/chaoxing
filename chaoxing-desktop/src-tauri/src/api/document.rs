//! 文档任务 API
//!
//! 对应 Python base.py study_document()
//! 简单 GET 请求即可完成

use std::sync::OnceLock;

use regex::Regex;

use crate::api::client::HttpClient;
use crate::error::AppError;
use crate::models::job::Job;
use crate::models::video::StudyResult;

/// 缓存 nodeId 正则
static NODE_ID_RE: OnceLock<Regex> = OnceLock::new();

fn node_id_regex() -> &'static Regex {
    NODE_ID_RE.get_or_init(|| Regex::new(r"nodeId_(.*?)-").unwrap())
}

/// 学习文档任务
pub async fn study_document(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    job: &Job,
) -> Result<StudyResult, AppError> {
    let knowledge_id = node_id_regex()
        .captures(&job.otherinfo)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .unwrap_or("");

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let url = format!(
        "https://mooc1.chaoxing.com/ananas/job/document?jobid={}&knowledgeid={}&courseid={}&clazzid={}&jtoken={}&_dc={}",
        job.jobid, knowledge_id, course_id, clazz_id, job.jtoken, timestamp
    );

    let resp = client.client.get(&url).send().await?;

    if resp.status() == 200 {
        tracing::info!("文档任务完成: {}", job.jobid);
        Ok(StudyResult::Success)
    } else {
        tracing::error!("文档任务失败: {}", resp.status());
        Ok(StudyResult::Error)
    }
}
