use tauri::State;

use crate::api::auth;
use crate::api::client::HttpClient;
use crate::error::AppError;
use crate::models::account::LoginResult;
use crate::state::AppState;

/// 账号密码登录
#[tauri::command]
pub async fn login(
    state: State<'_, AppState>,
    username: String,
    password: String,
) -> Result<LoginResult, AppError> {
    let client = HttpClient::new();
    let result = auth::login(&client, &username, &password).await?;
    if result.success {
        *state.client.write().await = Some(client);
    }
    Ok(result)
}

/// Cookie 登录
#[tauri::command]
pub async fn login_with_cookies(
    state: State<'_, AppState>,
    cookies_text: String,
) -> Result<LoginResult, AppError> {
    let client = HttpClient::new();

    // 将 cookie 文本注入到 cookie store
    // 格式: k1=v1;k2=v2
    let url: url::Url = "https://mooc2-ans.chaoxing.com"
        .parse()
        .map_err(|e| AppError::Other(format!("URL 解析失败: {}", e)))?;
    {
        let mut store = client.cookie_store.lock().unwrap();
        for pair in cookies_text.split(';') {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }
            if let Some((name, value)) = pair.split_once('=') {
                // 构造符合 Set-Cookie 格式的字符串，插入 cookie store
                let set_cookie = format!(
                    "{}={}; Domain=.chaoxing.com; Path=/",
                    name.trim(),
                    value.trim()
                );
                // cookie_store::CookieStore::insert_raw 接受 &cookie::Cookie 和 &Url
                if let Ok(c) = cookie_store::RawCookie::parse(set_cookie) {
                    let _ = store.insert_raw(&c, &url);
                }
            }
        }
    }

    // 验证 cookie 是否有效
    let valid = auth::validate_cookie_session(&client).await?;
    if valid {
        let uid = client.get_uid();
        *state.client.write().await = Some(client);
        Ok(LoginResult {
            success: true,
            message: "Cookie 登录成功".to_string(),
            uid,
        })
    } else {
        Ok(LoginResult {
            success: false,
            message: "Cookie 已失效，请重新登录".to_string(),
            uid: None,
        })
    }
}

/// 登出
#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), AppError> {
    *state.client.write().await = None;
    Ok(())
}

/// 验证当前会话是否有效
#[tauri::command]
pub async fn validate_session(state: State<'_, AppState>) -> Result<bool, AppError> {
    let lock = state.client.read().await;
    match lock.as_ref() {
        Some(client) => auth::validate_cookie_session(client).await,
        None => Ok(false),
    }
}
