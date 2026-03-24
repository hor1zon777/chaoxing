use crate::error::AppError;

pub struct Bark {
    url: String,
    client: reqwest::Client,
}

impl Bark {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn send(&self, message: &str) -> Result<(), AppError> {
        let resp = self
            .client
            .post(&self.url)
            .query(&[("body", message)])
            .send()
            .await?;
        if resp.status().is_success() {
            tracing::info!("Bark通知发送成功");
        } else {
            tracing::error!("Bark通知发送失败: {}", resp.status());
        }
        Ok(())
    }
}
