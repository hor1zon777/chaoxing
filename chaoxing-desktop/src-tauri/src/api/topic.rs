//! 讨论区话题 API（groupyd 接口）
//!
//! - 发帖：`POST https://groupyd.chaoxing.com/apis/topic/addTopic`
//! - 列话题：`GET https://groupyd.chaoxing.com/apis/topic/getTopicListWithPoff`
//!
//! 查询参数经 [`crate::crypto::inf_enc`] 签名；`bbsid` 由 courseId 派生（MD5）。
//! 响应为 GBK 编码，统一经 [`crate::utils::decode::decode_cx_bytes`] 解码。
//!
//! 仅用于用户自有、已授权账号下的合规调用。`add_topic` 为写操作，会真实发布话题。

use serde_json::Value;

use crate::api::client::HttpClient;
use crate::crypto::inf_enc::{bbsid_from_course_id, new_c0, sign_query, INF_ENC_TOKEN};
use crate::error::AppError;
use crate::models::topic::{AddTopicResult, TopicListItem};
use crate::utils::decode::decode_cx_bytes;

const ADD_TOPIC_URL: &str = "https://groupyd.chaoxing.com/apis/topic/addTopic";
const TOPIC_LIST_URL: &str = "https://groupyd.chaoxing.com/apis/topic/getTopicListWithPoff";

/// 当前毫秒时间戳字符串
fn now_ms() -> String {
    chrono::Utc::now().timestamp_millis().to_string()
}

/// 转义 HTML 文本中的 `&` `<` `>`，避免破坏 rtf_content 结构
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// 宽松地把 JSON 值解析为 i64（兼容数字或数字字符串）
fn as_i64_loose(v: &Value) -> i64 {
    v.as_i64()
        .or_else(|| v.as_str().and_then(|s| s.trim().parse::<i64>().ok()))
        .unwrap_or(0)
}

/// 把单条话题 JSON 映射为前端模型。
/// getTopicListWithPoff（groupyd）与 getMyTopicList（groupweb）的话题项字段名一致，故共用。
fn topic_item_from_json(t: &Value) -> TopicListItem {
    TopicListItem {
        id: as_i64_loose(&t["id"]),
        uuid: t["uuid"].as_str().unwrap_or("").to_string(),
        title: t["title"].as_str().unwrap_or("").to_string(),
        content: t["content"].as_str().unwrap_or("").to_string(),
        creator_name: t["createrName"].as_str().unwrap_or("").to_string(),
        create_time: as_i64_loose(&t["create_time"]),
        reply_count: as_i64_loose(&t["reply_count"]),
        read_count: as_i64_loose(&t["s_readcount"]),
        praise_count: as_i64_loose(&t["praise_count"]),
        share_url: t["shareUrl"].as_str().unwrap_or("").to_string(),
    }
}

/// 发布讨论区话题。
///
/// `puid` 为当前用户 uid；`tags` 取 `classId{clazz_id}`；`bbsid = md5(course_id)`。
/// `title` 为标题，`content` 为正文（自动包裹为 rtf）。
pub async fn add_topic(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    puid: &str,
    title: &str,
    content: &str,
) -> Result<AddTopicResult, AppError> {
    let bbsid = bbsid_from_course_id(course_id);
    let tags = format!("classId{}", clazz_id);
    let time = now_ms();
    let c0 = new_c0();
    let topic_uuid = new_c0();

    // 查询串签名（顺序对齐抓包：token,_time,_c_0_）
    let query = sign_query(&[
        ("token", INF_ENC_TOKEN),
        ("_time", &time),
        ("_c_0_", &c0),
    ]);
    let url = format!("{}?{}", ADD_TOPIC_URL, query);

    // 正文转 rtf；空正文则留空（对齐抓包行为）
    let rtf_content = if content.trim().is_empty() {
        String::new()
    } else {
        format!("<p element-id=\"init\">{}</p>", html_escape(content))
    };

    // 表单字段（不参与签名），全字段对齐抓包
    let form: Vec<(&str, &str)> = vec![
        ("allowViewReply", "1"),
        ("anonymous", "0"),
        ("attachment", "[]"),
        ("bbsid", &bbsid),
        ("chapterId", ""),
        ("describes", ""),
        ("files_url", ""),
        ("folderId", ""),
        ("isRtf", "1"),
        ("puid", puid),
        ("recordAnonymousScore", "0"),
        ("remind", ""),
        ("replyCanAnonymous", "0"),
        ("rtf_content", &rtf_content),
        ("sendNoticeToStu", "0"),
        ("tags", &tags),
        ("top", ""),
        ("topicContent", content),
        ("topicTitle", title),
        ("uuid", &topic_uuid),
    ];

    let resp = client.client.post(&url).form(&form).send().await?;
    if !resp.status().is_success() {
        return Err(AppError::Other(format!(
            "发布讨论话题失败: HTTP {}",
            resp.status()
        )));
    }

    let bytes = resp.bytes().await?;
    let text = decode_cx_bytes(&bytes);
    let json: Value = serde_json::from_str(&text)
        .map_err(|e| AppError::Parse(format!("发帖响应解析失败: {} | 原文: {}", e, text)))?;

    if json["result"].as_i64().unwrap_or(0) != 1 {
        let msg = json["msg"].as_str().unwrap_or("未知错误");
        return Err(AppError::Other(format!("发布讨论话题失败: {}", msg)));
    }

    let data = &json["data"];
    tracing::info!(
        "讨论话题发布成功: topicId={}",
        data["topicId"].as_i64().unwrap_or(0)
    );
    Ok(AddTopicResult {
        topic_id: as_i64_loose(&data["topicId"]),
        share_url: data["shareUrl"].as_str().unwrap_or("").to_string(),
        circle_id: as_i64_loose(&data["circleId"]),
        circle_name: data["circleName"].as_str().unwrap_or("").to_string(),
        uuid: data["uuid"].as_str().unwrap_or(&topic_uuid).to_string(),
    })
}

