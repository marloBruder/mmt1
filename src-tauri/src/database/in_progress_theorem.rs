use futures::TryStreamExt;
use sqlx::Row;
use tauri::async_runtime::Mutex;

use super::Error;
use crate::AppState;

#[tauri::command]
pub async fn get_in_progress_theorems(
    state: tauri::State<'_, Mutex<AppState>>,
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
    let mut app_state = state.lock().await;

    if let Some(ref mut conn) = app_state.db_conn {
        sqlx::query(sql::IN_PROGRESS_THEOREM_ADD)
            .bind(name)
            .bind(text)
            .execute(conn)
            .await
            .or(Err(Error::SqlError))?;
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
        sqlx::query(sql::IN_PROGRESS_THEOREM_NAME_UPDATE)
            .bind(new_name)
            .bind(old_name)
            .execute(conn)
            .await
            .or(Err(Error::SqlError))?;
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
        sqlx::query(sql::IN_PROGRESS_THEOREM_UPDATE)
            .bind(text)
            .bind(name)
            .execute(conn)
            .await
            .or(Err(Error::SqlError))?;
    }

    Ok(())
}

pub struct InProgressTheorem {
    pub name: String,
    pub text: String,
}

impl serde::Serialize for InProgressTheorem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("InProgressTheorem", 2)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("text", &self.text)?;
        state.end()
    }
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
}
