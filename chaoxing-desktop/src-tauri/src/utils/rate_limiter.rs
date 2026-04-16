use rand::Rng;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

#[derive(Clone)]
pub struct RateLimiter {
    last_call: Arc<Mutex<Instant>>,
    call_interval: Duration,
}

impl RateLimiter {
    pub fn new(interval_secs: f64) -> Self {
        Self {
            last_call: Arc::new(Mutex::new(Instant::now())),
            call_interval: Duration::from_secs_f64(interval_secs),
        }
    }

    /// 限速: 确保两次调用之间至少间隔 call_interval
    pub async fn limit_rate(&self) {
        let sleep_duration = {
            let mut last = self.last_call.lock().await;
            let elapsed = last.elapsed();
            if elapsed < self.call_interval {
                let duration = self.call_interval - elapsed;
                *last = Instant::now() + duration;
                Some(duration)
            } else {
                *last = Instant::now();
                None
            }
        };
        if let Some(d) = sleep_duration {
            tokio::time::sleep(d).await;
        }
    }

    /// 随机限速: 在 [min, max] 秒范围内随机等待后再执行限速
    pub async fn limit_rate_random(&self, min: f64, max: f64) {
        let wait = rand::thread_rng().gen_range(min..max);
        tokio::time::sleep(Duration::from_secs_f64(wait)).await;
        self.limit_rate().await;
    }
}
