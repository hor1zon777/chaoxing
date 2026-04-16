//! 言溪题库
//!
//! 对应 Python TikuYanxi 类
//! API: https://tk.enncy.cn/query
//! 支持多 token 轮换和余额不足自动切换

use std::sync::atomic::{AtomicUsize, Ordering};

use reqwest::Client;
use tracing;

/// 言溪题库
pub struct TikuYanxi {
    api: String,
    tokens: Vec<String>,
    current_token_index: AtomicUsize,
    client: Client,
}

impl TikuYanxi {
    /// 从逗号分隔的 token 字符串创建
    pub fn new(tokens: &str) -> Self {
        let token_list: Vec<String> = tokens
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            api: "https://tk.enncy.cn/query".to_string(),
            tokens: token_list,
            current_token_index: AtomicUsize::new(0),
            client: Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    /// 查询题目答案
    ///
    /// 对应 Python TikuYanxi._query()
    /// 如果当前 token 余额不足，自动切换到下一个 token 并重试
    pub async fn query(&self, title: &str, _q_type: &str, _options: &str) -> Option<String> {
        if self.tokens.is_empty() {
            tracing::error!("言溪题库: 未配置 token");
            return None;
        }

        self.query_with_retry(title, 0).await
    }

    /// 带重试的查询（token 用完时自动切换）
    ///
    /// 使用迭代而非递归，避免 async fn 递归需要 Box::pin 的问题。
    async fn query_with_retry(&self, title: &str, initial_depth: usize) -> Option<String> {
        let mut retry_depth = initial_depth;

        loop {
            // 防止无限重试
            if retry_depth >= self.tokens.len() {
                tracing::error!("言溪题库: 所有 TOKEN 已用完");
                return None;
            }

            let index = self.current_token_index.load(Ordering::SeqCst);
            let token = &self.tokens[index % self.tokens.len()];

            let resp = self
                .client
                .get(&self.api)
                .query(&[("question", title), ("token", token.as_str())])
                .send()
                .await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("言溪题库查询网络错误: {}", e);
                    return None;
                }
            };

            if resp.status().as_u16() != 200 {
                tracing::error!("言溪题库查询失败: HTTP {}", resp.status());
                return None;
            }

            let json: serde_json::Value = match resp.json().await {
                Ok(j) => j,
                Err(e) => {
                    tracing::error!("言溪题库响应解析失败: {}", e);
                    return None;
                }
            };

            // code 为 true 或 1 表示成功
            let code_ok = json["code"].as_bool() == Some(true)
                || json["code"].as_i64() == Some(1);

            if !code_ok {
                // 检查是否因为次数不足需要换 token
                let answer_text = json["data"]["answer"].as_str().unwrap_or("");
                if answer_text.contains("次数不足") {
                    tracing::info!("言溪题库: TOKEN 查询次数不足，切换下一个 token");
                    self.current_token_index.fetch_add(1, Ordering::SeqCst);
                    retry_depth += 1;
                    continue; // 迭代重试而非递归
                }

                let msg = json["message"].as_str().unwrap_or("未知错误");
                tracing::error!("言溪题库查询失败: {}", msg);
                return None;
            }

            // 提取答案
            return json["data"]["answer"]
                .as_str()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
        }
    }

    /// 名称
    pub fn name(&self) -> &str {
        "言溪题库"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_tokens() {
        let tiku = TikuYanxi::new("token1, token2, token3");
        assert_eq!(tiku.tokens.len(), 3);
        assert_eq!(tiku.tokens[0], "token1");
        assert_eq!(tiku.tokens[1], "token2");
        assert_eq!(tiku.tokens[2], "token3");
    }

    #[test]
    fn test_new_with_empty_tokens() {
        let tiku = TikuYanxi::new("");
        assert!(tiku.tokens.is_empty());
    }

    #[test]
    fn test_new_with_single_token() {
        let tiku = TikuYanxi::new("single_token");
        assert_eq!(tiku.tokens.len(), 1);
    }

    #[test]
    fn test_name() {
        let tiku = TikuYanxi::new("t1");
        assert_eq!(tiku.name(), "言溪题库");
    }
}
