use tauri::async_runtime::Mutex;

use crate::{
    model::{HeaderPath, HeaderRepresentation},
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
