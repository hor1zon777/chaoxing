//! 任务点列表解析 (course card / mArg)
//!
//! 精确复刻 Python decode.py 中 decode_course_card() 的逻辑，
//! 从页面内嵌 JavaScript 中提取 mArg JSON，解析出各种类型的任务。

use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::error::AppError;
use crate::models::job::{Job, JobInfo, JobType};

/// 缓存编译后的正则
static MARG_RE: OnceLock<Regex> = OnceLock::new();

fn marg_re() -> &'static Regex {
    MARG_RE.get_or_init(|| Regex::new(r"mArg=\{(.*?)\};").unwrap())
}

/// 解析任务点列表页面，提取任务列表和任务信息
///
/// 精确对应 Python decode_course_card() 的行为：
/// - 检查 "章节未开放"
/// - 从 HTML 中正则提取 mArg JSON 数据（先去空格再匹配）
/// - 解析 defaults 为 JobInfo
/// - 解析 attachments 为 Job 列表
///
/// 返回 (任务列表, 任务信息)
pub fn parse_course_card(html_text: &str) -> Result<(Vec<Job>, JobInfo), AppError> {
    // 检查章节是否未开放
    if html_text.contains("章节未开放") {
        return Ok((Vec::new(), JobInfo { not_open: true, ..Default::default() }));
    }

    // 去空格后正则提取 mArg JSON
    let cleaned = html_text.replace(' ', "");
    let caps = match marg_re().captures(&cleaned) {
        Some(c) => c,
        None => return Ok((Vec::new(), JobInfo::default())),
    };

    let json_str = format!("{{{}}}", &caps[1]);
    let cards_data: Value = serde_json::from_str(&json_str)
        .map_err(|e| AppError::Parse(format!("mArg JSON 解析失败: {}", e)))?;

    if cards_data.is_null() {
        return Ok((Vec::new(), JobInfo::default()));
    }

    // 提取任务基本信息 (defaults)
    let job_info = extract_job_info(&cards_data);

    // 处理所有附件任务
    let attachments = cards_data
        .get("attachments")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let job_list = process_attachment_cards(&attachments);

    Ok((job_list, job_info))
}

/// 从卡片数据中提取任务基本信息 (defaults)
fn extract_job_info(cards_data: &Value) -> JobInfo {
    let defaults = match cards_data.get("defaults") {
        Some(d) if !d.is_null() => d,
        _ => return JobInfo::default(),
    };

    JobInfo {
        ktoken: json_str(defaults, "ktoken"),
        mt_enc: json_str(defaults, "mtEnc"),
        report_time_interval: defaults
            .get("reportTimeInterval")
            .and_then(|v| v.as_u64())
            .unwrap_or(60) as u32,
        defenc: json_str(defaults, "defenc"),
        cardid: json_str(defaults, "cardid"),
        cpi: json_str(defaults, "cpi"),
        qnenc: json_str(defaults, "qnenc"),
        knowledgeid: json_str(defaults, "knowledgeid"),
        not_open: false,
    }
}

/// 处理所有附件任务卡片
fn process_attachment_cards(cards: &[Value]) -> Vec<Job> {
    let mut job_list = Vec::new();

    for card in cards {
        // 跳过已通过的任务
        if card.get("isPassed").and_then(|v| v.as_bool()).unwrap_or(false) {
            continue;
        }

        // 处理无 job 字段的特殊任务
        if card.get("job").is_none() || card.get("job") == Some(&Value::Null) {
            if let Some(read_job) = process_read_task(card) {
                job_list.push(read_job);
            }
            continue;
        }

        // 清理 otherInfo 字段中的无效参数（保留第一个 & 之前的部分）
        let other_info = card
            .get("otherInfo")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .split('&')
            .next()
            .unwrap_or("")
            .to_string();

        // 多维度判断任务类型
        let card_type = json_str(card, "type").to_lowercase();
        let property = card.get("property").cloned().unwrap_or(Value::Null);
        let prop_type = json_str(&property, "type").to_lowercase();
        let resource_type = json_str(&property, "resourceType").to_lowercase();

        // 直播任务特征
        let is_live = card_type.contains("live")
            || prop_type.contains("live")
            || resource_type.contains("live")
            || card_type.contains("livestream")
            || property.get("liveId").is_some_and(|v| !v.is_null())
            || property.get("streamName").is_some_and(|v| !v.is_null())
            || property.get("vdoid").is_some_and(|v| !v.is_null());

        if is_live {
            if let Some(job) = process_live_task(card, &other_info) {
                job_list.push(job);
            }
        } else if card_type == "video" {
            if let Some(job) = process_video_task(card, &other_info) {
                job_list.push(job);
            }
        } else if card_type == "document" {
            job_list.push(process_document_task(card, &other_info));
        } else if card_type == "workid" {
            job_list.push(process_work_task(card, &other_info));
        }
        // 未知类型静默跳过
    }

    job_list
}

