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
/// 成功返回 true（Python 中检测 302 重定向）
pub async fn submit_captcha(
    client: &HttpClient,
    code: &str,
) -> Result<bool, AppError> {
    let url = format!(
        "https://mooc1.chaoxing.com/html/processVerify.ac?ucode={}&app=0",
        code
    );
    let resp = client
        .client
        .get(&url)
        .send()
        .await?;
    // Python 中成功标志是 HTTP 302 重定向
    // reqwest 默认跟随重定向，所以检查最终 URL 或状态
    Ok(resp.status().is_success())
}
