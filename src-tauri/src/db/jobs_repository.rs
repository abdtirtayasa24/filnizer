use rusqlite::params;

use crate::db::AppDatabase;
use crate::domain::jobs::JobSummary;
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

    pub fn get_job(&self, id: &str) -> Result<Option<JobSummary>, AppError> {
        self.database.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT id, kind, status, name, total_files, completed_files,
                        created_at_unix_ms, updated_at_unix_ms, error_message
                 FROM jobs
                 WHERE id = ?1",
            )?;

            let result = statement.query_row([id], |row| {
                let kind: String = row.get(1)?;
                let status: String = row.get(2)?;

                Ok((
                    row.get::<_, String>(0)?,
                    kind,
                    status,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, Option<String>>(8)?,
                ))
            });

            match result {
                Ok((
                    id,
                    kind,
                    status,
                    name,
                    total_files,
                    completed_files,
                    created_at_unix_ms,
                    updated_at_unix_ms,
                    error_message,
                )) => Ok(Some(JobSummary {
                    id,
                    kind: serde_json::from_str(&kind)
                        .map_err(|error| AppError::Unexpected(error.to_string()))?,
                    status: serde_json::from_str(&status)
                        .map_err(|error| AppError::Unexpected(error.to_string()))?,
                    name,
                    total_files: total_files as u64,
                    completed_files: completed_files as u64,
                    created_at_unix_ms,
                    updated_at_unix_ms,
                    error_message,
                })),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(error) => Err(error.into()),
            }
        })
    }
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
}
