//! 任务点列表 API
//!
//! 对应 Python base.py get_job_list()
//! 通过 num 参数 0-6 循环请求，合并所有任务卡片

use crate::api::client::HttpClient;
use crate::error::AppError;
use crate::models::job::{Job, JobInfo};
use crate::parser::course_card::parse_course_card;

/// 获取章节任务点列表
///
/// 关键逻辑：num 从 0 到 6 循环请求，合并所有返回的任务
pub async fn get_job_list(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    knowledge_id: &str,
    cpi: &str,
) -> Result<(Vec<Job>, JobInfo), AppError> {
    client.rate_limiter.limit_rate().await;

    let mut all_jobs = Vec::new();
    let mut merged_info = JobInfo::default();

    // 用调用方传入的章节 ID 和课程 cpi 作为默认值
    // （mArg defaults 中这两个字段未必有，Python 原版的 study_work 直接信任传入值）
    merged_info.knowledgeid = knowledge_id.to_string();
    merged_info.cpi = cpi.to_string();

    for num in 0..=6 {
        let num_str = num.to_string();
        let params = [
            ("clazzid", clazz_id),
            ("courseid", course_id),
            ("knowledgeid", knowledge_id),
            ("ut", "s"),
            ("cpi", cpi),
            ("v", "2025-0424-1038-3"),
            ("mooc2", "1"),
            ("num", &num_str),
        ];

        let resp = client
            .client
            .get("https://mooc1.chaoxing.com/mooc-ans/knowledge/cards")
            .query(&params)
            .send()
            .await?;

        if resp.status() != 200 {
            tracing::error!("获取任务点失败: {}", resp.status());
            return Ok((vec![], JobInfo::default()));
        }

        let html = resp.text().await?;
        let (jobs, info) = parse_course_card(&html)?;

        if info.not_open {
            tracing::info!("该章节未开放");
            return Ok((vec![], info));
        }

        all_jobs.extend(jobs);

        // 合并 info：后面的覆盖前面的非空字段
        if !info.ktoken.is_empty() {
            merged_info.ktoken = info.ktoken;
        }
        if !info.knowledgeid.is_empty() {
            merged_info.knowledgeid = info.knowledgeid;
        }
        if !info.cpi.is_empty() {
            merged_info.cpi = info.cpi;
        }
        if !info.cardid.is_empty() {
            merged_info.cardid = info.cardid;
        }
        if !info.mt_enc.is_empty() {
            merged_info.mt_enc = info.mt_enc;
        }
        if !info.defenc.is_empty() {
            merged_info.defenc = info.defenc;
        }
        if !info.qnenc.is_empty() {
            merged_info.qnenc = info.qnenc;
        }
        merged_info.not_open = info.not_open;
    }

    Ok((all_jobs, merged_info))
}
