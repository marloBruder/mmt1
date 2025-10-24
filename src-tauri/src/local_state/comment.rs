use std::fs;

use tauri::async_runtime::Mutex;

use crate::{
    metamath::mmp_parser::LocateAfterRef,
    model::{Comment, HeaderPath},
    util, AppState, Error,
};

#[tauri::command]
pub async fn get_comment(
    state: tauri::State<'_, Mutex<AppState>>,
    header_path: HeaderPath,
    comment_i: usize,
) -> Result<Comment, Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok(header_path
        .resolve_comment_path(comment_i, &mm_data.database_header)
        .ok_or(Error::NotFoundError)?
        .clone())
}

#[tauri::command]
pub async fn get_comment_mmp_format(
    state: tauri::State<'_, Mutex<AppState>>,
    header_path: HeaderPath,
    comment_i: usize,
) -> Result<String, Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let comment_path = format!("{}#{}", header_path.to_string(), comment_i + 1);

    util::locate_after_to_mmp_file_format_of_statement_it_refers_to(
        LocateAfterRef::LocateAfterComment(&comment_path),
        mm_data,
    )
}

#[tauri::command]
pub async fn write_comment_mmp_format_to_file(
    state: tauri::State<'_, Mutex<AppState>>,
    header_path: HeaderPath,
    comment_i: usize,
    file_path: &str,
) -> Result<(), Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let comment_path = format!("{}#{}", header_path.to_string(), comment_i + 1);

    let mmp_format = util::locate_after_to_mmp_file_format_of_statement_it_refers_to(
        LocateAfterRef::LocateAfterComment(&comment_path),
        mm_data,
    )?;

    fs::write(file_path, mmp_format).map_err(|_| Error::FileWriteError)?;

    Ok(())
}
