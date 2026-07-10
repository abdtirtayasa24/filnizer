use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::commands::{CommandResponse, CommandResult};
use crate::converter::planner::{conversion_job_status, plan_conversion_outputs};
use crate::db::jobs_repository::JobsRepository;
use crate::db::operation_repository::OperationRepository;
use crate::domain::conversion::{ConversionFileResult, ConversionRequest};
use crate::domain::jobs::{JobKind, JobSummary};
use crate::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanConversionOutputsResponse {
    pub job_id: String,
    pub results: Vec<ConversionFileResult>,
}

#[tauri::command]
pub async fn plan_conversion_outputs_command(
    request: ConversionRequest,
    state: State<'_, AppState>,
) -> CommandResult<PlanConversionOutputsResponse> {
    let job_id = Uuid::now_v7().to_string();
    let now = current_unix_ms();
    let results = plan_conversion_outputs(&request)?;
    let status = conversion_job_status(&results);
    let total_files = results.len() as u64;
    let completed_files = results
        .iter()
        .filter(|result| {
            matches!(
                result.status,
                crate::domain::conversion::ConversionFileStatus::Completed
            )
        })
        .count() as u64;

    JobsRepository::new(state.database.clone()).insert_job(&JobSummary {
        id: job_id.clone(),
        kind: JobKind::Conversion,
        status,
        name: "Plan conversion outputs".to_string(),
        total_files,
        completed_files,
        created_at_unix_ms: now,
        updated_at_unix_ms: now,
        error_message: None,
    })?;
    OperationRepository::new(state.database.clone()).save_conversion_results(&job_id, &results)?;

    Ok(CommandResponse::new(PlanConversionOutputsResponse {
        job_id,
        results,
    }))
}

fn current_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}
