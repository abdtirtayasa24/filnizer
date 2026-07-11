use serde::Deserialize;
use tauri::State;

use crate::commands::{CommandResponse, CommandResult};
use crate::db::settings_repository::SettingsRepository;
use crate::domain::settings::AppSettings;
use crate::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveAppSettingsRequest {
    pub settings: AppSettings,
}

#[tauri::command]
pub async fn get_app_settings(state: State<'_, AppState>) -> CommandResult<AppSettings> {
    let repository = SettingsRepository::new(state.database.clone());
    Ok(CommandResponse::new(repository.get_app_settings()?))
}

#[tauri::command]
pub async fn save_app_settings(
    request: SaveAppSettingsRequest,
    state: State<'_, AppState>,
) -> CommandResult<AppSettings> {
    let repository = SettingsRepository::new(state.database.clone());
    repository.save_app_settings(&request.settings)?;
    Ok(CommandResponse::new(repository.get_app_settings()?))
}
