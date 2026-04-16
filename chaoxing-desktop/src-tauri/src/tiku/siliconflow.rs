//! 硅基流动大模型答题
//!
//! 对应 Python SiliconFlow 类
//! API: https://api.siliconflow.cn/v1/chat/completions
//! 支持请求间隔控制

use regex::Regex;
use reqwest::Client;
use std::sync::OnceLock;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing;

/// Markdown JSON 包装清理正则
static MD_JSON_RE: OnceLock<Regex> = OnceLock::new();

fn md_json_re() -> &'static Regex {
    MD_JSON_RE.get_or_init(|| Regex::new(r"(?s)^\s*```(?:json)?\s*(.*?)\s*```\s*$").unwrap())
}

/// 硅基流动大模型
pub struct TikuSiliconFlow {
    endpoint: String,
    api_key: String,
    model_name: String,
    min_interval_secs: u32,
    last_request_time: Mutex<Option<Instant>>,
    client: Client,
}

impl TikuSiliconFlow {
    /// 从配置创建
    pub fn new(endpoint: &str, api_key: &str, model_name: &str, min_interval: u32) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            api_key: api_key.to_string(),
            model_name: model_name.to_string(),
            min_interval_secs: min_interval,
            last_request_time: Mutex::new(None),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    /// 查询题目答案
    ///
    /// 对应 Python SiliconFlow._query()
    /// system prompt 与 Python 版完全一致
    pub async fn query(&self, title: &str, q_type: &str, options: &str) -> Option<String> {
        // 构造系统提示词（与 Python 完全一致）
        let system_prompt = match q_type {
            "single" => {
                "本题为单选题，请根据题目和选项选择唯一正确答案，输出的是选项的具体内容，而不是内容前的ABCD，并以JSON格式输出：示例回答：{\"Answer\": [\"正确选项内容\"]}。除此之外不要输出任何多余的内容，也不要使用MD语法。如果你使用了互联网搜索，也请不要返回搜索的结果和参考资料"
            }
            "multiple" => {
                "本题为多选题，请选择所有正确选项，输出的是选项的具体内容，而不是内容前的ABCD，以JSON格式输出：示例回答：{\"Answer\": [\"选项1\",\"选项2\"]}。除此之外不要输出任何多余的内容，也不要使用MD语法。如果你使用了互联网搜索，也请不要返回搜索的结果和参考资料"
            }
            "completion" => {
                "本题为填空题，请直接给出填空内容，以JSON格式输出：示例回答：{\"Answer\": [\"答案文本\"]}。除此之外不要输出任何多余的内容，也不要使用MD语法。如果你使用了互联网搜索，也请不要返回搜索的结果和参考资料"
            }
            "judgement" => {
                "本题为判断题，请回答'正确'或'错误'，以JSON格式输出：示例回答：{\"Answer\": [\"正确\"]}。除此之外不要输出任何多余的内容，也不要使用MD语法。如果你使用了互联网搜索，也请不要返回搜索的结果和参考资料"
            }
            _ => {
                // 简答题等其他类型
                "本题为简答题，请根据题目直接给出答案，以JSON格式输出：示例回答：{\"Answer\": [\"答案\"]}。除此之外不要输出任何多余的内容，也不要使用MD语法。如果你使用了互联网搜索，也请不要返回搜索的结果和参考资料"
            }
        };

        // 等待请求间隔
        self.wait_for_interval().await;

        let body = serde_json::json!({
            "model": self.model_name,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": format!("题目：{}\n选项：{}", title, options) },
            ],
            "stream": false,
            "max_tokens": 4096,
            "temperature": 0.7,
            "top_p": 0.7,
            "response_format": { "type": "text" },
        });

