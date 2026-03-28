use base64::{engine::general_purpose::STANDARD, Engine};

use crate::error::AppError;
use crate::models::account::{
    SavedAccountRecord,
    SavedAccountSummary,
    SavedAccountVault,
    StoredCookie,
};

const VAULT_FILE_NAME: &str = ".chaoxing-desktop-accounts.dat";

#[cfg(target_os = "windows")]
fn encrypt_bytes(data: &[u8]) -> Result<Vec<u8>, AppError> {
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{CryptProtectData, CRYPT_INTEGER_BLOB};

    unsafe {
        let input = CRYPT_INTEGER_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };
        let mut output = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: null_mut(),
        };

        if CryptProtectData(&input, null(), null(), null(), null(), 0, &mut output) == 0 {
            return Err(AppError::Other("加密账户 Cookie 失败".to_string()));
        }

        let encrypted = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
        LocalFree(output.pbData.cast());
        Ok(encrypted)
    }
}

#[cfg(target_os = "windows")]
fn decrypt_bytes(data: &[u8]) -> Result<Vec<u8>, AppError> {
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};

    unsafe {
        let input = CRYPT_INTEGER_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };
        let mut output = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: null_mut(),
        };

        if CryptUnprotectData(&input, null_mut(), null(), null(), null(), 0, &mut output) == 0 {
            return Err(AppError::Other("解密账户 Cookie 失败".to_string()));
        }

        let decrypted = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
        LocalFree(output.pbData.cast());
        Ok(decrypted)
    }
}

#[cfg(not(target_os = "windows"))]
fn encrypt_bytes(_data: &[u8]) -> Result<Vec<u8>, AppError> {
    Err(AppError::Other("当前平台暂不支持加密账户仓库".to_string()))
}

#[cfg(not(target_os = "windows"))]
fn decrypt_bytes(_data: &[u8]) -> Result<Vec<u8>, AppError> {
    Err(AppError::Other("当前平台暂不支持解密账户仓库".to_string()))
}

pub fn get_vault_file_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(std::path::PathBuf::from))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir()))
        .join(VAULT_FILE_NAME)
}

pub fn encrypt_cookie_pairs(cookies: &[StoredCookie]) -> Result<String, AppError> {
    let serialized = serde_json::to_vec(cookies)?;
    let encrypted = encrypt_bytes(&serialized)?;
    Ok(STANDARD.encode(encrypted))
}

pub fn decrypt_cookie_pairs(encrypted: &str) -> Result<Vec<StoredCookie>, AppError> {
    let encrypted_bytes = STANDARD
        .decode(encrypted)
        .map_err(|e| AppError::Other(format!("解码账户 Cookie 失败: {}", e)))?;
    let decrypted = decrypt_bytes(&encrypted_bytes)?;
    serde_json::from_slice(&decrypted).map_err(AppError::from)
}

pub fn load_account_vault() -> Result<SavedAccountVault, AppError> {
    let path = get_vault_file_path();
    if !path.exists() {
        return Ok(SavedAccountVault::default());
    }

    let content = std::fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok(SavedAccountVault::default());
    }

    let vault = serde_json::from_str::<SavedAccountVault>(&content)?;
    Ok(vault)
}

#[cfg(target_os = "windows")]
fn replace_file_atomically(source: &std::path::Path, target: &std::path::Path) -> Result<(), AppError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH};

    let source_wide = source
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>();
    let target_wide = target
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>();

    let result = unsafe {
        MoveFileExW(
            source_wide.as_ptr(),
            target_wide.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };

    if result == 0 {
        return Err(AppError::Io(std::io::Error::last_os_error()));
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn replace_file_atomically(source: &std::path::Path, target: &std::path::Path) -> Result<(), AppError> {
    std::fs::rename(source, target)?;
    Ok(())
}

pub fn save_account_vault(vault: &SavedAccountVault) -> Result<(), AppError> {
    use std::io::Write;

    let path = get_vault_file_path();
    let temp_path = path.with_extension("tmp");
    let content = serde_json::to_string_pretty(vault)?;

    let write_result = (|| -> Result<(), AppError> {
        let mut file = std::fs::File::create(&temp_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        drop(file);
        replace_file_atomically(&temp_path, &path)
    })();

    if write_result.is_err() && temp_path.exists() {
        let _ = std::fs::remove_file(&temp_path);
    }

    write_result
}

pub fn delete_account_vault_file() -> Result<(), AppError> {
    let path = get_vault_file_path();
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn build_account_id(uid: Option<&str>, cookies: &[StoredCookie]) -> String {
    uid.map(str::to_string).unwrap_or_else(|| {
        use md5_digest::{Digest, Md5};

        let source = cookies
            .iter()
            .map(StoredCookie::fingerprint)
            .collect::<Vec<_>>()
            .join(";");
        let mut hasher = Md5::new();
        hasher.update(source.as_bytes());
        format!("cookie:{:x}", hasher.finalize())
    })
}

pub fn build_display_name(login_type: &str, uid: Option<&str>, fallback: &str) -> String {
    uid.map(|value| format!("{} ({})", fallback, value))
        .unwrap_or_else(|| format!("{} ({})", login_type, fallback))
}

pub fn build_last_used_at() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn upsert_account_record(
    vault: &mut SavedAccountVault,
    cookies: &[StoredCookie],
    uid: Option<&str>,
    login_type: &str,
    display_name: &str,
) -> Result<SavedAccountSummary, AppError> {
    let account_id = build_account_id(uid, cookies);
    let encrypted_cookies = encrypt_cookie_pairs(cookies)?;
    let record = SavedAccountRecord {
        account_id: account_id.clone(),
        display_name: build_display_name(login_type, uid, display_name),
        uid: uid.map(str::to_string),
        login_type: login_type.to_string(),
        last_used_at: build_last_used_at(),
        encrypted_cookies,
    };

    if let Some(index) = vault.accounts.iter().position(|item| item.account_id == account_id) {
        vault.accounts[index] = record.clone();
    } else {
        vault.accounts.push(record.clone());
    }

    Ok(record.to_summary())
}

pub fn list_account_summaries(vault: &SavedAccountVault) -> Vec<SavedAccountSummary> {
    let mut summaries = vault
        .accounts
        .iter()
        .map(SavedAccountRecord::to_summary)
        .collect::<Vec<_>>();
    summaries.sort_by(|a, b| b.last_used_at.cmp(&a.last_used_at));
    summaries
}

pub fn find_account_record<'a>(
    vault: &'a SavedAccountVault,
    account_id: &str,
) -> Option<&'a SavedAccountRecord> {
    vault.accounts.iter().find(|item| item.account_id == account_id)
}

pub fn delete_account_record(vault: &mut SavedAccountVault, account_id: &str) {
    vault.accounts.retain(|item| item.account_id != account_id);
}
