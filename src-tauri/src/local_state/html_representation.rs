use crate::{
    model::{ColorInformation, HtmlRepresentation},
    AppState, Error,
};
use tauri::async_runtime::Mutex;

#[tauri::command]
pub async fn get_html_representations(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(Vec<HtmlRepresentation>, Vec<ColorInformation>), Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok((
        metamath_data.html_representations.clone(),
        metamath_data.calc_color_information(true),
    ))
}
