use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::errors::AppError;

pub mod jobs_repository;
pub mod settings_repository;

#[derive(Clone)]
pub struct AppDatabase {
    connection: Arc<Mutex<Connection>>,
}

impl AppDatabase {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let connection = Connection::open(path)?;
        let database = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        database.run_migrations()?;
        Ok(database)
    }

    pub fn open_app_data(app: &AppHandle) -> Result<Self, AppError> {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|error| AppError::Filesystem(error.to_string()))?;
        fs::create_dir_all(&app_data_dir)?;
        Self::open(app_data_dir.join("filnizer.sqlite3"))
    }

    pub fn with_connection<T>(
        &self,
        action: impl FnOnce(&Connection) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AppError::Database("database connection lock was poisoned".to_string()))?;
        action(&connection)
    }

    fn run_migrations(&self) -> Result<(), AppError> {
        self.with_connection(|connection| {
            connection.execute_batch(include_str!("../../migrations/001_initial.sql"))?;
            connection.execute_batch(include_str!("../../migrations/002_organizer_rules.sql"))?;
            Ok(())
        })
    }
}
