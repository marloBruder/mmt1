use sqlx::SqliteConnection;
use tauri::{async_runtime::Mutex, App, Manager};

mod database;

pub struct AppState {
    db_con: Option<SqliteConnection>,
}

fn app_setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(Mutex::new(AppState { db_con: None }));
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            database::create_database,
            database::create_or_override_database
        ])
        .setup(|app| app_setup(app))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
