use crate::error::AppError;

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
            client: reqwest::Client::new(),
        }
    }

    pub async fn send(&self, message: &str) -> Result<(), AppError> {
        let params = [
            ("chat_id", self.chat_id.as_str()),
            ("text", message),
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
            } else {
                tracing::error!("Telegram通知发送失败: {:?}", json);
            }
        } else {
            tracing::error!("Telegram通知发送失败: {}", resp.status());
        }
        Ok(())
    }
}
