use std::sync::atomic::Ordering;

use tauri::State;

use crate::api::auth;
use crate::api::client::HttpClient;
use crate::error::AppError;
use crate::models::account::{LoginResult, SavedAccountSummary};
use crate::state::AppState;
use crate::utils::account_vault::{
    build_display_name,
    build_last_used_at,
    decrypt_cookie_pairs,
    delete_account_record,
    delete_account_vault_file,
    find_account_record,
    list_account_summaries,
    load_account_vault,
    save_account_vault,
    upsert_account_record,
};

fn persist_client_account(
    client: &HttpClient,
    uid: Option<&str>,
    login_type: &str,
    display_name: &str,
) -> Result<SavedAccountSummary, AppError> {
    let mut vault = load_account_vault()?;
    let cookies = client.export_cookies();
    let summary = upsert_account_record(&mut vault, &cookies, uid, login_type, display_name)?;
    save_account_vault(&vault)?;
    Ok(summary)
}

fn restore_client_from_account(account_id: &str) -> Result<Option<HttpClient>, AppError> {
    let vault = load_account_vault()?;
    let Some(record) = find_account_record(&vault, account_id) else {
        return Ok(None);
    };

    let cookies = decrypt_cookie_pairs(&record.encrypted_cookies)?;
    let client = HttpClient::new();
    client.import_cookies(&cookies).map_err(AppError::Other)?;
    Ok(Some(client))
}

fn build_login_success(display_name: String, uid: Option<String>) -> LoginResult {
    LoginResult {
        success: true,
        message: display_name,
        uid,
    }
}

fn build_login_failure(message: &str) -> LoginResult {
    LoginResult {
        success: false,
        message: message.to_string(),
        uid: None,
    }
}

fn load_saved_account_summary(account_id: &str) -> Result<Option<SavedAccountSummary>, AppError> {
    let vault = load_account_vault()?;
    Ok(find_account_record(&vault, account_id).map(|record| record.to_summary()))
}

fn plain_display_name(display_name: &str) -> String {
    display_name
        .split_once(" (")
        .map(|(value, _)| value.to_string())
        .unwrap_or_else(|| display_name.to_string())
}

fn mark_saved_account_as_used(account_id: &str, uid: Option<&str>) -> Result<(), AppError> {
    let mut vault = load_account_vault()?;
    let Some(record) = vault
        .accounts
        .iter_mut()
        .find(|item| item.account_id == account_id)
    else {
        return Ok(());
    };

    record.last_used_at = build_last_used_at();
    if let Some(uid) = uid {
        record.uid = Some(uid.to_string());
        let fallback_name = plain_display_name(&record.display_name);
        record.display_name = build_display_name(&record.login_type, Some(uid), &fallback_name);
    }
    save_account_vault(&vault)
}

async fn set_active_session(
    state: &State<'_, AppState>,
    client: HttpClient,
    account_id: Option<String>,
) {
    *state.client.write().await = Some(client);
    *state.active_account_id.write().await = account_id;
}

async fn clear_active_session(state: &State<'_, AppState>) {
    *state.client.write().await = None;
    *state.active_account_id.write().await = None;
}

fn stop_running_tasks(state: &State<'_, AppState>) {
    state.is_running.store(false, Ordering::SeqCst);
    state.is_paused.store(false, Ordering::SeqCst);
}

async fn clear_active_session_and_tasks(state: &State<'_, AppState>) {
    stop_running_tasks(state);
    clear_active_session(state).await;
}

async fn is_active_account(state: &State<'_, AppState>, account_id: &str) -> bool {
    state
        .active_account_id
        .read()
        .await
        .as_deref()
        .is_some_and(|active_id| active_id == account_id)
}

/// 账号密码登录
#[tauri::command]
pub async fn login(
    state: State<'_, AppState>,
    username: String,
    password: String,
) -> Result<LoginResult, AppError> {
    let client = HttpClient::new();
    let result = auth::login(&client, &username, &password).await?;
    if !result.success {
        return Ok(result);
    }

    let uid = client.get_uid();
    let summary = persist_client_account(&client, uid.as_deref(), "password", &username)?;
    set_active_session(&state, client, Some(summary.account_id.clone())).await;
    Ok(build_login_success(summary.display_name, uid))
}