/// 处理阅读类型任务
fn process_read_task(card: &Value) -> Option<Job> {
    let card_type = json_str(card, "type");
    if card_type != "read" {
        return None;
    }

    let property = card.get("property").cloned().unwrap_or(Value::Null);
    // 已阅读的跳过
    if property.get("read").and_then(|v| v.as_bool()).unwrap_or(false) {
        return None;
    }

    Some(Job {
        job_type: JobType::Read,
        jobid: json_str(card, "jobid"),
        name: json_str(&property, "title"),
        otherinfo: json_str(card, "otherInfo"),
        mid: json_str(card, "mid"),
        objectid: String::new(),
        aid: json_str(card, "aid"),
        jtoken: json_str(card, "jtoken"),
        enc: json_str(card, "enc"),
        play_time: 0,
        rt: String::new(),
        att_duration: String::new(),
        att_duration_enc: String::new(),
        video_face_capture_enc: String::new(),
        property: HashMap::new(),
        live_id: None,
        stream_name: None,
    })
}

/// 处理直播类型任务
fn process_live_task(card: &Value, other_info: &str) -> Option<Job> {
    let property = card.get("property").cloned().unwrap_or(Value::Null);
    let prop_map = value_to_map(&property);

    let name = if !json_str(&property, "title").is_empty() {
        json_str(&property, "title")
    } else {
        json_str(&property, "name")
    };

    let jobid = if !json_str(card, "jobid").is_empty() {
        json_str(card, "jobid")
    } else {
        json_str_or_number(card, "id")
    };

    Some(Job {
        job_type: JobType::Live,
        jobid,
        name,
        otherinfo: other_info.to_string(),
        mid: json_str(card, "mid"),
        objectid: json_str(card, "objectId"),
        aid: json_str(card, "aid"),
        jtoken: String::new(),
        enc: String::new(),
        play_time: 0,
        rt: String::new(),
        att_duration: String::new(),
        att_duration_enc: String::new(),
        video_face_capture_enc: String::new(),
        property: prop_map,
        live_id: property.get("liveId").and_then(|v| v.as_str()).map(String::from),
        stream_name: property
            .get("streamName")
            .and_then(|v| v.as_str())
            .map(String::from),
    })
}

/// 处理视频类型任务
fn process_video_task(card: &Value, other_info: &str) -> Option<Job> {
    // mid 是必须字段，如果不存在则跳过（转码失败的视频）
    let mid = json_str(card, "mid");
    if mid.is_empty() {
        return None;
    }

    let property = card.get("property").cloned().unwrap_or(Value::Null);

    Some(Job {
        job_type: JobType::Video,
        jobid: json_str(card, "jobid"),
        name: json_str(&property, "name"),
        otherinfo: other_info.to_string(),
        mid,
        objectid: json_str(card, "objectId"),
        aid: json_str(card, "aid"),
        jtoken: String::new(),
        enc: String::new(),
        play_time: card.get("playTime").and_then(|v| v.as_u64()).unwrap_or(0),
        rt: json_str(&property, "rt"),
        att_duration: json_str(card, "attDuration"),
        att_duration_enc: json_str(card, "attDurationEnc"),
        video_face_capture_enc: json_str(card, "videoFaceCaptureEnc"),
        property: HashMap::new(),
        live_id: None,
        stream_name: None,
    })
}

/// 处理文档类型任务
fn process_document_task(card: &Value, other_info: &str) -> Job {
    let property = card.get("property").cloned().unwrap_or(Value::Null);

    Job {
        job_type: JobType::Document,
        jobid: json_str(card, "jobid"),
        name: String::new(),
        otherinfo: other_info.to_string(),
        mid: json_str(card, "mid"),
        objectid: json_str(&property, "objectid"),
        aid: json_str(card, "aid"),
        jtoken: json_str(card, "jtoken"),
        enc: json_str(card, "enc"),
        play_time: 0,
        rt: String::new(),
        att_duration: String::new(),
        att_duration_enc: String::new(),
        video_face_capture_enc: String::new(),
        property: HashMap::new(),
        live_id: None,
        stream_name: None,
    }
}

/// 处理作业类型任务
fn process_work_task(card: &Value, other_info: &str) -> Job {
    Job {
        job_type: JobType::Work,
        jobid: json_str(card, "jobid"),
        name: String::new(),
        otherinfo: other_info.to_string(),
        mid: json_str(card, "mid"),
        objectid: String::new(),
        aid: json_str(card, "aid"),
        jtoken: String::new(),
        enc: json_str(card, "enc"),
        play_time: 0,
        rt: String::new(),
        att_duration: String::new(),
        att_duration_enc: String::new(),
        video_face_capture_enc: String::new(),
        property: HashMap::new(),
        live_id: None,
        stream_name: None,
    }
}

// ---- 辅助函数 ----

