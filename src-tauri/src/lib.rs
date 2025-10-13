use std::{fmt, sync::Arc};

use model::MetamathData;
use serde::Deserialize;
use tauri::{async_runtime::Mutex, App, AppHandle, Emitter, Listener, Manager};

use crate::model::{FolderData, IdManager};

mod editor;
mod explorer;
mod local_state;
mod metamath;
mod model;
mod search;
mod util;

pub struct AppState {
    metamath_data: Option<MetamathData>,
    // Used to temporarily store MetamathData before the user confirms they wants to open a database
    // This way the old MetamathData is not lost, if they cancel
    temp_metamath_data: Option<MetamathData>,
    stop_database_calculations: Arc<std::sync::Mutex<bool>>,
    stop_temp_database_calculations: Arc<std::sync::Mutex<bool>>,
    id_manager: IdManager,
    open_folder_data: Option<FolderData>,
    settings: Settings,
}

#[derive(Deserialize, Default, Clone)]
pub struct Settings {
    #[serde(rename = "definitionsStartWith")]
    definitons_start_with: String,
    #[serde(rename = "colorUnicodePreview")]
    _color_unicode_preview: bool,
    #[serde(rename = "showUnifyResultInUnicodePreview")]
    show_unify_result_in_unicode_preview: bool,
    #[serde(rename = "defaultShowAll")]
    _default_show_all: bool,
    #[serde(rename = "proofFormat")]
    proof_format: ProofFormatOption,
}

#[derive(Default, Clone, Copy)]
pub enum ProofFormatOption {
    #[default]
    Uncompressed,
    Compressed,
}

fn app_setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(Mutex::new(AppState {
        metamath_data: None,
        temp_metamath_data: None,
        stop_database_calculations: Arc::new(std::sync::Mutex::new(false)),
        stop_temp_database_calculations: Arc::new(std::sync::Mutex::new(false)),
        id_manager: IdManager::new(),
        open_folder_data: None,
        settings: Settings::default(),
    }));
    // app.manage::<Mutex<Option<AppState>>>(Mutex::new(None));
    Ok(())
}

#[tauri::command]
fn setup_main_window(window: tauri::Window, app: AppHandle) {
    window.listen("tauri://close-requested", move |_| {
        // Do nothing if emit fails
        app.emit("external-window-close", ()).ok();
    });
    window.get_webview_window("main").unwrap().show().unwrap();
}

