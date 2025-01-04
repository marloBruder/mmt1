use tauri::async_runtime::Mutex;

use crate::{
    metamath::{self, calc_theorem_page_data},
    model::{Hypothesis, MetamathData, Theorem, TheoremPageData, TheoremPath},
    AppState,
};

#[tauri::command]
pub async fn get_theorem_page_data_local(
    state: tauri::State<'_, Mutex<AppState>>,
    name: &str,
) -> Result<TheoremPageData, metamath::Error> {
    let app_state = state.lock().await;

    if let Some(ref mm_data) = app_state.metamath_data {
        let theorem = mm_data
            .theorem_list_header
            .find_theorem_by_name(name)
            .ok_or(metamath::Error::NotFoundError)?;

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

pub fn get_theorem_insert_position(
    metamath_data: &MetamathData,
    position_name: &str,
) -> Result<TheoremPath, metamath::Error> {
    if position_name.contains(' ') {
        // Safe unwrap because of the prior condition
        let (_, header_title) = position_name.split_once(' ').unwrap();
        let header_path_res = metamath_data
            .theorem_list_header
            .calc_header_path_by_title(header_title);

        if let Some(header_path) = header_path_res {
            return Ok(TheoremPath {
                header_path,
                theorem_index: 0,
            });
        } else {
            return Err(metamath::Error::NotFoundError);
        }
    } else {
        let theorem_path_res = metamath_data
            .theorem_list_header
            .calc_theorem_path_by_name(position_name);

        if let Some(mut theorem_path) = theorem_path_res {
            theorem_path.theorem_index += 1;
            return Ok(theorem_path);
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
    insert_path: &TheoremPath,
) -> Result<(), metamath::Error> {
    let header = insert_path
        .header_path
        .resolve_mut(&mut metamath_data.theorem_list_header)
        .ok_or(metamath::Error::NotFoundError)?;

    if header.theorems.len() < insert_path.theorem_index {
        return Err(metamath::Error::NotFoundError);
    }

    header.theorems.insert(
        insert_path.theorem_index,
        Theorem {
            name: name.to_string(),
            description: description.to_string(),
            disjoints: disjoints.clone(),
            hypotheses: hypotheses.clone(),
            assertion: assertion.to_string(),
            proof: proof.map(|s| s.to_string()),
        },
    );

    Ok(())
}
