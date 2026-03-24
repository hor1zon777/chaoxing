use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default = "default_speed")]
    pub speed: f64,
    #[serde(default = "default_jobs")]
    pub jobs: u32,
    #[serde(default = "default_notopen_action")]
    pub notopen_action: String,
    // 题库
    #[serde(default)]
    pub tiku_provider: String,
    #[serde(default)]
    pub tiku_tokens: String,
    #[serde(default)]
    pub tiku_submit: bool,
    #[serde(default = "default_cover_rate")]
    pub tiku_cover_rate: f64,
    #[serde(default = "default_delay")]
    pub tiku_delay: f64,
    // AI
    #[serde(default)]
    pub ai_endpoint: String,
    #[serde(default)]
    pub ai_key: String,
    #[serde(default)]
    pub ai_model: String,
    #[serde(default)]
    pub ai_proxy: String,
    #[serde(default = "default_ai_interval")]
    pub ai_min_interval: u32,
    // SiliconFlow
    #[serde(default)]
    pub siliconflow_key: String,
    #[serde(default = "default_sf_model")]
    pub siliconflow_model: String,
    #[serde(default = "default_sf_endpoint")]
    pub siliconflow_endpoint: String,
    // LIKE
    #[serde(default)]
    pub like_search: bool,
    #[serde(default = "default_true")]
    pub like_vision: bool,
    #[serde(default = "default_like_model")]
    pub like_model: String,
    #[serde(default = "default_true")]
    pub like_retry: bool,
    #[serde(default = "default_retry_times")]
    pub like_retry_times: u32,
    // Adapter
    #[serde(default)]
    pub tiku_adapter_url: String,
    // 判断题
    #[serde(default = "default_true_list")]
    pub true_list: String,
    #[serde(default = "default_false_list")]
    pub false_list: String,
    // 通知
    #[serde(default)]
    pub notification_provider: String,
    #[serde(default)]
    pub notification_url: String,
    #[serde(default)]
    pub notification_tg_chat_id: String,
    // LLM 检查
    #[serde(default = "default_true")]
    pub check_llm_connection: bool,
}

fn default_speed() -> f64 {
    1.0
}
fn default_jobs() -> u32 {
    4
}
fn default_notopen_action() -> String {
    "retry".to_string()
}
fn default_cover_rate() -> f64 {
    0.9
}
fn default_delay() -> f64 {
    1.0
}
fn default_ai_interval() -> u32 {
    3
}
fn default_sf_model() -> String {
    "deepseek-ai/DeepSeek-R1".to_string()
}
fn default_sf_endpoint() -> String {
    "https://api.siliconflow.cn/v1/chat/completions".to_string()
}
fn default_like_model() -> String {
    "glm-4.5-air".to_string()
}
fn default_retry_times() -> u32 {
    3
}
fn default_true_list() -> String {
    "正确,对,√,是".to_string()
}
fn default_false_list() -> String {
    "错误,错,×,否,不对,不正确".to_string()
}
fn default_true() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        serde_json::from_str("{}").unwrap()
    }
}
