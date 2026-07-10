pub mod commands;
pub mod converter;
pub mod db;
pub mod domain;
pub mod errors;
pub mod organizer;

use db::AppDatabase;
use tauri::Manager;

pub struct AppState {
    pub database: AppDatabase,
}

impl AppState {
    pub fn new(database: AppDatabase) -> Self {
        Self { database }
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let database = AppDatabase::open_app_data(app.handle())?;
            app.manage(AppState::new(database));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::get_app_status,
            commands::converter::convert_image_files,
            commands::converter::convert_spreadsheet_files,
            commands::converter::plan_conversion_outputs_command,
            commands::organizer::apply_organizer_plan_command,
            commands::organizer::find_duplicate_files,
            commands::organizer::list_organizer_rules,
            commands::organizer::preview_organizer_plan_command,
            commands::organizer::save_organizer_rules,
            commands::organizer::start_organizer_scan,
            commands::organizer::undo_organizer_plan_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running Filnizer");
}
