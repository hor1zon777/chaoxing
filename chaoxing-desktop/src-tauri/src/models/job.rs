use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobType {
    Video,
    Document,
    Read,
    Live,
    #[serde(rename = "workid")]
    Work,
}

impl JobType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Video => "视频",
            Self::Document => "文档",
            Self::Read => "阅读",
            Self::Live => "直播",
            Self::Work => "作业",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    #[serde(rename = "type")]
    pub job_type: JobType,
    pub jobid: String,
    #[serde(default)]
    pub is_completed: bool,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub otherinfo: String,
    #[serde(default)]
    pub mid: String,
    #[serde(default)]
    pub objectid: String,
    #[serde(default)]
    pub aid: String,
    #[serde(default)]
    pub jtoken: String,
    #[serde(default)]
    pub enc: String,
    // 视频特有
    #[serde(default)]
    pub play_time: u64,
    #[serde(default)]
    pub rt: String,
    #[serde(default)]
    pub att_duration: String,
    #[serde(default)]
    pub att_duration_enc: String,
    #[serde(default)]
    pub video_face_capture_enc: String,
    // 直播特有
    #[serde(default)]
    pub property: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub live_id: Option<String>,
    #[serde(default)]
    pub stream_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobInfo {
    #[serde(default)]
    pub ktoken: String,
    #[serde(default)]
    pub mt_enc: String,
    #[serde(default)]
    pub report_time_interval: u32,
    #[serde(default)]
    pub defenc: String,
    #[serde(default)]
    pub cardid: String,
    #[serde(default)]
    pub cpi: String,
    #[serde(default)]
    pub qnenc: String,
    #[serde(default)]
    pub knowledgeid: String,
    #[serde(default)]
    pub not_open: bool,
}
