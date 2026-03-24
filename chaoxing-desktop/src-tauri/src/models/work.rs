use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuestionType {
    Single,
    Multiple,
    Completion,
    Judgement,
    #[serde(rename = "shortanswer")]
    ShortAnswer,
    Unknown,
}

impl QuestionType {
    pub fn from_code(code: &str) -> Self {
        match code {
            "0" => Self::Single,
            "1" => Self::Multiple,
            "2" => Self::Completion,
            "3" => Self::Judgement,
            "4" => Self::ShortAnswer,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub id: String,
    pub title: String,
    pub options: String,
    pub question_type: QuestionType,
    pub answer_field: HashMap<String, String>,
}
