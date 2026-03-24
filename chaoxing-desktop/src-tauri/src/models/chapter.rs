use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterPoint {
    pub id: String,
    pub title: String,
    pub job_count: u32,
    pub has_finished: bool,
    pub need_unlock: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterTree {
    pub has_locked: bool,
    pub points: Vec<ChapterPoint>,
}
