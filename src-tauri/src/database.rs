use sqlx::{migrate::MigrateDatabase, Connection, Sqlite, SqliteConnection};
use std::fmt;
use tauri::async_runtime::Mutex;

use crate::{model::MetamathData, AppState};

pub mod constant;
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
    sql::check_returns_rows_or_error(sql::THEOREM_TABLE_CHECK, &mut conn).await?;
    sql::check_returns_rows_or_error(sql::IN_PROGRESS_THEOREM_TABLE_CHECK, &mut conn).await?;

    let mut app_state = state.lock().await;
    app_state.db_conn = Some(conn);
    drop(app_state);

    let constants = constant::get_constants(&state).await?;
    let variables = variable::get_variable(&state).await?;
    let theorems = theorem::get_theorems(&state).await?;
    let in_progress_theorems = in_progress_theorem::get_in_progress_theorems(&state).await?;

    let mut app_state = state.lock().await;
    app_state.metamath_data = Some(MetamathData {
        constants,
        variables,
        theorems,
        in_progress_theorems,
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
    use super::Error;
    use crate::AppState;
    use sqlx::{Sqlite, SqliteConnection, Type};
    use tauri::async_runtime::Mutex;

    pub const INIT_DB: &str = "\
CREATE TABLE constant (
    [index] INTEGER PRIMARY KEY,
    symbol TEXT
);
CREATE TABLE variable (
    [index] INTEGER PRIMARY KEY,
    symbol TEXT
);
CREATE TABLE theorem (
    name TEXT PRIMARY KEY,
    description TEXT,
    disjoints TEXT,
    hypotheses TEXT,
    assertion TEXT,
    proof TEXT NULL
);
CREATE TABLE inProgressTheorem (
    name TEXT PRIMARY KEY,
    text TEXT
);
";

    pub const CONSTANT_TABLE_CHECK: &str =
        "SELECT name FROM sqlite_master WHERE type='table' AND name='constant';";
    pub const VARIABLE_TABLE_CHECK: &str =
        "SELECT name FROM sqlite_master WHERE type='table' AND name='variable';";
    pub const THEOREM_TABLE_CHECK: &str =
        "SELECT name FROM sqlite_master WHERE type='table' AND name='theorem';";
    pub const IN_PROGRESS_THEOREM_TABLE_CHECK: &str =
        "SELECT name FROM sqlite_master WHERE type='table' AND name='inProgressTheorem';";

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

    pub async fn execute_query_one_bind<'a, T>(
        state: &tauri::State<'_, Mutex<AppState>>,
        query: &'static str,
        bind_one: T,
    ) -> Result<(), Error>
    where
        T: sqlx::Encode<'a, Sqlite> + Type<Sqlite> + 'a,
    {
        let mut app_state = state.lock().await;

        if let Some(ref mut conn) = app_state.db_conn {
            sqlx::query(query)
                .bind(bind_one)
                .execute(conn)
                .await
                .or(Err(Error::SqlError))?;
        }
        Ok(())
    }

    pub async fn execute_query_two_bind<'a, T, S>(
        state: &tauri::State<'_, Mutex<AppState>>,
        query: &'static str,
        bind_one: T,
        bind_two: S,
    ) -> Result<(), Error>
    where
        T: sqlx::Encode<'a, Sqlite> + Type<Sqlite> + 'a,
        S: sqlx::Encode<'a, Sqlite> + Type<Sqlite> + 'a,
    {
        let mut app_state = state.lock().await;

        if let Some(ref mut conn) = app_state.db_conn {
            sqlx::query(query)
                .bind(bind_one)
                .bind(bind_two)
                .execute(conn)
                .await
                .or(Err(Error::SqlError))?;
        }
        Ok(())
    }
}
