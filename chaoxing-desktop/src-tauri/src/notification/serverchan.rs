use crate::error::AppError;

pub struct ServerChan {
    url: String,
    client: reqwest::Client,
}

impl ServerChan {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn send(&self, message: &str) -> Result<(), AppError> {
        let params = serde_json::json!({
            "text": message,
            "desp": message,
        });
        let resp = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json;charset=utf-8")
            .json(&params)
            .send()
            .await?;
        if resp.status().is_success() {
            tracing::info!("Server酱通知发送成功");
        } else {
            tracing::error!("Server酱通知发送失败: {}", resp.status());
        }
        Ok(())
    }
}
