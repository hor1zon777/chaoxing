//! 任务点列表 API
//!
//! 对应 Python base.py get_job_list()
//! 通过 num 参数 0-6 循环请求，合并所有返回的任务卡片。
//!
//! 两个公开入口：
//! - `get_job_list`：用于任务执行流程，受全局 rate_limiter 限速
//! - `get_job_list_fast`：用于选课页面预加载，绕过 rate_limiter 并发拉取 7 个 num，
//!   依赖调用方自行控制章节级 Semaphore 限速

use tokio::task::JoinSet;

use crate::api::client::HttpClient;
use crate::error::AppError;
use crate::models::job::{Job, JobInfo};
use crate::parser::course_card::parse_course_card;

/// 获取章节任务点列表（任务执行专用，受 rate_limiter 限速）
pub async fn get_job_list(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    knowledge_id: &str,
    cpi: &str,
) -> Result<(Vec<Job>, JobInfo), AppError> {
    client.rate_limiter.limit_rate().await;
    fetch_all_cards_serial(client, course_id, clazz_id, knowledge_id, cpi).await
}

/// 获取章节任务点列表（选课预加载专用，绕过 rate_limiter，7 个 num 并发）
pub async fn get_job_list_fast(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    knowledge_id: &str,
    cpi: &str,
) -> Result<(Vec<Job>, JobInfo), AppError> {
    let mut merged_info = JobInfo {
        knowledgeid: knowledge_id.to_string(),
        cpi: cpi.to_string(),
        ..Default::default()
    };

    let mut set: JoinSet<Result<(u8, Vec<Job>, JobInfo), AppError>> = JoinSet::new();
    for num in 0..=6u8 {
        let client = client.clone();
        let course_id = course_id.to_string();
        let clazz_id = clazz_id.to_string();
        let knowledge_id = knowledge_id.to_string();
        let cpi = cpi.to_string();
        set.spawn(async move {
            let (jobs, info) =
                fetch_single_card(&client, &course_id, &clazz_id, &knowledge_id, &cpi, num).await?;
            Ok((num, jobs, info))
        });
    }

    // 收集后按 num 排序，保证合并顺序确定
    let mut results: Vec<(u8, Vec<Job>, JobInfo)> = Vec::with_capacity(7);
    while let Some(joined) = set.join_next().await {
        let triple = joined.map_err(|e| AppError::Other(format!("join 失败: {}", e)))??;
        results.push(triple);
    }
    results.sort_by_key(|(n, _, _)| *n);

    let mut all_jobs = Vec::new();
    for (_, jobs, info) in results {
        if info.not_open {
            return Ok((vec![], info));
        }
        all_jobs.extend(jobs);
        merge_info(&mut merged_info, info);
    }
    Ok((all_jobs, merged_info))
}

async fn fetch_all_cards_serial(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    knowledge_id: &str,
    cpi: &str,
) -> Result<(Vec<Job>, JobInfo), AppError> {
    let mut merged_info = JobInfo {
        knowledgeid: knowledge_id.to_string(),
        cpi: cpi.to_string(),
        ..Default::default()
    };

    let mut all_jobs = Vec::new();
    for num in 0..=6u8 {
        let (jobs, info) =
            fetch_single_card(client, course_id, clazz_id, knowledge_id, cpi, num).await?;
        if info.not_open {
            return Ok((vec![], info));
        }
        all_jobs.extend(jobs);
        merge_info(&mut merged_info, info);
    }
    Ok((all_jobs, merged_info))
}

async fn fetch_single_card(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    knowledge_id: &str,
    cpi: &str,
    num: u8,
) -> Result<(Vec<Job>, JobInfo), AppError> {
    let num_str = num.to_string();
    let params = [
        ("clazzid", clazz_id),
        ("courseid", course_id),
        ("knowledgeid", knowledge_id),
        ("ut", "s"),
        ("cpi", cpi),
        ("v", "2025-0424-1038-3"),
        ("mooc2", "1"),
        ("num", num_str.as_str()),
    ];

    let resp = client
        .client
        .get("https://mooc1.chaoxing.com/mooc-ans/knowledge/cards")
        .query(&params)
        .send()
        .await?;

    if resp.status() != 200 {
        tracing::error!("获取任务点失败: {} (num={})", resp.status(), num);
        return Ok((vec![], JobInfo::default()));
    }

    let html = resp.text().await?;
    parse_course_card(&html)
}

fn merge_info(merged: &mut JobInfo, info: JobInfo) {
    if !info.ktoken.is_empty() {
        merged.ktoken = info.ktoken;
    }
    if !info.knowledgeid.is_empty() {
        merged.knowledgeid = info.knowledgeid;
    }
    if !info.cpi.is_empty() {
        merged.cpi = info.cpi;
    }
    if !info.cardid.is_empty() {
        merged.cardid = info.cardid;
    }
    if !info.mt_enc.is_empty() {
        merged.mt_enc = info.mt_enc;
    }
    if !info.defenc.is_empty() {
        merged.defenc = info.defenc;
    }
    if !info.qnenc.is_empty() {
        merged.qnenc = info.qnenc;
    }
    merged.not_open = info.not_open;
}

