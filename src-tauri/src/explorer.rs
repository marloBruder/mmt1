use tauri::async_runtime::Mutex;

use crate::{
    database::header::{add_header_database, calc_db_index_for_header},
    local_state::header::add_header_local,
    model::HeaderPath,
    AppState, Error,
};

#[tauri::command]
pub async fn add_header(
    state: tauri::State<'_, Mutex<AppState>>,
    title: &str,
    insert_path: HeaderPath,
) -> Result<(), Error> {
    if insert_path.path.len() == 0 {
        return Err(Error::InvaildArgumentError);
    }

    let mut app_state = state.lock().await;
    let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

    add_header_local(&mut db_state.metamath_data, title, &insert_path)?;

    let db_index = calc_db_index_for_header(&db_state.metamath_data, &insert_path)?;
    let depth = (insert_path.path.len() as i32) - 1;

    add_header_database(&mut db_state.db_conn, db_index, depth, title).await?;

    Ok(())
}
