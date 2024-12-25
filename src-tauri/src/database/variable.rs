use super::{
    sql::{execute_query_no_bind, execute_query_two_binds},
    Error,
};
use crate::{model::Variable, AppState};
use futures::TryStreamExt;
use sqlx::Row;
use tauri::{async_runtime::Mutex, State};

pub async fn get_variables(state: &State<'_, Mutex<AppState>>) -> Result<Vec<Variable>, Error> {
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

pub async fn set_variables(
    state: &State<'_, Mutex<AppState>>,
    symbols: &Vec<&str>,
) -> Result<(), Error> {
    execute_query_no_bind(state, sql::VARIABLES_DELETE).await?;

    for (index, &symbol) in symbols.iter().enumerate() {
        execute_query_two_binds(state, sql::VARIABLE_ADD, index.to_string(), symbol).await?;
    }

    let mut app_state = state.lock().await;

    if let Some(ref mut mm_data) = app_state.metamath_data {
        mm_data.set_variables(symbols);
    }

    Ok(())
}

mod sql {
    pub const VARIABLES_GET: &str = "\
SELECT * FROM variable
ORDER BY [index];";

    pub const VARIABLES_DELETE: &str = "DELETE FROM variable;";

    pub const VARIABLE_ADD: &str = "\
INSERT INTO variable ([index], symbol)
VALUES (?, ?);";
}
