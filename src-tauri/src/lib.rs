pub mod commands;
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
            commands::organizer::apply_organizer_plan_command,
            commands::organizer::list_organizer_rules,
            commands::organizer::preview_organizer_plan_command,
            commands::organizer::save_organizer_rules,
            commands::organizer::start_organizer_scan
        ])
        .run(tauri::generate_context!())
        .expect("error while running Filnizer");
}
