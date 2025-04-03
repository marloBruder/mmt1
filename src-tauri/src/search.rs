use serde::Deserialize;
use tauri::async_runtime::Mutex;

use crate::{model::TheoremListData, AppState, Error};

// page starts at 0
#[derive(Deserialize)]
pub struct SearchParameters {
    pub page: u32,
    pub label: String,
}

#[tauri::command]
pub async fn search_theorems(
    state: tauri::State<'_, Mutex<AppState>>,
    search_parameters: SearchParameters,
) -> Result<TheoremListData, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let mut theorem_amount: i32 = 0;
    let mut list = Vec::new();

    metamath_data
        .database_header
        .theorem_iter()
        .enumerate()
        .filter(|(_, theorem)| theorem.label.contains(&search_parameters.label))
        .enumerate()
        .for_each(|(search_result_number, (theorem_number, theorem))| {
            if search_parameters.page * 100 <= search_result_number as u32
                && (search_result_number as u32) < (search_parameters.page + 1) * 100
            {
                list.push(theorem.to_theorem_list_entry((theorem_number + 1) as u32));
            }
            theorem_amount += 1;
        });

    let page_amount = (((theorem_amount - 1) / 100) + 1) as u32;

    Ok(TheoremListData { list, page_amount })
}
