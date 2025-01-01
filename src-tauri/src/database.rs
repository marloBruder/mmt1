use sqlx::{migrate::MigrateDatabase, Connection, Sqlite, SqliteConnection};
use std::fmt;
use tauri::async_runtime::Mutex;

use crate::{model::MetamathData, AppState};

pub mod constant;
pub mod floating_hypothesis;
pub mod in_progress_theorem;
pub mod theorem;
pub mod variable;

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
    app_state.metamath_data = Some(Default::default());

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

    sql::check_returns_rows_or_error(sql::CONSTANT_TABLE_CHECK, &mut conn).await?;
    sql::check_returns_rows_or_error(sql::VARIABLE_TABLE_CHECK, &mut conn).await?;
    sql::check_returns_rows_or_error(sql::FLOATING_HYPOTHESIS_TABLE_CHECK, &mut conn).await?;
    sql::check_returns_rows_or_error(sql::THEOREM_TABLE_CHECK, &mut conn).await?;
    sql::check_returns_rows_or_error(sql::IN_PROGRESS_THEOREM_TABLE_CHECK, &mut conn).await?;
    sql::check_returns_rows_or_error(sql::HEADER_TABLE_CHECK, &mut conn).await?;

    let mut app_state = state.lock().await;

    let constants = constant::get_constants_database(&mut conn).await?;
    let variables = variable::get_variables_database(&mut conn).await?;
    let floating_hypotheses =
        floating_hypothesis::get_floating_hypotheses_database(&mut conn).await?;
    let theorem_list_header = theorem::get_theorem_list_header_database(&mut conn).await?;
    let in_progress_theorems =
        in_progress_theorem::get_in_progress_theorems_database(&mut conn).await?;

    app_state.db_conn = Some(conn);
    app_state.metamath_data = Some(MetamathData {
        constants,
        variables,
        floating_hypotheses,
        theorems: Vec::new(),
        in_progress_theorems,
        theorem_list_header,
    });

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    DatabaseExistsError,
    CreateDatabaseError,
    ConnectDatabaseError,
    WrongDatabaseFormatError,
    SqlError,
    InvalidDataError,
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
CREATE TABLE constant (
    [index] INTEGER PRIMARY KEY,
    symbol TEXT
);
CREATE TABLE variable (
    [index] INTEGER PRIMARY KEY,
    symbol TEXT
);
CREATE TABLE floating_hypothesis (
    [index] INTEGER PRIMARY KEY,
    label TEXT,
    typecode TEXT,
    variable TEXT
);
CREATE TABLE theorem (
    db_index INTEGER PRIMARY KEY,
    name TEXT,
    description TEXT,
    disjoints TEXT,
    hypotheses TEXT,
    assertion TEXT,
    proof TEXT NULL
);
CREATE TABLE in_progress_theorem (
    name TEXT PRIMARY KEY,
    text TEXT
);
CREATE TABLE header (
    db_index INTEGER
    depth INTEGER
    title TEXT
);
";

    pub const CONSTANT_TABLE_CHECK: &str =
        "SELECT name FROM sqlite_master WHERE type='table' AND name='constant';";
    pub const VARIABLE_TABLE_CHECK: &str =
        "SELECT name FROM sqlite_master WHERE type='table' AND name='variable';";
    pub const FLOATING_HYPOTHESIS_TABLE_CHECK: &str =
        "SELECT name FROM sqlite_master WHERE type='table' AND name='floating_hypothesis';";
    pub const THEOREM_TABLE_CHECK: &str =
        "SELECT name FROM sqlite_master WHERE type='table' AND name='theorem';";
    pub const IN_PROGRESS_THEOREM_TABLE_CHECK: &str =
        "SELECT name FROM sqlite_master WHERE type='table' AND name='in_progress_theorem';";
    pub const HEADER_TABLE_CHECK: &str =
        "SELECT name FROM sqlite_master WHERE type='table' AND name='header';";

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
