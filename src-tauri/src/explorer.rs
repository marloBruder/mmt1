use tauri::async_runtime::Mutex;

use crate::{
    database::header::{add_header_database, calc_db_index_for_header},
    local_state::header::add_header_local,
    metamath,
    model::HeaderPath,
    AppState,
};

#[tauri::command]
pub async fn add_header(
    state: tauri::State<'_, Mutex<AppState>>,
    title: &str,
    insert_path: HeaderPath,
) -> Result<(), metamath::Error> {
    if insert_path.path.len() == 0 {
        return Err(metamath::Error::InvaildArgumentError);
    }

    let mut app_state = state.lock().await;

    if let Some(ref mut mm_data) = app_state.metamath_data {
        add_header_local(mm_data, title, &insert_path)?;

        let db_index = calc_db_index_for_header(mm_data, &insert_path)?;
        let depth = (insert_path.path.len() as i32) - 1;

        if let Some(ref mut conn) = app_state.db_conn {
            add_header_database(conn, db_index, depth, title).await?;
        }
    }

    Ok(())
}
