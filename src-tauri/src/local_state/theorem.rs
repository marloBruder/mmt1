use tauri::async_runtime::Mutex;

use crate::{metamath, model::TheoremPageData, AppState, Error};

#[tauri::command]
pub async fn get_theorem_page_data_local(
    state: tauri::State<'_, Mutex<AppState>>,
    label: &str,
    show_all: bool,
) -> Result<TheoremPageData, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    metamath::calc_theorem_page_data(label, metamath_data, show_all)
}
