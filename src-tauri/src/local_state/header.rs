use tauri::async_runtime::Mutex;

use crate::{
    model::{HeaderPageData, HeaderPath, HeaderRepresentation},
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

#[tauri::command]
pub async fn get_header_page_data(
    state: tauri::State<'_, Mutex<AppState>>,
    header_path: HeaderPath,
) -> Result<HeaderPageData, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok(header_path
        .resolve(&metamath_data.database_header)
        .ok_or(Error::NotFoundError)?
        .calc_page_data(&header_path, metamath_data)?)
}
