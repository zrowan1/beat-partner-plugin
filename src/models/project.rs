use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub bpm: Option<i32>,
    pub key: Option<String>,
    pub genre: Option<String>,
    pub phase: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProject {
    pub name: String,
    pub bpm: Option<i32>,
    pub key: Option<String>,
    pub genre: Option<String>,
    pub phase: Option<String>,
    pub notes: Option<String>,
}
