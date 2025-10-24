use std::fs;

use tauri::async_runtime::Mutex;

use crate::{
    metamath::{self, mmp_parser::LocateAfterRef},
    model::TheoremPageData,
    util, AppState, Error,
};

#[tauri::command]
pub async fn get_theorem_page_data(
    state: tauri::State<'_, Mutex<AppState>>,
    label: &str,
    show_all: bool,
) -> Result<TheoremPageData, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    metamath::calc_theorem_page_data(label, metamath_data, show_all)
}

#[tauri::command]
pub async fn get_theorem_mmp_format(
    state: tauri::State<'_, Mutex<AppState>>,
    label: &str,
) -> Result<String, Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    util::locate_after_to_mmp_file_format_of_statement_it_refers_to(
        LocateAfterRef::LocateAfter(label),
        mm_data,
    )
}

#[tauri::command]
pub async fn write_theorem_mmp_format_to_file(
    state: tauri::State<'_, Mutex<AppState>>,
    label: &str,
    file_path: &str,
) -> Result<(), Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let mmp_format = util::locate_after_to_mmp_file_format_of_statement_it_refers_to(
        LocateAfterRef::LocateAfter(label),
        mm_data,
    )?;

    fs::write(file_path, mmp_format).map_err(|_| Error::FileWriteError)?;

    Ok(())
}
