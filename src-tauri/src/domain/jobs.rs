use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobSummary {
    pub id: String,
    pub kind: JobKind,
    pub status: JobStatus,
    pub name: String,
    pub total_files: u64,
    pub completed_files: u64,
    pub created_at_unix_ms: i64,
    pub updated_at_unix_ms: i64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JobKind {
    OrganizerScan,
    OrganizerApply,
    OrganizerUndo,
    DuplicateAnalysis,
    Conversion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Canceled,
    PartiallyCompleted,
}
