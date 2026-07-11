use rusqlite::params;

use crate::db::AppDatabase;
use crate::domain::settings::AppSettings;
use crate::errors::AppError;

const APP_SETTINGS_KEY: &str = "app_settings";

pub struct SettingsRepository {
    database: AppDatabase,
}

impl SettingsRepository {
    pub fn new(database: AppDatabase) -> Self {
        Self { database }
    }

    pub fn get_app_settings(&self) -> Result<AppSettings, AppError> {
        self.database.with_connection(|connection| {
            let mut statement = connection.prepare("SELECT value FROM settings WHERE key = ?1")?;
            let result = statement.query_row([APP_SETTINGS_KEY], |row| row.get::<_, String>(0));

            match result {
                Ok(value) => serde_json::from_str(&value)
                    .map_err(|error| AppError::Unexpected(error.to_string())),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AppSettings::default()),
                Err(error) => Err(error.into()),
            }
        })
    }

    pub fn save_app_settings(&self, settings: &AppSettings) -> Result<(), AppError> {
        let value = serde_json::to_string(settings)
            .map_err(|error| AppError::Unexpected(error.to_string()))?;
        let updated_at = current_unix_ms();

        self.database.with_connection(|connection| {
            connection.execute(
                "INSERT INTO settings (key, value, updated_at_unix_ms)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET
                   value = excluded.value,
                   updated_at_unix_ms = excluded.updated_at_unix_ms",
                params![APP_SETTINGS_KEY, value, updated_at],
            )?;
            Ok(())
        })
    }
}

fn current_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::SettingsRepository;
    use crate::db::AppDatabase;
    use crate::domain::operations::ConflictPolicy;
    use crate::domain::settings::AppSettings;

    #[test]
    fn settings_default_then_round_trip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let database = AppDatabase::open(temp_dir.path().join("test.sqlite3")).unwrap();
        let repository = SettingsRepository::new(database);

        assert_eq!(
            repository.get_app_settings().unwrap(),
            AppSettings::default()
        );

        let settings = AppSettings {
            default_output_directory: Some("C:/Users/Test/Output".to_string()),
            default_conflict_policy: ConflictPolicy::Skip,
            history_retention_days: Some(30),
            show_privacy_note: false,
            allow_network_installs: true,
            libreoffice_install_prompted: true,
        };
        repository.save_app_settings(&settings).unwrap();

        assert_eq!(repository.get_app_settings().unwrap(), settings);
    }
}