#[tauri::command]
async fn set_settings(
    state: tauri::State<'_, Mutex<AppState>>,
    settings: Settings,
) -> Result<(), ()> {
    let mut app_state = state.lock().await;

    app_state.settings = settings;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
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
            setup_main_window,
            set_settings,
            explorer::add_header,
            explorer::quick_search,
            search::search_theorems,
            search::search_by_parse_tree_syntax_check,
            search::axiom_autocomplete,
            search::definition_autocomplete,
            editor::open_folder,
            editor::close_folder,
            editor::get_subfolder,
            editor::create_folder,
            editor::rename_folder,
            editor::delete_folder,
            editor::open_file,
            editor::save_file,
            editor::close_file,
            editor::create_file,
            editor::rename_file,
            editor::delete_file,
            editor::get_opened_folder_path,
            editor::external_window::open_external_window,
            editor::external_window::close_external_window,
            editor::external_window::set_up_external_window_close_listener,
            editor::format::format,
            editor::on_edit::on_edit,
            editor::parse_mmp::add_to_database,
            editor::renumber::renumber,
            editor::unify::unify,
            // metamath::turn_into_theorem,
            // metamath::text_to_constants,
            // metamath::text_to_variables,
            // metamath::text_to_floating_hypotheses,
            // metamath::text_to_html_representations,
            metamath::export::new_database,
            metamath::export::save_database,
            metamath::export::export_database,
            metamath::mm_parser::open_metamath_database,
            metamath::mm_parser::cancel_open_metamath_database,
            metamath::mm_parser::confirm_open_metamath_database,
            metamath::mm_parser::perform_grammar_calculations,
            local_state::comment::get_comment_local,
            local_state::constant::get_constants_local,
            local_state::constant::get_constant_statement_local,
            local_state::variable::get_variables_local,
            local_state::variable::get_variable_statement_local,
            local_state::floating_hypothesis::get_floating_hypothesis_page_data_local,
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

#[derive(Debug, Clone, Copy)]
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
    InvalidFormatError, // can be removed
    InvalidProofError,

    // Importing Database Errors
    ClosedUnopenedScopeError, // Returned if there is a "$}" statement in the outermost scope
    UnclosedCommentError,     // Returned if a comment was not closed
    UnclosedHeaderError,      // Returned if a header title was not closed
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
    TypesettingFormatError,     // Returned if there is a format error in a typesetting comment
    AdditionalInfoCommentFormatError, // Returned if there is a format error in a additional information comment
    InvalidColorCodeError, // Returned if there is an invalid colorcode in an (alt)varcolorcode comment
    ExpressionParseError,  // Returned if an expression could not be successfully parsed

    InternalLogicError,
    InvaildArgumentError,
    InvalidDatabaseDataError,

    NoOpenFolderError,
    FolderReadError,
    FileOpenError,
    FileReadError,
    FileWriteError,

    // Parsing mmp file errors
    NonAsciiSymbolError, // Returned if there is a non ascii symbol in the text
    WhitespaceBeforeFirstTokenError, // Returned if there is whitepace before the first token on the same line
    DuplicateSymbolError, // Returned if an added symbol (const, var or label) exists in the database already
    // EmptyConstStatementError: Also used when parsing mmp files
    TooManyConstStatementsError, // Returned if there is more than one const statement
    // EmptyVarStatementError: Also used when parsing mmp files
    // FloatHypStatementFormatError: Also used when parsing mmp files
    LabelAlreadyExistsError, // Returned if there is a floating hypothesis which label already exists
    SymbolAlreadyExistsError, // Returned if you are adding a constant/variable which symbol already exists as a label or math symbol
    InvalidMathSymbolError, // Returned if there is a symbol being declared that is not a valid math symbol
    TwiceDeclaredMathSymbolError, // Returned if a symbol is declared twice in the same $c/$v statement
    TypecodeNotAConstantError, // Returned if there is a floating hypothesis with a typecode that is not a constant
    ExpectedActiveVariableError, // Returned if there is a floating hypothesis declaring the typecode of a symbol that is not an active variable
    VariableTypecodeAlreadyDeclaredError, // Returned if there is a floating hypothesis declaring the typcode of a variable which typecode is already declared
    MultipleMmpLabelsError, // Returned if there are more than one $theorem, $axiom or $header statements
    TooFewHeaderTokensError, // Returned if there are less than 2 tokens after $header statement
    InvalidHeaderPathFormatError, // Returned if the token after $header does not have the format of a valid Headerpath
    InvalidHeaderPathError, // Returned if the token after $header is not a valid new header path in the database
    MissingCommentPathError, // Returned if there is a $comment statement without a follow up token
    TooManyCommentPathTokensError, // Returned if there is a $comment statement with too many follow up tokens
    InvalidCommentPathFormatError, // Returned if the token after $comment does not have the format of a valid comment path
    InvalidCommentPathError, // Returned if the token after $comment is not a valid new comment path in the database
    MissingAxiomLabelError,  // Returned if there is a $axiom statement without a follow up token
    TooManyAxiomLabelTokensError, // Returned if there is a $axiom statement with too many follow up tokens
    MissingTheoremLabelError, // Returned if there is a $theorem statement without a follow up token
    TooManyTheoremLabelTokensError, // Returned if there is a $theorem statement with too many follow up tokens
    InvalidMmpStepPrefixFormatError, // Returned if there is an invalid mmp step prefix, such as "x:x:x:x" or "x:x"
    InvalidMmpStepNameError, // Returned if there is an invalid mmp step name, such as "", or not alphanumeric
    InvalidMmpStepNameStartsWithHError, // Returned if there is a mmp step name that is invalid because it starts with h
    HypNameDoesntExistError, // Returned if there is an mmp step with an hypothesis name not belonging to any previous step
    MissingMmpStepRefError,  // Returned if there is an mmp step with an empty ref
    TooManyHypothesesError,  // Returned if a proof step withg a theoerm ref has too many hypotheses
    MmpStepRefNotALabelError, // Returned if there is an mmp step which ref is not a valid label
    InvalidMmpStepForAxiomError, // Returned when adding an axiom and the mmp steps do not follow the required format
    MissingMmpStepsError,        // Returned if there are no mmp steps when adding theorem/axiom
    MissingQedStepError,         // Returned if there is no qed step, but a $thereom statement
    DuplicateStepNameError,      // Returned if there is a duplicate step name
    DuplicateHypLabelsError,     // Returned if there are two hypotheses with the same label
    TheoremLabelNotFoundError,   // Returned if a theorem label in a mmp step prefix does not exist
    MmpStepMissingHypError,      // Returned if an mmp step has too few hyps
    HypothesisWithHypsError,     // Returned if an mmp hypothesis step has hypothesis itself
    InactiveMathSymbolError,     // Returned if an expression contains a symbol that is notr active
    VariableWithoutTypecode,     // Returned if there is a variable used without typecode
    MmpStepParseError, // Returend if there the earley parser does not return the proof of an expression
    VarSubedWithDifferentStrsError, //Returned if the same variable has been substituted with different Strings
    MultipleAllowDiscouragedError,  // Returned if there are multiple $allowdiscouraged statements
    TokensAfterAllowDiscouragedError, // Returned if there are tokens after $allowdiscouraged
    MultipleLocateAfterError, // Returned if there are multiple $locateafter(-var/-const) statements
    TooFewLocateAfterTokensError, // Returned if there is a $locateafter statement without a follow up token
    TooManyLocateAfterTokensError, // Returned if there is a $locateafter statement with too many follow up tokens
    TooFewLocateAfterConstTokensError, // Returned if there is a $locateafterconst statement without a follow up token
    TooManyLocateAfterConstTokensError, // Returned if there is a $locateafterconst statement with too many follow up tokens
    TooFewLocateAfterVarTokensError, // Returned if there is a $locateaftervar statement without a follow up token
    TooManyLocateAfterVarTokensError, // Returned if there is a $locateaftervar statement with too many follow up tokens
    InvalidLocateAfterError, // Returned if locateafter statement is not a real place in database
    MissingMmpStepExpressionError, // Returned if a mmp step is missing it's expression
    InvalidDollarTokenError, // Returned if there is a statement that starts with $ not followed by a vaild statement type
    StatementOutOfPlaceError, // Returned if there is a statement out of place, for example if there is a $v and a $c statement
    ConstStatementOutOfPlaceError, // Returned if there is a constant statement when there shouldn't be
    VarStatementOutOfPlaceError, // Returned if there is a variable statement when there shouldn't be
    FloatHypStatementOutOfPlaceError, // Returned if there is a floating hypothesis when there shouldn't be
    AllowDiscouragedOutOfPlaceError, // Returned if there is an allow discouraged statement when there shouldn't be
    DistinctVarOutOfPlaceError, // Returned if there is a distinct variable statement when there shouldn't be
    LocateAfterOutOfPlaceError, // Returned if there is a locate after statement when there shouldn't be
    ProofLinesOutOfPlaceError,  // Returned if there are proof lines when there shouldn't be
    InvalidWorkVariableError,   // Returned if a work variable's syntax is not correct
    UnificationError, // Returned from unification algorithm when a line can't be unified or from the unification command
    SyntaxTheoremUsedError, // Returned if a step ref references a syntax theorem

    MissingExpressionError, // Returned when converting str to number vec and skipping the first, but the str is empty
    InvalidTypecodeError,
    SyntaxTypecodeWithoutFloatHypsError,

    AddingToInnerScopeError,

    OpenExternalWindowError,

    OpenDatabaseStoppedEarlyError,
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

impl<'de> Deserialize<'de> for ProofFormatOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.as_str() {
            "uncompressed" => Ok(ProofFormatOption::Uncompressed),
            "compressed" => Ok(ProofFormatOption::Compressed),
            _ => Err(serde::de::Error::custom(
                "Expected uncompressed or compressed for proofFormat",
            )),
        }
    }
}
