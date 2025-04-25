use tauri::async_runtime::Mutex;

use crate::{
    editor::parse_mmp::{self, Mmj2StepProcessed},
    AppState, Error,
};

// #[tauri::command]
// pub async fn unify_and_format(text: &str) -> Result<String, Error> {
//     let mut res = String::new();

//     let statements = parse_mmp::text_to_statements(text)?;
//     let mmp_structured_info = parse_mmp::statements_to_mmp_structured_info(statements)?;

//     if mmp_structured_info.statement_out_of_place() {
//         return Err(Error::StatementOutOfPlaceError);
//     }

//     if let Some(label) = mmp_structured_info.theorem_label.as_ref() {
//         res.push_str("$theorem ");
//         res.push_str(label);
//         res.push('\n');
//     }

//     if let Some(label) = mmp_structured_info.axiom_label.as_ref() {
//         res.push_str("$axiom ");
//         res.push_str(label);
//         res.push('\n');
//     }

//     Ok(res)
// }

#[tauri::command]
pub async fn unify(state: tauri::State<'_, Mutex<AppState>>, text: &str) -> Result<String, Error> {
    let mut app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_mut().ok_or(Error::NoMmDbError)?;

    if !text.is_ascii() {
        return Err(Error::InvalidCharactersError);
    }

    let mut res = String::new();

    let (whitespace_before_first_statement, statements) = text_to_statement_strs(text)?;

    Ok(res)
}

// IF successful, returns a tuple (a,b) where a is the whitespace before the first line and b is a vec of all the lines
pub fn text_to_statement_strs(text: &str) -> Result<(&str, Vec<&str>), Error> {
    let mut statements = Vec::new();

    let mut text_i: usize = 0;
    let text_bytes = text.as_bytes();

    while text_bytes
        .get(text_i)
        .is_some_and(|c| c.is_ascii_whitespace())
    {
        text_i += 1;
    }

    let whitespace_before_first_statement = text.get(0..text_i).ok_or(Error::InternalLogicError)?;

    if text_i != 0 && text_bytes.get(text_i - 1).is_some_and(|c| *c != b'\n') {
        return Err(Error::WhitespaceBeforeFirstTokenError);
    }

    let mut statement_start = text_i;
    text_i += 1;

    while let Some(&char) = text_bytes.get(text_i) {
        if !char.is_ascii_whitespace() && text_bytes.get(text_i - 1).is_some_and(|c| *c == b'\n') {
            statements.push(
                text.get(statement_start..text_i)
                    .ok_or(Error::InternalLogicError)?,
            );
            statement_start = text_i;
        }

        text_i += 1;
    }

    statements.push(
        text.get(statement_start..text_i)
            .ok_or(Error::InternalLogicError)?,
    );

    println!("{:?}", statements);

    Ok((whitespace_before_first_statement, statements))
}
