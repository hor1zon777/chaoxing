//! 视频/音频任务 API
//!
//! 精确复刻 Python base.py study_video() 和 video_progress_log()
//!
//! 关键逻辑：
//! - 获取视频信息后先尝试直接完成（上报当前进度 + 上报满进度）
//! - 未通过时进入模拟播放循环：每 30-90 秒随机间隔上报一次
//! - 进度计算使用真实时间差 * speed
//! - enc 签名用 get_video_enc()
//! - rt 参数：优先从 job.rt 获取，其次从 otherinfo 正则匹配
//! - 403 处理：刷新视频状态并重试，最多 2 次

use std::sync::OnceLock;
use std::time::Duration;

use rand::Rng;
use regex::Regex;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::api::client::HttpClient;
use crate::crypto::enc::get_video_enc;
use crate::error::AppError;
use crate::models::events::TaskEvent;
use crate::models::job::{Job, JobInfo};
use crate::models::video::StudyResult;

/// 循环中每次 sleep 的间隔秒数
const LOOP_INTERVAL: f64 = 1.0;

/// 模拟播放循环的最大运行时间（秒），防止服务器永不返回 isPassed 时无限循环
const MAX_PLAY_LOOP_SECONDS: u64 = 7200;

/// 缓存 rt 正则
static RT_RE: OnceLock<Regex> = OnceLock::new();

fn rt_regex() -> &'static Regex {
    RT_RE.get_or_init(|| Regex::new(r"-rt_([1d])").unwrap())
}

/// 从 job 中解析 rt 参数
///
/// 优先使用 job.rt 字段，其次从 otherinfo 正则匹配 `-rt_([1d])`
/// d -> "0.9", 1 -> "1"
fn resolve_rt(job: &Job) -> String {
    if !job.rt.is_empty() {
        return job.rt.clone();
    }
    rt_regex()
        .captures(&job.otherinfo)
        .map(|caps| {
            let c = caps.get(1).unwrap().as_str();
            if c == "d" {
                "0.9".to_string()
            } else {
                "1".to_string()
            }
        })
        .unwrap_or_default()
}

/// 获取当前毫秒时间戳
fn timestamp_millis() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}

/// 根据媒体类型返回对应的 Referer
fn media_referer(media_type: &str) -> &'static str {
    if media_type == "Video" {
        "https://mooc1.chaoxing.com/ananas/modules/video/index.html?v=2025-0725-1842"
    } else {
        "https://mooc1.chaoxing.com/ananas/modules/audio/index_new.html?v=2025-0725-1842"
    }
}

/// 视频进度上报
///
/// 对应 Python: Chaoxing.video_progress_log()
/// 返回 (is_passed, http_status_code)
async fn video_progress_log(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    cpi: &str,
    job: &Job,
    _job_info: &JobInfo,
    dtoken: &str,
    duration: u64,
    playing_time: u64,
    media_type: &str,
) -> Result<(bool, u16), AppError> {
    client.video_rate_limiter.limit_rate().await;

    let delay = rand::thread_rng().gen_range(0.0..2.0);
    tokio::time::sleep(Duration::from_secs_f64(delay)).await;

    let uid = client.get_uid().unwrap_or_default();
    let enc = get_video_enc(clazz_id, &job.jobid, &job.objectid, playing_time, duration, &uid);
    let referer = media_referer(media_type);
    let resolved_rt = resolve_rt(job);
    let url = format!(
        "https://mooc1.chaoxing.com/mooc-ans/multimedia/log/a/{}/{}",
        cpi, dtoken
    );

    let build_params = |rt: Option<&str>| {
        let mut params = vec![
            ("clazzId", clazz_id.to_string()),
            ("playingTime", playing_time.to_string()),
            ("duration", duration.to_string()),
            ("clipTime", format!("0_{}", duration)),
            ("objectId", job.objectid.clone()),
            ("otherInfo", job.otherinfo.clone()),
            ("courseId", course_id.to_string()),
            ("jobid", job.jobid.clone()),
            ("userid", uid.clone()),
            ("isdrag", "3".to_string()),
            ("view", "pc".to_string()),
            ("enc", enc.clone()),
            ("dtype", media_type.to_string()),
        ];

        if !job.video_face_capture_enc.is_empty() {
            params.push(("videoFaceCaptureEnc", job.video_face_capture_enc.clone()));
        }
        if !job.att_duration.is_empty() {
            params.push(("attDuration", job.att_duration.clone()));
        }
        if !job.att_duration_enc.is_empty() {
            params.push(("attDurationEnc", job.att_duration_enc.clone()));
        }
        if let Some(value) = rt {
            params.push(("rt", value.to_string()));
            params.push(("_t", timestamp_millis()));
        }

        params
    };

    let try_with_rt = async |rt: Option<&str>| -> Result<(bool, u16), AppError> {
        let resp = client
            .client
            .get(&url)
            .query(&build_params(rt))
            .header("Referer", referer)
            .send()
            .await?;

        let status = resp.status().as_u16();
        if status == 200 {
            let json: serde_json::Value = resp.json().await?;
            let is_passed = json["isPassed"].as_bool().unwrap_or(false);
            Ok((is_passed, 200))
        } else if status == 403 {
            tracing::warn!("视频进度上报返回 403");
            Ok((false, 403))
        } else {
            tracing::error!("视频进度上报未知错误: {}", status);
            Ok((false, status))
        }
    };

    if !resolved_rt.is_empty() {
        return try_with_rt(Some(&resolved_rt)).await;
    }

    for fallback_rt in ["0.9", "1"] {
        let result = try_with_rt(Some(fallback_rt)).await?;
        if result.1 == 200 {
            return Ok(result);
        }
        if result.1 != 403 {
            return Ok(result);
        }
    }

    Ok((false, 403))
}

