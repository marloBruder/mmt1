use super::Error;
use crate::{model::Variable, AppState};
use futures::TryStreamExt;
use sqlx::Row;
use tauri::{async_runtime::Mutex, State};

pub async fn get_variable(state: &State<'_, Mutex<AppState>>) -> Result<Vec<Variable>, Error> {
    let mut variables = Vec::new();

    let mut app_state = state.lock().await;

    if let Some(ref mut conn) = app_state.db_conn {
        let mut rows = sqlx::query(sql::VARIABLES_GET).fetch(conn);

        while let Some(row) = rows.try_next().await.or(Err(Error::SqlError))? {
            let symbol = row.try_get("symbol").or(Err(Error::SqlError))?;

            variables.push(Variable { symbol })
        }
    }

    Ok(variables)
}

mod sql {
    pub const VARIABLES_GET: &str = "\
SELECT * FROM variable
ORDER BY [index];    
";
}
