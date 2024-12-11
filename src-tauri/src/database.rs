use sqlx::{migrate::MigrateDatabase, Sqlite};
use std::fmt;

// Tauri Command for creating a new database
// If the database already exists, it will instead return an DatabaseExistsError
#[tauri::command]
pub async fn create_database(file_path: &str) -> Result<(), Error> {
    //println!("Trying to create db with path {}!", file_path);
    if Sqlite::database_exists(file_path).await.unwrap_or(false) {
        return Err(Error::DatabaseExistsError);
    }
    create_or_override_database_pure(file_path).await
}

// Tauri Command for creating a new database or overriding it if it already exists
#[tauri::command]
pub async fn create_or_override_database(file_path: &str) -> Result<(), Error> {
    create_or_override_database_pure(file_path).await
}

// The function that actually creates a new database or overrides it if it already exists
// It had to be it's own funtion, so that it can be called by multiple Tauri commands
async fn create_or_override_database_pure(_file_path: &str) -> Result<(), Error> {
    Sqlite::create_database(_file_path)
        .await
        .or(Err(Error::CreateDatabaseError))
}

#[derive(Debug)]
pub enum Error {
    DatabaseExistsError,
    CreateDatabaseError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
