use std::u32;

use tauri::async_runtime::Mutex;

use crate::{search, AppState, Error};

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
