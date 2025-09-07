use serde::Deserialize;
use tauri::async_runtime::Mutex;

use crate::{
    model::{Header, ListEntry, Theorem, TheoremListData},
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

// If successful, returns a tuple (a,b) where:
// a is whether the query is a valid axiom label
// b is a list of 5 axiom labels to be shown as autocomplete
#[tauri::command]
pub async fn axiom_autocomplete(
    state: tauri::State<'_, Mutex<AppState>>,
    query: &str,
    items: Vec<&str>,
) -> Result<(bool, Vec<String>), Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok((
        metamath_data
            .database_header
            .find_theorem_by_label(query)
            .is_some_and(|theorem| {
                theorem.proof.is_none()
                    && !theorem.label.starts_with("df-")
                    && !items.contains(&&*theorem.label)
            }),
        find_theorem_labels(&metamath_data.database_header, query, 5, |theorem| {
            theorem.label != query
                && theorem.proof.is_none()
                && !theorem.label.starts_with("df-")
                && !items.contains(&&*theorem.label)
        }),
    ))
}

// Find all theorem labels that match the query in the following order:
// 1: The name that fully matches the query (if it exists)
// 2: Labels that start with the query
// 3: Labels that contain the query
pub fn find_theorem_labels<T>(header: &Header, query: &str, limit: u32, filter: T) -> Vec<String>
where
    T: Fn(&Theorem) -> bool,
{
    let mut theorems = Vec::new();

    let exact_match = header.find_theorem_by_label(query);

    if let Some(theorem) = exact_match {
        if filter(theorem) {
            theorems.push(theorem.label.clone())
        }
    }

    theorems.extend(
        header
            .theorem_iter()
            .filter(|t| t.label != query && t.label.starts_with(query) && filter(t))
            .take((limit as usize) - theorems.len())
            .map(|t| t.label.clone()),
    );

    theorems.extend(
        header
            .theorem_iter()
            .filter(|t| {
                t.label != query
                    && !t.label.starts_with(query)
                    && t.label.contains(query)
                    && filter(t)
            })
            .take((limit as usize) - theorems.len())
            .map(|t| t.label.clone()),
    );

    theorems
}
