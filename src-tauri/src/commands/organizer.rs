use serde::Serialize;
use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::commands::{CommandResponse, CommandResult};
use crate::db::jobs_repository::JobsRepository;
use crate::db::operation_repository::OperationRepository;
use crate::domain::jobs::{JobKind, JobStatus, JobSummary};
use crate::domain::operations::OperationPlan;
use crate::organizer::apply::{
    applied_plan, apply_organizer_plan, ApplyFileStatus, ApplyOrganizerPlanRequest,
    ApplyOrganizerPlanResponse,
};
use crate::organizer::planner::{preview_organizer_plan, PreviewOrganizerPlanRequest};
use crate::organizer::rules::{OrganizerRule, OrganizerRulesRepository, SaveOrganizerRulesRequest};
use crate::organizer::scan::{scan_folders, ScanRequest};
use crate::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartScanResponse {
    pub job_id: String,
    pub files: Vec<crate::domain::files::FileEntry>,
}

#[tauri::command]
pub async fn start_organizer_scan(
    request: ScanRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<StartScanResponse> {
    let job_id = Uuid::now_v7().to_string();
    let now = current_unix_ms();
    let repository = JobsRepository::new(state.database.clone());

    repository.insert_job(&JobSummary {
        id: job_id.clone(),
        kind: JobKind::OrganizerScan,
        status: JobStatus::Running,
        name: "Organizer scan".to_string(),
        total_files: 0,
        completed_files: 0,
        created_at_unix_ms: now,
        updated_at_unix_ms: now,
        error_message: None,
    })?;

    let rules = OrganizerRulesRepository::new(state.database.clone()).list_rules()?;
    let scan_request = request.clone();
    let scan_job_id = job_id.clone();
    let scan_app = app.clone();
    let scan_result = tokio::task::spawn_blocking(move || {
        scan_folders(&scan_request, &scan_job_id, Some(&scan_app), &rules)
    })
    .await
    .map_err(|error| crate::errors::AppError::Unexpected(error.to_string()))?;

    match scan_result {
        Ok(result) => {
            repository.update_job_progress(
                &job_id,
                JobStatus::Completed,
                result.files.len() as u64,
                result.files.len() as u64,
                None,
            )?;
            Ok(CommandResponse::new(StartScanResponse {
                job_id,
                files: result.files,
            }))
        }
        Err(error) => {
            repository.update_job_progress(
                &job_id,
                JobStatus::Failed,
                0,
                0,
                Some(error.to_string()),
            )?;
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn apply_organizer_plan_command(
    request: ApplyOrganizerPlanRequest,
    state: State<'_, AppState>,
) -> CommandResult<ApplyOrganizerPlanResponse> {
    let now = current_unix_ms();
    let jobs = JobsRepository::new(state.database.clone());
    let operations = OperationRepository::new(state.database.clone());
    let response = apply_organizer_plan(&request.plan);
    let total_files = response.results.len() as u64;
    let successful_files = response
        .results
        .iter()
        .filter(|result| result.status == ApplyFileStatus::Success)
        .count() as u64;
    let failed_or_skipped = response.results.iter().any(|result| {
        matches!(result.status, ApplyFileStatus::Failed | ApplyFileStatus::Skipped)
    });
    let status = if successful_files == total_files {
        JobStatus::Completed
    } else if successful_files > 0 || failed_or_skipped {
        JobStatus::PartiallyCompleted
    } else {
        JobStatus::Failed
    };

    jobs.insert_job(&JobSummary {
        id: response.job_id.clone(),
        kind: JobKind::OrganizerApply,
        status,
        name: "Apply organizer plan".to_string(),
        total_files,
        completed_files: successful_files,
        created_at_unix_ms: now,
        updated_at_unix_ms: now,
        error_message: None,
    })?;
    operations.save_plan(&applied_plan(&request.plan, response.job_id.clone()))?;
    operations.save_file_results(&response.job_id, &response.results)?;

    Ok(CommandResponse::new(response))
}

#[tauri::command]
pub async fn preview_organizer_plan_command(
    request: PreviewOrganizerPlanRequest,
) -> CommandResult<OperationPlan> {
    Ok(CommandResponse::new(preview_organizer_plan(&request)?))
}

#[tauri::command]
pub async fn list_organizer_rules(
    state: State<'_, AppState>,
) -> CommandResult<Vec<OrganizerRule>> {
    let repository = OrganizerRulesRepository::new(state.database.clone());
    Ok(CommandResponse::new(repository.list_rules()?))
}

#[tauri::command]
pub async fn save_organizer_rules(
    request: SaveOrganizerRulesRequest,
    state: State<'_, AppState>,
) -> CommandResult<Vec<OrganizerRule>> {
    let repository = OrganizerRulesRepository::new(state.database.clone());
    Ok(CommandResponse::new(repository.replace_rules(&request.rules)?))
}

fn current_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}
