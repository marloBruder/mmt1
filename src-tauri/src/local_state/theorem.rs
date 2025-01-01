use tauri::async_runtime::Mutex;

use crate::{
    metamath::{self, calc_theorem_page_data},
    model::{Header, HeaderRepresentation, Hypothesis, MetamathData, Theorem, TheoremPageData},
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
    get_theorem_by_name_relative(&metamath_data.theorem_list_header, name)
        .ok_or(metamath::Error::NotFoundError)
}

fn get_theorem_by_name_relative<'a>(header: &'a Header, name: &str) -> Option<&'a Theorem> {
    for theorem in &header.theorems {
        if theorem.name == name {
            return Some(theorem);
        }
    }

    for sub_header in &header.sub_headers {
        let sub_header_res = get_theorem_by_name_relative(sub_header, name);
        if sub_header_res.is_some() {
            return sub_header_res;
        }
    }

    None
}

pub fn get_theorem_position_by_name_local(
    metamath_data: &mut MetamathData,
    name: &str,
) -> Option<Vec<i32>> {
    get_theorem_position_by_name_relative(&metamath_data.theorem_list_header, name)
}

fn get_theorem_position_by_name_relative(header: &Header, name: &str) -> Option<Vec<i32>> {
    for (index, theorem) in header.theorems.iter().enumerate() {
        if theorem.name == name {
            let mut res = Vec::new();
            res.push(index as i32);
            return Some(res);
        }
    }

    for (index, sub_header) in header.sub_headers.iter().enumerate() {
        let sub_header_res = get_theorem_position_by_name_relative(sub_header, name);
        if let Some(mut res) = sub_header_res {
            res.insert(0, index as i32);
            return Some(res);
        }
    }

    None
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
