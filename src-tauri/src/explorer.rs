use std::u32;

use tauri::async_runtime::Mutex;

use crate::{
    database::header::{add_header_database, calc_db_index_for_header},
    local_state::header::add_header_local,
    model::{Header, HeaderPath},
    AppState, Error,
};

#[tauri::command]
pub async fn add_header(
    state: tauri::State<'_, Mutex<AppState>>,
    title: &str,
    insert_path: HeaderPath,
) -> Result<(), Error> {
    if insert_path.path.len() == 0 {
        return Err(Error::InvaildArgumentError);
    }

    let mut app_state = state.lock().await;
    let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

    add_header_local(&mut db_state.metamath_data, title, &insert_path)?;

    let db_index = calc_db_index_for_header(&db_state.metamath_data, &insert_path)?;
    let depth = (insert_path.path.len() as i32) - 1;

    add_header_database(&mut db_state.db_conn, db_index, depth, title).await?;

    Ok(())
}

// Returns a tuple (theorems, more), where theorems are theorem names that match the query
// and more is whether there exist more theorems that do as well.
// If only_ten is true, only ten theorems will be returned
#[tauri::command]
pub async fn quick_search(
    state: tauri::State<'_, Mutex<AppState>>,
    query: &str,
    only_ten: bool,
) -> Result<(Vec<String>, bool), Error> {
    let mut app_state = state.lock().await;
    let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

    let limit = if only_ten { 11 } else { u32::MAX };

    let mut theorems =
        find_theorem_names(&db_state.metamath_data.theorem_list_header, query, limit);

    let mut more = false;
    if only_ten && theorems.len() == 11 {
        more = true;
        theorems.pop();
    }

    Ok((theorems, more))
}

// Find all theorem names that match the query in the following order:
// 1: The name that fully matches the query (if it exists)
// 2: Names that start with the query
// 3: Names that contain the query
fn find_theorem_names(header: &Header, query: &str, limit: u32) -> Vec<String> {
    let mut theorems = Vec::new();

    let exact_match = header.find_theorem_by_name(query);

    if let Some(theorem) = exact_match {
        theorems.push(theorem.name.clone())
    }

    theorems.append(&mut find_theorem_names_helper(
        header,
        query,
        limit - (theorems.len() as u32),
        true,
    ));

    theorems.append(&mut find_theorem_names_helper(
        header,
        query,
        limit - (theorems.len() as u32),
        false,
    ));

    theorems
}

fn find_theorem_names_helper(
    header: &Header,
    query: &str,
    limit: u32,
    starts_with: bool,
) -> Vec<String> {
    let mut res = Vec::new();

    for theorem in &header.theorems {
        let theorem_starts_with = theorem.name.starts_with(query);
        if theorem.name != query
            && ((starts_with && theorem_starts_with)
                || (!starts_with && !theorem_starts_with && theorem.name.contains(query)))
        {
            res.push(theorem.name.clone());

            if res.len() as u32 == limit {
                return res;
            }
        }
    }

    for subheader in &header.sub_headers {
        res.append(&mut find_theorem_names_helper(
            subheader,
            query,
            limit - (res.len() as u32),
            starts_with,
        ));

        if res.len() as u32 == limit {
            return res;
        }
    }

    res
}
