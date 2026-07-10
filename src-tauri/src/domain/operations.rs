use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationPlan {
    pub id: String,
    pub job_id: Option<String>,
    pub status: OperationPlanStatus,
    pub operations: Vec<PlannedOperation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedOperation {
    pub id: String,
    pub kind: OperationKind,
    pub source_path: String,
    pub target_path: String,
    pub conflict_policy: ConflictPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OperationKind {
    Move,
    Rename,
    Convert,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OperationPlanStatus {
    Preview,
    Applying,
    Applied,
    Failed,
    Undone,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConflictPolicy {
    Skip,
    Rename,
    Overwrite,
}
