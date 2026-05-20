//! 日志桥接：将 tracing 事件转发到前端 TaskEvent 通道
//!
//! 用法：
//! 1. `set_log_channel(tx)` — 任务开始前调用
//! 2. `clear_log_channel()` — 任务结束后调用
//! 3. 所有 tracing 事件自动出现在前端日志中

use std::sync::Mutex;
use tokio::sync::mpsc;
use tracing::Level;
use tracing_subscriber::Layer;

use crate::models::events::TaskEvent;

/// 全局日志发送器（任务执行期间设置）
static LOG_TX: Mutex<Option<mpsc::UnboundedSender<TaskEvent>>> = Mutex::new(None);

/// 设置日志通道（任务开始时调用）
pub fn set_log_channel(tx: mpsc::UnboundedSender<TaskEvent>) {
    if let Ok(mut guard) = LOG_TX.lock() {
        *guard = Some(tx);
    }
}

/// 清除日志通道（任务结束时调用）
pub fn clear_log_channel() {
    if let Ok(mut guard) = LOG_TX.lock() {
        *guard = None;
    }
}

/// 发送日志事件到前端
fn send_log(level: &str, message: String) {
    if let Ok(guard) = LOG_TX.lock() {
        if let Some(tx) = guard.as_ref() {
            let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
            let _ = tx.send(TaskEvent::Log {
                level: level.to_string(),
                message,
                timestamp,
            });
        }
    }
}

/// 自定义 tracing Layer：将所有事件转发到前端
pub struct FrontendLogLayer;

impl<S> Layer<S> for FrontendLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let level = match *event.metadata().level() {
            Level::ERROR => "error",
            Level::WARN => "warn",
            Level::INFO => "info",
            Level::DEBUG => "debug",
            Level::TRACE => "trace",
        };

        let mut visitor = StringVisitor(String::new());
        event.record(&mut visitor);

        if visitor.0.is_empty() {
            return;
        }

        send_log(level, visitor.0);
    }
}

/// 字符串访问器，用于提取 tracing 事件的消息内容
struct StringVisitor(String);

impl tracing::field::Visit for StringVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0 = value.to_string();
        }
    }
}
