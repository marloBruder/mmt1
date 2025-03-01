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
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok(header_path
        .resolve(&metamath_data.database_header)
        .ok_or(Error::NotFoundError)?
        .to_representation())
}

pub fn add_header_local(
    metamath_data: &mut MetamathData,
    title: &str,
    insert_path: &HeaderPath,
) -> Result<(), Error> {
    let mut header = &mut metamath_data.database_header;

    for (loop_index, &pos_index) in insert_path.path.iter().enumerate() {
        if loop_index != insert_path.path.len() - 1 {
            header = header
                .subheaders
                .get_mut(pos_index)
                .ok_or(Error::InternalLogicError)?;
        } else {
            header.subheaders.insert(
                pos_index,
                Header {
                    title: title.to_string(),
                    content: Vec::new(),
                    subheaders: Vec::new(),
                },
            );
        }
    }

    Ok(())
}
