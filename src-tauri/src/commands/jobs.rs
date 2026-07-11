use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::{CommandResponse, CommandResult};
use crate::db::jobs_repository::JobsRepository;
use crate::db::operation_repository::OperationRepository;
use crate::domain::jobs::{JobFileResult, JobSummary};
use crate::errors::AppError;
use crate::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListJobsRequest {
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetJobDetailsRequest {
    pub job_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobDetailsResponse {
    pub job: JobSummary,
    pub file_results: Vec<JobFileResult>,
}

#[tauri::command]
pub async fn list_jobs(
    request: Option<ListJobsRequest>,
    state: State<'_, AppState>,
) -> CommandResult<Vec<JobSummary>> {
    let limit = request.and_then(|value| value.limit).unwrap_or(50);
    let repository = JobsRepository::new(state.database.clone());
    Ok(CommandResponse::new(repository.list_jobs(limit)?))
}

#[tauri::command]
pub async fn get_job_details(
    request: GetJobDetailsRequest,
    state: State<'_, AppState>,
) -> CommandResult<JobDetailsResponse> {
    let jobs = JobsRepository::new(state.database.clone());
    let job = jobs
        .get_job(&request.job_id)?
        .ok_or_else(|| AppError::validation("Job history entry was not found"))?;
    let file_results =
        OperationRepository::new(state.database.clone()).list_file_results(&request.job_id)?;

    Ok(CommandResponse::new(JobDetailsResponse {
        job,
        file_results,
    }))
}
