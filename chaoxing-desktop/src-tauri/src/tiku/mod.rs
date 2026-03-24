//! 题库系统
//!
//! 对应 Python Tiku 基类 + 各题库实现
//! 采用 enum 分发模式（与 notification 模块一致），
//! 避免 dyn trait 的复杂性。

pub mod adapter;
pub mod ai;
pub mod answer_check;
pub mod cache;
pub mod like;
pub mod siliconflow;
pub mod yanxi;

use std::sync::OnceLock;

use regex::Regex;
use tracing;

use crate::models::config::AppConfig;

/// 缓存编译后的正则
static RE_NUM_PREFIX: OnceLock<Regex> = OnceLock::new();
static RE_SCORE_SUFFIX: OnceLock<Regex> = OnceLock::new();

fn re_num_prefix() -> &'static Regex {
    RE_NUM_PREFIX.get_or_init(|| Regex::new(r"^\d+").unwrap())
}

fn re_score_suffix() -> &'static Regex {
    RE_SCORE_SUFFIX.get_or_init(|| {
        // 匹配中文括号包裹的分数后缀，如 "（1.0分）"
        Regex::new(r"\u{ff08}\d+\.\d+\u{5206}\u{ff09}$").unwrap()
    })
}

/// 题库查询结果
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub answer: Option<String>,
    pub source: String,
}

/// 题库提供者 enum 分发
///
/// 与 notification::NotificationProvider 模式一致
pub enum TikuProvider {
    Yanxi(yanxi::TikuYanxi),
    Like(like::TikuLike),
    Adapter(adapter::TikuAdapter),
    Ai(ai::TikuAi),
    SiliconFlow(siliconflow::TikuSiliconFlow),
    Disabled,
}

impl TikuProvider {
    /// 分发查询到具体实现
    pub async fn query(&self, title: &str, q_type: &str, options: &str) -> Option<String> {
        match self {
            Self::Yanxi(t) => t.query(title, q_type, options).await,
            Self::Like(t) => t.query(title, q_type, options).await,
            Self::Adapter(t) => t.query(title, q_type, options).await,
            Self::Ai(t) => t.query(title, q_type, options).await,
            Self::SiliconFlow(t) => t.query(title, q_type, options).await,
            Self::Disabled => None,
        }
    }

    /// 名称
    pub fn name(&self) -> &str {
        match self {
            Self::Yanxi(t) => t.name(),
            Self::Like(t) => t.name(),
            Self::Adapter(t) => t.name(),
            Self::Ai(t) => t.name(),
            Self::SiliconFlow(t) => t.name(),
            Self::Disabled => "已禁用",
        }
    }

    /// 检查连接（仅 AI 类题库需要）
    pub async fn check_connection(&self) -> bool {
        match self {
            Self::Ai(t) => t.check_connection().await,
            Self::SiliconFlow(t) => t.check_connection().await,
            _ => true,
        }
    }
}

/// 题库管理器
///
/// 对应 Python Tiku 基类的 query() 方法和配置管理
pub struct TikuManager {
    pub provider: TikuProvider,
    pub cache: cache::CacheDAO,
    pub disabled: bool,
    pub submit: bool,
    pub cover_rate: f64,
    pub true_list: Vec<String>,
    pub false_list: Vec<String>,
    pub delay: f64,
}

impl TikuManager {
    /// 从配置创建题库管理器
    ///
    /// 对应 Python Tiku.get_tiku_from_config() + Tiku.init_tiku()
    pub fn from_config(config: &AppConfig) -> Self {
        let provider_name = config.tiku_provider.trim();
        let disabled = provider_name.is_empty();

        let provider = if disabled {
            TikuProvider::Disabled
        } else {
            match provider_name {
                "TikuYanxi" => TikuProvider::Yanxi(yanxi::TikuYanxi::new(&config.tiku_tokens)),
                "TikuLike" => TikuProvider::Like(like::TikuLike::new(
                    &config.tiku_tokens,
                    config.like_search,
                    config.like_vision,
                    &config.like_model,
                    config.like_retry,
                    config.like_retry_times,
                )),
                "TikuAdapter" => {
                    TikuProvider::Adapter(adapter::TikuAdapter::new(&config.tiku_adapter_url))
                }
                "AI" => TikuProvider::Ai(ai::TikuAi::new(
                    &config.ai_endpoint,
                    &config.ai_key,
                    &config.ai_model,
                    &config.ai_proxy,
                    config.ai_min_interval,
                )),
                "SiliconFlow" => TikuProvider::SiliconFlow(siliconflow::TikuSiliconFlow::new(
                    &config.siliconflow_endpoint,
                    &config.siliconflow_key,
                    &config.siliconflow_model,
                    config.ai_min_interval,
                )),
                other => {
                    tracing::error!("未知题库提供者: {}，题库功能已禁用", other);
                    TikuProvider::Disabled
                }
            }
        };

        let true_list: Vec<String> = config
            .true_list
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let false_list: Vec<String> = config
            .false_list
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // 缓存文件放在应用数据目录
        let cache_path = std::env::current_dir()
            .unwrap_or_default()
            .join("cache.json");

        Self {
            provider,
            cache: cache::CacheDAO::new(cache_path),
            disabled: disabled || matches!(&provider_name, &""),
            submit: config.tiku_submit,
            cover_rate: config.tiku_cover_rate,
            true_list,
            false_list,
            delay: config.tiku_delay,
        }
    }