/// Cookie 登录
#[tauri::command]
pub async fn login_with_cookies(
    state: State<'_, AppState>,
    cookies_text: String,
) -> Result<LoginResult, AppError> {
    // 防御性长度限制：浏览器 cookie header 通常 < 8KB；超过 64KB 视为异常粘贴
    const MAX_COOKIES_LEN: usize = 65_536;
    if cookies_text.len() > MAX_COOKIES_LEN {
        return Err(AppError::Other(format!(
            "Cookie 文本过长（{} 字节），请确认是否复制了无关内容",
            cookies_text.len()
        )));
    }
    if cookies_text.trim().is_empty() {
        return Err(AppError::Other("Cookie 文本为空".to_string()));
    }

    let client = HttpClient::new();

    // 过滤控制字符，避免请求头注入风险（按 RFC 7230，header value 中
    // 不应出现除 HTAB 外的 ASCII 控制字符）
    let is_valid_cookie_char = |c: char| !c.is_control() || c == '\t';

    let cookies = cookies_text
        .split(';')
        .filter_map(|pair| {
            let pair = pair.trim();
            if pair.is_empty() {
                return None;
            }
            let (name, value) = pair.split_once('=')?;
            let name = name.trim();
            let value = value.trim();
            if name.is_empty() {
                return None;
            }
            if !name.chars().all(is_valid_cookie_char)
                || !value.chars().all(is_valid_cookie_char)
            {
                tracing::warn!("已跳过包含控制字符的 cookie 项: {}", name);
                return None;
            }
            Some(crate::models::account::StoredCookie {
                name: name.to_string(),
                value: value.to_string(),
                domain: ".chaoxing.com".to_string(),
                path: "/".to_string(),
            })
        })
        .collect::<Vec<_>>();

    if cookies.is_empty() {
        return Err(AppError::Other("未解析到任何有效 cookie".to_string()));
    }

    client.import_cookies(&cookies).map_err(AppError::Other)?;

    let valid = auth::validate_cookie_session(&client).await?;
    if !valid {
        clear_active_session_and_tasks(&state).await;
        return Ok(build_login_failure("Cookie 已失效，请重新登录"));
    }

    let uid = client.get_uid();
    let summary = persist_client_account(&client, uid.as_deref(), "cookie", "Cookie 登录")?;
    set_active_session(&state, client, Some(summary.account_id.clone())).await;
    Ok(build_login_success(summary.display_name, uid))
}

#[tauri::command]
pub async fn list_saved_accounts() -> Result<Vec<SavedAccountSummary>, AppError> {
    let vault = load_account_vault()?;
    Ok(list_account_summaries(&vault))
}

#[tauri::command]
pub async fn login_with_saved_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<LoginResult, AppError> {
    let Some(client) = restore_client_from_account(&account_id)? else {
        clear_active_session_and_tasks(&state).await;
        return Ok(build_login_failure("未找到指定账户"));
    };

    let valid = auth::validate_cookie_session(&client).await?;
    if !valid {
        clear_active_session_and_tasks(&state).await;
        return Ok(build_login_failure("账户已失效，请重新登录或删除"));
    }

    let uid = client.get_uid();
    mark_saved_account_as_used(&account_id, uid.as_deref())?;
    let display_name = load_saved_account_summary(&account_id)?
        .map(|summary| summary.display_name)
        .unwrap_or_else(|| build_display_name("saved", uid.as_deref(), "已保存账户"));

    set_active_session(&state, client, Some(account_id)).await;
    Ok(build_login_success(display_name, uid))
}

#[tauri::command]
pub async fn delete_saved_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<(), AppError> {
    let was_active = is_active_account(&state, &account_id).await;
    let mut vault = load_account_vault()?;
    delete_account_record(&mut vault, &account_id);

    if vault.accounts.is_empty() {
        delete_account_vault_file()?;
    } else {
        save_account_vault(&vault)?;
    }

    if was_active {
        clear_active_session_and_tasks(&state).await;
    }

    Ok(())
}

/// 登出（仅退出当前活动会话，不删除已保存账户）
#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), AppError> {
    clear_active_session_and_tasks(&state).await;
    Ok(())
}

/// 验证当前活动会话是否有效（不自动恢复其他账户）
#[tauri::command]
pub async fn validate_session(state: State<'_, AppState>) -> Result<bool, AppError> {
    let valid = {
        let lock = state.client.read().await;
        match lock.as_ref() {
            Some(client) => auth::validate_cookie_session(client).await?,
            None => false,
        }
    };

    if !valid {
        clear_active_session(&state).await;
    }

    Ok(valid)
}
