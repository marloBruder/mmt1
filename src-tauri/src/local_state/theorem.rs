use tauri::async_runtime::Mutex;

use crate::{
    metamath::{self, calc_theorem_page_data},
    model::{HeaderRepresentation, Hypothesis, MetamathData, Theorem, TheoremPageData},
    AppState,
};

#[tauri::command]
pub async fn get_theorem_page_data_local(
    state: tauri::State<'_, Mutex<AppState>>,
    name: &str,
) -> Result<TheoremPageData, metamath::Error> {
    let app_state = state.lock().await;

    if let Some(ref mm_data) = app_state.metamath_data {
        for theorem in &mm_data.theorems {
            if theorem.name == name {
                return calc_theorem_page_data(&theorem, mm_data);
            }
        }
    }

    Err(metamath::Error::NotFoundError)
}

#[tauri::command]
pub async fn get_theorem_list_header_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<HeaderRepresentation, metamath::Error> {
    get_header_local(state, Vec::new()).await
}

#[tauri::command]
pub async fn get_header_local(
    state: tauri::State<'_, Mutex<AppState>>,
    location: Vec<i32>,
) -> Result<HeaderRepresentation, metamath::Error> {
    let app_state = state.lock().await;

    if let Some(ref mm_data) = app_state.metamath_data {
        let mut header = &mm_data.theorem_list_header;

        for i in location {
            header = header
                .sub_headers
                .get(i as usize)
                .ok_or(metamath::Error::NotFoundError)?;
        }

        return Ok(header.representation());
    }

    Err(metamath::Error::NoDatabaseOpenError)
}

#[tauri::command]
pub async fn get_theorem_names_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<String>, ()> {
    let app_state = state.lock().await;

    if let Some(ref mm_data) = app_state.metamath_data {
        let mut names: Vec<String> = Vec::new();
        for theorem in &mm_data.theorems {
            names.push(theorem.name.clone());
        }
        return Ok(names);
    }

    Err(())
}

pub fn get_theorem_by_name_local<'a>(
    metamath_data: &'a MetamathData,
    name: &str,
) -> Result<&'a Theorem, metamath::Error> {
    for theorem in &metamath_data.theorems {
        if theorem.name == name {
            return Ok(&theorem);
        }
    }

    Err(metamath::Error::NotFoundError)
}

pub fn add_theorem_local(
    metamath_data: &mut MetamathData,
    name: &str,
    description: &str,
    disjoints: &Vec<String>,
    hypotheses: &Vec<Hypothesis>,
    assertion: &str,
    proof: Option<&str>,
) {
    metamath_data.theorems.push(Theorem {
        name: name.to_string(),
        description: description.to_string(),
        disjoints: disjoints.clone(),
        hypotheses: hypotheses.clone(),
        assertion: assertion.to_string(),
        proof: proof.map(|s| s.to_string()),
    })
}
