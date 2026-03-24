use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("网络请求失败: {0}")]
    Network(#[from] reqwest::Error),
    #[error("数据解析失败: {0}")]
    Parse(String),
    #[error("登录失败: {0}")]
    Login(String),
    #[error("认证过期")]
    Unauthorized,
    #[error("配置错误: {0}")]
    Config(String),
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON错误: {0}")]
    Json(#[from] serde_json::Error),
    #[error("任务取消")]
    Cancelled,
    #[error("{0}")]
    Other(String),
}

// Tauri v2 要求 command 返回的 Error 实现 Serialize
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
