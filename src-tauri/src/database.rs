// use futures::TryStreamExt;
// use sqlx::Row;
use sqlx::{migrate::MigrateDatabase, Connection, Sqlite, SqliteConnection};
use std::fmt;
use tauri::async_runtime::Mutex;

use crate::AppState;

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
    create_or_override_database_pure(file_path, state).await
}

// Tauri Command for creating a new database or overriding it if it already exists
#[tauri::command]
pub async fn create_or_override_database(
    file_path: &str,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), Error> {
    create_or_override_database_pure(file_path, state).await
}

// The function that actually creates a new database or overrides it if it already exists
// It had to be it's own funtion, so that it can be called by multiple Tauri commands
async fn create_or_override_database_pure(
    file_path: &str,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), Error> {
    Sqlite::create_database(file_path)
        .await
        .or(Err(Error::CreateDatabaseError))?;

    let mut conn = SqliteConnection::connect(file_path)
        .await
        .or(Err(Error::ConnectDatabaseError))?;

    sqlx::query(sql::IN_PROGRESS_THEOREM_TABLE_CREATE)
        .execute(&mut conn)
        .await
        .or(Err(Error::SqlError))?;

    let mut app_state = state.lock().await;
    app_state.db_conn = Some(conn);

    Ok(())
}

#[tauri::command]
pub async fn open_database(
    file_path: &str,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), Error> {
    let mut conn = SqliteConnection::connect(file_path)
        .await
        .or(Err(Error::ConnectDatabaseError))?;

    let rows = sqlx::query(sql::IN_PROGRESS_THEOREM_TABLE_CHECK)
        .fetch_all(&mut conn)
        .await
        .or(Err(Error::SqlError))?;

    if rows.len() == 0 {
        return Err(Error::WrongDatabaseFormatError);
    }

    let mut app_state = state.lock().await;
    app_state.db_conn = Some(conn);

    Ok(())
}

// #[tauri::command]
// pub async fn insert_test(state: tauri::State<'_, Mutex<AppState>>) -> Result<(), ()> {
//     let mut app_state = state.lock().await;

//     if let Some(ref mut conn) = app_state.db_conn {
//         sqlx::query("INSERT INTO test (testVar) VALUES (3)")
//             .execute(conn)
//             .await
//             .or(Err(()))?;
//     }

//     Ok(())
// }

// #[tauri::command]
// pub async fn select_test(state: tauri::State<'_, Mutex<AppState>>) -> Result<Vec<i32>, ()> {
//     let mut app_state = state.lock().await;

//     let mut test_nums: Vec<i32> = Vec::new();

//     if let Some(ref mut conn) = app_state.db_conn {
//         let mut rows = sqlx::query("SELECT * FROM test").fetch(conn);

//         while let Some(row) = rows.try_next().await.or(Err(()))? {
//             let num: i32 = row.try_get("testVar").or(Err(()))?;

//             test_nums.push(num);
//         }
//     }

//     Ok(test_nums)
// }

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
    pub const IN_PROGRESS_THEOREM_TABLE_CREATE: &str = "CREATE TABLE inProgressTheorem (
        name TEXT,
        text TEXT
    );";

    pub const IN_PROGRESS_THEOREM_TABLE_CHECK: &str =
        "SELECT name FROM sqlite_master WHERE type='table' AND name='inProgressTheorem';";
}