        let resp = match self
            .client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("硅基流动 API 异常: {}", e);
                return None;
            }
        };

        // 更新最后请求时间
        {
            let mut last = self.last_request_time.lock().await;
            *last = Some(Instant::now());
        }

        if resp.status().as_u16() != 200 {
            tracing::error!("硅基流动 API 请求失败: HTTP {}", resp.status());
            return None;
        }

        let json: serde_json::Value = match resp.json().await {
            Ok(j) => j,
            Err(e) => {
                tracing::error!("硅基流动响应解析失败: {}", e);
                return None;
            }
        };

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("");

        self.parse_response(content)
    }

    /// 解析 AI 响应
    fn parse_response(&self, content: &str) -> Option<String> {
        let cleaned = remove_md_json_wrapper(content);

        let parsed: serde_json::Value = match serde_json::from_str(&cleaned) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("硅基流动: 无法解析输出内容: {}, 原始: {}", e, content);
                return None;
            }
        };

        if let Some(arr) = parsed["Answer"].as_array() {
            let answers: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
            let joined = answers.join("\n").trim().to_string();
            if joined.is_empty() {
                None
            } else {
                Some(joined)
            }
        } else {
            tracing::error!("硅基流动: 输出中缺少 Answer 字段: {}", content);
            None
        }
    }

    /// 等待请求间隔
    async fn wait_for_interval(&self) {
        let sleep_duration = {
            let mut last = self.last_request_time.lock().await;
            if let Some(last_time) = *last {
                let elapsed = last_time.elapsed();
                let min_interval = std::time::Duration::from_secs(self.min_interval_secs as u64);
                if elapsed < min_interval {
                    let sleep_time = min_interval - elapsed;
                    tracing::debug!("硅基流动请求间隔过短，等待 {:?}", sleep_time);
                    *last = Some(Instant::now() + sleep_time);
                    Some(sleep_time)
                } else {
                    *last = Some(Instant::now());
                    None
                }
            } else {
                *last = Some(Instant::now());
                None
            }
        };
        if let Some(d) = sleep_duration {
            tokio::time::sleep(d).await;
        }
    }

    /// 检查连接
    ///
    /// 对应 Python SiliconFlow.check_llm_connection()
    pub async fn check_connection(&self) -> bool {
        tracing::info!("正在检查硅基流动大模型连接...");

        let body = serde_json::json!({
            "model": self.model_name,
            "messages": [
                { "role": "user", "content": "你好，请回答：1+1 等于几？只回答数字。" }
            ],
            "stream": false,
            "max_tokens": 10,
            "temperature": 0.7,
            "top_p": 0.7,
            "response_format": { "type": "text" },
        });

        let resp = match self
            .client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("硅基流动连接检查失败: {}", e);
                return false;
            }
        };

        if resp.status().as_u16() != 200 {
            tracing::error!("硅基流动连接检查失败: HTTP {}", resp.status());
            return false;
        }

        let json: serde_json::Value = match resp.json().await {
            Ok(j) => j,
            Err(e) => {
                tracing::error!("硅基流动连接检查响应解析失败: {}", e);
                return false;
            }
        };

        let has_content = json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| !s.is_empty())
            .unwrap_or(false);

        if has_content {
            tracing::info!("硅基流动连接检查成功");
        } else {
            tracing::error!("硅基流动连接检查失败: 未收到有效响应");
        }
        has_content
    }

    /// 名称
    pub fn name(&self) -> &str {
        "硅基流动大模型"
    }
}

/// 移除 Markdown JSON 代码块包装
fn remove_md_json_wrapper(md_str: &str) -> String {
    let re = md_json_re();
    match re.captures(md_str) {
        Some(caps) => caps
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| md_str.trim().to_string()),
        None => md_str.trim().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response_valid() {
        let sf = TikuSiliconFlow::new(
            "https://api.siliconflow.cn/v1/chat/completions",
            "key",
            "model",
            0,
        );
        let result = sf.parse_response(r#"{"Answer": ["选项内容"]}"#);
        assert_eq!(result, Some("选项内容".to_string()));
    }

    #[test]
    fn test_parse_response_with_md_wrapper() {
        let sf = TikuSiliconFlow::new(
            "https://api.siliconflow.cn/v1/chat/completions",
            "key",
            "model",
            0,
        );
        let result = sf.parse_response("```json\n{\"Answer\": [\"正确\"]}\n```");
        assert_eq!(result, Some("正确".to_string()));
    }

    #[test]
    fn test_parse_response_invalid() {
        let sf = TikuSiliconFlow::new(
            "https://api.siliconflow.cn/v1/chat/completions",
            "key",
            "model",
            0,
        );
        assert!(sf.parse_response("invalid json").is_none());
    }
}
