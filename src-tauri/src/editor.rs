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

    if let Some(ref mut conn) = app_state.db_conn {
        add_in_progress_theorem_database(conn, name, text).await?;
    }

    if let Some(ref mut mm_data) = app_state.metamath_data {
        add_in_progress_theorem_local(mm_data, name, text);
    }

    Ok(())
}

#[tauri::command]
pub async fn set_in_progress_theorem_name(
    state: tauri::State<'_, Mutex<AppState>>,
    old_name: &str,
    new_name: &str,
) -> Result<(), Error> {
    let mut app_state = state.lock().await;

    if let Some(ref mut conn) = app_state.db_conn {
        set_in_progress_theorem_name_database(conn, old_name, new_name).await?;
    }

    if let Some(ref mut mm_data) = app_state.metamath_data {
        set_in_progress_theorem_name_local(mm_data, old_name, new_name);
    }

    Ok(())
}

#[tauri::command]
pub async fn set_in_progress_theorem(
    state: tauri::State<'_, Mutex<AppState>>,
    name: &str,
    text: &str,
) -> Result<(), Error> {
    let mut app_state = state.lock().await;

    if let Some(ref mut conn) = app_state.db_conn {
        set_in_progress_theorem_text_database(conn, name, text).await?;
    }

    if let Some(ref mut mm_data) = app_state.metamath_data {
        set_in_progress_theorem_text_local(mm_data, name, text);
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_in_progress_theorem(
    state: tauri::State<'_, Mutex<AppState>>,
    name: &str,
) -> Result<(), Error> {
    let mut app_state = state.lock().await;

    if let Some(ref mut conn) = app_state.db_conn {
        delete_in_progress_theorem_database(conn, name).await?;
    }

    if let Some(ref mut mm_data) = app_state.metamath_data {
        delete_in_progress_theorem_local(mm_data, name);
    }

    Ok(())
}
