//! 验证码处理
//!
//! 对应 Python: api/captcha.py (CxCaptcha)
//! Phase 4 实现前端手动输入方案，后续可集成 OCR

use crate::api::client::HttpClient;
use crate::error::AppError;
use rand::Rng;

/// 获取验证码图片 (PNG bytes)
pub async fn get_captcha(client: &HttpClient) -> Result<Vec<u8>, AppError> {
    let t: u32 = rand::thread_rng().gen_range(0..2147483647);
    let url = format!(
        "https://mooc1.chaoxing.com/processVerifyPng.ac?t={}",
        t
    );
    let resp = client
        .client
        .get(&url)
        .send()
        .await?;
    if resp.status().is_success() {
        Ok(resp.bytes().await?.to_vec())
    } else {
        Err(AppError::Other(format!("获取验证码失败: {}", resp.status())))
    }
}

/// 提交验证码
/// 成功返回 true：Python 原版以 HTTP 302 重定向为成功标志，
/// 这里使用不跟随重定向的客户端避免被跟到登录页后用 200 误判成功
pub async fn submit_captcha(
    client: &HttpClient,
    code: &str,
) -> Result<bool, AppError> {
    let url = format!(
        "https://mooc1.chaoxing.com/html/processVerify.ac?ucode={}&app=0",
        code
    );
    let no_redirect = client.client_builder_no_redirect();
    let resp = no_redirect.get(&url).send().await?;
    // 仅 3xx（典型为 302）视为成功；2xx / 4xx / 5xx 均视为失败
    Ok(resp.status().is_redirection())
}
