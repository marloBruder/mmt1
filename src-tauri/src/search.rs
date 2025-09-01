use serde::Deserialize;
use tauri::async_runtime::Mutex;

use crate::{
    model::{ListEntry, TheoremListData},
    AppState, Error,
};

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
    let mut list: Vec<ListEntry> = Vec::new();
    let mut page_limits: Vec<(u32, u32)> = Vec::new();
    let mut last_page_start: Option<u32> = None;
    let mut last_theorem_number: Option<u32> = None;

    metamath_data
        .database_header
        .theorem_iter()
        .enumerate()
        .filter(|(_, theorem)| theorem.label.contains(&search_parameters.label))
        .for_each(|(theorem_number, theorem)| {
            last_theorem_number = Some((theorem_number + 1) as u32);

            if theorem_amount % 100 == 0 {
                last_page_start = Some((theorem_number + 1) as u32);
            } else if theorem_amount % 100 == 99 {
                page_limits.push((
                    last_page_start.take().unwrap_or(0),
                    (theorem_number + 1) as u32,
                ));
            }

            if search_parameters.page * 100 <= theorem_amount as u32
                && (theorem_amount as u32) < (search_parameters.page + 1) * 100
            {
                list.push(ListEntry::Theorem(theorem.to_theorem_list_entry(
                    (theorem_number + 1) as u32,
                    &metamath_data.optimized_data,
                )));
            }

            theorem_amount += 1;
        });

    if let Some(last_theorem_number) = last_theorem_number {
        if let Some(last_page_start) = last_page_start {
            page_limits.push((last_page_start, last_theorem_number));
        }
    }

    let page_amount = (((theorem_amount - 1) / 100) + 1) as u32;

    Ok(TheoremListData {
        list,
        page_amount,
        page_limits: Some(page_limits),
    })
}
