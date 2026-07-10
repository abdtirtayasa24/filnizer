use rusqlite::params;

use crate::db::AppDatabase;
use crate::domain::operations::OperationPlan;
use crate::errors::AppError;
use crate::organizer::apply::ApplyFileResult;

pub struct OperationRepository {
    database: AppDatabase,
}

impl OperationRepository {
    pub fn new(database: AppDatabase) -> Self {
        Self { database }
    }

    pub fn save_plan(&self, plan: &OperationPlan) -> Result<(), AppError> {
        let plan_json =
            serde_json::to_string(plan).map_err(|error| AppError::Unexpected(error.to_string()))?;
        let status = serde_json::to_string(&plan.status)
            .map_err(|error| AppError::Unexpected(error.to_string()))?;

        self.database.with_connection(|connection| {
            connection.execute(
                "INSERT OR REPLACE INTO operation_plans
                 (id, job_id, status, plan_json, created_at_unix_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![plan.id, plan.job_id, status, plan_json, current_unix_ms()],
            )?;
            Ok(())
        })
    }

    pub fn save_file_results(
        &self,
        job_id: &str,
        results: &[ApplyFileResult],
    ) -> Result<(), AppError> {
        self.database.with_connection(|connection| {
            for result in results {
                connection.execute(
                    "INSERT INTO file_results
                     (job_id, source_path, target_path, status, message, updated_at_unix_ms)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        job_id,
                        result.source_path,
                        result.target_path,
                        serde_json::to_string(&result.status)
                            .map_err(|error| AppError::Unexpected(error.to_string()))?,
                        result.message,
                        current_unix_ms(),
                    ],
                )?;
            }
            Ok(())
        })
    }

    pub fn count_file_results(&self, job_id: &str) -> Result<u64, AppError> {
        self.database.with_connection(|connection| {
            let count = connection.query_row(
                "SELECT COUNT(*) FROM file_results WHERE job_id = ?1",
                [job_id],
                |row| row.get::<_, i64>(0),
            )?;
            Ok(count as u64)
        })
    }
}

fn current_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}
