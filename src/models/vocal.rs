use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VocalProductionNotes {
    pub id: i64,
    pub project_id: i64,
    pub mic_choice: Option<String>,
    pub vocal_chain_json: Option<String>,
    pub recording_notes: Option<String>,
    pub editing_notes: Option<String>,
    pub tuning_notes: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceVocal {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub bpm: Option<i32>,
    pub key: Option<String>,
    pub duration: Option<f64>,
}
