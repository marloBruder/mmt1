use tauri::async_runtime::Mutex;

use crate::{
    database::{
        in_progress_theorem::{
            add_in_progress_theorem_database, delete_in_progress_theorem_database,
            set_in_progress_theorem_name_database, set_in_progress_theorem_text_database,
        },
        Error,
    },
    local_state::in_progress_theorem::{
        add_in_progress_theorem_local, delete_in_progress_theorem_local,
        set_in_progress_theorem_name_local, set_in_progress_theorem_text_local,
    },
    AppState,
};

#[tauri::command]
pub async fn add_in_progress_theorem(
    state: tauri::State<'_, Mutex<AppState>>,
    name: &str,
    text: &str,
) -> Result<(), Error> {
    let mut app_state = state.lock().await;
    let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

    add_in_progress_theorem_database(&mut db_state.db_conn, name, text).await?;

    add_in_progress_theorem_local(&mut db_state.metamath_data, name, text);

    Ok(())
}

#[tauri::command]
pub async fn set_in_progress_theorem_name(
    state: tauri::State<'_, Mutex<AppState>>,
    old_name: &str,
    new_name: &str,
) -> Result<(), Error> {
    let mut app_state = state.lock().await;
    let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

    set_in_progress_theorem_name_database(&mut db_state.db_conn, old_name, new_name).await?;

    set_in_progress_theorem_name_local(&mut db_state.metamath_data, old_name, new_name);

    Ok(())
}

#[tauri::command]
pub async fn set_in_progress_theorem(
    state: tauri::State<'_, Mutex<AppState>>,
    name: &str,
    text: &str,
) -> Result<(), Error> {
    let mut app_state = state.lock().await;
    let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

    set_in_progress_theorem_text_database(&mut db_state.db_conn, name, text).await?;

    set_in_progress_theorem_text_local(&mut db_state.metamath_data, name, text);

    Ok(())
}

#[tauri::command]
pub async fn delete_in_progress_theorem(
    state: tauri::State<'_, Mutex<AppState>>,
    name: &str,
) -> Result<(), Error> {
    let mut app_state = state.lock().await;
    let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

    delete_in_progress_theorem_database(&mut db_state.db_conn, name).await?;

    delete_in_progress_theorem_local(&mut db_state.metamath_data, name);

    Ok(())
}
