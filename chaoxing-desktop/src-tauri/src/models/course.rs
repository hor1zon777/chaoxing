use serde::{Deserialize, Serialize};

use crate::models::job::JobType;

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
    pub course_type: CourseType,
    pub course_type_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseFolder {
    pub id: String,
    pub rename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CourseType {
    Course,
    Archive,
    Other,
}

impl CourseType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Course => "课程",
            Self::Archive => "归档",
            Self::Other => "其他",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseJob {
    pub id: String,
    pub name: String,
    pub job_type: JobType,
    pub type_label: String,
    pub is_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterSelectionPoint {
    pub id: String,
    pub title: String,
    pub job_count: u32,
    pub has_finished: bool,
    pub need_unlock: bool,
    pub jobs: Vec<CourseJob>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseSelectionTree {
    pub has_locked: bool,
    pub points: Vec<ChapterSelectionPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoursePointSelection {
    pub point_id: String,
    pub selected_job_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseTaskSelection {
    pub course_id: String,
    pub clazz_id: String,
    pub cpi: String,
    pub title: String,
    pub selected_points: Vec<CoursePointSelection>,
}
