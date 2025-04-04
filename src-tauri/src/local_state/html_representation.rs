use crate::{model::HtmlRepresentation, AppState, Error};
use tauri::async_runtime::Mutex;

#[tauri::command]
pub async fn get_html_representations_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<HtmlRepresentation>, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok(metamath_data.html_representations.clone())
}

// pub fn set_html_representations_local(
//     metamath_data: &mut MetamathData,
//     html_representations: &Vec<HtmlRepresentation>,
// ) {
//     metamath_data.html_representations = html_representations.clone();
// }
