use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    pub id: String,
    pub course_id: String,
    pub clazz_id: String,
    pub cpi: String,
    pub title: String,
    pub teacher: String,
    pub desc: String,
    pub info: String,
    pub roleid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseFolder {
    pub id: String,
    pub rename: String,
}
