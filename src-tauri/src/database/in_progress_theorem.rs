use futures::TryStreamExt;
use sqlx::Row;
use tauri::async_runtime::Mutex;

use super::{
    sql::{execute_query_one_bind, execute_query_two_bind},
    Error,
};
use crate::{model::InProgressTheorem, AppState};

pub async fn get_in_progress_theorems(
    state: &tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<InProgressTheorem>, Error> {
    let mut app_state = state.lock().await;

    let mut result = Vec::new();

    if let Some(ref mut conn) = app_state.db_conn {
        let mut rows = sqlx::query(sql::IN_PROGRESS_THEOREMS_GET).fetch(conn);

        while let Some(row) = rows.try_next().await.or(Err(Error::SqlError))? {
            let name: String = row.try_get("name").or(Err(Error::SqlError))?;
            let text: String = row.try_get("text").or(Err(Error::SqlError))?;

            result.push(InProgressTheorem { name, text });
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn add_in_progress_theorem(
    state: tauri::State<'_, Mutex<AppState>>,
    name: &str,
    text: &str,
) -> Result<(), Error> {
    execute_query_two_bind(&state, sql::IN_PROGRESS_THEOREM_ADD, name, text).await?;

    let mut app_state = state.lock().await;

    if let Some(ref mut mm_data) = app_state.metamath_data {
        mm_data.add_in_progress_theorem(name, text);
    }

    Ok(())
}

#[tauri::command]
pub async fn set_in_progress_theorem_name(
    state: tauri::State<'_, Mutex<AppState>>,
    old_name: &str,
    new_name: &str,
) -> Result<(), Error> {
    execute_query_two_bind(
        &state,
        sql::IN_PROGRESS_THEOREM_NAME_UPDATE,
        new_name,
        old_name,
    )
    .await?;

    let mut app_state = state.lock().await;

    if let Some(ref mut mm_data) = app_state.metamath_data {
        mm_data.set_in_progress_theorem_name(old_name, new_name);
    }

    Ok(())
}

#[tauri::command]
pub async fn set_in_progress_theorem(
    state: tauri::State<'_, Mutex<AppState>>,
    name: &str,
    text: &str,
) -> Result<(), Error> {
    execute_query_two_bind(&state, sql::IN_PROGRESS_THEOREM_UPDATE, text, name).await?;

    let mut app_state = state.lock().await;

    if let Some(ref mut mm_data) = app_state.metamath_data {
        mm_data.set_in_progress_theorem_text(name, text);
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_in_progress_theorem(
    state: tauri::State<'_, Mutex<AppState>>,
    name: &str,
) -> Result<(), Error> {
    execute_query_one_bind(&state, sql::IN_PROGRESS_THEOREM_DELETE, name).await?;

    let mut app_state = state.lock().await;

    if let Some(ref mut mm_data) = app_state.metamath_data {
        mm_data.delete_in_progress_theorem(name);
    }

    Ok(())
}

mod sql {

    pub const IN_PROGRESS_THEOREMS_GET: &str = "SELECT * FROM inProgressTheorem;";

    pub const IN_PROGRESS_THEOREM_ADD: &str =
        "INSERT INTO inProgressTheorem (name, text) VALUES (?, ?)";

    pub const IN_PROGRESS_THEOREM_NAME_UPDATE: &str = "UPDATE inProgressTheorem
      SET name = ?
      WHERE name = ?;";

    pub const IN_PROGRESS_THEOREM_UPDATE: &str = "UPDATE inProgressTheorem
        SET text = ?
        WHERE name = ?;";

    pub const IN_PROGRESS_THEOREM_DELETE: &str = "DELETE FROM inProgressTheorem WHERE name = ?";
}
