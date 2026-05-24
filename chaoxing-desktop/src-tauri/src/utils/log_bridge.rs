//! 日志桥接：将 tracing 事件转发到前端 TaskEvent 通道
//!
//! 用法：
//! 1. `set_log_channel(tx)` — 任务开始前调用
//! 2. `clear_log_channel()` — 任务结束后调用
//! 3. 所有 tracing 事件自动出现在前端日志中

use std::sync::RwLock;
use tokio::sync::mpsc;
use tracing::Level;
use tracing_subscriber::Layer;

use crate::models::events::TaskEvent;

/// 全局日志发送器（任务执行期间设置）
///
/// 用 RwLock 替代 Mutex：日志路径是"高频读 + 低频写"，
/// RwLock 允许多线程同时持读锁，避免高吞吐日志下的互斥竞争；
/// 同时 set / clear 通过 try_write 拿写锁，避免在 tracing 事件回调
/// 内部触发的递归 tracing 调用造成的死锁链路。
static LOG_TX: RwLock<Option<mpsc::UnboundedSender<TaskEvent>>> = RwLock::new(None);

/// 设置日志通道（任务开始时调用）
pub fn set_log_channel(tx: mpsc::UnboundedSender<TaskEvent>) {
    if let Ok(mut guard) = LOG_TX.write() {
        *guard = Some(tx);
    }
}

/// 清除日志通道（任务结束时调用）
pub fn clear_log_channel() {
    if let Ok(mut guard) = LOG_TX.write() {
        *guard = None;
    }
}

/// 发送日志事件到前端
fn send_log(level: &str, message: String) {
    // 用 try_read：当写锁正持有时跳过本条日志，防止递归 tracing
    // （例如 set/clear 内部 tracing 调用）触发的死锁
    let guard = match LOG_TX.try_read() {
        Ok(g) => g,
        Err(_) => return,
    };
    if let Some(tx) = guard.as_ref() {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        let _ = tx.send(TaskEvent::Log {
            level: level.to_string(),
            message,
            timestamp,
        });
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
