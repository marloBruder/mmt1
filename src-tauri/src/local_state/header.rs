use tauri::async_runtime::Mutex;

use crate::{
    model::{Header, HeaderPath, HeaderRepresentation, MetamathData},
    AppState, Error,
};

#[tauri::command]
pub async fn get_header_local(
    state: tauri::State<'_, Mutex<AppState>>,
    header_path: HeaderPath,
) -> Result<HeaderRepresentation, Error> {
    let app_state = state.lock().await;
    let db_state = app_state.db_state.as_ref().ok_or(Error::NoDatabaseError)?;

    Ok(header_path
        .resolve(&db_state.metamath_data.theorem_list_header)
        .ok_or(Error::NotFoundError)?
        .representation())
}

pub fn add_header_local(
    metamath_data: &mut MetamathData,
    title: &str,
    insert_path: &HeaderPath,
) -> Result<(), Error> {
    let mut header = &mut metamath_data.theorem_list_header;

    for (loop_index, &pos_index) in insert_path.path.iter().enumerate() {
        if loop_index != insert_path.path.len() - 1 {
            header = header
                .sub_headers
                .get_mut(pos_index)
                .ok_or(Error::InternalLogicError)?;
        } else {
            header.sub_headers.insert(
                pos_index,
                Header {
                    title: title.to_string(),
                    theorems: Vec::new(),
                    sub_headers: Vec::new(),
                },
            );
        }
    }

    Ok(())
}
