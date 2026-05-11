use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lyrics {
    pub id: i64,
    pub project_id: i64,
    pub content: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricAnnotation {
    pub id: i64,
    pub lyrics_id: i64,
    pub start_index: i32,
    pub end_index: i32,
    pub tag: AnnotationTag,
    pub color: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnnotationTag {
    Melody,
    AdLib,
    Harmony,
    Flow,
    Emphasis,
    Note,
}
