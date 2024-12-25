use super::{
    sql::{execute_query_no_bind, execute_query_two_binds},
    Error,
};
use crate::{model::Constant, AppState};
use futures::TryStreamExt;
use sqlx::Row;
use tauri::{async_runtime::Mutex, State};

pub async fn get_constants(state: &State<'_, Mutex<AppState>>) -> Result<Vec<Constant>, Error> {
    let mut constants = Vec::new();

    let mut app_state = state.lock().await;

    if let Some(ref mut conn) = app_state.db_conn {
        let mut rows = sqlx::query(sql::CONSTANTS_GET).fetch(conn);

        while let Some(row) = rows.try_next().await.or(Err(Error::SqlError))? {
            let symbol = row.try_get("symbol").or(Err(Error::SqlError))?;

            constants.push(Constant { symbol })
        }
    }

    Ok(constants)
}

pub async fn set_constants(
    state: &State<'_, Mutex<AppState>>,
    symbols: &Vec<&str>,
) -> Result<(), Error> {
    execute_query_no_bind(state, sql::CONSTANTS_DELETE).await?;

    for (index, &symbol) in symbols.iter().enumerate() {
        execute_query_two_binds(state, sql::CONSANT_ADD, index.to_string(), symbol).await?;
    }

    let mut app_state = state.lock().await;

    if let Some(ref mut mm_data) = app_state.metamath_data {
        mm_data.set_constants(symbols);
    }

    Ok(())
}

mod sql {
    pub const CONSTANTS_GET: &str = "\
SELECT * FROM constant
ORDER BY [index];";

    pub const CONSTANTS_DELETE: &str = "DELETE FROM constant;";

    pub const CONSANT_ADD: &str = "\
INSERT INTO constant ([index], symbol)
VALUES (?, ?);";
}