/// 获取视频状态信息
///
/// 返回 (dtoken, duration) 或 None
async fn fetch_video_info(
    client: &HttpClient,
    object_id: &str,
    fid: &str,
    referer: &str,
) -> Result<Option<(String, u64, serde_json::Value)>, AppError> {
    let info_url = format!(
        "https://mooc1.chaoxing.com/ananas/status/{}?k={}&flag=normal",
        object_id, fid
    );

    let resp = client
        .client
        .get(&info_url)
        .header("Referer", referer)
        .send()
        .await?;

    let video_info: serde_json::Value = resp.json().await?;

    if video_info["status"].as_str() != Some("success") {
        tracing::error!("视频状态异常: {:?}", video_info["status"]);
        return Ok(None);
    }

    let dtoken = video_info["dtoken"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let duration = video_info["duration"].as_u64().unwrap_or(0);

    Ok(Some((dtoken, duration, video_info)))
}

/// 学习视频任务
///
/// 对应 Python: Chaoxing.study_video()
///
/// 流程：
/// 1. 获取视频信息（dtoken, duration）
/// 2. 先尝试直接完成（上报当前进度 + 满进度）
/// 3. 未通过则进入模拟播放循环
/// 4. 循环中每 30-90 秒上报一次进度
/// 5. 遇到 403 时刷新 dtoken 并重试
pub async fn study_video(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    cpi: &str,
    job: &Job,
    job_info: &JobInfo,
    speed: f64,
    media_type: &str,
    event_tx: Option<&tokio::sync::mpsc::UnboundedSender<TaskEvent>>,
    is_running: &Arc<AtomicBool>,
    is_paused: &Arc<AtomicBool>,
) -> Result<StudyResult, AppError> {
    let fid = client.get_fid().unwrap_or_default();
    let referer = media_referer(media_type);

    // 1. 获取视频信息
    let (dtoken, duration, _video_info) =
        match fetch_video_info(client, &job.objectid, &fid, referer).await? {
            Some(info) => info,
            None => return Ok(StudyResult::Error),
        };

    // playTime 在 Python 中是毫秒，转换为秒
    let mut play_time = job.play_time / 1000;

    tracing::info!(
        "开始任务: {}, 总时长: {}s, 已进行: {}s",
        job.name,
        duration,
        play_time
    );

    // 2. 先尝试直接完成
    let (passed1, _) = video_progress_log(
        client, course_id, clazz_id, cpi, job, job_info, &dtoken, duration, play_time, media_type,
    )
    .await?;

    let (passed2, _) = video_progress_log(
        client, course_id, clazz_id, cpi, job, job_info, &dtoken, duration, duration, media_type,
    )
    .await?;

    if passed1 || passed2 {
        tracing::info!("任务瞬间完成: {}", job.name);
        return Ok(StudyResult::Success);
    }

    // 3. 进入模拟播放循环
    let mut last_log_time: u64 = 0;
    let mut wait_time = rand::thread_rng().gen_range(30..=90) as u64;
    let mut forbidden_retry = 0u32;
    let max_forbidden_retry = 2u32;
    let mut current_dtoken = dtoken;
    let loop_start = tokio::time::Instant::now();

    loop {
        // 安全上限：防止服务器永不返回 isPassed 时无限循环
        if loop_start.elapsed().as_secs() > MAX_PLAY_LOOP_SECONDS {
            tracing::warn!("视频播放循环超过安全时限 ({}s)，强制退出: {}", MAX_PLAY_LOOP_SECONDS, job.name);
            return Ok(StudyResult::Error);
        }
        if !is_running.load(Ordering::SeqCst) {
            tracing::info!("视频任务已取消: {}", job.name);
            return Ok(StudyResult::Cancelled);
        }

        while is_paused.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if !is_running.load(Ordering::SeqCst) {
                tracing::info!("视频任务已取消: {}", job.name);
                return Ok(StudyResult::Cancelled);
            }
        }

        // 已到达满时长：只做完成检查，不再模拟播放
        if play_time == duration {
            let (passed, status) = video_progress_log(
                client, course_id, clazz_id, cpi, job, job_info,
                &current_dtoken, duration, play_time, media_type,
            )
            .await?;

            if passed {
                tracing::info!("任务完成: {}", job.name);
                return Ok(StudyResult::Success);
            }

            if status == 403 {
                if forbidden_retry >= max_forbidden_retry {
                    tracing::warn!("403 重试上限，跳过任务");
                    return Ok(StudyResult::Forbidden);
                }
                forbidden_retry += 1;
                tracing::warn!("出现 403，尝试恢复 (第 {} 次)", forbidden_retry);

                let retry_delay = rand::thread_rng().gen_range(2.0..4.0);
                tokio::time::sleep(Duration::from_secs_f64(retry_delay)).await;

                if let Ok(Some((new_dtoken, _, _))) =
                    fetch_video_info(client, &job.objectid, &fid, referer).await
                {
                    current_dtoken = new_dtoken;
                }
                continue;
            } else if status != 200 {
                return Ok(StudyResult::Error);
            }

            // 服务器尚未确认完成，等待后重试
            let retry_delay = rand::thread_rng().gen_range(10u64..=20);
            for _ in 0..retry_delay {
                if !is_running.load(Ordering::SeqCst) {
                    tracing::info!("视频任务已取消: {}", job.name);
                    return Ok(StudyResult::Cancelled);
                }
                while is_paused.load(Ordering::SeqCst) {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    if !is_running.load(Ordering::SeqCst) {
                        tracing::info!("视频任务已取消: {}", job.name);
                        return Ok(StudyResult::Cancelled);
                    }
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            continue;
        }

        // 到达上报间隔（saturating 避免未来 reset play_time 时溢出 panic）
        if play_time.saturating_sub(last_log_time) >= wait_time {
            let (passed, status) = video_progress_log(
                client, course_id, clazz_id, cpi, job, job_info,
                &current_dtoken, duration, play_time, media_type,
            )
            .await?;

            if passed {
                tracing::info!("任务完成: {}", job.name);
                return Ok(StudyResult::Success);
            }

            if status == 403 {
                if forbidden_retry >= max_forbidden_retry {
                    tracing::warn!("403 重试上限，跳过任务");
                    return Ok(StudyResult::Forbidden);
                }
                forbidden_retry += 1;
                tracing::warn!("出现 403，尝试恢复 (第 {} 次)", forbidden_retry);

                let retry_delay = rand::thread_rng().gen_range(2.0..4.0);
                tokio::time::sleep(Duration::from_secs_f64(retry_delay)).await;

                if let Ok(Some((new_dtoken, _, _))) =
                    fetch_video_info(client, &job.objectid, &fid, referer).await
                {
                    current_dtoken = new_dtoken;
                }
                continue;
            } else if status != 200 {
                return Ok(StudyResult::Error);
            }

            wait_time = rand::thread_rng().gen_range(30..=90);
            last_log_time = play_time;
        }

        play_time = ((play_time as f64) + LOOP_INTERVAL * speed)
            .min(duration as f64) as u64;

        if let Some(tx) = event_tx {
            let _ = tx.send(TaskEvent::VideoProgress {
                course_id: course_id.to_string(),
                job_id: job.jobid.clone(),
                job_name: job.name.clone(),
                current_time: play_time,
                total_duration: duration,
            });
        }

        tokio::time::sleep(Duration::from_secs_f64(LOOP_INTERVAL)).await;
    }
}
