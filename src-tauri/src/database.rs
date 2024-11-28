use diesel::prelude::*;

#[tauri::command]
pub fn create_database(file_path: &str) {
    //println!("Trying to create db with path {}!", file_path);
    SqliteConnection::establish(&file_path)
        .unwrap_or_else(|_| panic!("Error connecting to {}", file_path));
}