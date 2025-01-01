use super::Error;
use crate::model::Variable;
use futures::TryStreamExt;
use sqlx::{Row, SqliteConnection};

pub async fn get_variables_database(conn: &mut SqliteConnection) -> Result<Vec<Variable>, Error> {
    let mut variables = Vec::new();

    let mut rows = sqlx::query(sql::VARIABLES_GET).fetch(conn);

    while let Some(row) = rows.try_next().await.or(Err(Error::SqlError))? {
        let symbol = row.try_get("symbol").or(Err(Error::SqlError))?;

        variables.push(Variable { symbol })
    }

    Ok(variables)
}

pub async fn set_variables_database(
    conn: &mut SqliteConnection,
    symbols: &Vec<&str>,
) -> Result<(), Error> {
    sqlx::query(sql::VARIABLES_DELETE)
        .execute(&mut *conn)
        .await
        .or(Err(Error::SqlError))?;

    for (index, &symbol) in symbols.iter().enumerate() {
        sqlx::query(sql::VARIABLE_ADD)
            .bind(index.to_string())
            .bind(symbol)
            .execute(&mut *conn)
            .await
            .or(Err(Error::SqlError))?;
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
