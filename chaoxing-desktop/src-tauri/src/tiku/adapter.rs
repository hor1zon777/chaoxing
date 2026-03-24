//! TikuAdapter 题库
//!
//! 对应 Python TikuAdapter 类
//! 参考: https://github.com/DokiDoki1103/tikuAdapter
//! 通过 POST JSON 请求用户自建的题库适配器

use regex::Regex;
use reqwest::Client;
use std::sync::OnceLock;
use tracing;

/// TikuAdapter 选项清理正则
static OPTION_PREFIX_RE: OnceLock<Regex> = OnceLock::new();

fn option_prefix_re() -> &'static Regex {
    OPTION_PREFIX_RE.get_or_init(|| Regex::new(r"^[A-Za-z]\.?\u{3001}?\s?").unwrap())
}

/// TikuAdapter 题库
pub struct TikuAdapter {
    url: String,
    client: Client,
}

impl TikuAdapter {
    /// 从 URL 创建
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("创建 TikuAdapter HTTP 客户端失败"),
        }
    }

    /// 查询题目答案
    ///
    /// 对应 Python TikuAdapter._query()
    /// POST JSON 格式: { question, options: [...], type: 0/1/2/3/4 }
    pub async fn query(&self, title: &str, q_type: &str, options: &str) -> Option<String> {
        if self.url.is_empty() {
            tracing::error!("TikuAdapter: 未配置 URL");
            return None;
        }

        // 映射题目类型到数字
        let type_code = match q_type {
            "single" => 0,
            "multiple" => 1,
            "completion" => 2,
            "judgement" => 3,
            _ => 4,
        };

        // 清理选项前缀（去除 A. B. C. 等）
        let re = option_prefix_re();
        let options_list: Vec<String> = options
            .split('\n')
            .map(|o| re.replace(o, "").to_string())
            .collect();

        let body = serde_json::json!({
            "question": title,
            "options": options_list,
            "type": type_code,
        });

        let resp = match self.client.post(&self.url).json(&body).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("TikuAdapter 查询网络错误: {}", e);
                return None;
            }
        };

        if resp.status().as_u16() != 200 {
            tracing::error!("TikuAdapter 查询失败: HTTP {}", resp.status());
            return None;
        }

        let json: serde_json::Value = match resp.json().await {
            Ok(j) => j,
            Err(e) => {
                tracing::error!("TikuAdapter 响应解析失败: {}", e);
                return None;
            }
        };

        // 提取 bestAnswer 数组
        let best_answer = &json["answer"]["bestAnswer"];
        if let Some(arr) = best_answer.as_array() {
            if arr.is_empty() {
                tracing::error!("TikuAdapter: 查询失败，bestAnswer 为空");
                return None;
            }
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
            tracing::error!("TikuAdapter: 查询失败，bestAnswer 字段不存在");
            None
        }
    }

    /// 名称
    pub fn name(&self) -> &str {
        "TikuAdapter题库"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_prefix_cleaning() {
        let re = option_prefix_re();
        assert_eq!(re.replace("A. 选项内容", "").as_ref(), "选项内容");
        assert_eq!(re.replace("B、选项内容", "").as_ref(), "选项内容");
        assert_eq!(re.replace("C 选项内容", "").as_ref(), "选项内容");
        assert_eq!(re.replace("D.选项内容", "").as_ref(), "选项内容");
        // 不应清理非字母开头的内容
        assert_eq!(re.replace("1. 选项", "").as_ref(), "1. 选项");
    }

    #[test]
    fn test_new() {
        let adapter = TikuAdapter::new("http://localhost:8080/api");
        assert_eq!(adapter.url, "http://localhost:8080/api");
    }
}
