use diesel::prelude::*;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn create_database(file_path: &str) {
    //println!("Trying to create db with path {}!", file_path);
    SqliteConnection::establish(&file_path)
        .unwrap_or_else(|_| panic!("Error connecting to {}", file_path));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, create_database])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
