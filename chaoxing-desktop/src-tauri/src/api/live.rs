//! 直播任务 API
//!
//! 对应 Python: api/live.py (Live 类)

use crate::api::client::HttpClient;
use crate::error::AppError;
use crate::models::job::Job;

const LIVE_REFERER: &str =
    "https://mooc1.chaoxing.com/ananas/modules/live/index.html?v=2022-1214-1139";

/// 获取直播状态（含总时长）
/// 对应 Python: Live.get_status()
pub async fn get_live_status(
    client: &HttpClient,
    job: &Job,
    course_id: &str,
    clazz_id: &str,
    knowledge_id: &str,
    user_id: &str,
) -> Result<Option<serde_json::Value>, AppError> {
    let live_id = job
        .property
        .get("liveId")
        .and_then(|v| v.as_str())
        .or_else(|| job.live_id.as_deref())
        .unwrap_or("");
    let jobid = job
        .property
        .get("_jobid")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if live_id.is_empty() || user_id.is_empty() || clazz_id.is_empty() || knowledge_id.is_empty()
    {
        tracing::error!("缺少直播状态查询必要参数");
        return Ok(None);
    }

    let url = format!(
        "https://mooc1.chaoxing.com/ananas/live/liveinfo?liveid={}&userid={}&clazzid={}&knowledgeid={}&courseid={}&jobid={}&ut=s",
        live_id, user_id, clazz_id, knowledge_id, course_id, jobid
    );

    let resp = client
        .client
        .get(&url)
        .header("Referer", LIVE_REFERER)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    if resp.status().is_success() {
        let json: serde_json::Value = resp.json().await?;
        Ok(Some(json))
    } else {
        tracing::error!("获取直播状态失败: {}", resp.status());
        Ok(None)
    }
}

/// 提交直播观看时长
/// 对应 Python: Live.do_finish()
pub async fn submit_live_time(
    client: &HttpClient,
    job: &Job,
    course_id: &str,
    user_id: &str,
) -> Result<bool, AppError> {
    let stream_name = job
        .property
        .get("streamName")
        .and_then(|v| v.as_str())
        .or_else(|| job.stream_name.as_deref())
        .unwrap_or("");
    let vdoid = job
        .property
        .get("vdoid")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if stream_name.is_empty() || vdoid.is_empty() || user_id.is_empty() {
        tracing::error!("缺少直播必要参数，无法提交时长");
        return Ok(false);
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let url = format!(
        "https://zhibo.chaoxing.com/saveTimePc?streamName={}&vdoid={}&userId={}&isStart=0&t={}&courseId={}",
        stream_name, vdoid, user_id, timestamp, course_id
    );

    let resp = client
        .client
        .get(&url)
        .header("Referer", LIVE_REFERER)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    if resp.status().is_success() {
        let text = resp.text().await?;
        let success = text.trim() == "@success";
        tracing::debug!("直播时长提交响应: {}", text);
        Ok(success)
    } else {
        tracing::error!("提交直播时长失败: {}", resp.status());
        Ok(false)
    }
}
