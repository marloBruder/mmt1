use super::Error;
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
            // while let Some(row) = rows.try_next().await.map_err(|err| {
            //     println!("Hello1");
            //     println!("{:?}", err);
            //     Error::SqlError
            // })? {
            let symbol = row.try_get("symbol").or(Err(Error::SqlError))?;
            // let symbol = row.try_get("symbol").map_err(|err| {
            //     println!("Hello2");
            //     println!("{:?}", err);
            //     Error::SqlError
            // })?;

            constants.push(Constant { symbol })
        }
    }

    Ok(constants)
}

mod sql {
    pub const CONSTANTS_GET: &str = "\
SELECT * FROM constant
ORDER BY [index];    
";
}
