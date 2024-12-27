use super::{
    sql::{execute_query_four_binds, execute_query_no_bind},
    Error,
};
use crate::{model::FloatingHypohesis, AppState};
use futures::TryStreamExt;
use sqlx::Row;
use tauri::{async_runtime::Mutex, State};

pub async fn get_floating_hypotheses(
    state: &State<'_, Mutex<AppState>>,
) -> Result<Vec<FloatingHypohesis>, Error> {
    let mut floating_hypotheses = Vec::new();

    let mut app_state = state.lock().await;

    if let Some(ref mut conn) = app_state.db_conn {
        let mut rows = sqlx::query(sql::FLOATING_HYPOTHESES_GET).fetch(conn);

        while let Some(row) = rows.try_next().await.or(Err(Error::SqlError))? {
            let label = row.try_get("label").or(Err(Error::SqlError))?;
            let typecode = row.try_get("typecode").or(Err(Error::SqlError))?;
            let variable = row.try_get("variable").or(Err(Error::SqlError))?;

            floating_hypotheses.push(FloatingHypohesis {
                label,
                typecode,
                variable,
            });
        }
    }

    Ok(floating_hypotheses)
}

pub async fn set_floating_hypotheses(
    state: &State<'_, Mutex<AppState>>,
    floating_hypotheses: &Vec<FloatingHypohesis>,
) -> Result<(), Error> {
    execute_query_no_bind(state, sql::FLOATING_HYPOTHESES_DELETE).await?;

    for (index, floating_hypothesis) in floating_hypotheses.iter().enumerate() {
        execute_query_four_binds(
            state,
            &sql::FLOATING_HYPOTHESIS_ADD,
            index.to_string(),
            &floating_hypothesis.label,
            &floating_hypothesis.typecode,
            &floating_hypothesis.variable,
        )
        .await?;
    }

    let mut app_state = state.lock().await;

    if let Some(ref mut mm_data) = app_state.metamath_data {
        mm_data.set_floating_hypotheses(floating_hypotheses);
    }

    Ok(())
}

mod sql {
    pub const FLOATING_HYPOTHESES_GET: &str = "\
SELECT * FROM floating_hypothesis
ORDER BY [index];";

    pub const FLOATING_HYPOTHESES_DELETE: &str = "DELETE FROM floating_hypothesis;";

    pub const FLOATING_HYPOTHESIS_ADD: &str = "\
INSERT INTO floating_hypothesis ([index], label, typecode, variable)
VALUES (?, ?, ?, ?);";
}
