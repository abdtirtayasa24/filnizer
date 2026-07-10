pub mod commands;
pub mod domain;
pub mod errors;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![commands::app::get_app_status])
        .run(tauri::generate_context!())
        .expect("error while running Filnizer");
}
