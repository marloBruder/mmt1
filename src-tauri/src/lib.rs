use std::fmt;

use model::MetamathData;
use tauri::{async_runtime::Mutex, App, Manager};

mod editor;
mod explorer;
mod local_state;
mod metamath;
mod model;
mod search;
mod util;

pub struct AppState {
    metamath_data: Option<MetamathData>,
    open_folder: Option<String>,
}

fn app_setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(Mutex::new(AppState {
        metamath_data: None,
        open_folder: None,
    }));
    // app.manage::<Mutex<Option<AppState>>>(Mutex::new(None));
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // database::create_database,
            // database::create_or_override_database,
            // database::open_database,
            // database::import_database,
            // database::import_and_override_database,
            // editor::add_in_progress_theorem,
            // editor::set_in_progress_theorem_name,
            // editor::set_in_progress_theorem,
            // editor::delete_in_progress_theorem,
            explorer::add_header,
            explorer::quick_search,
            search::search_theorems,
            editor::open_folder,
            editor::close_folder,
            editor::get_subfolder,
            editor::read_file,
            editor::save_file,
            editor::parse_mmp::add_to_database,
            metamath::turn_into_theorem,
            // metamath::text_to_constants,
            // metamath::text_to_variables,
            // metamath::text_to_floating_hypotheses,
            // metamath::text_to_html_representations,
            metamath::export::new_database,
            metamath::export::save_database,
            metamath::export::export_database,
            metamath::parse::open_metamath_database,
            metamath::unify::unify,
            local_state::comment::get_comment_local,
            local_state::constant::get_constants_local,
            local_state::variable::get_variables_local,
            local_state::floating_hypothesis::get_floating_hypothesis_local,
            local_state::floating_hypothesis::get_floating_hypotheses_local,
            local_state::theorem::get_theorem_page_data_local,
            local_state::theorem::get_theorem_list_local,
            local_state::header::get_header_local,
            local_state::html_representation::get_html_representations_local,
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
    NoMmDbError,

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
    NonVarInDisjError, // Returned if a symbol in a disjoint statement is not an active variable
    ZeroOrOneSymbolDisjError, // Returned if a disjoint statement is empty
    NonSymbolInExpressionError, // Returned if an expression contains a symbol that is neither a const or a var
    InvalidHeaderDepthError,    // Retured if a header has an invalid depth
    TokenOutsideStatementError, // Returned if a token does not belong to any statement
    MissingLabelError,          // Returned if a statement is missing its label
    InvalidLabelError,          // Returned if a label is not valid
    TypesettingFormatError,     // Returned if there is an format error in a typesetting comment

    LabelAlreadyExistsError,

    InternalLogicError,
    InvaildArgumentError,
    InvalidDatabaseDataError,

    NoOpenFolderError,
    FolderReadError,
    FileReadError,
    FileWriteError,

    // Parsing mmp file errors
    WhitespaceBeforeFirstTokenError, // Returned if there is whitepace before the first token
    MultipleHeaderStatementError,    // Returned if there are mutliple $header statements
    TooFewHeaderTokensError, // Returned if there are less than 2 tokens after $header statement
    MutipleConstStatementsError, // Returned if there are multipe $c statements
    // EmptyConstStatementError: Also used when parsing mmp files
    // EmptyVarStatementError: Also used when parsing mmp files
    // FloatHypStatementFormatError: Also used when parsing mmp files
    MultipleTheoremLabelError, // Returned if there are more than one $theorem statements
    MissingTheoremLabelError, // Returned if there is a $theorem statement without a follow up token
    TooManyTheoremLabelTokensError, // Returned if there is a $theorem statement with too many follow up tokens
    MultipleAxiomLabelError,        // Returned if there are more than one $axiom statements
    MissingAxiomLabelError, // Returned if there is a $axiom statement without a follow up token
    TooManyAxiomLabelTokensError, // Returned if there is a $axiom statement with too many follow up tokens
    MultipleAllowDiscouragedError, // Returned if there are multiple $allowdiscouraged statements
    TokensAfterAllowDiscouragedError, // Returned if there are tokens after $allowdiscouraged
    MultipleLocateAfterError, // Returned if there are multiple $locateafter(-var/-const) statements
    MissingLocateAfterLabelError, // Returned if there is a $locateafter statement without a follow up token
    TooManyLocateAfterTokensError, // Returned if there is a $locateafter statement with too many follow up tokens
    InvalidDollarTokenError, // Returned if there is a statement that starts with $ not followed by a vaild statement type
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
