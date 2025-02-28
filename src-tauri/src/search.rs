use serde::Deserialize;
use tauri::async_runtime::Mutex;

use crate::{model::TheoremListEntry, AppState, Error};

#[derive(Deserialize)]
pub struct SearchParameters {
    pub start: u32,
    pub amount: u32,
    pub label: String,
}

#[tauri::command]
pub async fn search_theorems(
    state: tauri::State<'_, Mutex<AppState>>,
    search_parameters: SearchParameters,
) -> Result<Vec<TheoremListEntry>, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok(metamath_data
        .database_header
        .theorem_iter()
        .enumerate()
        .filter(|(_, theorem)| theorem.label.contains(&search_parameters.label))
        .skip(search_parameters.start as usize)
        .take(search_parameters.amount as usize)
        .map(|(theorem_number, theorem)| theorem.to_theorem_list_entry((theorem_number as u32) + 1))
        .collect())
}
