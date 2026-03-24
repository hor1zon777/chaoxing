//! 登录认证 API
//!
//! 精确复刻 Python api/base.py 中 Chaoxing.login() 和
//! Chaoxing._validate_cookie_session() 的逻辑。

use crate::api::client::HttpClient;
use crate::crypto::aes::encrypt;
use crate::error::AppError;
use crate::models::account::LoginResult;

/// 账号密码登录
/// 对应 Python: Chaoxing.login(login_with_cookies=False)
pub async fn login(
    client: &HttpClient,
    username: &str,
    password: &str,
) -> Result<LoginResult, AppError> {
    let encrypted_username = encrypt(username);
    let encrypted_password = encrypt(password);

    let params = [
        ("fid", "-1"),
        ("uname", &encrypted_username),
        ("password", &encrypted_password),
        ("refer", "https%3A%2F%2Fi.chaoxing.com"),
        ("t", "True"),
        ("forbidotherlogin", "0"),
        ("validate", ""),
        ("doubleFactorLogin", "0"),
        ("independentId", "0"),
    ];

    let resp = client
        .client
        .post("https://passport2.chaoxing.com/fanyalogin")
        .form(&params)
        .send()
        .await?;

    let json: serde_json::Value = resp.json().await?;

    if json["status"].as_bool() == Some(true) {
        let uid = client.get_uid();
        Ok(LoginResult {
            success: true,
            message: "登录成功".to_string(),
            uid,
        })
    } else {
        let msg = json["msg2"]
            .as_str()
            .unwrap_or("登录失败")
            .to_string();
        Ok(LoginResult {
            success: false,
            message: msg,
            uid: None,
        })
    }
}

/// 验证 Cookie 会话有效性
/// 对应 Python: Chaoxing._validate_cookie_session()
pub async fn validate_cookie_session(client: &HttpClient) -> Result<bool, AppError> {
    // 检查 _uid cookie 是否存在
    if client.get_uid().is_none() {
        return Ok(false);
    }

    let params = [
        ("courseType", "1"),
        ("courseFolderId", "0"),
        ("query", ""),
        ("superstarClass", "0"),
    ];

    let resp = client
        .client
        .post("https://mooc2-ans.chaoxing.com/mooc2-ans/visit/courselistdata")
        .form(&params)
        .timeout(std::time::Duration::from_secs(8))
        .send()
        .await;

    match resp {
        Ok(resp) => {
            if resp.status() != 200 {
                return Ok(false);
            }
            let text = resp.text().await.unwrap_or_default();
            // 如果响应包含登录页面特征，说明 cookie 已过期
            if text.contains("passport2.chaoxing.com")
                || text.to_lowercase().contains("login")
            {
                return Ok(false);
            }
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_params_deterministic() {
        // 确保加密参数是确定性的
        let e1 = encrypt("test_user");
        let e2 = encrypt("test_user");
        assert_eq!(e1, e2);
        assert!(!e1.is_empty());
    }
}
