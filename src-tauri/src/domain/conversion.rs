use serde::{Deserialize, Serialize};

use crate::domain::operations::ConflictPolicy;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionRequest {
    pub input_paths: Vec<String>,
    pub output_directory: String,
    pub output_format: String,
    pub conflict_policy: ConflictPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionFileResult {
    pub input_path: String,
    pub output_path: Option<String>,
    pub status: ConversionFileStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConversionFileStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}