    /// 查询题目答案（带缓存 + 答案验证）
    ///
    /// 对应 Python Tiku.query()
    /// 流程：预处理标题 -> 查缓存 -> 查题库 -> 验证答案 -> 缓存
    pub async fn query(&self, q_title: &str, q_type: &str, options: &str) -> QueryResult {
        if self.disabled {
            return QueryResult {
                answer: None,
                source: "disabled".to_string(),
            };
        }

        // 预处理标题：去除开头数字和分数后缀
        let title = preprocess_title(q_title);
        tracing::debug!("原始标题: {} -> 处理后: {}", q_title, title);

        // 先查缓存
        if let Some(cached) = self.cache.get(&title).await {
            tracing::info!("从缓存中获取答案: {} -> {}", title, cached);
            return QueryResult {
                answer: Some(cached.trim().to_string()),
                source: "cache".to_string(),
            };
        }

        // 查题库
        let result = self.provider.query(&title, q_type, options).await;
        if let Some(ref answer) = result {
            let answer = answer.trim().to_string();
            tracing::info!(
                "从{}获取答案: {} -> {}",
                self.provider.name(),
                title,
                answer
            );

            // 验证答案
            if answer_check::check_answer(&answer, q_type, &self.true_list, &self.false_list) {
                self.cache.set(&title, &answer).await;
                return QueryResult {
                    answer: Some(answer),
                    source: "tiku".to_string(),
                };
            } else {
                tracing::info!(
                    "从{}获取到的答案类型与题目类型不符，已舍弃",
                    self.provider.name()
                );
            }
        } else {
            tracing::error!("从{}获取答案失败: {}", self.provider.name(), title);
        }

        QueryResult {
            answer: None,
            source: "none".to_string(),
        }
    }

    /// 判断题答案选择
    ///
    /// 对应 Python Tiku.judgement_select()
    /// 将获取到的答案与 true_list / false_list 比对，返回布尔值
    pub fn judgement_select(&self, answer: &str) -> bool {
        let a = answer.trim();
        if self.true_list.iter().any(|t| t == a) {
            return true;
        }
        if self.false_list.iter().any(|f| f == a) {
            return false;
        }
        // 无法判断，随机选择
        tracing::error!(
            "无法判断答案 -> {} 对应的是正确还是错误，本次将随机选择",
            answer
        );
        rand::random()
    }

    /// 获取提交参数
    ///
    /// 对应 Python Tiku.get_submit_params()
    /// 留空直接提交，"1" 保存但不提交
    pub fn get_submit_flag(&self) -> &str {
        if self.submit {
            ""
        } else {
            "1"
        }
    }

    /// 检查 LLM 连接
    pub async fn check_connection(&self) -> bool {
        self.provider.check_connection().await
    }

    /// 初始化（LIKE 知识库需要获取余额）
    pub async fn init(&self) {
        if let TikuProvider::Like(ref like) = self.provider {
            like.init().await;
        }
    }
}

/// 预处理题目标题
///
/// 对应 Python Tiku.query() 中的预处理逻辑：
/// 1. 去除开头的数字序号
/// 2. 去除结尾的分数标记（如 "（1.0分）"）
fn preprocess_title(title: &str) -> String {
    let t = re_num_prefix().replace(title, "").to_string();
    re_score_suffix().replace(&t, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_title_removes_number_prefix() {
        assert_eq!(preprocess_title("1以下哪个是正确的"), "以下哪个是正确的");
        assert_eq!(preprocess_title("23题目内容"), "题目内容");
        assert_eq!(preprocess_title("100多选题"), "多选题");
    }

    #[test]
    fn test_preprocess_title_removes_score_suffix() {
        assert_eq!(
            preprocess_title("题目内容\u{ff08}1.0\u{5206}\u{ff09}"),
            "题目内容"
        );
        assert_eq!(
            preprocess_title("题目\u{ff08}2.5\u{5206}\u{ff09}"),
            "题目"
        );
    }

    #[test]
    fn test_preprocess_title_removes_both() {
        assert_eq!(
            preprocess_title("1题目内容\u{ff08}1.0\u{5206}\u{ff09}"),
            "题目内容"
        );
    }

    #[test]
    fn test_preprocess_title_no_change() {
        assert_eq!(preprocess_title("普通题目"), "普通题目");
    }

    #[test]
    fn test_from_config_disabled() {
        let config = AppConfig::default();
        let manager = TikuManager::from_config(&config);
        assert!(manager.disabled);
        assert!(matches!(manager.provider, TikuProvider::Disabled));
    }

    #[test]
    fn test_judgement_select_true() {
        let config = AppConfig::default();
        let manager = TikuManager::from_config(&config);
        assert!(manager.judgement_select("正确"));
        assert!(manager.judgement_select("对"));
    }

    #[test]
    fn test_judgement_select_false() {
        let config = AppConfig::default();
        let manager = TikuManager::from_config(&config);
        assert!(!manager.judgement_select("错误"));
        assert!(!manager.judgement_select("错"));
    }

    #[test]
    fn test_get_submit_flag() {
        let mut config = AppConfig::default();

        config.tiku_submit = true;
        let manager = TikuManager::from_config(&config);
        assert_eq!(manager.get_submit_flag(), "");

        config.tiku_submit = false;
        let manager2 = TikuManager::from_config(&config);
        assert_eq!(manager2.get_submit_flag(), "1");
    }
}
