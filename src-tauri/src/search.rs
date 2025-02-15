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
    let db_state = app_state.db_state.as_ref().ok_or(Error::NoDatabaseError)?;

    let mut result = Vec::new();

    // if let Some((theorem, theorem_number)) = db_state
    //     .metamath_data
    //     .theorem_list_header
    //     .find_theorem_by_name_calc_number(&search_parameters.label)
    // {
    //     result.push(theorem.to_theorem_list_entry(theorem_number));
    // }

    result.append(
        &mut db_state
            .metamath_data
            .theorem_list_header
            .theorem_iter()
            .enumerate()
            .filter(|(_, theorem)| theorem.name.contains(&search_parameters.label))
            .skip(search_parameters.start as usize)
            .take(search_parameters.amount as usize)
            .map(|(theorem_number, theorem)| {
                theorem.to_theorem_list_entry((theorem_number as u32) + 1)
            })
            .collect(),
    );

    Ok(result)
}