/// 从 JSON Value 中安全提取字符串
fn json_str(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// 从 JSON Value 中提取字符串，若字段为数字则转为字符串
fn json_str_or_number(value: &Value, key: &str) -> String {
    match value.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

/// 将 JSON Value 对象转为 HashMap<String, Value>
fn value_to_map(value: &Value) -> HashMap<String, Value> {
    match value.as_object() {
        Some(obj) => obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        None => HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_not_open() {
        let html = "<html><body>章节未开放</body></html>";
        let (jobs, info) = parse_course_card(html).unwrap();
        assert!(jobs.is_empty());
        assert!(info.not_open);
    }

    #[test]
    fn test_parse_no_marg() {
        let html = "<html><body>正常内容但没有mArg</body></html>";
        let (jobs, info) = parse_course_card(html).unwrap();
        assert!(jobs.is_empty());
        assert!(!info.not_open);
    }

    #[test]
    fn test_parse_video_task() {
        let html = r#"
        <script>
        mArg = {"attachments":[{"type":"video","jobid":"job001","mid":"mid001","objectId":"obj001","aid":"aid001","otherInfo":"info001&extra","playTime":120,"attDuration":"600","attDurationEnc":"enc001","videoFaceCaptureEnc":"vfc001","job":true,"property":{"name":"测试视频","rt":"0.9"}}],"defaults":{"ktoken":"kt001","mtEnc":"mt001","reportTimeInterval":60,"defenc":"de001","cardid":"card001","cpi":"cpi001","qnenc":"qn001","knowledgeid":"kn001"}};
        </script>
        "#;

        let (jobs, info) = parse_course_card(html).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].job_type, JobType::Video);
        assert_eq!(jobs[0].jobid, "job001");
        assert_eq!(jobs[0].mid, "mid001");
        assert_eq!(jobs[0].name, "测试视频");
        assert_eq!(jobs[0].otherinfo, "info001"); // & 后截断
        assert_eq!(jobs[0].play_time, 120);
        assert_eq!(jobs[0].att_duration, "600");

        assert_eq!(info.ktoken, "kt001");
        assert_eq!(info.mt_enc, "mt001");
        assert_eq!(info.report_time_interval, 60);
        assert_eq!(info.knowledgeid, "kn001");
    }

    #[test]
    fn test_skip_passed_task() {
        let html = r#"
        <script>
        mArg = {"attachments":[{"type":"video","jobid":"j1","mid":"m1","job":true,"isPassed":true,"property":{"name":"已完成"}}],"defaults":{}};
        </script>
        "#;

        let (jobs, _) = parse_course_card(html).unwrap();
        assert!(jobs.is_empty());
    }

    #[test]
    fn test_parse_work_task() {
        let html = r#"
        <script>
        mArg = {"attachments":[{"type":"workid","jobid":"w001","mid":"m001","aid":"a001","otherInfo":"winfo","enc":"wenc","job":true}],"defaults":{}};
        </script>
        "#;

        let (jobs, _) = parse_course_card(html).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].job_type, JobType::Work);
        assert_eq!(jobs[0].jobid, "w001");
        assert_eq!(jobs[0].enc, "wenc");
    }

    #[test]
    fn test_parse_live_task() {
        let html = r#"
        <script>
        mArg = {"attachments":[{"type":"live","jobid":"l001","mid":"m001","objectId":"o001","aid":"a001","otherInfo":"linfo","job":true,"property":{"title":"直播课","liveId":"live001","streamName":"stream001"}}],"defaults":{}};
        </script>
        "#;

        let (jobs, _) = parse_course_card(html).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].job_type, JobType::Live);
        assert_eq!(jobs[0].name, "直播课");
        assert_eq!(jobs[0].live_id, Some("live001".to_string()));
        assert_eq!(jobs[0].stream_name, Some("stream001".to_string()));
    }

    #[test]
    fn test_parse_document_task() {
        let html = r#"
        <script>
        mArg = {"attachments":[{"type":"document","jobid":"d001","mid":"m001","aid":"a001","otherInfo":"dinfo","jtoken":"jt001","enc":"de001","job":true,"property":{"objectid":"pobj001"}}],"defaults":{}};
        </script>
        "#;

        let (jobs, _) = parse_course_card(html).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].job_type, JobType::Document);
        assert_eq!(jobs[0].objectid, "pobj001");
        assert_eq!(jobs[0].jtoken, "jt001");
    }

    #[test]
    fn test_parse_read_task() {
        let html = r#"
        <script>
        mArg = {"attachments":[{"type":"read","jobid":"r001","mid":"m001","aid":"a001","otherInfo":"rinfo","jtoken":"jt001","enc":"re001","property":{"title":"阅读材料","read":false,"id":"rid001"}}],"defaults":{}};
        </script>
        "#;

        let (jobs, _) = parse_course_card(html).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].job_type, JobType::Read);
        assert_eq!(jobs[0].name, "阅读材料");
    }

    #[test]
    fn test_skip_already_read() {
        let html = r#"
        <script>
        mArg = {"attachments":[{"type":"read","jobid":"r001","mid":"m001","property":{"title":"已读","read":true}}],"defaults":{}};
        </script>
        "#;

        let (jobs, _) = parse_course_card(html).unwrap();
        assert!(jobs.is_empty());
    }
}
