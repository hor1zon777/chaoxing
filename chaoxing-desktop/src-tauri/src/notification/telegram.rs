use crate::error::AppError;

/// 转义 Telegram HTML parse_mode 的特殊字符
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub struct Telegram {
    url: String,
    chat_id: String,
    client: reqwest::Client,
}

impl Telegram {
    pub fn new(url: &str, chat_id: &str) -> Self {
        Self {
            url: url.to_string(),
            chat_id: chat_id.to_string(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn send(&self, message: &str) -> Result<(), AppError> {
        let escaped = escape_html(message);
        let params = [
            ("chat_id", self.chat_id.as_str()),
            ("text", escaped.as_str()),
            ("parse_mode", "HTML"),
        ];
        let resp = self
            .client
            .post(&self.url)
            .form(&params)
            .send()
            .await?;
        if resp.status().is_success() {
            let json: serde_json::Value = resp.json().await.unwrap_or_default();
            if json["ok"].as_bool() == Some(true) {
                tracing::info!("Telegram通知发送成功");
                Ok(())
            } else {
                let desc = json["description"]
                    .as_str()
                    .unwrap_or("未知错误");
                tracing::error!("Telegram通知发送失败: {}", desc);
                Err(AppError::Notification(format!(
                    "Telegram通知发送失败: {}",
                    desc
                )))
            }
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::error!("Telegram通知发送失败: {} - {}", status, body);
            Err(AppError::Notification(format!(
                "Telegram通知发送失败: HTTP {}",
                status
            )))
        }
    }
}
