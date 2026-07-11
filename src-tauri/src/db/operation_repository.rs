use rusqlite::params;

use crate::db::AppDatabase;
use crate::domain::conversion::ConversionFileResult;
use crate::domain::jobs::JobFileResult;
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

    pub fn save_conversion_results(
        &self,
        job_id: &str,
        results: &[ConversionFileResult],
    ) -> Result<(), AppError> {
        self.database.with_connection(|connection| {
            for result in results {
                connection.execute(
                    "INSERT INTO file_results
                     (job_id, source_path, target_path, status, message, updated_at_unix_ms)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        job_id,
                        result.input_path,
                        result.output_path,
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

    pub fn list_file_results(&self, job_id: &str) -> Result<Vec<JobFileResult>, AppError> {
        self.database.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT id, job_id, source_path, target_path, status, message, updated_at_unix_ms
                 FROM file_results
                 WHERE job_id = ?1
                 ORDER BY id ASC",
            )?;
            let rows = statement.query_map([job_id], |row| {
                let status: String = row.get(4)?;
                Ok(JobFileResult {
                    id: row.get(0)?,
                    job_id: row.get(1)?,
                    source_path: row.get(2)?,
                    target_path: row.get(3)?,
                    status: deserialize_status_label(&status),
                    message: row.get(5)?,
                    updated_at_unix_ms: row.get(6)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        })
    }
}

fn deserialize_status_label(raw_status: &str) -> String {
    serde_json::from_str::<String>(raw_status).unwrap_or_else(|_| raw_status.to_string())
}

fn current_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use crate::db::jobs_repository::JobsRepository;
    use crate::db::operation_repository::OperationRepository;
    use crate::db::AppDatabase;
    use crate::domain::conversion::{ConversionFileResult, ConversionFileStatus};
    use crate::domain::jobs::{JobKind, JobStatus, JobSummary};

    #[test]
    fn list_file_results_returns_persisted_status_labels() {
        let temp_dir = tempfile::tempdir().unwrap();
        let database = AppDatabase::open(temp_dir.path().join("test.sqlite3")).unwrap();
        JobsRepository::new(database.clone())
            .insert_job(&JobSummary {
                id: "job-1".to_string(),
                kind: JobKind::Conversion,
                status: JobStatus::Completed,
                name: "Conversion".to_string(),
                total_files: 2,
                completed_files: 1,
                created_at_unix_ms: 1,
                updated_at_unix_ms: 1,
                error_message: None,
            })
            .unwrap();
        let repository = OperationRepository::new(database);

        repository
            .save_conversion_results(
                "job-1",
                &[
                    ConversionFileResult {
                        input_path: "input-a.png".to_string(),
                        output_path: Some("output-a.jpg".to_string()),
                        status: ConversionFileStatus::Completed,
                        message: None,
                    },
                    ConversionFileResult {
                        input_path: "input-b.png".to_string(),
                        output_path: None,
                        status: ConversionFileStatus::Failed,
                        message: Some("could not convert".to_string()),
                    },
                ],
            )
            .unwrap();

        let results = repository.list_file_results("job-1").unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].source_path, "input-a.png");
        assert_eq!(results[0].target_path.as_deref(), Some("output-a.jpg"));
        assert_eq!(results[0].status, "completed");
        assert_eq!(results[1].status, "failed");
        assert_eq!(results[1].message.as_deref(), Some("could not convert"));
    }
}
