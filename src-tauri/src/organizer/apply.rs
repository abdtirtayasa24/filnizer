use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::operations::{
    ConflictPolicy, OperationKind, OperationPlan, OperationPlanStatus,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyOrganizerPlanRequest {
    pub plan: OperationPlan,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyOrganizerPlanResponse {
    pub job_id: String,
    pub plan_id: String,
    pub results: Vec<ApplyFileResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyFileResult {
    pub operation_id: String,
    pub source_path: String,
    pub target_path: Option<String>,
    pub status: ApplyFileStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApplyFileStatus {
    Success,
    Failed,
    Skipped,
}

pub fn apply_organizer_plan(plan: &OperationPlan) -> ApplyOrganizerPlanResponse {
    let job_id = plan
        .job_id
        .clone()
        .unwrap_or_else(|| Uuid::now_v7().to_string());
    let mut results = Vec::new();

    for operation in &plan.operations {
        if !matches!(operation.kind, OperationKind::Move | OperationKind::Rename) {
            results.push(ApplyFileResult {
                operation_id: operation.id.clone(),
                source_path: operation.source_path.clone(),
                target_path: Some(operation.target_path.clone()),
                status: ApplyFileStatus::Failed,
                message: Some("Unsupported operation for organizer apply".to_string()),
            });
            continue;
        }

        let source_path = Path::new(&operation.source_path);
        let target_path = Path::new(&operation.target_path);
        results.push(apply_single_operation(
            &operation.id,
            source_path,
            target_path,
            &operation.conflict_policy,
        ));
    }

    ApplyOrganizerPlanResponse {
        job_id,
        plan_id: plan.id.clone(),
        results,
    }
}

pub fn applied_plan(original: &OperationPlan, job_id: String) -> OperationPlan {
    let mut plan = original.clone();
    plan.job_id = Some(job_id);
    plan.status = OperationPlanStatus::Applied;
    plan
}

fn apply_single_operation(
    operation_id: &str,
    source_path: &Path,
    target_path: &Path,
    conflict_policy: &ConflictPolicy,
) -> ApplyFileResult {
    if !source_path.exists() {
        return failed(
            operation_id,
            source_path,
            target_path,
            "Source file does not exist",
        );
    }

    if target_path.exists() && !matches!(conflict_policy, ConflictPolicy::Overwrite) {
        return ApplyFileResult {
            operation_id: operation_id.to_string(),
            source_path: source_path.to_string_lossy().to_string(),
            target_path: Some(target_path.to_string_lossy().to_string()),
            status: ApplyFileStatus::Skipped,
            message: Some("Target exists; overwrite was not selected".to_string()),
        };
    }

    if let Some(parent) = target_path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            return failed(operation_id, source_path, target_path, &error.to_string());
        }
    }

    if target_path.exists() {
        if let Err(error) = fs::remove_file(target_path) {
            return failed(operation_id, source_path, target_path, &error.to_string());
        }
    }

    match fs::rename(source_path, target_path) {
        Ok(()) => ApplyFileResult {
            operation_id: operation_id.to_string(),
            source_path: source_path.to_string_lossy().to_string(),
            target_path: Some(target_path.to_string_lossy().to_string()),
            status: ApplyFileStatus::Success,
            message: None,
        },
        Err(error) => failed(operation_id, source_path, target_path, &error.to_string()),
    }
}

fn failed(
    operation_id: &str,
    source_path: &Path,
    target_path: &Path,
    message: &str,
) -> ApplyFileResult {
    ApplyFileResult {
        operation_id: operation_id.to_string(),
        source_path: source_path.to_string_lossy().to_string(),
        target_path: Some(target_path.to_string_lossy().to_string()),
        status: ApplyFileStatus::Failed,
        message: Some(message.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::domain::operations::{
        ConflictPolicy, OperationKind, OperationPlan, OperationPlanStatus, PlannedOperation,
    };
    use crate::organizer::apply::{apply_organizer_plan, ApplyFileStatus};

    #[test]
    fn apply_plan_moves_file_and_creates_parent_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let source = temp_dir.path().join("source.txt");
        let target = temp_dir.path().join("Documents").join("source.txt");
        fs::write(&source, b"hello").unwrap();

        let response = apply_organizer_plan(&plan(
            source.to_string_lossy(),
            target.to_string_lossy(),
            ConflictPolicy::Rename,
        ));

        assert_eq!(response.results[0].status, ApplyFileStatus::Success);
        assert!(!source.exists());
        assert_eq!(fs::read(&target).unwrap(), b"hello");
    }

    #[test]
    fn apply_plan_skips_existing_target_without_overwrite() {
        let temp_dir = tempfile::tempdir().unwrap();
        let source = temp_dir.path().join("source.txt");
        let target = temp_dir.path().join("target.txt");
        fs::write(&source, b"source").unwrap();
        fs::write(&target, b"target").unwrap();

        let response = apply_organizer_plan(&plan(
            source.to_string_lossy(),
            target.to_string_lossy(),
            ConflictPolicy::Rename,
        ));

        assert_eq!(response.results[0].status, ApplyFileStatus::Skipped);
        assert_eq!(fs::read(&source).unwrap(), b"source");
        assert_eq!(fs::read(&target).unwrap(), b"target");
    }

    fn plan(
        source_path: std::borrow::Cow<'_, str>,
        target_path: std::borrow::Cow<'_, str>,
        conflict_policy: ConflictPolicy,
    ) -> OperationPlan {
        OperationPlan {
            id: "plan-1".to_string(),
            job_id: None,
            status: OperationPlanStatus::Preview,
            operations: vec![PlannedOperation {
                id: "operation-1".to_string(),
                kind: OperationKind::Move,
                source_path: source_path.to_string(),
                target_path: target_path.to_string(),
                conflict_policy,
            }],
        }
    }
}
