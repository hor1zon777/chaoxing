use crate::error::AppError;

pub struct Qmsg {
    url: String,
    client: reqwest::Client,
}

impl Qmsg {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
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
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::error!("Qmsg酱通知发送失败: {} - {}", status, body);
            Err(AppError::Notification(format!(
                "Qmsg酱通知发送失败: HTTP {}",
                status
            )))
        }
    }
}
