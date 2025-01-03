use tauri::async_runtime::Mutex;

use crate::{
    metamath::{self, calc_theorem_page_data},
    model::{Header, Hypothesis, MetamathData, Theorem, TheoremPageData},
    AppState,
};

use super::header::get_header_position_by_title_local;

#[tauri::command]
pub async fn get_theorem_page_data_local(
    state: tauri::State<'_, Mutex<AppState>>,
    name: &str,
) -> Result<TheoremPageData, metamath::Error> {
    let app_state = state.lock().await;

    if let Some(ref mm_data) = app_state.metamath_data {
        let theorem = get_theorem_by_name_local(mm_data, name)?;
        return calc_theorem_page_data(theorem, mm_data);
    }

    Err(metamath::Error::NotFoundError)
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
    metamath_data: &MetamathData,
    name: &str,
) -> Option<Vec<usize>> {
    get_theorem_position_by_name_relative(&metamath_data.theorem_list_header, name)
}

fn get_theorem_position_by_name_relative(header: &Header, name: &str) -> Option<Vec<usize>> {
    for (index, theorem) in header.theorems.iter().enumerate() {
        if theorem.name == name {
            let mut res = Vec::new();
            res.push(index);
            return Some(res);
        }
    }

    for (index, sub_header) in header.sub_headers.iter().enumerate() {
        let sub_header_res = get_theorem_position_by_name_relative(sub_header, name);
        if let Some(mut res) = sub_header_res {
            res.insert(0, index);
            return Some(res);
        }
    }

    None
}

pub fn get_theorem_insert_position(
    metamath_data: &MetamathData,
    position_name: &str,
) -> Result<Vec<usize>, metamath::Error> {
    if position_name.contains(' ') {
        // Safe unwrap because of the prior condition
        let (_, header_title) = position_name.split_once(' ').unwrap();
        let header_position = get_header_position_by_title_local(metamath_data, header_title);
        if let Some(pos) = header_position {
            let mut position = pos;
            position.push(0);
            return Ok(position);
        } else {
            return Err(metamath::Error::NotFoundError);
        }
    } else {
        let theorem_position = get_theorem_position_by_name_local(metamath_data, position_name);
        if let Some(pos) = theorem_position {
            let mut position = pos;
            *position
                .last_mut()
                .ok_or(metamath::Error::InternalLogicError)? += 1;
            return Ok(position);
        } else {
            return Err(metamath::Error::NotFoundError);
        }
    }
}

pub fn add_theorem_local(
    metamath_data: &mut MetamathData,
    name: &str,
    description: &str,
    disjoints: &Vec<String>,
    hypotheses: &Vec<Hypothesis>,
    assertion: &str,
    proof: Option<&str>,
    insert_position: &Vec<usize>,
) -> Result<(), metamath::Error> {
    let mut section = &mut metamath_data.theorem_list_header;
    for (loop_index, &pos_index) in insert_position.iter().enumerate() {
        if loop_index != insert_position.len() - 1 {
            section = section
                .sub_headers
                .get_mut(pos_index)
                .ok_or(metamath::Error::InternalLogicError)?;
        } else {
            section.theorems.insert(
                pos_index,
                Theorem {
                    name: name.to_string(),
                    description: description.to_string(),
                    disjoints: disjoints.clone(),
                    hypotheses: hypotheses.clone(),
                    assertion: assertion.to_string(),
                    proof: proof.map(|s| s.to_string()),
                },
            );
        }
    }

    Ok(())
}
