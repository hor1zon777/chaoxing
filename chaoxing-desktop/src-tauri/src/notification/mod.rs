pub mod serverchan;
pub mod qmsg;
pub mod bark;
pub mod telegram;

use crate::error::AppError;
use crate::models::config::AppConfig;

/// 通知服务 trait
pub trait NotificationService: Send + Sync {
    fn name(&self) -> &str;
    fn send(&self, message: &str) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
}

/// 通知服务 enum 分发
pub enum NotificationProvider {
    ServerChan(serverchan::ServerChan),
    Qmsg(qmsg::Qmsg),
    Bark(bark::Bark),
    Telegram(telegram::Telegram),
    Disabled,
}

fn normalize_serverchan_url(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }

    if trimmed.is_empty() {
        return String::new();
    }

    format!("https://sctapi.ftqq.com/{trimmed}.send")
}

fn normalize_qmsg_url(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }

    if trimmed.is_empty() {
        return String::new();
    }

    format!("https://qmsg.zendee.cn/send/{trimmed}")
}

fn normalize_bark_url(value: &str) -> String {
    let trimmed = value.trim().trim_matches('/');
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }

    if trimmed.is_empty() {
        return String::new();
    }

    format!("https://api.day.app/{trimmed}")
}

fn normalize_telegram_url(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }

    if trimmed.is_empty() {
        return String::new();
    }

    format!("https://api.telegram.org/bot{trimmed}/sendMessage")
}

impl NotificationProvider {
    /// 从配置创建
    pub fn from_config(config: &AppConfig) -> Self {
        let provider = config.notification_provider.trim().to_ascii_lowercase();
        let raw_url = config.notification_url.trim();
        if raw_url.is_empty() {
            return Self::Disabled;
        }

        match provider.as_str() {
            "serverchan" => {
                let url = normalize_serverchan_url(raw_url);
                if url.is_empty() {
                    Self::Disabled
                } else {
                    Self::ServerChan(serverchan::ServerChan::new(&url))
                }
            }
            "qmsg" => {
                let url = normalize_qmsg_url(raw_url);
                if url.is_empty() {
                    Self::Disabled
                } else {
                    Self::Qmsg(qmsg::Qmsg::new(&url))
                }
            }
            "bark" => {
                let url = normalize_bark_url(raw_url);
                if url.is_empty() {
                    Self::Disabled
                } else {
                    Self::Bark(bark::Bark::new(&url))
                }
            }
            "telegram" => {
                let url = normalize_telegram_url(raw_url);
                if url.is_empty() {
                    Self::Disabled
                } else {
                    Self::Telegram(telegram::Telegram::new(
                        &url,
                        &config.notification_tg_chat_id,
                    ))
                }
            }
            _ => Self::Disabled,
        }
    }

    /// 发送通知
    pub async fn send(&self, message: &str) -> Result<(), AppError> {
        match self {
            Self::ServerChan(s) => s.send(message).await,
            Self::Qmsg(s) => s.send(message).await,
            Self::Bark(s) => s.send(message).await,
            Self::Telegram(s) => s.send(message).await,
            Self::Disabled => Ok(()),
        }
    }
}
