use crate::error::AppError;

pub struct Qmsg {
    url: String,
    client: reqwest::Client,
}

impl Qmsg {
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
            .header("Content-Type", "application/json;charset=utf-8")
            .query(&[("msg", message)])
            .send()
            .await?;
        if resp.status().is_success() {
            tracing::info!("Qmsg酱通知发送成功");
        } else {
            tracing::error!("Qmsg酱通知发送失败: {}", resp.status());
        }
        Ok(())
    }
}
