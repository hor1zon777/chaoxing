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

impl NotificationProvider {
    /// 从配置创建
    pub fn from_config(config: &AppConfig) -> Self {
        let url = config.notification_url.trim();
        if url.is_empty() {
            return Self::Disabled;
        }
        match config.notification_provider.as_str() {
            "ServerChan" => Self::ServerChan(serverchan::ServerChan::new(url)),
            "Qmsg" => Self::Qmsg(qmsg::Qmsg::new(url)),
            "Bark" => Self::Bark(bark::Bark::new(url)),
            "Telegram" => Self::Telegram(telegram::Telegram::new(
                url,
                &config.notification_tg_chat_id,
            )),
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
