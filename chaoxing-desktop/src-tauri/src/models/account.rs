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
