//! AI 大模型答题（OpenAI 兼容接口）
//!
//! 对应 Python AI 类
//! 使用 reqwest 直接调用 OpenAI 兼容 API
//! 支持代理、请求间隔控制

use regex::Regex;
use reqwest::Client;
use std::sync::OnceLock;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing;

/// Markdown JSON 包装清理正则
static MD_JSON_RE: OnceLock<Regex> = OnceLock::new();
/// 选项字母前缀清理正则
static OPTION_LETTER_RE: OnceLock<Regex> = OnceLock::new();

fn md_json_re() -> &'static Regex {
    MD_JSON_RE.get_or_init(|| Regex::new(r"(?s)^\s*```(?:json)?\s*(.*?)\s*```\s*$").unwrap())
}

fn option_letter_re() -> &'static Regex {
    OPTION_LETTER_RE.get_or_init(|| Regex::new(r"^[A-Z]\s*").unwrap())
}

/// AI 大模型答题
pub struct TikuAi {
    endpoint: String,
    key: String,
    model: String,
    proxy: String,
    min_interval_secs: u32,
    last_request_time: Mutex<Option<Instant>>,
    client: Client,
}

impl TikuAi {
    /// 从配置创建
    pub fn new(endpoint: &str, key: &str, model: &str, proxy: &str, min_interval: u32) -> Self {
        let client = if !proxy.is_empty() {
            Client::builder()
                .proxy(reqwest::Proxy::all(proxy).expect("AI 代理配置无效"))
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("创建 AI HTTP 客户端失败")
        } else {
            Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("创建 AI HTTP 客户端失败")
        };

        Self {
            endpoint: endpoint.to_string(),
            key: key.to_string(),
            model: model.to_string(),
            proxy: proxy.to_string(),
            min_interval_secs: min_interval,
            last_request_time: Mutex::new(None),
            client,
        }
    }

    /// 查询题目答案
    ///
    /// 对应 Python AI._query()
    /// 根据题目类型构造不同的 system prompt，解析 JSON 输出
    pub async fn query(&self, title: &str, q_type: &str, options: &str) -> Option<String> {
        // 去除选项字母前缀，防止大模型直接输出字母而非内容
        let re = option_letter_re();
        let cleaned_options: Vec<String> = options
            .split('\n')
            .map(|o| re.replace(o, "").to_string())
            .collect();
        let options_text = cleaned_options.join("\n");

        // 根据题目类型构造 system prompt（与 Python 完全一致）
        let system_prompt = match q_type {
            "single" => {
                "本题为单选题，你只能选择一个选项，请根据题目和选项回答问题，以json格式输出正确的选项内容，示例回答：{\"Answer\": [\"答案\"]}。除此之外不要输出任何多余的内容，也不要使用MD语法。如果你使用了互联网搜索，也请不要返回搜索的结果和参考资料"
            }
            "multiple" => {
                "本题为多选题，你必须选择两个或以上选项，请根据题目和选项回答问题，以json格式输出正确的选项内容，示例回答：{\"Answer\": [\"答案1\",\n\"答案2\",\n\"答案3\"]}。除此之外不要输出任何多余的内容，也不要使用MD语法。如果你使用了互联网搜索，也请不要返回搜索的结果和参考资料"
            }
            "completion" => {
                "本题为填空题，你必须根据语境和相关知识填入合适的内容，请根据题目回答问题，以json格式输出正确的答案，示例回答：{\"Answer\": [\"答案\"]}。除此之外不要输出任何多余的内容，也不要使用MD语法。如果你使用了互联网搜索，也请不要返回搜索的结果和参考资料"
            }
            "judgement" => {
                "本题为判断题，你只能回答正确或者错误，请根据题目回答问题，以json格式输出正确的答案，示例回答：{\"Answer\": [\"正确\"]}。除此之外不要输出任何多余的内容，也不要使用MD语法。如果你使用了互联网搜索，也请不要返回搜索的结果和参考资料"
            }
            _ => {
                "本题为简答题，你必须根据语境和相关知识填入合适的内容，请根据题目回答问题，以json格式输出正确的答案，示例回答：{\"Answer\": [\"这是我的答案\"]}。除此之外不要输出任何多余的内容，也不要使用MD语法。如果你使用了互联网搜索，也请不要返回搜索的结果和参考资料"
            }
        };

        // 构造用户消息
        let user_content = if q_type == "single" || q_type == "multiple" {
            format!("题目：{}\n选项：{}", title, options_text)
        } else {
            format!("题目：{}", title)
        };

        // 等待请求间隔
        self.wait_for_interval().await;

        // 构造 OpenAI 兼容请求
        let url = format!("{}/chat/completions", self.endpoint.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_content },
            ],
        });

        let resp = match self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.key))
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("AI 大模型查询网络错误: {}", e);
                return None;
            }
        };

        // 更新最后请求时间
        {
            let mut last = self.last_request_time.lock().await;
            *last = Some(Instant::now());
        }

        if resp.status().as_u16() != 200 {
            tracing::error!("AI 大模型查询失败: HTTP {}", resp.status());
            return None;
        }

        let json: serde_json::Value = match resp.json().await {
            Ok(j) => j,
            Err(e) => {
                tracing::error!("AI 大模型响应解析失败: {}", e);
                return None;
            }
        };

        // 提取回复内容
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("");

        self.parse_ai_response(content)
    }

    /// 解析 AI 响应，提取 Answer 数组
    fn parse_ai_response(&self, content: &str) -> Option<String> {
        // 移除可能的 Markdown JSON 包装
        let cleaned = remove_md_json_wrapper(content);

        // 解析 JSON
        let parsed: serde_json::Value = match serde_json::from_str(&cleaned) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("无法解析大模型输出内容: {}, 原始: {}", e, content);
                return None;
            }
        };

        // 提取 Answer 数组
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
            tracing::error!("大模型输出中缺少 Answer 字段: {}", content);
            None
        }
    }

    /// 等待请求间隔
    async fn wait_for_interval(&self) {
        let last = self.last_request_time.lock().await;
        if let Some(last_time) = *last {
            let elapsed = last_time.elapsed();
            let min_interval = std::time::Duration::from_secs(self.min_interval_secs as u64);
            if elapsed < min_interval {
                let sleep_time = min_interval - elapsed;
                tracing::debug!("AI 请求间隔过短，等待 {:?}", sleep_time);
                drop(last); // 释放锁后再 sleep
                tokio::time::sleep(sleep_time).await;
            }
        }
    }

    /// 检查连接
    ///
    /// 对应 Python AI.check_llm_connection()
    pub async fn check_connection(&self) -> bool {
        tracing::info!("正在检查 AI 大模型连接...");

        let url = format!("{}/chat/completions", self.endpoint.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "user", "content": "你好，请回答：1+1 等于几？只回答数字。" }
            ],
            "max_tokens": 10,
        });

        let resp = match self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.key))
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("AI 大模型连接检查失败: {}", e);
                return false;
            }
        };

        if resp.status().as_u16() != 200 {
            tracing::error!("AI 大模型连接检查失败: HTTP {}", resp.status());
            return false;
        }

        let json: serde_json::Value = match resp.json().await {
            Ok(j) => j,
            Err(e) => {
                tracing::error!("AI 大模型连接检查响应解析失败: {}", e);
                return false;
            }
        };

        let has_content = json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| !s.is_empty())
            .unwrap_or(false);

        if has_content {
            tracing::info!("AI 大模型连接检查成功");
        } else {
            tracing::error!("AI 大模型连接检查失败: 未收到响应");
        }
        has_content
    }

    /// 名称
    pub fn name(&self) -> &str {
        "AI大模型答题"
    }
}

