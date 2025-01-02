use tauri::async_runtime::Mutex;

use crate::{
    metamath,
    model::{Header, HeaderRepresentation, MetamathData},
    AppState,
};

#[tauri::command]
pub async fn get_theorem_list_header_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<HeaderRepresentation, metamath::Error> {
    get_header_local(state, Vec::new()).await
}

#[tauri::command]
pub async fn get_header_local(
    state: tauri::State<'_, Mutex<AppState>>,
    location: Vec<usize>,
) -> Result<HeaderRepresentation, metamath::Error> {
    let app_state = state.lock().await;

    if let Some(ref mm_data) = app_state.metamath_data {
        let mut header = &mm_data.theorem_list_header;

        for i in location {
            header = header
                .sub_headers
                .get(i)
                .ok_or(metamath::Error::NotFoundError)?;
        }

        return Ok(header.representation());
    }

    Err(metamath::Error::NoDatabaseOpenError)
}

pub fn get_header_position_by_title_local(
    metamath_data: &MetamathData,
    title: &str,
) -> Option<Vec<usize>> {
    get_header_position_by_title_relative(&metamath_data.theorem_list_header, title)
}

fn get_header_position_by_title_relative(header: &Header, title: &str) -> Option<Vec<usize>> {
    for (index, sub_header) in header.sub_headers.iter().enumerate() {
        if sub_header.title == title {
            let mut res = Vec::new();
            res.push(index);
            return Some(res);
        }

        let sub_header_res = get_header_position_by_title_relative(sub_header, title);
        if let Some(mut res) = sub_header_res {
            res.insert(0, index);
            return Some(res);
        }
    }

    None
}
