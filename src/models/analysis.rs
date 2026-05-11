use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioAnalysis {
    pub id: i64,
    pub file_path: Option<String>,
    pub file_hash: Option<String>,
    pub analysis_type: String,
    pub results_json: String,
    pub analyzed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BpmAnalysisResult {
    pub bpm: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyAnalysisResult {
    pub key: String,
    pub confidence: f32,
}
