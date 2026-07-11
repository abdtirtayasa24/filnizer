use rusqlite::params;

use crate::db::AppDatabase;
use crate::domain::conversion::ConversionFileResult;
use crate::domain::files::FileEntry;
use crate::domain::jobs::JobFileResult;
use crate::domain::operations::OperationPlan;
use crate::errors::AppError;
use crate::organizer::apply::ApplyFileResult;
use crate::organizer::duplicates::DuplicateSet;

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

    pub fn save_scan_results(&self, job_id: &str, files: &[FileEntry]) -> Result<(), AppError> {
        self.database.with_connection(|connection| {
            for file in files {
                let message = serde_json::to_string(&file.category)
                    .map_err(|error| AppError::Unexpected(error.to_string()))?;
                connection.execute(
                    "INSERT INTO file_results
                     (job_id, source_path, target_path, status, message, updated_at_unix_ms)
                     VALUES (?1, ?2, NULL, ?3, ?4, ?5)",
                    params![job_id, file.path, "scanned", message, current_unix_ms()],
                )?;
            }
            Ok(())
        })
    }

    pub fn save_duplicate_results(
        &self,
        job_id: &str,
        sets: &[DuplicateSet],
    ) -> Result<(), AppError> {
        self.database.with_connection(|connection| {
            for duplicate_set in sets {
                let message = format!(
                    "Set {} · {} bytes · {} files",
                    duplicate_set.blake3,
                    duplicate_set.size_bytes,
                    duplicate_set.paths.len()
                );
                for path in &duplicate_set.paths {
                    connection.execute(
                        "INSERT INTO file_results
                         (job_id, source_path, target_path, status, message, updated_at_unix_ms)
                         VALUES (?1, ?2, NULL, ?3, ?4, ?5)",
                        params![job_id, path, "duplicate", message, current_unix_ms()],
                    )?;
                }
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
    use crate::domain::files::{FileCategory, FileEntry, HashStatus};
    use crate::domain::jobs::{JobKind, JobStatus, JobSummary};
    use crate::organizer::duplicates::DuplicateSet;

    #[test]
    fn list_file_results_returns_persisted_status_labels() {
        let temp_dir = tempfile::tempdir().unwrap();
        let database = seeded_database(
            temp_dir.path().join("test.sqlite3"),
            "job-1",
            JobKind::Conversion,
        );
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

    #[test]
    fn save_scan_results_persists_each_scanned_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let database = seeded_database(
            temp_dir.path().join("test.sqlite3"),
            "scan-1",
            JobKind::OrganizerScan,
        );
        let repository = OperationRepository::new(database);

        repository
            .save_scan_results(
                "scan-1",
                &[FileEntry {
                    path: "C:/Downloads/report.pdf".to_string(),
                    name: "report.pdf".to_string(),
                    extension: Some("pdf".to_string()),
                    size_bytes: 100,
                    modified_unix_ms: None,
                    category: FileCategory::Pdfs,
                    hash_status: HashStatus::NotRequested,
                }],
            )
            .unwrap();

        let results = repository.list_file_results("scan-1").unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, "scanned");
        assert_eq!(results[0].source_path, "C:/Downloads/report.pdf");
        assert_eq!(results[0].message.as_deref(), Some("\"pdfs\""));
    }

    #[test]
    fn save_duplicate_results_persists_each_duplicate_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let database = seeded_database(
            temp_dir.path().join("test.sqlite3"),
            "dupes-1",
            JobKind::DuplicateAnalysis,
        );
        let repository = OperationRepository::new(database);

        repository
            .save_duplicate_results(
                "dupes-1",
                &[DuplicateSet {
                    size_bytes: 42,
                    blake3: "hash".to_string(),
                    paths: vec!["a.txt".to_string(), "b.txt".to_string()],
                }],
            )
            .unwrap();

        let results = repository.list_file_results("dupes-1").unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].status, "duplicate");
        assert!(results[0].message.as_ref().unwrap().contains("42 bytes"));
    }

    fn seeded_database(
        path: impl AsRef<std::path::Path>,
        job_id: &str,
        kind: JobKind,
    ) -> AppDatabase {
        let database = AppDatabase::open(path).unwrap();
        JobsRepository::new(database.clone())
            .insert_job(&JobSummary {
                id: job_id.to_string(),
                kind,
                status: JobStatus::Completed,
                name: "History job".to_string(),
                total_files: 2,
                completed_files: 1,
                created_at_unix_ms: 1,
                updated_at_unix_ms: 1,
                error_message: None,
            })
            .unwrap();
        database
    }
}
