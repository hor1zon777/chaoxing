use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct Account {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResult {
    pub success: bool,
    pub message: String,
    pub uid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedAccountSummary {
    pub account_id: String,
    pub display_name: String,
    pub uid: Option<String>,
    pub login_type: String,
    pub last_used_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
}

impl StoredCookie {
    pub fn fingerprint(&self) -> String {
        format!("{}={};{}{}", self.name, self.value, self.domain, self.path)
    }
}

impl SavedAccountSummary {
    pub fn plain_display_name(&self) -> String {
        self.display_name
            .split_once(" (")
            .map(|(value, _)| value.to_string())
            .unwrap_or_else(|| self.display_name.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedAccountRecord {
    pub account_id: String,
    pub display_name: String,
    pub uid: Option<String>,
    pub login_type: String,
    pub last_used_at: String,
    pub encrypted_cookies: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SavedAccountVault {
    pub accounts: Vec<SavedAccountRecord>,
}

impl SavedAccountRecord {
    pub fn to_summary(&self) -> SavedAccountSummary {
        SavedAccountSummary {
            account_id: self.account_id.clone(),
            display_name: self.display_name.clone(),
            uid: self.uid.clone(),
            login_type: self.login_type.clone(),
            last_used_at: self.last_used_at.clone(),
        }
    }
}

