//! LIKE 知识库
//!
//! 对应 Python TikuLike 类
//! API: https://app.datam.site/api/v1/query
//! 支持多 token、余额管理、重试机制

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

use rand::seq::SliceRandom;
use reqwest::Client;
use tokio::sync::RwLock;
use tracing;

/// LIKE 知识库配置
pub struct LikeConfig {
    pub search: bool,
    pub vision: bool,
    pub model: String,
    pub retry: bool,
    pub retry_times: u32,
}

/// LIKE 知识库
pub struct TikuLike {
    query_api: String,
    balance_api: String,
    tokens: Vec<String>,
    balance: RwLock<HashMap<String, i64>>,
    config: LikeConfig,
    client: Client,
    query_count: AtomicU32,
}

impl TikuLike {
    /// 从配置创建
    pub fn new(
        tokens: &str,
        search: bool,
        vision: bool,
        model: &str,
        retry: bool,
        retry_times: u32,
    ) -> Self {
        let token_list: Vec<String> = tokens
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            query_api: "https://app.datam.site/api/v1/query".to_string(),
            balance_api: "https://app.datam.site/api/v1/balance".to_string(),
            tokens: token_list,
            balance: RwLock::new(HashMap::new()),
            config: LikeConfig {
                search,
                vision,
                model: model.to_string(),
                retry,
                retry_times,
            },
            client: Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .expect("创建 LIKE 知识库 HTTP 客户端失败"),
            query_count: AtomicU32::new(0),
        }
    }

    /// 初始化：获取所有 token 的余额
    pub async fn init(&self) {
        self.update_all_balances().await;
    }

    /// 查询题目答案
    ///
    /// 对应 Python TikuLike._query()
    pub async fn query(&self, title: &str, q_type: &str, options: &str) -> Option<String> {
        if self.tokens.is_empty() {
            tracing::error!("LIKE知识库: 未配置 token");
            return None;
        }

        // 构造题目类型前缀
        let q_type_map: HashMap<&str, &str> = [
            ("single", "【单选题】"),
            ("multiple", "【多选题】"),
            ("completion", "【填空题】"),
            ("judgement", "【判断题】"),
        ]
        .iter()
        .copied()
        .collect();

        let prefix = q_type_map.get(q_type).unwrap_or(&"【其他类型题目】");
        let mut question = format!("{}{}\n", prefix, title);

        if q_type == "single" || q_type == "multiple" {
            question.push_str(&format!("选项为: {}\n", options));
        }

        // 选择一个有余额的 token
        let token = self.select_token().await?;

        let mut ans: Option<String> = None;
        let mut try_times = 0;
        let max_retries = if self.config.retry {
            self.config.retry_times
        } else {
            1
        };

        while ans.is_none() && try_times < max_retries {
            ans = self.query_single(&token, &question).await;
            try_times += 1;

            if ans.is_some() {
                // 查询成功，减少余额
                let mut balance = self.balance.write().await;
                let bal = balance.entry(token.clone()).or_insert(0);
                *bal -= 1;
                tracing::info!(
                    "LIKE知识库: 使用 Token ...{} 查询成功，剩余次数: {}",
                    &token[token.len().saturating_sub(5)..],
                    bal
                );
                break;
            } else if try_times < max_retries {
                tracing::warn!(
                    "LIKE知识库: 使用 Token ...{} 查询失败，进行第 {} 次重试...",
                    &token[token.len().saturating_sub(5)..],
                    try_times + 1
                );
            }
        }

        // 每 10 次查询后更新余额
        let count = self.query_count.fetch_add(1, Ordering::SeqCst);
        if (count + 1) % 10 == 0 {
            self.update_all_balances().await;
        }

        ans
    }

    /// 选择一个有余额的 token
    async fn select_token(&self) -> Option<String> {
        let balance = self.balance.read().await;
        let mut rng = rand::thread_rng();

        // 先随机选一个
        let token = self.tokens.choose(&mut rng)?;

        // 检查余额
        let bal = balance.get(token).copied().unwrap_or(0);
        if bal > 0 {
            return Some(token.clone());
        }

        // 当前 token 余额不足，查找其他有余额的
        tracing::error!(
            "LIKE知识库: 当前 Token ...{} 查询次数不足",
            &token[token.len().saturating_sub(5)..]
        );

        let available: Vec<&String> = self
            .tokens
            .iter()
            .filter(|t| balance.get(*t).copied().unwrap_or(0) > 0)
            .collect();

        if available.is_empty() {
            tracing::error!("LIKE知识库: 所有 Token 查询次数都不足");
            return None;
        }

        available.choose(&mut rng).map(|t| (*t).clone())
    }

    /// 查询单个问题
    ///
    /// 对应 Python TikuLike._query_single()
    async fn query_single(&self, token: &str, question: &str) -> Option<String> {
        let request_data = serde_json::json!({
            "query": question,
            "model": if self.config.model.is_empty() { "" } else { &self.config.model },
            "search": self.config.search,
            "vision": self.config.vision,
        });

        let resp = self
            .client
            .post(&self.query_api)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .json(&request_data)
            .send()
            .await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("LIKE知识库查询异常: {}", e);
                return None;
            }
        };

        let status = resp.status().as_u16();
        if status != 200 {
            match status {
                401 => tracing::error!("LIKE知识库: 认证失败，请检查 Token"),
                429 => tracing::error!("LIKE知识库: 请求过于频繁"),
                500 => tracing::error!("LIKE知识库: 服务器内部错误"),
                400 => tracing::error!("LIKE知识库: 请求参数错误"),
                403 => tracing::error!("LIKE知识库: 访问被拒绝"),
                _ => tracing::error!("LIKE知识库: 查询失败，状态码 {}", status),
            }
            return None;
        }

        self.parse_response(resp).await
    }

    /// 解析 API 响应
    ///
    /// 对应 Python TikuLike._parse_response() + _extract_answer_by_type()
    async fn parse_response(&self, response: reqwest::Response) -> Option<String> {
        let json: serde_json::Value = match response.json().await {
            Ok(j) => j,
            Err(e) => {
                tracing::error!("LIKE知识库: 响应解析失败: {}", e);
                return None;
            }
        };

        // 记录消息
        if let Some(msg) = json["message"].as_str() {
            if !msg.is_empty() {
                tracing::info!("LIKE知识库响应消息: {}", msg);
            }
        }

        let results = &json["results"];
        if results.is_null() || !results.is_object() {
            tracing::error!("LIKE知识库: results 字段格式不正确");
            return None;
        }

        let output = &results["output"];
        if output.is_null() || !output.is_object() {
            tracing::error!("LIKE知识库: output 字段格式不正确");
            return None;
        }

        let q_type = match output["questionType"].as_str() {
            Some(t) => t,
            None => {
                tracing::error!("LIKE知识库: questionType 字段不存在");
                return None;
            }
        };

        let answer = &output["answer"];
        if answer.is_null() || !answer.is_object() {
            tracing::error!("LIKE知识库: answer 字段不存在或格式错误");
            return None;
        }

        self.extract_answer_by_type(q_type, answer)
    }

    /// 根据题目类型提取答案
    fn extract_answer_by_type(&self, q_type: &str, answer: &serde_json::Value) -> Option<String> {
        match q_type {
            "CHOICE" => {
                let options = &answer["selectedOptions"];
                if let Some(arr) = options.as_array() {
                    let valid: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    if valid.is_empty() {
                        tracing::error!("LIKE知识库: CHOICE 类型没有有效选项");
                        None
                    } else {
                        Some(valid.join("\n"))
                    }
                } else {
                    tracing::error!("LIKE知识库: CHOICE 类型缺少 selectedOptions 字段");
                    None
                }
            }
            "FILL_IN_BLANK" => {
                let blanks = &answer["blanks"];
                if let Some(arr) = blanks.as_array() {
                    let valid: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    if valid.is_empty() {
                        tracing::error!("LIKE知识库: FILL_IN_BLANK 类型没有有效填空内容");
                        None
                    } else {
                        Some(valid.join("\n"))
                    }
                } else {
                    tracing::error!("LIKE知识库: FILL_IN_BLANK 类型缺少 blanks 字段");
                    None
                }
            }
            "JUDGMENT" => {
                match answer["isCorrect"].as_bool() {
                    Some(true) => Some("正确".to_string()),
                    Some(false) => Some("错误".to_string()),
                    None => {
                        tracing::error!("LIKE知识库: JUDGMENT 类型缺少 isCorrect 字段");
                        None
                    }
                }
            }
            _ => {
                // 其他类型尝试 otherText
                answer["otherText"]
                    .as_str()
                    .map(|s| s.to_string())
                    .or_else(|| {
                        tracing::error!("LIKE知识库: 未知题目类型 {} 且缺少 otherText 字段", q_type);
                        None
                    })
            }
        }
    }

    /// 获取单个 token 的余额
    async fn get_api_balance(&self, token: &str) -> i64 {
        let resp = self
            .client
            .get(&self.balance_api)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("LIKE知识库: 获取余额失败: {}", e);
                return 0;
            }
        };

        if resp.status().as_u16() != 200 {
            tracing::error!("LIKE知识库: 获取余额失败，状态码: {}", resp.status());
            return 0;
        }

        let json: serde_json::Value = match resp.json().await {
            Ok(j) => j,
            Err(e) => {
                tracing::error!("LIKE知识库: 余额响应解析失败: {}", e);
                return 0;
            }
        };

        json["balance"].as_i64().unwrap_or(0)
    }

    /// 更新所有 token 的余额
    ///
    /// 先并发获取所有余额（不持锁），再短暂持写锁批量更新
    async fn update_all_balances(&self) {
        let mut results = Vec::with_capacity(self.tokens.len());
        for token in &self.tokens {
            let bal = self.get_api_balance(token).await;
            tracing::info!(
                "LIKE知识库 Token: ...{} 剩余查询次数: {} (仅供参考)",
                &token[token.len().saturating_sub(5)..],
                bal
            );
            results.push((token.clone(), bal));
        }
        let mut balance_map = self.balance.write().await;
        for (token, bal) in results {
            balance_map.insert(token, bal);
        }
    }

    /// 名称
    pub fn name(&self) -> &str {
        "LIKE知识库"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_tokens() {
        let tiku = TikuLike::new("token1,token2", false, true, "", true, 3);
        assert_eq!(tiku.tokens.len(), 2);
    }

    #[test]
    fn test_new_with_empty() {
        let tiku = TikuLike::new("", false, true, "", true, 3);
        assert!(tiku.tokens.is_empty());
    }

    #[test]
    fn test_extract_choice_answer() {
        let tiku = TikuLike::new("t", false, true, "", true, 3);
        let answer = serde_json::json!({
            "selectedOptions": ["选项A", "选项B"]
        });
        let result = tiku.extract_answer_by_type("CHOICE", &answer);
        assert_eq!(result, Some("选项A\n选项B".to_string()));
    }

    #[test]
    fn test_extract_judgment_answer() {
        let tiku = TikuLike::new("t", false, true, "", true, 3);
        let answer = serde_json::json!({ "isCorrect": true });
        assert_eq!(
            tiku.extract_answer_by_type("JUDGMENT", &answer),
            Some("正确".to_string())
        );

        let answer_false = serde_json::json!({ "isCorrect": false });
        assert_eq!(
            tiku.extract_answer_by_type("JUDGMENT", &answer_false),
            Some("错误".to_string())
        );
    }

    #[test]
    fn test_extract_fill_in_blank() {
        let tiku = TikuLike::new("t", false, true, "", true, 3);
        let answer = serde_json::json!({ "blanks": ["答案1", "答案2"] });
        assert_eq!(
            tiku.extract_answer_by_type("FILL_IN_BLANK", &answer),
            Some("答案1\n答案2".to_string())
        );
    }
}
