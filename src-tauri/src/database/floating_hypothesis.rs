use super::Error;
use crate::model::FloatingHypohesis;
use futures::TryStreamExt;
use sqlx::{Row, SqliteConnection};

pub async fn get_floating_hypotheses_database(
    conn: &mut SqliteConnection,
) -> Result<Vec<FloatingHypohesis>, Error> {
    let mut floating_hypotheses = Vec::new();
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

    Ok(floating_hypotheses)
}

pub async fn set_floating_hypotheses_database(
    conn: &mut SqliteConnection,
    floating_hypotheses: &Vec<FloatingHypohesis>,
) -> Result<(), Error> {
    sqlx::query(sql::FLOATING_HYPOTHESES_DELETE)
        .execute(&mut *conn)
        .await
        .or(Err(Error::SqlError))?;

    for (index, floating_hypothesis) in floating_hypotheses.iter().enumerate() {
        sqlx::query(sql::FLOATING_HYPOTHESIS_ADD)
            .bind(index.to_string())
            .bind(&floating_hypothesis.label)
            .bind(&floating_hypothesis.typecode)
            .bind(&floating_hypothesis.variable)
            .execute(&mut *conn)
            .await
            .or(Err(Error::SqlError))?;
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