/// 列出课程讨论区的话题（首页 20 条），用于确认发帖结果 / 校验 bbsid。
pub async fn list_topics(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    puid: &str,
) -> Result<Vec<TopicListItem>, AppError> {
    let bbsid = bbsid_from_course_id(course_id);
    let tags = format!("classId0000001,classId{},courseId{}", clazz_id, course_id);
    let time = now_ms();
    let c0 = new_c0();

    // 顺序对齐抓包 entry[0]
    let query = sign_query(&[
        ("puid", puid),
        ("pageSize", "20"),
        ("searchType", "0"),
        ("order", "0"),
        ("bbsid", &bbsid),
        ("_time", &time),
        ("maxW", "320"),
        ("token", INF_ENC_TOKEN),
        ("tags", &tags),
        ("_c_0_", &c0),
    ]);
    let url = format!("{}?{}", TOPIC_LIST_URL, query);

    let resp = client.client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(AppError::Other(format!(
            "获取讨论话题列表失败: HTTP {}",
            resp.status()
        )));
    }

    let bytes = resp.bytes().await?;
    let text = decode_cx_bytes(&bytes);
    let json: Value = serde_json::from_str(&text)
        .map_err(|e| AppError::Parse(format!("话题列表解析失败: {}", e)))?;

    if json["result"].as_i64().unwrap_or(0) != 1 {
        let msg = json["msg"].as_str().unwrap_or("未知错误");
        return Err(AppError::Other(format!("获取讨论话题列表失败: {}", msg)));
    }

    let empty = Vec::new();
    let list = json["data"]["list"].as_array().unwrap_or(&empty);
    let items = list.iter().map(topic_item_from_json).collect();

    Ok(items)
}

/// 列出“我发布的”话题（groupweb `getMyTopicList`，cookie 鉴权，UTF-8 JSON）。
///
/// 该端点由服务端按当前登录用户过滤，直接返回 `{ "datas": [...] }`，
/// 无 `result` 包裹、也无需客户端比对 uid。话题项字段名与 groupyd 列表一致。
pub async fn list_my_topics(
    client: &HttpClient,
    course_id: &str,
) -> Result<Vec<TopicListItem>, AppError> {
    let bbsid = bbsid_from_course_id(course_id);
    let url = format!(
        "https://groupweb.chaoxing.com/course/topic/{}/getMyTopicList?lastValue=null&sortType=2",
        bbsid
    );
    // Referer 模拟浏览器“我的话题”页，提升兼容性
    let referer = format!(
        "https://groupweb.chaoxing.com/course/topic/{}/myTopic?courseId={}&classId=",
        bbsid, course_id
    );

    let resp = client
        .client
        .get(&url)
        .header("Referer", referer)
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(AppError::Other(format!(
            "获取我的话题列表失败: HTTP {}",
            resp.status()
        )));
    }

    let bytes = resp.bytes().await?;
    let text = decode_cx_bytes(&bytes);
    let json: Value = serde_json::from_str(&text)
        .map_err(|e| AppError::Parse(format!("我的话题列表解析失败: {}", e)))?;

    // getMyTopicList 直接返回 { "datas": [...] }，无 result 字段
    let datas = match json.get("datas").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => {
            let msg = json
                .get("msg")
                .and_then(|v| v.as_str())
                .or_else(|| json.get("errorMsg").and_then(|v| v.as_str()))
                .unwrap_or("获取我的话题列表失败（响应缺少 datas，可能未登录或会话失效）");
            return Err(AppError::Other(msg.to_string()));
        }
    };

    Ok(datas.iter().map(topic_item_from_json).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("a<b>&c"), "a&lt;b&gt;&amp;c");
        assert_eq!(html_escape("普通文本"), "普通文本");
    }

    #[test]
    fn test_as_i64_loose() {
        assert_eq!(as_i64_loose(&Value::from(123)), 123);
        assert_eq!(as_i64_loose(&Value::from("456")), 456);
        assert_eq!(as_i64_loose(&Value::Null), 0);
        assert_eq!(as_i64_loose(&Value::from("abc")), 0);
    }
}
