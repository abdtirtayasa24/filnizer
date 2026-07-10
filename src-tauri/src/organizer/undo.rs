use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::operations::{OperationKind, OperationPlan, OperationPlanStatus};
use crate::organizer::apply::{ApplyFileResult, ApplyFileStatus};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoOrganizerPlanRequest {
    pub plan: OperationPlan,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoOrganizerPlanResponse {
    pub job_id: String,
    pub plan_id: String,
    pub results: Vec<ApplyFileResult>,
}

pub fn undo_organizer_plan(plan: &OperationPlan) -> UndoOrganizerPlanResponse {
    let job_id = Uuid::now_v7().to_string();
    let mut results = Vec::new();

    for operation in plan.operations.iter().rev() {
        if !matches!(operation.kind, OperationKind::Move | OperationKind::Rename) {
            results.push(ApplyFileResult {
                operation_id: operation.id.clone(),
                source_path: operation.target_path.clone(),
                target_path: Some(operation.source_path.clone()),
                status: ApplyFileStatus::Failed,
                message: Some("Unsupported operation for organizer undo".to_string()),
            });
            continue;
        }

        results.push(undo_single_operation(
            &operation.id,
            Path::new(&operation.target_path),
            Path::new(&operation.source_path),
        ));
    }

    UndoOrganizerPlanResponse {
        job_id,
        plan_id: plan.id.clone(),
        results,
    }
}

pub fn undone_plan(original: &OperationPlan, job_id: String) -> OperationPlan {
    let mut plan = original.clone();
    plan.job_id = Some(job_id);
    plan.status = OperationPlanStatus::Undone;
    plan
}

fn undo_single_operation(
    operation_id: &str,
    current_path: &Path,
    original_path: &Path,
) -> ApplyFileResult {
    if !current_path.exists() {
        return unsafe_refusal(
            operation_id,
            current_path,
            original_path,
            "Undo refused because the organized file is missing",
        );
    }

    if original_path.exists() {
        return unsafe_refusal(
            operation_id,
            current_path,
            original_path,
            "Undo refused because the original path already exists",
        );
    }

    if let Some(parent) = original_path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            return failed(
                operation_id,
                current_path,
                original_path,
                &error.to_string(),
            );
        }
    }

    match fs::rename(current_path, original_path) {
        Ok(()) => ApplyFileResult {
            operation_id: operation_id.to_string(),
            source_path: current_path.to_string_lossy().to_string(),
            target_path: Some(original_path.to_string_lossy().to_string()),
            status: ApplyFileStatus::Success,
            message: None,
        },
        Err(error) => failed(
            operation_id,
            current_path,
            original_path,
            &error.to_string(),
        ),
    }
}

fn unsafe_refusal(
    operation_id: &str,
    current_path: &Path,
    original_path: &Path,
    message: &str,
) -> ApplyFileResult {
    ApplyFileResult {
        operation_id: operation_id.to_string(),
        source_path: current_path.to_string_lossy().to_string(),
        target_path: Some(original_path.to_string_lossy().to_string()),
        status: ApplyFileStatus::Skipped,
        message: Some(message.to_string()),
    }
}

fn failed(
    operation_id: &str,
    current_path: &Path,
    original_path: &Path,
    message: &str,
) -> ApplyFileResult {
    ApplyFileResult {
        operation_id: operation_id.to_string(),
        source_path: current_path.to_string_lossy().to_string(),
        target_path: Some(original_path.to_string_lossy().to_string()),
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
    use crate::organizer::apply::ApplyFileStatus;
    use crate::organizer::undo::undo_organizer_plan;

    #[test]
    fn undo_plan_moves_file_back_to_original_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original = temp_dir.path().join("original.txt");
        let organized = temp_dir.path().join("Documents").join("original.txt");
        fs::create_dir_all(organized.parent().unwrap()).unwrap();
        fs::write(&organized, b"hello").unwrap();

        let response = undo_organizer_plan(&plan(
            original.to_string_lossy(),
            organized.to_string_lossy(),
        ));

        assert_eq!(response.results[0].status, ApplyFileStatus::Success);
        assert_eq!(fs::read(&original).unwrap(), b"hello");
        assert!(!organized.exists());
    }

    #[test]
    fn undo_plan_refuses_to_overwrite_original_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original = temp_dir.path().join("original.txt");
        let organized = temp_dir.path().join("Documents").join("original.txt");
        fs::create_dir_all(organized.parent().unwrap()).unwrap();
        fs::write(&original, b"new user data").unwrap();
        fs::write(&organized, b"organized").unwrap();

        let response = undo_organizer_plan(&plan(
            original.to_string_lossy(),
            organized.to_string_lossy(),
        ));

        assert_eq!(response.results[0].status, ApplyFileStatus::Skipped);
        assert_eq!(fs::read(&original).unwrap(), b"new user data");
        assert_eq!(fs::read(&organized).unwrap(), b"organized");
    }

    fn plan(
        source_path: std::borrow::Cow<'_, str>,
        target_path: std::borrow::Cow<'_, str>,
    ) -> OperationPlan {
        OperationPlan {
            id: "plan-1".to_string(),
            job_id: None,
            status: OperationPlanStatus::Applied,
            operations: vec![PlannedOperation {
                id: "operation-1".to_string(),
                kind: OperationKind::Move,
                source_path: source_path.to_string(),
                target_path: target_path.to_string(),
                conflict_policy: ConflictPolicy::Rename,
            }],
        }
    }
}
