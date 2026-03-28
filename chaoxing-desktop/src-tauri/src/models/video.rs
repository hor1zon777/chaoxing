use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StudyResult {
    Success,
    Forbidden,
    Error,
    Timeout,
    Cancelled,
}

impl StudyResult {
    pub fn is_success(&self) -> bool {
        matches!(self, StudyResult::Success)
    }

    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}
