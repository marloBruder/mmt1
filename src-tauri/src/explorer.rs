use std::u32;

use tauri::async_runtime::Mutex;

use crate::{
    // database::header::{add_header_database, calc_db_index_for_header},
    local_state::header::add_header_local,
    model::HeaderPath,
    search,
    AppState,
    Error,
};

#[tauri::command]
pub async fn add_header(
    state: tauri::State<'_, Mutex<AppState>>,
    title: &str,
    insert_path: HeaderPath,
) -> Result<(), Error> {
    if insert_path.path.len() == 0 {
        return Err(Error::InvaildArgumentError);
    }

    let mut app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_mut().ok_or(Error::NoMmDbError)?;

    add_header_local(metamath_data, title, &insert_path)?;

    // let db_index = calc_db_index_for_header(&db_state.metamath_data, &insert_path)?;
    // let depth = (insert_path.path.len() as i32) - 1;

    // add_header_database(&mut db_state.db_conn, db_index, depth, title).await?;

    Ok(())
}

// Returns a tuple (theorems, more), where theorems are theorem names that match the query
// and more is whether there exist more theorems that do as well.
// If only_ten is true, only ten theorems will be returned
#[tauri::command]
pub async fn quick_search(
    state: tauri::State<'_, Mutex<AppState>>,
    query: &str,
    only_ten: bool,
) -> Result<(Vec<String>, bool), Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let limit = if only_ten { 11 } else { u32::MAX };

    let mut theorems =
        search::find_theorem_labels(&metamath_data.database_header, query, limit, |_| true);

    let mut more = false;
    if only_ten && theorems.len() == 11 {
        more = true;
        theorems.pop();
    }

    Ok((theorems, more))
}