/// 移除 Markdown JSON 代码块包装
///
/// 对应 Python: remove_md_json_wrapper()
fn remove_md_json_wrapper(md_str: &str) -> String {
    let re = md_json_re();
    match re.captures(md_str) {
        Some(caps) => caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_else(|| md_str.trim().to_string()),
        None => md_str.trim().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_md_json_wrapper_no_wrapper() {
        let input = r#"{"Answer": ["选项A"]}"#;
        assert_eq!(remove_md_json_wrapper(input), input.trim());
    }

    #[test]
    fn test_remove_md_json_wrapper_with_json_block() {
        let input = "```json\n{\"Answer\": [\"选项A\"]}\n```";
        assert_eq!(remove_md_json_wrapper(input), r#"{"Answer": ["选项A"]}"#);
    }

    #[test]
    fn test_remove_md_json_wrapper_with_plain_block() {
        let input = "```\n{\"Answer\": [\"B\"]}\n```";
        assert_eq!(remove_md_json_wrapper(input), r#"{"Answer": ["B"]}"#);
    }

    #[test]
    fn test_parse_ai_response() {
        let ai = TikuAi::new("http://localhost", "key", "model", "", 0);
        let content = r#"{"Answer": ["选项A内容"]}"#;
        assert_eq!(ai.parse_ai_response(content), Some("选项A内容".to_string()));
    }

    #[test]
    fn test_parse_ai_response_multiple() {
        let ai = TikuAi::new("http://localhost", "key", "model", "", 0);
        let content = r#"{"Answer": ["答案1", "答案2"]}"#;
        assert_eq!(
            ai.parse_ai_response(content),
            Some("答案1\n答案2".to_string())
        );
    }

    #[test]
    fn test_parse_ai_response_invalid() {
        let ai = TikuAi::new("http://localhost", "key", "model", "", 0);
        assert!(ai.parse_ai_response("这不是JSON").is_none());
    }

    #[test]
    fn test_option_letter_cleaning() {
        let re = option_letter_re();
        assert_eq!(re.replace("A 选项内容", "").as_ref(), "选项内容");
        assert_eq!(re.replace("B选项内容", "").as_ref(), "选项内容");
        assert_eq!(re.replace("C  选项", "").as_ref(), "选项"); // \s* 贪婪匹配所有空格
    }
}
