use rusqlite::params;

use crate::db::AppDatabase;
use crate::domain::jobs::{JobStatus, JobSummary};
use crate::errors::AppError;

pub struct JobsRepository {
    database: AppDatabase,
}

impl JobsRepository {
    pub fn new(database: AppDatabase) -> Self {
        Self { database }
    }

    pub fn insert_job(&self, job: &JobSummary) -> Result<(), AppError> {
        let kind = serde_json::to_string(&job.kind)
            .map_err(|error| AppError::Unexpected(error.to_string()))?;
        let status = serde_json::to_string(&job.status)
            .map_err(|error| AppError::Unexpected(error.to_string()))?;

        self.database.with_connection(|connection| {
            connection.execute(
                "INSERT INTO jobs (
                   id, kind, status, name, total_files, completed_files,
                   created_at_unix_ms, updated_at_unix_ms, error_message
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    job.id,
                    kind,
                    status,
                    job.name,
                    job.total_files as i64,
                    job.completed_files as i64,
                    job.created_at_unix_ms,
                    job.updated_at_unix_ms,
                    job.error_message,
                ],
            )?;
            Ok(())
        })
    }

    pub fn update_job_progress(
        &self,
        id: &str,
        status: JobStatus,
        total_files: u64,
        completed_files: u64,
        error_message: Option<String>,
    ) -> Result<(), AppError> {
        let status = serde_json::to_string(&status)
            .map_err(|error| AppError::Unexpected(error.to_string()))?;
        let updated_at = current_unix_ms();

        self.database.with_connection(|connection| {
            connection.execute(
                "UPDATE jobs
                 SET status = ?2,
                     total_files = ?3,
                     completed_files = ?4,
                     updated_at_unix_ms = ?5,
                     error_message = ?6
                 WHERE id = ?1",
                params![
                    id,
                    status,
                    total_files as i64,
                    completed_files as i64,
                    updated_at,
                    error_message,
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_job(&self, id: &str) -> Result<Option<JobSummary>, AppError> {
        self.database.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT id, kind, status, name, total_files, completed_files,
                        created_at_unix_ms, updated_at_unix_ms, error_message
                 FROM jobs
                 WHERE id = ?1",
            )?;

            let result = statement.query_row([id], read_job_summary_row);

            match result {
                Ok(job) => Ok(Some(job)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(error) => Err(error.into()),
            }
        })
    }

    pub fn list_jobs(&self, limit: u64) -> Result<Vec<JobSummary>, AppError> {
        let limit = limit.clamp(1, 200) as i64;
        self.database.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT id, kind, status, name, total_files, completed_files,
                        created_at_unix_ms, updated_at_unix_ms, error_message
                 FROM jobs
                 ORDER BY updated_at_unix_ms DESC, created_at_unix_ms DESC
                 LIMIT ?1",
            )?;
            let rows = statement.query_map([limit], read_job_summary_row)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        })
    }
}

fn read_job_summary_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<JobSummary> {
    let kind: String = row.get(1)?;
    let status: String = row.get(2)?;

    Ok(JobSummary {
        id: row.get(0)?,
        kind: serde_json::from_str(&kind).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        status: serde_json::from_str(&status).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        name: row.get(3)?,
        total_files: row.get::<_, i64>(4)? as u64,
        completed_files: row.get::<_, i64>(5)? as u64,
        created_at_unix_ms: row.get(6)?,
        updated_at_unix_ms: row.get(7)?,
        error_message: row.get(8)?,
    })
}

fn current_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::JobsRepository;
    use crate::db::AppDatabase;
    use crate::domain::jobs::{JobKind, JobStatus, JobSummary};

    #[test]
    fn job_round_trip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let database = AppDatabase::open(temp_dir.path().join("test.sqlite3")).unwrap();
        let repository = JobsRepository::new(database);
        let job = JobSummary {
            id: "job-1".to_string(),
            kind: JobKind::OrganizerScan,
            status: JobStatus::Queued,
            name: "Scan Downloads".to_string(),
            total_files: 10,
            completed_files: 0,
            created_at_unix_ms: 1000,
            updated_at_unix_ms: 1000,
            error_message: None,
        };

        repository.insert_job(&job).unwrap();

        assert_eq!(repository.get_job("job-1").unwrap(), Some(job));
        assert_eq!(repository.get_job("missing").unwrap(), None);
    }

    #[test]
    fn list_jobs_returns_newest_first_with_limit() {
        let temp_dir = tempfile::tempdir().unwrap();
        let database = AppDatabase::open(temp_dir.path().join("test.sqlite3")).unwrap();
        let repository = JobsRepository::new(database);

        for index in 1..=3 {
            repository
                .insert_job(&JobSummary {
                    id: format!("job-{index}"),
                    kind: JobKind::Conversion,
                    status: JobStatus::Completed,
                    name: format!("Job {index}"),
                    total_files: index,
                    completed_files: index,
                    created_at_unix_ms: index as i64,
                    updated_at_unix_ms: index as i64,
                    error_message: None,
                })
                .unwrap();
        }

        let jobs = repository.list_jobs(2).unwrap();

        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].id, "job-3");
        assert_eq!(jobs[1].id, "job-2");
    }
}
