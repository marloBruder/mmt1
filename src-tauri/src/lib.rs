use model::MetamathData;
use sqlx::SqliteConnection;
use tauri::{async_runtime::Mutex, App, Manager};

mod database;
mod local_state;
mod metamath;
mod model;

pub struct AppState {
    db_conn: Option<SqliteConnection>,
    metamath_data: Option<MetamathData>,
}

fn app_setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(Mutex::new(AppState {
        db_conn: None,
        metamath_data: None,
    }));
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            database::create_database,
            database::create_or_override_database,
            database::open_database,
            database::in_progress_theorem::add_in_progress_theorem,
            database::in_progress_theorem::set_in_progress_theorem_name,
            database::in_progress_theorem::set_in_progress_theorem,
            database::in_progress_theorem::delete_in_progress_theorem,
            metamath::turn_into_theorem,
            metamath::text_to_constants,
            metamath::text_to_variables,
            metamath::text_to_floating_hypotheses,
            local_state::get_constants_local,
            local_state::get_variables_local,
            local_state::get_floating_hypotheses_local,
            local_state::get_theorem_page_data_local,
            local_state::get_theorem_names_local,
            local_state::get_in_progress_theorem_local,
            local_state::get_in_progress_theorem_names_local,
        ])
        .setup(|app| app_setup(app))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
