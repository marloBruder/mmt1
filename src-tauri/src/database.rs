use sqlx::{migrate::MigrateDatabase, Connection, Sqlite, SqliteConnection};
use std::fmt;
use tauri::async_runtime::Mutex;

use crate::AppState;

pub mod in_progress_theorem;

// Tauri Command for creating a new database
// If the database already exists, it will instead return an DatabaseExistsError
#[tauri::command]
pub async fn create_database(
    file_path: &str,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), Error> {
    //println!("Trying to create db with path {}!", file_path);
    if Sqlite::database_exists(file_path).await.unwrap_or(false) {
        return Err(Error::DatabaseExistsError);
    }
    create_or_override_database(file_path, state).await
}

// Tauri Command for creating a new database or overriding it if it already exists
#[tauri::command]
pub async fn create_or_override_database(
    file_path: &str,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), Error> {
    Sqlite::create_database(file_path)
        .await
        .or(Err(Error::CreateDatabaseError))?;

    let mut conn = SqliteConnection::connect(file_path)
        .await
        .or(Err(Error::ConnectDatabaseError))?;

    sqlx::query(sql::INIT_DB)
        .execute(&mut conn)
        .await
        .or(Err(Error::SqlError))?;
    // .map_err(|e| {
    //     print!("{:?}", e);
    //     Error::SqlError
    // })?;

    let mut app_state = state.lock().await;
    app_state.db_conn = Some(conn);

    Ok(())
}

#[tauri::command]
pub async fn open_database(
    file_path: &str,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<MetamathData, Error> {
    let mut conn = SqliteConnection::connect(file_path)
        .await
        .or(Err(Error::ConnectDatabaseError))?;

    sql::check_returns_rows_or_error(sql::IN_PROGRESS_THEOREM_TABLE_CHECK, &mut conn).await?;
    sql::check_returns_rows_or_error(sql::THEOREM_TABLE_CHECK, &mut conn).await?;

    let mut app_state = state.lock().await;
    app_state.db_conn = Some(conn);
    drop(app_state); // Unlock Mutex

    Ok(MetamathData {
        in_progress_theorems: in_progress_theorem::get_in_progress_theorems(state).await?,
    })
}

pub struct MetamathData {
    in_progress_theorems: Vec<in_progress_theorem::InProgressTheorem>,
}

impl serde::Serialize for MetamathData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("MetamathData", 3)?;
        state.serialize_field("in_progress_theorems", &self.in_progress_theorems)?;
        state.end()
    }
}

#[derive(Debug)]
pub enum Error {
    DatabaseExistsError,
    CreateDatabaseError,
    ConnectDatabaseError,
    WrongDatabaseFormatError,
    SqlError,
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

mod sql {
    use sqlx::SqliteConnection;

    pub const INIT_DB: &str = "\
CREATE TABLE inProgressTheorem (
    name TEXT PRIMARY KEY,
    text TEXT
);
CREATE TABLE theorem (
    name TEXT PRIMARY KEY,
    description TEXT,
    disjoints TEXT,
    hypotheses TEXT,
    assertion TEXT,
    proof TEXT
);
";

    pub const IN_PROGRESS_THEOREM_TABLE_CHECK: &str =
        "SELECT name FROM sqlite_master WHERE type='table' AND name='inProgressTheorem';";
    pub const THEOREM_TABLE_CHECK: &str =
        "SELECT name FROM sqlite_master WHERE type='table' AND name='theorem';";

    // Checks whether the query returns rows and returns the correct error if not
    pub async fn check_returns_rows_or_error(
        query: &str,
        conn: &mut SqliteConnection,
    ) -> Result<(), super::Error> {
        let rows = sqlx::query(query)
            .fetch_all(conn)
            .await
            .or(Err(super::Error::SqlError))?;

        if rows.len() == 0 {
            return Err(super::Error::WrongDatabaseFormatError);
        }
        Ok(())
    }
}
