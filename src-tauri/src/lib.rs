use std::fmt;

use model::MetamathData;
use sqlx::SqliteConnection;
use tauri::{async_runtime::Mutex, App, Manager};

mod database;
mod editor;
mod explorer;
mod local_state;
mod metamath;
mod model;

pub struct AppState {
    db_state: Option<DatabaseState>,
}

pub struct DatabaseState {
    db_conn: SqliteConnection,
    metamath_data: MetamathData,
}

fn app_setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(Mutex::new(AppState { db_state: None }));
    // app.manage::<Mutex<Option<AppState>>>(Mutex::new(None));
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
            database::import_database,
            database::import_and_override_database,
            editor::add_in_progress_theorem,
            editor::set_in_progress_theorem_name,
            editor::set_in_progress_theorem,
            editor::delete_in_progress_theorem,
            explorer::add_header,
            metamath::turn_into_theorem,
            metamath::text_to_constants,
            metamath::text_to_variables,
            metamath::text_to_floating_hypotheses,
            local_state::constant::get_constants_local,
            local_state::variable::get_variables_local,
            local_state::floating_hypothesis::get_floating_hypotheses_local,
            local_state::theorem::get_theorem_page_data_local,
            local_state::theorem::get_theorem_names_local,
            local_state::header::get_header_local,
            local_state::in_progress_theorem::get_in_progress_theorem_local,
            local_state::in_progress_theorem::get_in_progress_theorem_names_local,
        ])
        .setup(|app| app_setup(app))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Debug)]
pub enum Error {
    DatabaseExistsError,
    CreateDatabaseError,
    ConnectDatabaseError,
    WrongDatabaseFormatError,
    NoDatabaseError,

    FileNotFoundError,

    SqlError,

    NotFoundError,

    InvalidCharactersError,
    InvalidFormatError,
    InvalidProofError,

    // Importing Database Errors
    ClosedUnopenedScopeError, // Returned if there is a "$}" statement in the outermost scope
    ConstStatementScopeError, // Returned if a constant statement is found outside outermost scope
    EmptyConstStatementError, // Returned if a constant statement has no symbols
    TwiceDeclaredConstError,  // Returned if a constant has been declared twice
    EmptyVarStatementError,   // Returned if a variable statement has no symbols
    TwiceDeclaredVarError,    // Returned if a variable has been declared twice
    InvalidSymbolError,       // Returned if a symbol contains invalid characters
    FloatHypStatementFormatError, // Returned if a floating hypothesis statement does not have 2 tokens
    FloatHypTypecodeError, // Returned if a floating hypothesis statements typecode is not an active constant
    FloatHypVariableError, // Returned if a floating hypothesis statements variable is not an active variable
    VarTypeDeclaredTwiceError, // Returned if a variable appears in two active $f statements
    VarDeclaredMultipleTypesError, // Returned if a variable has been declared with two different types
    TokenOutsideStatementError,    // Returned if a token does not belong to any statement
    MissingLabelError,             // Returned if a statement is missing its label

    InternalLogicError,
    InvaildArgumentError,
    InvalidDatabaseDataError,
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
